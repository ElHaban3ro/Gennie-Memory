#!/usr/bin/env python3
"""Remove duplicate query vectors from public reports after matching a frozen snapshot."""

import argparse
import hashlib
import json
from pathlib import Path


def compact(report: dict, snapshot: dict[str, dict], original_sha256: str,
            snapshot_name: str = "v5-campaign-hybrid-results.jsonl") -> dict:
    rows = report.get("individual_results")
    if not isinstance(rows, list) or len(rows) != len(snapshot):
        raise ValueError("Report and snapshot have different case counts")
    seen = set()
    for row in rows:
        case_id = row["id"]
        if case_id in seen or case_id not in snapshot:
            raise ValueError("Duplicate or unknown case ID")
        seen.add(case_id)
        if row.get("retrieval_request") != snapshot[case_id].get("hybrid_request"):
            raise ValueError(f"Query vector differs from frozen snapshot: {case_id}")
        del row["retrieval_request"]
    report["full_report_sha256"] = original_sha256
    report["query_vectors_in"] = snapshot_name
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--snapshot", type=Path, required=True)
    parser.add_argument("--report", type=Path, action="append", required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    snapshot_rows = [json.loads(line) for line in args.snapshot.read_text().splitlines() if line.strip()]
    snapshot = {row["id"]: row for row in snapshot_rows}
    if len(snapshot) != len(snapshot_rows) or any("hybrid_request" not in row for row in snapshot_rows):
        raise ValueError("Frozen snapshot has missing or duplicate hybrid requests")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    destinations = set()
    for path in args.report:
        destination = args.output_dir / path.name.replace("-vector-", "-")
        if destination in destinations:
            raise ValueError("Two raw reports would overwrite the same output")
        destinations.add(destination)
        if destination.resolve() == path.resolve() or destination.resolve() == args.snapshot.resolve():
            raise ValueError("Output must not overwrite a raw report or snapshot")
        raw = path.read_bytes()
        report = compact(json.loads(raw), snapshot, hashlib.sha256(raw).hexdigest(),
                         args.snapshot.name)
        destination.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({"report": path.name, "cases": report["cases"],
                          "bytes": destination.stat().st_size}))


if __name__ == "__main__":
    main()
