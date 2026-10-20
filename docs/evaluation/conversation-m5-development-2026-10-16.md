# M5: métricas de desarrollo para recuerdos de conversación

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

Hubo una llamada piloto válida (US$0,0000763 informado) y una prueba inicial de red restringida que terminó en `transport_error` sin coste informado; no entran en las tablas anteriores. El coste informado conjunto de las **187 llamadas válidas** de línea base, reformulación, controles, repeticiones y piloto fue **US$0,0112488**.

## Límite y siguiente paso

Estas cinco conversaciones usan categorías y redacciones controladas por el mismo diseño que fija sus etiquetas. Son útiles para localizar fallos, pero ni 70/75 ni 25/25 predicen comportamiento humano o demuestran que el 95 % se sostenga en sujetos nuevos. No se ha validado una mejora y no se abrirán las particiones de calibración o prueba reservada para elegir otra pregunta o instrucción. Conforme al [plan M5](../../ROADMAP.md), siguen la revisión humana independiente de etiquetas, aislar posición frente a distractores en desarrollo y preparar conversaciones nuevas separadas por sujeto.
