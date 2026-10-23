# Resultados V11 de desarrollo: single-pass V1

V11 es una cohorte sintética nueva de 16 casos, con revisión ciega previa (4 por relación), sin etiquetas en los mensajes al proveedor. Decisions baseline obtuvo **15/16**: apoyo 4/4, refutación 4/4, parcial 3/4 y negativos vacíos 4/4. La política congelada `epistemic_partial_single_pass_v1` obtuvo **16/16** y rescató únicamente `v11_field_q03`; conservó los otros 15 IDs exactos, sin IDs ajenos, pérdidas, degradaciones ni fallos técnicos.

| Métrica | Baseline | Single-pass V1 | Relación |
| --- | ---: | ---: | ---: |
| Exactitud | 15/16 (93,75 %) | 16/16 (100 %) | +1 caso |
| Coste informado | USD 0,0004059 | USD 0,0005883 | 1,449× |
| Servicio p50 | 434,57 ms | 455,00 ms | 1,047× |
| Servicio p95 | 1.108,59 ms | 928,50 ms | 0,838× |
| Recibos | 16/16 | 16/16 | completos |

Pasa el gate de desarrollo, pero no autoriza por sí solo adopción: el protocolo exigía confirmación independiente, completada en [V14](epistemic-partial-single-pass-v1-v14-results-2026-10-23.md). La cohorte es sintética, pequeña y revisada dentro de una misma familia de modelos; no mide calidad poblacional. DB SHA-256 `8457b7fc2c0353a7549dd315d103eeaec940e90c9f0cb20afba6abca6a3cf97f`, casos `4f7e9d8af02effe2f42eaad7c2cad21adde14d63aa520e5098345bf70543592e`, binario `e99793096b1c2bc5260b062607a14082b2d7a17c0109c8933967d5a8bc89d7a2`. La clave, los diarios y recibos están en `assets/quality-goal-dev-v11/` y `outputs/iromem/evaluation/quality-goal-dev-v11-2026-10-23/`; [protocolo](epistemic-partial-single-pass-v1-v11-protocol-2026-10-23.md).
