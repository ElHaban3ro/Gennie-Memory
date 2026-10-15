# Mapa de módulos

| Crate | Responsabilidad | Dependencias obligatorias | Opcionales | Recursos propios | Contratos |
| --- | --- | --- | --- | --- | --- |
| `iromem-kernel` (`kernel/`) | `AppModule`, configuración y conexión/migraciones SQLite | Ninguna interna | Ninguna | Tabla transversal `schema_migrations` | `AppModule`, `Migration`, `Config` |
| `iromem-retrieval` (`apps/retrieval/`) | Cargar fixtures, vectores y buscar candidatos autorizados por ámbito simulado | `iromem-kernel` | Ninguna | `memories`, `memory_revisions`, `search_documents`, `memory_fts`, `embedding_spaces`, `memory_embeddings` | `FixtureMemory`, `SearchRequest`, `HybridRequest`, `SeedRequest` y respuestas |
| `iromem` (`bootstrap/`) | Registrar módulos, abrir DB y adaptar CLI/JSON | `iromem-kernel`, `iromem-retrieval` | Ninguna | No posee tablas de negocio | Comandos `init-db`, `load-fixtures`, `load-embeddings`, `search`, `hybrid-search`, `search-batch`, `hybrid-search-batch`, `compose-seed`, `compose-seed-batch`, `session` |

El único módulo registrado se llama `retrieval`, con migraciones 1 a 4 y activo por defecto. No hay tareas, rutas HTTP, permisos reales, eventos ni activación runtime. El orden topológico ya rechaza ciclos, ausencias y nombres duplicados; sus pruebas de regresión se ampliarán al introducir un segundo módulo. Los datos de ámbitos actuales son sintéticos, no una autorización de host.

Al sumar embeddings, decide si son parte profunda del módulo de recuperación o una app autónoma según responsabilidad y dependencias reales. Actualiza aquí propietario de datos, contratos consumidos/publicados y dependencias antes de tratar el nuevo módulo como integrado.
