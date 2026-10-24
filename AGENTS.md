# Gennie Memory

Gennie Memory evalúa recuperación de recuerdos semilla en esta Mac. La base actual incluye FTS5 por palabras clave como ruta predeterminada, un filtro opcional de entidad, condición y fuente declarada por revisión, un índice semántico experimental versionado, coseno exacto y RRF. `recall` encadena en Rust búsqueda, Decisions/Jev por OpenRouter y composición; `gennie-memory-eval` ejecuta campañas, comparaciones y gráficas en Rust. Los vectores Laya congelados sirven al experimento híbrido histórico; no hay un codificador Laya activo. El perfil de embedding definitivo, verificación de fuentes e integración Codex siguen pendientes en [ROADMAP.md](ROADMAP.md). `kernel/` posee infraestructura y contratos transversales; `apps/retrieval/` posee datos y búsqueda; `bootstrap/` compone módulos y expone CLI/JSONL sin servidor HTTP; `tools/evaluation/` aloja la evaluación Rust fuera del producto. No hay `contracts/` porque todavía no existe un ciclo entre apps, ni hook o servidor MCP.

**Antes de modificar código, identifica las áreas afectadas y lee los archivos de reglas correspondientes. Los enlaces de este índice son instrucciones de lectura, no sustituyen su contenido.** Si continúas esta inicialización o una parte incompleta del MVP, lee también `.creation/initialization-prompt.md`, `.creation/answers.md`, `.creation/decisions.md` y `.creation/plan.md` antes de planificar o generar código.

El nombre técnico vigente es `gennie-memory` y el remoto es `git@github.com:ElHaban3ro/Gennie-Memory.git`. Conserva las excepciones históricas autorizadas en la [decisión 0048](docs/decisions/0048-gennie-memory-rename.md).

## Invariantes y colaboración

- Conserva un grafo de crates acíclico: kernel no importa apps ni contracts; bootstrap compone implementaciones. La conexión SQLite es única por proceso; no crees conexiones independientes dentro de apps.
- Separa secretos del estado y de Git. Las migraciones aplicadas son inmutables y verificadas por checksum. Publica cambios o avisos solo después de confirmar su transacción cuando se incorporen.
- Usa inglés en archivos, símbolos, SQL e IDs técnicos; español en documentación y mensajes dirigidos al usuario. Usa UTC RFC 3339 con milisegundos para horas intercambiadas y `Option<T>` para ausencias.
- Conserva la numeración del usuario al responder listas. Corrige problemas incidentales claros y pequeños dentro del alcance autorizado e informa el arreglo; explica por separado cambios de arquitectura o mecanismos nuevos y su coste antes de incorporarlos si amplían el encargo.
- Ante un fallo inspecciona síntoma y causa con evidencia. Distingue hechos, hipótesis, pruebas estáticas, compilación, ejecución y despliegue; no defiendas un resultado no comprobado.
- Preserva cambios existentes y la rama actual. Otro worktree o rama requiere solicitud o autorización explícita. Antes de iniciar un servicio local, anuncia propósito y puerto, usa uno libre, conserva su identificación, no detengas procesos ajenos y apaga los temporales que iniciaste. Las ejecuciones CLI son procesos breves o sesiones por stdin/stdout, sin puerto.
- Commit y push solo con autorización explícita para el cambio correspondiente. Idioma y formato de commits: pendiente de acordar. Publicar código y desplegar son operaciones distintas.
- Documenta el estado vigente y retira afirmaciones obsoletas. Si se autorizan agentes paralelos, asigna archivos y verificación propios; esta regla no obliga a delegar.

## Lectura obligatoria por tarea

| Condición | Leer antes de cambiar | Motivo |
| --- | --- | --- |
| Servicios Rust, CLI, errores, estado compartido o futuro HTTP/autenticación | [backend.md](docs/rules/backend.md) | Límite entre servicio y adaptador |
| Crates, Cargo.toml, features, registro, dependencias, eventos o lifecycle | [modules.md](docs/rules/modules.md) | Grafo y composición explícita |
| SQL, repositorio, conexión, transacciones, esquema o migraciones | [database.md](docs/rules/database.md) | Propiedad e integridad de datos |
| `.env`, variables, secretos, perfiles o configuración runtime | [configuration.md](docs/rules/configuration.md) | Precedencia y validación |
| Futura interfaz de usuario | [frontend.md](docs/rules/frontend.md) | Diseño y comportamiento del cliente local |
| Escribir o ejecutar pruebas y elegir verificación | [testing.md](docs/rules/testing.md) | Alcance proporcional |
| Cambio transversal o app nueva | [overview.md](docs/architecture/overview.md), [modules.md](docs/architecture/modules.md) y reglas anteriores aplicables | Mapa de dependencias y recursos |
| Continuar trabajo previo o investigar decisiones | [memory/index.md](docs/memory/index.md), [decisions/README.md](docs/decisions/README.md) y entradas pertinentes | Historia no equivale a estado vigente |

El dashboard fue retirado por decisión del usuario; [frontend.md](docs/rules/frontend.md) registra ese estado. Para una app nueva con datos y CLI, lee modules, backend, database, configuration y testing; si se añade interfaz, lee también su regla específica. Si falta un documento obligatorio, resuelve el acceso antes de modificar el área dependiente.

## Comandos actuales

`cargo fmt --all --check`; `cargo test --offline --workspace`; `cargo run --offline -p gennie-memory -- init-db`; `cargo run --offline -p gennie-memory -- mem-write <request.json>`; `cargo run --offline -p gennie-memory -- mem-write-batch <requests.jsonl>`; `cargo run --offline -p gennie-memory -- load-fixtures tests/fixtures/small-corpus.json`; `cargo run --offline -p gennie-memory -- search tests/fixtures/small-query.json`; `cargo run --offline -p gennie-memory -- recall tests/fixtures/small-recall.json`; `cargo run --offline -p gennie-memory-eval -- validate-benchmark tests/benchmark/v5`; `cargo run --offline -p gennie-memory-eval -- validate-general-memory tests/fixtures/general-memory-v1`. La CLI requiere `GENNIE_MEMORY_DATABASE_PATH` exportada o `.env`; los comandos de ejemplo escriben en `.local/iromem.db`. `mem-write-batch` acepta hasta 2.500 creaciones idempotentes. Los comandos por lotes, `compose-seed`, `session`, los vectores y los pilotos se describen en [contract-v1.md](docs/evaluation/contract-v1.md). Consulta [testing.md](docs/rules/testing.md) antes de ampliar pruebas.

Cada regla detallada tiene un único documento propietario. Si las reglas se contradicen, identifica y resuelve la contradicción antes del cambio; instrucciones explícitas del usuario y restricciones del entorno tienen prioridad. Actualiza regla, arquitectura, decisiones y verificaciones relacionadas en el mismo cambio.
