#!/usr/bin/env python3
"""Validate final holdout retrieval outputs and report candidate coverage."""

import argparse
import hashlib
import json
from pathlib import Path


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def quantile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads((args.root / "manifest.json").read_text())
    summary = {"kind": "candidate_matrix_final_retrieval", "synthetic_only": True,
               "manifest_sha256": hashlib.sha256((args.root / "manifest.json").read_bytes()).hexdigest(),
               "groups": {}}
    for scope in ("scoped", "unscoped"):
        query_path = args.root / f"{scope}-queries.jsonl"
        result_path = args.root / f"{scope}-results.jsonl"
        queries = rows(query_path)
        results = rows(result_path)
        if (hashlib.sha256(query_path.read_bytes()).hexdigest() !=
                manifest["variants"][scope]["queries_sha256"] or
                len(queries) != 480 or len(results) != 480 or
                [row["id"] for row in queries] != [row["id"] for row in results]):
            raise ValueError(f"Final input/result mismatch: {scope}")
        for query, result in zip(queries, results):
            if query["split"] != result["split"] or query["split"] != "final" or (
                    query["expected_ids"] != result["expected_ids"] or
                    result["response"]["status"] not in ("complete", "degraded")):
                raise ValueError(f"Final label/result mismatch: {query['id']}")
        positive = [row for row in results if row["expected_ids"]]
        negative = [row for row in results if not row["expected_ids"]]
        if len(positive) != 400 or len(negative) != 80:
            raise ValueError("Final label counts changed")
        times = [row["elapsed_ms"] for row in results]
        summary["groups"][scope] = {
            "queries_sha256": manifest["variants"][scope]["queries_sha256"],
            "results_sha256": hashlib.sha256(result_path.read_bytes()).hexdigest(),
            "positive_cases": len(positive), "negative_cases": len(negative),
            "candidate_coverage": {
                str(cap): sum(row["expected_ids"][0] in [candidate["memory_id"] for candidate in
                               row["response"]["candidates"][:cap]] for row in positive)
                for cap in (5, 10, 15, 20)},
            "retrieval_ms": {"p50": quantile(times, .5), "p95": quantile(times, .95),
                             "p99": quantile(times, .99)},
        }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({scope: summary["groups"][scope]["candidate_coverage"]
                      for scope in ("scoped", "unscoped")}))


if __name__ == "__main__":
    main()
