#!/usr/bin/env python3
"""Measure one-query Laya embedding latency on frozen synthetic requests."""

import argparse
import hashlib
import json
import os
import time
from pathlib import Path

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def quantile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    args = parser.parse_args()
    source = args.queries.read_bytes()
    rows = [json.loads(line) for line in source.splitlines() if line.strip()]
    if len(rows) < 100 or any(row["label_source"] != "deterministic_fixture_rule" for row in rows):
        raise ValueError("Need at least one hundred frozen synthetic cases")
    model = Path(json.loads(args.soc_runtime.read_text())["model"]).resolve()
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
    if str(agent.device) != "mps":
        raise ValueError("Expected MPS")
    embed = embed_fn_from_agent(agent, max_length=512, batch_size=1)
    warm_started = time.perf_counter()
    embed(["Synthetic encoder warmup query"])
    warmup_ms = (time.perf_counter() - warm_started) * 1000
    elapsed = []
    for row in rows:
        text = row["request"]["search"]["query"]
        started = time.perf_counter()
        vector = embed([text])
        elapsed.append((time.perf_counter() - started) * 1000)
        if vector.shape[0] != 1 or not np.isfinite(vector).all():
            raise ValueError("Invalid query vector")
    model_hash = hashlib.sha256()
    with (model / "model.safetensors").open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            model_hash.update(chunk)
    report = {"kind": "candidate_matrix_one_query_embedding_latency", "synthetic_only": True,
              "device": str(agent.device), "model_version": laya.__version__,
              "model_sha256": model_hash.hexdigest(),
              "queries_sha256": hashlib.sha256(source).hexdigest(), "cases": len(rows),
              "load_seconds": load_seconds, "warmup_ms": warmup_ms,
              "embedding_ms": {"p50": quantile(elapsed, .5),
                               "p95": quantile(elapsed, .95),
                               "p99": quantile(elapsed, .99)}}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: report[key] for key in ("cases", "load_seconds", "warmup_ms",
                                                 "embedding_ms")}))


if __name__ == "__main__":
    main()
