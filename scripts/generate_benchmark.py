#!/usr/bin/env python3
"""Generate deterministic, entirely fictional retrieval cases for iROMEM."""

import argparse
import hashlib
import json
from pathlib import Path


PROMPTS = (
    "¿Qué convención debo seguir para {task} en {project}?",
    "Voy a {task} para {project}. ¿Qué preferencia debo recordar?",
    "Antes de trabajar en {project}, recuérdame cómo {task}.",
    "Necesito la regla de {project} sobre {task}.",
    "¿Cómo se acostumbra {task} en {project}?",
    "Dame el criterio vigente para {task} de {project}.",
    "Estoy preparando {task} de {project}; recupera su instrucción.",
    "¿Cuál era el acuerdo para {task} en {project}?",
    "Busca el recuerdo que gobierna {task} de {project}.",
    "No improvises al {task} en {project}: ¿qué se decidió?",
    "What is the standing preference for {task_en} in {project}?",
    "I am about to {task_en} for {project}; what should I remember?",
    "Retrieve the current rule for {project} regarding {task_en}.",
    "How should I {task_en} for {project}?",
    "Find the relevant memory before I {task_en} in {project}.",
    "¿Qué convención aplica a {alias} cuando toca {task}?",
    "Para {alias}, ¿cómo debo {task}?",
    "Recupera la preferencia de {alias} sobre {task}.",
    "What does {alias} require when I {task_en}?",
    "Check {alias}'s remembered rule for {task_en}.",
    "Si alguien me pide {task} en {project}, ¿qué debo aplicar?",
    "Encuentra la pauta de {project} para la tarea de {task}.",
    "Quiero cumplir la preferencia de {project} al {task}.",
    "¿Qué recuerdo es útil para {task} dentro de {project}?",
    "Recordatorio necesario: {project}, {task}.",
)

TASKS = (
    ("redactar el informe semanal", "write the weekly report", "formato", "usar una tabla de tres columnas"),
    ("enviar avisos de avance", "send progress updates", "canal", "usar el canal Faro"),
    ("presentar métricas", "present metrics", "unidad", "mostrar cifras en miles"),
    ("cerrar incidencias", "close incidents", "cierre", "adjuntar una nota de causa raíz"),
    ("preparar reuniones", "prepare meetings", "agenda", "incluir una agenda de cuatro puntos"),
)

NAMES = ("Alerce", "Bruma", "Cobalto", "Duna", "Estuario", "Faro", "Greda", "Hiedra")
AS_OF = "2026-10-15T12:00:00.000Z"


