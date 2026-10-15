#!/usr/bin/env python3
"""Compare RRF fusion to its two input rankings on the same synthetic cases."""

import argparse
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--text-results", type=Path, required=True)
    parser.add_argument("--semantic-report", type=Path, required=True)
    parser.add_argument("--rrf-k", type=int, default=60)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.rrf_k < 1:
        parser.error("rrf-k must be positive")
    text_rows = {r["id"]: r for r in map(json.loads, args.text_results.read_text().splitlines())}
    semantic = json.loads(args.semantic_report.read_text())
    if semantic["method"] != "laya_encoder_exact_cosine_pilot":
        raise ValueError("Unexpected semantic method")
    rows = []
    for item in semantic["individual_results"]:
        text = text_rows[item["id"]]
        text_ids = [c["memory_id"] for c in text["response"]["candidates"]]
        semantic_ids = item["top_ids"]
        scores = {}
        for ranking in (text_ids, semantic_ids):
            for position, identity in enumerate(ranking, 1):
                scores[identity] = scores.get(identity, 0.0) + 1.0 / (args.rrf_k + position)
        text_rank = {identity: position for position, identity in enumerate(text_ids, 1)}
        semantic_rank = {identity: position for position, identity in enumerate(semantic_ids, 1)}
        fused = sorted(scores, key=lambda identity: (-scores[identity],
                                                    text_rank.get(identity, 10**9),
                                                    semantic_rank.get(identity, 10**9), identity))
        expected = text["expected_ids"]
        rows.append({"id": item["id"], "expected_ids": expected,
                     "text_top": text_ids[:1], "semantic_top": semantic_ids[:1],
                     "fused_top": fused[:1], "fused_ids": fused,
                     "forbidden_leak": bool(set(text["forbidden_ids"]) & set(fused))})
    positive = [row for row in rows if row["expected_ids"]]
    negative = [row for row in rows if not row["expected_ids"]]
    def accuracy(key: str) -> float | None:
        return sum(row[key] == row["expected_ids"][:1] for row in positive) / len(positive) if positive else None
    report = {"method": "rrf_pilot", "rrf_k": args.rrf_k, "cases": len(rows),
              "positive_cases": len(positive), "negative_cases": len(negative),
              "text_top_1": accuracy("text_top"),
              "semantic_top_1": accuracy("semantic_top"),
              "fused_top_1": accuracy("fused_top"),
              "fused_recall": sum(bool(set(row["expected_ids"]) & set(row["fused_ids"])) for row in positive) / len(positive) if positive else None,
              "fused_negative_empty_rate": sum(not row["fused_ids"] for row in negative) / len(negative) if negative else None,
              "forbidden_leak_count": sum(row["forbidden_leak"] for row in rows)}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False))


if __name__ == "__main__":
    main()
