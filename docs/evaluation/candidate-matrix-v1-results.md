# Matriz híbrida de candidatos y clasificación v1

Estado: evaluación sintética final completada el 2026-10-15 en esta Mac. No hay hook ni integración automática con Codex. [Plan y método](candidate-matrix-v1-plan.md).

## Conclusión

Con la entidad del proyecto indicada en la consulta, búsqueda híbrida FTS5 + coseno exacto + RRF, máximo de veinte candidatos y una pregunta de elección simple, **Jev 1.13 produjo la semilla correcta en 389/400 positivos (97,25 %)** y semilla vacía en **80/80 negativos** de la partición final. OpenAI GPT-6 Luna Decisions logró **387/400 (96,75 %)** y **64/80**. Ambos superan el objetivo observado del 95 % en positivos; Jev tuvo mejor resultado negativo y menor coste informado en esta prueba. Las diferencias positivas de dos casos no establecen superioridad general entre modelos.

El resultado depende de disponer de `entity_id`: el banco final sin entidad solo incluyó el recuerdo correcto entre veinte candidatos en **320/400 positivos (80 %)**. Ningún clasificador puede llegar al 95 % bajo ese techo. Gennie Memory todavía no resuelve por sí mismo la entidad a partir del texto, y el coseno exacto sobre todo el ámbito elevó la latencia de búsqueda.

Laya local 0.3.5, usado como clasificador `choice` con la misma formulación simple y los mismos candidatos finales, acertó **0/400 positivos** y **79/80 negativos**: eligió `none` en 479/480 casos. Bajo esta configuración no sirve como selector de recuerdos, aunque su encoder experimental sí permitió construir el índice semántico y sus decisiones fueron rápidas. No se infiere que cualquier otro ajuste o versión de Laya vaya a comportarse igual.

## Diseño de la prueba

- Tres corpus anidados de 100, 200 y 2.000 recuerdos activos; veinte recuerdos por proyecto, diez tareas de reunión y dos modalidades. Para cada tamaño se cruzaron máximos de 5, 10, 15 y 20 candidatos con entidad conocida o ausente: doce combinaciones de tamaño/tope por ámbito.
- El índice textual usa FTS5; la rama semántica usa vectores del encoder multilingüe del snapshot Laya compartido por SOC, coseno exacto y fusión RRF. El perfil definitivo de embeddings sigue pendiente. La entidad, cuando existe, la aporta el caso sintético antes de buscar.
- En desarrollo y calibración se compararon tres instrucciones (`plain`, `conditional`, `structured`) para Jev y OpenAI sobre las mismas listas. La pantalla local probó las tres con Laya. La política `plain` + veinte candidatos + entidad conocida se [congeló tras desarrollo](../../outputs/iromem/evaluation/candidate-matrix-v1/development-policy.json), antes de leer calibración o evaluar las entidades finales.
- La prueba final usó veinte proyectos no presentes en la pantalla: 400 positivos y 80 negativos por ámbito. Los dos clasificadores remotos y Laya recibieron exactamente los mismos candidatos con entidad conocida. El escenario sin entidad se evaluó hasta la etapa de recuperación porque su techo del 80 % ya impide cumplir la meta final.
- Las etiquetas proceden de reglas deterministas de datos sintéticos, sin revisión humana. Las veinte tareas por proyecto comparten estructura lingüística; los 400 positivos no equivalen a 400 intenciones humanas independientes.

## Cobertura del buscador

En la pantalla de recuperación, cada cifra es el número de positivos cuyo recuerdo esperado apareció dentro del tope, sobre 100 preguntas por escenario:

| Recuerdos | Entidad | Top 5 | Top 10 | Top 15 | Top 20 | Búsqueda p50 |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 100 | Conocida | 50 | 74 | 87 | 100 | 4,74 ms |
| 100 | Ausente | 39 | 71 | 78 | 80 | 20,14 ms |
| 200 | Conocida | 54 | 80 | 88 | 100 | 5,03 ms |
| 200 | Ausente | 39 | 69 | 76 | 81 | 39,84 ms |
| 2.000 | Conocida | 50 | 78 | 88 | 100 | 8,85 ms |
| 2.000 | Ausente | 46 | 54 | 70 | 79 | 393,15 ms |

En las 400 preguntas positivas finales de 2.000 recuerdos, la cobertura fue **190/295/342/400** con entidad y **221/252/290/320** sin ella para topes 5/10/15/20. La búsqueda final con entidad midió p50 **8,45 ms**, p95 **11,06 ms**; sin entidad, p50 **453,56 ms**, p95 **937,80 ms**. El corpus mayor afecta mucho más cuando el filtro de entidad no puede acotar el coseno exacto. [Datos de pantalla](../../outputs/iromem/evaluation/candidate-matrix-v1/retrieval-summary.json) · [datos finales](../../outputs/iromem/evaluation/candidate-matrix-v1/final-retrieval-summary.json).

