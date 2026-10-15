"""Both decision runners must read textual and hybrid benchmark requests."""

import unittest

from scripts.evaluate_laya import search_request as laya_search_request
from scripts.probe_openrouter_decisions import search_request as remote_search_request


class HybridQueryAdapterTest(unittest.TestCase):
    def test_text_and_hybrid_shapes(self) -> None:
        base = {"query": "synthetic task", "scope_id": "synthetic_scope"}
        for adapter in (laya_search_request, remote_search_request):
            self.assertEqual(adapter({"request": base}), base)
            self.assertEqual(adapter({"request": {"search": base,
                                                  "space_id": "synthetic_space",
                                                  "query_vector": [1.0]}}), base)


if __name__ == "__main__":
    unittest.main()
