# Recuerdos HappyDB escritos para revisión humana

Estado: 2.500 recuerdos creados por `mem-write`; la revisión humana y cualquier evaluación de recuperación siguen pendientes.

## Corpus y límites

[HappyDB](https://github.com/megagonlabs/HappyDB) contiene relatos breves en primera persona recogidos por crowdsourcing. Las instrucciones pedían tres momentos felices ocurridos en las últimas 24 horas o durante los últimos tres meses. Sirve para revisar recuerdos episódicos y la fidelidad de sus reinterpretaciones; no representa preferencias persistentes ni conversaciones naturales con un asistente.

El repositorio anidado `HappyDB/` está en la revisión `b9e529e1e639db96871b6fa3250bf8664ba1ff85`. No encontramos un archivo de licencia en esa revisión ni una licencia declarada detectable en GitHub. El corpus y sus derivados se conservan localmente; consulta a sus responsables antes de redistribuir relatos o publicar resultados que los contengan.

## Preparación y escritura

`tools/datasets/prepare_happydb.py` lee únicamente `cleaned_hm.csv`; no carga edad, país, género, estado civil ni etiquetas de investigación. De 100.535 filas seleccionó una muestra determinista de 2.500 en 486 ámbitos aislados por contribuyente. El `wid` se transforma mediante SHA-256 sin clave; es seudonimización, no anonimización demostrada.

Tres subagentes produjeron reinterpretaciones para 2.470 relatos (lotes de 824, 823 y 823). Para los otros 30 se conservaron solo las reinterpretaciones generadas previamente: se descartaron sus antiguas preguntas, respuestas y citas. Una comprobación literal detectó que 751 resúmenes coincidían con el relato original o `cleaned_hm`; los tres subagentes reformularon esos casos. La unión final tiene 2.500 `hmid` únicos, exactamente los de la muestra, y cero resúmenes idénticos a esos dos textos. Todas las filas quedan marcadas `needs_human_review: true`; 7 conservan una nota de ambigüedad/contexto. Son propuestas de agentes y no han recibido adjudicación humana: la comprobación literal no verifica equivalencia semántica.

Cada solicitud `mem-write` conserva el `original_text` del corpus, guarda la reinterpretación en `summary`, usa `type: event` y atribuye el origen a `cleaned_hm.csv` fijado al commit clonado, con `hmid` y `reflection_period` en el localizador. No se inventa `occurred_on`: el corpus solo acota el periodo de reflexión, no da una fecha concreta. Las condiciones y fechas de vigencia no especificadas permanecen `null`. `cleaned_hm` queda disponible en el paquete de revisión como referencia editorial, no reemplaza al texto original.

El lote se escribió en una base local aislada, sin llamadas de modelos ni conexión de red:

- `.local/happydb/write-validation.db`: 2.500 recuerdos, revisiones, documentos FTS y operaciones de escritura.
- `.local/happydb/mem-write-requests.jsonl`: 2.500 solicitudes para reproducir `mem-write-batch`.
- `.local/happydb/write-receipts.jsonl`: 2.500 recibos iniciales.
- `.local/happydb/replay-receipts.jsonl`: los mismos 2.500 reintentos idempotentes.
- `.local/happydb/mem-write-requests.human-review.csv` y `.local/happydb/human-review.jsonl`: texto original, referencia limpia, reinterpretación, nota y columnas vacías para el dictamen humano.
- `.local/happydb/agent-rewrites/`: los tres lotes separados y el archivo unido.

Para reconstruir la muestra y escribirla desde la raíz del repo, con el corpus y las reinterpretaciones locales disponibles:

```sh
python3 tools/datasets/prepare_happydb.py \
  --limit 2500 \
  --rewrites .local/happydb/agent-rewrites/all.jsonl \
  --output .local/happydb/mem-write-requests.jsonl \
  --review-output .local/happydb/human-review.jsonl
GENNIE_MEMORY_DATABASE_PATH="$PWD/.local/happydb/write-validation.db" \
  cargo run --offline -p gennie-memory -- mem-write-batch .local/happydb/mem-write-requests.jsonl
```

La verificación local comprobó 2.500/2.500 filas almacenadas exactamente frente a su solicitud: original, reinterpretación, tipo, ámbito, procedencia y opcionales. SQLite contiene 2.500 recuerdos, 2.500 revisiones, 2.500 documentos FTS y 2.500 operaciones de escritura, en 486 ámbitos. Al repetir el lote, los 2.500 recibos indicaron `replayed: true`, devolvieron los mismos IDs y no crearon filas adicionales. La búsqueda FTS devolvió coincidencias. Esto valida la ruta de escritura local y su persistencia, no la fidelidad semántica de los resúmenes.

## Contrato de escritura implementado

`mem-write` implementa `action: create`; `mem-write-batch` acepta hasta 2.500 creaciones y llama a la misma operación una vez por recuerdo. Cada escritura confirma recuerdo, revisión, proyección FTS e idempotencia en una sola transacción. Repetir el mismo UUID y solicitud recupera la identidad guardada; reutilizar ese UUID con contenido distinto devuelve `idempotency_conflict`.

La superficie es local y confía en el operador CLI. No implementa autenticación multiusuario, ACL ni registro de ámbitos, validación remota de procedencia, deduplicación semántica, reconciliación ni otras acciones de ciclo de vida. Véase [diseño de memoria §2.1](../architecture/memory-design.md).

## Revisión humana y evidencia pendiente

La persona revisora puede marcar fidelidad, conservación del significado y detalle inventado; puede anotar decisiones, identificador y hora UTC en el CSV. No sobrescriba la reinterpretación original. Hasta completar esa revisión, ninguna propuesta debe tratarse como gold ni usarse para afirmar calidad. La creación y escritura no constituyen validación de recuperación; esa campaña se planificará aparte después de la revisión humana.
