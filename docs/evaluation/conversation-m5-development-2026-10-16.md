# M5: métricas de desarrollo para recuerdos de conversación

> Corrección de cierre, 2026-10-17: se conservan el protocolo, los datos y las puntuaciones históricas. Las selecciones discordantes con una etiqueta negativa no prueban una afirmación falsa; algunas preservan evidencia que refuta la pregunta. Las conclusiones de mejora de verdad o superioridad derivadas solo de esa abstención quedan retiradas. Véase el [cierre de M5](m5-closeout-2026-10-17.md).

Estado: sondeo de **desarrollo**, ejecutado el 2026-10-16. No modifica la ruta de producto, el README ni el resultado final 018–020. El [fixture](../../tests/fixtures/conversation-m5-v1/README.md) contiene cinco conversaciones ficticias nuevas, con etiquetas deterministas aún sin revisión humana independiente. Los [resultados por caso](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/) permiten inspeccionar búsquedas, diarios y reportes.

## Línea base: 100 preguntas

Se cargaron 150 recuerdos en una SQLite temporal nueva. Cada conversación tiene quince hechos positivos y quince distractores; las preguntas fueron 75 positivas y 25 negativas. La búsqueda usó `fts5_keywords` con máximo veinte candidatos y Decisions fue el clasificador único. El modelo servido informado por OpenRouter fue `openai/gpt-6-luna-decisions-20261006` en las cien llamadas. No hubo degradación ni IDs prohibidos.

| Métrica | Resultado observado |
| --- | ---: |
| Recuerdo esperado entre candidatos | **74/75** (98,67 %) |
| Recuerdo esperado primero en FTS5 | **64/75** (85,33 %) |
| Semilla positiva con ID correcto, Decisions | **70/75** (93,33 %) |
| Selección correcta cuando el recuerdo estaba entre candidatos | **70/74** (94,59 %) |
| Negativos con semilla vacía | **25/25** |
| Negativos con lista de candidatos vacía | **0/25** |
| Búsqueda local p50 / p95 / p99 | 1,12 / 1,31 / 1,55 ms |
| Consulta → semilla p50 / p95 | 373 / 1.513 ms |
| Tokens de entrada informados, p50 | 760 |
| Coste informado de las cien llamadas | US$0,0075499 |

Los cinco fallos positivos se concentran en dos categorías: cuatro abstenciones en `response_style` (1/5 correctos) con el recuerdo esperado en puestos 3–4, y una elección incorrecta en `changed_assumption` (4/5 correctos) cuando FTS5 no incluyó el recuerdo esperado. `temporary_constraint`, `open_question`, `tool_result` y las demás categorías positivas fueron 5/5 en este conjunto pequeño. La búsqueda local no mide acierto de semilla; el 0/25 de listas negativas vacías no es un falso recuerdo porque Decisions se abstuvo en los 25 casos.

Los tiempos de búsqueda vienen de `search-benchmark` con sesión Rust caliente; excluyen carga inicial de SQLite. Los tiempos de consulta a semilla vienen del diario de `campaign` e incluyen búsqueda, proveedor y composición en el servicio; excluyen arranque de proceso y entrega a Codex. El coste es el informado por OpenRouter, no una factura auditada. La base temporal y el binario quedaron identificados por hashes en los manifiestos de salida.

## Sondeo posterior: seis reformulaciones

Después de observar la línea base se cambiaron solo las palabras de cinco preguntas de estilo de respuesta y una de supuesto abandonado, conservando hechos y ámbito. La búsqueda y el clasificador se ejecutaron otra vez; **la lista y el orden de candidatos podían cambiar**, por lo que este sondeo no aísla la causa del clasificador.

| Métrica del sondeo | Resultado |
| --- | ---: |
| Esperado entre candidatos | 5/6 |
| Semilla correcta | 5/6 |
| Estilo de respuesta: semilla correcta | 4/5, frente a 1/5 con la redacción inicial |
| Supuesto abandonado: semilla correcta | 1/1, frente a 0/1 inicialmente |
| Coste informado de seis llamadas | US$0,0004542 |

La reformulación del supuesto llevó el recuerdo ausente al primer puesto y lo seleccionó. En estilo, una reformulación sacó el recuerdo esperado del top 20 y produjo una selección incorrecta, aunque cuatro mejoraron. Esta tensión entre búsqueda y clasificación impide adoptar la reformulación como política.

## Control de distractores con las cuatro preguntas originales

Para las cuatro abstenciones de estilo, se conservaron las **preguntas originales** y las memorias originales en bases temporales nuevas. Primero se dejó, por conversación, el esperado y los dos primeros distractores de la búsqueda base; luego solo el esperado. En la condición de tres candidatos el esperado quedó tercero en las cuatro listas; en la de uno, primero. Cada condición se llamó una sola vez por pregunta, sin negativos; la poda artificial de memoria no es una política propuesta para el producto.

