#!/usr/bin/env python3
"""Measure synthetic search -> local choice -> validated seed in one local run."""

import argparse
import hashlib
import json
import math
import os
import subprocess
import time
from collections import defaultdict
from pathlib import Path

from evaluate_laya import PAIRWISE_QUESTION, QUESTION, choice_criteria, read_jsonl
from probe_openrouter_decisions import call as call_openrouter, read_key

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


REMOTE_MODELS = {"jev": "typesafe/jev-1.13", "openai": "openai/gpt-6-luna-decisions"}
CONDITION_QUESTION = {
    "type": "noul",
    "instructions": (
        "Is this memory appropriate to use for the user's exact request? Answer yes only if "
        "its project and task match and every stated condition is established by the request. "
        "Answer no when a condition is unknown, the memory concerns another situation, or "
        "the user explicitly asks not to consult memories."
    ),
}
CONDITION_CHOICE_QUESTION = {
    "type": "choice",
    "instructions": (
        "Choose the one memory appropriate to use for the user's exact request. Its project "
        "and task must match, and every declared condition must be established by the request. "
        "Choose none if a condition is unknown, no candidate matches, or the user asks not "
        "to consult memories. Use only the candidate descriptions."
    ),
}


def quantile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    if not ordered:
        return 0.0
    position = (len(ordered) - 1) * fraction
    lower = int(position)
    upper = min(len(ordered) - 1, lower + 1)
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def campaign_manifest(args: argparse.Namespace, cases: list[dict]) -> dict:
    """Bind a resumable local run to its exact inputs and evaluation code."""
    model_files = {}
    if args.selector.startswith("laya") or args.retrieval == "hybrid":
        runtime = json.loads(args.soc_runtime.read_text())
        model = Path(runtime["model"]).resolve()
        model_files = {
            "weights_sha256": sha256(model / "model.safetensors"),
            "tokenizer_sha256": sha256(model / "tokenizer/tokenizer.json"),
        }
    return {
        "version": 1, "kind": "integrated_local_seed_campaign",
        "split": args.split, "selector": args.selector,
        "retrieval": args.retrieval, "threshold": args.threshold,
        "require_mps": args.require_mps,
        "max_candidates": args.max_candidates,
        "case_ids": [case["id"] for case in cases],
        "queries_sha256": sha256(args.queries),
        "database_sha256": sha256(args.database),
        "binary_sha256": sha256(args.binary),
        "runner_sha256": sha256(Path(__file__)),
        "embedding_batch_sha256": sha256(args.embedding_batch) if args.embedding_batch else None,
        "model": model_files,
    }


def append_campaign_event(path: Path, event: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(event, ensure_ascii=False, separators=(",", ":")) + "\n")
        stream.flush()
        os.fsync(stream.fileno())


def load_campaign_rows(path: Path, manifest: dict) -> list[dict]:
    if not path.exists():
        append_campaign_event(path, {"phase": "manifest", "value": manifest})
        return []
    events = [json.loads(line) for line in path.read_text().splitlines()]
    if not events or events[0] != {"phase": "manifest", "value": manifest}:
        raise ValueError("Campaign journal does not match the current inputs")
    rows = []
    for event in events[1:]:
        if (event.get("phase") != "case" or len(rows) >= len(manifest["case_ids"]) or
            event.get("value", {}).get("id") != manifest["case_ids"][len(rows)]):
            raise ValueError("Campaign journal has a duplicate, missing, or reordered case")
        rows.append(event["value"])
    return rows


def checked_profile(batch_path: Path, model: Path) -> dict:
    batch = json.loads(batch_path.read_text())
    if (batch["model_sha256"] != sha256(model / "model.safetensors") or
        batch["tokenizer_sha256"] != sha256(model / "tokenizer/tokenizer.json") or
        batch["procedure"] != "last-hidden-mean-pool-no-prefix-v1" or
        not isinstance(batch["space_id"], str) or not batch["space_id"]):
        raise ValueError("Embedding profile does not match the SOC Laya snapshot")
    return {"space_id": batch["space_id"], "model_id": batch["model_id"],
            "dimensions": batch["dimensions"], "procedure": batch["procedure"]}


