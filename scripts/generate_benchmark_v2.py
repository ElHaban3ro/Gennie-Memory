#!/usr/bin/env python3
"""Generate a larger synthetic benchmark with 1,000 auditable prompt forms."""

import argparse
import hashlib
import json
from pathlib import Path

from generate_benchmark import AS_OF, NAMES, TASKS, generate, write_json, write_jsonl


FRAMES = (
    ("direct", "¿Qué pauta debo aplicar al {task} de {project}?"),
    ("direct", "Estoy por {task} en {project}; dime la preferencia vigente."),
    ("direct", "Para {project}, necesito la instrucción sobre {task}."),
    ("direct", "Antes de {task} para {project}, ¿qué habíamos acordado?"),
    ("direct", "Busca el recuerdo que cambia cómo debo {task} en {project}."),
    ("indirect", "Mi siguiente entrega para {project} implica {task}. ¿Qué debo tener presente?"),
    ("indirect", "Voy a empezar {task}; el trabajo pertenece a {project}. ¿Qué detalle importa?"),
    ("indirect", "Si mañana tengo que {task} para {project}, ¿qué norma previa me afecta?"),
    ("alias", "PX identificado como {alias}: ¿cómo debo {task}?"),
    ("alias", "En {alias} me toca {task}. Recupera su criterio."),
    ("alias", "¿Hay una preferencia de {alias} que deba aplicar al {task}?"),
    ("alias", "Usaré el código {alias} para {task}; dime la regla recordada."),
    ("english", "What remembered instruction applies when I {task_en} for {project}?"),
    ("english", "I am about to {task_en} in {project}. Which prior preference matters?"),
    ("english", "Find the current memory for {project} before I {task_en}."),
    ("english", "For {alias}, how did we agree to {task_en}?"),
    ("code_switch", "Estoy trabajando en {project}; which rule applies when I {task_en}?"),
    ("code_switch", "For {alias}, ¿qué debo recordar antes de {task}?"),
    ("long_context", "Tengo varias tareas abiertas, pero ahora debo {task} para {project}. ¿Qué preferencia concreta rige esta entrega?"),
    ("long_context", "Ya resolví la planificación y las reuniones. En este momento solo necesito {task} de {project}. ¿Qué recuerdo previo aplica?"),
)

PREFIXES = (
    "",
    "Revisa el contexto guardado. ",
    "Necesito actuar sin repetir una decisión antigua. ",
    "Antes de responder, comprueba el acuerdo del proyecto. ",
    "La respuesta depende de una pauta previa. ",
    "Estoy retomando este trabajo después de una pausa. ",
    "Evita adivinar la convención. ",
    "El encargo llegó sin todas las instrucciones. ",
    "Quiero respetar las preferencias ya registradas. ",
    "En una sesión nueva falta contexto. ",
)

SUFFIXES = (
    "",
    " Responde solo con la pauta aplicable.",
    " Si no hay una pauta pertinente, dilo.",
    " Usa la versión vigente del recuerdo.",
    " Mantén el ámbito de este proyecto.",
)

VALUE_BANKS = (
    ("usar una tabla de tres columnas", "abrir con una síntesis de dos frases",
     "cerrar con una lista de riesgos", "separar logros y bloqueos en secciones",
     "usar viñetas con responsables", "poner las fechas al inicio de cada sección"),
    ("usar el canal Faro", "usar el canal Bruma", "enviar al buzón ficticio Delta",
     "publicar en el tablero Alerce", "usar el hilo semanal del equipo",
     "enviar un resumen al grupo ficticio Nube"),
    ("mostrar cifras en miles", "redondear a dos decimales",
     "usar porcentajes con un decimal", "mostrar valores absolutos antes de tasas",
     "separar ingresos y costes", "incluir la unidad junto a cada cifra"),
    ("adjuntar una nota de causa raíz", "añadir una cronología breve",
     "pedir aprobación del responsable", "registrar la prueba de corrección",
     "anotar el impacto para usuarios", "vincular el ticket de seguimiento"),
    ("incluir una agenda de cuatro puntos", "abrir con decisiones pendientes",
     "reservar diez minutos para riesgos", "enviar lecturas previas al inicio",
     "cerrar con responsables y fechas", "empezar con un resumen del estado"),
)

LONG_CONTEXT = (
    "En el tablero hay pendientes de calendario, revisión de diseño, coordinación del equipo, "
    "orden de entrega y mensajes que llegaron por distintos canales. Algunas notas antiguas "
    "hablan de formatos, otras de métricas, y varias se refieren a proyectos distintos. "
    "No mezcles esas notas: el siguiente pedido corresponde solamente al proyecto y la tarea "
    "que se nombran después de este contexto. Una respuesta genérica podría parecer razonable, "
    "pero necesito aplicar la preferencia concreta que quedó guardada para ese trabajo. "
)


def value_for(index: int) -> str:
    bank = VALUE_BANKS[index % len(TASKS)]
    digest = hashlib.sha256(f"iromem-v2-value-{index}".encode()).digest()
    return bank[int.from_bytes(digest[:4], "big") % len(bank)]


def forms() -> list[dict]:
    return [dict(id=index, segment=segment,
                 template=(LONG_CONTEXT if segment == "long_context" else "") + prefix + frame + suffix)
            for index, (segment, frame, prefix, suffix) in enumerate(
                (segment, frame, prefix, suffix)
                for segment, frame in FRAMES
                for prefix in PREFIXES
                for suffix in SUFFIXES)]


