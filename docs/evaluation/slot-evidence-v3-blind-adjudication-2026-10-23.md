# Adjudicación ciega de atribución Slot Evidence V3

Fecha: 2026-10-23. Estado: banco sintético nuevo adjudicado de forma ciega; comparación del clasificador aún no ejecutada.

## Banco y procedimiento

`tests/fixtures/slot-evidence-v3-attribution-blind-v1.json` contiene 14 pares de pregunta y resumen, todos sobre la ciudad del contratista Bob Torres (B-29). SHA-256: `ede12b665adfcfe182f7517d9fa7c74a2bafc4c955ebd245fcf774e72d1b7103`. Incluye afirmaciones directas, subordinadas cognitivas con y sin complementizador, verbos de reporte, relativas, appositivas, premodificadores, una coordinación plural y controles donde Bob es destinatario u objeto. El paquete ciego contiene solo IDs, preguntas y resúmenes, sin trazas, política, predicciones ni etiquetas.

Una revisión independiente recibió únicamente `id`, `query` y `summary`. Para cada fila indicó si respondía la ciudad de Bob y clasificó la relación. No encontró casos `partial`, `refutes` o `ambiguous`; 7 casos fueron `answers` y 7 `unrelated`. Los dictámenes se conservaron por ID:

| Caso | Relación ciega | Ciudad de Bob seleccionable | Fundamento breve |
| --- | --- | --- | --- |
| 01 | `answers` | Sí | Quito se predica directamente de Bob. |
| 02 | `unrelated` | No | Bob piensa el dato; Lima se predica de Carlos. |
| 03 | `unrelated` | No | Bob entiende la proposición sobre Carlos. |
| 04 | `unrelated` | No | Bob realiza algo; la residencia pasada es de Carlos. |
| 05 | `unrelated` | No | Bob reporta la residencia de Carlos. |
| 06 | `unrelated` | No | La appositiva identifica a Carlos como sujeto de Lima. |
| 07 | `unrelated` | No | El cargo appositivo no cambia el sujeto Carlos. |
| 08 | `answers` | Sí | Quito es la cláusula matriz sobre Bob; la relativa expresa una creencia sobre Carlos. |
| 09 | `answers` | Sí | Bob es sujeto compartido de “told … a joke and lives”. |
| 10 | `answers` | Sí | Carlos es objeto de “considered”; Bob vive en Quito. |
| 11 | `unrelated` | No | “Who live in Lima” aplica a Carlos y Dana, no a Bob. |
| 12 | `answers` | Sí | Bob y Carlos forman el sujeto plural que vive en Quito. |
| 13 | `answers` | Sí | Quito se predica de Bob; Lima aparece en una relativa de Ana. |
| 14 | `answers` | Sí | Quito se predica de Bob en una cláusula explícita. |

La prueba `v3_new_blind_bank_matches_independent_attribution_adjudication` construye trazas V3 locales para estas 14 filas y comprueba validación, democión y selección frente a la adjudicación: 7/7 respuestas útiles conservadas y 0/7 selecciones ajenas. Las trazas son de prueba escritas localmente; esto mide el selector estructural con evidencia proporcionada, no la capacidad del clasificador para producir citas fieles. La suite workspace completa pasó con 1.023 pruebas correctas, 25 ignoradas y cero fallidas; `cargo fmt --all --check` y `git diff --check` también pasaron. La suite de retrieval incluye ahora 93 aprobadas y 1 ignorada.

## Revisión adversarial y límite

La revisión crítica posterior cubrió los contraejemplos identificados durante la iteración: complemento cognitivo sin “that”, sujeto ajeno seguido de appositiva entre comas o paréntesis, adverbios preverbales como “currently”, falso rechazo al encontrar una relativa ajena anterior a la cláusula matriz, objeto coordinado frente a sujeto plural compartido y mención de un destinatario antes de una afirmación posterior de Bob. Las regresiones se conservan en retrieval. El revisor no encontró un falso positivo concreto en la lista de premodificadores permitidos; sí señaló que una forma no listada puede causar abstención o pérdida de una propuesta ajena no demovida. No es un analizador sintáctico ni prueba semántica general, y sus listas lingüísticas tienen cobertura finita.

Resultado: evidencia estructural alentadora en una cohorte dirigida de 14 casos, no una tasa poblacional ni una mejora del clasificador. No se llamó a OpenRouter; coste API USD 0. No hubo coste Codex cuantificable. V3 permanece experimental, el clasificador no se ha puntuado y el baseline no cambia. Antes de cualquier smoke hay que conservar este banco como holdout, congelar el payload/schema/prompt/binario/configuración y ejecutar el gate de estabilidad del protocolo; cada salida de modelo requiere adjudicación ciega independiente.
