# Embeddings locales v1

Estado: investigación implementada y medida en esta Mac, 2026-10-24; el usuario decidió conservarla fuera de la ruta de producto por su mayor latencia. Perfil seleccionado para investigación: [E5 small](../../config/embedding-profiles/e5-small.json). [Plan y puertas](local-embeddings-plan.md) · [Resultados y pruebas](../evaluation/local-embeddings/README.md) · [Decisión y dependencias](../decisions/0051-independent-local-embeddings.md).

## Perfil independiente

`intfloat/multilingual-e5-small`, revisión `614241f622f53c4eeff9890bdc4f31cfecc418b3`, 384 dimensiones y máximo 512 tokens. Prefijos `query: ` y `passage: `, tokenizer fijado, media de tokens reales incluidos especiales, sin padding, normalización L2 y f32. Runtime CPU Candle 0.11.0 y tokenizers 0.23.2; la implementación no usa Decisions, Jev, Laya o Python. El ID de espacio es SHA-256 del perfil completo: cambia al cambiar pesos, configuración, tokenizer, runtime o procedimiento. Las rutas locales no forman parte de la identidad semántica.

Se eligió E5 sobre el control MiniLM por el desempate fijado antes de inferencia: mismo top-5 y mayor ventana de contexto. Es el perfil de esta versión local, no una afirmación de superioridad general. [Artefactos y hashes](../evaluation/local-embeddings/e5-small-artifacts.json); los pesos coinciden también con el SHA-256 LFS declarado por el repositorio oficial. Licencias y aviso transitivo de mantenimiento están registrados en la decisión 0051.

Los tres archivos necesarios son `model.safetensors`, `tokenizer.json` y `config.json` bajo `artifact_dir`. Están descargados en `.local/embedding-models/e5-small/` e ignorados por Git. En otra instalación deben descargarse de esa revisión de Hugging Face; el runtime comprueba los tres hashes antes de usar el modelo. No descarga, ejecuta código remoto, altera modelos compartidos ni hace fallback a otro encoder.

## Fragmentos y publicación

Original y síntesis se tokenizan por separado con truncamiento y padding desactivados. El particionador reserva tokens del prefijo/especiales, verifica cada fragmento contra el límite real y conserva rangos UTF-8 contiguos sin huecos ni solapamiento. Al concatenarlos se recuperan todos los bytes del texto, incluidos espacios, negaciones y condiciones. Normalizar para tokenizar no reemplaza el texto guardado. Cada fragmento registra campo, ordinal, rango, hash del texto fuente, hash del fragmento y cantidad real de tokens. Las consultas excesivas fallan con `embedding_input_too_long`.

`prepare` captura hasta 10.000 recuerdos activos actuales de un actor/ámbito, incluidas revisiones con vigencia futura o pasada, y genera sus vectores fuera de una transacción de escritura. Máximo 100.000 fragmentos. `publish` comprueba perfil, dimensiones/finitud/norma, tokenización reproducida, ambos campos completos, IDs y ausencia de fragmentos huérfanos. Dentro de una transacción revalida el inventario/texto/revisión y publica todo o nada. El recibo es `published_complete`. Las claves `(index_id,memory_id,revision,field,ordinal)` conservan revisiones y espacios separados.

El ID de índice liga el lote completo, incluidos vectores; repetirlo devuelve `replayed:true`. La migración 9 guarda vectores como f32 little-endian en BLOB para evitar parsear JSON en cada barrido. No cambia las tablas/vectores históricos. La fuente declarada permanece sin verificar.

Se activa una versión suministrando su ID publicado en cada búsqueda. Se comprueba de nuevo el manifiesto del ámbito y la cantidad de fragmentos: una nueva memoria, revisión o cambio de texto/estado vuelve obsoleto ese índice. No se sirve cobertura parcial ni vectores antiguos. La reconstrucción es explícita; no hay cola, leases, reintentos o trabajador persistente en esta etapa.

## Búsqueda y presupuestos

`search` acepta `index_id`, `search:SearchRequest`, `deadline_ms`, `max_scored_chunks` (100.000 por defecto, máximo 1.000.000) y `semantic_only` (false por defecto). Usa la conexión compartida y una instantánea SQLite. Aplica actor, ámbito, entidad, tipo, revisión, estado y vigencia antes de leer/puntuar los fragmentos autorizados. Agrupa por memoria/revisión usando el mayor coseno de sus fragmentos; conserva la ubicación del mejor fragmento.

Con `semantic_only:true`, ordena por coseno exacto. Con false, fusiona top-20 textual y semántico mediante RRF 60, con desempate textual, semántico e ID. Un rango semántico superior a 20 no suma voto RRF. El resultado informa tiempo de embedding, barrido exacto y total, número de recuerdos elegibles/fragmentos puntuados y si el límite de candidatos recorta el conjunto. El máximo del coseno no es probabilidad, aplicabilidad demostrada ni criterio calibrado de abstención: las consultas ajenas también producen candidatos.

`deadline_ms` admite 1–300.000 ms en preparación/búsqueda; publicación tiene 30.000 ms. Se comprueba entre etapas, fragmentos y llamadas tensoriales, y después de inferir. No interrumpe a mitad una llamada de Candle: es un plazo cooperativo, no una garantía de cancelación del kernel. Agotar plazo o trabajo produce error sin lista parcial. La carga inicial del modelo se mide en `ready.model_load_ms`, fuera del plazo de las consultas de una sesión ya abierta.

## CLI/JSONL

Desde la raíz del repositorio:

```sh
cargo build --offline --release -p gennie-memory --features local-embeddings
GENNIE_MEMORY_DATABASE_PATH=.local/embedding-demo.db target/release/gennie-memory load-fixtures tests/fixtures/local-embeddings-v1/corpus.json
GENNIE_MEMORY_DATABASE_PATH=.local/embedding-demo.db target/release/gennie-memory embedding-session config/embedding-profiles/e5-small.json
```

En una instalación nueva, ejecutar primero `cargo fetch --locked` con red para obtener las dependencias fijadas. Los pesos se obtienen por separado de su revisión fijada. Sin la feature, FTS5 y los contratos anteriores siguen funcionando sin cargar el encoder.

La sesión emite primero `ready`, luego recibe una operación por línea y responde `{status,response|error,elapsed_ms}`. Operaciones:

| Operación | `request` |
| --- | --- |
| `index` | `{scope:{kind,id},actor_id?:string,deadline_ms}`: prepara y publica |
| `prepare` | Lo mismo; devuelve lote serializable sin publicarlo |
| `publish` | Lote completo obtenido de prepare |
| `publish_file` | `{path}` a un lote JSON local ≤512 MiB |
| `search` | Consulta descrita arriba |
| `segment` | `{text}` para inspeccionar fragmentos sin inferencia |
| `text_search` | `SearchRequest`, mismo servicio FTS5 |
| `load_fixtures` | Array de fixtures para evaluación local |

Cada línea está limitada a 64 MiB. Un error de operación deja la sesión utilizable; un error de carga inicial termina el proceso. Cerrar stdin termina la sesión. No hay puerto ni servicio de fondo. `load_fixtures` sigue siendo herramienta del operador local y no equivale a escritura autenticada.

El [banco congelado](../../tests/fixtures/local-embeddings-v1/README.md) permite publicar vectores ya calculados sin repetir inferencia documental. No usar sus ID de índice con otro corpus o perfil. Las rutas `recall`/`recall-v1` siguen textuales y no consumen automáticamente este índice; incorporar el híbrido a composición/clasificación corresponde a M3.