| Lista ofrecida a Decisions | Correctas | Abstenciones | Coste informado |
| --- | ---: | ---: | ---: |
| Original, hasta veinte candidatos | 0/4 | 4/4 | Incluido en la línea base |
| Esperado + dos distractores | 0/4 | 4/4 | US$0,0001070 |
| Solo el esperado | 3/4 | 1/4 | US$0,0000857 |

Recortar a tres no bastó; quitar todos los distractores cambió tres respuestas, mientras una abstención persistió incluso con la memoria correcta como única opción. Esto es compatible con interferencia entre candidatos y con sensibilidad a la formulación, pero una sola pasada no permite asignar una proporción causal a cada factor. Los [casos, subconjuntos de memoria, búsquedas y diarios](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/) conservan este control.

### Repetición emparejada de preguntas y negativos

Se repitieron **tres veces** las mismas cuatro preguntas positivas originales (conversaciones 022–025) y una pregunta negativa equivalente por conversación, en cada una de las tres bases anteriores. Las listas de búsqueda tuvieron exactamente 20, 3 y 1 candidatos por consulta positiva; el esperado estuvo en puestos 3–4, 3 y 1, respectivamente. No se cambió el prompt ni la pregunta dentro de cada pareja. Cada condición hizo 24 llamadas; las 72 respuestas fueron `complete` sin degradación.

| Candidatos por conversación | Positivos correctos, 12 llamadas | Abstenciones positivas | Negativos vacíos, 12 llamadas | Coste informado |
| --- | ---: | ---: | ---: | ---: |
| Hasta 20 | 0/12 | 12/12 | 12/12 | US$0,0018123 |
| Tres, esperado tercero | 0/12 | 12/12 | 12/12 | US$0,0006456 |
| Uno, el esperado | 9/12 | 3/12 | 12/12 | US$0,0005178 |

Las conversaciones 022, 023 y 025 pasaron de tres abstenciones a tres aciertos cada una al retirar los dos distractores restantes. La 024 se abstuvo en las tres repeticiones incluso con el recuerdo correcto como único candidato. La diferencia es consistente **en estas repeticiones**, pero solo hay cuatro preguntas positivas distintas y cuatro negativas distintas: las fracciones de doce no estiman una tasa poblacional ni comprueban seguridad de una política de candidato único. Tampoco separan la posición del recuerdo (tercero frente a primero) de la presencia de distractores; esa es la siguiente variable por aislar si continúa el diagnóstico.

### Control de posición y búsqueda natural

En una cuarta condición diagnóstica se mantuvieron exactamente los tres IDs y resúmenes por conversación del control reducido, pero se añadieron términos de la consulta al **texto original** del recuerdo esperado en una base temporal. FTS5 lo puso primero; Decisions solo recibió los mismos tres resúmenes en distinto orden. Con las mismas cuatro preguntas positivas y cuatro negativas repetidas tres veces, el resultado fue **3/12 positivos correctos**, frente a **0/12** con el esperado tercero; los **12/12 negativos** siguieron vacíos. Los tres aciertos corresponden a una sola conversación (023). Este cambio artificial del texto no representa una política de producto: la posición influye, pero no resuelve las otras tres preguntas. El [fixture alterado](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/rank-first-memories.json), las búsquedas y el diario conservan el control. Coste informado: US$0,0006456.

Para el quinto fallo, `changed_assumption` en la conversación 023, una búsqueda local con modo `natural` colocó el esperado en el puesto 7 en vez de dejarlo fuera de los primeros veinte. Al ejecutar esa búsqueda local sobre las 100 preguntas, la cobertura positiva fue **75/75**, frente a **74/75** con `keywords`, y el primer puesto **65/75**, frente a **64/75**. El contraste completo de semilla y negativos aparece a continuación. Los resultados locales están en `natural-search-results.jsonl` y `natural-retrieval-report.json`.

### Contraste completo de `natural`, instrucción y etiquetas

Se ejecutaron las cien consultas originales con `query_mode: natural`, la misma SQLite y la instrucción vigente de Decisions. La cobertura local subió a 75/75, pero la semilla correcta permaneció en **70/75** y los negativos vacíos bajaron a **24/25**. `changed_assumption` pasó de 4/5 a 5/5 y `response_style` de 1/5 a 2/5; `open_question` y `rejected_option` bajaron de 5/5 a 4/5, y una pregunta negativa sobre fecha de despliegue seleccionó erróneamente la fecha de revisión. Coste informado: US$0,0076026; p95 de consulta a semilla: 740 ms, frente a 1.513 ms en la línea base. La variación temporal entre campañas remotas no justifica atribuir esa diferencia al modo de búsqueda. **No se cambia el modo predeterminado.** Los [casos con el modo explícito](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/natural-cases.jsonl), diario y reporte conservan la comparación.

