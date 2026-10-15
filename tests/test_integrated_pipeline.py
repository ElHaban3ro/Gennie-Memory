import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from evaluate_pipeline import (CONDITION_CHOICE_QUESTION, CONDITION_QUESTION, QUESTION,
                               ConditionChoiceAgent, PairwiseAgent, RemoteAgent, choose,
                               evaluate_case, load_campaign_rows, append_campaign_event,
                               quantile, selected_cases)


class InvalidChoiceAgent:
    def predict(self, _state, _question):
        return {"answers": {"relevance": {"choice": "unseen", "probabilities": {
            "A": 0.6, "none": 0.4}}}, "usage": {"input_tokens": 5}}


class PipelineContractTests(unittest.TestCase):
    def test_local_journal_resumes_only_the_matching_case_prefix(self):
        manifest = {"case_ids": ["a", "b"], "database_sha256": "first"}
        with tempfile.TemporaryDirectory() as directory:
            journal = Path(directory) / "campaign.jsonl"
            self.assertEqual(load_campaign_rows(journal, manifest), [])
            append_campaign_event(journal, {"phase": "case", "value": {"id": "a", "exact": True}})
            self.assertEqual(load_campaign_rows(journal, manifest), [{"id": "a", "exact": True}])
            with self.assertRaisesRegex(ValueError, "does not match"):
                load_campaign_rows(journal, {**manifest, "database_sha256": "changed"})
            append_campaign_event(journal, {"phase": "case", "value": {"id": "a"}})
            with self.assertRaisesRegex(ValueError, "duplicate, missing, or reordered"):
                load_campaign_rows(journal, manifest)

    def test_two_case_percentiles_do_not_both_use_the_minimum(self):
        self.assertEqual(quantile([100.0, 200.0], .5), 150.0)
        self.assertEqual(quantile([100.0, 200.0], .95), 195.0)

    def test_rejects_unseen_laya_choice(self):
        case = {"id": "synthetic_case", "request": {"query": "synthetic question", "scope_id": "synthetic"}}
        candidates = [{"memory_id": "memory_1", "summary": "synthetic memory"}]
        with self.assertRaisesRegex(ValueError, "Invalid Laya choice"):
            choose(InvalidChoiceAgent(), case, candidates, 4)

    def test_rejects_non_development_cases_before_model_loading(self):
        row = {"id": "case_1", "split": "final", "segment": "direct",
               "label_source": "deterministic_fixture_rule", "expected_ids": []}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cases.jsonl"
            path.write_text(json.dumps(row) + "\n")
            with self.assertRaisesRegex(ValueError, "development"):
                selected_cases(path, 1, ["case_1"])

    def test_calibration_selection_requires_explicit_split_and_keeps_final_closed(self):
        rows = [
            {"id": "cal_1", "split": "calibration", "segment": "absent_entity",
             "label_source": "deterministic_fixture_rule", "expected_ids": []},
            {"id": "final_1", "split": "final", "segment": "absent_entity",
             "label_source": "deterministic_fixture_rule", "expected_ids": []},
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cases.jsonl"
            path.write_text("".join(json.dumps(row) + "\n" for row in rows))
            self.assertEqual([row["id"] for row in selected_cases(path, 1, split="calibration")],
                             ["cal_1"])
            with self.assertRaisesRegex(ValueError, "calibration"):
                selected_cases(path, 1, ["final_1"], "calibration")
            with self.assertRaisesRegex(ValueError, "Only development or calibration"):
                selected_cases(path, 1, split="final")

    def test_remote_choice_maps_short_label_to_authorized_id(self):
        case = {"id": "synthetic_case", "request": {"query": "synthetic question", "scope_id": "synthetic"}}
        candidates = [{"memory_id": "memory_1", "summary": "synthetic memory"}]
        agent = RemoteAgent("test-only-key", "test-model")
        decision = {"chosen_id": "A", "probabilities": {"A": .9, "none": .1},
                    "input_tokens": 7, "reported_cost_usd": .00001, "served_model": "test-model"}
        with patch("evaluate_pipeline.call_openrouter", return_value=decision) as provider:
            chosen, elapsed, tokens = choose(agent, case, candidates, 4)
        self.assertEqual((chosen, tokens), ("memory_1", 7))
        self.assertGreaterEqual(elapsed, 0)
        self.assertEqual(provider.call_args.kwargs["instructions"], QUESTION["instructions"])

    def test_pairwise_threshold_selects_only_a_presented_candidate(self):
        class FakeLaya:
            def __init__(self):
                self.values = iter([.4, .8])

            def predict(self, _state, _question):
                return {"answers": {"relevance": {"noul": next(self.values)}},
                        "usage": {"input_tokens": 4}}

        case = {"id": "synthetic_case", "request": {"query": "synthetic question", "scope_id": "synthetic"}}
        candidates = [{"memory_id": "memory_1", "summary": "first"},
                      {"memory_id": "memory_2", "summary": "second"}]
        agent = PairwiseAgent(FakeLaya(), .5)
        chosen, _elapsed, tokens = choose(agent, case, candidates, 4)
        self.assertEqual((chosen, tokens), ("memory_2", 8))
        self.assertEqual(agent.last_scores, {"memory_1": .4, "memory_2": .8})

    def test_condition_pairwise_passes_structured_condition_to_laya(self):
        class FakeLaya:
            def predict(self, state, questions):
                assert state["candidate_conditions"] == "remote only"
                assert questions["relevance"] is CONDITION_QUESTION
                return {"answers": {"relevance": {"noul": .9}},
                        "usage": {"input_tokens": 4}}

        case = {"id": "synthetic_case", "request": {"query": "synthetic question", "scope_id": "synthetic"}}
        candidates = [{"memory_id": "memory_1", "summary": "synthetic memory",
                       "conditions": "remote only"}]
        agent = PairwiseAgent(FakeLaya(), .5, CONDITION_QUESTION, include_conditions=True)
        chosen, _elapsed, _tokens = choose(agent, case, candidates, 4)
        self.assertEqual(chosen, "memory_1")

    def test_condition_choice_preserves_id_mapping_and_declared_condition(self):
        class FakeLaya:
            def predict(self, _state, questions):
                criteria = questions["relevance"]["criteria"]
                assert questions["relevance"]["instructions"] == CONDITION_CHOICE_QUESTION["instructions"]
                assert "remote only" in criteria["A"]
                return {"answers": {"relevance": {"choice": "A", "probabilities": {
                    "A": .9, "none": .1}}}, "usage": {"input_tokens": 6}}

        case = {"id": "synthetic_case", "request": {"query": "synthetic question", "scope_id": "synthetic"}}
        candidates = [{"memory_id": "memory_1", "summary": "synthetic memory",
                       "conditions": "remote only"}]
        chosen, _elapsed, tokens = choose(ConditionChoiceAgent(FakeLaya()), case, candidates, 4)
        self.assertEqual((chosen, tokens), ("memory_1", 6))

    def test_remote_failure_yields_degraded_seed_not_a_correct_abstention(self):
        candidate = {"memory_id": "memory_1", "revision": 1, "summary": "synthetic memory"}
        case = {"id": "synthetic_negative", "segment": "negative",
                "request": {"query": "synthetic question", "scope_id": "synthetic"},
                "expected_ids": [], "expected_fact": None, "forbidden_ids": [],
                "expected_condition": None, "expected_source": None}

        class FakeSession:
            def call(self, op, request):
                if op == "search":
                    return {"response": {"status": "complete", "method": "fts5",
                                         "semantic_coverage": "disabled", "candidates": [candidate]},
                            "elapsed_ms": 1.0}, 1.1
                self_request = request
                assert self_request["classification_status"] == "failed"
                assert self_request["selected"] == []
                return {"response": {"status": "degraded", "memories": [], "seed_text": None},
                        "elapsed_ms": 1.0}, 1.1

        agent = RemoteAgent("test-only-key", "test-model")
        with patch("evaluate_pipeline.call_openrouter", side_effect=RuntimeError("OpenRouter HTTP 503")):
            row = evaluate_case(FakeSession(), agent, case, 4)
        self.assertEqual(row["decision_status"], "failed")
        self.assertTrue(row["provider_called"])
        self.assertFalse(row["exact"])
        self.assertFalse(row["fact_fidelity"])

    def test_hybrid_seed_reuses_the_measured_query_vector_and_search(self):
        source = {"kind": "document", "ref": "synthetic:guide", "locator": "remote"}
        memory = {"memory_id": "memory_1", "revision": 2, "summary": "synthetic fact",
                  "conditions": "remote only", "source": source}
        case = {"id": "synthetic_positive", "segment": "direct", "request": {
            "query": "synthetic question", "scope_id": "synthetic"},
            "expected_ids": ["memory_1"], "expected_fact": "synthetic fact",
            "expected_condition": "remote only", "expected_source": source, "forbidden_ids": []}

        class FakeVector:
            def __len__(self):
                return 2

            def __iter__(self):
                return iter([1.0, 0.0])

            def astype(self, _kind):
                return self

            def tolist(self):
                return [1.0, 0.0]

        class FakeSession:
            def call(self, op, request):
                if op == "hybrid_search":
                    assert request["query_vector"] == [1.0, 0.0]
                    return {"response": {"status": "complete", "method": "fts5_semantic_rrf",
                                         "semantic_coverage": "complete", "candidates": [memory]},
                            "elapsed_ms": 1.0}, 1.1
                assert op == "compose_seed"
                assert request["search"] is None
                assert request["hybrid"]["query_vector"] == [1.0, 0.0]
                assert request["selected"] == [{"memory_id": "memory_1", "revision": 2}]
                return {"response": {"status": "complete", "memories": [memory],
                                     "seed_text": "synthetic fact remote only synthetic:guide"},
                        "elapsed_ms": 1.0}, 1.1

        row = evaluate_case(FakeSession(), None, case, 4, lambda _texts: [FakeVector()],
                            {"space_id": "synthetic_space", "dimensions": 2})
        self.assertEqual(row["retrieval_method"], "fts5_semantic_rrf")
        self.assertEqual(row["retrieval_snapshot"]["candidates"][0]["memory_id"], "memory_1")
        self.assertTrue(row["fact_fidelity"])
        self.assertTrue(row["condition_fidelity"])
        self.assertTrue(row["source_fidelity"])
        self.assertGreaterEqual(row["embedding_ms"], 0)


if __name__ == "__main__":
    unittest.main()
