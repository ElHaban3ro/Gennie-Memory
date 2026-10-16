# Mapa de módulos

| Crate | Responsabilidad | Dependencias obligatorias | Opcionales | Recursos propios | Contratos |
| --- | --- | --- | --- | --- | --- |
| `iromem-kernel` (`kernel/`) | `AppModule`, configuración y conexión/migraciones SQLite | Ninguna interna | Ninguna | Tabla transversal `schema_migrations` | `AppModule`, `Migration`, `Config` |
| `iromem-retrieval` (`apps/retrieval/`) | Cargar fixtures, vectores, buscar candidatos autorizados y orquestar elección/composición de semilla | `iromem-kernel` | Ninguna | `memories`, `memory_revisions`, `search_documents`, `memory_fts`, `embedding_spaces`, `memory_embeddings` | `FixtureMemory`, `SearchRequest`, `HybridRequest`, `SeedRequest`, `RecallRequest`, `Classifier` y respuestas |
| `iromem` (`bootstrap/`) | Registrar módulos, abrir DB, adaptar CLI/JSON y llamar a OpenRouter para `recall` | `iromem-kernel`, `iromem-retrieval` | Ninguna | No posee tablas de negocio | Comandos `init-db`, `load-fixtures`, `load-embeddings`, `search`, `recall`, `hybrid-search`, `search-batch`, `hybrid-search-batch`, `compose-seed`, `compose-seed-batch`, `session` |
| `iromem-eval` (`tools/evaluation/`) | Validar bancos, ejecutar campañas y comparaciones sintéticas y generar figuras | Ninguna interna; invoca los binarios por CLI | Ninguna | Archivos y directorios explícitos; diarios sincronizados | Comandos `validate-*`, `retrieval`, `campaign`, `engram-benchmark`, `codex-*`, `render-charts` |

El único módulo registrado se llama `retrieval`, con migraciones 1 a 4 y activo por defecto. No hay tareas, rutas HTTP, permisos reales, eventos ni activación runtime. El orden topológico ya rechaza ciclos, ausencias y nombres duplicados; sus pruebas de regresión se ampliarán al introducir un segundo módulo. Los datos de ámbitos actuales son sintéticos, no una autorización de host.

Al sumar embeddings, decide si son parte profunda del módulo de recuperación o una app autónoma según responsabilidad y dependencias reales. Actualiza aquí propietario de datos, contratos consumidos/publicados y dependencias antes de tratar el nuevo módulo como integrado.
