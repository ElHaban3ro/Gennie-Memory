#!/usr/bin/env python3
"""Import LoCoMo session memories through mem-write and bind all QA evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import uuid
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPOSITORY = "https://github.com/snap-research/LoCoMo"
REVISION = "3eb6f2c585f5e1699204e3c3bdf7adc5c28cb376"
SOURCE_URL = f"{REPOSITORY}/blob/{REVISION}/data/locomo10.json"
NAMESPACE = uuid.UUID("2cc06631-a8d8-5f1a-96cc-03f136c5756d")
TURN_ID = re.compile(r"D\d+:\d+")
EVIDENCE_SEPARATORS = " \t\r\n,;|"
BENCHMARK_AS_OF = "2026-10-24T00:00:00.000Z"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def session_number(key: str) -> int | None:
    match = re.fullmatch(r"session_(\d+)", key)
    return int(match.group(1)) if match else None


def rfc3339_session_time(value: Any) -> str:
    if not isinstance(value, str):
        raise ValueError("falta fecha de sesión")
    parsed = datetime.strptime(value, "%I:%M %p on %d %B, %Y").replace(tzinfo=timezone.utc)
    return parsed.isoformat(timespec="milliseconds").replace("+00:00", "Z")


def make_session_records(sample: dict[str, Any]) -> tuple[list[dict[str, Any]], dict[str, str]]:
    conversation = sample.get("conversation")
    if not isinstance(conversation, dict):
        raise ValueError("conversation debe ser un objeto")
    sample_id = sample.get("sample_id")
    if not isinstance(sample_id, str) or not re.fullmatch(r"conv-[A-Za-z0-9-]+", sample_id):
        raise ValueError("sample_id inválido")

    records: list[dict[str, Any]] = []
    turn_to_operation: dict[str, str] = {}
    canonical_turns: dict[str, set[str]] = {}
    scope_id = f"locomo-{sample_id}"
    session_times: list[str] = []

    for key in sorted((key for key in conversation if session_number(key) is not None), key=session_number):
        number = session_number(key)
        turns = conversation[key]
        summary_key = f"session_{number}_summary"
        summary = sample.get("session_summary", {}).get(summary_key)
        timestamp = rfc3339_session_time(conversation.get(f"{summary_key.removesuffix('_summary')}_date_time"))
        if not isinstance(turns, list) or not turns:
            raise ValueError(f"{sample_id}/{key}: sesión vacía o inválida")
        if not isinstance(summary, str) or not summary.strip():
            raise ValueError(f"{sample_id}/{key}: falta session_summary")

        rendered_turns: list[str] = []
        operation_id = str(uuid.uuid5(NAMESPACE, f"locomo-write-v1:{sample_id}:{number}"))
        for turn in turns:
            if not isinstance(turn, dict):
                raise ValueError(f"{sample_id}/{key}: turno inválido")
            dia_id, speaker, text = turn.get("dia_id"), turn.get("speaker"), turn.get("text")
            if not isinstance(dia_id, str) or not re.fullmatch(r"D\d+:\d+", dia_id):
                raise ValueError(f"{sample_id}/{key}: dia_id inválido")
            if not isinstance(speaker, str) or not speaker.strip() or not isinstance(text, str) or not text.strip():
                raise ValueError(f"{sample_id}/{key}/{dia_id}: speaker o texto vacío")
            if dia_id in turn_to_operation:
                raise ValueError(f"{sample_id}: dia_id duplicado: {dia_id}")
            turn_to_operation[dia_id] = operation_id
            parts = dia_id[1:].split(":")
            canonical_turns.setdefault(f"D{int(parts[0])}:{int(parts[1])}", set()).add(dia_id)
            rendered_turns.append(f"{speaker}: {text}")

        original_text = "\n".join(rendered_turns)
        locator = f"sample_id={sample_id}; session={key}; timestamp={timestamp}"
        request = {
            "schema_version": 1,
            "operation_id": operation_id,
            "action": "create",
            "scope": {"kind": "conversation", "id": scope_id},
            "memory": {
                "original_text": original_text,
                "summary": summary.strip(),
                "type": "event",
                "source": {"kind": "document", "ref": SOURCE_URL, "locator": locator},
                "conditions": None,
                "valid_from": None,
                "valid_until": None,
            },
        }
        if len(original_text) > 16000 or len(summary.strip()) > 1600:
            raise ValueError(f"{sample_id}/{key}: texto fuera de límites de mem-write")
        records.append(request)
        session_times.append(timestamp)

    # A QA question's as_of is the end of its own conversation, not a claim
    # about when a source utterance happened in UTC.
    latest_time = max(session_times, key=lambda item: datetime.fromisoformat(item.replace("Z", "+00:00")))
    turn_to_session = {
        turn: operation for turn, operation in turn_to_operation.items()
    }
    sample["_locomo_scope_id"] = scope_id
    sample["_locomo_as_of"] = latest_time
    sample["_locomo_turn_to_operation"] = turn_to_session
    sample["_locomo_canonical_turns"] = {key: sorted(value) for key, value in canonical_turns.items()}
    return records, turn_to_operation


def resolve_evidence(raw_evidence: Any, sample: dict[str, Any]) -> tuple[list[str], list[str], list[str]]:
    if not isinstance(raw_evidence, list):
        raise ValueError("qa.evidence debe ser una lista")
    if not raw_evidence:
        return [], [], []

    turn_to_operation = sample["_locomo_turn_to_operation"]
    canonical_turns = sample["_locomo_canonical_turns"]
    operations: list[str] = []
    resolved_turns: list[str] = []
    unresolved: list[str] = []
    for raw in raw_evidence:
        if not isinstance(raw, str):
            unresolved.append(str(raw))
            continue
        tokens = TURN_ID.findall(raw)
        remainder = TURN_ID.sub("", raw).strip(EVIDENCE_SEPARATORS)
        if remainder or not tokens:
            unresolved.append(raw)
        for token in tokens:
            target = token if token in turn_to_operation else None
            if target is None:
                parts = token[1:].split(":")
                canonical = f"D{int(parts[0])}:{int(parts[1])}"
                candidates = canonical_turns.get(canonical, [])
                if len(candidates) == 1:
                    target = candidates[0]
                else:
                    unresolved.append(token)
                    continue
            resolved_turns.append(target)
            operation = turn_to_operation[target]
            if operation not in operations:
                operations.append(operation)
    return operations, resolved_turns, unresolved


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")


def import_batch(database: Path, requests_path: Path) -> list[dict[str, Any]]:
    repo_root = Path(__file__).resolve().parents[2]
    env = dict(__import__("os").environ)
    env["GENNIE_MEMORY_DATABASE_PATH"] = str(database.resolve())
    process = subprocess.run(
        ["cargo", "run", "--offline", "-p", "gennie-memory", "--", "mem-write-batch", str(requests_path.resolve())],
        cwd=repo_root,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if process.returncode != 0:
        raise RuntimeError(process.stderr[-8000:] or f"mem-write-batch terminó con {process.returncode}")
    receipts = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
    return receipts


def run(source: Path, output_dir: Path, database: Path) -> dict[str, Any]:
    source_rows = json.loads(source.read_text(encoding="utf-8"))
    if not isinstance(source_rows, list) or len(source_rows) != 10:
        raise ValueError("locomo10.json debe contener las diez conversaciones publicadas")

    requests: list[dict[str, Any]] = []
    question_rows: list[dict[str, Any]] = []
    prepared: list[tuple[dict[str, Any], list[dict[str, Any]]]] = []
    operation_to_locator: dict[str, dict[str, str]] = {}
    for sample in source_rows:
        session_requests, _ = make_session_records(sample)
        for request in session_requests:
            locator = request["memory"]["source"]["locator"]
            operation_to_locator[request["operation_id"]] = {
                "sample_id": sample["sample_id"],
                "session": locator.split("; ")[1].split("=", 1)[1],
            }
        requests.extend(session_requests)
        prepared.append((sample, session_requests))

    # mem-write-batch is bounded at 2,500 operations; this dataset's 272
    # session memories fit in one transactionally idempotent batch.
    if not requests or len(requests) > 2500:
        raise ValueError("el número de sesiones está fuera del límite de mem-write-batch")
    requests_path = output_dir / "mem-write-requests.jsonl"
    write_jsonl(requests_path, requests)
    receipts = import_batch(database, requests_path)
    if len(receipts) != len(requests):
        raise RuntimeError(f"recibos incompletos: {len(receipts)}/{len(requests)}")
    receipt_by_operation = {row.get("operation_id"): row for row in receipts}
    if set(receipt_by_operation) != set(operation_to_locator):
        raise RuntimeError("los recibos no corresponden exactamente a las operaciones preparadas")
    if any(row.get("status") != "committed" for row in receipts):
        raise RuntimeError("al menos una escritura no se confirmó")

    write_jsonl(output_dir / "mem-write-receipts.jsonl", receipts)
    memory_corpus = []
    for request in requests:
        receipt = receipt_by_operation[request["operation_id"]]
        memory = request["memory"]
        scope = request["scope"]
        memory_corpus.append({
            "id": receipt["memory_id"],
            "summary": memory["summary"],
            "original_text": memory["original_text"],
            "scope_kind": scope["kind"],
            "scope_id": scope["id"],
        })
    memory_corpus_path = output_dir / "memory-corpus.json"
    memory_corpus_path.write_text(
        json.dumps(memory_corpus, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    all_cases: list[dict[str, Any]] = []
    searchable_cases: list[dict[str, Any]] = []
    unresolved_cases: list[dict[str, Any]] = []
    statuses: Counter[str] = Counter()
    for sample, session_requests in prepared:
        operation_ids = {request["operation_id"] for request in session_requests}
        for index, qa in enumerate(sample.get("qa", []), start=1):
            if not isinstance(qa, dict) or not isinstance(qa.get("question"), str):
                raise ValueError(f"{sample['sample_id']}: pregunta inválida")
            answer = qa.get("answer")
            adversarial_answer = qa.get("adversarial_answer")
            if answer is not None and not isinstance(answer, (str, int, float, bool)):
                raise ValueError(f"{sample['sample_id']}: answer tiene tipo inválido")
            if adversarial_answer is not None and not isinstance(adversarial_answer, str):
                raise ValueError(f"{sample['sample_id']}: adversarial_answer tiene tipo inválido")
            expected_operations, resolved_turns, unresolved_refs = resolve_evidence(qa.get("evidence"), sample)
            if not set(expected_operations).issubset(operation_ids):
                raise ValueError(f"{sample['sample_id']}: evidencia enlazada a otra conversación")
            raw_evidence = qa["evidence"]
            status = "no_evidence" if not raw_evidence else ("unresolved" if unresolved_refs else "complete")
            statuses[status] += 1
            case_id = f"locomo-{sample['sample_id']}-q{index:03d}"
            memory_ids = [receipt_by_operation[op]["memory_id"] for op in expected_operations]
            row = {
                "id": case_id,
                "sample_id": sample["sample_id"],
                "scope_id": sample["_locomo_scope_id"],
                "question": qa["question"],
                "answer": answer,
                "adversarial_answer": adversarial_answer,
                "answer_kind": "adversarial" if qa.get("category") == 5 else "standard",
                "category": qa.get("category"),
                "evidence": raw_evidence,
                "resolved_evidence_turns": resolved_turns,
                "expected_operation_ids": expected_operations,
                "expected_memory_ids": memory_ids,
                "evidence_status": status,
                "unresolved_evidence": unresolved_refs,
            }
            all_cases.append(row)
            if status == "unresolved":
                unresolved_cases.append(row)
                continue
            search_case = {
                "id": case_id,
                "split": "locomo_full_v1",
                "segment": str(qa.get("category", "unknown")),
                "expected_ids": memory_ids,
                "forbidden_ids": [],
                "label_source": "external_benchmark_annotation",
                "selection_semantics": "any_evidence_memory",
                "request": {
                    "search": {
                        "query": qa["question"],
                        "scope_kind": "conversation",
                        "scope_id": sample["_locomo_scope_id"],
                        "as_of": BENCHMARK_AS_OF,
                        "limit": 20,
                        "query_mode": "keywords",
                    }
                },
            }
            searchable_cases.append(search_case)

    total_questions = sum(len(sample.get("qa", [])) for sample in source_rows)
    if len(all_cases) != total_questions:
        raise RuntimeError(f"no se conservaron todas las preguntas: {len(all_cases)}/{total_questions}")
    write_jsonl(output_dir / "all-questions.jsonl", all_cases)
    write_jsonl(output_dir / "search-cases.jsonl", searchable_cases)
    write_jsonl(output_dir / "unresolved-evidence.jsonl", unresolved_cases)

    manifest = {
        "dataset": "LoCoMo",
        "dataset_revision": REVISION,
        "source_url": SOURCE_URL,
        "source_sha256": sha256(source),
        "memory_granularity": "conversation_session",
        "memory_count": len(requests),
        "memory_corpus_sha256": sha256(memory_corpus_path),
        "conversation_count": len(source_rows),
        "question_count": total_questions,
        "search_case_count": len(searchable_cases),
        "evidence_status_counts": dict(sorted(statuses.items())),
        "database_path": str(database),
        "mem_write_batch_idempotent": True,
    }
    (output_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return manifest


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=root / "LoCoMo/data/locomo10.json")
    parser.add_argument("--output-dir", type=Path, default=root / ".local/locomo")
    parser.add_argument("--database", type=Path, default=root / ".local/locomo/write-validation.db")
    args = parser.parse_args()
    try:
        manifest = run(args.source, args.output_dir, args.database)
    except (OSError, ValueError, RuntimeError, json.JSONDecodeError, subprocess.SubprocessError) as error:
        print(f"prepare_locomo: {error}", file=sys.stderr)
        return 1
    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
