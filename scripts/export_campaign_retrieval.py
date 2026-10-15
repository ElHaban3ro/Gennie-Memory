#!/usr/bin/env python3
"""Freeze the exact synthetic candidates seen by a completed local seed campaign."""

import argparse
import json
import math
from pathlib import Path


LABEL_FIELDS = ("split", "segment", "expected_ids", "forbidden_ids", "label_source",
                "expected_condition", "expected_source")


def export_rows(queries: list[dict], report: dict) -> list[dict]:
    split = queries[0]["split"] if queries else None
    if (split not in ("development", "calibration") or report.get("split") != split or
        not report.get("method", "").endswith("_seed_campaign") or
        report.get("cases") != len(queries)):
        raise ValueError("Expected a complete synthetic development or calibration campaign report")
    by_id = {case["id"]: case for case in queries}
    if len(by_id) != len(queries) or any(
        case.get("split") != split or
        case.get("label_source") != "deterministic_fixture_rule" for case in queries
    ):
        raise ValueError("Queries must be unique labeled synthetic development or calibration cases")
    observed = report.get("individual_results")
    if not isinstance(observed, list) or len(observed) != len(queries):
        raise ValueError("Report has missing or extra case results")
    result_ids = [row["id"] for row in observed]
    if len(set(result_ids)) != len(result_ids) or set(result_ids) != set(by_id):
        raise ValueError("Report IDs do not match the query file")
    output = []
    profile = report.get("embedding_profile")
    for row in observed:
        case = by_id[row["id"]]
        response = row.get("retrieval_snapshot")
        if (row.get("segment") != case["segment"] or
            row.get("positive") != bool(case["expected_ids"]) or
            not isinstance(response, dict) or
            response.get("status") not in ("complete", "degraded") or
            response.get("method") not in ("fts5", "fts5_semantic_rrf") or
            response.get("method") != row.get("retrieval_method") or
            not isinstance(response.get("candidates"), list) or
            len(response["candidates"]) != row.get("candidate_count")):
            raise ValueError(f"Invalid retrieval snapshot: {row['id']}")
        candidates = response["candidates"]
        candidate_ids = [candidate["memory_id"] for candidate in candidates]
        if (len(set(candidate_ids)) != len(candidate_ids) or
            set(candidate_ids) & set(case["forbidden_ids"]) or
            any(not isinstance(candidate.get("summary"), str) or
                not isinstance(candidate.get("revision"), int) for candidate in candidates)):
            raise ValueError(f"Invalid candidate list: {row['id']}")
        retrieval_request = row.get("retrieval_request")
        if response["method"] == "fts5_semantic_rrf":
            vector = retrieval_request.get("query_vector") if isinstance(retrieval_request, dict) else None
            if (not isinstance(profile, dict) or
                not isinstance(retrieval_request, dict) or
                retrieval_request.get("search") != case["request"] or
                retrieval_request.get("space_id") != profile.get("space_id") or
                not isinstance(vector, list) or len(vector) != profile.get("dimensions") or
                any(isinstance(value, bool) or not isinstance(value, (int, float)) or
                    not math.isfinite(value) for value in vector)):
                raise ValueError(f"Invalid hybrid request: {row['id']}")
        elif retrieval_request != case["request"]:
            raise ValueError(f"Invalid textual request: {row['id']}")
        exported = {"id": row["id"], **{field: case.get(field) for field in LABEL_FIELDS},
                    "response": response, "elapsed_ms": row["search_rust_ms"]}
        if response["method"] == "fts5_semantic_rrf":
            exported["hybrid_request"] = retrieval_request
        output.append(exported)
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.resolve() in {args.queries.resolve(), args.report.resolve()}:
        parser.error("Output must not overwrite an input")
    queries = [json.loads(line) for line in args.queries.read_text().splitlines() if line.strip()]
    rows = export_rows(queries, json.loads(args.report.read_text()))
    payload = "".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                      for row in rows)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(payload)
    print(json.dumps({"cases": len(rows), "output": str(args.output)}))


if __name__ == "__main__":
    main()
