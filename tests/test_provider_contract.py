"""Focused contract checks without provider calls or credentials."""

import importlib.util
import unittest
from pathlib import Path


path = Path(__file__).resolve().parents[1] / "scripts/probe_openrouter_decisions.py"
spec = importlib.util.spec_from_file_location("openrouter_probe", path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class DecisionContractTests(unittest.TestCase):
    def test_valid_choice_and_refusal_or_unknown_id(self):
        criteria = {"synthetic_0001": "fictitious memory", "none": "no memory"}
        body = {"model": "test-model", "answers": {"relevance": {
            "type": "choice", "choice": "synthetic_0001",
            "probabilities": {"synthetic_0001": .8, "none": .2}}},
            "usage": {"input_tokens": 10, "output_tokens": 0, "cost": .001}}
        result = module.parse_decision(body, criteria, "requested-model", 12.0)
        self.assertEqual(result["chosen_id"], "synthetic_0001")
        self.assertEqual(result["reported_cost_usd"], .001)
        bad = {**body, "answers": {"relevance": {"type": "refusal"}}}
        with self.assertRaisesRegex(ValueError, "valid choice"):
            module.parse_decision(bad, criteria, "requested-model", 12.0)
        bad = {**body, "answers": {"relevance": {
            "type": "choice", "choice": "foreign_id",
            "probabilities": {"synthetic_0001": .8, "none": .2}}}}
        with self.assertRaisesRegex(ValueError, "invalid choice IDs"):
            module.parse_decision(bad, criteria, "requested-model", 12.0)

    def test_invalid_usage_is_rejected(self):
        criteria = {"synthetic_0001": "fictitious memory", "none": "no memory"}
        base = {"model": "test-model", "answers": {"relevance": {
            "type": "choice", "choice": "none",
            "probabilities": {"synthetic_0001": .1, "none": .9}}}}
        with self.assertRaisesRegex(ValueError, "invalid cost"):
            module.parse_decision({**base, "usage": {"cost": -1}}, criteria, "requested-model", 12.0)
        with self.assertRaisesRegex(ValueError, "invalid input_tokens"):
            module.parse_decision({**base, "usage": {"input_tokens": True}}, criteria,
                                  "requested-model", 12.0)


if __name__ == "__main__":
    unittest.main()
