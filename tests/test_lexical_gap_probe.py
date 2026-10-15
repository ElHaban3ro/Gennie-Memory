"""The paraphrase probe must stay tied to disjoint labeled v5 source cases."""

import hashlib
import json
import unittest
from pathlib import Path

from scripts.generate_lexical_gap_probe import CONDITIONS, derive


ROOT = Path(__file__).resolve().parents[1]
BENCHMARK = ROOT / "tests/benchmark/v5"
PROBE = BENCHMARK / "lexical-gap-probe"


class LexicalGapProbeTests(unittest.TestCase):
    def test_reproducible_labels_and_entity_partitions(self):
        sources = {"development": BENCHMARK / "development-campaign-1600.jsonl",
                   "calibration": BENCHMARK / "calibration-campaign-1480.jsonl"}
        manifest = json.loads((PROBE / "manifest.json").read_text())
        regenerated = derive(sources, manifest["entities_per_split"])
        source_cases = {}
        for path in sources.values():
            source_cases.update({case["id"]: case for case in
                                 (json.loads(line) for line in path.read_text().splitlines())})
        entity_sets = {}
        for split in sources:
            path = PROBE / f"{split}.jsonl"
            actual = [json.loads(line) for line in path.read_text().splitlines()]
            expected = [case for case in regenerated if case["split"] == split]
            self.assertEqual(actual, expected)
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),
                             manifest["case_sha256"][split])
            self.assertEqual(len(actual), 100)
            entity_sets[split] = {case["request"]["entity_id"] for case in actual}
            self.assertEqual(len(entity_sets[split]), 50)
            for case in actual:
                source = source_cases[case["derived_from_case"]]
                self.assertEqual(case["expected_ids"], source["expected_ids"])
                self.assertEqual(case["expected_condition"], source["expected_condition"])
                self.assertIn(case["expected_condition"], CONDITIONS)
                self.assertEqual(case["expected_source"], source["expected_source"])
                self.assertEqual(case["request"]["scope_id"], source["request"]["scope_id"])
        self.assertFalse(entity_sets["development"] & entity_sets["calibration"])


if __name__ == "__main__":
    unittest.main()
