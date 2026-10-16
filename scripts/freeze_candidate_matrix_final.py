#!/usr/bin/env python3
"""Freeze unseen entities for a final synthetic check without running retrieval."""

import argparse
import hashlib
import json
from pathlib import Path


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def entity(case: dict) -> str:
    return "_".join(case["id"].split("_")[1:3])


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    args = parser.parse_args()
    destination = args.root / "final"
    destination.mkdir(parents=True, exist_ok=True)
    screen = rows(args.root / "100" / "screen-scoped.jsonl")
    seen = {entity(case) for case in screen}
    manifest = {"version": 1, "kind": "candidate_matrix_final_holdout",
                "source_size": 2000, "excluded_screen_entities": sorted(seen),
                "selected_entities": [], "variants": {}}
    for variant in ("scoped", "unscoped"):
        source = (rows(args.root / "2000" / f"{variant}-development-hybrid.jsonl") +
                  rows(args.root / "2000" / f"{variant}-calibration-hybrid.jsonl"))
        entities = sorted({entity(case) for case in source})
        chosen = entities[-20:]
        if len(entities) != 100 or len(chosen) != 20 or seen & set(chosen):
            raise ValueError("Final entities overlap screen or source size is wrong")
        if manifest["selected_entities"] and manifest["selected_entities"] != chosen:
            raise ValueError("Scoped and unscoped final entities differ")
        manifest["selected_entities"] = chosen
        selected = [{**case, "split": "final"} for case in source if entity(case) in chosen]
        if len(selected) != 480 or sum(bool(case["expected_ids"]) for case in selected) != 400:
            raise ValueError("Final case count is wrong")
        path = destination / f"{variant}-queries.jsonl"
        frozen = "".join(json.dumps(case, ensure_ascii=False, separators=(",", ":")) + "\n"
                         for case in selected)
        if path.exists() and path.read_text() != frozen:
            raise ValueError(f"Final holdout changed: {path}")
        path.write_text(frozen)
        manifest["variants"][variant] = {"queries_sha256": hashlib.sha256(frozen.encode()).hexdigest(),
                                         "cases": len(selected), "positive": 400, "negative": 80}
    path = destination / "manifest.json"
    value = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    if path.exists() and path.read_text() != value:
        raise ValueError("Final holdout manifest changed")
    path.write_text(value)
    print(json.dumps({"entities": 20, "cases_per_variant": 480,
                      "positive_per_variant": 400, "retrieval_run": False}))


if __name__ == "__main__":
    main()
