#!/usr/bin/env python3
"""Build compose-seed batch requests from frozen classifier reports."""

import argparse
import json
from pathlib import Path


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--retrieval", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    queries = {row["id"]: row for row in rows(args.queries)}
    retrieval = {row["id"]: row for row in rows(args.retrieval)}
    report = json.loads(args.report.read_text())
    selected = [row for row in report["results"] if row["requested_model"] == args.model]
    if set(queries) != set(retrieval) or len(selected) != len(queries) or (
            {row["case_id"] for row in selected} != set(queries)):
        raise ValueError("Query, retrieval, and classifier cases differ")
    output = []
    for decision in selected:
        case_id = decision["case_id"]
        case = queries[case_id]
        candidates = retrieval[case_id]["response"]["candidates"]
        by_id = {row["memory_id"]: row for row in candidates}
        if decision["status"] not in ("complete", "skipped_empty") or (
                not set(decision["candidate_ids"]) <= by_id.keys()):
            raise ValueError(f"Invalid classifier result: {case_id}")
        presented = [{"memory_id": identity, "revision": by_id[identity]["revision"]}
                     for identity in decision["candidate_ids"]]
        chosen = decision["chosen_id"]
        if chosen != "none" and chosen not in decision["candidate_ids"]:
            raise ValueError(f"Chosen memory was not presented: {case_id}")
        selected_ids = ([{"memory_id": chosen, "revision": by_id[chosen]["revision"]}]
                        if chosen != "none" else [])
        output.append({
            "id": case_id, "split": case["split"], "segment": case["segment"],
            "request": {"search": None, "hybrid": case["request"],
                        "presented": presented, "selected": selected_ids,
                        "classifier": args.model, "classification_status": "complete"},
            "expected_ids": case["expected_ids"],
            "expected_fact": case.get("expected_fact"),
            "expected_condition": case.get("expected_condition"),
            "expected_source": case.get("expected_source"),
            "forbidden_ids": case["forbidden_ids"],
            "label_source": case["label_source"],
        })
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                                   for row in output))
    print(json.dumps({"cases": len(output), "selected": sum(bool(row["request"]["selected"])
                                                        for row in output)}))


if __name__ == "__main__":
    main()
