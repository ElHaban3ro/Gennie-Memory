#!/usr/bin/env python3
"""Run a bounded, read-only local Laya reranking pilot on synthetic cases."""

import argparse
import json
import math
import os
import time
from collections import defaultdict
from pathlib import Path

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")

QUESTION = {
    "type": "choice",
    "instructions": (
        "Choose the single current memory that directly answers the user's request. "
        "Choose none if no candidate is about the requested project and task. "
        "Use only the visible candidate summaries; do not invent a fact."
    ),
}


def choice_criteria(candidates: list[dict], labels: str) -> tuple[dict[str, str], dict[str, str]]:
    memory_ids = [candidate["memory_id"] for candidate in candidates]
    if "none" in memory_ids or len(memory_ids) != len(set(memory_ids)):
        raise ValueError("Duplicate or reserved candidate ID")
    if labels == "letters":
        options = [chr(ord("A") + index) for index in range(len(candidates))]
    elif labels == "memory_id":
        options = memory_ids
    else:
        raise ValueError("Unknown choice labels")
    if "none" in options or len(options) != len(set(options)):
        raise ValueError("Duplicate or reserved candidate ID")
    criteria = {label: candidate["summary"] for label, candidate in zip(options, candidates)}
    criteria["none"] = "No candidate memory is relevant to this request."
    mapping = {label: candidate["memory_id"] for label, candidate in zip(options, candidates)}
    mapping["none"] = "none"
    return criteria, mapping
PAIRWISE_QUESTION = {
    "type": "noul",
    "instructions": (
        "Does the candidate memory directly provide the current instruction needed for "
        "this user's specific project and task? Answer yes only when both match."
    ),
}


def quantile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * fraction))] if ordered else 0.0


def read_jsonl(path: Path) -> dict[str, dict]:
    rows = {}
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        value = json.loads(line)
        if value["id"] in rows:
            raise ValueError(f'Duplicate ID: {value["id"]}')
        rows[value["id"]] = value
    return rows


