"""Short model-facing labels must map back to the authorized memory IDs."""

import unittest

from scripts.evaluate_laya import choice_criteria


class LayaChoiceLabelsTest(unittest.TestCase):
    def test_letters_preserve_candidate_identity(self) -> None:
        candidates = [
            {"memory_id": "m_opaque_1", "summary": "Synthetic fact one"},
            {"memory_id": "m_opaque_2", "summary": "Synthetic fact two"},
        ]
        criteria, mapping = choice_criteria(candidates, "letters")
        self.assertEqual(list(criteria), ["A", "B", "none"])
        self.assertEqual(mapping, {"A": "m_opaque_1", "B": "m_opaque_2",
                                   "none": "none"})
        ids, original = choice_criteria(candidates, "memory_id")
        self.assertEqual(list(ids), ["m_opaque_1", "m_opaque_2", "none"])
        self.assertEqual(original["m_opaque_2"], "m_opaque_2")

    def test_reserved_memory_id_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "reserved"):
            choice_criteria([{"memory_id": "none", "summary": "Synthetic"}],
                            "memory_id")


if __name__ == "__main__":
    unittest.main()
