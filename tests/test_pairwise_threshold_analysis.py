"""Threshold analysis must account for abstention on negatives and positives."""

import unittest

from scripts.analyze_pairwise_threshold import analyze


class PairwiseThresholdAnalysisTests(unittest.TestCase):
    def test_finds_exact_score_breakpoint_without_reclassifying(self):
        queries = [{"id": "positive", "split": "development", "expected_ids": ["wanted"]},
                   {"id": "negative", "split": "development", "expected_ids": []}]
        report = {"split": "development",
                  "method": "integrated_hybrid_laya_pairwise_conditions_seed_campaign",
                  "individual_results": [
                      {"id": "positive", "candidate_scores": {"wanted": .6}},
                      {"id": "negative", "candidate_scores": {"wrong": .4}},
                  ]}
        result = analyze(queries, report)
        self.assertEqual(result["correct_positive_top_rank"], 1)
        self.assertEqual(result["best_over_all_score_breakpoints"]["total_exact"], 2)
        self.assertEqual(result["fixed_thresholds"][2],
                         {"threshold": .5, "positive_exact": 1,
                          "negative_empty": 1, "total_exact": 2})
        self.assertFalse(result["production_threshold_selected"])
        for case in queries:
            case["split"] = "calibration"
        report["split"] = "calibration"
        self.assertEqual(analyze(queries, report)["kind"],
                         "exploratory_calibration_threshold_analysis")
        for case in queries:
            case["split"] = "final"
        report["split"] = "final"
        with self.assertRaisesRegex(ValueError, "development or calibration"):
            analyze(queries, report)


if __name__ == "__main__":
    unittest.main()
