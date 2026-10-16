#!/usr/bin/env python3
"""Summarize the frozen final classifier report by model and label."""

import argparse
import json
import math
import random
from collections import defaultdict
from pathlib import Path


def quantile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def wilson_95(successes: int, total: int) -> list[float]:
    z = 1.959963984540054
    rate = successes / total
    denominator = 1 + z * z / total
    center = (rate + z * z / (2 * total)) / denominator
    radius = (z * math.sqrt(rate * (1 - rate) / total + z * z / (4 * total * total)) /
              denominator)
    return [max(0.0, center - radius), min(1.0, center + radius)]


def entity_cluster_bootstrap_95(rows: list[dict]) -> list[float]:
    by_entity = defaultdict(list)
    for row in rows:
        by_entity["_".join(row["case_id"].split("_")[1:3])].append(bool(row["correct"]))
    if len(by_entity) != 20 or any(len(group) != 20 for group in by_entity.values()):
        raise ValueError("Final positive entity clusters changed")
    rates = [sum(group) / len(group) for _entity, group in sorted(by_entity.items())]
    rng = random.Random(20261015)
    samples = sorted(sum(rates[rng.randrange(len(rates))] for _ in rates) / len(rates)
                     for _ in range(20_000))
    return [samples[499], samples[19_499]]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = json.loads(args.report.read_text())
    if report["incomplete_tasks"] or report["uncertain_tasks"] or report["cases"] != 960:
        raise ValueError("Final campaign is incomplete")
    groups = {}
    for model in ("typesafe/jev-1.13", "openai/gpt-6-luna-decisions"):
        rows = [row for row in report["results"] if row["requested_model"] == model]
        if len(rows) != 480 or len({row["case_id"] for row in rows}) != 480 or any(
                row["status"] not in ("complete", "skipped_empty") for row in rows):
            raise ValueError(f"Final model results incomplete: {model}")
        positive = [row for row in rows if row["expected_ids"]]
        negative = [row for row in rows if not row["expected_ids"]]
        if len(positive) != 400 or len(negative) != 80:
            raise ValueError("Final label counts changed")
        decisions = [row for row in rows if row["status"] == "complete"]
        times = [row["elapsed_ms"] for row in decisions]
        positive_exact = sum(row["correct"] for row in positive)
        groups[model] = {
            "positive_exact": positive_exact,
            "positive_cases": len(positive),
            "positive_rate": positive_exact / len(positive),
            "positive_wilson_95": wilson_95(positive_exact, len(positive)),
            "positive_entity_cluster_bootstrap_95": entity_cluster_bootstrap_95(positive),
            "candidate_covered": sum(bool(set(row["expected_ids"]) &
                                          set(row["candidate_ids"])) for row in positive),
            "negative_empty": sum(row["correct"] for row in negative),
            "negative_cases": len(negative),
            "decision_calls": len(decisions),
            "decision_ms": {"p50": quantile(times, .5), "p95": quantile(times, .95),
                            "p99": quantile(times, .99)},
            "input_tokens": sum(row.get("input_tokens") or 0 for row in decisions),
            "output_tokens": sum(row.get("output_tokens") or 0 for row in decisions),
            "reported_cost_usd": sum(row.get("reported_cost_usd") or 0 for row in decisions),
        }
    summary = {"kind": "candidate_matrix_final_classification", "synthetic_only": True,
               "plan_sha256": report["plan_sha256"], "calls": report["calls"],
               "reported_or_reserved_cost_usd": report["reported_or_reserved_cost_usd"],
               "groups": groups}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({model: {key: value[key] for key in ("positive_exact", "negative_empty")}
                      for model, value in groups.items()}))


if __name__ == "__main__":
    main()
