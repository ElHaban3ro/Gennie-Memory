"""Paired campaign comparisons must reject different retrieval candidates."""

import json
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

from scripts.summarize_local_campaign import summarize


class LocalCampaignSummaryTests(unittest.TestCase):
    def test_requires_identical_candidates_for_paired_comparison(self):
        case = {"id": "q5_0002_0000", "split": "development", "segment": "direct",
                "label_source": "deterministic_fixture_rule", "expected_ids": ["wanted"]}
        row = {"id": case["id"], "segment": "direct", "positive": True,
               "semantic_coverage": "complete", "decision_status": "complete",
               "seed_status": "complete", "forbidden_leak": False,
               "retrieval_snapshot": {"method": "fts5_semantic_rrf", "status": "complete",
                                      "candidates": [{"memory_id": "wanted"}]},
               "exact": True, "chosen_id": "wanted", "expected_in_candidates": True,
               "fact_fidelity": True, "condition_fidelity": True, "source_fidelity": True,
               "total_ms": 12.0}
        report = {"method": "integrated_hybrid_top1_seed_campaign", "split": "development",
                  "retrieval": "hybrid", "device": "mps", "cases": 1,
                  "individual_results": [row], "input_tokens": 0,
                  "load_seconds": 1.0, "warmup_ms": 1.0,
                  "latency_ms": {"total_ms": {"p50": 12.0, "p95": 12.0}}}
        other = deepcopy(report)
        other["method"] = "integrated_hybrid_laya_seed_campaign"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            queries = root / "queries.jsonl"
            first, second = root / "first.json", root / "second.json"
            queries.write_text(json.dumps(case) + "\n")
            first.write_text(json.dumps(report))
            second.write_text(json.dumps(other))
            result = summarize([first, second], queries)
            pair = next(iter(result["paired"].values()))
            self.assertEqual(pair, {"both": 1, "left_only": 0,
                                    "right_only": 0, "neither": 0})
            other["individual_results"][0]["retrieval_snapshot"]["candidates"][0]["memory_id"] = "different"
            second.write_text(json.dumps(other))
            with self.assertRaisesRegex(ValueError, "candidates differ"):
                summarize([first, second], queries)

            other["individual_results"][0]["retrieval_snapshot"] = deepcopy(
                report["individual_results"][0]["retrieval_snapshot"])
            case["split"] = "calibration"
            report["split"] = other["split"] = "calibration"
            queries.write_text(json.dumps(case) + "\n")
            first.write_text(json.dumps(report))
            second.write_text(json.dumps(other))
            self.assertEqual(summarize([first, second], queries)["kind"],
                             "v5_calibration_local_seed_campaign")
            case["split"] = "final"
            queries.write_text(json.dumps(case) + "\n")
            with self.assertRaisesRegex(ValueError, "development or calibration"):
                summarize([first, second], queries)


if __name__ == "__main__":
    unittest.main()
