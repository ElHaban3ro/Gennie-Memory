# Expansión léxica de la ruta rápida

Estado: evaluación sintética en esta Mac, 2026-10-15. El [paquete por caso](../../outputs/iromem/evaluation/keyword-prefix-v1/) conserva consultas, diarios, manifiestos y resúmenes de Rust. Esta corrida mejora la ruta FTS5 vigente; no activa la búsqueda híbrida ni integra Codex. Los fallos de los sujetos 002–005 ya eran conocidos de campañas anteriores y ayudaron a plantear la expansión. La cohorte 010–013 se preparó después para una comprobación adicional sin ajustar más el cambio.

Las figuras históricas de esta cohorte muestran [aciertos](assets/current-accuracy.svg) y [búsqueda de candidatos](assets/current-latency.svg). Se regeneran con `gennie-memory-eval render-current-charts` desde los resúmenes de Gennie Memory, Decisions y Engram para las preguntas 010–013. La [comparación anterior de respuestas con Codex](assets/current-codex-comparison.svg) proviene del [piloto de seis turnos](codex-current-v1.md) sobre la misma ruta. Las [figuras de ajuste de aciertos](assets/keyword-accuracy-v1.svg) y [latencias](assets/keyword-latency-v1.svg) se conservan como evidencia histórica y se regeneran con `gennie-memory-eval render-keyword-charts` desde los seis resúmenes anteriores. La [comparación vigente](full-comparison-2026-10-16.md) usa sujetos 014–017.

## Cambio congelado antes de la cohorte nueva

La consulta de palabras clave conserva cada término exacto y, para términos de siete o más caracteres, añade una coincidencia FTS5 por prefijo que omite los dos últimos caracteres. Por ejemplo, `invitación` puede alcanzar `invitaciones`. No se modifica el índice, el ámbito, la entidad, la vigencia ni el límite de veinte candidatos. La instrucción de Decisions ahora pide distinguir el hecho solicitado de una simple coincidencia de persona o proyecto, admite paráfrasis claras y mantiene `none` cuando falta el dato. La idea de expansión partió de fallos conocidos de 002–005; las variantes concretas se compararon en desarrollo.

## Desarrollo: 115 positivos y 20 negativos

| Variante | Candidato correcto en top 20 | Semilla correcta | Negativos vacíos | p50 consulta → semilla | Tokens de entrada p50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ruta e instrucción anteriores | 112/115 | 105/115 | 20/20 | 379 ms | 537 |
| Prefijos, instrucción anterior | 114/115 | 108/115 | 20/20 | 370 ms | 537 |
| Prefijos, instrucción nueva | 114/115 | **109/115** | 20/20 | 380 ms | 562 |

La expansión recuperó los recuerdos de invitación y documentación en desarrollo. El caso de supuesto abandonado siguió sin coincidencia léxica. La nueva instrucción cambió tres selecciones positivas: acertó dos que la anterior falló y falló una que la anterior acertó. La ganancia neta observada fue una semilla; el mayor prompt informó 25 tokens adicionales de mediana. Los fallos previamente publicados de 002–005 orientaron el mecanismo; estas cifras de desarrollo eligieron la variante antes de repetir 002–005.

## Repetición de 002–005: 460 positivos y 80 negativos

| Medición | Ruta anterior | Prefijos e instrucción nueva |
| --- | ---: | ---: |
| Recuerdo correcto entre 20 candidatos | 452/460 | **457/460** |
| Semilla correcta en positivos | 424/460 (92,2 %) | **442/460 (96,1 %)** |
| Negativos con semilla vacía | 80/80 | 80/80 |
| Búsqueda p50 / p95 | 5,72 / 6,63 ms | 6,45 / 7,03 ms |
| Consulta → semilla p50 / p95 | 376 / 558 ms | 382 / 631 ms |
| Tokens de entrada informados p50 | 535 | 560 |
| Coste total informado en 540 llamadas | US$0,0283857 | US$0,0297487 |
| Respuestas degradadas / fugas de IDs prohibidos | 0 / 0 | 0 / 0 |

La tasa positiva observada superó el objetivo del 95 % **en este banco sintético reutilizado**. Quedaron tres positivos sin el recuerdo correcto entre candidatos y quince con candidato disponible pero sin semilla correcta. La búsqueda es 0,73 ms más lenta en mediana; la consulta completa es 5,6 ms más lenta en mediana y 73 ms más lenta en p95. No atribuí esa cola únicamente al cambio local: las llamadas a OpenRouter ocurrieron en corridas diferentes. Los 80 negativos vacíos miden abstención de semilla, no ausencia de candidatos: ambos métodos encontraron candidatos también para negativos.

## Cohorte nueva 010–013: 460 positivos y 80 negativos

