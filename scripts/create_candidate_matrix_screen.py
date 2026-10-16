#!/usr/bin/env python3
"""Freeze the same short classifier screen across all nested corpus sizes."""

import argparse
import hashlib
import json
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vector-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"version": 1, "kind": "candidate_matrix_classifier_screen",
                "selection": "Four positive task/mode combinations per entity and eight negative controls",
                "scenarios": {}}
    for variant in ("scoped", "unscoped"):
        source_100 = rows(args.vector_dir / "100" / f"{variant}-development-hybrid.jsonl") + rows(
            args.vector_dir / "100" / f"{variant}-calibration-hybrid.jsonl")
        entities = sorted({row["id"].split("_")[1] + "_" + row["id"].split("_")[2]
                           for row in source_100})
        if len(entities) != 5:
            raise ValueError("Expected five entities in the 100-memory corpus")
        chosen = []
        for entity in entities:
            for task, mode in ((0, "remote"), (3, "onsite"), (6, "remote"), (9, "onsite")):
                chosen.append(f"matrix_{entity}_{task}_{mode}_{variant}")
        for segment in ("mode_unknown", "wrong_task", "explicit_no_recall", "absent_project"):
            chosen.extend(f"matrix_{entity}_{segment}_{variant}" for entity in entities[:2])
        if len(chosen) != 28 or len(set(chosen)) != 28:
            raise ValueError("Invalid screen selection")
        reference = None
        for size in (100, 200, 2000):
            source = rows(args.vector_dir / str(size) / f"{variant}-development-hybrid.jsonl") + rows(
                args.vector_dir / str(size) / f"{variant}-calibration-hybrid.jsonl")
            by_id = {row["id"]: row for row in source}
            if len(by_id) != len(source) or not set(chosen) <= by_id.keys():
                raise ValueError(f"Missing or duplicate screen IDs at size {size}")
            selected = [by_id[identity] for identity in chosen]
            comparison = [(row["id"], row["request"]["search"], row["expected_ids"])
                          for row in selected]
            if reference is not None and comparison != reference:
                raise ValueError("Matched cases differ across corpus sizes")
            reference = comparison
            directory = args.output_dir / str(size)
            directory.mkdir(parents=True, exist_ok=True)
            output = directory / f"screen-{variant}.jsonl"
            output.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                                      for row in selected))
            manifest["scenarios"][f"{size}/{variant}"] = {
                "sha256": digest(output), "positives": 20, "negatives": 8,
                "case_ids": chosen}
    (args.output_dir / "screen-manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"scenarios": len(manifest["scenarios"]), "cases_each": 28}))


if __name__ == "__main__":
    main()
