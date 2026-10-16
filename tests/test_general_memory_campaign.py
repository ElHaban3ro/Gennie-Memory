"""Synthetic cohort separation and general-memory typed-choice contracts."""

import tempfile
import json
import unittest
from pathlib import Path

from scripts.general_memory_prompts import choice
from scripts.prepare_general_memory_v1 import prepare
from scripts.prepare_general_memory_fallback import prepare as prepare_fallback
from scripts.summarize_general_memory_followup import merged_decisions, summarize


class GeneralMemoryCampaignTests(unittest.TestCase):
    def test_entity_disjoint_cohorts_and_label_counts(self) -> None:
        fixture = Path(__file__).parent / "fixtures/general-memory-v1"
        with tempfile.TemporaryDirectory() as directory:
            manifest = prepare(fixture, Path(directory))
        self.assertEqual(manifest["counts"]["development/positive"], 115)
        self.assertEqual(manifest["counts"]["final/positive"], 460)
        self.assertEqual(manifest["counts"]["final/negative"], 80)
        self.assertEqual(sum(manifest["counts"].values()), 3300)

    def test_choice_maps_short_labels_and_keeps_none(self) -> None:
        candidates = [{"memory_id": "memory_a", "summary": "Fictitious name is Ada."},
                      {"memory_id": "memory_b", "summary": "Fictitious city is Sol."}]
        for variant in ("direct", "grounded", "spanish", "relation", "explicit", "relation_context"):
            question, mapping = choice(candidates, variant)
            self.assertEqual(mapping, {"A": "memory_a", "B": "memory_b", "none": "none"})
            self.assertEqual(set(question["criteria"]), set(mapping))
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            choice(candidates + candidates[:1], "direct")
        enriched, _ = choice([{**candidates[0], "conditions": "Before publishing"}],
                             "relation_context")
        self.assertIn("Before publishing", enriched["criteria"]["A"])

    def test_fallback_only_calls_jev_for_abstentions_and_accounts_for_negatives(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cases = [
                {"id": "positive", "split": "calibration", "expected_ids": ["target"],
                 "request": {"scope_kind": "conversation"}},
                {"id": "negative", "split": "calibration", "expected_ids": [],
                 "request": {"scope_kind": "project"}},
            ]
            screens = [{"id": row["id"], "expected_ids": row["expected_ids"],
                        "response": {"candidates": [{"memory_id": "target", "summary": "Fact"}]}}
                       for row in cases]
            queries, results = root / "queries.jsonl", root / "results.jsonl"
            queries.write_text("".join(json.dumps(row) + "\n" for row in cases))
            results.write_text("".join(json.dumps(row) + "\n" for row in screens))
            primary = root / "primary.json"
            primary.write_text(json.dumps({"incomplete_tasks": 0, "uncertain_tasks": 0, "results": [
                {"case_id": row["id"], "requested_model": "openai/gpt-6-luna-decisions",
                 "served_model": "openai/gpt-6-luna-decisions",
                 "status": "complete", "chosen_id": "none", "expected_ids": row["expected_ids"],
                 "candidate_ids": ["target"], "elapsed_ms": 100,
                 "reported_cost_usd": .01} for row in cases]}))
            manifest = prepare_fallback(queries, results, primary, root / "fallback")
            self.assertEqual((manifest["positive"], manifest["negative"]), (1, 1))
            jev = root / "jev.json"
            jev.write_text(json.dumps({"incomplete_tasks": 0, "uncertain_tasks": 0,
                                       "max_candidates": 20, "results": [
                {"case_id": row["id"], "requested_model": "typesafe/jev-1.13",
                 "served_model": "typesafe/jev-1.13",
                 "status": "complete", "chosen_id": "target", "expected_ids": row["expected_ids"],
                 "candidate_ids": ["target"], "elapsed_ms": 120,
                 "reported_cost_usd": .01} for row in cases]}))
            summary = summarize(queries, results, primary, jev)
            self.assertEqual(summary["groups"]["cascade"]["positive_correct"], 1)
            self.assertEqual(summary["groups"]["cascade"]["negative_empty"], 0)
            self.assertEqual(summary["groups"]["cascade"]["decision_p50_ms"], 220)
            merged = merged_decisions(primary, jev)
            self.assertEqual(len(merged["results"]), 2)
            self.assertTrue(all(row["chosen_id"] == "target" for row in merged["results"]))


if __name__ == "__main__":
    unittest.main()
