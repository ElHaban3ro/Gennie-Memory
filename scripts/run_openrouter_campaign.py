#!/usr/bin/env python3
"""Freeze and run a bounded synthetic Decisions comparison with an append-only journal."""

import argparse
import hashlib
import json
import math
import os
import time
from pathlib import Path

if __package__:
    from . import probe_openrouter_decisions as probe
else:
    import probe_openrouter_decisions as probe


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def read_jsonl(path: Path) -> dict[str, dict]:
    return probe.read_jsonl(path)


def positive_number(value: str) -> float:
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise argparse.ArgumentTypeError("Must be a positive finite number")
    return number


def prepare(args: argparse.Namespace) -> dict:
    if not 1 <= args.max_candidates <= 8 or args.max_calls < 1:
        raise ValueError("max-candidates must be 1..8 and max-calls must be positive")
    if not args.model or len(set(args.model)) != len(args.model):
        raise ValueError("Models must be unique")
    queries = read_jsonl(args.queries)
    results = read_jsonl(args.results)
    if not queries or set(queries) != set(results):
        raise ValueError("Query and retrieval IDs must match exactly")
    cases = []
    for case_id in sorted(queries):
        case, result = queries[case_id], results[case_id]
        if case["split"] != args.split or case["label_source"] != "deterministic_fixture_rule":
            raise ValueError(f"Case is outside the frozen synthetic split: {case_id}")
        for field in ("split", "segment", "expected_ids", "forbidden_ids", "label_source"):
            if result[field] != case[field]:
                raise ValueError(f"Retrieval label mismatch for {case_id}: {field}")
        for field in ("expected_condition", "expected_source"):
            if result.get(field) != case.get(field):
                raise ValueError(f"Retrieval label mismatch for {case_id}: {field}")
        response = result["response"]
        if response["status"] not in ("complete", "degraded") or response["method"] not in (
            "fts5", "fts5_semantic_rrf"
        ):
            raise ValueError(f"Unexpected retrieval result for {case_id}")
        search = probe.search_request(case)
        if not isinstance(search["query"], str) or not isinstance(search["scope_id"], str):
            raise ValueError(f"Invalid search request for {case_id}")
        candidates = response["candidates"][:args.max_candidates]
        identifiers = [candidate["memory_id"] for candidate in candidates]
        if "none" in identifiers or len(identifiers) != len(set(identifiers)):
            raise ValueError(f"Invalid candidate IDs for {case_id}")
        if set(identifiers) & set(case["forbidden_ids"]):
            raise ValueError(f"Forbidden candidate in {case_id}")
        if any(not isinstance(candidate["summary"], str) for candidate in candidates):
            raise ValueError(f"Invalid candidate summary for {case_id}")
        cases.append({
            "case_id": case_id, "segment": case["segment"],
            "expected_ids": case["expected_ids"],
            "state": {"user_prompt": search["query"], "scope": search["scope_id"]},
            "candidates": [{"memory_id": item["memory_id"], "summary": item["summary"]}
                           for item in candidates],
        })
    planned_calls = sum(bool(case["candidates"]) for case in cases) * len(args.model)
    if planned_calls > args.max_calls:
        raise ValueError(f"Plan needs {planned_calls} calls but max-calls is {args.max_calls}")
    if planned_calls * args.reserve_per_call_usd > args.max_cost_usd:
        raise ValueError("Reserved call cost exceeds max-cost-usd")
    plan = {
        "version": 1, "method": "openrouter_decisions_choice_probe",
        "synthetic_only": True, "split": args.split,
        "queries_sha256": digest(args.queries.read_bytes()),
        "results_sha256": digest(args.results.read_bytes()),
        "models": args.model, "max_candidates": args.max_candidates,
        "max_calls": args.max_calls, "max_seconds": args.max_seconds,
        "max_cost_usd": args.max_cost_usd,
        "reserve_per_call_usd": args.reserve_per_call_usd,
        "planned_calls": planned_calls, "cases": cases,
    }
    plan["plan_sha256"] = digest(canonical(plan))
    return plan


def tasks(plan: dict) -> list[tuple[str, dict]]:
    return [(model, case) for case in plan["cases"] for model in plan["models"]]


