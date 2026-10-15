"""Frozen synthetic campaign inputs and journal recovery without provider calls."""

import argparse
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import run_openrouter_campaign as campaign


class CampaignTest(unittest.TestCase):
    def fixture(self, directory: Path) -> argparse.Namespace:
        queries = directory / "queries.jsonl"
        results = directory / "results.jsonl"
        cases = [
            {"id": "positive", "split": "development", "segment": "direct",
             "label_source": "deterministic_fixture_rule", "expected_ids": ["mem_1"],
             "forbidden_ids": [], "request": {"query": "Synthetic preference?",
                                                "scope_id": "synthetic_project"}},
            {"id": "negative", "split": "development", "segment": "wrong_task",
             "label_source": "deterministic_fixture_rule", "expected_ids": [],
             "forbidden_ids": [], "request": {"query": "Synthetic unknown?",
                                                "scope_id": "synthetic_project"}},
        ]
        outputs = [
            {key: case[key] for key in ("id", "split", "segment", "label_source",
                                        "expected_ids", "forbidden_ids")}
            | {"response": {"status": "complete", "method": "fts5",
                            "candidates": [{"memory_id": "mem_1", "summary": "Synthetic fact"}]}}
            for case in cases
        ]
        queries.write_text("".join(json.dumps(row) + "\n" for row in cases))
        results.write_text("".join(json.dumps(row) + "\n" for row in outputs))
        return argparse.Namespace(
            queries=queries, results=results, split="development",
            model=["typesafe/jev-1.13"], max_candidates=4, max_calls=2,
            max_cost_usd=.01, reserve_per_call_usd=.001, max_seconds=10,
        )

    def test_freeze_run_resume_and_export(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = self.fixture(root)
            plan = campaign.prepare(args)
            self.assertEqual(plan["planned_calls"], 2)
            journal = root / "journal.jsonl"
            calls = []

            def fake_call(_key, model, state, criteria):
                calls.append(model)
                choice = "mem_1" if "preference" in state["user_prompt"] else "none"
                return {"requested_model": model, "served_model": model,
                        "chosen_id": choice, "reported_cost_usd": .0002,
                        "probabilities": {key: 1.0 if key == choice else 0.0
                                          for key in criteria}}

            with patch.object(campaign.probe, "read_key", return_value="fake"):
                first = campaign.run(plan, journal, call=fake_call)
                second = campaign.run(plan, journal, call=fake_call)
            self.assertEqual(len(calls), 2)
            self.assertEqual(first["finished_tasks"], 2)
            self.assertEqual(second["attempted_calls"], 2)
            report = campaign.export_report(plan, journal)
            self.assertEqual([row["correct"] for row in report["results"]], [True, True])
            self.assertEqual(report["calls"], 2)

    def test_mismatch_and_uncertain_call_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = self.fixture(root)
            rows = [json.loads(line) for line in args.results.read_text().splitlines()]
            rows[0]["expected_ids"] = ["wrong"]
            args.results.write_text("".join(json.dumps(row) + "\n" for row in rows))
            with self.assertRaisesRegex(ValueError, "label mismatch"):
                campaign.prepare(args)
            args = self.fixture(root)
            args.max_cost_usd = .001
            with self.assertRaisesRegex(ValueError, "Reserved call cost"):
                campaign.prepare(args)
            args.max_cost_usd = .01
            plan = campaign.prepare(args)
            journal = root / "journal.jsonl"
            campaign.append_event(journal, {
                "plan_sha256": plan["plan_sha256"], "model": plan["models"][0],
                "case_id": "negative", "phase": "started",
            })
            with self.assertRaisesRegex(ValueError, "Uncertain in-flight"):
                campaign.run(plan, journal, call=lambda *_: self.fail("Should not call"))
            with self.assertRaisesRegex(ValueError, "incomplete"):
                campaign.export_report(plan, journal)
            partial = campaign.export_report(plan, journal, allow_partial=True)
            self.assertEqual((partial["cases"], partial["incomplete_tasks"],
                              partial["uncertain_tasks"]), (0, 1, 1))
            campaign.resolve_uncertain(plan, journal, plan["models"][0], "negative")
            self.assertEqual(campaign.load_journal(journal, plan)[(plan["models"][0], "negative")]["phase"],
                             "finished")

    def test_provider_failure_stops_batch_without_retry(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plan = campaign.prepare(self.fixture(root))
            journal = root / "journal.jsonl"
            with patch.object(campaign.probe, "read_key", return_value="fake"):
                first = campaign.run(plan, journal,
                                     call=lambda *_: (_ for _ in ()).throw(RuntimeError("HTTP 429")))
                self.assertEqual(first["attempted_calls"], 1)

                def recovered(_key, model, _state, criteria):
                    return {"requested_model": model, "served_model": model,
                            "chosen_id": "mem_1", "reported_cost_usd": .0002,
                            "probabilities": {key: 1.0 if key == "mem_1" else 0.0
                                              for key in criteria}}

                second = campaign.run(plan, journal, call=recovered)
            self.assertEqual(second["attempted_calls"], 2)
            rows = campaign.export_report(plan, journal)["results"]
            self.assertEqual([row["status"] for row in rows], ["failed", "complete"])


if __name__ == "__main__":
    unittest.main()
