# Línea base con rúbrica de evidencia: desarrollo v1

El 2026-10-17 se fijó una [rúbrica de cuatro relaciones](evidence-rubric-v1.md), se creó un banco sintético nuevo y se ejecutó la configuración vigente una sola vez por pregunta, según el [protocolo previo](conversation-evidence-v1-baseline-protocol.md). **La búsqueda cubrió toda la evidencia anotada; la selección omitió ocho refutaciones y cuatro contextos parciales.** No se cambia la configuración ni se presenta esta campaña como mejora frente a cohortes anteriores.

## Banco y procedimiento

Ocho conversaciones nuevas (`034`–`041`), 64 recuerdos y 64 preguntas: 16 por relación. Las frases fuente, citas, justificaciones y límites de lo afirmable se fijaron antes de la campaña remota. Todo es desarrollo sintético conocido, redactado y revisado por el mismo agente; no hay revisión independiente ni humana. El [registro de revisión](../../tests/fixtures/conversation-evidence-v1/agent-review.md) conserva una ambigüedad corregida antes de ejecutar, sin usar resultados del proveedor.

`validate-evidence-benchmark` comprobó integridad de archivos y rúbrica, fuente literal, ámbitos, IDs, procedencia y equilibrio. Se cargaron 64 recuerdos en una SQLite temporal, se ejecutó `search-benchmark` y se verificaron los candidatos mediante `report-evidence-retrieval`. Después se ejecutaron 64 consultas Decisions `baseline`, `keywords`, límite 20, sin cambiar instrucciones, etiquetas, orden o modelo entre casos. El diario registra cada inicio y final; no hubo reintentos.

## Métricas observadas

| Relación | Casos | Evidencia cubierta por búsqueda | Selección deseada | Omisiones de evidencia cubierta |
| --- | ---: | ---: | ---: | ---: |
| Respuesta completa | 16 | 16/16 | 16/16 IDs permitidos | 0 |
| Refutación de premisa | 16 | 16/16 | 8/16 IDs permitidos | 8 |
| Contexto parcial | 16 | 16/16 | 12/16 IDs permitidos | 4 |
| Ausencia de evidencia | 16 | No aplica | 16/16 semillas vacías | No aplica |

No hubo selección de un ID ajeno a los permitidos ni degradaciones. Las doce discordancias fueron abstenciones con evidencia presente; no inventaron una respuesta. El top 1 del buscador contenía el ID permitido en 16/16 respuestas, 15/16 refutaciones y 15/16 contextos parciales. La refutación omitida que no estaba primera sí estaba entre las cinco primeras; el contexto parcial que no estaba primero apareció en el puesto siete y sí se seleccionó.

Todas las listas tenían ocho candidatos, incluidas las preguntas sin evidencia: el 16/16 de abstenciones no procede de omitir al proveedor por una búsqueda vacía. Sin embargo, este banco no prueba selección con listas grandes, truncamiento, compresión, conversación libre ni conocimiento real. No se calcula una puntuación global que permita compensar omisiones correctivas con abstenciones. Los porcentajes por categoría son descriptivos de estos 16 casos, no estimaciones independientes de población.

| Recurso medido | Resultado |
| --- | ---: |
| Llamadas y recibos de coste | 64/64 |
| Modelo servido | `openai/gpt-6-luna-decisions-20261006` |
| Tokens de entrada reportados | 21.918 |
| Tokens de salida reportados | 0, según proveedor de elección tipada |
| Coste reportado total, USD | 0,0021918 |
| Tiempo de servicio, p50 / p95 / p99 | 356,74 / 838,88 / 1.382,46 ms |
| Tiempo del runner, p50 / p95 / p99 | 357,96 / 840,15 / 1.383,35 ms |

Los tiempos son los de esta ejecución, sin comparación temporal emparejada ni prueba de latencia de integración. No se equiparan los tokens reportados del proveedor con tokens de semilla ni el coste reportado con una auditoría de factura.

## Discordancias conservadas

