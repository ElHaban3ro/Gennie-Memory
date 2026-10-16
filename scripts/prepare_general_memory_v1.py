#!/usr/bin/env python3
"""Adapt the synthetic general-memory fixture to the existing hybrid benchmark contract."""

import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_jsonl(path: Path, rows: list[dict]) -> None:
    path.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                            for row in rows))


def prepare(fixture: Path, output: Path) -> dict:
    memories_path = fixture / "memories.json"
    queries_path = fixture / "queries.jsonl"
    memories = json.loads(memories_path.read_text())
    raw = [json.loads(line) for line in queries_path.read_text().splitlines() if line.strip()]
    by_id = {memory["id"]: memory for memory in memories}
    if len(by_id) != len(memories) or len({row["case_id"] for row in raw}) != len(raw):
        raise ValueError("Duplicate fixture ID")
    scopes = defaultdict(set)
    for row in raw:
        scopes[row["scope_kind"]].add(row["scope_id"])
    if set(scopes) != {"user", "project", "conversation", "agent"}:
        raise ValueError("Unexpected scope types")
    split_by_scope = {}
    for kind, identities in scopes.items():
        ordered = sorted(identities)
        if len(ordered) < 5:
            raise ValueError(f"Too few entities in {kind}")
        split_by_scope[kind] = {identity: ("development" if index == 0 else
                                           "final" if index <= 4 else "holdout")
                                for index, identity in enumerate(ordered)}
    cases = []
    for row in raw:
        if row["label"] not in ("positive", "negative") or bool(row["expected_memory_ids"]) != (row["label"] == "positive"):
            raise ValueError(f"Inconsistent label: {row['case_id']}")
        expected = row["expected_memory_ids"]
        if len(expected) > 1 or any(identity not in by_id for identity in expected):
            raise ValueError(f"Invalid target: {row['case_id']}")
        target = by_id[expected[0]] if expected else None
        if target and (target["scope_kind"], target["scope_id"]) != (row["scope_kind"], row["scope_id"]):
            raise ValueError(f"Target crosses scope: {row['case_id']}")
        cases.append({
            "id": row["case_id"], "split": split_by_scope[row["scope_kind"]][row["scope_id"]],
            "segment": f"{row['scope_kind']}/{row['topic']}",
            "request": {key: row[key] for key in
                        ("query", "scope_kind", "scope_id", "entity_id", "as_of", "limit")},
            "expected_ids": expected, "expected_fact": target["summary"] if target else None,
            "expected_condition": target["conditions"] if target else None,
            "expected_source": target["source"] if target else None,
            "forbidden_ids": [], "label_source": "deterministic_fixture_rule",
        })
    output.mkdir(parents=True, exist_ok=True)
    cases_path = output / "queries.jsonl"
    write_jsonl(cases_path, cases)
    counts = Counter((case["split"], "positive" if case["expected_ids"] else "negative") for case in cases)
    manifest = {"kind": "general_memory_v1_adapter", "synthetic_only": True,
                "selection": "Entity-disjoint: first ID per scope development; next four final; rest holdout",
                "fixture_sha256": {"memories": sha256(memories_path), "queries": sha256(queries_path)},
                "adapted_queries_sha256": sha256(cases_path),
                "counts": {f"{split}/{label}": count for (split, label), count in sorted(counts.items())}}
    (output / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(prepare(args.fixture, args.output), ensure_ascii=False))
