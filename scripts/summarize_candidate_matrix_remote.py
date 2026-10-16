#!/usr/bin/env python3
"""Summarize completed OpenRouter matrix reports without changing frozen inputs."""

import argparse
import json
from pathlib import Path


MODELS = ("typesafe/jev-1.13", "openai/gpt-6-luna-decisions")
VARIANTS = ("plain", "conditional", "structured")


def quantile(values: list[float], fraction: float) -> float | None:
    if not values:
        return None
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--split", choices=("development", "calibration"), required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    groups = {}
    total_cost = 0.0
    total_calls = 0
    for size in (100, 200, 2000):
        for scope in ("scoped", "unscoped"):
            for cap in (5, 10, 15, 20):
                for variant in VARIANTS:
                    path = (args.root / str(size) / "remote" / args.split /
                            f"{scope}-{cap}-{variant}-report.json")
                    report = json.loads(path.read_text())
                    if report["incomplete_tasks"] or report["uncertain_tasks"]:
                        raise ValueError(f"Incomplete report: {path}")
                    total_cost += report["reported_or_reserved_cost_usd"]
                    total_calls += report["calls"]
                    for model in MODELS:
                        rows = [row for row in report["results"] if row["requested_model"] == model]
                        if len(rows) * len(MODELS) != report["cases"] or any(
                                row["status"] not in ("complete", "skipped_empty") for row in rows):
                            raise ValueError(f"Missing or failed result: {path} {model}")
                        positive = [row for row in rows if row["expected_ids"]]
                        negative = [row for row in rows if not row["expected_ids"]]
                        covered = [row for row in positive if set(row["expected_ids"]) &
                                   set(row["candidate_ids"])]
                        decisions = [row for row in rows if row["status"] == "complete"]
                        latencies = [row["elapsed_ms"] for row in decisions]
                        groups[f"{size}/{scope}/{cap}/{variant}/{model}"] = {
                            "positive_exact": sum(row["correct"] for row in positive),
                            "positive_cases": len(positive),
                            "candidate_covered": len(covered),
                            "correct_when_covered": sum(row["correct"] for row in covered),
                            "negative_empty": sum(row["correct"] for row in negative),
                            "negative_cases": len(negative),
                            "decisions": len(decisions),
                            "decision_ms": {"p50": quantile(latencies, .5),
                                            "p95": quantile(latencies, .95)},
                            "input_tokens": sum(row.get("input_tokens") or 0 for row in decisions),
                            "output_tokens": sum(row.get("output_tokens") or 0 for row in decisions),
                            "reported_cost_usd": sum(row.get("reported_cost_usd") or 0
                                                     for row in decisions),
                        }
    summary = {"kind": "candidate_matrix_remote_summary", "synthetic_only": True,
               "split": args.split, "cells": len(groups), "calls": total_calls,
               "reported_or_reserved_cost_usd": total_cost, "groups": groups}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: summary[key] for key in ("split", "cells", "calls",
                                                   "reported_or_reserved_cost_usd")}))


if __name__ == "__main__":
    main()