def selected_cases(path: Path, limit: int, case_ids: list[str] | None = None,
                   split: str = "development") -> list[dict]:
    if split not in ("development", "calibration"):
        raise ValueError("Only development or calibration synthetic cases are permitted")
    all_cases = read_jsonl(path)
    by_segment = defaultdict(list)
    for case in all_cases.values():
        if case["split"] == split:
            if case["label_source"] != "deterministic_fixture_rule":
                raise ValueError(f"Only labeled synthetic {split} cases are permitted")
            by_segment[case["segment"]].append(case)
    if not by_segment and not case_ids:
        raise ValueError(f"No synthetic {split} cases")
    selected = []
    if case_ids:
        if len(case_ids) != len(set(case_ids)) or any(identity not in all_cases for identity in case_ids):
            raise ValueError("Unknown or duplicate case ID")
        selected = [all_cases[identity] for identity in case_ids]
        if any(case["split"] != split or
               case["label_source"] != "deterministic_fixture_rule" for case in selected):
            raise ValueError(f"Only labeled synthetic {split} cases are permitted")
    else:
        offset = 0
        segments = sorted(by_segment)
        while len(selected) < limit and segments:
            for segment in list(segments):
                if offset < len(by_segment[segment]):
                    selected.append(by_segment[segment][offset])
                    if len(selected) == limit:
                        break
                else:
                    segments.remove(segment)
            offset += 1
    for case in selected:
        if case["expected_ids"] and (case.get("expected_condition") is None or
                                      case.get("expected_source") is None):
            raise ValueError("Positive cases require condition and source labels")
    return selected


class RemoteAgent:
    def __init__(self, key: str, model: str):
        self.key = key
        self.model = model
        self.last_decision = None
        self.attempted = False

    def predict(self, state: dict, questions: dict) -> dict:
        self.last_decision = None
        question = questions["relevance"]
        self.attempted = True
        decision = call_openrouter(self.key, self.model, state, question["criteria"],
                                   instructions=question["instructions"])
        self.last_decision = decision
        return {"answers": {"relevance": {"choice": decision["chosen_id"],
                                           "probabilities": decision["probabilities"]}},
                "usage": {"input_tokens": decision["input_tokens"]}}


class PairwiseAgent:
    def __init__(self, agent, threshold: float, question: dict = PAIRWISE_QUESTION,
                 include_conditions: bool = False):
        self.agent = agent
        self.threshold = threshold
        self.question = question
        self.include_conditions = include_conditions
        self.last_scores = {}

    def select(self, case: dict, candidates: list[dict]) -> tuple[str, float, int]:
        self.last_scores = {}
        started = time.perf_counter()
        tokens = 0
        for candidate in candidates:
            state = {"user_prompt": case["request"]["query"],
                     "candidate_memory": candidate["summary"],
                     "scope": case["request"]["scope_id"]}
            if self.include_conditions:
                state["candidate_conditions"] = candidate.get("conditions")
            answer = self.agent.predict(state, {"relevance": self.question})
            score = answer["answers"]["relevance"]["noul"]
            usage = answer["usage"]["input_tokens"]
            if (isinstance(score, bool) or not isinstance(score, (int, float)) or
                not math.isfinite(score) or not 0 <= score <= 1 or
                isinstance(usage, bool) or not isinstance(usage, int) or usage < 0):
                raise ValueError(f"Invalid Laya pairwise result for {case['id']}")
            self.last_scores[candidate["memory_id"]] = score
            tokens += usage
        top = max(self.last_scores, key=self.last_scores.get)
        chosen = top if self.last_scores[top] >= self.threshold else "none"
        return chosen, (time.perf_counter() - started) * 1000, tokens


class ConditionChoiceAgent:
    def __init__(self, agent):
        self.agent = agent

    def predict(self, state: dict, questions: dict) -> dict:
        return self.agent.predict(state, questions)


class Session:
    def __init__(self, binary: Path, database: Path):
        env = dict(os.environ, IROMEM_DATABASE_PATH=str(database))
        self.process = subprocess.Popen([str(binary), "session"], stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                        text=True, bufsize=1, env=env)

    def call(self, op: str, request: dict) -> tuple[dict, float]:
        started = time.perf_counter()
        self.process.stdin.write(json.dumps({"op": op, "request": request}, ensure_ascii=False) + "\n")
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        elapsed_ms = (time.perf_counter() - started) * 1000
        if not line:
            raise RuntimeError(f"iromem session terminated before {op}; exit={self.process.poll()}")
        response = json.loads(line)
        if response["status"] != "complete":
            raise ValueError(f"iromem {op} failed: {response['error']}")
        return response, elapsed_ms

    def close(self) -> None:
        self.process.stdin.close()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            self.process.wait(timeout=5)
        if self.process.returncode:
            raise RuntimeError(f"iromem session exited with {self.process.returncode}: "
                               f"{self.process.stderr.read()[:1000]}")
        self.process.stdout.close()
        self.process.stderr.close()


