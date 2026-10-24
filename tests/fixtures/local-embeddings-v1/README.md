# Banco local de embeddings v1

Controles sintéticos del mismo autor, congelados antes de inferir el 2026-10-24. `suite.json` contiene 25 recuerdos y 29 consultas: 25 positivas y cuatro ajenas. `corpus.json` extrae los mismos recuerdos para la CLI. No hay datos personales, revisión humana o reserva de generalización; las cuatro consultas ajenas no calibran un umbral.

`e5-frozen-index.json` conserva los **53 fragmentos/vectores reales** del perfil E5 seleccionado, con original/síntesis, rangos UTF-8, tokens, hashes, revisión y espacio. `manifest.json` liga hashes del banco, índice congelado y settings, además de los IDs exactos de publicación. [Protocolo](../../../docs/architecture/local-embeddings-plan.md) · [Mediciones](../../../docs/evaluation/local-embeddings/README.md).

El índice solo se puede publicar con el corpus íntegro y el perfil correspondiente; un cambio de revisión/texto hace fallar la importación. Para probarlo en una DB de demostración separada:

```sh
cargo build --offline --release -p gennie-memory --features local-embeddings
GENNIE_MEMORY_DATABASE_PATH=.local/embedding-demo.db target/release/gennie-memory load-fixtures tests/fixtures/local-embeddings-v1/corpus.json
GENNIE_MEMORY_DATABASE_PATH=.local/embedding-demo.db target/release/gennie-memory embedding-session config/embedding-profiles/e5-small.json
```

Luego enviar una línea:

```json
{"op":"publish_file","request":{"path":"tests/fixtures/local-embeddings-v1/e5-frozen-index.json"}}
```

Usar el `index_id` del recibo para consultar:

```json
{"op":"search","request":{"index_id":"idx-004d6cf69c04e63cf745451866cd5a9a33b7ba9e302c3cf151dec22bbdc2efaa","search":{"query":"What food causes Marta an allergic reaction?","scope_kind":"project","scope_id":"embedding-study","as_of":"2026-10-24T12:00:00.000Z","limit":5},"deadline_ms":1000,"semantic_only":true}}
```

Esto importa los vectores existentes; la consulta sí ejecuta el encoder local. Los pesos están en `.local/embedding-models/e5-small`, ignorados por Git, y se verifican por hash al arrancar. Cerrar stdin termina el proceso. El cargador de fixtures no admite volver a insertar la misma revisión; usar una DB nueva para repetir desde cero.
