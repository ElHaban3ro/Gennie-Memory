#!/usr/bin/env python3
"""Reuse one verified encoder run across nested synthetic corpus sizes and scopes."""

import argparse
import hashlib
import json
from pathlib import Path


SIZES = (100, 200, 2000)
SPLITS = ("development", "calibration")
VARIANTS = ("scoped", "unscoped")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def dump_jsonl(path: Path, values: list[dict]) -> None:
    path.write_text("".join(json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n"
                            for value in values))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--matrix-dir", type=Path, required=True)
    parser.add_argument("--development-batch", type=Path, required=True)
    parser.add_argument("--calibration-batch", type=Path, required=True)
    parser.add_argument("--development-vectors", type=Path, required=True)
    parser.add_argument("--calibration-vectors", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    batches = {split: json.loads(getattr(args, f"{split}_batch").read_text()) for split in SPLITS}
    for key in ("space_id", "model_id", "model_sha256", "tokenizer_sha256", "procedure", "dimensions"):
        if batches["development"][key] != batches["calibration"][key]:
            raise ValueError(f"Encoder profile differs between splits: {key}")
    if batches["development"]["vectors"] != batches["calibration"]["vectors"]:
        raise ValueError("Memory vectors differ between splits")
    vectors = {row["memory_id"]: row for row in batches["development"]["vectors"]}
    if len(vectors) != len(batches["development"]["vectors"]):
        raise ValueError("Duplicate memory vectors")
    encoded = {}
    for split in SPLITS:
        values = rows(getattr(args, f"{split}_vectors"))
        encoded[split] = {row["id"]: row for row in values}
        if len(encoded[split]) != len(values):
            raise ValueError("Duplicate query vectors")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"version": 1, "kind": "candidate_matrix_vectors",
                "source_sha256": {name: digest(getattr(args, name)) for name in
                                  ("development_batch", "calibration_batch",
                                   "development_vectors", "calibration_vectors")},
                "space_id": batches["development"]["space_id"], "outputs": {}}
    for size in SIZES:
        source_dir = args.matrix_dir / str(size)
        target_dir = args.output_dir / str(size)
        target_dir.mkdir(parents=True, exist_ok=True)
        corpus = json.loads((source_dir / "corpus.json").read_text())
        ids = {row["id"] for row in corpus}
        if len(ids) != size or not ids <= vectors.keys():
            raise ValueError(f"Missing or duplicate memory vector at size {size}")
        batch = {**batches["development"],
                 "vectors": [row for row in batches["development"]["vectors"]
                             if row["memory_id"] in ids]}
        batch_path = target_dir / "embeddings.json"
        batch_path.write_text(json.dumps(batch, ensure_ascii=False, separators=(",", ":")) + "\n")
        manifest["outputs"][str(batch_path.relative_to(args.output_dir))] = digest(batch_path)
        for variant in VARIANTS:
            source_cases = rows(source_dir / f"{variant}.jsonl")
            for split in SPLITS:
                prepared = []
                for case in source_cases:
                    if case["split"] != split:
                        continue
                    vector_id = case["id"] if variant == "scoped" else case["id"].replace("_unscoped", "_scoped")
                    encoded_case = encoded[split].get(vector_id)
                    if encoded_case is None or encoded_case["request"]["search"]["query"] != case["request"]["query"]:
                        raise ValueError(f"Missing or mismatched query vector: {case['id']}")
                    prepared.append({key: case[key] for key in
                                     ("id", "split", "segment", "expected_ids", "expected_fact",
                                      "expected_condition", "expected_source", "forbidden_ids",
                                      "label_source")} |
                                    {"request": {"search": case["request"],
                                                 "space_id": batches["development"]["space_id"],
                                                 "query_vector": encoded_case["request"]["query_vector"]}})
                output_path = target_dir / f"{variant}-{split}-hybrid.jsonl"
                dump_jsonl(output_path, prepared)
                manifest["outputs"][str(output_path.relative_to(args.output_dir))] = digest(output_path)
    (args.output_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"sizes": SIZES, "profile": manifest["space_id"],
                      "outputs": len(manifest["outputs"])}))


if __name__ == "__main__":
    main()