def load_journal(path: Path, plan: dict) -> dict[tuple[str, str], dict]:
    state = {}
    if not path.exists():
        return state
    valid = {(model, case["case_id"]) for model, case in tasks(plan)}
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        event = json.loads(line)
        if event["plan_sha256"] != plan["plan_sha256"]:
            raise ValueError("Journal belongs to a different plan")
        identity = (event["model"], event["case_id"])
        if identity not in valid:
            raise ValueError(f"Unknown journal task at line {line_number}")
        previous = state.get(identity)
        if event["phase"] == "started":
            if previous is not None:
                raise ValueError(f"Repeated started task at line {line_number}")
        elif event["phase"] == "finished":
            if previous is None or previous["phase"] != "started":
                raise ValueError(f"Finish without start at line {line_number}")
        elif event["phase"] == "skipped_empty":
            if previous is not None:
                raise ValueError(f"Repeated skipped task at line {line_number}")
        else:
            raise ValueError(f"Invalid journal phase at line {line_number}")
        state[identity] = event
    return state


def append_event(path: Path, event: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(event, ensure_ascii=False, separators=(",", ":")) + "\n")
        stream.flush()
        os.fsync(stream.fileno())


def usage_from_journal(path: Path, plan: dict) -> tuple[int, float, float]:
    calls = 0
    cost = 0.0
    active_seconds = 0.0
    if not path.exists():
        return calls, cost, active_seconds
    for line in path.read_text().splitlines():
        event = json.loads(line)
        if event["plan_sha256"] != plan["plan_sha256"]:
            raise ValueError("Journal belongs to a different plan")
        if event["phase"] == "started":
            calls += 1
            cost += plan["reserve_per_call_usd"]
        elif event["phase"] == "finished":
            reported = event["result"].get("reported_cost_usd")
            if isinstance(reported, (int, float)) and math.isfinite(reported) and reported >= 0:
                cost += reported - plan["reserve_per_call_usd"]
            active_seconds += event["result"].get("attempt_elapsed_ms", 0) / 1000
    return calls, cost, active_seconds


def run(plan: dict, journal: Path, call=probe.call) -> dict:
    state = load_journal(journal, plan)
    if any(event["phase"] == "started" for event in state.values()):
        raise ValueError("Uncertain in-flight call: resolve it as failed before resuming; never retry automatically")
    calls, estimated_cost, prior_active_seconds = usage_from_journal(journal, plan)
    key = probe.read_key()
    started_at = time.monotonic()
    for model, case in tasks(plan):
        identity = (model, case["case_id"])
        if identity in state:
            continue
        base = {"plan_sha256": plan["plan_sha256"], "model": model,
                "case_id": case["case_id"]}
        candidates = case["candidates"]
        if not candidates:
            event = {**base, "phase": "skipped_empty"}
            append_event(journal, event)
            state[identity] = event
            continue
        if (calls >= plan["max_calls"] or
            estimated_cost + plan["reserve_per_call_usd"] > plan["max_cost_usd"] or
            prior_active_seconds + time.monotonic() - started_at >= plan["max_seconds"]):
            break
        criteria = {item["memory_id"]: item["summary"] for item in candidates}
        criteria["none"] = "No candidate memory is relevant to this request."
        append_event(journal, {**base, "phase": "started"})
        calls += 1
        estimated_cost += plan["reserve_per_call_usd"]
        attempt_started = time.monotonic()
        try:
            result = call(key, model, case["state"], criteria)
            reported = result.get("reported_cost_usd")
            if isinstance(reported, bool) or not isinstance(reported, (int, float)) or (
                not math.isfinite(reported) or reported < 0
            ):
                raise ValueError("Provider returned no valid cost")
            estimated_cost += reported - plan["reserve_per_call_usd"]
            result.update(status="complete", case_id=case["case_id"],
                          expected_ids=case["expected_ids"],
                          candidate_ids=[item["memory_id"] for item in candidates],
                          correct=(result["chosen_id"] in case["expected_ids"]
                                   if case["expected_ids"] else result["chosen_id"] == "none"))
        except (RuntimeError, ValueError) as error:
            result = {"requested_model": model, "case_id": case["case_id"],
                      "status": "failed",
                      "candidate_ids": [item["memory_id"] for item in candidates],
                      "error": str(error)}
        result["attempt_elapsed_ms"] = (time.monotonic() - attempt_started) * 1000
        event = {**base, "phase": "finished", "result": result}
        append_event(journal, event)
        state[identity] = event
        if result["status"] == "failed":
            break
    return {"planned_tasks": len(tasks(plan)), "finished_tasks": len(state),
            "attempted_calls": calls, "reported_or_reserved_cost_usd": estimated_cost,
            "uncertain_calls": sum(event["phase"] == "started" for event in state.values())}


