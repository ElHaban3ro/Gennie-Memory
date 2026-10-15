"""Duplicate vectors may be removed only after exact snapshot matching."""

import unittest

from scripts.compact_campaign_reports import compact


class CompactCampaignReportsTests(unittest.TestCase):
    def test_requires_exact_hybrid_request_before_removing_it(self):
        request = {"search": {"query": "synthetic"}, "space_id": "space",
                   "query_vector": [1.0, 0.0]}
        report = {"individual_results": [{"id": "q1", "retrieval_request": request}]}
        snapshot = {"q1": {"hybrid_request": request}}
        result = compact(report, snapshot, "synthetic-hash")
        self.assertNotIn("retrieval_request", result["individual_results"][0])
        self.assertEqual(result["full_report_sha256"], "synthetic-hash")

        report = {"individual_results": [{"id": "q1", "retrieval_request": request}]}
        snapshot["q1"]["hybrid_request"] = {**request, "query_vector": [0.0, 1.0]}
        with self.assertRaisesRegex(ValueError, "vector differs"):
            compact(report, snapshot, "synthetic-hash")


if __name__ == "__main__":
    unittest.main()
