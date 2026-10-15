#!/usr/bin/env python3
"""Summarize measured retrieval results without treating missing cases as wins."""

import argparse
import json
import statistics
from collections import defaultdict
from pathlib import Path


def quantile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    if not ordered:
        return 0.0
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * fraction))]


def summarize(rows: list[dict]) -> dict:
    positive = [r for r in rows if r["expected_ids"]]
    negative = [r for r in rows if not r["expected_ids"]]
    return {
        "cases": len(rows),
        "positive": len(positive),
        "negative": len(negative),
        "recall_at_20": sum(bool(set(r["expected_ids"]) & set(r["candidate_ids"])) for r in positive) / len(positive) if positive else None,
        "top_1_accuracy": sum(r["candidate_ids"][:1] == r["expected_ids"][:1] for r in positive) / len(positive) if positive else None,
        "negative_empty_rate": sum(not r["candidate_ids"] for r in negative) / len(negative) if negative else None,
        "forbidden_leak_count": sum(bool(set(r["forbidden_ids"]) & set(r["candidate_ids"])) for r in rows),
        "latency_ms": {"p50": quantile([r["elapsed_ms"] for r in rows], .5),
                       "p95": quantile([r["elapsed_ms"] for r in rows], .95),
                       "p99": quantile([r["elapsed_ms"] for r in rows], .99)},
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", type=Path)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    query_rows = [json.loads(line) for line in args.queries.read_text().splitlines() if line.strip()]
    queries = {item["id"]: item for item in query_rows}
    if len(queries) != len(query_rows):
        raise ValueError("Duplicate case IDs in query file")
    rows = []
    for number, line in enumerate(args.results.read_text().splitlines(), 1):
        if not line.strip():
            continue
        item = json.loads(line)
        if item["response"]["status"] not in ("complete", "degraded") or item["response"]["method"] not in ("fts5", "fts5_semantic_rrf"):
            raise ValueError(f"Unexpected retrieval status at line {number}")
        item["candidate_ids"] = [candidate["memory_id"] for candidate in item["response"]["candidates"]]
        expected = queries.get(item["id"])
        if expected is None or any(item[key] != expected[key] for key in
                                   ("split", "segment", "expected_ids", "forbidden_ids", "label_source")):
            raise ValueError(f"Result does not match source labels at line {number}")
        if any(item.get(key) != expected.get(key) for key in
               ("expected_condition", "expected_source")):
            raise ValueError(f"Result does not match metadata labels at line {number}")
        rows.append(item)
    if len({item["id"] for item in rows}) != len(rows):
        raise ValueError("Duplicate case IDs in result file")
    if {item["id"] for item in rows} != set(queries):
        raise ValueError("Result file is incomplete or contains extra cases")
    methods = {row["response"]["method"] for row in rows}
    if len(methods) != 1:
        raise ValueError("Result file mixes retrieval methods")
    groups: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        groups[f'{row["split"]}/{row["segment"]}'].append(row)
    report = {"method": methods.pop(), "overall": summarize(rows),
              "by_split_segment": {key: summarize(values) for key, values in sorted(groups.items())}}
    report["degraded_cases"] = sum(row["response"]["status"] == "degraded" for row in rows)
    report["semantic_coverage"] = {name: sum(row["response"]["semantic_coverage"] == name for row in rows)
                                   for name in ("disabled", "complete", "partial", "missing")}
    encoded = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.write_text(encoded)
    else:
        print(encoded, end="")


if __name__ == "__main__":
    main()