def resolve_uncertain(plan: dict, journal: Path, model: str, case_id: str) -> None:
    state = load_journal(journal, plan)
    identity = (model, case_id)
    if identity not in state or state[identity]["phase"] != "started":
        raise ValueError("Task has no uncertain in-flight call")
    case = next(case for case in plan["cases"] if case["case_id"] == case_id)
    append_event(journal, {
        "plan_sha256": plan["plan_sha256"], "model": model, "case_id": case_id,
        "phase": "finished",
        "result": {"requested_model": model, "case_id": case_id, "status": "failed",
                   "candidate_ids": [item["memory_id"] for item in case["candidates"]],
                   "attempt_elapsed_ms": 20000.0,
                   "error": "interrupted_call_outcome_unknown"},
    })


def export_report(plan: dict, journal: Path, allow_partial: bool = False) -> dict:
    state = load_journal(journal, plan)
    incomplete = len(tasks(plan)) - len(state)
    uncertain = sum(event["phase"] == "started" for event in state.values())
    if (incomplete or uncertain) and not allow_partial:
        raise ValueError("Campaign has incomplete or uncertain tasks")
    rows = []
    for model, case in tasks(plan):
        event = state.get((model, case["case_id"]))
        if event is None or event["phase"] == "started":
            continue
        if event["phase"] == "skipped_empty":
            rows.append({"requested_model": model, "case_id": case["case_id"],
                         "status": "skipped_empty", "chosen_id": "none",
                         "candidate_ids": [], "expected_ids": case["expected_ids"],
                         "correct": not case["expected_ids"],
                         "reported_cost_usd": 0.0, "reason": None})
        else:
            rows.append(event["result"])
    attempted, cost, _active_seconds = usage_from_journal(journal, plan)
    return {"method": plan["method"], "synthetic_only": True,
            "endpoint": probe.ENDPOINT, "max_candidates": plan["max_candidates"],
            "plan_sha256": plan["plan_sha256"], "calls": attempted,
            "planned_tasks": len(tasks(plan)), "incomplete_tasks": incomplete,
            "uncertain_tasks": uncertain,
            "reported_or_reserved_cost_usd": cost,
            "cases": len(rows), "results": rows}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    prep = sub.add_parser("prepare")
    prep.add_argument("--queries", type=Path, required=True)
    prep.add_argument("--results", type=Path, required=True)
    prep.add_argument("--split", choices=("development", "calibration", "final"), required=True)
    prep.add_argument("--model", action="append", choices=probe.MODELS, required=True)
    prep.add_argument("--max-candidates", type=int, default=4)
    prep.add_argument("--max-calls", type=int, required=True)
    prep.add_argument("--max-cost-usd", type=positive_number, required=True)
    prep.add_argument("--reserve-per-call-usd", type=positive_number, required=True)
    prep.add_argument("--max-seconds", type=positive_number, required=True)
    prep.add_argument("--output", type=Path, required=True)
    running = sub.add_parser("run")
    running.add_argument("--plan", type=Path, required=True)
    running.add_argument("--journal", type=Path, required=True)
    resolve = sub.add_parser("resolve-uncertain")
    resolve.add_argument("--plan", type=Path, required=True)
    resolve.add_argument("--journal", type=Path, required=True)
    resolve.add_argument("--model", choices=probe.MODELS, required=True)
    resolve.add_argument("--case-id", required=True)
    export = sub.add_parser("export")
    export.add_argument("--plan", type=Path, required=True)
    export.add_argument("--journal", type=Path, required=True)
    export.add_argument("--output", type=Path, required=True)
    export.add_argument("--partial", action="store_true",
                        help="Export terminal rows only, with explicit missing/uncertain counts")
    args = parser.parse_args()
    if args.command == "prepare":
        plan = prepare(args)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(plan, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({"plan_sha256": plan["plan_sha256"], "cases": len(plan["cases"]),
                          "planned_calls": plan["planned_calls"]}))
        return
    plan = json.loads(args.plan.read_text())
    actual = plan.pop("plan_sha256")
    if digest(canonical(plan)) != actual or plan["synthetic_only"] is not True:
        raise ValueError("Plan checksum or synthetic-only flag is invalid")
    plan["plan_sha256"] = actual
    if args.command == "run":
        print(json.dumps(run(plan, args.journal)))
    elif args.command == "resolve-uncertain":
        resolve_uncertain(plan, args.journal, args.model, args.case_id)
    else:
        report = export_report(plan, args.journal, allow_partial=args.partial)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({"cases": report["cases"], "calls": report["calls"],
                          "incomplete_tasks": report["incomplete_tasks"],
                          "uncertain_tasks": report["uncertain_tasks"]}))


if __name__ == "__main__":
    main()
