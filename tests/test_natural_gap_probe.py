"""Natural paraphrases must preserve split, scope and deterministic v5 labels."""

import hashlib
import json
import unittest
from pathlib import Path

from scripts.generate_natural_gap_probe import derive


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "tests/benchmark/v5/lexical-gap-probe"
PROBE = ROOT / "tests/benchmark/v5/natural-gap-probe"


class NaturalGapProbeTests(unittest.TestCase):
    def test_complete_reproducible_requests(self):
        regenerated = derive(SOURCE)
        manifest = json.loads((PROBE / "manifest.json").read_text())
        parent_cases = {}
        for path in SOURCE.glob("*.jsonl"):
            parent_cases.update({case["id"]: case for case in
                                 (json.loads(line) for line in path.read_text().splitlines())})
        entities = {}
        for split, expected in regenerated.items():
            path = PROBE / f"{split}.jsonl"
            actual = [json.loads(line) for line in path.read_text().splitlines()]
            self.assertEqual(actual, expected)
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),
                             manifest["case_sha256"][split])
            self.assertEqual(len(actual), 200)
            self.assertEqual(sum(bool(case["expected_ids"]) for case in actual), 100)
            entities[split] = {case["request"]["entity_id"] for case in actual}
            self.assertEqual(len(entities[split]), 50)
            for case in actual:
                parent = parent_cases[case["derived_from_case"]]
                self.assertEqual(case["split"], parent["split"])
                self.assertEqual(case["expected_ids"], parent["expected_ids"])
                self.assertEqual(case["expected_condition"], parent["expected_condition"])
                self.assertEqual(case["expected_source"], parent["expected_source"])
                self.assertEqual(case["request"]["scope_id"], parent["request"]["scope_id"])
                self.assertGreaterEqual(len(case["request"]["query"].split()), 8)
        self.assertFalse(entities["development"] & entities["calibration"])


if __name__ == "__main__":
    unittest.main()
