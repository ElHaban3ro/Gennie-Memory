# M4: cobertura, atribución y operación local acotada

Fecha: 2026-10-24. Esta entrega añade una evaluación semántica prospectiva pequeña, intervenciones reproducibles de componentes y una observación local de concurrencia con mantenimiento. Los bancos grandes y resultados históricos siguen identificados como históricos. No demuestra uso autónomo general, continuidad de meses ni integración con Codex.

## Banco y revisión

El [banco de factores](../../../tests/fixtures/m4-factor-v1/manifest.json) tiene 24 fuentes y 24 preguntas, con ocho casos por desarrollo, calibración y final. Los nombres, fuentes y ámbitos son disjuntos entre particiones; **las ocho familias de plantillas se repiten**, por lo que esto no mide generalización a familias nuevas. Cada ámbito tiene una memoria: se aísla el juicio sobre referente, dato y corrección; no se mide discriminación entre ocho distractores.

Las [etiquetas](../../../tests/fixtures/m4-factor-v1/labels.json) fijan factores, relación de evidencia, respuesta permitida y error prohibido antes de inferencia. Hay apoyo directo, refutación de premisa, respuesta parcial, sujeto ajeno, condición aplicable, condición sin establecer, traducción inglés/español y ausencia que no implica negación. Las respuestas permitidas son etiquetas, no respuestas finales generadas y verificadas.

Un segundo agente juzgó las 24 parejas sin abrir gold ni salidas: [revisión y adenda](independent-label-review.json). Coincidió con las etiquetas; se conserva una [adjudicación previa a inferencia](label-adjudication.json) que elimina la ambigüedad lógica de «solo si» en tres fuentes. Fernando confirmó las [ocho etiquetas de la muestra](human-review-sample.md) y la corrección de la fuente condicional: [registro humano](human-review.json). Esa confirmación no se extiende a las otras 16 preguntas ni aprueba a un proveedor.

La [matriz determinista anterior](../product-coverage-v1/README.md) conserva controles de diacríticos, seis idiomas/Unicode, original largo con evidencia al final, homónimos, contradicciones condicionadas, duplicados, revisión/tiempo, intersección de filtros, corpus vacío y fallos simulados de proveedor. Los dobles verifican contratos de degradación; no se suman como aciertos semánticos ni llamadas reales.

## Comparación semántica y fiabilidad

El coordinador ejecutó el protocolo de la campaña real en `campaign-v2/`, con el banco y perfil congelados, 48 operaciones como máximo entre Decisions y Jev, coste máximo USD 0,03 y plazo total 180 segundos. No hay reintentos ni fallback. El primer intento `campaign-v1/` conserva una llamada Decisions válida de desarrollo y un fallo de preparación local de Jev antes de HTTP; no se presenta como campaña terminada. El nuevo intento conserva sus resultados, recibos y errores en su propio diario, con máximo combinado de 49 HTTP y USD 0,03 para ambos intentos. La campaña se mantiene fuera de pruebas offline obligatorias.

`m4-factor-calibrate` ajusta dos intervalos prefijados por factor: `[0,0.5]` y `(0.5,1]`. La tasa empírica usa `(positivos + 1)/(n + 2)`. Un intervalo sin observaciones impide publicar el perfil; no se rellena ni se modifica el corte usando final. Las probabilidades nativas ausentes no se derivan de los booleanos. `m4-factor-report` reconstruye el calibrador desde los ocho registros de calibración, vincula bytes del perfil, pregunta/política, fuente/revisión, alias solicitado y snapshot servido por separado, e implementación, y comprueba en final los valores publicados por el producto. Desglosa Brier y ECE en cinco intervalos fijos, booleanos y omisiones.

Ocho casos por partición ofrecen soporte mínimo y errores de estimación grandes. Un valor calibrado no es confianza factual ni probabilidad conjunta de utilidad. El informe puede demostrar empeoramiento; no existe promoción automática de política o umbral.

### Resultado de la campaña real

La [campaña v2](campaign-v2/terminal.json) completó las **48 operaciones en 61,75 s**, coste informado USD 0,002298616 y cero degradaciones. Con la llamada del intento inicial: **49 HTTP y USD 0,002361316**. Los seis informes Rust de desarrollo, calibración y final reconstruyen los artefactos y sus valores publicados. [Resumen auditado por ID](campaign-summary.json).

