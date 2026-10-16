#!/usr/bin/env python3
"""Verify final seed composition, labels, provenance, and measured latency."""

import argparse
import hashlib
import json
from pathlib import Path


def quantile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def summarize(path: Path) -> dict:
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    if len(rows) != 480 or len({row["id"] for row in rows}) != 480:
        raise ValueError(f"Invalid final seed count: {path}")
    positive = [row for row in rows if row["expected_ids"]]
    negative = [row for row in rows if not row["expected_ids"]]
    if len(positive) != 400 or len(negative) != 80:
        raise ValueError("Final seed labels changed")
    exact = []
    faithful = []
    for row in positive:
        response = row["response"]
        memories = response["memories"]
        correct = [item["memory_id"] for item in memories] == row["expected_ids"]
        exact.append(correct)
        if correct:
            if len(memories) != 1 or response["seed_text"] is None:
                raise ValueError(f"Correct memory missing from seed: {row['id']}")
            faithful.append(row["expected_fact"] in response["seed_text"] and
                            memories[0]["conditions"] == row["expected_condition"] and
                            memories[0]["source"] == row["expected_source"])
    times = [row["elapsed_ms"] for row in rows]
    return {"results_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "positive_exact": sum(exact), "positive_cases": len(positive),
            "faithful_when_exact": sum(faithful), "exact_cases": len(faithful),
            "negative_empty": sum(not row["response"]["memories"] for row in negative),
            "negative_cases": len(negative),
            "composition_ms": {"p50": quantile(times, .5),
                               "p95": quantile(times, .95),
                               "p99": quantile(times, .99)}}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    summary = {"kind": "candidate_matrix_final_seeds", "synthetic_only": True,
               "groups": {"typesafe/jev-1.13": summarize(args.root / "jev-seeds-results.jsonl"),
                          "openai/gpt-6-luna-decisions": summarize(
                              args.root / "openai-seeds-results.jsonl")}}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({model: {key: value[key] for key in ("positive_exact", "negative_empty",
                                                         "faithful_when_exact")}
                      for model, value in summary["groups"].items()}))


if __name__ == "__main__":
    main()
