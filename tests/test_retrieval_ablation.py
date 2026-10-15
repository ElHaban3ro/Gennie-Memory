"""Retrieval ablations must compare complete runs on the same labeled cases."""

import json
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

from scripts.compare_retrieval_ablation import compare


class RetrievalAblationTests(unittest.TestCase):
    def test_detects_candidate_and_choice_difference(self):
        case = {"id": "synthetic_1", "split": "development", "segment": "direct",
                "label_source": "deterministic_fixture_rule", "expected_ids": ["wanted"]}
        candidate = {"memory_id": "wanted", "revision": 1, "state": "active",
                     "summary": "synthetic", "conditions": None, "source": None}
        row = {"id": case["id"], "segment": "direct", "positive": True,
               "decision_status": "complete", "seed_status": "complete",
               "forbidden_leak": False, "semantic_coverage": "disabled",
               "retrieval_snapshot": {"candidates": [candidate]},
               "chosen_id": "wanted", "exact": True}
        fts = {"method": "integrated_fts5_top1_seed_campaign", "split": "development",
               "cases": 1, "decision_failures": 0, "degraded_seeds": 0,
               "forbidden_leaks": 0, "exact_selection": 1, "positive_fact_fidelity": 1,
               "negative_empty": 0, "candidate_coverage": 1,
               "latency_ms": {"total_ms": {"p50": 5, "p95": 5}},
               "individual_results": [row]}
        hybrid = deepcopy(fts)
        hybrid["method"] = "integrated_hybrid_top1_seed_campaign"
        hybrid["individual_results"][0]["semantic_coverage"] = "complete"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            queries, fts_path, hybrid_path = (root / name for name in
                                               ("queries.jsonl", "fts.json", "hybrid.json"))
            queries.write_text(json.dumps(case) + "\n")
            fts_path.write_text(json.dumps(fts))
            hybrid_path.write_text(json.dumps(hybrid))
            self.assertEqual(compare(queries, fts_path, hybrid_path)["different_candidate_order"], 0)
            hybrid["individual_results"][0]["retrieval_snapshot"]["candidates"][0]["memory_id"] = "other"
            hybrid["individual_results"][0]["chosen_id"] = "other"
            hybrid["individual_results"][0]["exact"] = False
            hybrid_path.write_text(json.dumps(hybrid))
            result = compare(queries, fts_path, hybrid_path)
            self.assertEqual((result["different_candidate_order"], result["different_choice"],
                              result["different_exact"]), (1, 1, 1))
            case["split"] = "final"
            queries.write_text(json.dumps(case) + "\n")
            with self.assertRaisesRegex(ValueError, "development or calibration"):
                compare(queries, fts_path, hybrid_path)


if __name__ == "__main__":
    unittest.main()
