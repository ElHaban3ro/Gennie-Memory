#!/usr/bin/env python3
"""Freeze balanced synthetic retrieval samples for the candidate matrix."""

import argparse
import hashlib
import json
from pathlib import Path


def digest(value: str) -> str:
    return hashlib.sha256(value.encode()).hexdigest()


def read_jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vector-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    manifest = {"version": 1, "kind": "candidate_matrix_retrieval_sample",
                "selection": "SHA-256 of case ID, separately by split and positive/negative label",
                "samples": {}}
    for size in (100, 200, 2000):
        out_dir = args.output_dir / str(size)
        out_dir.mkdir(parents=True, exist_ok=True)
        for variant in ("scoped", "unscoped"):
            chosen = []
            for split in ("development", "calibration"):
                source = args.vector_dir / str(size) / f"{variant}-{split}-hybrid.jsonl"
                values = read_jsonl(source)
                positive = sorted((row for row in values if row["expected_ids"]),
                                  key=lambda row: digest(row["id"]))
                negative = sorted((row for row in values if not row["expected_ids"]),
                                  key=lambda row: digest(row["id"]))
                chosen.extend(positive if size == 100 else positive[:50])
                chosen.extend(negative[:20])
            chosen.sort(key=lambda row: row["id"])
            output = out_dir / f"sample-{variant}.jsonl"
            payload = "".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                              for row in chosen)
            output.write_text(payload)
            manifest["samples"][f"{size}/{variant}"] = {
                "positive": sum(bool(row["expected_ids"]) for row in chosen),
                "negative": sum(not row["expected_ids"] for row in chosen),
                "sha256": hashlib.sha256(payload.encode()).hexdigest(),
                "case_ids": [row["id"] for row in chosen],
            }
    (args.output_dir / "sample-manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: {k: v[k] for k in ("positive", "negative")}
                      for key, v in manifest["samples"].items()}))


if __name__ == "__main__":
    main()
