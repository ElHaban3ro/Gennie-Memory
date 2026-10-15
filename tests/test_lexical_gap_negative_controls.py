"""Negative lexical-gap cases must inherit authorized v5 no-memory labels."""

import hashlib
import json
import unittest
from pathlib import Path

from scripts.generate_lexical_gap_negatives import derive


ROOT = Path(__file__).resolve().parents[1]
PROBE = ROOT / "tests/benchmark/v5/lexical-gap-probe"
QUERIES = ROOT / "tests/benchmark/v5/queries.jsonl"


class LexicalGapNegativeControlsTests(unittest.TestCase):
    def test_reproducible_disjoint_controls(self):
        positives = {split: PROBE / f"{split}.jsonl" for split in ("development", "calibration")}
        manifest = json.loads((PROBE / "negative-manifest.json").read_text())
        expected = derive(positives, QUERIES)
        original = {case["id"]: case for case in
                    (json.loads(line) for line in QUERIES.read_text().splitlines() if line.strip())}
        entity_sets = {}
        for split, cases in expected.items():
            path = PROBE / f"negatives-{split}.jsonl"
            actual = [json.loads(line) for line in path.read_text().splitlines()]
            self.assertEqual(actual, cases)
            self.assertEqual(len(actual), 100)
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),
                             manifest["case_sha256"][split])
            entity_sets[split] = {case["request"]["entity_id"] for case in actual}
            self.assertEqual(len(entity_sets[split]), 50)
            for case in actual:
                source = original[case["derived_from_case"]]
                self.assertEqual(source["split"], split)
                self.assertEqual(case["expected_ids"], source["expected_ids"])
                self.assertEqual(case["expected_ids"], [])
                self.assertEqual(case["forbidden_ids"], source["forbidden_ids"])
                self.assertEqual(case["request"]["scope_id"], source["request"]["scope_id"])
        self.assertFalse(entity_sets["development"] & entity_sets["calibration"])


if __name__ == "__main__":
    unittest.main()
