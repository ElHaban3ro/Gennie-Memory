#!/usr/bin/env python3
"""Validate and summarize paired candidate-cap retrieval samples."""

import argparse
import hashlib
import json
import math
from pathlib import Path


CAPS = (5, 10, 15, 20)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def quantile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("No observations")
    position = (len(ordered) - 1) * fraction
    lower = math.floor(position)
    upper = min(lower + 1, len(ordered) - 1)
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def summarize(sample: Path, result: Path) -> dict:
    expected = rows(sample)
    actual = rows(result)
    if len(expected) != len(actual) or [item["id"] for item in expected] != [item["id"] for item in actual]:
        raise ValueError("Sample/result case IDs or order differ")
    positives = 0
    negatives = 0
    coverage = {str(cap): 0 for cap in CAPS}
    candidate_count = {}
    timings = []
    for case, row in zip(expected, actual):
        for key in ("split", "segment", "expected_ids", "expected_fact", "expected_condition",
                    "expected_source", "forbidden_ids", "label_source"):
            if case.get(key) != row.get(key):
                raise ValueError(f"Label mismatch in {case['id']}: {key}")
        response = row["response"]
        if response["method"] != "fts5_semantic_rrf" or response["status"] != "complete":
            raise ValueError(f"Unexpected retrieval result in {case['id']}")
        candidates = response["candidates"]
        identifiers = [item["memory_id"] for item in candidates]
        if len(identifiers) != len(set(identifiers)) or set(identifiers) & set(case["forbidden_ids"]):
            raise ValueError(f"Duplicate or forbidden candidate in {case['id']}")
        candidate_count[str(len(candidates))] = candidate_count.get(str(len(candidates)), 0) + 1
        if case["expected_ids"]:
            positives += 1
            for cap in CAPS:
                coverage[str(cap)] += bool(set(case["expected_ids"]) & set(identifiers[:cap]))
        else:
            negatives += 1
        elapsed = row["elapsed_ms"]
        if isinstance(elapsed, bool) or not isinstance(elapsed, (int, float)) or not math.isfinite(elapsed) or elapsed < 0:
            raise ValueError(f"Invalid latency in {case['id']}")
        timings.append(elapsed)
    return {"cases": len(actual), "positives": positives, "negatives": negatives,
            "candidate_count": candidate_count, "positive_coverage": coverage,
            "retrieval_ms": {"p50": quantile(timings, .5), "p95": quantile(timings, .95),
                             "p99": quantile(timings, .99)},
            "sample_sha256": sha256(sample), "result_sha256": sha256(result)}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    summary = {"version": 1, "kind": "candidate_matrix_retrieval_summary",
               "caps": CAPS, "scenarios": {}}
    for size in (100, 200, 2000):
        for variant in ("scoped", "unscoped"):
            directory = args.root / str(size)
            key = f"{size}/{variant}"
            summary["scenarios"][key] = summarize(
                directory / f"sample-{variant}.jsonl",
                directory / f"sample-{variant}-results.jsonl")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: {"coverage": value["positive_coverage"],
                            "p50_ms": round(value["retrieval_ms"]["p50"], 2)}
                      for key, value in summary["scenarios"].items()}))


if __name__ == "__main__":
    main()
