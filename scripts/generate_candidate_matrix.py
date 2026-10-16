#!/usr/bin/env python3
"""Generate nested 100/200/2000-memory synthetic corpora for candidate-cap tests."""

import argparse
import hashlib
import json
from pathlib import Path

from generate_candidate_cap_probe import TASKS


SCOPE = "candidate_matrix_lab"
SIZES = (100, 200, 2000)
MODES = (("remote", "remota", "remotas", "online"),
         ("onsite", "presencial", "presenciales", "in person"))


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def jsonl(rows: list[dict]) -> bytes:
    return "".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                   for row in rows).encode()


def source_entities(paths: dict[str, Path]) -> tuple[list[str], dict[str, str]]:
    partitions = {}
    for split, path in paths.items():
        rows = [json.loads(line) for line in path.read_text().splitlines()]
        entities = sorted({row["request"]["entity_id"] for row in rows})
        if len(entities) != 50 or len(rows) != 200 or any(row["split"] != split for row in rows):
            raise ValueError(f"Unexpected source partition: {split}")
        partitions[split] = entities
    if set(partitions["development"]) & set(partitions["calibration"]):
        raise ValueError("Source partitions overlap")
    interleaved = [entity for pair in zip(partitions["development"], partitions["calibration"])
                   for entity in pair]
    split_by_entity = {entity: split for split, entities in partitions.items() for entity in entities}
    return interleaved, split_by_entity


def make_memories(source: list[dict], entities: list[str]) -> tuple[list[dict], dict]:
    selected = set(entities)
    originals = [row for row in source if row["entity_id"] in selected and
                 row["state"] == "active" and row["scope_id"].startswith("laboratorio_")]
    by_entity = {}
    for row in originals:
        by_entity.setdefault(row["entity_id"], []).append(row)
    if set(by_entity) != selected or any(len(rows) != 2 for rows in by_entity.values()):
        raise ValueError("Expected exactly two original active memories per entity")
    memories = []
    lookup = {}
    for entity in entities:
        originals_for_entity = by_entity[entity]
        project = originals_for_entity[0]["summary"].split(":", 1)[0]
        for row in originals_for_entity:
            mode = row["source"]["locator"]
            current = {**row, "scope_id": SCOPE}
            memories.append(current)
            fact = row["summary"].split(", ", 1)[-1].rstrip(".")
            lookup[(entity, 0, mode)] = (current, fact)
        for task_index, (task_es, _task_en, action) in enumerate(TASKS[:9], start=1):
            for mode, mode_es, mode_plural, _mode_en in MODES:
                identity = "m_" + digest(f"matrix-v1:{entity}:{task_index}:{mode}".encode())[:20]
                fact = f"{action} para {task_es} ({mode_es})"
                summary = f"{project}: para una reunión {mode_es} de {task_es}, {fact}."
                row = {
                    "id": identity, "scope_kind": "project", "scope_id": SCOPE,
                    "entity_id": entity, "revision": 1, "memory_type": "procedure",
                    "original_text": summary, "summary": summary,
                    "conditions": f"Solo para reuniones {mode_plural} de {task_es}",
                    "source": {"kind": "document",
                               "ref": f"synthetic:meeting-task-guide/{entity}/{task_index}@v1",
                               "locator": mode},
                    "state": "active", "valid_from": None, "valid_until": None,
                }
                memories.append(row)
                lookup[(entity, task_index, mode)] = (row, fact)
    if len(memories) != len(entities) * 20 or len({row["id"] for row in memories}) != len(memories):
        raise ValueError("Memory count or identity invariant failed")
    return memories, lookup