def choose(agent, case: dict, candidates: list[dict], max_candidates: int) -> tuple[str, float, int | None]:
    presented = candidates[:max_candidates]
    if not presented:
        return "none", 0.0, 0
    if agent is None:
        return presented[0]["memory_id"], 0.0, 0
    if isinstance(agent, PairwiseAgent):
        return agent.select(case, presented)
    if isinstance(agent, ConditionChoiceAgent):
        descriptions = [{**candidate, "summary": candidate["summary"] +
                         (f"\nDeclared condition: {candidate['conditions']}" if candidate.get("conditions") else "")}
                        for candidate in presented]
        question = CONDITION_CHOICE_QUESTION
    else:
        descriptions = presented
        question = QUESTION
    criteria, mapping = choice_criteria(descriptions, "letters")
    state = {"user_prompt": case["request"]["query"], "scope": case["request"]["scope_id"]}
    started = time.perf_counter()
    answer = agent.predict(state, {"relevance": dict(question, criteria=criteria)})
    elapsed_ms = (time.perf_counter() - started) * 1000
    decision = answer["answers"]["relevance"]
    probabilities = decision["probabilities"]
    if decision["choice"] not in mapping or set(probabilities) != set(criteria) or any(
        isinstance(value, bool) or not isinstance(value, (int, float)) or
        not math.isfinite(value) or not 0 <= value <= 1 for value in probabilities.values()
    ) or abs(sum(probabilities.values()) - 1) > .01:
        raise ValueError(f"Invalid Laya choice for {case['id']}")
    tokens = answer["usage"]["input_tokens"]
    if tokens is not None and (isinstance(tokens, bool) or not isinstance(tokens, int) or tokens < 0):
        raise ValueError(f"Invalid classifier token usage for {case['id']}")
    return mapping[decision["choice"]], elapsed_ms, tokens


