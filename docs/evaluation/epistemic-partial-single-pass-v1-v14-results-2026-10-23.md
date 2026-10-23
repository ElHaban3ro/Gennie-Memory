# Resultados de confirmación V14: single-pass V1

## Decisión

La confirmación sintética V14 pasa sus criterios congelados. Se habilita `epistemic_partial_single_pass_v1` como política **optativa y explícita** para recuperar hechos parciales del mismo atributo. No cambia el proveedor ni el predeterminado (`baseline`): 32 casos sintéticos en dos cohortes no demuestran una mejora universal para todos los usuarios.

## Calidad emparejada

Decisions con la misma DB, binario, roster y orden obtuvo 14/16 con baseline y 16/16 con la candidata. La mejora fueron exactamente `v14_marine_q03` y `v14_shipping_q03`, ambos hechos parciales con caveat explícito que baseline había omitido. La candidata conservó los otros 14 resultados exactos: apoyos 4/4, refutaciones 4/4 y negativos vacíos 4/4; rescató parciales de 2/4 a 4/4. Cero IDs ajenos, pérdidas, degradaciones o errores técnicos.

| Métrica V14 | Baseline | Single-pass V1 | Cambio |
| --- | ---: | ---: | ---: |
| Casos exactos | 14/16 (87,5 %) | 16/16 (100 %) | +2 casos; +12,5 puntos |
| Apoyo | 4/4 | 4/4 | sin cambio |
| Refutación | 4/4 | 4/4 | sin cambio |
| Parcial/provisional | 2/4 | 4/4 | +2 casos |
| Negativo vacío | 4/4 | 4/4 | sin cambio |
| Coste informado | USD 0,0004180 | USD 0,0006004 | 1,436× (+43,6 %) |
| Servicio p50 | 391,24 ms | 405,15 ms | 1,036× |
| Servicio p95 | 656,32 ms | 627,82 ms | 0,957× |
| Recibos de coste | 16/16 | 16/16 | completos |

El límite congelado era coste y p95 ≤1,5× baseline; ambos pasan. Hubo 32 peticiones HTTP, sin reintentos, por USD 0,0010184 en V14. El p95 de 16 casos es descriptivo de esta tanda y no una garantía de latencia poblacional.

## Comparación con el desarrollo y alcance

V11 también pasó su gate de desarrollo: baseline 15/16 y candidata 16/16, rescatando el único parcial omitido. Costes: USD 0,0004059 y USD 0,0005883, respectivamente; relación 1,449×. El p95 de servicio fue 1.108,59 ms frente a 928,50 ms. Sumadas solo las dos tandas válidas V11 y V14: 64 peticiones, USD 0,0020126 informados. V10 había empatado 16/16 y no se cuenta como ganancia; su coste conjunto fue USD 0,000989. V9 cascade fue rechazada por presupuesto/latencia y permanece retirada. Los costes Codex de revisión no tienen recibo monetario comparable y no se agregan.

El banco y la campaña V14 son sintéticos y pequeños; la revisión ciega la realizó un subagente de la misma familia de modelos. No prueban fidelidad en conversaciones reales, generalización poblacional ni mejora de toda consulta. La adopción queda limitada a la selección optativa de esta política; `baseline` sigue siendo el valor predeterminado.

## Integridad, revisión y reproducción

V14 contiene 4 apoyos, 4 refutaciones, 4 parciales y 4 consultas sin respuesta. La revisión ciega examinó cada pregunta contra sus cuatro candidatos y verificó sujeto/atributo; [adjudicación](assets/quality-goal-confirm-v14/blind-adjudication.md) y [protocolo congelado](epistemic-partial-single-pass-v1-v14-protocol-2026-10-23.md). DB SHA-256 `3505ac8d456ed76ec586a559148b811e726fe95e61e1f5931731edd436372a30`; binario `e99793096b1c2bc5260b062607a14082b2d7a17c0109c8933967d5a8bc89d7a2`; casos `ba1ec2107b0f9a8d84f7c0bd745c3c342bcd8686540f1475d09eeecfbd325acf`. Archivos fuente y revisiones conservados en `assets/quality-goal-confirm-v14/`; diarios, recibos y reportes en `outputs/iromem/evaluation/quality-goal-confirm-v14-2026-10-23/`.

V12 se descartó antes de inferencia: `v12_shipping_q04`, `v12_appointments_q04` y `v12_software_q04` estaban etiquetadas como negativas aunque sus preguntas y candidatos contenían la respuesta literal. La primera revisión ciega no detectó esa discordancia. V13 corrigió las tres, pero una sustitución dejó `v13_marine_q02` preguntando por Sora cuando el recuerdo corregía a Rhea; se descartó también antes de inferencia. V14 corrigió sujeto y consultas sin respuesta y recibió revisión estricta caso por caso. Ninguna versión rechazada generó peticiones HTTP; no se ajustó gold tras llamadas y se conservan sus archivos/hashes.
