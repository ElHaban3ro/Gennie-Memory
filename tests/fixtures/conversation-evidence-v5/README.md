# Evidencia conversacional sintética v5

Banco nuevo de desarrollo: ocho conversaciones 066–073, 224 recuerdos (28 por ámbito) y 64 preguntas (ocho por ámbito), con corte `2026-10-19T12:00:00.000Z`. Inicialmente hay 16 preguntas por relación: `supports_answer`, `refutes_premise`, `pending_or_partial` y `unrelated`. Las cuotas son descriptivas y ceden ante revisión semántica previa a cualquier llamada.

`source.json` conserva conversaciones explícitamente sintéticas; `memories.json` contiene citas literales idénticas en `summary` y `original_text`. La fuente declarada es `user` con referencia `synthetic://conversation-evidence-v5/<ámbito>/<frase>`, sin representar hechos ni personas reales. No se verificaron fuentes externas.

`development.jsonl` anota todos los recuerdos individualmente útiles detectados por el autor, con relación por ID, justificación y límite de respuesta. Un ID parcial permitido no equivale a respuesta completa; varias citas pueden completar conjuntamente algunos datos. Cuando hay varios IDs, `expected_fact` y `expected_source` son nulos; con uno reproducen el recuerdo. `expected_ids` vacío exige abstención por falta de evidencia útil en todo el ámbito, aunque existan distractores plausibles.

`blind-questions.json` contiene exclusivamente ID, ámbito y consulta para revisión separada sin etiquetas del autor. `agent-review.md` documenta la autoría y posterior revisión ciega separada. `human_review` e `independent_review` siguen en `false`. La autoría y la revisión de este paquete no constituyen revisión humana ni validación independiente. La revisión ciega y conciliación previas están cerradas y ligadas por manifiesto. Se conservaron tres correcciones de conjuntos permitidos en adjudication.json antes de llamadas; las etiquetas originales están en outputs/iromem/evaluation/roles-v5-preparation-2026-10-19/.

Situaciones: censo de aves, panadería, archivo teatral, orientación, pigmentos, préstamo de herramientas, conservación de mapas y riego. Se incluyen referentes compartidos en una frase, correcciones explícitas, cambio de modalidad, permisos condicionados, resultados incompletos y distractores de otro periodo, objeto o atributo. La redacción no se ajustó a rangos observados de FTS ni a selecciones de una política.

Este banco no mide compresión, verdad de una respuesta final, integración automática, datos reales ni generalización poblacional. No se leyeron políticas ni resultados de selección para redactarlo. No se hicieron llamadas remotas.
