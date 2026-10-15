#!/usr/bin/env python3
"""Compare complete FTS5 and hybrid top-1 seed runs on identical synthetic cases."""

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path


CONTENT_FIELDS = ("memory_id", "revision", "state", "summary", "conditions", "source")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compare(queries_path: Path, fts_path: Path, hybrid_path: Path) -> dict:
    queries = [json.loads(line) for line in queries_path.read_text().splitlines() if line.strip()]
    split = queries[0]["split"] if queries else None
    if (split not in ("development", "calibration") or
        len({case["id"] for case in queries}) != len(queries) or
        any(case["split"] != split or case["label_source"] != "deterministic_fixture_rule"
            for case in queries)):
        raise ValueError("Expected unique labeled development or calibration cases")
    reports = [json.loads(path.read_text()) for path in (fts_path, hybrid_path)]
    methods = ("integrated_fts5_top1_seed_campaign", "integrated_hybrid_top1_seed_campaign")
    indexed = []
    for report, method in zip(reports, methods):
        rows = report.get("individual_results")
        if (report.get("split") != split or report.get("method") != method or
            report.get("cases") != len(queries) or not isinstance(rows, list) or
            len(rows) != len(queries) or len({row["id"] for row in rows}) != len(queries) or
            {row["id"] for row in rows} != {case["id"] for case in queries} or
            report.get("decision_failures") or report.get("degraded_seeds") or
            report.get("forbidden_leaks")):
            raise ValueError(f"Incomplete or mismatched report: {method}")
        indexed.append({row["id"]: row for row in rows})
    by_segment = defaultdict(lambda: {"cases": 0, "fts5_exact": 0, "hybrid_exact": 0})
    different_order = different_content = different_choice = different_exact = 0
    for case in queries:
        left, right = (rows[case["id"]] for rows in indexed)
        if any(row["segment"] != case["segment"] or
               row["positive"] != bool(case["expected_ids"]) or
               row["decision_status"] != "complete" or row["seed_status"] != "complete" or
               row["forbidden_leak"] for row in (left, right)):
            raise ValueError(f"Invalid case result: {case['id']}")
        if left["semantic_coverage"] != "disabled" or right["semantic_coverage"] != "complete":
            raise ValueError(f"Unexpected semantic coverage: {case['id']}")
        left_candidates = left["retrieval_snapshot"]["candidates"]
        right_candidates = right["retrieval_snapshot"]["candidates"]
        left_order = [(c["memory_id"], c["revision"]) for c in left_candidates]
        right_order = [(c["memory_id"], c["revision"]) for c in right_candidates]
        different_order += left_order != right_order
        different_content += ([tuple(c.get(field) for field in CONTENT_FIELDS) for c in left_candidates] !=
                              [tuple(c.get(field) for field in CONTENT_FIELDS) for c in right_candidates])
        different_choice += left["chosen_id"] != right["chosen_id"]
        different_exact += left["exact"] != right["exact"]
        group = by_segment[case["segment"]]
        group["cases"] += 1
        group["fts5_exact"] += bool(left["exact"])
        group["hybrid_exact"] += bool(right["exact"])
    return {"kind": "v5_top1_retrieval_ablation", "split": split,
            "cases": len(queries), "queries_sha256": sha256(queries_path),
            "report_sha256": {"fts5": sha256(fts_path), "hybrid": sha256(hybrid_path)},
            "different_candidate_order": different_order,
            "different_candidate_content": different_content,
            "different_choice": different_choice, "different_exact": different_exact,
            "methods": {name: {"exact": report["exact_selection"],
                               "positive_exact": report["positive_fact_fidelity"],
                               "negative_empty": report["negative_empty"],
                               "candidate_coverage": report["candidate_coverage"],
                               "latency_ms": report["latency_ms"]["total_ms"]}
                        for name, report in zip(("fts5", "hybrid"), reports)},
            "segments": dict(sorted(by_segment.items()))}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--fts5", type=Path, required=True)
    parser.add_argument("--hybrid", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.resolve() in {p.resolve() for p in (args.queries, args.fts5, args.hybrid)}:
        parser.error("Output must not overwrite an input")
    result = compare(args.queries, args.fts5, args.hybrid)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"split": result["split"], "cases": result["cases"],
                      "different_candidate_order": result["different_candidate_order"],
                      "different_exact": result["different_exact"]}))


if __name__ == "__main__":
    main()
