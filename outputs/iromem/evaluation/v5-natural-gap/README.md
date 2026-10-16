# Sondeo v5 de preguntas completas sin coincidencia textual

Se evaluaron **400 peticiones sintéticas completas**: 200 positivas que piden una pauta para videollamada o encuentro presencial, y 200 negativas que no eligen modalidad o prohíben consultar recuerdos. La [muestra congelada](../../../../tests/benchmark/v5/natural-gap-probe/manifest.json) tiene 200 casos de desarrollo y 200 de calibración, con 50 entidades distintas por partición y sin abrir `final`. Las etiquetas, el ámbito y las fuentes se heredaron de v5; las preguntas en inglés se redactaron para este sondeo. Hay 40 frases distintas, reutilizadas entre entidades; estas 400 peticiones no son 400 formulaciones independientes.

FTS5 devolvió **cero candidatos en los 400 casos**. La búsqueda híbrida con el encoder Laya local tuvo cobertura semántica completa e incluyó el recuerdo etiquetado entre sus dos candidatos en los 200 positivos. Los tres selectores híbridos vieron los **mismos candidatos y vectores exactos** por caso.

| Método | Positivos exactos / 200 | Negativos vacíos / 200 | p50 desarrollo / calibración |
| --- | ---: | ---: | ---: |
| Filtros + FTS5, top‑1 | 0 | 200 | 0,51 / 0,51 ms |
| Híbrida, top‑1 | 104 | 0 | 13,33 / 13,37 ms |
| Híbrida, Laya `choice` | 2 | 188 | 43,58 / 42,01 ms |
| Híbrida, Laya `noul` condicionado (0,5) | 99 | 0 | 71,54 / 72,18 ms |

Cada partición tiene 100 positivos y 100 negativos. En desarrollo, top‑1 acertó 52 positivos, `choice` 2 y Laya condicionado 50; en calibración, 52, 0 y 49. `Choice` dejó vacíos 94/100 negativos en cada partición. Top‑1 y la variante condicionada añadieron memoria en **todos** los negativos. FTS5 acertó los negativos porque no encontró candidatos, pero omitió todos los positivos.

| Segmento positivo | Casos por partición | Top‑1 desarrollo / calibración | Laya condicionado desarrollo / calibración |
| --- | ---: | ---: | ---: |
| Videollamada | 50 | 11 / 16 | 47 / 45 |
| Encuentro presencial | 50 | 41 / 36 | 3 / 4 |

La asimetría de orden muestra que recuperar el recuerdo entre candidatos no basta para resolver la semilla. El índice semántico aporta cobertura cuando la coincidencia textual falla, pero ningún selector evaluado maneja bien simultáneamente positivos y negativos. La mezcla 50/50 hace que una exactitud agregada sea poco informativa; se reportan ambos tipos por separado. En los positivos acertados, hecho, condición y fuente declarada permanecieron fieles. No hubo fallos de decisión, semillas degradadas, filtraciones de IDs prohibidos ni llamadas remotas. Top‑1 usa Laya para codificar la pregunta, aunque no lo llama como clasificador.

Un barrido exploratorio de **todos los puntos de corte** de las puntuaciones binarias ya guardadas encontró como máximo 100/200 exactos en cada partición, al abstenerse de **todos** los positivos y negativos. A umbral 0,95, desarrollo conservó 1/100 positivos y dejó vacíos 84/100 negativos; calibración 1/100 y 88/100. El candidato correcto solo quedó primero por puntuación binaria en 50/100 y 49/100 positivos. Ajustar únicamente el umbral no produce una política útil aquí; no se eligió umbral de producción.

Las frases son más parecidas a peticiones completas que las listas de palabras del [primer sondeo léxico](../v5-lexical-gap/README.md), pero siguen siendo **plantillas sintéticas escritas para eliminar coincidencias FTS5**. No son una muestra de tráfico humano y las etiquetas derivadas no han tenido revisión humana. Cada método se ejecutó una vez y en orden fijo; los percentiles no estiman variación térmica o entre repeticiones. Este sondeo no fija una política de producción ni permite abrir `final`.

## Evidencia

- [Resumen emparejado](summary.json): aciertos por partición y segmento, latencias, tokens y huellas de entradas e informes. [Manifiestos de los ocho diarios](execution-manifests.json): 201 eventos por recorrido, misma base, binario y versión de runner.
- [Barrido de umbral en desarrollo](development-threshold-analysis.json) y [calibración](calibration-threshold-analysis.json): reaplican umbrales a puntuaciones congeladas, sin otra inferencia.
- [Instantánea híbrida de desarrollo](development-hybrid-results.jsonl) y [de calibración](calibration-hybrid-results.jsonl): candidatos y vectores exactos. Los vectores de `choice` y condicionado se verificaron contra estas instantáneas antes de compactar sus informes.
- Desarrollo: [FTS5](v5-natural-gap-development-fts5-report.json), [híbrida top‑1](v5-natural-gap-development-top1-report.json), [Laya `choice`](v5-natural-gap-development-choice-report.json), [Laya condicionado](v5-natural-gap-development-conditions-report.json).
- Calibración: [FTS5](v5-natural-gap-calibration-fts5-report.json), [híbrida top‑1](v5-natural-gap-calibration-top1-report.json), [Laya `choice`](v5-natural-gap-calibration-choice-report.json), [Laya condicionado](v5-natural-gap-calibration-conditions-report.json).
- El generador histórico de estas peticiones fue retirado; los casos v5 derivados permanecen congelados.
