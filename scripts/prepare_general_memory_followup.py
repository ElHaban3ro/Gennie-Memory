#!/usr/bin/env python3
"""Freeze a new entity-disjoint calibration cohort from unused synthetic subjects."""

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def write(path: Path, rows: list[dict]) -> None:
    path.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                    for row in rows))


def search(case: dict) -> dict:
    request = case["request"]
    return request.get("search", request)


def prepare(queries: Path, results: Path, hybrid_queries: Path, output: Path) -> dict:
    cases, screens = read(queries), read(results)
    if len(cases) != 3300 or len(screens) != len(cases):
        raise ValueError("Expected the complete general-memory fixture")
    subjects = defaultdict(set)
    for case, screen in zip(cases, screens):
        if case["id"] != screen["id"] or case["split"] != screen["split"] or case["expected_ids"] != screen["expected_ids"]:
            raise ValueError("Queries and retrieval screens differ")
        request = search(case)
        subjects[request["scope_kind"]].add(request["scope_id"])
    selected = {kind: sorted(identities)[5:9] for kind, identities in subjects.items()}
    if set(selected) != {"user", "project", "conversation", "agent"} or any(len(ids) != 4 for ids in selected.values()):
        raise ValueError("Insufficient unseen subjects")
    chosen = [(case, screen) for case, screen in zip(cases, screens)
              if case["split"] == "holdout" and
              search(case)["scope_id"] in selected[search(case)["scope_kind"]]]
    if len(chosen) != 540 or sum(bool(case["expected_ids"]) for case, _ in chosen) != 460:
        raise ValueError("Calibration labels changed")
    hybrid = read(hybrid_queries)
    if len(hybrid) != len(cases) or [row["id"] for row in hybrid] != [row["id"] for row in cases]:
        raise ValueError("Hybrid query snapshot differs")
    hybrid_by_id = {row["id"]: row["request"] for row in hybrid}
    if any(hybrid_by_id[case["id"]].get("search") != search(case) for case, _ in chosen):
        raise ValueError("Hybrid query search differs")
    output.mkdir(parents=True, exist_ok=True)
    query_path, result_path = output / "calibration-queries.jsonl", output / "calibration-results.jsonl"
    write(query_path, [{**case, "split": "calibration", "expected_source":
                        ({**case["expected_source"],
                          "locator": case["expected_source"].get("locator")}
                         if case["expected_source"] else None)} for case, _ in chosen])
    write(result_path, [{**screen, "split": "calibration"} for _, screen in chosen])
    seed_path = output / "calibration-seed-results.jsonl"
    write(seed_path, [{**screen, "split": "calibration",
                       "hybrid_request": hybrid_by_id[case["id"]]}
                      for case, screen in chosen])
    manifest = {"kind": "general_memory_followup_calibration", "synthetic_only": True,
                "source_sha256": {"queries": digest(queries), "results": digest(results)},
                "selected_subjects": selected,
                "cohort_sha256": {"queries": digest(query_path), "results": digest(result_path)},
                "seed_results_sha256": digest(seed_path),
                "positive": 460, "negative": 80}
    (output / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--hybrid-queries", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(prepare(args.queries, args.results, args.hybrid_queries, args.output), ensure_ascii=False))