def evaluate_case(session: Session, agent, case: dict, max_candidates: int,
                  embedder=None, profile: dict | None = None) -> dict:
    started = time.perf_counter()
    embedding_ms = 0.0
    if profile is None:
        search_op = "search"
        retrieval_request = case["request"]
    else:
        if embedder is None:
            raise ValueError("Hybrid retrieval needs the query encoder")
        embedding_started = time.perf_counter()
        vector = embedder([case["request"]["query"]])[0]
        embedding_ms = (time.perf_counter() - embedding_started) * 1000
        if len(vector) != profile["dimensions"] or any(not math.isfinite(float(item)) for item in vector):
            raise ValueError(f"Invalid query vector for {case['id']}")
        search_op = "hybrid_search"
        retrieval_request = {"search": case["request"], "space_id": profile["space_id"],
                             "query_vector": vector.astype(float).tolist()}
    found, search_roundtrip_ms = session.call(search_op, retrieval_request)
    candidates = found["response"]["candidates"]
    allowed_ids = {item["memory_id"] for item in candidates}
    if allowed_ids & set(case["forbidden_ids"]):
        raise ValueError(f"Forbidden candidate returned for {case['id']}")
    if isinstance(agent, RemoteAgent):
        agent.last_decision = None
        agent.attempted = False
    if isinstance(agent, PairwiseAgent):
        agent.last_scores = {}
    decision_status = "complete"
    decision_error = None
    decision_started = time.perf_counter()
    try:
        chosen, choice_ms, tokens = choose(agent, case, candidates, max_candidates)
    except (RuntimeError, ValueError, KeyError, TypeError) as error:
        if agent is None:
            raise
        chosen, choice_ms, tokens = "none", (time.perf_counter() - decision_started) * 1000, None
        decision_status = "failed"
        decision_error = f"{type(error).__name__}: {error}"
    provider_decision = agent.last_decision if isinstance(agent, RemoteAgent) else None
    provider_called = agent.attempted if isinstance(agent, RemoteAgent) else False
    presented = [{"memory_id": item["memory_id"], "revision": item["revision"]}
                 for item in candidates[:max_candidates]]
    selected = [item for item in presented if item["memory_id"] == chosen]
    if chosen != "none" and len(selected) != 1:
        raise ValueError(f"Classifier selected an unseen candidate for {case['id']}")
    classifier = (agent.model if isinstance(agent, RemoteAgent) else
                  f"laya_pairwise_conditions_t{agent.threshold}" if isinstance(agent, PairwiseAgent)
                  and agent.include_conditions else
                  f"laya_pairwise_t{agent.threshold}" if isinstance(agent, PairwiseAgent) else
                  "laya_conditions_choice" if isinstance(agent, ConditionChoiceAgent) else
                  "laya_choice_letters" if agent is not None else "deterministic_top1")
    request = {"search": case["request"] if profile is None else None,
               "hybrid": retrieval_request if profile is not None else None,
               "presented": presented,
               "selected": selected, "classifier": classifier,
               "classification_status": decision_status}
    composed, compose_roundtrip_ms = session.call("compose_seed", request)
    total_ms = (time.perf_counter() - started) * 1000
    seed = composed["response"]
    expected_seed_status = "degraded" if decision_status == "failed" else found["response"]["status"]
    if seed["status"] != expected_seed_status:
        raise ValueError(f"Unexpected seed status for {case['id']}")
    memories = seed["memories"]
    chosen_ids = [item["memory_id"] for item in memories]
    if chosen_ids != ([chosen] if chosen != "none" else []):
        raise ValueError(f"Seed differs from validated choice for {case['id']}")
    expected = case["expected_ids"]
    exact = seed["status"] == "complete" and chosen_ids == expected
    positive = bool(expected)
    first = memories[0] if len(memories) == 1 else None
    seed_text = seed["seed_text"] or ""
    fact_fidelity = exact and (case["expected_fact"] in seed_text if positive else not seed_text)
    condition_fidelity = (exact and first is not None and
                          first["conditions"] == case["expected_condition"] and
                          case["expected_condition"] in seed_text) if positive else None
    source_fidelity = (exact and first is not None and first["source"] == case["expected_source"] and
                       case["expected_source"]["ref"] in seed_text) if positive else None
    return {"id": case["id"], "segment": case["segment"], "positive": positive,
            "retrieval_method": found["response"]["method"],
            "retrieval_request": retrieval_request,
            "retrieval_snapshot": found["response"],
            "semantic_coverage": found["response"]["semantic_coverage"],
            "decision_status": decision_status, "decision_error": decision_error,
            "seed_status": seed["status"],
            "provider_called": provider_called,
            "candidate_scores": agent.last_scores if isinstance(agent, PairwiseAgent) else None,
            "served_model": provider_decision.get("served_model") if provider_decision else None,
            "reported_cost_usd": provider_decision.get("reported_cost_usd") if provider_decision else None,
            "expected_in_candidates": bool(set(expected) & {item["memory_id"] for item in presented}),
            "candidate_count": len(candidates), "presented_count": len(presented),
            "chosen_id": chosen, "exact": exact, "fact_fidelity": fact_fidelity,
            "condition_fidelity": condition_fidelity, "source_fidelity": source_fidelity,
            "forbidden_leak": bool(set(chosen_ids) & set(case["forbidden_ids"])),
            "input_tokens": tokens, "total_ms": total_ms, "embedding_ms": embedding_ms,
            "search_roundtrip_ms": search_roundtrip_ms,
            "search_rust_ms": found["elapsed_ms"], "choice_ms": choice_ms,
            "compose_roundtrip_ms": compose_roundtrip_ms, "compose_rust_ms": composed["elapsed_ms"]}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--split", choices=("development", "calibration"), default="development")
    parser.add_argument("--database", type=Path, required=True)
    parser.add_argument("--binary", type=Path, default=Path("target/debug/iromem"))
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    parser.add_argument("--limit", type=int, default=12)
    parser.add_argument("--max-candidates", type=int, default=4)
    parser.add_argument("--selector", choices=("laya", "laya_conditions_choice", "laya_pairwise", "laya_pairwise_conditions",
                                               "top1", "jev", "openai"), default="laya")
    parser.add_argument("--threshold", type=float, default=.5)
    parser.add_argument("--retrieval", choices=("fts5", "hybrid"), default="fts5")
    parser.add_argument("--require-mps", action="store_true",
                        help="Fail instead of silently measuring Laya on CPU")
    parser.add_argument("--embedding-batch", type=Path)
    parser.add_argument("--case-id", action="append")
    parser.add_argument("--journal", type=Path,
                        help="Append-only local case journal for a resumable synthetic campaign")
    parser.add_argument("--max-seconds-per-run", type=float,
                        help="Stop a journaled invocation before the next case when time is exhausted")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    max_limit = 10000 if args.journal and args.selector not in REMOTE_MODELS else 100
    if (not 1 <= args.limit <= max_limit or not 1 <= args.max_candidates <= 20 or
        not math.isfinite(args.threshold) or not 0 <= args.threshold <= 1):
        parser.error(f"limit must be 1..{max_limit} and max-candidates 1..20")
    if args.case_id and len(args.case_id) > args.limit:
        parser.error("--case-id count exceeds --limit")
    cases = selected_cases(args.queries, args.limit, args.case_id, args.split)
    if args.journal and len(cases) != args.limit:
        parser.error("A journaled campaign requires exactly --limit labeled cases")
    if args.selector in REMOTE_MODELS and len(cases) > 4:
        parser.error("Integrated remote pilot is capped at four cases per invocation")
    if not args.database.is_file() or not args.binary.is_file():
        parser.error("database and compiled binary must exist")
    if args.retrieval == "hybrid" and not args.embedding_batch:
        parser.error("--retrieval hybrid requires --embedding-batch")
    if args.require_mps and args.selector == "top1" and args.retrieval == "fts5":
        parser.error("--require-mps needs a Laya selector or hybrid retrieval")
    if args.journal and args.journal.resolve() == args.output.resolve():
        parser.error("journal and output must use different paths")
    if args.output.resolve() in {args.queries.resolve(), args.database.resolve(), args.binary.resolve()}:
        parser.error("output must not overwrite an input or the executable")
    if args.journal and args.selector in REMOTE_MODELS:
        parser.error("The local journal does not cover remote calls; use the bounded remote pilot")
    if args.max_seconds_per_run is not None and (
        not args.journal or not math.isfinite(args.max_seconds_per_run) or
        args.max_seconds_per_run <= 0
    ):
        parser.error("--max-seconds-per-run requires a journal and a positive finite value")
    manifest = campaign_manifest(args, cases) if args.journal else None
    rows = load_campaign_rows(args.journal, manifest) if args.journal else []
    resumed_cases = len(rows)
    invocation_started = time.monotonic()
    agent = None
    encoder_agent = None
    embedder = None
    profile = None
    load_seconds = 0.0
    warmup_ms = 0.0
    model_version = None
    device = None
    if args.selector in ("laya", "laya_conditions_choice", "laya_pairwise", "laya_pairwise_conditions") or args.retrieval == "hybrid":
        runtime = json.loads(args.soc_runtime.read_text())
        model = Path(runtime["model"]).resolve()
        if not all((model / name).is_file() for name in
                   ("model.safetensors", "rl_agent_config.json", "encoder/config.json",
                    "tokenizer/tokenizer.json")):
            raise ValueError("SOC Laya snapshot is incomplete")
        import torch
        import laya
        from transformers.initialization import no_init_weights

        if args.require_mps and not torch.backends.mps.is_available():
            raise RuntimeError("MPS is unavailable; refusing the requested Mac performance run")
        torch.set_num_threads(2)
        load_started = time.perf_counter()
        with no_init_weights():
            encoder_agent = laya.load(str(model), device="mps")
        load_seconds = time.perf_counter() - load_started
        warmup_started = time.perf_counter()
        if args.selector == "laya":
            agent = encoder_agent
            agent.predict({"user_prompt": "Synthetic warmup", "scope": "warmup"},
                          {"relevance": dict(QUESTION, criteria={"none": "No relevant memory",
                                                               "A": "Synthetic memory"})})
        elif args.selector == "laya_conditions_choice":
            agent = ConditionChoiceAgent(encoder_agent)
            agent.predict({"user_prompt": "Synthetic warmup", "scope": "warmup"},
                          {"relevance": dict(CONDITION_CHOICE_QUESTION,
                                             criteria={"none": "No relevant memory",
                                                       "A": "Synthetic memory\nDeclared condition: Synthetic condition"})})
        elif args.selector in ("laya_pairwise", "laya_pairwise_conditions"):
            conditional = args.selector == "laya_pairwise_conditions"
            agent = PairwiseAgent(encoder_agent, args.threshold,
                                  CONDITION_QUESTION if conditional else PAIRWISE_QUESTION,
                                  include_conditions=conditional)
            encoder_agent.predict({"user_prompt": "Synthetic warmup",
                                   "candidate_memory": "Synthetic memory", "scope": "warmup",
                                   "candidate_conditions": "Synthetic condition"} if conditional else
                                  {"user_prompt": "Synthetic warmup", "candidate_memory": "Synthetic memory",
                                   "scope": "warmup"},
                                  {"relevance": agent.question})
        if args.retrieval == "hybrid":
            from laya.shortlist import embed_fn_from_agent
            profile = checked_profile(args.embedding_batch, model)
            for case in cases:
                if len(encoder_agent.tok(case["request"]["query"], add_special_tokens=True)["input_ids"]) > 512:
                    raise ValueError(f"Query exceeds encoder token budget: {case['id']}")
            embedder = embed_fn_from_agent(encoder_agent, max_length=512, batch_size=1)
            embedder(["Synthetic warmup"])
        warmup_ms = (time.perf_counter() - warmup_started) * 1000
        model_version = (getattr(laya, "__version__", "unknown")
                         if args.selector in ("laya", "laya_conditions_choice", "laya_pairwise", "laya_pairwise_conditions") else None)
        device = str(encoder_agent.device)
    if args.selector in REMOTE_MODELS:
        agent = RemoteAgent(read_key(), REMOTE_MODELS[args.selector])
        model_version = agent.model
        device = "openrouter" if encoder_agent is None else f"openrouter+encoder:{encoder_agent.device}"

    if len(rows) < len(cases):
        session = Session(args.binary.resolve(), args.database.resolve())
        try:
            for case in cases[len(rows):]:
                if (args.max_seconds_per_run is not None and
                    time.monotonic() - invocation_started >= args.max_seconds_per_run):
                    break
                row = evaluate_case(session, agent, case, args.max_candidates, embedder, profile)
                if args.journal:
                    append_campaign_event(args.journal, {"phase": "case", "value": row})
                rows.append(row)
        finally:
            session.close()
    if len(rows) < len(cases):
        print(json.dumps({"status": "partial", "completed_cases": len(rows),
                          "planned_cases": len(cases), "journal": str(args.journal)},
                         ensure_ascii=False))
        return
    positive = [row for row in rows if row["positive"]]
    negative = [row for row in rows if not row["positive"]]
    run_kind = "campaign" if args.journal else "pilot"
    report = {"method": f"integrated_{args.retrieval}_{args.selector}_seed_{run_kind}",
              "split": args.split,
              "quantile_method": "linear_interpolation",
              "retrieval": args.retrieval, "embedding_profile": profile,
              "threshold": args.threshold if args.selector.startswith("laya_pairwise") else None,
              "model_version": model_version, "device": device,
              "encoder_model_version": getattr(laya, "__version__", "unknown") if encoder_agent else None,
              "cases": len(rows), "positive": len(positive),
              "negative": len(negative), "resumed_cases": resumed_cases,
              "load_seconds": load_seconds, "warmup_ms": warmup_ms,
              "candidate_coverage": sum(row["expected_in_candidates"] for row in positive),
              "decision_failures": sum(row["decision_status"] == "failed" for row in rows),
              "degraded_seeds": sum(row["seed_status"] == "degraded" for row in rows),
              "provider_calls": sum(row["provider_called"] for row in rows),
              "exact_selection": sum(row["exact"] for row in rows),
              "positive_fact_fidelity": sum(row["fact_fidelity"] for row in positive),
              "positive_condition_fidelity": sum(row["condition_fidelity"] for row in positive),
              "positive_source_fidelity": sum(row["source_fidelity"] for row in positive),
              "negative_empty": sum(row["fact_fidelity"] for row in negative),
              "forbidden_leaks": sum(row["forbidden_leak"] for row in rows),
              "input_tokens": sum(row["input_tokens"] or 0 for row in rows),
              "missing_token_usage": sum(row["input_tokens"] is None for row in rows),
              "reported_cost_usd": sum(row["reported_cost_usd"] or 0 for row in rows),
              "missing_reported_cost": sum(row["reported_cost_usd"] is None and
                                           row["provider_called"] and
                                           row["decision_status"] == "complete" for row in rows),
              "latency_ms": {key: {"p50": quantile([row[key] for row in rows], .5),
                                   "p95": quantile([row[key] for row in rows], .95)}
                             for key in ("total_ms", "embedding_ms", "search_roundtrip_ms", "search_rust_ms",
                                         "choice_ms", "compose_roundtrip_ms", "compose_rust_ms")},
              "individual_results": rows}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key != "individual_results"},
                     ensure_ascii=False))


if __name__ == "__main__":
    main()
