#!/usr/bin/env python3
"""Validate and summarize paired, complete local seed campaigns."""

import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def quantile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    position = (len(ordered) - 1) * fraction
    lower = int(position)
    upper = min(len(ordered) - 1, lower + 1)
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def summarize(paths: list[Path], queries_path: Path) -> dict:
    queries = [json.loads(line) for line in queries_path.read_text().splitlines() if line.strip()]
    split = queries[0]["split"] if queries else None
    query_ids = [case["id"] for case in queries]
    if split not in ("development", "calibration") or len(query_ids) != len(set(query_ids)) or any(
        case["split"] != split or
        case["label_source"] != "deterministic_fixture_rule" for case in queries
    ):
        raise ValueError("Expected unique labeled synthetic development or calibration queries")
    by_id = {case["id"]: case for case in queries}
    reports = [json.loads(path.read_text()) for path in paths]
    names = [report["method"] for report in reports]
    if len(names) != len(set(names)):
        raise ValueError("Duplicate campaign method")
    rows_by_method = {}
    baseline_snapshots = None
    for name, report in zip(names, reports):
        rows = report.get("individual_results")
        if (not name.endswith("_seed_campaign") or report.get("split") != split or
            report.get("retrieval") != "hybrid" or report.get("device") != "mps" or
            report.get("cases") != len(queries) or not isinstance(rows, list) or
            len(rows) != len(queries)):
            raise ValueError(f"Incomplete or non-MPS campaign: {name}")
        ids = [row["id"] for row in rows]
        if len(set(ids)) != len(ids) or set(ids) != set(query_ids):
            raise ValueError(f"Mismatched campaign IDs: {name}")
        indexed = {row["id"]: row for row in rows}
        snapshots = {}
        for case_id, case in by_id.items():
            row = indexed[case_id]
            snapshot = row.get("retrieval_snapshot")
            if (row["segment"] != case["segment"] or
                row["positive"] != bool(case["expected_ids"]) or
                row["semantic_coverage"] != "complete" or
                row["decision_status"] != "complete" or
                row["seed_status"] != "complete" or row["forbidden_leak"] or
                not isinstance(snapshot, dict)):
                raise ValueError(f"Invalid case result: {name}/{case_id}")
            snapshots[case_id] = json.dumps(snapshot, ensure_ascii=False, sort_keys=True)
        if baseline_snapshots is None:
            baseline_snapshots = snapshots
        elif snapshots != baseline_snapshots:
            raise ValueError(f"Retrieval candidates differ: {name}")
        rows_by_method[name] = indexed
    methods = {}
    for name, report in zip(names, reports):
        indexed = rows_by_method[name]
        rows = [indexed[case_id] for case_id in query_ids]
        positive = [row for row in rows if row["positive"]]
        negative = [row for row in rows if not row["positive"]]
        groups = defaultdict(list)
        for row in rows:
            groups[row["segment"]].append(row)
        methods[name] = {
            "cases": len(rows), "exact": sum(row["exact"] for row in rows),
            "positive_exact": sum(row["exact"] for row in positive),
            "negative_empty": sum(row["exact"] for row in negative),
            "positive_missed": sum(row["chosen_id"] == "none" for row in positive),
            "positive_wrong_memory": sum(row["chosen_id"] not in
                                         ("none", *case["expected_ids"])
                                         for case, row in ((by_id[row["id"]], row) for row in positive)),
            "negative_false_memory": sum(row["chosen_id"] != "none" for row in negative),
            "candidate_coverage": sum(row["expected_in_candidates"] for row in positive),
            "positive_fact_fidelity": sum(row["fact_fidelity"] for row in positive),
            "positive_condition_fidelity": sum(row["condition_fidelity"] for row in positive),
            "positive_source_fidelity": sum(row["source_fidelity"] for row in positive),
            "input_tokens": report["input_tokens"],
            "load_seconds": report["load_seconds"],
            "warmup_ms": report["warmup_ms"],
            "latency_ms": report["latency_ms"],
            "total_ms_p99": quantile([row["total_ms"] for row in rows], .99),
            "sum_case_seconds": sum(row["total_ms"] for row in rows) / 1000,
            "segments": {segment: {"cases": len(group), "exact": sum(row["exact"] for row in group)}
                         for segment, group in sorted(groups.items())},
        }
    paired = {}
    for left_index, left in enumerate(names):
        for right in names[left_index + 1:]:
            left_only = right_only = both = neither = 0
            for case_id in query_ids:
                a = rows_by_method[left][case_id]["exact"]
                b = rows_by_method[right][case_id]["exact"]
                if a and b:
                    both += 1
                elif a:
                    left_only += 1
                elif b:
                    right_only += 1
                else:
                    neither += 1
            paired[f"{left}__vs__{right}"] = {
                "both": both, "left_only": left_only,
                "right_only": right_only, "neither": neither,
            }
    return {"kind": f"v5_{split}_local_seed_campaign", "queries_sha256": sha256(queries_path),
            "report_sha256": {name: sha256(path) for name, path in zip(names, paths)},
            "cases": len(queries), "segments": dict(sorted(Counter(case["segment"] for case in queries).items())),
            "methods": methods, "paired": paired}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--report", type=Path, action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = summarize(args.report, args.queries)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": result["cases"],
                      "methods": {name: value["exact"] for name, value in result["methods"].items()}},
                     ensure_ascii=False))


if __name__ == "__main__":
    main()
