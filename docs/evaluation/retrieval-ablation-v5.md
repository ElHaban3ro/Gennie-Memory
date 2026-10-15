# Ablación v5: filtros estructurados + FTS5 frente a búsqueda híbrida

Se ejecutó top‑1 sobre las muestras sintéticas congeladas de [desarrollo](../../tests/benchmark/v5/development-campaign-1600.jsonl) y [calibración](../../tests/benchmark/v5/calibration-campaign-1480.jsonl). Ambas rutas aplicaron los filtros estructurados del mismo núcleo Rust y compusieron semillas validadas; la ruta híbrida añadió encoder Laya, coseno exacto y RRF. Se usaron la misma base SQLite y el mismo binario por partición, sin llamadas remotas ni hook.

| Partición | Casos | FTS5 exactas | Híbrida exactas | Candidatos distintos | FTS5 p50 / p95 | Híbrida p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Desarrollo | 1.600 | 919 | 919 | 0 | 5,50 / 11,61 ms | 20,04 / 37,90 ms |
| Calibración | 1.480 | 861 | 861 | 0 | 5,51 / 11,72 ms | 20,12 / 38,31 ms |

La comparación por ID encontró **cero diferencias** en orden de candidatos, contenido de hecho/condición/fuente, elección y exactitud en 3.080 casos. El recuerdo esperado estuvo entre los cuatro candidatos de FTS5 en los 2.000 positivos. La ruta híbrida informó cobertura semántica completa en ambas corridas; la textual la informó deshabilitada. No hubo fallos de decisión, semillas degradadas ni filtraciones de IDs prohibidos. Los tiempos son del recorrido completo local y provienen de ejecuciones separadas, en orden fijo; caché y estado térmico pueden influir en la diferencia. El encoder Laya forma parte del tiempo híbrido; top‑1 no llama al clasificador en ninguna de las dos rutas.

La conclusión válida para este banco es que **añadir búsqueda semántica no cambió la calidad top‑1 y elevó la latencia observada**. No demuestra que FTS5 sea suficiente en consultas reales: las preguntas positivas del generador conservan señales léxicas y ninguna forzó un fallo de cobertura textual. Un [sondeo adicional equilibrado de 400 preguntas sin coincidencia textual](../../outputs/iromem/evaluation/v5-lexical-gap/README.md) encontró que FTS5 devolvió cero candidatos; la ruta híbrida recuperó el recuerdo esperado entre candidatos en 200/200 positivos, pero top‑1 acertó solo 113 y añadió memoria en 200/200 negativos. Ese sondeo está construido para provocar el fallo léxico y tampoco representa tráfico humano. Aún faltan entidades ambiguas, preguntas naturales y etiquetas revisadas.

Un [segundo sondeo con 400 peticiones sintéticas completas](../../outputs/iromem/evaluation/v5-natural-gap/README.md) mantuvo la cobertura híbrida de 200/200 positivos y cero candidatos FTS5. Top‑1 acertó 104 positivos y ningún negativo; Laya `choice`, 2 positivos y 188 negativos; Laya condicionado, 99 positivos y ningún negativo. Esto confirma el intercambio de selección con frases completas, pero todavía faltan preguntas escritas por usuarios y etiquetas revisadas.

Evidencia reproducible:

- [Comparación por caso de desarrollo](../../outputs/iromem/evaluation/v5-campaign-development/fts5-vs-hybrid-top1.json) y [calibración](../../outputs/iromem/evaluation/v5-campaign-calibration/fts5-vs-hybrid-top1.json), generadas con [`compare_retrieval_ablation.py`](../../scripts/compare_retrieval_ablation.py).
- Informes FTS5 por caso: [desarrollo](../../outputs/iromem/evaluation/v5-campaign-development/v5-development-fts5-top1-1600-report.json) y [calibración](../../outputs/iromem/evaluation/v5-campaign-calibration/v5-calibration-fts5-top1-1480-report.json). Las comparaciones registran hashes de estos informes y de los informes híbridos.
- Manifiestos FTS5: [desarrollo](../../outputs/iromem/evaluation/v5-campaign-development/fts5-top1-execution.json) y [calibración](../../outputs/iromem/evaluation/v5-campaign-calibration/fts5-top1-execution.json); validan los diarios completos y la igualdad de consulta, base y binario con cada campaña híbrida.

Para regenerar las comparaciones desde los informes publicados, ejecutar `python3 scripts/compare_retrieval_ablation.py --queries <muestra> --fts5 <informe-fts5> --hybrid <informe-híbrido> --output <salida>` con las rutas enlazadas arriba. El script rechaza particiones mezcladas o `final`, informes incompletos y estados degradados.
