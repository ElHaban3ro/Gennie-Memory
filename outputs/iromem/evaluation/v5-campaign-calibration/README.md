# Campaña local v5 de memorias semilla — calibración

Estado: **tres recorridos locales completos** el 2026-10-15 en esta Mac. Corpus: [1.480 consultas sintéticas congeladas](../../../../tests/benchmark/v5/calibration-campaign-1480.jsonl), SHA-256 `e93b7a64d0083603423f3fa21042475c7a6738054bd25d14b1b4ed746e3209a5`: una consulta por cada una de las 1.000 formas positivas y 480 negativas (80 por segmento). Cero IDs y cero entidades compartidos con la [campaña de desarrollo](../v5-campaign-development/README.md). La partición `final` no se ejecutó.

Los tres selectores usaron MPS, el mismo índice de 1.200 recuerdos activos, el encoder experimental Laya 0.3.5 de 768 dimensiones y los **mismos candidatos en el mismo orden** para cada consulta. Los vectores híbridos de las tres corridas coincidieron caso por caso. La cobertura semántica fue completa en 1.480/1.480 y el recuerdo etiquetado apareció entre los cuatro candidatos en 1.000/1.000 positivos. No hubo fallos de decisión, semillas degradadas, filtraciones de IDs prohibidos ni llamadas remotas.

| Selector | Positivos exactos / 1.000 | Negativos vacíos / 480 | Semillas exactas / 1.480 | p50 total | p95 total | p99 total | Tokens de clasificación |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Top‑1 sin clasificador | 701 | 160 | 861 | 20,12 ms | 38,31 ms | 40,07 ms | 0 |
| Laya `choice` | 149 | 453 | 602 | 52,92 ms | 87,20 ms | 90,06 ms | 235.264 |
| Laya `noul` con condiciones, umbral 0,5 | 456 | 160 | 616 | 87,11 ms | 135,57 ms | 140,23 ms | 490.816 |

En los positivos correctos, hecho, condición y referencia de fuente se conservaron en la semilla. Top‑1 eligió otra memoria en 299 positivos y añadió memoria a 320 negativos. `choice` omitió 672 positivos, eligió otra memoria en 179 y añadió memoria a 27 negativos. `noul` condicionado omitió 229 positivos, eligió otra memoria en 315 y añadió memoria a 320 negativos. La separación vista en desarrollo persiste en entidades distintas: `choice` evita más falsos recuerdos negativos, mientras top‑1 conserva más recuerdos útiles. Ninguna variante es una política lista para producción.

| Segmento | Casos | Top‑1 | Laya `choice` | Laya `noul` condicionado |
| --- | ---: | ---: | ---: | ---: |
| `absent_entity` | 80 | 80 | 80 | 80 |
| `alias` | 200 | 160 | 31 | 84 |
| `code_switch` | 100 | 58 | 19 | 31 |
| `direct` | 250 | 202 | 46 | 80 |
| `english` | 200 | 100 | 37 | 87 |
| `explicit_no_recall` | 80 | 0 | 80 | 0 |
| `indirect` | 150 | 130 | 9 | 79 |
| `long_context` | 100 | 51 | 7 | 95 |
| `mode_unknown` | 80 | 0 | 80 | 0 |
| `mode_unspecified` | 80 | 0 | 80 | 0 |
| `wrong_scope` | 80 | 80 | 80 | 80 |
| `wrong_task` | 80 | 0 | 53 | 0 |

El [barrido de umbrales](threshold-analysis.json) sobre las puntuaciones binarias ya guardadas encontró un máximo exploratorio de 733/1.480 en umbral 0, frente a 861/1.480 de top‑1. El candidato correcto quedó primero por puntuación binaria en 573/1.000 positivos. Cambiar solo ese umbral no resuelve los errores de orden ni la aplicabilidad; no se eligió un umbral de producción.

## Evidencia

Una [ablación posterior](../../../../docs/evaluation/retrieval-ablation-v5.md) encontró que filtros estructurados + FTS5 devolvieron exactamente los mismos candidatos y 861/1.480 semillas exactas que esta ruta híbrida, con p50 de 5,51 ms frente a 20,12 ms.

- [Resumen validado](v5-calibration-summary.json): métricas por segmento, pares de aciertos y huellas de los informes públicos.
- [Manifiestos](execution-manifests.json): tres diarios con 1.481 eventos cada uno, ligados a la misma muestra, base, binario, lote semántico, modelo y versión del runner. Los diarios originales permanecen locales en `.local/pilots/`.
- Informes por caso: [top‑1](v5-calibration-top1-1480-report.json), [Laya `choice`](v5-calibration-laya-choice-1480-report.json) y [Laya condicionado](v5-calibration-laya-conditions-1480-report.json). Cada uno conserva candidatos, estado, fidelidad y latencias; registra la huella del informe original antes de deduplicar los vectores.
- [Instantánea híbrida](v5-calibration-hybrid-results.jsonl): candidatos y vectores exactos de las 1.480 consultas, validados contra el recorrido top‑1. Los vectores de los otros dos recorridos se contrastaron con esta instantánea antes de compactar sus informes.

Las etiquetas son reglas deterministas del generador, sin revisión humana. Aunque las entidades no se repiten entre desarrollo y calibración, ambos conjuntos comparten las mismas familias de plantillas; esto no prueba generalización a consultas humanas. El host sintético ya resuelve la entidad y las referencias de fuente son declaradas. Un solo recorrido por método no estima variación térmica o de caché. Top‑1 sigue usando Laya para codificar la consulta en la búsqueda híbrida; sus cero tokens corresponden únicamente a clasificación. No se midió ahorro neto del agente principal ni CPU/RAM global, porque este MVP no tiene hook.