| Proveedor | Alias solicitado | Snapshot servido | Selección desarrollo / calibración / final |
| --- | --- | --- | --- |
| Decisions | `openai/gpt-6-luna-decisions` | `openai/gpt-6-luna-decisions-20261006` | 8/8 · 7/8 · 8/8 |
| Jev | `typesafe/jev-1.13` | `typesafe/jev-1.13-20260917` | 7/8 · 7/8 · 7/8 |

La selección conjunta correcta no implica que todos sus factores lo sean: Decisions tuvo 8/8 decisiones finales correctas, pero solo 7/8 juicios correctos de dato solicitado y 7/8 de corrección. Jev tuvo 6/8 de contexto, 7/8 de dato y 8/8 de corrección. No hubo probabilidades ausentes en las 16 operaciones finales.

| Proveedor y factor | Brier nativo → empírico | ECE nativo → empírico |
| --- | ---: | ---: |
| Decisions: contexto | 0,01634 → 0,10938 | 0,10375 → 0,31250 |
| Decisions: dato | 0,08579 → 0,11959 | 0,22375 → 0,12857 |
| Decisions: corrección | 0,06390 → 0,07870 | 0,15250 → 0,12500 |
| Jev: contexto | 0,11359 → 0,13889 | 0,20375 → 0,08333 |
| Jev: dato | 0,06145 → 0,11959 | 0,12250 → 0,12857 |
| Jev: corrección | 0,00326 → 0,02469 | 0,02625 → 0,13889 |

En ambas métricas menor es mejor; cada fila tiene ocho juicios de final. **Brier empeoró en los tres factores de ambos proveedores. La receta de calibración queda rechazada como mejora para adopción.** Algunas ECE descendieron, pero no compensan ese resultado ni justifican escoger retrospectivamente otra receta. Los artefactos se conservan como experimentales reproducibles; no se recomiendan perfiles calibrados para uso general y no cambian la selección ni los valores predeterminados. No se retocaron intervalos, etiquetas o instrucciones después de abrir final.

Errores de selección: Decisions aceptó evidencia no aplicable en `calibration-06`; Jev omitió evidencia parcial de canal sin hora en `development-03`, `calibration-03` y `final-03`. Los juicios discordantes de factores, incluidos los que no cambiaron selección, permanecen en `campaign-summary.json`: Decisions en `development-06`, `calibration-02`, `calibration-06`, `calibration-08`, `final-02` y `final-06`; Jev en los casos `02`, `03` y `08` de cada partición. No se convirtieron estos errores en aciertos por tener cita literal o JSON válido. Tampoco se generaron respuestas finales para medir su verdad.

## Ablaciones ejecutadas

La [serie actual de filtros](structured-filter-ablation.json) ejecutó las mismas 20 consultas del corpus de producto, en `natural` y `keywords`, con cuatro variantes: todos los filtros, sin entidad, sin tipo y sin ambos. Son **160 búsquedas reales del servicio actual**, sin clasificador. Actor y ámbito se conservan siempre. Los filtros retirados incorporaron 16 apariciones adicionales de candidatos, incluidos homónimos y tipos distintos; no se filtró otra identidad de actor o ámbito. Todos los conjuntos cabían en el límite. El test tardó 0,16 s excluyendo compilación; sus tiempos por consulta son una observación breve, no una estimación estable.

`m4-rank-replay` reconstituye RRF y ejecuta intervenciones sobre los mismos pools congelados históricos. Exige cobertura completa, todos los recuerdos elegibles presentes y un máximo de 20: así no inventa candidatos excluidos por el top-k. Retirar RRF significa reemplazar su suma por el mejor rango individual en la misma unión de candidatos. Los pisos de coseno solo retiran la contribución semántica; conservan la coincidencia textual. No ejecuta un encoder ni compone semillas o respuestas.

| Brazo | Principal: positivos / 1.000 | Principal: negativos vacíos / 600 | Fallo léxico: positivos / 100 | Fallo léxico: negativos vacíos / 100 |
| --- | ---: | ---: | ---: | ---: |
| Sin recuperación | 0 | 600 | 0 | 100 |
| Sin embeddings: FTS | 719 | 200 | 0 | 100 |
| Semántica sola | 524 | 200 | 52 | 0 |
| Sin suma RRF: mejor rango | 719 | 200 | 52 | 0 |
| RRF congelado | 719 | 200 | 52 | 0 |
| Piso coseno 0,7 | 716 | 200 | 49 | 5 |
| Piso coseno 0,8 | 689 | 200 | 11 | 84 |
| Piso coseno 0,9 | 707 | 200 | 0 | 100 |

