# Revisión ciega y recuperación local: conversation-evidence-v7

## Resultado

Dos revisores independientes recibieron copias congeladas que contenían únicamente `source.json` y `blind-questions.json`. Cada uno clasificó las 64 preguntas, identificó el referente y citó mensajes fuente; ninguno vio el corpus, la clave ni las etiquetas de autoría. Después de congelar ambos archivos de revisión se abrió la clave.

Las dos revisiones coinciden en **64/64 relaciones y listas de mensajes fuente**. Las **56 citas con evidencia** apuntan a mensajes existentes y las dos revisiones identifican el ID estable correcto en los 64 casos. La comparación posterior contra la clave confirma **64/64 relaciones y referencias de evidencia**. Los casos parciales se trataron como parciales, no como respuestas completas; las ocho preguntas de género musical se marcaron como no relacionadas.

| Relación de la clave | Casos | Evidencia recuperada en top 20 | Evidencia en posición 1 |
| --- | ---: | ---: | ---: |
| Respuesta directa | 16 | 16 | 16 |
| Refutación de premisa | 16 | 16 | 16 |
| Parcial / dato pendiente | 16 | 16 | 8 |
| No medido explícitamente | 8 | 8 | 8 |
| No relacionado | 8 | 0 esperada | 0 |

## Diagnóstico de FTS5

Se cargaron los 128 recuerdos en una base SQLite temporal aislada y se ejecutó la búsqueda Rust `search-batch` en modo natural, límite 20, ámbito de conversación y sin filtro opcional de entidad. La consulta fue idéntica a la versión ciega. No se llamó a un clasificador ni a un proveedor.

FTS5 colocó evidencia esperada en la lista para **56/56 preguntas con evidencia** (40 respuestas completas y ocho refutaciones más 16 parciales); **48/56** quedaron en primer lugar. La búsqueda devolvió **14 candidatos para cada una de las ocho preguntas no relacionadas** (112 candidatos fuera del dato solicitado). Esos recuerdos pertenecen al mismo sujeto y ámbito, pero hablan de otros atributos; son candidatos crudos, no selecciones finales ni falsos positivos del clasificador. Esta tanda muestra que el problema de rechazo sigue siendo real: la recuperación encuentra evidencia pertinente, pero no resuelve por sí sola la relevancia respecto del dato solicitado.

La latencia local de `search` fue **0,405 ms de mediana** y **0,598 ms en p95 nearest-rank**. No es latencia de `recall` ni incluye clasificación o composición. Coste API de esta tanda: **USD 0**; coste Codex de la revisión no medido.

## Integridad, límites y decisión

Las entradas ciegas conservan SHA-256 `source.json` `7132dfc1f33809a86abc586d2cfdb72c686eda1ced8f692b15a21c3b0222a006` y `blind-questions.json` `09c777e71cb6a03f773da0cbf826722dcaaa26b5f08b11f2db3a3fbd36f68ffb`. Los juicios independientes se conservan en [revisión A](../../tests/fixtures/conversation-evidence-v7/blind-reviewer-a/review.json) (SHA-256 `19000777f6ac90add546dc79c68e48fc32fd8a253440fc061740fa9600efe916`) y [revisión B](../../tests/fixtures/conversation-evidence-v7/blind-reviewer-b/review.json) (SHA-256 `54c0c66aed4fcf6685d2734c2c4fc78b9e5f37b37cb0555dc398d9362591b989`). La [captura reproducible de búsqueda](assets/conversation-evidence-v7-local-retrieval/) conserva las solicitudes, respuestas y resumen de métricas.

Aunque hay 64 preguntas, son **ocho patrones semánticos repetidos en ocho ámbitos**, con entidades ficticias e IDs explícitos. El 100% de concordancia acredita consistencia de estas etiquetas sencillas, no independencia estadística ni generalización a conversaciones naturales. Este banco es un control sintético de desarrollo, no una prueba de calidad del clasificador.

**Desviación del protocolo pre-registrado:** se preparó y revisó esta tanda V7 antes de superar la puerta anterior de tres smokes válidos y 24 operaciones completas. Por eso queda congelada como material exploratorio de desarrollo y **no cuenta como la cohorte confirmatoria de 64** ni puede usarse para decidir adopción. El smoke r30 terminó en HTTP 429 sin recibo ni modelo servido. No se reporta comparación contra la configuración actual, mejora del selector o coste remoto; la configuración vigente permanece intacta. Un futuro contraste confirmatorio requiere una cohorte nueva, distinta de V7, después de pasar todas las puertas.
