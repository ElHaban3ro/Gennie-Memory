#!/usr/bin/env python3
"""Validate and summarize a completed synthetic remote seed campaign."""

import argparse
import hashlib
import json
import math
from collections import defaultdict
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def read_jsonl(path: Path) -> dict[str, dict]:
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    indexed = {row["id"]: row for row in rows}
    if len(indexed) != len(rows):
        raise ValueError(f"Duplicate case ID in {path}")
    return indexed


def quantile(values: list[float], fraction: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    position = (len(ordered) - 1) * fraction
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def tally_pair(left: dict[str, bool], right: dict[str, bool]) -> dict[str, int]:
    if set(left) != set(right):
        raise ValueError("Paired comparisons require identical case IDs")
    result = {"both": 0, "left_only": 0, "right_only": 0, "neither": 0}
    for case_id in left:
        key = ("both" if left[case_id] and right[case_id] else
               "left_only" if left[case_id] else
               "right_only" if right[case_id] else "neither")
        result[key] += 1
    return result


def summarize(plan_path: Path, decisions_path: Path,
              seed_paths: list[tuple[str, Path]],
              local_paths: list[tuple[str, Path]]) -> dict:
    plan = json.loads(plan_path.read_text())
    checksum = plan.pop("plan_sha256")
    if hashlib.sha256(canonical(plan)).hexdigest() != checksum or plan["synthetic_only"] is not True:
        raise ValueError("Invalid synthetic campaign plan")
    plan["plan_sha256"] = checksum
    cases = {case["case_id"]: case for case in plan["cases"]}
    models = plan["models"]
    if len(cases) != len(plan["cases"]) or set(dict(seed_paths)) != set(models):
        raise ValueError("Case or model list is incomplete")
    decisions = json.loads(decisions_path.read_text())
    if (decisions["plan_sha256"] != checksum or decisions["synthetic_only"] is not True or
        decisions["incomplete_tasks"] or decisions["uncertain_tasks"] or
        decisions["cases"] != len(cases) * len(models) or
        decisions["calls"] != plan["planned_calls"]):
        raise ValueError("Decision report is not a complete matching campaign")
    indexed_decisions = {}
    for row in decisions["results"]:
        key = (row["requested_model"], row["case_id"])
        if key in indexed_decisions or key[0] not in models or key[1] not in cases:
            raise ValueError("Duplicate or unexpected decision")
        case = cases[key[1]]
        candidates = [item["memory_id"] for item in case["candidates"]]
        if row["candidate_ids"] != candidates:
            raise ValueError(f"Candidate mismatch: {key}")
        status = row["status"]
        if status == "skipped_empty":
            if candidates or row["chosen_id"] != "none":
                raise ValueError(f"Invalid empty result: {key}")
        elif status == "complete":
            if not candidates or row["chosen_id"] not in (*candidates, "none"):
                raise ValueError(f"Invalid completed choice: {key}")
            correct = ([row["chosen_id"]] == case["expected_ids"] if row["chosen_id"] != "none"
                       else not case["expected_ids"])
            if row["correct"] != correct:
                raise ValueError(f"Incorrect recorded label: {key}")
            if any(not isinstance(row.get(field), (int, float)) or
                   not math.isfinite(row[field]) or row[field] < 0
                   for field in ("elapsed_ms", "reported_cost_usd")):
                raise ValueError(f"Invalid provider measurement: {key}")
        elif status != "failed":
            raise ValueError(f"Unknown decision status: {key}")
        indexed_decisions[key] = row
    if len(indexed_decisions) != len(cases) * len(models):
        raise ValueError("Missing decision")

    metrics = {}
    exact_by_name = {}
    paths = dict(seed_paths)
    for model in models:
        seeds = read_jsonl(paths[model])
        if set(seeds) != set(cases):
            raise ValueError(f"Incomplete seed results: {model}")
        exact = {}
        segments = defaultdict(lambda: {"cases": 0, "exact": 0})
        counts = {"positive_exact": 0, "negative_empty": 0, "complete_cases": 0,
                  "fact_fidelity": 0, "condition_fidelity": 0,
                  "source_fidelity": 0, "forbidden_leaks": 0}
        statuses = defaultdict(int)
        latencies = []
        composition = []
        tokens = 0
        reported_cost = 0.0
        served = defaultdict(int)
        for case_id, case in cases.items():
            row = indexed_decisions[(model, case_id)]
            result = seeds[case_id]
            if (result["segment"] != case["segment"] or
                result["expected_ids"] != case["expected_ids"] or
                result["label_source"] != "deterministic_fixture_rule"):
                raise ValueError(f"Seed label mismatch: {model}/{case_id}")
            response = result["response"]
            selected = response["memories"]
            selected_ids = [item["memory_id"] for item in selected]
            if row["status"] == "failed":
                if response["status"] != "degraded" or selected_ids:
                    raise ValueError(f"Provider failure was hidden: {model}/{case_id}")
            else:
                chosen = row["chosen_id"]
                if (response["status"] != "complete" or
                    selected_ids != ([] if chosen == "none" else [chosen])):
                    raise ValueError(f"Seed composition changed the choice: {model}/{case_id}")
            forbidden = set(result["forbidden_ids"])
            counts["forbidden_leaks"] += bool(forbidden & set(selected_ids))
            good = response["status"] == "complete" and selected_ids == case["expected_ids"]
            exact[case_id] = good
            segment = segments[case["segment"]]
            segment["cases"] += 1
            segment["exact"] += good
            counts["complete_cases"] += response["status"] == "complete"
            if case["expected_ids"]:
                counts["positive_exact"] += good
                if good:
                    text = response["seed_text"] or ""
                    label = result["expected_fact"]
                    condition = result["expected_condition"]
                    source = result["expected_source"]
                    memory = selected[0]
                    counts["fact_fidelity"] += label in text
                    counts["condition_fidelity"] += (memory.get("conditions") == condition and
                                                     condition in text)
                    counts["source_fidelity"] += (memory.get("source") == source and
                                                  source["ref"] in text)
            else:
                counts["negative_empty"] += good
            statuses[row["status"]] += 1
            composition.append(result["elapsed_ms"])
            if row["status"] == "complete":
                latencies.append(row["elapsed_ms"])
                tokens += row["input_tokens"] or 0
                reported_cost += row["reported_cost_usd"]
                served[row["served_model"]] += 1
        if counts["forbidden_leaks"]:
            raise ValueError(f"Forbidden memory leaked: {model}")
        exact_by_name[model] = exact
        metrics[model] = {
            **counts, "cases": len(cases), "exact": sum(exact.values()),
            "decision_status": dict(sorted(statuses.items())),
            "input_tokens_reported": tokens,
            "provider_cost_reported_usd": round(reported_cost, 9),
            "provider_cost_or_reserved_usd": round(
                reported_cost + statuses["failed"] * plan["reserve_per_call_usd"], 9),
            "served_models": dict(sorted(served.items())),
            "provider_latency_ms": {f"p{int(q*100)}": quantile(latencies, q)
                                    for q in (.5, .95, .99)},
            "composition_latency_ms": {f"p{int(q*100)}": quantile(composition, q)
                                       for q in (.5, .95)},
            "segments": dict(sorted(segments.items())),
            "seed_results_sha256": sha256(paths[model]),
        }
    for name, path in local_paths:
        report = json.loads(path.read_text())
        rows = report["individual_results"]
        indexed = {row["id"]: row for row in rows}
        if len(indexed) != len(rows) or set(indexed) != set(cases):
            raise ValueError(f"Local report does not match remote cases: {name}")
        exact_by_name[name] = {case_id: bool(indexed[case_id]["exact"]) for case_id in cases}
    paired = {}
    names = list(exact_by_name)
    for i, left in enumerate(names):
        for right in names[i + 1:]:
            paired[f"{left}__vs__{right}"] = tally_pair(exact_by_name[left], exact_by_name[right])
    return {
        "version": 1, "kind": f"v5_{plan['split']}_remote_seed_campaign",
        "plan_sha256": checksum, "plan_file_sha256": sha256(plan_path),
        "decision_report_sha256": sha256(decisions_path),
        "cases": len(cases), "models": metrics, "paired": paired,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--decisions", type=Path, required=True)
    parser.add_argument("--seed", nargs=2, metavar=("MODEL", "PATH"), action="append", required=True)
    parser.add_argument("--local", nargs=2, metavar=("NAME", "PATH"), action="append", default=[])
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = summarize(args.plan, args.decisions,
                       [(model, Path(path)) for model, path in args.seed],
                       [(name, Path(path)) for name, path in args.local])
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({name: {"exact": value["exact"], "failed": value["decision_status"].get("failed", 0)}
                      for name, value in result["models"].items()}))


if __name__ == "__main__":
    main()