| Caso | Relación | Información que no llegó a la semilla |
| --- | --- | --- |
| `cev1q_035_03` | Refutación | Se rechazó publicar en abierto las entrevistas. |
| `cev1q_036_04` | Refutación | La visita pasó del martes al jueves. |
| `cev1q_037_03` | Refutación | Edición digital; impresión descartada. |
| `cev1q_037_04` | Refutación | Se retiró el capítulo de rapaces. |
| `cev1q_038_04` | Refutación | La clase presencial se sustituyó por una sesión virtual. |
| `cev1q_039_03` | Refutación | Se retiró el sensor de viento averiado. |
| `cev1q_040_03` | Refutación | La exposición no tendrá concurso ni premios. |
| `cev1q_041_03` | Refutación | El tramo del mirador está cerrado. |
| `cev1q_037_05` | Contexto parcial | Hay cuarenta mapas; escala desconocida. |
| `cev1q_037_06` | Contexto parcial | Ubicación del glosario conocida; aprobación desconocida. |
| `cev1q_040_05` | Contexto parcial | Hay treinta piezas; pesos desconocidos. |
| `cev1q_040_06` | Contexto parcial | Montaje empieza el lunes; fin desconocido. |

El [diario original](../../outputs/iromem/evaluation/conversation-evidence-v1-2026-10-17/decisions-journal.jsonl) conserva consultas, candidatos y semillas. Las etiquetas no se corrigieron después de observar estas abstenciones.

## Interpretación y siguiente experimento

La [instrucción vigente](../../bootstrap/src/openrouter.rs) pide elegir un hecho que responda a la pregunta específica y abstenerse cuando ninguno proporcione la respuesta solicitada. La nueva rúbrica pide conservar además correcciones y partes útiles. **Hay una diferencia de objetivo entre esa instrucción y la rúbrica**, además de variabilidad observada: que una abstención cumpla una lectura estricta de respuesta completa no significa que sirva para construir una memoria útil. Esta inspección estática sugiere una hipótesis; no demuestra causalmente que cambiar la instrucción resolverá las doce omisiones.

El siguiente experimento razonable sería una variante explícita de selección por utilidad que acepte respuesta, refutación o contexto parcial y preserve sus límites, usando esta línea base como desarrollo. Debe seguir exigiendo ausencia cuando solo coincida el tema, contrastarse por pares y conservar los 16 casos de ausencia y los 16 de respuesta. No se implementó ni promovió esa variante en esta entrega. Cualquier ajuste usando estos casos necesitará otra cohorte y revisión independiente antes de aprobación. `evidence_v1` y `support_gate_v1` históricos no se rehabilitan: sus instrucciones endurecen la exigencia de respuesta completa.

## Verificación y límites

La auditoría Rust validó 512 apariciones de candidatos y 36 recuerdos de semillas, incluidos texto, condiciones, fuente declarada, ámbito, revisión, estado y vigencia. Las 28 semillas vacías no contienen recuerdos que auditar. La validación mecánica no acredita la categoría semántica por sí sola, no verifica fuentes externas ni mide afirmaciones de una respuesta final generada. La revisión independiente continúa pendiente. Las 21 pruebas del evaluador pasaron; se verificaron formato e integridad de archivos. El producto, el límite 20 y las métricas del README permanecen vigentes sin cambios por este desarrollo.

Evidencia: [banco y hashes](../../tests/fixtures/conversation-evidence-v1/manifest.json), [preflight anterior al proveedor](../../outputs/iromem/evaluation/conversation-evidence-v1-2026-10-17/preflight.json), [cobertura por relación y rango por caso](../../outputs/iromem/evaluation/conversation-evidence-v1-2026-10-17/retrieval-report.json), [auditoría y desglose por conversación](../../outputs/iromem/evaluation/conversation-evidence-v1-2026-10-17/audit.json), [comprobación independiente mecánica](../../outputs/iromem/evaluation/conversation-evidence-v1-2026-10-17/verification.json) y [manifiesto de resultados](../../outputs/iromem/evaluation/conversation-evidence-v1-2026-10-17/manifest.json). «Independiente» en esa comprobación significa otro cálculo sobre los diarios; no revisión semántica por otra persona o agente.
