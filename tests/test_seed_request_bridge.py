"""Check that a classifier can select only candidates it actually received."""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/build_seed_requests.py"


class SeedRequestBridgeTest(unittest.TestCase):
    def test_remote_hybrid_seed_uses_the_frozen_query_vector(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            case = {"id": "q5_0002_0000", "split": "development", "segment": "direct",
                    "request": {"query": "synthetic remote request", "scope_kind": "project",
                                "scope_id": "synthetic", "as_of": "2026-10-15T12:00:00.000Z",
                                "limit": 4},
                    "expected_ids": ["wanted"], "expected_fact": "synthetic fact",
                    "forbidden_ids": [], "label_source": "deterministic_fixture_rule",
                    "expected_condition": "remote only",
                    "expected_source": {"kind": "document", "ref": "synthetic:guide"}}
            result = {key: case[key] for key in
                      ("id", "split", "segment", "expected_ids", "forbidden_ids",
                       "label_source", "expected_condition", "expected_source")}
            result["response"] = {"method": "fts5_semantic_rrf", "candidates": [
                {"memory_id": "wanted", "revision": 2, "summary": "synthetic fact"}]}
            result["hybrid_request"] = {"search": case["request"], "space_id": "synthetic-space",
                                        "query_vector": [1.0, 0.0]}
            report = {"method": "openrouter_decisions_choice_probe", "max_candidates": 4,
                      "results": [{"case_id": case["id"], "requested_model": "synthetic-model",
                                   "served_model": "synthetic-model", "status": "complete",
                                   "candidate_ids": ["wanted"], "chosen_id": "wanted"}]}
            (root / "queries.jsonl").write_text(json.dumps(case) + "\n")
            (root / "results.jsonl").write_text(json.dumps(result) + "\n")
            (root / "report.json").write_text(json.dumps(report))
            command = [sys.executable, str(SCRIPT), "--queries", str(root / "queries.jsonl"),
                       "--retrieval-results", str(root / "results.jsonl"),
                       "--decision-report", str(root / "report.json"),
                       "--model", "synthetic-model", "--output", str(root / "seed.jsonl")]
            completed = subprocess.run(command, capture_output=True, text=True, check=True)
            request = json.loads((root / "seed.jsonl").read_text())["request"]
            self.assertIsNone(request["search"])
            self.assertEqual(request["hybrid"]["query_vector"], [1.0, 0.0])
            self.assertEqual(request["selected"], [{"memory_id": "wanted", "revision": 2}])

            del result["hybrid_request"]
            (root / "results.jsonl").write_text(json.dumps(result) + "\n")
            rejected = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn("frozen hybrid request", rejected.stderr)

    def test_rejects_unseen_selection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            labels = {"id": "q1", "split": "development", "segment": "direct",
                      "request": {"query": "synthetic query", "scope_kind": "project",
                                  "scope_id": "test", "as_of": "2026-10-15T12:00:00.000Z",
                                  "limit": 20},
                      "expected_ids": ["a"], "expected_fact": "synthetic fact",
                      "forbidden_ids": [], "label_source": "deterministic_fixture_rule"}
            (root / "queries.jsonl").write_text(json.dumps(labels) + "\n")
            result = {key: labels[key] for key in
                      ("id", "split", "segment", "expected_ids", "expected_fact",
                       "forbidden_ids", "label_source")}
            result["response"] = {"method": "fts5", "candidates": [
                {"memory_id": "a", "revision": 2}, {"memory_id": "b", "revision": 1}]}
            (root / "results.jsonl").write_text(json.dumps(result) + "\n")
            report = {"method": "laya_choice_rerank_pilot", "model_version": "test",
                      "max_candidates": 1, "individual_results": [
                          {"id": "q1", "candidate_ids": ["a"], "chosen_id": "a"}]}
            report_path = root / "report.json"
            report_path.write_text(json.dumps(report))
            command = [sys.executable, str(SCRIPT), "--queries", str(root / "queries.jsonl"),
                       "--retrieval-results", str(root / "results.jsonl"),
                       "--decision-report", str(report_path), "--output", str(root / "seed.jsonl")]
            completed = subprocess.run(command, capture_output=True, text=True, check=True)
            self.assertIn('"cases": 1', completed.stdout)
            request = json.loads((root / "seed.jsonl").read_text())["request"]
            self.assertEqual(request["presented"], [{"memory_id": "a", "revision": 2}])
            self.assertEqual(request["selected"], request["presented"])

            report["individual_results"][0]["chosen_id"] = "b"
            report_path.write_text(json.dumps(report))
            rejected = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn("unseen candidate", rejected.stderr)

            report["individual_results"][0]["chosen_id"] = "a"
            report_path.write_text(json.dumps(report))
            labels["expected_condition"] = "Solo remoto"
            result["expected_condition"] = "Solo presencial"
            (root / "queries.jsonl").write_text(json.dumps(labels) + "\n")
            (root / "results.jsonl").write_text(json.dumps(result) + "\n")
            rejected = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn("Metadata label mismatch", rejected.stderr)


if __name__ == "__main__":
    unittest.main()
