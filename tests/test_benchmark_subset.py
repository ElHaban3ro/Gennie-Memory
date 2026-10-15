"""A second development sample must not reuse cases from the first."""

import json
import tempfile
import unittest
from pathlib import Path

from scripts.select_benchmark_cases import select_cases, select_form_coverage_cases


class BenchmarkSubsetTests(unittest.TestCase):
    def test_form_coverage_selects_each_form_and_negative_control(self):
        rows = [
            {"id": "q5_0001_0000", "split": "development", "segment": "direct",
             "label_source": "deterministic_fixture_rule", "expected_ids": ["m1"],
             "request": {"entity_id": "old"}},
            {"id": "q5_0002_0000", "split": "development", "segment": "direct",
             "label_source": "deterministic_fixture_rule", "expected_ids": ["m2"],
             "request": {"entity_id": "new"}},
            {"id": "q5_0002_0001", "split": "development", "segment": "english",
             "label_source": "deterministic_fixture_rule", "expected_ids": ["m2"],
             "request": {"entity_id": "new"}},
            {"id": "negative_0002", "split": "development", "segment": "wrong_task",
             "label_source": "deterministic_fixture_rule", "expected_ids": [],
             "request": {"entity_id": "new"}},
        ]
        with tempfile.TemporaryDirectory() as directory:
            queries = Path(directory) / "queries.jsonl"
            forms = Path(directory) / "forms.json"
            queries.write_text("".join(json.dumps(row) + "\n" for row in rows))
            forms.write_text(json.dumps([{"id": 0, "segment": "direct"},
                                         {"id": 1, "segment": "english"}]))
            selected = select_form_coverage_cases(queries, forms, "development", 1,
                                                  set(), {"old"})
        self.assertEqual({row["id"] for row in selected},
                         {"q5_0002_0000", "q5_0002_0001", "negative_0002"})

    def test_excludes_prior_ids_before_balanced_selection(self):
        rows = [
            {"id": "dev_a_1", "split": "development", "segment": "a"},
            {"id": "dev_a_2", "split": "development", "segment": "a"},
            {"id": "dev_b_1", "split": "development", "segment": "b"},
            {"id": "dev_b_2", "split": "development", "segment": "b"},
            {"id": "final_a", "split": "final", "segment": "a"},
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "queries.jsonl"
            path.write_text("".join(json.dumps(row) + "\n" for row in rows))
            selected = select_cases(path, "development", 1, {"dev_a_1", "dev_b_1"})
        self.assertEqual({row["id"] for row in selected}, {"dev_a_2", "dev_b_2"})

    def test_excludes_prior_entities_even_with_new_query_ids(self):
        rows = [
            {"id": "old", "split": "development", "segment": "a",
             "request": {"entity_id": "project_1"}},
            {"id": "new_same_project", "split": "development", "segment": "a",
             "request": {"entity_id": "project_1"}},
            {"id": "new_project", "split": "development", "segment": "a",
             "request": {"entity_id": "project_2"}},
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "queries.jsonl"
            path.write_text("".join(json.dumps(row) + "\n" for row in rows))
            selected = select_cases(path, "development", 1, {"old"}, {"project_1"})
        self.assertEqual([row["id"] for row in selected], ["new_project"])


if __name__ == "__main__":
    unittest.main()