[1.600 casos de desarrollo principal](rank-replay-main-development.json) y [200 de desarrollo con fallo léxico](rank-replay-natural-gap-development.json). Son concordancias con etiquetas históricas de IDs, no verdad de una respuesta. El efecto de los pisos depende del corpus: una mejor abstención puede eliminar todo positivo. No se escoge un umbral productivo con estos resultados observados. Los vectores Laya siguen congelados; E5 no se integra ni se reactiva para esta entrega.

## Concurrencia y mantenimiento

La [observación instrumentada](concurrency-observation.json) usa cinco procesos CLI con una conexión por proceso: dos escritores, dos lectores y un operador de retirada/restauración y encolado/cancelación de índices. La base es temporal, las fuentes son sintéticas y no hay puerto, modelo ni proveedor. Cada trabajador se mantiene al menos 30 segundos con cadencia prefijada.

Se completaron **960 operaciones**: 240 escrituras, 480 lecturas y 240 de mantenimiento; cero errores después del arreglo. Cada escritor terminó en revisión 120. `integrity_check` devolvió `ok`, sin violaciones de claves foráneas. Un `BEGIN IMMEDIATE` de prueba retuvo el escritor durante unos 300 ms: ambos escritores y mantenimiento ya habían enviado su operación y seguían pendientes, mientras un lector completó. Los eventos conservan intervalos de cada operación.

| Trabajador | Operaciones | p50 / p95 ms |
| --- | ---: | ---: |
| Escritor 0 | 120 | 7,90 / 28,05 |
| Escritor 1 | 120 | 11,13 / 28,25 |
| Lector 2 | 240 | 2,71 / 4,26 |
| Lector 3 | 240 | 2,81 / 4,47 |
| Mantenimiento | 240 | 1,53 / 13,22 |

Hay 29 muestras `ps` de PID, RSS y porcentaje de CPU acumulado durante la vida del proceso, aproximadamente una por segundo. El pico de suma de RSS de los cinco trabajadores fue **60.016 KiB**; no incluye al test harness. `ps %cpu` no es consumo por operación. La base pasó de 323.584 a 1.368.064 bytes. Los picos de latencia de unos 354–359 ms incluyen deliberadamente el bloqueo inicial.

La primera ejecución detectó un fallo real: el intento de elevar una transacción diferida devolvía `database is locked` inmediatamente en vez de esperar. Se minimizó y reprodujo dos veces antes del arreglo; el control `BEGIN EXCLUSIVE` aisló el mecanismo. La corrección productiva y sus pruebas pertenecen a la [entrega recall](../recall-completion-v1/sqlite-contention-diagnosis.md). No se añadieron reintentos a este harness para ocultarlo.

## Reproducción y verificación

Todos los comandos parten de la raíz. Los informes exigen un nombre de salida nuevo; no se sobrescribe evidencia. Los procesos y directorios temporales propios se cierran al terminar.

```sh
cargo test --offline -p gennie-memory-eval m4_
cargo test --offline -p gennie-memory-retrieval --test product_coverage
cargo test --offline -p gennie-memory --test product_coverage_m4_concurrency
GENNIE_M4_OBSERVATION_REPORT=/ruta/nueva/observacion.json cargo test --offline -p gennie-memory --test product_coverage_m4_concurrency -- --ignored --nocapture
GENNIE_M4_FILTER_REPORT=/ruta/nueva/filtros.json cargo test --offline -p gennie-memory-retrieval --test product_coverage every_matrix_query
```

La observación de 30 segundos es explícita e ignorada por defecto; los tres controles mínimos de bloqueo sí son breves y ordinarios. Los tests del evaluador incluyen probabilidades ausentes, pregunta/fuente/política/implementación alteradas, tasas publicadas que no coinciden con el calibrador y conservación de informes existentes. Los dobles internos del evaluador solo verifican aritmética e integridad del reporte; no cuentan como campaña.

[Resultados focales y hashes](verification.json). [Verificación conjunta final y logs completos](joint-verification.json): **1.173 aprobadas** por defecto y **1.189 con embeddings**, cero fallos y 26 ignoradas por suite. Formato y revisión de espacios correctos. El único cambio posterior a las suites fue formato de `tools/evaluation/src/main.rs`, con hashes antes/después registrados, sin cambiar comportamiento. No hay commit, push o despliegue asociado a esta entrega.
