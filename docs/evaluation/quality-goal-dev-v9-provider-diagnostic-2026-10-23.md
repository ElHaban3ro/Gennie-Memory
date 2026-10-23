# Diagnóstico emparejado de proveedor V9 — 2026-10-23

## Resultado

Decisions `baseline` sigue siendo la mejor de las dos configuraciones medidas en este banco de desarrollo: **14/16 exactas** frente a **12/16** con Jev 1.13. No se adopta una candidata ni se altera la configuración vigente. Jev cuesta menos y responde más rápido en esta tanda, pero pierde dos casos exactos; por sí solo no resuelve el problema de calidad.

| Relación | Casos | Decisions `baseline` | Jev 1.13 `baseline` |
| --- | ---: | ---: | ---: |
| Apoya la respuesta | 4 | 4/4 | 3/4 |
| Refuta la premisa | 4 | 4/4 | 4/4 |
| Parcial o provisional | 4 | 2/4 | 1/4 |
| Ajeno | 4 | 4/4 vacíos | 4/4 vacíos |
| **Exactitud total** | **16** | **14/16** | **12/16** |

Decisions omite `v9_field_q03` (pronóstico provisional frente a medición real) y `v9_transit_q03` (hora prevista frente a llegada real). Jev omite esos dos parciales y además `v9_proc_q03` (fecha estimada frente a fecha confirmada); en `v9_incident_q01` también se abstiene ante una respuesta completa. Ambos conservan las cuatro refutaciones y se abstienen ante los cuatro controles ajenos. Los diarios contienen los IDs seleccionados por caso.

## Coste y latencia observados

| Medida | Decisions `baseline` | Jev 1.13 `baseline` |
| --- | ---: | ---: |
| Operaciones / recibos de coste | 16 / 16 | 16 / 16 |
| Modelo servido | `openai/gpt-6-luna-decisions-20261006` | `typesafe/jev-1.13-20260917` |
| Coste comunicado | USD 0,000398700 | USD 0,000318276 |
| Tokens de entrada p50 / salida p50 | 247 / 0 | 471 / 47 |
| Clasificador p50 / p95 | 426,2 / 1.006,8 ms | 265,5 / 515,8 ms |
| Servicio p50 / p95 | 428,5 / 1.009,3 ms | 267,3 / 517,9 ms |

Jev comunicó 20,2 % menos coste total y menor latencia en esta tanda. Son campañas de red separadas y la muestra es pequeña: no atribuir causalidad ni generalizar esos percentiles. Ninguna operación se degradó; no hubo reintentos. Coste API nuevo de este contraste: **USD 0,000318276**. No se cuantifica coste de Codex.

## Diseño, procedencia y límites

Ambas campañas usaron los mismos 16 casos, revisión ciega, binario (`cef62706…`), SQLite (`2fa0d0a7…`), orden, candidatos FTS y política `baseline`; solo cambió el proveedor solicitado. El manifiesto confirma hashes iguales de casos (`cbd0d2cb…`), binario y base. Las etiquetas no se enviaron al clasificador. Es un diagnóstico del conjunto sintético de desarrollo V9, no una cohorte confirmatoria, estimación poblacional, evaluación de respuestas finales ni prueba de procedencia externa.

La diferencia observada se concentra en la selección de evidencia parcial/provisional; intercambiar Decisions por Jev no mejora el mecanismo. Se descarta el cambio de proveedor como solución y se conserva Decisions `baseline`. Los candidatos `utility_v1`, V2/V3, `roles_v1` y `witness_v1` siguen retirados según sus resultados anteriores; este contraste no los rehabilita ni los repite. El paso 3 continúa abierto. Una próxima candidata debe abordar la distinción entre evidencia parcial relevante y resultado confirmado, a la vez que conserva apoyos, refutaciones y abstenciones ajenas. Solo si supera el gate local se abrirá una validación nueva revisada a ciegas.

## Archivos reproducibles

- Banco y dictamen: [V9](assets/quality-goal-dev-v9/blind-adjudication.md)
- Preflight de búsqueda: [resultado local](assets/quality-goal-dev-v9/local-search-results.md)
- Diarios y reportes de ambas campañas: [`outputs/iromem/evaluation/quality-goal-dev-v9-2026-10-23/`](../../outputs/iromem/evaluation/quality-goal-dev-v9-2026-10-23/)
