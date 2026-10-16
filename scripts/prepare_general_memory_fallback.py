#!/usr/bin/env python3
"""Freeze Jev calls only for primary Decisions abstentions on synthetic cases."""

import argparse
import hashlib
import json
from pathlib import Path

if __package__:
    from .prepare_general_memory_followup import read, write
else:
    from prepare_general_memory_followup import read, write


PRIMARY = "openai/gpt-6-luna-decisions"


def prepare(queries: Path, results: Path, primary_report: Path, output: Path) -> dict:
    cases, screens = read(queries), read(results)
    decisions = [row for row in json.loads(primary_report.read_text())["results"]
                 if row["requested_model"] == PRIMARY]
    if len(cases) != len(screens) or len(decisions) != len(cases):
        raise ValueError("Primary decisions are incomplete")
    by_id = {row["case_id"]: row for row in decisions}
    if len(by_id) != len(decisions) or set(by_id) != {case["id"] for case in cases}:
        raise ValueError("Primary case IDs differ")
    selected = []
    for case, screen in zip(cases, screens):
        decision = by_id[case["id"]]
        ids = [item["memory_id"] for item in screen["response"]["candidates"][:20]]
        if (case["id"] != screen["id"] or decision["requested_model"] != PRIMARY or
                decision["status"] != "complete" or decision["expected_ids"] != case["expected_ids"] or
                decision["candidate_ids"] != ids):
            raise ValueError(f"Primary decision or candidates differ for {case['id']}")
        if decision["chosen_id"] == "none":
            selected.append((case, screen))
    output.mkdir(parents=True, exist_ok=True)
    query_path, result_path = output / "fallback-queries.jsonl", output / "fallback-results.jsonl"
    write(query_path, [case for case, _ in selected])
    write(result_path, [screen for _, screen in selected])
    manifest = {"kind": "general_memory_fallback_subset", "synthetic_only": True,
                "primary_report_sha256": hashlib.sha256(primary_report.read_bytes()).hexdigest(),
                "queries_sha256": hashlib.sha256(query_path.read_bytes()).hexdigest(),
                "results_sha256": hashlib.sha256(result_path.read_bytes()).hexdigest(),
                "cases": len(selected),
                "positive": sum(bool(case["expected_ids"]) for case, _ in selected),
                "negative": sum(not case["expected_ids"] for case, _ in selected)}
    (output / "fallback-manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--primary-report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(prepare(args.queries, args.results, args.primary_report, args.output), ensure_ascii=False))
