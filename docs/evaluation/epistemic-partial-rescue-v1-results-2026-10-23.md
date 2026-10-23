# Rescate de estado epistémico parcial V1 — resultado

## Decisión

La variante alcanzó **16/16 exactas** en el banco de desarrollo V9: recuperó los dos parciales omitidos por el control y mantuvo sus otros 14 aciertos. Sin embargo, excedió las puertas predefinidas de recursos: coste **1,557×** y p95 de servicio **2,230×** del control. **Se rechaza para adopción y se retira del producto**; Decisions `baseline` permanece vigente. No se abrió una cohorte nueva ni V6.

## Calidad y preservación

| Relación | Casos | Decisions `baseline` | Rescate V1 |
| --- | ---: | ---: | ---: |
| Apoya respuesta | 4 | 4/4 | 4/4 |
| Refuta premisa | 4 | 4/4 | 4/4 |
| Parcial/provisional | 4 | 2/4 | 4/4 |
| Ajeno | 4 | 4/4 vacíos | 4/4 vacíos |
| **Exactitud** | **16** | **14/16** | **16/16** |

V1 conserva los 14 IDs correctos del diario baseline y añade solo `v9_field_03` para `v9_field_q03` y `v9_transit_03` para `v9_transit_q03`. No perdió selección útil ni añadió IDs ajenos. El resultado prueba el comportamiento en este conjunto conocido de desarrollo; la revisión ciega previa fue de un agente Codex de la misma familia y no sustituye una cohorte fresca.

## Recursos y puerta

| Medida | Baseline | Rescate V1 | Razón |
| --- | ---: | ---: | ---: |
| Operaciones | 16 | 16 | 1,00× |
| Peticiones HTTP | 16 | 22 inferidas del recibo por caso y la cascada | 1,375× |
| Coste comunicado | USD 0,000398700 | USD 0,000620800 | **1,557× — falla** |
| Clasificador p50/p95 | 426,2 / 1.006,8 ms | 876,5 / 2.247,9 ms | 2,057× / 2,234× |
| Servicio p50/p95 | 428,5 / 1.009,3 ms | 879,8 / 2.250,9 ms | 2,053× / **2,230× — falla** |
| Recibos de coste completos | 16/16 | 16/16 | pasa |
| Degradaciones | 0 | 0 | pasa |

El máximo protocolizado era 32 HTTP; el consumo estimado en la salida por operación indica seis rescates adicionales (16 primeras elecciones + 6 segundas). Coste API nuevo: **USD 0,000620800**. La candidata falla el tope predefinido de coste y p95 ≤1,5×, aunque pase calidad; no se relajan los límites después de ver el resultado. No hubo reintentos. Coste Codex no cuantificado.

## Procedencia, implementación y verificación

Campaña única con Decisions, sin etiquetas en el payload; mismos casos, corpus/base y roster de búsqueda que el control. Hashes: casos `cbd0d2cb95f54b4351532a5e24abc8e004167459ef8611101fdbfa01f56b2581`, SQLite `2fa0d0a7e46bff77b2b3b1970bbc1638439235dff47c615ef1c74970e1dd4b30`, binario de clasificación `f4ae47457f97beb35bb8acbc5c5227df4a0892a2392cd0fc9bf99a173250f794`, evaluador `938c85868adb9afd5feedffbe34102eb0829a476500f22fb6c2239d25cdfab2f`. El manifiesto y el diario por consulta están en [`outputs/iromem/evaluation/quality-goal-dev-v9-2026-10-23/`](../../outputs/iromem/evaluation/quality-goal-dev-v9-2026-10-23/).

`cargo fmt --all --check` y `cargo test --offline --workspace --quiet` pasaron antes de la campaña (1.040 pruebas aprobadas, 25 ignoradas; cero fallidas). El runner de campaña primero rechazó localmente la nueva política porque faltaba en su segunda lista de allowlist; no creó diario ni hizo llamadas. Se añadió una regresión de allowlist, se repitió el test del evaluador y se reconstruyeron binarios antes de la única campaña remota. Tras el rechazo por recursos, se retiró la política del runtime y del runner; el experimento se conserva en este protocolo, informe y diarios, sin cambiar `baseline`.

## Siguiente paso acotado

La cascada recupera información pero duplica trabajo en abstenciones ajenas y parciales; por eso no pasa coste/latencia. No se debe ajustar V1 ni reejecutarlo en V9. El siguiente intento requiere una hipótesis que evite la segunda petición general para controles sin señal relevante y una cohorte de desarrollo nueva; una eventual cohorte confirmatoria debe ser distinta y revisarse a ciegas antes de inferencia. Paso 3 continúa abierto; paso 4 parcial; paso 5 pendiente.