## Elección y semilla final

| Clasificador, tope 20 `plain` | Positivos con semilla correcta | Negativos con semilla vacía | Decisión p50 / p95 | Coste remoto informado |
| --- | ---: | ---: | ---: | ---: |
| Jev 1.13 por OpenRouter | 389/400 (97,25 %) | 80/80 | 506 / 630 ms | US$0,02620 |
| OpenAI GPT-6 Luna Decisions por OpenRouter | 387/400 (96,75 %) | 64/80 | 496 / 774 ms | US$0,04043 |
| Laya local, `choice` | 0/400 | 79/80 | 50 / 92 ms | Sin llamada remota |

`compose-seed-batch` repitió la búsqueda y comprobó candidatos presentados y revisiones. Conservó el mismo número de aciertos finales: **389/400** para Jev y **387/400** para OpenAI. En **todos** esos aciertos, la semilla incluyó el hecho esperado y conservó condición y fuente **declarada**. Composición p50: 9,21 ms con Jev y 8,67 ms con OpenAI en corridas separadas. La procedencia documental no se verificó externamente. [Clasificación final](../../outputs/iromem/evaluation/candidate-matrix-v1/final-classification-summary.json) · [semillas finales](../../outputs/iromem/evaluation/candidate-matrix-v1/final-seeds-summary.json) · [Laya final](../../outputs/iromem/evaluation/candidate-matrix-v1/laya-final-summary.json).

El intervalo Wilson del 95 % para la tasa positiva fue **95,14–98,46 %** en Jev y **94,52–98,09 %** en OpenAI, si se tratan los casos como independientes. Un remuestreo fijo por **proyecto** da **95,75–98,50 %** y **94,25–98,75 %**, respectivamente. Estos intervalos describen la incertidumbre dentro del generador sintético; no son garantías para preguntas humanas.

## Latencia del recorrido y coste de investigación

Con Laya en MPS, vectorizar una consulta individual en una muestra de 140 casos dio p50 **11,51 ms**, p95 **15,43 ms**; cargar el modelo costó **2,24 s** y se informa aparte. En pilotos integrados de 16 preguntas de desarrollo, con un proceso CLI y una conexión SQLite compartida, el tiempo de punta a punta medido fue p50 **569 ms**, p95 **745 ms** con Jev y p50 **585 ms**, p95 **1.066 ms** con OpenAI. Esos pilotos incluyen vectorización, búsqueda, llamada remota y composición; no son percentiles finales sobre los 480 casos. [Vectorización](../../outputs/iromem/evaluation/candidate-matrix-v1/embedding-latency-summary.json) · [piloto Jev](../../outputs/iromem/evaluation/candidate-matrix-v1/end-to-end-jev-development-summary.json) · [piloto OpenAI](../../outputs/iromem/evaluation/candidate-matrix-v1/end-to-end-development-summary.json).

Desarrollo registró **2.235** intentos y US$**0,133294** informado o reservado; calibración, **1.658** y US$**0,097491**; final, **920** y US$**0,066630**. Hubo tres reintentos auditados en desarrollo y dos en calibración por HTTP 503 o pérdida de respuesta. El sondeo remoto inicial y los dos pilotos integrados añadieron aproximadamente US$**0,002541**. El total de esta matriz fue aproximadamente **US$0,299956** informado o reservado, sin tope de gasto. Los costes reservados de intentos sin respuesta no se presentan como cargos confirmados del proveedor. [Desarrollo](../../outputs/iromem/evaluation/candidate-matrix-v1/remote-development-summary.json) · [calibración](../../outputs/iromem/evaluation/candidate-matrix-v1/remote-calibration-summary.json).

## Lectura para el MVP

La recuperación de semillas **cumple el criterio sintético del 95 %** si la consulta llega con una entidad fiable y se presentan hasta veinte candidatos. Entre los clasificadores probados, Jev es la opción provisional más favorable en este banco por positivos, negativos y coste; OpenAI también supera el 95 % observado en positivos. Mantener el filtro de entidad es una condición práctica del resultado, y resolverla automáticamente exige trabajo adicional. La cobertura sin entidad, los negativos de OpenAI, la diversidad limitada y las fuentes solo declaradas impiden trasladar este porcentaje directamente a uso real o afirmar ahorro de tiempo de Codex. No se activó ningún hook.
