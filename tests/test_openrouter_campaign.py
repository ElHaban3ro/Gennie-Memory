"""Frozen synthetic campaign inputs and journal recovery without provider calls."""

import argparse
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import run_openrouter_campaign as campaign
from scripts import classifier_policy


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

    def test_uncapped_research_still_reports_cost(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = self.fixture(root)
            args.max_cost_usd = None
            plan = campaign.prepare(args)
            self.assertIsNone(plan["max_cost_usd"])

            def fake_call(_key, model, _state, criteria):
                return {"requested_model": model, "served_model": model,
                        "chosen_id": "none", "reported_cost_usd": 10.0,
                        "probabilities": {key: 1.0 if key == "none" else 0.0
                                          for key in criteria}}

            with patch.object(campaign.probe, "read_key", return_value="fake"):
                result = campaign.run(plan, root / "journal.jsonl", call=fake_call)
            self.assertEqual(result["attempted_calls"], 2)
            self.assertEqual(result["reported_or_reserved_cost_usd"], 20.0)

    def test_default_profile_uses_decisions_and_explicit_secondary_uses_jev(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            args = self.fixture(Path(temp))
            args.model = None
            args.max_candidates = None
            args.prompt_variant = None
            primary = campaign.prepare(args)
            self.assertEqual(primary["models"], ["openai/gpt-6-luna-decisions"])
            self.assertEqual((primary["max_candidates"], primary["prompt_variant"]),
                             (20, "direct"))
            self.assertIn("prompt_sha256", primary)
            seen = []

            def fake_call(_key, model, _state, criteria, *, instructions):
                seen.append((model, set(criteria), instructions))
                return {"requested_model": model, "served_model": model,
                        "chosen_id": "none", "reported_cost_usd": .0001}

            with patch.object(campaign.probe, "read_key", return_value="fake"):
                campaign.run(primary, Path(temp) / "primary-journal.jsonl", call=fake_call)
            self.assertEqual(len(seen), 2)
            self.assertTrue(all(model == "openai/gpt-6-luna-decisions" and
                                "none" in options and "directly answers" in instructions
                                for model, options, instructions in seen))
            args.provider = "secondary"
            secondary = campaign.prepare(args)
            self.assertEqual(secondary["models"], ["typesafe/jev-1.13"])
            self.assertNotEqual(primary["plan_sha256"], secondary["plan_sha256"])

    def test_profile_disables_automatic_fallback_and_can_disable_secondary(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = self.fixture(root)
            args.model = None
            args.max_candidates = None
            profile = json.loads(classifier_policy.DEFAULT_PROFILE.read_text())
            profile["automatic_fallback"] = True
            path = root / "profile.json"
            path.write_text(json.dumps(profile))
            args.profile = path
            with self.assertRaisesRegex(ValueError, "Unsupported classifier profile"):
                campaign.prepare(args)
            profile["automatic_fallback"] = False
            profile["secondary_enabled"] = False
            path.write_text(json.dumps(profile))
            args.provider = "secondary"
            with self.assertRaisesRegex(ValueError, "Secondary classifier is disabled"):
                campaign.prepare(args)

    def test_retry_only_selected_terminal_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = self.fixture(root)
            plan = campaign.prepare(args)
            journal = root / "journal.jsonl"

            def failed(_key, model, _state, _criteria):
                raise RuntimeError("synthetic transport failure")

            with patch.object(campaign.probe, "read_key", return_value="fake"):
                campaign.run(plan, journal, call=failed)
                campaign.run(plan, journal, call=failed)

                def success(_key, model, _state, criteria):
                    return {"requested_model": model, "served_model": model,
                            "chosen_id": "none", "reported_cost_usd": .0002,
                            "probabilities": {key: 1.0 if key == "none" else 0.0
                                              for key in criteria}}

                campaign.run(plan, journal, call=success, retry_failed=True,
                             retry_only=("typesafe/jev-1.13", "positive"))
            state = campaign.load_journal(journal, plan)
            self.assertEqual(state[("typesafe/jev-1.13", "positive")]["result"]["status"], "complete")
            self.assertEqual(state[("typesafe/jev-1.13", "negative")]["result"]["status"], "failed")
            with self.assertRaisesRegex(ValueError, "no terminal failure"):
                campaign.run(plan, journal, call=success, retry_failed=True,
                             retry_only=("typesafe/jev-1.13", "positive"))

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

    def test_twenty_structured_options_map_back_to_memory_ids(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = self.fixture(root)
            args.prompt_variant = "structured"
            args.max_candidates = 20
            args.max_cost_usd = .1
            args.reserve_per_call_usd = .001
            cases = [json.loads(line) for line in args.results.read_text().splitlines()]
            candidates = [{"memory_id": f"mem_{index}", "summary": f"Synthetic fact {index}",
                           "conditions": "Synthetic condition"} for index in range(20)]
            cases[0]["response"]["candidates"] = candidates
            args.results.write_text("".join(json.dumps(row) + "\n" for row in cases))
            plan = campaign.prepare(args)
            seen = []

            def fake_call(_key, model, state, criteria, *, instructions):
                seen.append((len(criteria), instructions, state["user_prompt"]))
                label = "A"
                return {"requested_model": model, "served_model": model,
                        "chosen_id": label, "reported_cost_usd": .0002,
                        "probabilities": {key: 1.0 if key == label else 0.0 for key in criteria}}

            with patch.object(campaign.probe, "read_key", return_value="fake"):
                campaign.run(plan, root / "journal.jsonl", call=fake_call)
            report = campaign.export_report(plan, root / "journal.jsonl")
            positive = next(row for row in report["results"] if row["case_id"] == "positive")
            self.assertEqual(positive["chosen_id"], "mem_0")
            self.assertTrue(any(count == 21 and "meeting modality" in instructions
                                for count, instructions, _prompt in seen))

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

    def test_explicit_retry_of_known_failure_keeps_audit_events(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plan = campaign.prepare(self.fixture(root))
            journal = root / "journal.jsonl"
            with patch.object(campaign.probe, "read_key", return_value="fake"):
                first = campaign.run(plan, journal,
                                     call=lambda *_: (_ for _ in ()).throw(RuntimeError("HTTP 503")))
                self.assertEqual(first["failed_tasks"], 1)

                def recovered(_key, model, state, criteria):
                    choice = "mem_1" if "preference" in state["user_prompt"] else "none"
                    return {"requested_model": model, "served_model": model,
                            "chosen_id": choice, "reported_cost_usd": .0002,
                            "probabilities": {key: 1.0 if key == choice else 0.0
                                              for key in criteria}}

                second = campaign.run(plan, journal, call=recovered, retry_failed=True)
            self.assertEqual(second["attempted_calls"], 3)
            self.assertEqual(second["failed_tasks"], 0)
            self.assertEqual(len(journal.read_text().splitlines()), 6)
            self.assertTrue(all(row["correct"] for row in
                                campaign.export_report(plan, journal)["results"]))


if __name__ == "__main__":
    unittest.main()
