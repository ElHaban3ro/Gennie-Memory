#!/usr/bin/env python3
"""Validate and summarize the paired general-memory final comparison."""

import argparse
import json
import math
from collections import defaultdict
from pathlib import Path


MODELS = ("typesafe/jev-1.13", "openai/gpt-6-luna-decisions")


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    index = (len(ordered) - 1) * fraction
    low = int(index)
    high = min(low + 1, len(ordered) - 1)
    return ordered[low] + (ordered[high] - ordered[low]) * (index - low)


def wilson(successes: int, total: int) -> list[float]:
    z = 1.959963984540054
    rate = successes / total
    denominator = 1 + z * z / total
    center = (rate + z * z / (2 * total)) / denominator
    radius = z * math.sqrt(rate * (1 - rate) / total + z * z / (4 * total * total)) / denominator
    return [center - radius, center + radius]


def summarize(rows: list[dict], query_map: dict[str, dict]) -> dict:
    if len(rows) != len(query_map) or {row["case_id"] for row in rows} != set(query_map):
        raise ValueError("Final cases are incomplete or duplicated")
    positive = [row for row in rows if row["expected_ids"]]
    negative = [row for row in rows if not row["expected_ids"]]
    if len(positive) != 460 or len(negative) != 80:
        raise ValueError("Final label counts changed")
    groups = {}
    for scope in ("user", "project", "conversation", "agent"):
        scope_rows = [row for row in rows if query_map[row["case_id"]]["request"]["search"]["scope_kind"] == scope]
        scope_positive = [row for row in scope_rows if row["expected_ids"]]
        scope_negative = [row for row in scope_rows if not row["expected_ids"]]
        groups[scope] = {"positive_exact": sum(row["correct"] for row in scope_positive),
                         "positive_cases": len(scope_positive),
                         "negative_empty": sum(row["correct"] for row in scope_negative),
                         "negative_cases": len(scope_negative)}
    times = [row["elapsed_ms"] for row in rows if row["elapsed_ms"] > 0]
    positive_exact = sum(row["correct"] for row in positive)
    return {"positive_exact": positive_exact, "positive_cases": len(positive),
            "positive_rate": positive_exact / len(positive),
            "positive_wilson_95": wilson(positive_exact, len(positive)),
            "candidate_covered": sum(row["expected_ids"][0] in row["candidate_ids"] for row in positive),
            "positive_abstentions": sum(row["chosen_id"] == "none" for row in positive),
            "negative_empty": sum(row["correct"] for row in negative),
            "negative_cases": len(negative),
            "decision_failures": sum(row.get("status") == "failed" for row in rows),
            "decision_ms": {"p50": percentile(times, .5), "p95": percentile(times, .95)},
            "by_scope": groups}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--retrieval", type=Path, required=True)
    parser.add_argument("--laya-journal", type=Path, required=True)
    parser.add_argument("--remote-report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    queries = [json.loads(line) for line in args.queries.read_text().splitlines() if line.strip()]
    query_map = {row["id"]: row for row in queries}
    retrieval = json.loads(args.retrieval.read_text())
    local_events = [json.loads(line) for line in args.laya_journal.read_text().splitlines() if line.strip()]
    remote = json.loads(args.remote_report.read_text())
    if (len(queries) != 540 or len(query_map) != 540 or
            remote["incomplete_tasks"] or remote["uncertain_tasks"] or len(remote["results"]) != 1080 or
            local_events[0]["phase"] != "manifest" or
            local_events[0]["value"]["queries_sha256"] != retrieval["split_sha256"]["final"]["queries"] or
            remote["plan_sha256"] is None):
        raise ValueError("Final inputs or manifests are incomplete")
    expected_candidates = {}
    normalized_remote = []
    for row in remote["results"]:
        if row["status"] not in ("complete", "skipped_empty", "failed"):
            raise ValueError("Unknown remote decision status")
        case = query_map.get(row["case_id"])
        if case is None or row.get("expected_ids", case["expected_ids"]) != case["expected_ids"]:
            raise ValueError("Remote label mismatch")
        prior = expected_candidates.setdefault(row["case_id"], row["candidate_ids"])
        if row["candidate_ids"] != prior:
            raise ValueError("Models saw different candidates")
        normalized_remote.append({**row, "expected_ids": case["expected_ids"],
                                  "chosen_id": row.get("chosen_id", "failed"),
                                  "correct": row.get("correct", False),
                                  "elapsed_ms": row.get("elapsed_ms", 0.0)})
    local_rows = [event["value"] for event in local_events[1:] if event["phase"] == "decision"]
    if len(local_rows) != 1080:
        raise ValueError("Incomplete local final decisions")
    groups = {}
    for variant in ("direct", "grounded"):
        rows = [row for row in local_rows if row["variant"] == variant and row["cap"] == 20]
        if any(row["candidate_ids"] != expected_candidates[row["case_id"]] for row in rows):
            raise ValueError("Laya saw different candidates")
        groups[f"laya/{variant}"] = summarize(rows, query_map)
    for model in MODELS:
        rows = [row for row in normalized_remote if row["requested_model"] == model]
        group = summarize(rows, query_map)
        group["reported_cost_usd"] = sum(row.get("reported_cost_usd") or 0 for row in rows)
        group["served_models"] = sorted({row["served_model"] for row in rows if row.get("served_model")})
        groups[model] = group
    output = {"kind": "general_memory_v1_final_comparison", "synthetic_only": True,
              "cohort": "Four final entities per scope; no development entity reused",
              "candidate_cap": 20, "prompt_variant_remote": "direct",
              "remote_attempted_calls": remote["calls"],
              "remote_reported_or_reserved_cost_usd": remote["reported_or_reserved_cost_usd"],
              "groups": groups}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: {"positive_exact": value["positive_exact"],
                             "negative_empty": value["negative_empty"]}
                      for key, value in groups.items()}, ensure_ascii=False))


if __name__ == "__main__":
    main()
