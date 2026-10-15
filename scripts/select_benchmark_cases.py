#!/usr/bin/env python3
"""Select a deterministic, balanced synthetic subset without touching held-out splits."""

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path


def select_cases(queries: Path, split: str, per_segment: int, excluded: set[str],
                 excluded_entities: set[str] | None = None) -> list[dict]:
    excluded_entities = excluded_entities or set()
    grouped = defaultdict(list)
    seen = set()
    for line in queries.read_text().splitlines():
        if not line.strip():
            continue
        case = json.loads(line)
        if case["id"] in seen:
            raise ValueError(f'Duplicate case ID: {case["id"]}')
        seen.add(case["id"])
        if (case["split"] == split and case["id"] not in excluded and
            case.get("request", {}).get("entity_id") not in excluded_entities):
            grouped[case["segment"]].append(case)
    if not grouped:
        raise ValueError("No cases in the requested split")
    selected = []
    for segment, cases in sorted(grouped.items()):
        if len(cases) < per_segment:
            raise ValueError(f"Too few cases for segment {segment}")
        cases.sort(key=lambda case: (hashlib.sha256(case["id"].encode()).hexdigest(), case["id"]))
        selected.extend(cases[:per_segment])
    selected.sort(key=lambda case: case["id"])
    return selected


def select_form_coverage_cases(queries: Path, forms_path: Path, split: str,
                               negative_per_segment: int, excluded: set[str],
                               excluded_entities: set[str]) -> list[dict]:
    """Cover every positive prompt form once, plus balanced negative controls."""
    forms = json.loads(forms_path.read_text())
    expected_forms = {form["id"]: form["segment"] for form in forms}
    if len(expected_forms) != len(forms):
        raise ValueError("Duplicate form ID")
    positives = defaultdict(list)
    negatives = defaultdict(list)
    seen = set()
    for line in queries.read_text().splitlines():
        if not line.strip():
            continue
        case = json.loads(line)
        case_id = case["id"]
        if case_id in seen:
            raise ValueError(f"Duplicate case ID: {case_id}")
        seen.add(case_id)
        if (case["split"] != split or case_id in excluded or
            case["request"].get("entity_id") in excluded_entities):
            continue
        if case["label_source"] != "deterministic_fixture_rule":
            raise ValueError(f"Unlabeled case: {case_id}")
        if case["expected_ids"]:
            parts = case_id.split("_")
            if len(parts) != 3 or parts[0] != "q5":
                raise ValueError(f"Positive case has no v5 form ID: {case_id}")
            form_id = int(parts[2])
            if expected_forms.get(form_id) != case["segment"]:
                raise ValueError(f"Form segment mismatch: {case_id}")
            positives[form_id].append(case)
        else:
            negatives[case["segment"]].append(case)
    missing = set(expected_forms) - set(positives)
    if missing:
        raise ValueError(f"No eligible case for {len(missing)} positive forms")
    selected = [min(positives[form_id], key=lambda case: (
        hashlib.sha256(case["id"].encode()).hexdigest(), case["id"]))
        for form_id in sorted(expected_forms)]
    if not negatives:
        raise ValueError("No negative segments")
    for segment, cases in sorted(negatives.items()):
        if len(cases) < negative_per_segment:
            raise ValueError(f"Too few negative cases for segment {segment}")
        cases.sort(key=lambda case: (hashlib.sha256(case["id"].encode()).hexdigest(), case["id"]))
        selected.extend(cases[:negative_per_segment])
    selected.sort(key=lambda case: case["id"])
    return selected


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--split", choices=("development", "calibration", "final"),
                        default="development")
    parser.add_argument("--per-segment", type=int, default=5)
    parser.add_argument("--cover-forms", type=Path,
                        help="Select one positive v5 case per form, plus --per-segment negatives")
    parser.add_argument("--exclude-ids", type=Path, action="append",
                        help="JSONL cases whose IDs must not appear in this subset; repeatable")
    parser.add_argument("--exclude-entities-from", type=Path, action="append",
                        help="JSONL cases whose entity IDs must not appear in this subset")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.per_segment <= 1000:
        parser.error("per-segment must be 1..1000")
    excluded = set()
    for path in args.exclude_ids or []:
        for line in path.read_text().splitlines():
            if line.strip():
                excluded.add(json.loads(line)["id"])
    excluded_entities = set()
    for path in args.exclude_entities_from or []:
        for line in path.read_text().splitlines():
            if line.strip():
                entity_id = json.loads(line)["request"].get("entity_id")
                if entity_id is not None:
                    excluded_entities.add(entity_id)
    selected = (select_form_coverage_cases(args.queries, args.cover_forms, args.split,
                                           args.per_segment, excluded, excluded_entities)
                if args.cover_forms else
                select_cases(args.queries, args.split, args.per_segment, excluded,
                             excluded_entities))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    payload = "".join(json.dumps(case, ensure_ascii=False, separators=(",", ":")) + "\n"
                      for case in selected)
    args.output.write_text(payload)
    print(json.dumps({"split": args.split, "cases": len(selected),
                      "segments": dict(sorted((name, sum(case["segment"] == name for case in selected))
                                              for name in {case["segment"] for case in selected})),
                      "sha256": hashlib.sha256(payload.encode()).hexdigest()}))


if __name__ == "__main__":
    main()
