# Contrato de evaluación v1

Estado vigente: el producto y el evaluador activo son Rust. Los bancos y resultados de pilotos anteriores siguen versionados como evidencia histórica, pero sus scripts Python fueron retirados. Las cifras nuevas de la ruta predeterminada están en [la medición Rust](rust-migration-v1.md).

## Bancos y etiquetas

Los bancos sintéticos v1–v5 están congelados en `tests/benchmark/`. `iromem-eval validate-benchmark <directorio>` comprueba hashes declarados, unicidad, particiones por entidad, revisiones, ámbito, vigencia y correspondencia de hechos, condiciones y fuentes. v1 contiene 10.800 consultas; v2–v4, 41.600 cada uno; v5, 42.400. Las etiquetas proceden de reglas deterministas, no de revisión humana. Los detalles de generación y las limitaciones están en los manifiestos y [el primer piloto](pilot-2026-10-15.md). La regeneración byte por byte desde las plantillas Python se retiró; los archivos versionados son el corpus de referencia.

`iromem-eval validate-general-memory tests/fixtures/general-memory-v1` comprueba los 2.800 recuerdos, 3.300 preguntas etiquetadas y 140 preguntas sin ámbito explícito, incluidos IDs y ámbitos esperados. `validate-candidate-matrix tests/benchmark/candidate-matrix-v1/<100|200|2000>` comprueba hashes, tamaños, etiquetas e identidades de las tres matrices. Los sujetos de desarrollo y final del banco general están separados; no se ajustan instrucciones con la cohorte final.

## Producto: búsqueda y semilla

`search` recibe `SearchRequest`: `query`, `scope_kind`, `scope_id`, `entity_id` opcional, `as_of` UTC RFC 3339 y `limit`. FTS5 por palabras clave es el modo predeterminado: combina términos exactos con prefijos de dos caracteres menos cuando el término tiene siete o más. `query_mode: "natural"` selecciona explícitamente la búsqueda FTS5 anterior; `hybrid-search` acepta un vector y espacio versionados, aplica filtros, coseno exacto y RRF en Rust. El índice híbrido permanece experimental. Ni `search` ni `recall` generan vectores con Laya.

`recall <archivo.json>` recibe `search` y `provider` opcional (`decisions` por defecto o `jev` explícito). Si se omite `limit`, usa veinte candidatos. Busca, ofrece solo candidatos autorizados al clasificador tipado de OpenRouter, valida el ID elegido y la revisión, y compone una semilla determinista. Una lista vacía termina como `skipped_empty` sin llamar al proveedor. Una respuesta incierta o inválida devuelve `degraded`, candidatos y código de error, sin presentar una semilla fiable. No hay cascada automática ni reintento del proveedor. El comando requiere `IROMEM_DATABASE_PATH`; la llamada remota lee `IROMEM_OPENROUTER_API_KEY` del entorno o `.env`, nunca del JSON.

Ejemplo sintético:

```json
{
  "search": {
    "query": "¿Qué formato prefiere Marina?",
    "scope_kind": "project",
    "scope_id": "demo_alpha",
    "as_of": "2026-10-15T12:00:00.000Z",
    "limit": 20
  },
  "provider": "decisions"
}
```

La respuesta incluye `retrieval` (método y candidatos), `classification_status`, proveedor/modelo pedido y servido, elección, `seed`, `timings`, tokens y coste cuando el proveedor los informa. `timings.total_ms` comienza al entrar en el servicio: incluye búsqueda, llamada remota y recomprobación para componer; excluye arranque del proceso, apertura/migración de SQLite, lectura del JSON y construcción del cliente HTTP. `session` mantiene una conexión SQLite y acepta líneas JSONL con `op: search`, `recall`, `hybrid_search` o `compose_seed`; devuelve una línea por operación. `compose-seed` revalida que cada ID y revisión seleccionados estuvieran entre los presentados y sigan vigentes.

## Evaluación Rust

`iromem-eval` es un binario separado del producto. Sus comandos actuales son:

| Comando | Entrada y resultado |
| --- | --- |
| `retrieval <resultados.jsonl> <consultas.jsonl> [informe.json]` | Verifica etiquetas y calcula recall@20, top-1, negativos vacíos, IDs prohibidos, estados y percentiles de búsqueda. |
| `validate-benchmark`, `validate-general-memory`, `validate-candidate-matrix`, `verify-hashes` | Comprueban bancos sintéticos versionados y paquetes históricos; `verify-hashes` comprueba integridad de archivos, no reejecuta el clasificador. |
| `select-cohort <consultas-fixture.jsonl> <memorias-fixture.json> <primer-sujeto> <último-sujeto> <salida-nueva.jsonl>` | Deriva una cohorte etiquetada de ámbitos sintéticos numerados, verificando IDs, ámbitos y positivos/negativos sin usar Python. |
| `search-benchmark <consultas.jsonl> <binario-iromem> <db> <salida.jsonl> <split> <keywords|natural>` | Mantiene abierta una sesión Rust, calienta una consulta y registra tiempos y candidatos por caso, más hashes. |
| `campaign <consultas.jsonl> <binario-iromem> <db> <diario.jsonl> <decisions|jev> <split> <max_calls>` | Ejecuta `recall` por sesión Rust, registra manifiesto y eventos `started`/`finished` con sincronización a disco; reanuda sin repetir una llamada incierta. |
| `campaign-report <diario.jsonl> [informe.json]` | Comprueba integridad del diario y resume aciertos, negativos, latencias, uso y coste informado. |
| `engram-benchmark <consultas.jsonl> <memorias.json> <binario-engram> <nuevo-data-dir> <salida.jsonl> <split>` | Importa el fixture a un almacén aislado, usa MCP persistente y registra búsqueda comparable de candidatos. |
| `codex-prompt <diario> <consultas> <selección.json> <binario-codex> <nuevo-directorio> <seed|control>` | Ejecuta turnos efímeros de Codex con o sin semilla inicial y guarda eventos y tokens por caso. |
| `codex-engram <consultas> <selección.json> <binario-codex> <binario-engram> <data-dir> <nuevo-directorio>` | Ejecuta Codex con Engram MCP aislado; exige llamadas MCP y prohíbe shell/web. |
| `paired-flow-report <diario-recall> <codex-semilla.json> <codex-engram.json> <codex-control.json> <salida-nueva.json>` | Empareja por caso tiempos y tokens; suma preparación de semilla y turno de Codex como cálculo, no como tiempo de integración medido. |
| `render-charts <búsqueda-iromem.json> <búsqueda-engram.json> <decisions.json> <jev.json> <codex-semilla.json> <codex-engram.json> <directorio>` | Regenera las dos figuras históricas desde informes observados. |
| `render-keyword-charts <búsqueda-anterior.json> <decisions-anterior.json> <búsqueda-ajustada.json> <decisions-ajustada.json> <búsqueda-nueva.json> <decisions-nueva.json> <directorio>` | Regenera las dos figuras históricas de ajuste, con las cohortes identificadas. |
| `render-current-charts <búsqueda-iromem.json> <decisions.json> <búsqueda-engram.json> <directorio>` | Regenera las figuras de aciertos y búsqueda vigentes del README. |
| `render-codex-current <codex-iromem.json> <codex-engram.json> <emparejado.json> <directorio>` | Regenera la comparación actual de Codex y verifica que ambos caminos y el informe emparejado coincidan en muestra y medianas. |

Los archivos de salida de una campaña deben ser nuevos. El diario congela hashes de casos, binario y base; un `started` sin `finished` bloquea la reanudación para no duplicar una llamada posiblemente cobrada. Los informes de seis turnos de Codex son simulaciones de entrega inicial, no prueban un hook instalado. La [campaña actual](keyword-prefix-v1.md) y el [piloto de Codex](codex-current-v1.md) enlazan datos, aciertos y límites.

## Reproducción local mínima

```sh
cargo fmt --all --check
cargo test --offline --workspace
cargo run --offline -p iromem-eval -- validate-benchmark tests/benchmark/v5
cargo run --offline -p iromem-eval -- validate-general-memory tests/fixtures/general-memory-v1
cargo run --offline -p iromem-eval -- validate-candidate-matrix tests/benchmark/candidate-matrix-v1/2000
IROMEM_DATABASE_PATH=/tmp/iromem-pilot.db cargo run --offline -p iromem -- load-fixtures tests/fixtures/small-corpus.json
IROMEM_DATABASE_PATH=/tmp/iromem-pilot.db cargo run --offline -p iromem -- search tests/fixtures/small-query.json
```

Usa una ruta SQLite temporal nueva; `load-fixtures` no está diseñado para recargar una revisión existente. Las campañas remotas se ejecutan solo sobre datos sintéticos, con propósito, tamaño y duración comunicados antes de iniciarlas, fuera de la suite obligatoria. Los resultados antiguos con Laya, top-1, híbrido y variantes de prompt se mantienen en sus [informes históricos](general-memory-v1-results.md); sus scripts de generación ya no son parte del flujo actual. El perfil definitivo de embeddings, validación de fuentes y evaluación con conversaciones reales siguen pendientes.
