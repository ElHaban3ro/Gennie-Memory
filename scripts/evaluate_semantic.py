#!/usr/bin/env python3
"""Pilot exact cosine search with the cached Laya encoder (not a dedicated embedding model)."""

import argparse
import json
import os
import time
from collections import defaultdict
from pathlib import Path

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def quantile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * fraction))] if ordered else 0.0


def choose_cases(path: Path, split: str, limit: int) -> list[dict]:
    grouped = defaultdict(list)
    for line in path.read_text().splitlines():
        case = json.loads(line)
        if case["split"] == split:
            grouped[case["segment"]].append(case)
    segments = sorted(grouped)
    selected = []
    step = {segment: max(1, len(grouped[segment]) // max(1, (limit + len(segments) - 1) // len(segments)))
            for segment in segments}
    offset = 0
    while len(selected) < limit and segments:
        for segment in list(segments):
            index = offset * step[segment]
            if index < len(grouped[segment]):
                selected.append(grouped[segment][index])
                if len(selected) >= limit:
                    break
            else:
                segments.remove(segment)
        offset += 1
    return selected


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    parser.add_argument("--split", choices=("development", "calibration", "final"), default="development")
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument("--top-k", type=int, default=20)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.limit <= 100000 or not 1 <= args.top_k <= 40:
        parser.error("limit must be positive and top-k between 1 and 40")
    corpus = json.loads(args.corpus.read_text())
    cases = choose_cases(args.queries, args.split, args.limit)
    if not cases:
        raise ValueError("No cases in selected split")
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
    embed = embed_fn_from_agent(agent, max_length=512, batch_size=16)
    started = time.perf_counter()
    embed(["Synthetic warmup text"])
    warmup_ms = (time.perf_counter() - started) * 1000
    texts = [memory["summary"] for memory in corpus]
    for text in texts + [case["request"]["query"] for case in cases]:
        if len(agent.tok(text, add_special_tokens=True)["input_ids"]) > 512:
            raise ValueError("Text exceeds encoder token budget")
    started = time.perf_counter()
    memory_vectors = embed(texts)
    memory_vectors = memory_vectors / np.maximum(np.linalg.norm(memory_vectors, axis=1, keepdims=True), 1e-12)
    index_seconds = time.perf_counter() - started
    if not np.isfinite(memory_vectors).all():
        raise ValueError("Non-finite memory embeddings")

    results = []
    for case in cases:
        request = case["request"]
        allowed = [i for i, memory in enumerate(corpus)
                   if memory["scope_kind"] == request["scope_kind"] and
                   memory["scope_id"] == request["scope_id"] and memory["state"] == "active" and
                   (memory["valid_from"] is None or memory["valid_from"] <= request["as_of"]) and
                   (memory["valid_until"] is None or memory["valid_until"] > request["as_of"])]
        started = time.perf_counter()
        query_vector = embed([request["query"]])[0]
        query_vector = query_vector / max(np.linalg.norm(query_vector), 1e-12)
        scores = [(corpus[i]["id"], float(memory_vectors[i] @ query_vector)) for i in allowed]
        scores.sort(key=lambda item: (-item[1], item[0]))
        elapsed_ms = (time.perf_counter() - started) * 1000
        top = scores[:args.top_k]
        results.append({"id": case["id"], "segment": case["segment"],
                        "expected_ids": case["expected_ids"], "top_ids": [item[0] for item in top],
                        "top_score": top[0][1] if top else None, "elapsed_ms": elapsed_ms,
                        "forbidden_leak": bool(set(case["forbidden_ids"]) & {item[0] for item in top})})

    positives = [result for result in results if result["expected_ids"]]
    negatives = [result for result in results if not result["expected_ids"]]
    negative_scores = [result["top_score"] for result in negatives if result["top_score"] is not None]
    report = {"method": "laya_encoder_exact_cosine_pilot", "model_path": str(model),
              "model_version": getattr(laya, "__version__", "unknown"), "device": str(agent.device),
              "split": args.split, "top_k": args.top_k, "cases": len(results),
              "load_seconds": load_seconds, "index_seconds": index_seconds,
              "warmup_ms": warmup_ms,
              "positive_recall_at_k": sum(bool(set(r["expected_ids"]) & set(r["top_ids"])) for r in positives) / len(positives) if positives else None,
              "positive_top_1": sum(r["top_ids"][:1] == r["expected_ids"][:1] for r in positives) / len(positives) if positives else None,
              "negative_empty_rate": sum(not r["top_ids"] for r in negatives) / len(negatives) if negatives else None,
              "negative_top_score": {"min": min(negative_scores, default=None),
                                     "max": max(negative_scores, default=None)},
              "forbidden_leak_count": sum(r["forbidden_leak"] for r in results),
              "query_latency_ms": {"p50": quantile([r["elapsed_ms"] for r in results], .5),
                                   "p95": quantile([r["elapsed_ms"] for r in results], .95)},
              "individual_results": results}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key != "individual_results"},
                     ensure_ascii=False))


if __name__ == "__main__":
    main()
