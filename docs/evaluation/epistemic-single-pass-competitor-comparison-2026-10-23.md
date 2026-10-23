# Comparación de single-pass con Decisions y motores de búsqueda

## Lectura principal

Se ejecutó una comparación emparejada sobre la cohorte congelada 014–017: 2.800 recuerdos, 540 preguntas (460 positivas y 80 negativas), misma consulta, base SQLite, binario, modelo solicitado y proveedor Decisions. La única diferencia de producto entre los brazos fue `selection_policy`, enviada explícitamente en cada solicitud. El baseline se midió de nuevo después de corregir el evaluador para que no dependiera del default runtime.

En esta cohorte, `epistemic_partial_single_pass_v1` acertó **401/460 positivas (87,17 %)** y baseline **441/460 (95,87 %)**. Ambos dejaron vacías las 80 negativas, sin IDs prohibidos ni respuestas degradadas. Caso a caso hubo **42 aciertos solo con baseline**, **2 solo con single-pass** y 496 casos con el mismo resultado. La nueva política recuperó 457/460 positivos entre los candidatos igual que baseline, por lo que la diferencia aparece después de la búsqueda, durante la selección.

El coste informado por Decisions fue USD 0,035898 con single-pass y USD 0,029742 con baseline (**+20,7 %**). La latencia total p50/p95 fue 422/950 ms y 409/863 ms; los tokens de entrada p50 fueron 674 y 560. Son corridas secuenciales del mismo modelo solicitado; el tiempo de servicio puede variar entre llamadas.

Tras revisar los resultados, el usuario decidió restaurar `baseline` como política predeterminada. Single-pass queda disponible explícitamente como opción experimental. Esta campaña contradice la ganancia observada en las cohortes V11/V14; no debe presentarse como una mejora general. El banco es sintético. Véase la [decisión 0044](../decisions/0044-restore-baseline-default.md).

## Búsqueda frente a Engram, Lodis y Basic Memory

La política actúa después de recuperar candidatos y no modifica FTS5, ranking ni límites. Por eso se reutilizan las cuatro corridas históricas sobre la misma cohorte; no se repitieron llamadas a otros motores.

| Motor y ruta de búsqueda | Esperado en top 20 | Esperado en top 1 | p50 / p95 |
| --- | ---: | ---: | ---: |
| iROMEM FTS5 | **457/460 (99,35 %)** | 420/460 (91,30 %) | **6,93 / 7,77 ms** |
| Engram 3.4.0 MCP | 456/460 (99,13 %) | **427/460 (92,83 %)** | 7,22 / 10,71 ms |
| Lodis 0.6.0 híbrido MCP | 389/460 (84,57 %) | 47/460 (10,22 %) | 19,67 / 40,94 ms |
| Basic Memory local | 280/460 (60,87 %) | 276/460 (60,00 %) | 24,12 / 127,72 ms |

Esta tabla compara recuperación, no selección de semilla. El banco no incluye un clasificador de semilla equivalente para Engram, Lodis o Basic Memory. Basic Memory dejó vacías 80/80 búsquedas negativas, pero eso no equivale a una decisión clasificada de abstención.

## Ejecuciones y coste

- Campaña válida single-pass + Decisions: 540 llamadas, USD 0,035898 informado.
- Control válido baseline + Decisions: 540 llamadas, USD 0,029742 informado.
- Coste de comparación válida: **USD 0,065640**, por 1.080 llamadas.
- Se detectó un defecto en el evaluador: la primera corrida solicitada como baseline omitía `selection_policy` y, tras el cambio de default, ejecutó single-pass. Sus 540 llamadas y USD 0,035898 se conservan bajo `invalid-omitted-policy-defaulted-to-single-pass.jsonl`; no se cuentan como control. Gasto informado total de las tres campañas de esta revisión: **USD 0,101538**.
- No se hicieron llamadas nuevas a Engram, Lodis ni Basic Memory.

El coste es el informado por el proveedor, no una factura auditada. Las respuestas son sintéticas, con etiquetas de fixture deterministas; no estiman calidad en recuerdos de usuarios.

## Reproducción

- [Diario y manifiesto de single-pass](../../outputs/iromem/evaluation/epistemic-single-pass-competitor-2026-10-23/decisions-single-pass.jsonl)
- [Reporte de single-pass](../../outputs/iromem/evaluation/epistemic-single-pass-competitor-2026-10-23/decisions-single-pass-report.json)
- [Diario y manifiesto de baseline explícito](../../outputs/iromem/evaluation/epistemic-single-pass-competitor-2026-10-23/decisions-baseline-verified.jsonl)
- [Reporte de baseline](../../outputs/iromem/evaluation/epistemic-single-pass-competitor-2026-10-23/decisions-baseline-verified-report.json)
- [Corrida descartada por omitir la política](../../outputs/iromem/evaluation/epistemic-single-pass-competitor-2026-10-23/invalid-omitted-policy-defaulted-to-single-pass.jsonl)
- [Comparación ampliada histórica de búsqueda](broader-comparison-2026-10-16.md)
- [Figura SVG](assets/epistemic-single-pass-competitor-2026-10-23.svg)