def make_cases(entities: list[str], splits: dict[str, str], memories: dict) -> dict[str, list[dict]]:
    variants = {"scoped": [], "unscoped": []}
    for entity in entities:
        project = memories[(entity, 0, "remote")][0]["summary"].split(":", 1)[0]
        for task_index in range(10):
            for mode, mode_es, _mode_plural, mode_en in MODES:
                memory, fact = memories[(entity, task_index, mode)]
                if task_index == 0:
                    query = (f"In {project}, which saved setup rule applies to an upcoming "
                             f"{mode_en} team meeting?")
                else:
                    task_es, task_en, _action = TASKS[task_index - 1]
                    query = (f"En {project}, ¿qué pauta guardada aplica a la reunión {mode_es} "
                             f"de {task_es}?" if task_index % 2 == 0 else
                             f"For the {mode_en} {task_en} meeting in {project}, "
                             "which saved instruction should I follow?")
                for variant in variants:
                    request = {"query": query, "scope_kind": "project", "scope_id": SCOPE,
                               "as_of": "2026-10-15T00:00:00.000Z", "limit": 20}
                    if variant == "scoped":
                        request["entity_id"] = entity
                    variants[variant].append({
                        "id": f"matrix_{entity}_{task_index}_{mode}_{variant}",
                        "split": splits[entity], "segment": f"task_{task_index}_{mode}",
                        "request": request, "expected_ids": [memory["id"]],
                        "expected_fact": fact, "expected_condition": memory["conditions"],
                        "expected_source": memory["source"], "forbidden_ids": [],
                        "label_source": "deterministic_fixture_rule",
                    })
        negatives = (
            ("mode_unknown", f"In {project}, the meeting format is undecided. Which setup rule applies?"),
            ("wrong_task", f"In {project}, what is the saved rule for closing an expense claim?"),
            ("explicit_no_recall", f"For {project}, do not use saved memories in this answer."),
            ("absent_project", f"What saved setup rule applies to Unlisted Project {entity}?"),
        )
        for segment, query in negatives:
            for variant in variants:
                request = {"query": query, "scope_kind": "project", "scope_id": SCOPE,
                           "as_of": "2026-10-15T00:00:00.000Z", "limit": 20}
                if variant == "scoped":
                    request["entity_id"] = entity if segment != "absent_project" else f"absent_{entity}"
                variants[variant].append({
                    "id": f"matrix_{entity}_{segment}_{variant}",
                    "split": splits[entity], "segment": segment,
                    "request": request, "expected_ids": [], "expected_fact": None,
                    "expected_condition": None, "expected_source": None,
                    "forbidden_ids": [], "label_source": "deterministic_fixture_rule",
                })
    return variants


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--development", type=Path, required=True)
    parser.add_argument("--calibration", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    paths = {"development": args.development, "calibration": args.calibration}
    entities, splits = source_entities(paths)
    source = json.loads(args.corpus.read_text())
    source_hashes = {"corpus": digest(args.corpus.read_bytes()),
                     **{name: digest(path.read_bytes()) for name, path in paths.items()}}
    for size in SIZES:
        chosen = entities[:size // 20]
        corpus, lookup = make_memories(source, chosen)
        cases = make_cases(chosen, splits, lookup)
        destination = args.output_dir / str(size)
        destination.mkdir(parents=True, exist_ok=True)
        corpus_bytes = (json.dumps(corpus, ensure_ascii=False, indent=2) + "\n").encode()
        (destination / "corpus.json").write_bytes(corpus_bytes)
        case_hashes = {}
        for variant, rows in cases.items():
            payload = jsonl(rows)
            (destination / f"{variant}.jsonl").write_bytes(payload)
            case_hashes[variant] = digest(payload)
        manifest = {"version": 1, "kind": "candidate_matrix_v1", "active_memories": size,
                    "entities": len(chosen), "eligible_per_scoped_query": 20,
                    "caps": [5, 10, 15, 20], "variants": list(cases),
                    "cases_per_variant": len(cases["scoped"]),
                    "positive_per_variant": len(chosen) * 20,
                    "source_sha256": source_hashes, "corpus_sha256": digest(corpus_bytes),
                    "case_sha256": case_hashes,
                    "label_provenance": "Deterministic synthetic task and modality labels; no human review"}
        (destination / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({"size": size, "entities": len(chosen), "cases": len(cases["scoped"])},
                         ensure_ascii=False))


if __name__ == "__main__":
    main()
