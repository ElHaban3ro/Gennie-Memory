"""The no-classifier baseline uses only the first presented retrieval candidate."""

import unittest

from scripts.baseline_decisions import make_report


class BaselineDecisionsTest(unittest.TestCase):
    def test_positive_and_negative_with_candidate(self) -> None:
        queries = {
            "a": {"split": "development", "segment": "direct",
                  "expected_ids": ["mem"], "forbidden_ids": [],
                  "label_source": "deterministic_fixture_rule"},
            "b": {"split": "development", "segment": "wrong_task",
                  "expected_ids": [], "forbidden_ids": [],
                  "label_source": "deterministic_fixture_rule"},
        }
        results = {
            key: {**case, "response": {"status": "complete",
                                      "candidates": [{"memory_id": "mem"}]}}
            for key, case in queries.items()
        }
        report = make_report(queries, results)
        self.assertEqual([row["correct"] for row in report["individual_results"]],
                         [True, False])
        self.assertEqual(report["max_candidates"], 1)

    def test_mismatched_labels_fail(self) -> None:
        query = {"split": "development", "segment": "direct",
                 "expected_ids": ["mem"], "forbidden_ids": [],
                 "label_source": "deterministic_fixture_rule"}
        result = {**query, "expected_ids": ["other"],
                  "response": {"status": "complete", "candidates": []}}
        with self.assertRaisesRegex(ValueError, "mismatch"):
            make_report({"a": query}, {"a": result})


if __name__ == "__main__":
    unittest.main()
