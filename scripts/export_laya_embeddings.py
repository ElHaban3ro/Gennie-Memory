#!/usr/bin/env python3
"""Export current synthetic memory vectors and one query from SOC's cached Laya encoder."""

import argparse
import hashlib
import json
import os
import time
from pathlib import Path

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--queries", type=Path, required=True)
    selector = parser.add_mutually_exclusive_group(required=True)
    selector.add_argument("--case-id")
    selector.add_argument("--split", choices=("development", "calibration", "final", "all"))
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    parser.add_argument("--out-batch", type=Path, required=True)
    parser.add_argument("--out-query", type=Path)
    parser.add_argument("--out-queries", type=Path)
    args = parser.parse_args()
    if (args.case_id and not args.out_query) or (args.split and not args.out_queries):
        parser.error("--case-id needs --out-query; --split needs --out-queries")
    corpus = json.loads(args.corpus.read_text())
    all_cases = [json.loads(line) for line in args.queries.read_text().splitlines()]
    cases = ([next(case for case in all_cases if case["id"] == args.case_id)] if args.case_id else
             [case for case in all_cases if args.split == "all" or case["split"] == args.split])
    if not cases or any(case["label_source"] != "deterministic_fixture_rule" for case in cases):
        raise ValueError("Only synthetic benchmark cases are accepted")
    current = {}
    for row in corpus:
        if row["id"] not in current or row["revision"] > current[row["id"]]["revision"]:
            current[row["id"]] = row
    memories = [current[identity] for identity in sorted(current) if current[identity]["state"] == "active"]
    model = Path(json.loads(args.soc_runtime.read_text())["model"]).resolve()
    if not all((model / name).is_file() for name in ("model.safetensors", "rl_agent_config.json",
                                                    "encoder/config.json", "tokenizer/tokenizer.json")):
        raise ValueError("SOC Laya snapshot is incomplete")

    import numpy as np
    import torch
    import laya
    from laya.shortlist import embed_fn_from_agent
    from transformers.initialization import no_init_weights

    torch.set_num_threads(2)
    started = time.perf_counter()
    with no_init_weights():
        agent = laya.load(str(model), device="mps")
    load_seconds = time.perf_counter() - started
    texts = [row["summary"] for row in memories]
    query_texts = [case["request"]["query"] for case in cases]
    for text in texts + query_texts:
        if len(agent.tok(text, add_special_tokens=True)["input_ids"]) > 512:
            raise ValueError("Text exceeds encoder token budget")
    embed = embed_fn_from_agent(agent, max_length=512, batch_size=16)
    started = time.perf_counter()
    vectors = embed(texts + query_texts)
    embed_seconds = time.perf_counter() - started
    if not np.isfinite(vectors).all() or np.any(np.linalg.norm(vectors, axis=1) <= 0):
        raise ValueError("Invalid encoder vectors")
    model_sha = sha256(model / "model.safetensors")
    tokenizer_sha = sha256(model / "tokenizer/tokenizer.json")
    space_id = f"laya-mmbert-mean-v1-{model_sha[:12]}-{tokenizer_sha[:12]}"
    batch = {"space_id": space_id, "model_id": f"convaiinnovations/laya@{model.parent.name}/multilingual",
             "model_sha256": model_sha, "tokenizer_sha256": tokenizer_sha,
             "procedure": "last-hidden-mean-pool-no-prefix-v1",
             "dimensions": int(vectors.shape[1]),
             "vectors": [{"memory_id": row["id"], "revision": row["revision"],
                          "values": vector.astype(float).tolist()}
                         for row, vector in zip(memories, vectors[:len(memories)])]}
    hybrid_requests = [{"search": case["request"], "space_id": space_id,
                        "query_vector": vector.astype(float).tolist()}
                       for case, vector in zip(cases, vectors[len(memories):])]
    args.out_batch.parent.mkdir(parents=True, exist_ok=True)
    args.out_batch.write_text(json.dumps(batch, ensure_ascii=False) + "\n")
    if args.out_query:
        args.out_query.parent.mkdir(parents=True, exist_ok=True)
        args.out_query.write_text(json.dumps(hybrid_requests[0], ensure_ascii=False) + "\n")
    if args.out_queries:
        args.out_queries.parent.mkdir(parents=True, exist_ok=True)
        with args.out_queries.open("w") as output:
            for case, request in zip(cases, hybrid_requests):
                item = {key: case[key] for key in ("id", "split", "segment", "expected_ids",
                                                  "expected_fact", "forbidden_ids", "label_source")}
                item["expected_condition"] = case.get("expected_condition")
                item["expected_source"] = case.get("expected_source")
                item["request"] = request
                output.write(json.dumps(item, ensure_ascii=False, separators=(",", ":")) + "\n")
    print(json.dumps({"device": str(agent.device), "model_version": laya.__version__,
                      "space_id": space_id, "dimensions": int(vectors.shape[1]),
                      "memory_vectors": len(memories), "query_vectors": len(cases),
                      "load_seconds": load_seconds,
                      "embedding_seconds": embed_seconds}))


if __name__ == "__main__":
    main()
