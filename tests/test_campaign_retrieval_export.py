"""A remote plan must reuse only complete, authorized local candidates."""

import argparse
import json
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

from scripts.export_campaign_retrieval import export_rows
from scripts.run_openrouter_campaign import prepare


class CampaignRetrievalExportTests(unittest.TestCase):
    def setUp(self):
        self.case = {
            "id": "q5_0002_0000", "split": "development", "segment": "direct",
            "expected_ids": ["wanted"], "forbidden_ids": ["other_scope"],
            "label_source": "deterministic_fixture_rule",
            "expected_condition": "remote only",
            "expected_source": {"kind": "document", "ref": "synthetic:guide"},
            "request": {"query": "What is the remote rule?", "scope_id": "synthetic"},
        }
        self.candidates = [
            {"memory_id": "distractor", "revision": 1, "summary": "local meeting"},
            {"memory_id": "wanted", "revision": 2, "summary": "remote meeting"},
        ]
        self.report = {
            "method": "integrated_hybrid_top1_seed_campaign", "split": "development",
            "cases": 1, "embedding_profile": {"space_id": "synthetic", "dimensions": 2},
            "individual_results": [{
                "id": self.case["id"], "segment": "direct", "positive": True,
                "retrieval_method": "fts5_semantic_rrf", "candidate_count": 2,
                "search_rust_ms": 3.0,
                "retrieval_request": {"search": deepcopy(self.case["request"]), "space_id": "synthetic",
                                      "query_vector": [1.0, 0.0]},
                "retrieval_snapshot": {"status": "complete", "method": "fts5_semantic_rrf",
                                       "candidates": self.candidates},
            }],
        }

    def test_preserves_order_and_labels_for_remote_plan(self):
        rows = export_rows([self.case], self.report)
        self.assertEqual([item["memory_id"] for item in rows[0]["response"]["candidates"]],
                         ["distractor", "wanted"])
        self.assertEqual(rows[0]["expected_condition"], "remote only")
        self.assertEqual(rows[0]["hybrid_request"]["query_vector"], [1.0, 0.0])

    def test_rejects_forbidden_candidate(self):
        self.candidates[0]["memory_id"] = "other_scope"
        with self.assertRaisesRegex(ValueError, "Invalid candidate list"):
            export_rows([self.case], self.report)

    def test_rejects_incomplete_report(self):
        self.report["cases"] = 2
        with self.assertRaisesRegex(ValueError, "complete"):
            export_rows([self.case], self.report)

    def test_calibration_export_and_final_rejection(self):
        self.case["split"] = self.report["split"] = "calibration"
        self.assertEqual(export_rows([self.case], self.report)[0]["split"], "calibration")
        self.case["split"] = self.report["split"] = "final"
        with self.assertRaisesRegex(ValueError, "complete synthetic"):
            export_rows([self.case], self.report)

    def test_rejects_hybrid_vector_for_another_query(self):
        self.report["individual_results"][0]["retrieval_request"]["search"]["query"] = "changed"
        with self.assertRaisesRegex(ValueError, "Invalid hybrid request"):
            export_rows([self.case], self.report)

    def test_export_feeds_remote_plan_without_new_retrieval(self):
        rows = export_rows([self.case], self.report)
        with tempfile.TemporaryDirectory() as directory:
            queries = Path(directory) / "queries.jsonl"
            results = Path(directory) / "results.jsonl"
            queries.write_text(json.dumps(self.case) + "\n")
            results.write_text(json.dumps(rows[0]) + "\n")
            args = argparse.Namespace(queries=queries, results=results, split="development",
                                      model=["typesafe/jev-1.13"], max_candidates=4,
                                      max_calls=1, reserve_per_call_usd=.0002,
                                      max_cost_usd=.001, max_seconds=60)
            plan = prepare(args)
        self.assertEqual(plan["planned_calls"], 1)
        self.assertEqual([item["memory_id"] for item in plan["cases"][0]["candidates"]],
                         ["distractor", "wanted"])


if __name__ == "__main__":
    unittest.main()
