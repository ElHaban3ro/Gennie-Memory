# Scoped Anchor V1: control literal en una petición

Variante optativa `per_memory_scoped_anchor_v1`: siete candidatos, mismas vías de dato/corrección de Scoped V1 y elección global del nombre del caso dentro del mismo HTTP. Un control local comprueba presencia literal en cada resumen. No demuestra identidad semántica: el modelo puede elegir un nombre equivocado, y `none` conserva el comportamiento scoped. La configuración predeterminada sigue Decisions/baseline.

El recorte a siete conserva retrospectivamente todos los IDs seleccionados por Scoped V1 en sus 64 preguntas V5. Esa comprobación no demuestra calidad de la nueva política. Se congelaron protocolo, inputs, binarios, fuentes y recibos antes del piloto; la fuente ejecutada se archivó separada de cambios posteriores del evaluador.

| Piloto conocido, 8 preguntas / 16 operaciones | Scoped V1 | Scoped Anchor V1 |
| --- | ---: | ---: |
| Coste reportado USD | 0,000677670 | 0,000641172 |
| Mediana del runner, ms | 366,98 | 348,09 |
| p95 del runner, ms | 1154,93 | 385,89 |
| Degradaciones | 0 | 0 |

Mismas selecciones útiles, cero IDs no permitidos, cero pérdidas respecto al control contemporáneo y al Scoped V1 histórico. Coste −5,39 %. Pasa la puerta del piloto, que solo permite ampliar; no acredita mejora semántica ni generalización. Las colas de ocho operaciones son descriptivas, no una garantía de latencia. El tiempo es del runner de recall, no de una respuesta completa de Codex. Coste nuevo de esta primera etapa: USD 0,001318842, recibos completos, sin coste de programación/revisión de Codex.

[Protocolo previo](scoped-anchor-v1-protocol-2026-10-19.md), [piloto y auditoría](../../outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19/report.json).

La ampliación V5 completa pasó calidad, recursos y preservación adicional. Sigue siendo desarrollo conocido, no validación nueva.

| V5, 64 pares / 128 operaciones | Decisions/base | Scoped Anchor V1 |
| --- | ---: | ---: |
| Apoyo: selección permitida sin ID ajeno | 16/16 | 16/16 |
| Refutación: selección permitida sin ID ajeno | 15/16 | 16/16 |
| Parcial: selección permitida sin ID ajeno | 15/16 | 16/16 |
| Ausencia: abstención correcta | 15/16 | 16/16 |
| Coste USD | 0,003480200 | 0,005112450 |
| Mediana del runner, ms | 553,38 | 337,99 |
| p95 del runner, ms | 920,00 | 506,54 |

Ratio coste 1,469×, p50 0,611× y p95 0,551×; puertas ≤1,5×. Dos abstenciones útiles resueltas, nueve casos con evidencia adicional y una selección no permitida de la base eliminada. Cero pérdida de IDs útiles respecto a base contemporánea y Scoped V1 histórico. No significa recuperar todo el gold ni cubrir todas las partes: V5 carece de la anotación por parte de V6. Coste emparejado USD 0,008592650 con recibos completos.

La traza real muestra que el modelo volvió a marcar como corrección el recuerdo de Neblina ante Luciérnaga (06803/06807). El testigo literal eligió Luciérnaga y lo excluyó localmente, conservando la corrección de autoría del recuerdo permitido. Es una explicación verificable de esa decisión; no una demostración causal general ni prueba de identidad semántica. En 63/64 consultas hubo nombre; una conservó el comportamiento sin testigo. [Diagnóstico posterior sobre trazas](../../outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19/paired-v5/anchor-use-diagnostic.json).

[Comparación completa](../../outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19/paired-v5/report.json), [preservación adicional](../../outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19/paired-v5/preservation.json). V4 conocida no pasa calidad; no se abre V2, estabilidad ni V6. La variante sigue experimental y el objetivo activo.

| V4, 64 pares / 128 operaciones | Decisions/base | Scoped Anchor V1 |
| --- | ---: | ---: |
| Apoyo: selección permitida sin ID ajeno | 16/16 | 15/16 |
| Refutación: selección permitida sin ID ajeno | 14/17 | 17/17 |
| Parcial: selección permitida sin ID ajeno | 13/16 | 12/16 |
| Ausencia: abstención correcta | 13/15 | 14/15 |
| Coste USD | 0,004500400 | 0,005309178 |

Cinco ganancias de utilidad y tres casos con evidencia adicional no compensan dos selecciones no permitidas y tres pérdidas de evidencia útil de la base. Recursos sí pasan: coste 1,180×, p50 0,817× y p95 0,646×. Cero degradaciones, recibos completos. Coste de esta comparación USD 0,009809578. [Informe V4](../../outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19/paired-v4/report.json).

Causas visibles: no registrar un domicilio no excluye que exista; un borrado puntual no establece una política de M4; el modelo rechaza cantidad parcial/estado pendiente, cantidades ilegibles y duración mínima observada. Los tres recuerdos perdidos estaban dentro del prefijo y pasaban presencia literal: el fallo visible está en los juicios semánticos, no en el recorte o veto literal. Se conserva gold original y las cinco trazas; no se recalculan como aciertos. [Diagnóstico de fallos](../../outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19/paired-v4/failure-analysis.json).

Siguiente experimento propuesto: distinguir explícitamente dato completo, parte/límite/estado de medición y refutación de una afirmación realmente contenida en la pregunta; exigir que una instancia aislada no se transforme en una regla general. Debe medirse antes de implementar una mejora aprobada, conservar el testigo pequeño y respetar el presupuesto. No relanzar esta variante buscando una ejecución favorable. Alias y estabilidad siguen pendientes, V6 sin llamadas. No se atribuye el pase V5 a la reducción de tokens o al testigo por sí solos.

Esta tanda: **272 operaciones HTTP, dos campañas completas y un piloto; USD 0,019721070**, sin coste de Codex de programación/revisión. 194 pruebas workspace pasan, formato y build offline correctos. La base temporal propia se retiró al cerrar la tanda; no se inició servidor ni se tocó el dashboard. Acumulado del objetivo: 4858 operaciones, 23 campañas, USD 0,519314328 conocidos y tres operaciones históricas con importe desconocido. Todas las nuevas tienen recibo. Ninguna promoción.
