# M5: comprobación secundaria y contraste de proveedor en conversaciones nuevas

> Corrección de cierre, 2026-10-17: se conservan el protocolo, los datos y las puntuaciones históricas. Las selecciones discordantes con una etiqueta negativa no prueban una afirmación falsa; algunas preservan evidencia que refuta la pregunta. Las conclusiones de mejora de verdad o superioridad derivadas solo de esa abstención quedan retiradas. Véase el [cierre de M5](m5-closeout-2026-10-17.md).

Estado: diagnóstico de desarrollo del 2026-10-17. El [protocolo](conversation-m5-support-gate-v1-protocol.md) se fijó antes de ejecutar `support_gate_v1`. El [banco nuevo](../../tests/fixtures/conversation-m5-v3-development/README.md) contiene tres conversaciones ficticias (`031`–`033`), 36 recuerdos y 36 preguntas agente-sintéticas sin revisión humana ni adversarial independiente: 24 positivas y 12 negativas por resultado no establecido. Los [diarios y reportes](../../outputs/iromem/evaluation/conversation-m5-support-gate-v1-2026-10-17/) registran hashes, respuestas, costes y una comparación por caso.

La validación estructural y de seis hashes pasó. La búsqueda `keywords` encontró 24/24 positivos en top 1, top 5 y top 20. Los 12 negativos recuperaron al menos un candidato: no se resolvieron por lista vacía. Entre los diarios Decisions y Jev, **36/36 listas de candidatos fueron idénticas** en contenido y orden. Los tres casos del piloto de gate también recibieron esas mismas listas. Las etiquetas se fijaron antes de las campañas; los ejemplos son relativamente literales y no incluyen revisiones sucesivas, permisos ni condiciones.

Todos los casos completos informaron `openai/gpt-6-luna-decisions-20261006` para Decisions y `typesafe/jev-1.13-20260917` para Jev.

| Medición | Decisions base | Jev, contraste exploratorio |
| --- | ---: | ---: |
| Casos correctos | 28/36 | 36/36 |
| Positivos correctos | 24/24 | 24/24 |
| Negativos vacíos | 4/12 | 12/12 |
| Degradados | 0 | 0 |
| Tiempo total p50 / p95 | 365,4 / 878,7 ms | 371,7 / 512,7 ms |
| Coste informado, 36 casos | US$0,0015369 | US$0,001100232 |

Jev se abstuvo en los ocho negativos en que Decisions eligió un recuerdo relacionado pero insuficiente, incluidos contrato no adjudicado, permiso pendiente, certificado no emitido, licencia sin firma y resultado preliminar. Los percentiles proceden de dos campañas de red distintas y no demuestran una mejora causal de latencia. El coste informado corresponde a estas llamadas, no a una predicción universal. La comparación Jev se decidió después de ver el fallo del piloto de gate y por ello es exploratoria, no el contraste predefinido de dos variantes.

`support_gate_v1` conservó la primera elección Decisions y consultó al mismo modelo una segunda vez con solo el candidato elegido. En el piloto de tres casos, acertó 2/2 positivos, pero **0/1 negativos**: aprobó el recuerdo que decía expresamente que no se había firmado la licencia de streaming. Es el mismo falso recuerdo de la ruta base. La mediana de entrada de esos tres casos fue 694 tokens y el coste informado total US$0,000209. El criterio fijado exigía 12/12 negativos vacíos; ese primer fallo lo hizo imposible, así que no se gastaron llamadas en los otros 33 casos. El [resumen del piloto](../../outputs/iromem/evaluation/conversation-m5-support-gate-v1-2026-10-17/gate-pilot-summary.json) no se debe presentar como resultado de 36 casos.

**Decisión:** conservar Decisions `baseline` como ruta predeterminada. `support_gate_v1` queda disponible solo como experimento reproducible y descartado para calibración. Jev merece una comparación de proveedor predefinida en conversaciones nuevas con etiquetas revisadas independientemente; **36/36 en este banco no justifica cambiar el producto**. En el banco v2, Jev todavía eligió un proveedor que el recuerdo decía que no había sido adjudicado. Ninguna de las dos muestras prueba el comportamiento de usuarios reales. Calibración, reserva y muestra natural humana permanecen pendientes.

Verificación: `cargo test --offline --workspace` pasó con 29 pruebas unitarias; la validación estructural del banco y la búsqueda local pasaron. Las campañas usaron una SQLite temporal, no la base del usuario. No se hizo commit, push ni despliegue.
