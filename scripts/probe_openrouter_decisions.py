#!/usr/bin/env python3
"""Run a tiny synthetic-only OpenRouter Decisions probe with a strict call cap."""

import argparse
import json
import math
import os
import time
import urllib.error
import urllib.request
from pathlib import Path

ENDPOINT = "https://openrouter.ai/api/alpha/decisions"
MODELS = ("typesafe/jev-1.14", "typesafe/jev-1.13", "openai/gpt-6-luna-decisions")
QUESTION = ("Choose the single current memory that directly answers the user's "
            "specific project and task. Choose none if no candidate is relevant. "
            "Do not invent a fact beyond the candidate summaries.")


def read_key() -> str:
    value = os.environ.get("IROMEM_OPENROUTER_API_KEY", "")
    if not value:
        path = Path(".env")
        if path.exists():
            for line in path.read_text().splitlines():
                key, separator, contents = line.partition("=")
                if separator and key.strip() == "IROMEM_OPENROUTER_API_KEY":
                    value = contents.strip().strip("\"'")
                    break
    if not value or any(char.isspace() for char in value):
        raise ValueError("IROMEM_OPENROUTER_API_KEY missing or invalid")
    return value


def read_jsonl(path: Path) -> dict[str, dict]:
    rows = {}
    for line in path.read_text().splitlines():
        item = json.loads(line)
        if item["id"] in rows:
            raise ValueError("Duplicate case ID")
        rows[item["id"]] = item
    return rows


def search_request(case: dict) -> dict:
    request = case["request"]
    return request["search"] if "search" in request else request


def call(key: str, model: str, state: dict, criteria: dict, instructions: str = QUESTION) -> dict:
    payload = {"model": model, "state": state,
               "questions": {"relevance": {"type": "choice", "instructions": instructions,
                                           "criteria": criteria}}}
    request = urllib.request.Request(
        ENDPOINT, data=json.dumps(payload, ensure_ascii=False).encode(), method="POST",
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json",
                 "X-Title": "iROMEM synthetic benchmark"})
    started = time.perf_counter()
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            body = json.load(response)
    except urllib.error.HTTPError as error:
        raise RuntimeError(f"OpenRouter HTTP {error.code}") from None
    except urllib.error.URLError as error:
        raise RuntimeError(f"OpenRouter transport error: {type(error.reason).__name__}") from None
    except OSError as error:
        raise RuntimeError(f"OpenRouter transport error: {type(error).__name__}; outcome unknown") from None
    elapsed_ms = (time.perf_counter() - started) * 1000
    return parse_decision(body, criteria, model, elapsed_ms)


def parse_decision(body: dict, criteria: dict, model: str, elapsed_ms: float) -> dict:
    answers = body.get("answers", {})
    answer = answers.get("relevance") if isinstance(answers, dict) else None
    if not isinstance(answer, dict) or answer.get("type") != "choice":
        raise ValueError("OpenRouter returned no valid choice answer")
    chosen = answer.get("choice")
    probabilities = answer.get("probabilities")
    if chosen not in criteria or not isinstance(probabilities, dict) or set(probabilities) != set(criteria):
        raise ValueError("OpenRouter returned invalid choice IDs")
    if any(isinstance(value, bool) or not isinstance(value, (int, float)) or
           not math.isfinite(value) or not 0 <= value <= 1 for value in probabilities.values()):
        raise ValueError("OpenRouter returned invalid probabilities")
    if abs(sum(probabilities.values()) - 1) > .02:
        raise ValueError("OpenRouter probabilities do not sum to one")
    usage = body.get("usage", {})
    if not isinstance(usage, dict):
        raise ValueError("OpenRouter returned invalid usage")
    for name in ("input_tokens", "output_tokens"):
        value = usage.get(name)
        if value is not None and (isinstance(value, bool) or not isinstance(value, int) or value < 0):
            raise ValueError(f"OpenRouter returned invalid {name}")
    cost = usage.get("cost")
    if cost is not None and (isinstance(cost, bool) or not isinstance(cost, (int, float)) or
                             not math.isfinite(cost) or cost < 0):
        raise ValueError("OpenRouter returned invalid cost")
    return {"requested_model": model, "served_model": body.get("model"),
            "chosen_id": chosen, "probability": probabilities[chosen],
            "probabilities": probabilities, "elapsed_ms": elapsed_ms,
            "input_tokens": usage.get("input_tokens"),
            "output_tokens": usage.get("output_tokens"),
            "reported_cost_usd": cost, "reason": None}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--case-id", action="append", required=True)
    parser.add_argument("--model", action="append", choices=MODELS, required=True)
    parser.add_argument("--max-candidates", type=int, default=4)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.max_candidates <= 8 or len(args.case_id) * len(args.model) > 4:
        parser.error("Probe is capped at four provider calls and eight candidates")
    queries, results = read_jsonl(args.queries), read_jsonl(args.results)
    key = read_key()
    measured = []
    for case_id in args.case_id:
        case, result = queries[case_id], results[case_id]
        if case["split"] != "development" or case["label_source"] != "deterministic_fixture_rule":
            raise ValueError("Only deterministic development cases are allowed")
        candidates = result["response"]["candidates"][:args.max_candidates]
        criteria = {candidate["memory_id"]: candidate["summary"] for candidate in candidates}
        if not candidates:
            for model in args.model:
                measured.append({"requested_model": model, "case_id": case_id,
                                 "status": "skipped_empty", "chosen_id": "none",
                                 "candidate_ids": [],
                                 "expected_ids": case["expected_ids"],
                                 "correct": not case["expected_ids"],
                                 "reported_cost_usd": 0.0, "reason": None})
            continue
        criteria["none"] = "No candidate memory is relevant to this request."
        query_request = search_request(case)
        state = {"user_prompt": query_request["query"],
                 "scope": query_request["scope_id"]}
        for model in args.model:
            try:
                row = call(key, model, state, criteria)
            except (RuntimeError, ValueError) as error:
                measured.append({"requested_model": model, "case_id": case_id,
                                 "status": "failed", "candidate_ids": list(criteria)[:-1],
                                 "error": str(error)})
                continue
            row.update(case_id=case_id, expected_ids=case["expected_ids"],
                       candidate_ids=list(criteria)[:-1],
                       status="complete",
                       correct=row["chosen_id"] in case["expected_ids"] if case["expected_ids"] else row["chosen_id"] == "none")
            measured.append(row)
    calls = sum(row["status"] in ("complete", "failed") for row in measured)
    report = {"method": "openrouter_decisions_choice_probe", "synthetic_only": True,
              "endpoint": ENDPOINT, "max_candidates": args.max_candidates,
              "calls": calls, "cases": len(measured), "results": measured}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"calls": calls, "results": [
        {key: row[key] for key in ("requested_model", "status", "served_model", "case_id", "chosen_id",
                                  "correct", "elapsed_ms", "input_tokens", "output_tokens", "reported_cost_usd", "error")
         if key in row}
        for row in measured]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
