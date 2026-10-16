#!/usr/bin/env python3
"""Join validated classifier choices with the retrieval snapshot for seed composition."""

import argparse
import json
import math
from pathlib import Path


def read_jsonl(path: Path) -> dict[str, dict]:
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    result = {row["id"]: row for row in rows}
    if len(result) != len(rows):
        raise ValueError("Duplicate case IDs")
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--retrieval-results", type=Path, required=True)
    parser.add_argument("--decision-report", type=Path, required=True)
    parser.add_argument("--model", help="Required when a provider report has multiple models")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    queries = read_jsonl(args.queries)
    retrieval = read_jsonl(args.retrieval_results)
    report = json.loads(args.decision_report.read_text())
    if report["method"] == "deterministic_top1":
        decisions = report["individual_results"]
        classifier = "deterministic_top1"
    elif report["method"].startswith("laya_"):
        decisions = report["individual_results"]
        classifier = f'laya@{report["model_version"]}/{report["method"]}'
    elif report["method"] == "openrouter_decisions_choice_probe":
        if not args.model:
            parser.error("--model is required for provider reports")
        decisions = [row for row in report["results"] if row["requested_model"] == args.model]
        classifier = args.model
    else:
        raise ValueError("Unknown decision report method")
    if not decisions:
        raise ValueError("No matching decisions")
    max_candidates = report.get("max_candidates")
    if isinstance(max_candidates, bool) or not isinstance(max_candidates, int) or not 1 <= max_candidates <= 20:
        raise ValueError("Decision report has no valid max_candidates")
    if len({row["id"] if "id" in row else row["case_id"] for row in decisions}) != len(decisions):
        raise ValueError("Duplicate decisions for the same case")

    output = []
    methods = set()
    for decision in decisions:
        identity = decision.get("id", decision.get("case_id"))
        case = queries[identity]
        result = retrieval[identity]
        if any(case[key] != result[key] for key in
               ("split", "segment", "expected_ids", "forbidden_ids", "label_source")):
            raise ValueError(f"Label mismatch for {identity}")
        if any(case.get(key) != result.get(key) for key in
               ("expected_condition", "expected_source")):
            raise ValueError(f"Metadata label mismatch for {identity}")
        method = result["response"]["method"]
        if method not in ("fts5", "fts5_semantic_rrf"):
            raise ValueError(f"Unknown retrieval method for {identity}")
        methods.add(method)
        choice = decision.get("chosen_id", "none")
        status = decision.get("status", "complete")
        if status not in ("complete", "failed", "skipped_empty"):
            raise ValueError(f"Invalid classification status for {identity}")
        candidates = result["response"]["candidates"]
        candidate_ids = decision.get("candidate_ids")
        if not isinstance(candidate_ids, list) or any(not isinstance(item, str) for item in candidate_ids):
            raise ValueError(f"Missing classifier candidate IDs for {identity}")
        expected_presented = [item["memory_id"] for item in candidates[:max_candidates]]
        if candidate_ids != expected_presented:
            raise ValueError(f"Classifier candidate list differs from retrieval for {identity}")
        presented = [{"memory_id": item["memory_id"], "revision": item["revision"]}
                     for item in candidates[:max_candidates]]
        selected = []
        if choice != "none":
            candidate = next((candidate for candidate in presented if candidate["memory_id"] == choice), None)
            if candidate is None:
                raise ValueError(f"Decision selected an unseen candidate for {identity}")
            selected.append(candidate)
        if status == "failed" and selected:
            raise ValueError(f"Failed classifier selected a memory for {identity}")
        if status == "skipped_empty" and candidates:
            raise ValueError(f"Skipped classifier had candidates for {identity}")
        search = case["request"].get("search", case["request"])
        hybrid = result.get("hybrid_request") if method == "fts5_semantic_rrf" else None
        if method == "fts5_semantic_rrf":
            vector = hybrid.get("query_vector") if isinstance(hybrid, dict) else None
            if (not isinstance(hybrid, dict) or hybrid.get("search") != search or
                not isinstance(hybrid.get("space_id"), str) or not hybrid["space_id"] or
                not isinstance(vector, list) or not vector or
                any(isinstance(value, bool) or not isinstance(value, (int, float)) or
                    not math.isfinite(value) for value in vector)):
                raise ValueError(f"Missing or invalid frozen hybrid request for {identity}")
        request = {"search": search if method == "fts5" else None,
                   "hybrid": hybrid,
                   "presented": presented,
                   "selected": selected,
                   "classifier": ("deterministic-empty" if status == "skipped_empty" else
                                  decision.get("served_model") or classifier),
                   "classification_status": "failed" if status == "failed" else "complete"}
        output.append({"id": identity, "split": case["split"], "segment": case["segment"],
                       "request": request, "expected_ids": case["expected_ids"],
                       "expected_fact": case["expected_fact"],
                       "expected_condition": case.get("expected_condition"),
                       "expected_source": case.get("expected_source"),
                       "forbidden_ids": case["forbidden_ids"],
                       "label_source": case["label_source"]})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                                   for row in output))
    print(json.dumps({"cases": len(output), "retrieval_methods": sorted(methods),
                      "failed": sum(row["request"]["classification_status"] == "failed" for row in output)}))


if __name__ == "__main__":
    main()
