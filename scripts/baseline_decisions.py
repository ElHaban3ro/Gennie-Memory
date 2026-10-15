#!/usr/bin/env python3
"""Create a deterministic top-1 decision report from synthetic retrieval results."""

import argparse
import json
from pathlib import Path


def read_jsonl(path: Path) -> dict[str, dict]:
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    values = {row["id"]: row for row in rows}
    if len(values) != len(rows):
        raise ValueError("Duplicate case ID")
    return values


def make_report(queries: dict[str, dict], results: dict[str, dict]) -> dict:
    if not queries or set(queries) != set(results):
        raise ValueError("Query and result IDs must match")
    measured = []
    for case_id in sorted(queries):
        case, result = queries[case_id], results[case_id]
        if case["label_source"] != "deterministic_fixture_rule":
            raise ValueError(f"Non-synthetic label in {case_id}")
        if any(result[key] != case[key] for key in
               ("split", "segment", "expected_ids", "forbidden_ids", "label_source")):
            raise ValueError(f"Query/result label mismatch in {case_id}")
        if any(result.get(key) != case.get(key) for key in
               ("expected_condition", "expected_source")):
            raise ValueError(f"Query/result metadata mismatch in {case_id}")
        if result["response"]["status"] not in ("complete", "degraded"):
            raise ValueError(f"Invalid retrieval result in {case_id}")
        candidates = result["response"]["candidates"][:1]
        identifiers = [candidate["memory_id"] for candidate in candidates]
        if set(identifiers) & set(case["forbidden_ids"]):
            raise ValueError(f"Forbidden candidate in {case_id}")
        chosen = identifiers[0] if identifiers else "none"
        measured.append({
            "id": case_id, "status": "complete" if identifiers else "skipped_empty",
            "chosen_id": chosen, "candidate_ids": identifiers,
            "expected_ids": case["expected_ids"],
            "correct": (chosen in case["expected_ids"]
                        if case["expected_ids"] else chosen == "none"),
        })
    return {"method": "deterministic_top1", "synthetic_only": True,
            "max_candidates": 1, "cases": len(measured),
            "individual_results": measured}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = make_report(read_jsonl(args.queries), read_jsonl(args.results))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": report["cases"],
                      "correct": sum(row["correct"] for row in report["individual_results"])}))


if __name__ == "__main__":
    main()
