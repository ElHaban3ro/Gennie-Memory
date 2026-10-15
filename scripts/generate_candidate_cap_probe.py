#!/usr/bin/env python3
"""Expand frozen v5 cases with distinct synthetic meeting tasks per entity."""

import argparse
import hashlib
import json
from pathlib import Path


TASKS = (
    ("presupuesto", "budget review", "confirmar las cifras aprobadas"),
    ("incidentes", "incident review", "enumerar los incidentes abiertos"),
    ("incorporación", "onboarding", "presentar a los responsables de bienvenida"),
    ("lanzamiento", "launch planning", "revisar las fechas de lanzamiento"),
    ("entrega", "delivery review", "verificar los entregables pendientes"),
    ("demostración", "product demo", "preparar el recorrido de demostración"),
    ("seguridad", "security review", "revisar los riesgos registrados"),
    ("formación", "training", "compartir los materiales de formación"),
    ("contratación", "hiring", "confirmar el perfil de la vacante"),
    ("retrospectiva", "retrospective", "repasar las mejoras acordadas"),
)
MODES = (("remote", "remota", "remotas"), ("onsite", "presencial", "presenciales"))


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def dump_jsonl(rows: list[dict]) -> bytes:
    return "".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                   for row in rows).encode()


def build(corpus: list[dict], cases: dict[str, list[dict]]) -> tuple[list[dict], dict[str, list[dict]], dict]:
    entities_by_split = {
        split: sorted({row["request"]["entity_id"] for row in rows})
        for split, rows in cases.items()
    }
    if set(entities_by_split["development"]) & set(entities_by_split["calibration"]):
        raise ValueError("Development and calibration entities overlap")
    selected_entities = set(entities_by_split["development"] + entities_by_split["calibration"])
    selected_corpus = [row for row in corpus if row["entity_id"] in selected_entities]
    existing = {row["id"] for row in selected_corpus}
    current = {}
    for row in selected_corpus:
        if row["state"] == "active" and row["scope_id"].startswith("laboratorio_"):
            current.setdefault(row["entity_id"], row)
    if set(current) != selected_entities:
        raise ValueError("Some entities lack a current source memory")
    added = []
    task_memory = {}
    for entity in sorted(selected_entities):
        original = current[entity]
        project = original["summary"].split(":", 1)[0]
        for task_index, (task_es, _task_en, action) in enumerate(TASKS):
            for mode, mode_es, mode_plural in MODES:
                memory_id = "m_" + digest(f"candidate-cap-v1:{entity}:{task_index}:{mode}".encode())[:20]
                if memory_id in existing:
                    raise ValueError("Synthetic memory ID collision")
                existing.add(memory_id)
                fact = f"{action} ({mode_es})"
                summary = f"{project}: para una reunión {mode_es} de {task_es}, {fact}."
                source = {"kind": "document",
                          "ref": f"synthetic:meeting-task-guide/{entity}/{task_index + 1}@v1",
                          "locator": mode}
                condition = f"Solo para reuniones {mode_plural} de {task_es}"
                added.append({
                    "id": memory_id, "scope_kind": original["scope_kind"],
                    "scope_id": original["scope_id"], "entity_id": entity,
                    "revision": 1, "memory_type": "procedure",
                    "original_text": summary, "summary": summary,
                    "conditions": condition, "source": source,
                    "state": "active", "valid_from": None, "valid_until": None,
                })
                task_memory[(entity, task_index, mode)] = (memory_id, fact, condition, source)
    revised_cases = {}
    for split, rows in cases.items():
        revised = []
        for row in rows:
            entity = row["request"]["entity_id"]
            if row["segment"] != "natural_gap_onsite":
                revised.append(row)
                continue
            task_index = (int(entity.rsplit("_", 1)[1]) - 1) % len(TASKS)
            task_es, task_en, _action = TASKS[task_index]
            memory_id, fact, condition, source = task_memory[(entity, task_index, "onsite")]
            project = current[entity]["summary"].split(":", 1)[0]
            if task_index % 2:
                query = (f"For the in-person {task_en} meeting in {project}, "
                         "which saved instruction should I follow?")
            else:
                query = (f"En la reunión presencial de {task_es} de {project}, "
                         "¿qué pauta guardada debo aplicar?")
            revised.append({**row, "id": f"dense_{row['id']}",
                            "request": {**row["request"], "query": query},
                            "expected_ids": [memory_id], "expected_fact": fact,
                            "expected_condition": condition, "expected_source": source,
                            "derived_from_case": row["id"]})
        revised_cases[split] = revised
    for split, rows in revised_cases.items():
        if (len(rows) != 200 or len({row["id"] for row in rows}) != 200 or
                sum(bool(row["expected_ids"]) for row in rows) != 100):
            raise ValueError(f"Unexpected {split} case counts")
    return selected_corpus + added, revised_cases, {
        "entities": len(selected_entities), "added_memories": len(added),
        "active_memories_per_entity": 22, "cases_per_split": 200,
        "positive_per_split": 100, "negative_per_split": 100,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--development", type=Path, required=True)
    parser.add_argument("--calibration", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    sources = {"corpus": args.corpus, "development": args.development,
               "calibration": args.calibration}
    cases = {split: [json.loads(line) for line in sources[split].read_text().splitlines()]
             for split in ("development", "calibration")}
    for split, rows in cases.items():
        if len(rows) != 200 or any(row["split"] != split or
                                   row["label_source"] != "deterministic_fixture_rule" for row in rows):
            raise ValueError(f"Unexpected {split} input cases")
    corpus, revised_cases, counts = build(json.loads(args.corpus.read_text()), cases)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    corpus_bytes = (json.dumps(corpus, ensure_ascii=False, indent=2) + "\n").encode()
    (args.output_dir / "corpus.json").write_bytes(corpus_bytes)
    case_hashes = {}
    for split, rows in revised_cases.items():
        payload = dump_jsonl(rows)
        (args.output_dir / f"{split}.jsonl").write_bytes(payload)
        case_hashes[split] = digest(payload)
    manifest = {"version": 1, "kind": "v5_candidate_cap_dense_probe",
                "source_sha256": {name: digest(path.read_bytes()) for name, path in sources.items()},
                "corpus_sha256": digest(corpus_bytes), "case_sha256": case_hashes,
                "candidate_caps": [5, 10, 15, 20], "tasks": [item[0] for item in TASKS],
                "label_provenance": "Synthetic v5 natural-gap labels and deterministic task-specific additions",
                **counts}
    (args.output_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: manifest[key] for key in counts}, ensure_ascii=False))


if __name__ == "__main__":
    main()
