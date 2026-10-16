#!/usr/bin/env python3
"""Prepare, run, and export the frozen remote candidate matrix by split."""

import argparse
import json
from pathlib import Path

import run_openrouter_campaign as campaign


MODELS = ("typesafe/jev-1.13", "openai/gpt-6-luna-decisions")


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def frozen_split(root: Path, size: int, scope: str, split: str) -> tuple[Path, Path, int]:
    directory = root / str(size)
    destination = directory / "remote" / split
    destination.mkdir(parents=True, exist_ok=True)
    query_rows = [row for row in rows(directory / f"screen-{scope}.jsonl")
                  if row["split"] == split]
    result_rows = [row for row in rows(directory / f"screen-{scope}-results.jsonl")
                   if row["split"] == split]
    if not query_rows or [row["id"] for row in query_rows] != [row["id"] for row in result_rows]:
        raise ValueError(f"Unmatched split inputs: {size}/{scope}/{split}")
    queries = destination / f"{scope}-queries.jsonl"
    results = destination / f"{scope}-results.jsonl"
    for path, data in ((queries, query_rows), (results, result_rows)):
        frozen = "".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                         for row in data)
        if path.exists() and path.read_text() != frozen:
            raise ValueError(f"Frozen split file changed: {path}")
        path.write_text(frozen)
    return queries, results, len(query_rows)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--split", choices=("development", "calibration"), required=True)
    parser.add_argument("--phase", choices=("prepare", "run", "export"), required=True)
    parser.add_argument("--retry-failed", action="store_true",
                        help="Retry terminal failures in their append-only journals")
    args = parser.parse_args()
    total_calls = 0
    total_cost = 0.0
    for size in (100, 200, 2000):
        for scope in ("scoped", "unscoped"):
            queries, results, count = frozen_split(args.root, size, scope, args.split)
            for cap in (5, 10, 15, 20):
                for variant in campaign.classifier_prompts.CHOICE_VARIANTS:
                    name = f"{scope}-{cap}-{variant}"
                    directory = args.root / str(size) / "remote" / args.split
                    plan_path = directory / f"{name}-plan.json"
                    journal_path = directory / f"{name}-journal.jsonl"
                    report_path = directory / f"{name}-report.json"
                    params = argparse.Namespace(queries=queries, results=results,
                                                split=args.split, model=list(MODELS),
                                                max_candidates=cap, prompt_variant=variant,
                                                max_calls=count * len(MODELS),
                                                max_cost_usd=None,
                                                reserve_per_call_usd=.001,
                                                max_seconds=1800)
                    plan = campaign.prepare(params)
                    if plan_path.exists():
                        if json.loads(plan_path.read_text()) != plan:
                            raise ValueError(f"Frozen plan changed: {plan_path}")
                    else:
                        plan_path.write_text(json.dumps(plan, ensure_ascii=False, indent=2) + "\n")
                    total_calls += plan["planned_calls"]
                    if args.phase == "run":
                        progress = campaign.run(plan, journal_path,
                                                retry_failed=args.retry_failed)
                        total_cost += progress["reported_or_reserved_cost_usd"]
                        print(json.dumps({"scenario": f"{size}/{scope}/{cap}/{variant}",
                                          **progress}), flush=True)
                        if (progress["finished_tasks"] != progress["planned_tasks"] or
                                progress["failed_tasks"]):
                            raise RuntimeError(f"Incomplete remote cell: {size}/{name}")
                    elif args.phase == "export":
                        report = campaign.export_report(plan, journal_path)
                        report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
                        total_cost += report["reported_or_reserved_cost_usd"]
    print(json.dumps({"split": args.split, "phase": args.phase,
                      "planned_calls": total_calls,
                      "reported_or_reserved_cost_usd": total_cost}), flush=True)


if __name__ == "__main__":
    main()