`gennie-memory-eval select-cohort` derivó y congeló [540 consultas](../../outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-queries.jsonl) de 16 ámbitos sintéticos previamente reservados (sujetos 010–013 en `user`, `project`, `conversation` y `agent`); SHA-256 `bffea972319ca1b61b91c869c7c0d80bac78509e80ed2d23c0299edd0461b3a0`. El código de búsqueda y la instrucción ya estaban fijados. No se volvió a ajustar tras observar esta cohorte.

| Medición de la cohorte nueva | Resultado |
| --- | ---: |
| Recuerdo correcto entre 20 candidatos | 456/460 |
| Semilla correcta en positivos | **439/460 (95,4 %)** |
| Negativos con semilla vacía | 80/80 |
| Búsqueda p50 / p95 | 7,11 / 26,54 ms |
| Consulta → semilla p50 / p95 | 386 / 958 ms |
| Tokens de entrada informados p50 | 561 |
| Coste total informado en 540 llamadas | US$0,0297485 |
| Respuestas degradadas / fugas de IDs prohibidos | 0 / 0 |

También aquí se alcanzó el **95 % observado**, aunque 21 positivos no produjeron la semilla correcta: cuatro sin candidato esperado y diecisiete con candidato disponible. El p95 alto se observó en esta ejecución, sin una comparación emparejada de red que permita atribuirlo a la expansión. Los casos comparten plantillas y banco de recuerdos con las particiones anteriores; las personas y respuestas concretas son distintas. La proporción observada no garantiza al 95 % estadístico una tasa poblacional superior al objetivo ni representa conversaciones humanas.

## Engram en las mismas 540 preguntas

El evaluador Rust importó los mismos 2.800 recuerdos sintéticos a un almacén aislado de Engram 3.4.0. `mem_search` recibió las mismas 540 preguntas de sujetos 010–013, el ámbito equivalente, `match_mode=any` y un máximo de 20 resultados. Se calentó una consulta fuera de la medición; ambas herramientas se midieron en corridas locales seriales distintas. El [diario Engram por caso](../../outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-engram.jsonl), el [resumen](../../outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-engram-summary.json) y el manifiesto conservan sus entradas y resultados.

| Búsqueda | Gennie Memory vigente | Engram MCP |
| --- | ---: | ---: |
| Recuerdo correcto entre 20 candidatos | 456/460 | 456/460 |
| Recuerdo correcto en primer lugar | 420/460 | 425/460 |
| Latencia p50 | 7,11 ms | 10,80 ms |
| Latencia p95 | 26,54 ms | 14,96 ms |

Gennie Memory fue 3,69 ms más rápido en la mediana de esta ejecución; Engram tuvo una cola p95 11,58 ms más corta. Una sola corrida por método, en orden fijo, no permite declarar superioridad estable. Engram recibió el resumen como título y el texto original como cuerpo, sin las condiciones ni fuentes declaradas de Gennie Memory. Esta comparación mide recuperación de candidatos, no una semilla clasificada de Engram ni respuestas de Codex. La campaña de Decisions se ejecutó por separado y no se atribuye a Engram.

## Método y límites

`gennie-memory-eval search-benchmark` ejecutó 135 búsquedas de desarrollo, 540 repetidas de 002–005 y 540 de 010–013 sobre la misma base SQLite sintética de 2.800 recuerdos, con sesión Rust persistente. `gennie-memory-eval retrieval` verificó etiquetas, candidatos, ámbitos y latencias. `gennie-memory-eval campaign` realizó 135 llamadas para cada variante de desarrollo y una campaña de 540 llamadas para cada una de las dos cohortes posteriores con Decisions; `campaign-report` calculó semillas, negativos, uso y tiempos. Los diarios guardan SHA-256 de consultas, base y binario. El informe anterior y sus registros siguen en [rust-migration-v1.md](rust-migration-v1.md).

El primer intento de desarrollo dentro del entorno de red restringido terminó en `transport_error` para todas las llamadas y **no se incluyó**. Una comprobación de una llamada sintética confirmó el transporte con acceso de red; los diarios completos nuevos no tuvieron respuestas degradadas. Las pruebas usan sujetos sintéticos y plantillas repetidas entre particiones; alcanzar 96,1 % aquí no prueba esa tasa en recuerdos reales ni garantiza el límite inferior estadístico del 95 %. El piloto Codex anterior usó la ruta anterior; no se recalculó su tiempo con esta implementación.

Queda investigar el vacío léxico de los supuestos abandonados sin introducir una regla ajustada a esas tres etiquetas finales, y medir la integración automática con Codex cuando exista. El p95 de clasificación es otra línea de trabajo antes de declarar la viabilidad del producto.