def split_for(index: int) -> str:
    # Group by entity so no paraphrase of an entity crosses partitions.
    bucket = int.from_bytes(hashlib.sha256(f"iromem-v1-{index}".encode()).digest()[:2], "big") % 10
    return "development" if bucket < 6 else "calibration" if bucket < 8 else "final"


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def write_jsonl(path: Path, values: list[dict]) -> None:
    path.write_text("".join(json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n" for value in values))


def generate(entity_count: int) -> tuple[list[dict], list[dict]]:
    corpus: list[dict] = []
    cases: list[dict] = []
    for index in range(entity_count):
        number = index + 1
        name = NAMES[index % len(NAMES)]
        project = f"Proyecto {name} {number:04d}"
        alias = f"PX-{number:04d}"
        scope = f"laboratorio_{index % 20:02d}"
        task, task_en, attribute, value = TASKS[index % len(TASKS)]
        memory_id = f"synthetic_{number:04d}"
        split = split_for(index)
        original = f"En {project} (alias {alias}), al {task}, la preferencia vigente es {value}."
        summary = f"{project} ({alias}): al {task}, {value}."
        revision = 2 if index % 10 == 0 else 1
        if revision == 2:
            corpus.append(dict(id=memory_id, scope_kind="project", scope_id=scope, revision=1,
                               memory_type="preference",
                               original_text=f"En {project} (alias {alias}), al {task}, la regla anterior era no {value}.",
                               summary=f"{project} ({alias}): no {value} (regla anterior).",
                               state="corrected", valid_from=None, valid_until=None))
        corpus.append(dict(id=memory_id, scope_kind="project", scope_id=scope, revision=revision,
                           memory_type="preference", original_text=original, summary=summary,
                           state="active", valid_from=None, valid_until=None))
        if index % 10 == 0:
            corpus.append(dict(id=f"expired_{number:04d}", scope_kind="project", scope_id=scope,
                               revision=1, memory_type="preference", original_text=f"{project}: no {value}.",
                               summary=f"{project}: no {value} (caducado).", state="expired",
                               valid_from=None, valid_until="2025-01-01T00:00:00.000Z"))
        # Same entity and task in another scope, but a contradictory value. This tests
        # that scope filtering happens before ranking and model classification.
        shadow_scope = f"laboratorio_{(index + 1) % 20:02d}"
        corpus.append(dict(id=f"shadow_{number:04d}", scope_kind="project", scope_id=shadow_scope,
                           revision=1, memory_type="preference",
                           original_text=f"En {project} (alias {alias}), al {task}, la preferencia es no {value}.",
                           summary=f"{project} ({alias}): no {value}.", state="active",
                           valid_from=None, valid_until=None))
        for variant, template in enumerate(PROMPTS):
            query = template.format(project=project, alias=alias, task=task, task_en=task_en)
            cases.append(dict(id=f"q_{number:04d}_{variant:02d}", split=split,
                              segment="alias" if 15 <= variant <= 19 else "english" if 10 <= variant <= 14 else "direct",
                              request=dict(query=query, scope_kind="project", scope_id=scope,
                                           as_of=AS_OF, limit=20), expected_ids=[memory_id],
                              expected_fact=value, forbidden_ids=[f"shadow_{number:04d}", f"expired_{number:04d}"],
                              label_source="deterministic_fixture_rule"))
        cases.append(dict(id=f"negative_{number:04d}", split=split, segment="absent_entity",
                          request=dict(query=f"¿Qué regla aplica a Proyecto Inexistente {number:04d}?",
                                       scope_kind="project", scope_id=scope, as_of=AS_OF, limit=20),
                          expected_ids=[], expected_fact=None, forbidden_ids=[],
                          label_source="deterministic_fixture_rule"))
        cases.append(dict(id=f"wrong_scope_{number:04d}", split=split, segment="wrong_scope",
                          request=dict(query=PROMPTS[0].format(project=project, alias=alias,
                                                                task=task, task_en=task_en),
                                       scope_kind="project", scope_id=f"unassigned_{index % 20:02d}",
                                       as_of=AS_OF, limit=20),
                          expected_ids=[], expected_fact=None,
                          forbidden_ids=[memory_id, f"shadow_{number:04d}"],
                          label_source="deterministic_fixture_rule"))
    return corpus, cases


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--entities", type=int, default=400)
    parser.add_argument("--output", type=Path, default=Path("tests/benchmark/v1"))
    args = parser.parse_args()
    if not 1 <= args.entities <= 10000:
        parser.error("--entities must be between 1 and 10000")
    args.output.mkdir(parents=True, exist_ok=True)
    corpus, cases = generate(args.entities)
    write_json(args.output / "corpus.json", corpus)
    write_jsonl(args.output / "queries.jsonl", cases)
    manifest = dict(version=1, synthetic=True, generator="scripts/generate_benchmark.py",
                    entity_count=args.entities, fixture_revision_count=len(corpus),
                    unique_memories=len({memory["id"] for memory in corpus}), query_count=len(cases),
                    positive_queries=args.entities * len(PROMPTS), negative_queries=args.entities * 2,
                    as_of=AS_OF, partitions={name: sum(case["split"] == name for case in cases)
                                          for name in ("development", "calibration", "final")},
                    files={name: hashlib.sha256((args.output / name).read_bytes()).hexdigest()
                           for name in ("corpus.json", "queries.jsonl")})
    write_json(args.output / "manifest.json", manifest)
    print(json.dumps({key: value for key, value in manifest.items() if key != "files"}))


if __name__ == "__main__":
    main()
