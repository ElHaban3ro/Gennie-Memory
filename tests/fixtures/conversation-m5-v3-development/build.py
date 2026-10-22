"""Build the frozen synthetic status-development fixture from source.json."""

from __future__ import annotations

import hashlib
import json
import random
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE = json.loads((ROOT / "source.json").read_text())


def opaque(prefix: str, value: str) -> str:
    return prefix + hashlib.sha256(value.encode()).hexdigest()[:20]


memories = []
cases = []
for scope in SOURCE["scopes"]:
    assert len(scope["items"]) == 12
    assert sum(item["answerable"] for item in scope["items"]) == 8
    assert len({item["slug"] for item in scope["items"]}) == 12
    number = scope["id"].rsplit("_", 1)[1]
    for item in scope["items"]:
        slug = item["slug"]
        source = {"kind": "user", "ref": f"synthetic://conversation-m5-v3/{number}/{slug}"}
        memory_id = opaque("cm5v3_", f"{scope['id']}:{slug}")
        memories.append(
            {
                "id": memory_id,
                "scope_kind": "conversation",
                "scope_id": scope["id"],
                "entity_id": None,
                "revision": 1,
                "memory_type": "event",
                "original_text": "Ejemplo completamente sintético. " + item["fact"],
                "summary": item["fact"],
                "conditions": None,
                "source": source,
                "state": "active",
                "valid_from": None,
                "valid_until": None,
            }
        )
        cases.append(
            {
                "id": opaque("cm5v3q_", f"{scope['id']}:{item['question']}"),
                "split": "development",
                "segment": f"conversation/{slug}",
                "label_source": "agent_synthetic_unreviewed",
                "request": {
                    "search": {
                        "query": item["question"],
                        "scope_kind": "conversation",
                        "scope_id": scope["id"],
                        "entity_id": None,
                        "as_of": "2026-10-17T12:00:00.000Z",
                        "limit": 20,
                    }
                },
                "expected_ids": [memory_id] if item["answerable"] else [],
                "forbidden_ids": [],
                "expected_fact": item["fact"] if item["answerable"] else None,
                "expected_source": source if item["answerable"] else None,
                "expected_condition": None,
            }
        )

assert len(SOURCE["scopes"]) == 3 and len(memories) == len(cases) == 36
random.Random(2026101703).shuffle(memories)
random.Random(2026101704).shuffle(cases)
(ROOT / "memories.json").write_text(json.dumps(memories, ensure_ascii=False, indent=2) + "\n")
(ROOT / "development.jsonl").write_text(
    "".join(json.dumps(case, ensure_ascii=False, sort_keys=True) + "\n" for case in cases)
)
files = ["source.json", "build.py", "README.md", "label-audit.md", "memories.json", "development.jsonl"]
manifest = {
    "version": SOURCE["version"],
    "scope_count": 3,
    "memories": 36,
    "cases": 36,
    "positive": 24,
    "negative": 12,
    "label_source": "agent_synthetic_unreviewed",
    "human_review": "pending",
    "scored_split": "development",
    "files_sha256": {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest() for name in files},
}
(ROOT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