Una instrucción experimental de Decisions, [guardada literalmente](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/prompt-v2-instructions.txt), se compiló solo para repetir las cuatro preguntas de estilo y cuatro negativas tres veces con las listas reducidas de tres candidatos. Dio **0/12 positivos y 12/12 negativos vacíos**, igual que la instrucción vigente; se restauró el código original. Coste informado: US$0,0007656. Jev, con la instrucción vigente y esos mismos casos, dio también **0/12 y 12/12** (US$0,00050526). Estas repeticiones son cuatro preguntas positivas y cuatro negativas distintas, no 24 sujetos independientes.

La auditoría del diseño encontró una **ambigüedad plausible**: «¿Qué formato o estilo de respuesta se pidió?» coexiste en cada conversación con un recuerdo sobre la estructura de la respuesta y otro sobre el idioma pedido. La etiqueta original solo acepta el primero; no se reetiqueta retroactivamente ni se afirma que la ambigüedad explique por sí sola las abstenciones. Como diagnóstico separado se formuló «¿Qué formato o estructura, aparte del idioma, se pidió para responder…?» para las cinco conversaciones, con cinco controles negativos de tipografía ausente. Decisions obtuvo **5/5 positivos correctos y 5/5 negativos vacíos** (US$0,0007591). El esperado estuvo en puestos 2, 1, 1, 1 y 1; por tanto, el ensayo mezcla precisión de la pregunta y cambio de orden FTS5. Son [diez casos sintéticos nuevos](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/clarified-style-cases.jsonl), aún sin etiquetas revisadas por otra persona, y **no** una estimación de desempeño con usuarios.

Para separar mejor redacción y posición, se completó una matriz 2×2 con las **cuatro preguntas que habían fallado**, repetidas tres veces. En los positivos, los tres IDs, sus resúmenes y su orden fueron idénticos entre preguntas original y aclarada dentro de cada posición. Se forzó el puesto del esperado mediante texto original adicional en memorias temporales; ese texto no llega al clasificador, que solo ve los resúmenes. Las preguntas negativas conservaron los mismos tres IDs, aunque su orden pudo variar. No se usó esta alteración en la ruta de producto.

La notación «2×2» se refiere a **dos factores** (redacción original/aclarada y puesto primero/tercero), no a dos recuerdos. Cada celda de esta matriz entregó tres candidatos al clasificador. La línea base de este banco entregó veinte por pregunta; el banco v5 anterior construía cuatro candidatos por positivo. Son diseños de evaluación distintos, no un cambio del límite de producto a dos o tres recuerdos.

| Pregunta y puesto del esperado entre tres candidatos | Positivos correctos | Negativos vacíos |
| --- | ---: | ---: |
| Original, tercero | 0/12 | 12/12 |
| Original, primero | 3/12 | 12/12 |
| Aclarada, tercero | 0/12 | 12/12 |
| Aclarada, primero | 12/12 | 12/12 |

La pregunta aclarada solo funcionó de forma estable **cuando el esperado estaba primero** en estos cuatro casos. Las repeticiones no convierten cuatro preguntas en doce sujetos; además, la etiqueta original puede ser ambigua y el orden se preparó artificialmente. Esto identifica una interacción para probar en sujetos nuevos, no una regla para favorecer siempre el primer candidato. Los [casos](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/clarified-rank3-cases.jsonl), [memorias de control](../../outputs/iromem/evaluation/conversation-m5-development-2026-10-16/clarified-rank3-memories.json), búsquedas y diarios permiten reproducir la comparación. Cada una de las dos condiciones nuevas costó US$0,0006516 informados.

Hubo una llamada piloto válida (US$0,0000763 informado) y una prueba inicial de red restringida que terminó en `transport_error` sin coste informado; no entran en las tablas anteriores. El coste informado conjunto de las **417 llamadas válidas** de desarrollo, incluidos ambos proveedores y los controles, fue **US$0,0228302** (redondeado del total de reportes).

## Límite y siguiente paso

Estas cinco conversaciones usan categorías y redacciones controladas por el mismo diseño que fija sus etiquetas. Son útiles para localizar fallos, pero ni 70/75 ni 25/25 predicen comportamiento humano o demuestran que el 95 % se sostenga en sujetos nuevos. La pregunta de estilo requiere auditoría independiente antes de usar su etiqueta para elegir una política: quedó preparado un [paquete ciego de cinco preguntas](conversation-m5-style-audit-blind.md), todavía sin respuestas humanas. Una [revisión adversarial entre agentes](conversation-m5-adversarial-agent-review-2026-10-16.md) identificó dos respuestas explícitas plausibles por pregunta, pero no sustituye esa auditoría. No se ha validado una mejora de producto y no se abrirán las particiones de calibración o prueba reservada para elegir otra pregunta o instrucción. Conforme al [plan M5](../../ROADMAP.md), siguen la revisión humana de etiquetas, una intervención de selección con candidatos congelados y controles negativos, y conversaciones nuevas separadas por sujeto.
