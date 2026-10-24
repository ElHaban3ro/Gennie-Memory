#!/usr/bin/env python3
"""Prepare a bounded, deterministic HappyDB import for Gennie Memory."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
from collections import defaultdict
from pathlib import Path
from typing import Any

MAX_MEMORIES = 2500
MAX_PER_PERSON = 10
SEED = "gennie-memory-happydb-v1"


def digest(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def operation_uuid(hmid: str) -> str:
    """Stable UUID-shaped operation key so a batch can be retried safely."""
    value = bytearray(hashlib.sha256(f"gennie-happydb-write-v1:{hmid}".encode()).digest()[:16])
    value[6] = (value[6] & 0x0F) | 0x50
    value[8] = (value[8] & 0x3F) | 0x80
    raw = value.hex()
    return f"{raw[:8]}-{raw[8:12]}-{raw[12:16]}-{raw[16:20]}-{raw[20:]}"


def ordered(value: str) -> bytes:
    return hashlib.sha256(f"{SEED}:{value}".encode("utf-8")).digest()


def read_rows(path: Path) -> list[dict[str, str]]:
    people: dict[str, list[dict[str, str]]] = defaultdict(list)
    seen_ids: set[str] = set()
    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        required = {"hmid", "wid", "original_hm", "cleaned_hm", "reflection_period"}
        if reader.fieldnames is None or not required.issubset(reader.fieldnames):
            raise ValueError("cleaned_hm.csv no contiene todas las columnas esperadas")
        for row in reader:
            hmid, wid = (row.get(name, "").strip() for name in ("hmid", "wid"))
            original = row.get("original_hm", "")
            cleaned = row.get("cleaned_hm", "").strip()
            if not hmid.isdigit() or not wid.isdigit() or not original.strip() or not cleaned:
                continue
            if hmid in seen_ids:
                raise ValueError(f"hmid duplicado en HappyDB: {hmid}")
            seen_ids.add(hmid)
            people[wid].append(row)
    for rows in people.values():
        rows.sort(key=lambda row: ordered(row["hmid"].strip()))
    return people


def choose(people: dict[str, list[dict[str, str]]], limit: int) -> list[dict[str, str]]:
    eligible = [
        (wid, rows[:MAX_PER_PERSON])
        for wid, rows in people.items()
        if len(rows) >= 2
    ]
    eligible.sort(key=lambda item: ordered(item[0]))
    contributors = []
    capacity = 0
    for item in eligible:
        contributors.append(item)
        capacity += len(item[1])
        if capacity >= limit:
            break
    if capacity < limit:
        raise ValueError(f"HappyDB solo permite seleccionar {capacity} filas elegibles")
    selected: list[dict[str, str]] = []
    for round_index in range(MAX_PER_PERSON):
        for _, rows in contributors:
            if round_index < len(rows):
                selected.append(rows[round_index])
                if len(selected) == limit:
                    return selected
    if len(selected) < limit:
        raise ValueError(f"HappyDB solo permite seleccionar {len(selected)} filas elegibles")
    return selected


def load_rewrites(path: Path | None) -> dict[str, dict[str, Any]]:
    rewrites: dict[str, dict[str, Any]] = {}
    if path is None:
        return rewrites
    with path.open(encoding="utf-8") as handle:
        for line_no, line in enumerate(handle, 1):
            if not line.strip():
                continue
            value = json.loads(line)
            hmid = str(value.get("hmid", "")).strip()
            summary = value.get("agent_rewrite")
            if not hmid.isdigit() or not isinstance(summary, str) or not summary.strip():
                raise ValueError(f"reescritura inválida en línea {line_no}")
            if hmid in rewrites:
                raise ValueError(f"reescritura duplicada para hmid {hmid}")
            rewrites[hmid] = value
    return rewrites


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source",
        type=Path,
        default=Path("HappyDB/happydb/data/cleaned_hm.csv"),
        help="CSV cleaned_hm.csv del repositorio HappyDB",
    )
    parser.add_argument("--limit", type=int, default=MAX_MEMORIES)
    parser.add_argument(
        "--rewrites",
        type=Path,
        required=True,
        help="JSONL requerido con una reinterpretación fiel por hmid seleccionado",
    )
    parser.add_argument(
        "--review-output",
        type=Path,
        help="JSONL local con fuente y reinterpretación para revisión humana",
    )
    parser.add_argument("--output", type=Path, required=True, help="JSONL de solicitudes para mem-write-batch")
    args = parser.parse_args()
    if not 1 <= args.limit <= MAX_MEMORIES:
        parser.error(f"--limit debe estar entre 1 y {MAX_MEMORIES}")

    people = read_rows(args.source)
    selected = choose(people, args.limit)
    rewrites = load_rewrites(args.rewrites)
    ids = {row["hmid"].strip() for row in selected}
    unexpected = set(rewrites) - ids
    if unexpected:
        raise ValueError(f"hay reescrituras fuera de la muestra: {sorted(unexpected)[:5]}")
    missing = ids - set(rewrites)
    if missing:
        raise ValueError(f"faltan reinterpretaciones de agente para {len(missing)} recuerdos; primeros hmid: {sorted(missing)[:10]}")

    requests = []
    annotations = []
    for row in selected:
        hmid = row["hmid"].strip()
        wid = row["wid"].strip()
        original = row["original_hm"]
        cleaned = row["cleaned_hm"].strip()
        annotation = rewrites.get(hmid)
        agent_rewrite = annotation["agent_rewrite"]
        if agent_rewrite is not None and (not isinstance(agent_rewrite, str) or not agent_rewrite.strip()):
            raise ValueError(f"reescritura vacía para hmid {hmid}")
        summary = agent_rewrite.strip()
        if len(original) > 16000 or len(summary) > 1600:
            raise ValueError(f"texto fuera de límites de mem-write para hmid {hmid}")
        scope_id = f"happydb-person-{digest(wid)[:20]}"
        request = {
            "schema_version": 1,
            "operation_id": operation_uuid(hmid),
            "action": "create",
            "scope": {"kind": "user", "id": scope_id},
            "memory": {
                "original_text": original,
                "summary": summary,
                "type": "event",
                "source": {
                    "kind": "document",
                    "ref": "https://github.com/megagonlabs/HappyDB/blob/b9e529e1e639db96871b6fa3250bf8664ba1ff85/happydb/data/cleaned_hm.csv",
                    "locator": f"hmid={hmid}; reflection_period={row['reflection_period'].strip()}",
                },
                "conditions": None,
                "valid_from": None,
                "valid_until": None,
            },
        }
        requests.append(request)
        annotations.append({
            "hmid": hmid,
            "scope_id": scope_id,
            "original_text": original,
            "source_cleaned_text": cleaned,
            "agent_rewrite": summary,
            "needs_human_review": True,
            "rewrite_note": annotation.get("rewrite_note", ""),
        })

    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = args.output.with_suffix(args.output.suffix + ".tmp")
    temporary.write_text("".join(json.dumps(item, ensure_ascii=False) + "\n" for item in requests), encoding="utf-8")
    temporary.replace(args.output)
    if args.rewrites:
        review_output = args.review_output or args.output.with_suffix(".human-review.jsonl")
        review_output.parent.mkdir(parents=True, exist_ok=True)
        review_tmp = review_output.with_suffix(review_output.suffix + ".tmp")
        review_tmp.write_text(
            "".join(json.dumps(item, ensure_ascii=False) + "\n" for item in annotations),
            encoding="utf-8",
        )
        review_tmp.replace(review_output)
        review_csv = args.output.with_suffix(".human-review.csv")
        fields = ["hmid", "original_text", "source_cleaned_text", "agent_rewrite", "rewrite_note", "needs_human_review",
                  "rewrite_faithful", "meaning_preserved", "invented_detail", "reviewer_notes", "reviewer_id", "reviewed_at_utc"]
        buffer = io.StringIO(newline="")
        writer = csv.DictWriter(buffer, fieldnames=fields, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(annotations)
        review_csv_tmp = review_csv.with_suffix(review_csv.suffix + ".tmp")
        review_csv_tmp.write_text(buffer.getvalue(), encoding="utf-8")
        review_csv_tmp.replace(review_csv)
    print(
        json.dumps(
            {
                "source_rows": sum(map(len, people.values())),
                "selected_memories": len(requests),
                "contributors": len({item["scope"]["id"] for item in requests}),
                "agent_rewrites": len(annotations),
                "limit": MAX_MEMORIES,
                "write_requests_jsonl": str(args.output),
                "human_review_packet": str(args.review_output or args.output.with_suffix(".human-review.jsonl")) if args.rewrites else None,
                "human_review_csv": str(args.output.with_suffix(".human-review.csv")) if args.rewrites else None,
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
