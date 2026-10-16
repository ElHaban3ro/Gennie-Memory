#!/usr/bin/env python3
"""Compare paired Decisions choices and a conditional Jev fallback."""

import argparse
import json
from collections import defaultdict
from pathlib import Path

if __package__:
    from .prepare_general_memory_followup import read
else:
    from prepare_general_memory_followup import read


OPENAI = "openai/gpt-6-luna-decisions"
JEV = "typesafe/jev-1.13"
CASCADE = "iromem/cascade-decisions-jev"


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("No latencies")
    index = (len(ordered) - 1) * fraction
    low = int(index)
    high = min(low + 1, len(ordered) - 1)
    return ordered[low] + (ordered[high] - ordered[low]) * (index - low)


def collect(report: Path, model: str) -> dict[str, dict]:
    data = json.loads(report.read_text())
    if data["incomplete_tasks"] or data["uncertain_tasks"]:
        raise ValueError("Report has incomplete calls")
    rows = [row for row in data["results"] if row["requested_model"] == model]
    if any(row["status"] != "complete" for row in rows):
        raise ValueError("Report has failed decisions")
    by_id = {row["case_id"]: row for row in rows}
    if len(by_id) != len(rows):
        raise ValueError("Duplicate decision")
    return by_id


def summarize(queries: Path, results: Path, primary: Path, fallback: Path | None = None) -> dict:
    cases, screens = read(queries), read(results)
    decisions = collect(primary, OPENAI)
    jev = collect(fallback, JEV) if fallback else {}
    fallback_cap = json.loads(fallback.read_text())["max_candidates"] if fallback else 20
    if len(cases) != len(screens) or set(decisions) != {case["id"] for case in cases}:
        raise ValueError("Primary coverage mismatch")
    expected_fallback = {cid for cid, row in decisions.items() if row["chosen_id"] == "none"}
    if fallback and set(jev) != expected_fallback:
        raise ValueError("Fallback coverage mismatch")
    statistics = defaultdict(lambda: {"positive": 0, "positive_correct": 0,
                                      "negative": 0, "negative_empty": 0})
    primary_times, cascade_times = [], []
    changed = 0
    for case, screen in zip(cases, screens):
        cid = case["id"]
        if screen["id"] != cid or screen["expected_ids"] != case["expected_ids"]:
            raise ValueError("Retrieval label mismatch")
        row = decisions[cid]
        ids = [item["memory_id"] for item in screen["response"]["candidates"][:20]]
        if row["candidate_ids"] != ids or row["expected_ids"] != case["expected_ids"]:
            raise ValueError("Primary saw different candidates")
        second = jev.get(cid)
        if second and (second["candidate_ids"] != ids[:fallback_cap] or
                       second["expected_ids"] != case["expected_ids"]):
            raise ValueError("Fallback saw different candidates")
        chosen = second["chosen_id"] if second and second["chosen_id"] != "none" else row["chosen_id"]
        if chosen != row["chosen_id"]:
            changed += 1
        scope = case["request"].get("search", case["request"])["scope_kind"]
        for name, selected in (("primary", row["chosen_id"]), ("cascade", chosen)):
            group = statistics[f"{name}/{scope}"]
            if case["expected_ids"]:
                group["positive"] += 1
                group["positive_correct"] += selected in case["expected_ids"]
            else:
                group["negative"] += 1
                group["negative_empty"] += selected == "none"
        primary_times.append(row["elapsed_ms"])
        cascade_times.append(row["elapsed_ms"] + (second["elapsed_ms"] if second else 0))
    groups = {}
    for name, values in (("primary", primary_times), ("cascade", cascade_times)):
        scope_groups = [statistics[f"{name}/{scope}"] for scope in ("user", "project", "conversation", "agent")]
        groups[name] = {"positive_correct": sum(g["positive_correct"] for g in scope_groups),
                        "positive": sum(g["positive"] for g in scope_groups),
                        "negative_empty": sum(g["negative_empty"] for g in scope_groups),
                        "negative": sum(g["negative"] for g in scope_groups),
                        "decision_p50_ms": percentile(values, .5),
                        "decision_p95_ms": percentile(values, .95),
                        "by_scope": {scope: statistics[f"{name}/{scope}"] for scope in
                                     ("user", "project", "conversation", "agent")}}
    return {"kind": "general_memory_followup", "synthetic_only": True,
            "cases": len(cases), "fallback_calls": len(jev), "cascade_choices_changed": changed,
            "groups": groups,
            "reported_cost_usd": sum(row.get("reported_cost_usd") or 0 for row in
                                     [*decisions.values(), *jev.values()])}


def merged_decisions(primary: Path, fallback: Path) -> dict:
    decisions, jev = collect(primary, OPENAI), collect(fallback, JEV)
    if set(jev) != {cid for cid, row in decisions.items() if row["chosen_id"] == "none"}:
        raise ValueError("Fallback is incomplete")
    rows = []
    for cid, first in decisions.items():
        second = jev.get(cid)
        if second and (second["candidate_ids"] != first["candidate_ids"] or
                       second["expected_ids"] != first["expected_ids"]):
            raise ValueError("Fallback candidates or labels differ")
        selected = second if second and second["chosen_id"] != "none" else first
        chosen = selected["chosen_id"]
        rows.append({"case_id": cid, "requested_model": CASCADE,
                     "served_model": selected["served_model"], "status": "complete",
                     "chosen_id": chosen, "candidate_ids": first["candidate_ids"],
                     "expected_ids": first["expected_ids"],
                     "correct": chosen in first["expected_ids"] if first["expected_ids"] else chosen == "none",
                     "elapsed_ms": first["elapsed_ms"] + (second["elapsed_ms"] if second else 0),
                     "reported_cost_usd": (first.get("reported_cost_usd") or 0) +
                                          ((second.get("reported_cost_usd") or 0) if second else 0),
                     "decision_chain": [OPENAI, JEV] if second else [OPENAI]})
    return {"method": "openrouter_decisions_choice_probe", "synthetic_only": True,
            "max_candidates": 20, "cases": len(rows), "planned_tasks": len(rows),
            "incomplete_tasks": 0, "uncertain_tasks": 0, "results": rows}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--primary-report", type=Path, required=True)
    parser.add_argument("--fallback-report", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--merged-report", type=Path)
    args = parser.parse_args()
    summary = summarize(args.queries, args.results, args.primary_report, args.fallback_report)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    if args.merged_report:
        if not args.fallback_report:
            parser.error("--merged-report requires --fallback-report")
        args.merged_report.parent.mkdir(parents=True, exist_ok=True)
        args.merged_report.write_text(json.dumps(merged_decisions(args.primary_report,
                                                                  args.fallback_report),
                                                 ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"groups": {name: {key: group[key] for key in
                                       ("positive_correct", "positive", "negative_empty", "negative")}
                                  for name, group in summary["groups"].items()},
                      "fallback_calls": summary["fallback_calls"]}, ensure_ascii=False))
