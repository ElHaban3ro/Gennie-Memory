#!/usr/bin/env python3
"""Analyze all attainable thresholds of frozen synthetic pairwise scores."""

import argparse
import json
import math
from collections import defaultdict
from pathlib import Path


def analyze(queries: list[dict], report: dict) -> dict:
    split = queries[0]["split"] if queries else None
    if (split not in ("development", "calibration") or report.get("split") != split or
        report.get("method") != "integrated_hybrid_laya_pairwise_conditions_seed_campaign"):
        raise ValueError("Expected a development or calibration conditioned pairwise report")
    labels = {case["id"]: case["expected_ids"] for case in queries}
    if len(labels) != len(queries) or any(case["split"] != split for case in queries):
        raise ValueError("Queries must be unique cases from one permitted split")
    rows = report.get("individual_results")
    if not isinstance(rows, list) or len(rows) != len(queries) or {row["id"] for row in rows} != set(labels):
        raise ValueError("Report does not match queries")
    events = defaultdict(lambda: [0, 0])
    initial_positive = initial_negative = 0
    ranking_correct = 0
    for row in rows:
        scores = row.get("candidate_scores")
        if not isinstance(scores, dict) or any(
            isinstance(value, bool) or not isinstance(value, (int, float)) or
            not math.isfinite(value) or not 0 <= value <= 1 for value in scores.values()
        ):
            raise ValueError(f"Invalid candidate scores: {row['id']}")
        expected = labels[row["id"]]
        if not scores:
            if not expected:
                initial_negative += 1
            continue
        top = max(scores, key=scores.get)
        top_score = scores[top]
        if expected:
            if top in expected:
                initial_positive += 1
                ranking_correct += 1
                events[top_score][0] -= 1
        else:
            events[top_score][1] += 1

    def at(threshold: float) -> dict:
        positive, negative = initial_positive, initial_negative
        for score, (positive_delta, negative_delta) in events.items():
            if score < threshold:
                positive += positive_delta
                negative += negative_delta
        return {"threshold": threshold, "positive_exact": positive,
                "negative_empty": negative, "total_exact": positive + negative}

    best = at(0.0)
    positive, negative = initial_positive, initial_negative
    for score, (positive_delta, negative_delta) in sorted(events.items()):
        positive += positive_delta
        negative += negative_delta
        threshold = math.nextafter(score, math.inf)
        if threshold <= 1 and positive + negative > best["total_exact"]:
            best = {"threshold": threshold, "positive_exact": positive,
                    "negative_empty": negative, "total_exact": positive + negative}
    return {"kind": f"exploratory_{split}_threshold_analysis",
            "cases": len(queries), "positive": sum(bool(value) for value in labels.values()),
            "negative": sum(not value for value in labels.values()),
            "correct_positive_top_rank": ranking_correct,
            "distinct_score_breakpoints": len(events),
            "fixed_thresholds": [at(value) for value in (0.0, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99, 1.0)],
            "best_over_all_score_breakpoints": best,
            "production_threshold_selected": False}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    queries = [json.loads(line) for line in args.queries.read_text().splitlines() if line.strip()]
    result = analyze(queries, json.loads(args.report.read_text()))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": result["cases"],
                      "best": result["best_over_all_score_breakpoints"]},
                     ensure_ascii=False))


if __name__ == "__main__":
    main()