def generate_v2(entity_count: int, variants_per_entity: int) -> tuple[list[dict], list[dict], list[dict]]:
    corpus, old_cases = generate(entity_count)
    for row in corpus:
        index = int(row["id"].split("_")[-1]) - 1
        old_value = TASKS[index % len(TASKS)][3]
        new_value = value_for(index)
        row["original_text"] = row["original_text"].replace(old_value, new_value)
        row["summary"] = row["summary"].replace(old_value, new_value)
    split_by_entity = {int(case["id"].split("_")[1]): case["split"]
                       for case in old_cases if case["id"].startswith("q_")}
    prompt_forms = forms()
    assert len(prompt_forms) == 1000 and len({form["template"] for form in prompt_forms}) == 1000
    cases = []
    for index in range(entity_count):
        number = index + 1
        project = f"Proyecto {NAMES[index % len(NAMES)]} {number:04d}"
        alias = f"PX-{number:04d}"
        task, task_en, _, _ = TASKS[index % len(TASKS)]
        value = value_for(index)
        other_task = TASKS[(index + 1) % len(TASKS)][0]
        scope = f"laboratorio_{index % 20:02d}"
        forbidden = [f"shadow_{number:04d}"]
        if index % 10 == 0:
            forbidden.append(f"expired_{number:04d}")
        common = dict(split=split_by_entity[number], label_source="deterministic_fixture_rule")
        request = dict(scope_kind="project", scope_id=scope, as_of=AS_OF, limit=20)
        for variant in range(variants_per_entity):
            # Every entity sees all 20 intent frames before a frame repeats.
            # Across ten entities, each frame receives all 50 prefix/suffix forms.
            form_id = (variant % len(FRAMES)) * 50 + ((index * 5 + variant // len(FRAMES)) % 50)
            form = prompt_forms[form_id]
            query = form["template"].format(project=project, alias=alias,
                                              task=task, task_en=task_en)
            cases.append(dict(id=f"q2_{number:04d}_{form['id']:04d}", segment=form["segment"],
                              request=dict(request, query=query), expected_ids=[f"synthetic_{number:04d}"],
                              expected_fact=value, forbidden_ids=forbidden, **common))
        negatives = (
            ("absent_entity", f"¿Qué regla aplica a Proyecto Inexistente {number:04d}?", scope, []),
            ("wrong_scope", f"¿Qué pauta aplica al {task} de {project}?",
             f"unassigned_{index % 20:02d}", [f"synthetic_{number:04d}", f"shadow_{number:04d}"]),
            ("wrong_task", f"Para {project}, ¿qué pauta rige al {other_task}?", scope, []),
            ("explicit_no_recall", f"No consultes recuerdos de {project}; escribe solo una frase genérica de ejemplo.", scope, []),
        )
        for segment, query, query_scope, forbidden in negatives:
            cases.append(dict(id=f"{segment}_{number:04d}", segment=segment,
                              request=dict(request, query=query, scope_id=query_scope),
                              expected_ids=[], expected_fact=None,
                              forbidden_ids=forbidden, **common))
    return corpus, cases, prompt_forms


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--entities", type=int, default=400)
    parser.add_argument("--variants-per-entity", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("tests/benchmark/v2"))
    args = parser.parse_args()
    if not 10 <= args.entities <= 10000 or not 10 <= args.variants_per_entity <= 1000:
        parser.error("entities must be 10..10000 and variants-per-entity 10..1000")
    args.output.mkdir(parents=True, exist_ok=True)
    corpus, cases, prompt_forms = generate_v2(args.entities, args.variants_per_entity)
    write_json(args.output / "corpus.json", corpus)
    write_json(args.output / "forms.json", prompt_forms)
    write_jsonl(args.output / "queries.jsonl", cases)
    manifest = dict(version=2, synthetic=True, generator="scripts/generate_benchmark_v2.py",
                    entity_count=args.entities, prompt_form_count=len(prompt_forms),
                    used_prompt_forms=len({int(case["id"].split("_")[-1]) for case in cases
                                           if case["id"].startswith("q2_")}),
                    positive_variants_per_entity=args.variants_per_entity,
                    fixture_revision_count=len(corpus),
                    unique_memories=len({memory["id"] for memory in corpus}), query_count=len(cases),
                    positive_queries=args.entities * args.variants_per_entity,
                    negative_queries=args.entities * 4,
                    distinct_values_by_task={str(task_index): len({value_for(index)
                        for index in range(task_index, args.entities, len(TASKS))})
                        for task_index in range(len(TASKS))},
                    as_of=AS_OF, partitions={name: sum(case["split"] == name for case in cases)
                                          for name in ("development", "calibration", "final")},
                    segments={name: sum(case["segment"] == name for case in cases)
                              for name in sorted({case["segment"] for case in cases})},
                    files={name: hashlib.sha256((args.output / name).read_bytes()).hexdigest()
                           for name in ("corpus.json", "forms.json", "queries.jsonl")})
    write_json(args.output / "manifest.json", manifest)
    print(json.dumps({key: value for key, value in manifest.items() if key != "files"}))


if __name__ == "__main__":
    main()