def search_request(case: dict) -> dict:
    request = case["request"]
    return request["search"] if "search" in request else request


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    parser.add_argument("--split", choices=("development", "calibration", "final"), default="development")
    parser.add_argument("--limit", type=int, default=8)
    parser.add_argument("--max-candidates", type=int, default=4)
    parser.add_argument("--mode", choices=("choice", "pairwise"), default="pairwise")
    parser.add_argument("--choice-labels", choices=("memory_id", "letters"), default="memory_id")
    parser.add_argument("--threshold", type=float, default=.5)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.limit <= 100000 or not 1 <= args.max_candidates <= 8 or not 0 <= args.threshold <= 1:
        parser.error("limit must be positive and max-candidates between 1 and 8")

    queries = read_jsonl(args.queries)
    results = read_jsonl(args.results)
    by_segment = defaultdict(list)
    for result in results.values():
        if result["split"] == args.split:
            by_segment[result["segment"]].append(result)
    selected = []
    segments = sorted(by_segment)
    step = {segment: max(1, len(by_segment[segment]) // max(1, (args.limit + len(segments) - 1) // len(segments)))
            for segment in segments}
    offset = 0
    while len(selected) < args.limit and segments:
        for segment in list(segments):
            index = offset * step[segment]
            if index < len(by_segment[segment]):
                selected.append(by_segment[segment][index])
                if len(selected) == args.limit:
                    break
            else:
                segments.remove(segment)
        offset += 1
    if not selected:
        raise ValueError("No cases in selected split")
    runtime = json.loads(args.soc_runtime.read_text())
    model = Path(runtime["model"]).resolve()
    if not all((model / name).is_file() for name in ("model.safetensors", "rl_agent_config.json",
                                                    "encoder/config.json", "tokenizer/tokenizer.json")):
        raise ValueError("SOC Laya snapshot is incomplete")

    import torch
    import laya
    from transformers.initialization import no_init_weights

    torch.set_num_threads(2)
    load_started = time.perf_counter()
    with no_init_weights():
        agent = laya.load(str(model), device="mps")
    load_seconds = time.perf_counter() - load_started
    warmup_started = time.perf_counter()
    if args.mode == "choice":
        agent.predict({"user_prompt": "Synthetic warmup", "scope": "warmup"},
                      {"relevance": dict(QUESTION, criteria={"none": "No relevant memory",
                                                           "synthetic": "Synthetic memory"})})
    else:
        agent.predict({"user_prompt": "Synthetic warmup", "candidate_memory": "Synthetic memory",
                       "scope": "warmup"}, {"relevance": PAIRWISE_QUESTION})
    warmup_ms = (time.perf_counter() - warmup_started) * 1000

    measured = []
    for result in selected:
        case = queries[result["id"]]
        query_request = search_request(case)
        candidates = result["response"]["candidates"][:args.max_candidates]
        started = time.perf_counter()
        status = "complete"
        if not candidates:
            chosen = "none"
            probability = 1.0
            token_count = 0
            scores = {"none": 1.0}
            status = "skipped_empty"
        elif args.mode == "choice":
            criteria, mapping = choice_criteria(candidates, args.choice_labels)
            state = {"user_prompt": query_request["query"],
                     "scope": query_request["scope_id"]}
            answer = agent.predict(state, {"relevance": dict(QUESTION, criteria=criteria)})
            decision = answer["answers"]["relevance"]
            label = decision["choice"]
            probabilities = decision["probabilities"]
            if label not in criteria or set(probabilities) != set(criteria) or any(
                isinstance(value, bool) or not isinstance(value, (float, int)) or
                not math.isfinite(value) or not 0 <= value <= 1 for value in probabilities.values()
            ) or abs(sum(probabilities.values()) - 1) > .01:
                raise ValueError(f'Invalid Laya decision for {result["id"]}')
            chosen = mapping[label]
            probability = probabilities[label]
            token_count = answer["usage"]["input_tokens"]
            scores = {mapping[key]: value for key, value in probabilities.items()}
        else:
            scores = {}
            token_count = 0
            for candidate in candidates:
                state = {"user_prompt": query_request["query"],
                         "candidate_memory": candidate["summary"],
                         "scope": query_request["scope_id"]}
                answer = agent.predict(state, {"relevance": PAIRWISE_QUESTION})
                score = answer["answers"]["relevance"]["noul"]
                if isinstance(score, bool) or not isinstance(score, (float, int)) or not math.isfinite(score) or not 0 <= score <= 1:
                    raise ValueError(f'Invalid Laya score for {result["id"]}')
                scores[candidate["memory_id"]] = score
                token_count += answer["usage"]["input_tokens"]
            top = max(scores, key=scores.get) if scores else "none"
            chosen = top if top != "none" and scores[top] >= args.threshold else "none"
            probability = scores.get(chosen, 1 - max(scores.values())) if scores else 1.0
        elapsed_ms = (time.perf_counter() - started) * 1000
        expected = case["expected_ids"]
        measured.append({"id": result["id"], "split": result["split"],
                         "segment": result["segment"], "status": status, "chosen_id": chosen,
                         "candidate_ids": [candidate["memory_id"] for candidate in candidates],
                         "expected_ids": expected, "candidate_count": len(candidates),
                         "expected_in_candidates": bool(set(expected) & {c["memory_id"] for c in candidates}),
                         "correct": chosen in expected if expected else chosen == "none",
                         "probability": probability, "scores": scores, "elapsed_ms": elapsed_ms,
                         "input_tokens": token_count,
                         "reason": None})
    report = {
        "method": f"laya_{args.mode}_rerank_pilot",
        "model_path": str(model),
        "model_version": getattr(laya, "__version__", "unknown"),
        "device": str(agent.device),
        "split": args.split,
        "max_candidates": args.max_candidates,
        "choice_labels": args.choice_labels if args.mode == "choice" else None,
        "threshold": args.threshold if args.mode == "pairwise" else None,
        "load_seconds": load_seconds,
        "warmup_ms": warmup_ms,
        "cases": len(measured),
        "candidate_coverage": sum(row["expected_in_candidates"] for row in measured if row["expected_ids"]) /
            max(1, sum(bool(row["expected_ids"]) for row in measured)),
        "accuracy": sum(row["correct"] for row in measured) / len(measured),
        "inference_ms": {"p50": quantile([row["elapsed_ms"] for row in measured], .5),
                         "p95": quantile([row["elapsed_ms"] for row in measured], .95)},
        "input_tokens_total": sum(row["input_tokens"] for row in measured),
        "individual_results": measured,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key != "individual_results"},
                     ensure_ascii=False))


if __name__ == "__main__":
    main()
