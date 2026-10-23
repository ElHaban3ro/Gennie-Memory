# Resultado emparejado: 20 frente a 10 candidatos

El 2026-10-17 se completó la comparación prevista en el [protocolo previo](candidate-budget-paired-protocol-2026-10-17.md). **Se mantiene el límite predeterminado de 20 candidatos.** Con 10 se redujeron los tokens de entrada y el coste reportado un 10,04 %, pero se perdió una refutación útil seleccionada con 20. Esto incumple el criterio previo para seguir promoviendo la variante; no cambia el cierre limitado de M5.

Se ejecutaron 136 preguntas sintéticas conocidas de desarrollo, cada una con ambos límites: 272 operaciones, 268 respuestas del proveedor y cuatro omisiones sin candidatos (`skipped_empty`). Decisions con política `baseline` sirvió `openai/gpt-6-luna-decisions-20261006` en las 268 llamadas. Los pares quedaron adyacentes y alternaron el orden. No hubo degradaciones ni reintentos para rescatar resultados.

| Selección observada | 20 candidatos | 10 candidatos |
| --- | ---: | ---: |
| Positivos con ID esperado, v2 | 75/75 | 75/75 |
| Positivos con ID esperado, v3 | 24/24 | 24/24 |
| Refutaciones seleccionadas, v2 | 1 | 1 |
| Refutaciones seleccionadas, v3 | 8 | 7 |
| Contextos parciales seleccionados, v2 | 2 | 2 |
| Semillas vacías, v2 | 22 | 22 |
| Semillas vacías, v3 | 4 | 5 |
| Tokens de entrada reportados, total | 48.140 | 43.305 |
| Coste reportado, USD | 0,0048140 | 0,0043305 |

Los bancos contienen 13 preguntas con evidencia de refutación y nueve con contexto parcial según la anotación retrospectiva: la selección fue 9/13 frente a 8/13 y 2/9 en ambas configuraciones, respectivamente. La cobertura del buscador conservó esas 22 evidencias con ambos límites. No se observó una selección nueva no revisada o irrelevante. Mantener 99/99 positivos no basta para aprobar el recorte.

La única diferencia de selección ocurrió en `cm5v3q_3d856662c0c47ec4ec42`, ante «¿Qué robot universitario ganó la final?». Con 20 se eligió `cm5v3_56230a85778493db265f`: «Dos robots universitarios llegaron a la final; el jurado aún no declaró un ganador». Con 10 la semilla quedó vacía, aunque ese mismo recuerdo estaba entre los candidatos. Es pérdida de información correctiva pertinente, no una reducción demostrada de falsedades. Un solo par no demuestra que el límite cause siempre esa pérdida; sí basta para bloquear la promoción conforme al filtro conservador fijado antes de ejecutar.

| Tiempo del runner, ms | 20: p50 / p95 / p99 | 10: p50 / p95 / p99 |
| --- | ---: | ---: |
| v2 | 353,52 / 584,86 / 909,79 | 353,90 / 584,52 / 689,39 |
| v3 | 366,44 / 527,86 / 539,68 | 347,26 / 522,05 / 600,50 |

La mediana de la diferencia por par (10 menos 20) fue +1,52 ms en v2 y −5,99 ms en v3. Los resultados temporales son mixtos, con red y una observación por pregunta y configuración; no prueban una aceleración general ni latencia de integración con Codex. El ahorro de tokens incluye toda la entrada reportada, por eso difiere del 21,2 % de bytes de resúmenes observado en el cribado local. El gasto total reportado de esta comparación fue USD 0,0091445, con 134 recibos por configuración y dos omisiones sin llamada por configuración; no equivale a una auditoría de factura.

La auditoría verificó 1.524 apariciones de candidatos y 219 recuerdos de semillas contra los corpus: texto, condiciones, fuente declarada, ámbito, revisión, estado y vigencia, además de hashes y eventos completos. Las listas con límite 10 fueron exactamente prefijos de las listas con límite 20. Las etiquetas originales y corpus permanecen intactos. Esto verifica fidelidad a los fixtures; no verifica fuentes externas ni la verdad de una respuesta final generada. Son datos sintéticos conocidos y etiquetas retrospectivas sin revisión humana independiente, sin abrir una reserva.

Se incorporaron al evaluador Rust la preparación emparejada y el reporte reproducible; sus 19 pruebas pasaron, incluida la comprobación de que perder una refutación bloquea el cambio. El producto mantiene su configuración. El siguiente trabajo útil es fijar una rúbrica que distinga respuesta, refutación, contexto parcial y ausencia, y validarla con datos nuevos antes de otra promoción. Ampliar por sí solo las plantillas conocidas o repetir el par fallido no demostraría generalización.

Evidencia reproducible: [suite](../../outputs/iromem/evaluation/candidate-budget-paired-2026-10-17/suite.json), [preparación y hashes de originales](../../outputs/iromem/evaluation/candidate-budget-paired-2026-10-17/runs/preparation.json), [reporte de pares y consumo](../../outputs/iromem/evaluation/candidate-budget-paired-2026-10-17/runs/report.json), [auditoría de semillas](../../outputs/iromem/evaluation/candidate-budget-paired-2026-10-17/runs/audit.json), [comprobación independiente local](../../outputs/iromem/evaluation/candidate-budget-paired-2026-10-17/runs/verification.json) y [manifiesto de integridad](../../outputs/iromem/evaluation/candidate-budget-paired-2026-10-17/runs/manifest.json).
