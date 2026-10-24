# Workspace, módulos y dependencias

Propietarios: `Cargo.toml`, `kernel/src/core.rs`, `bootstrap/src/registry.rs`, `apps/*/src/module.rs`. El workspace usa edición 2024, resolver 3 y dependencias centralizadas; cada miembro hereda paquete y dependencias internas. `kernel` nunca importa una app. `apps/retrieval` depende de kernel; `bootstrap` conoce e instancia los módulos. No crees `contracts/` hasta necesitar romper un ciclo real. Los módulos son crates en un solo proceso, no microservicios.

`AppModule` declara nombre estable, estado inicial, dependencias obligatorias/opcionales, migraciones ordenadas y lifecycle sin efecto por defecto. El registro explícito rechaza nombres duplicados, dependencias ausentes y ciclos antes de migrar. Solo está registrado `retrieval`. El dashboard fue retirado; no registrar una app ficticia para mantener sus tablas. Bootstrap verifica exclusivamente las migraciones retiradas ya aplicadas, usando su SQL congelado, sin aplicar ese esquema en bases nuevas. No hay activación en caliente ni eventos. Cuando existan: alinea Cargo/features, `depends_on()` y estado runtime; las dependencias opcionales ordenan si están presentes pero no obligan activación. Separa llamada directa, trait en contracts y evento según necesidad real. Un bus efímero no garantiza entrega; para durabilidad usa outbox/cola. No añadas capacidad vacía solo para completar el árbol.

El contrato local de creación de recuerdos vive en `apps/retrieval/src/write.rs`; bootstrap solo adapta JSON/JSONL a esa función. El módulo retrieval es propietario de la migración de idempotencia 5 y de la transacción SQL; no muevas SQL ni reglas de escritura al CLI.

Antes de incorporar dependencias verifica versión exacta en docs.rs/registro oficial, features, compatibilidad y avisos RustSec; revisa señales de compromiso y registra límites de la comprobación en `docs/decisions/`. El [registro de versiones inicial](../decisions/0001-local-baseline.md) documenta lo comprobado. Un resultado de búsqueda sin aviso no demuestra ausencia de vulnerabilidades. Mantén herramientas de prueba fuera del binario productivo.

Para añadir una app: identifica responsabilidad y recursos, crea crate y `AppModule`, declara dependencias, regístrala en bootstrap, resuelve orden de migración, añade pruebas que detecten colisiones/ciclos, documenta contratos y actualiza `docs/architecture/modules.md`. Revisa consumidores de contratos y ejecuta `cargo test --offline --workspace`. Antes de cambiar Cargo.toml lee también `docs/rules/testing.md`.

`gennie-memory/http2-transport` es una feature optativa de bootstrap, defaultvacío, que propaga `reqwest/http2` y h2=0.4.20fijado. Las cinco dependencias añadidas y sus comprobaciones están en[0022](../decisions/0022-opt-in-http2-evaluation.md); no modifica dependencias entre apps ni activa servicios.

El registro de entidades y alias vive dentro de retrieval (`entity.rs`), con migración 8 y transacción sobre la misma conexión; no es una app ni dependencia nueva. Bootstrap adapta CLI y session mutable a `entity::resolve`. [Propiedad y contrato](../architecture/retrieval-isolation.md).

`local-embeddings` es optativa en retrieval y bootstrap: Candle 0.11.0 y tokenizers 0.23.2 se fijan en workspace; los modelos quedan fuera de Git. Las migraciones, incluida la 9, no dependen de features para preservar un esquema único. La carga/inferencia se habilita solo con la feature y settings explícitos. No incorporar dependencias ML al kernel o al evaluador: este último invoca la CLI. [Decisión y revisión de dependencias](../decisions/0051-independent-local-embeddings.md).

## Conteo de tokens de producto (2026-10-24)

`retrieval::token_budget` posee el contador local, perfil y trazas; añade `tiktoken-rs =0.12.0` sin features predeterminadas. `Classifier` publica entradas acotadas que fallan cerrado; bootstrap posee la inspección del cuerpo HTTP. El grafo acíclico y SQLite única permanecen. El vocabulario se carga localmente una vez; no hay descarga runtime. [Decisión 0052](../decisions/0052-recall-token-budgets.md).

## Procedencia local (2026-10-24)

Retrieval posee source_verification.rs y migración 10; no es una app nueva. Register/verify/check_candidates operan sobre la conexión existente; check_evidence expone comparación para una evidencia aún sin memoria y usa transacción prestada sin gestionarla. Bootstrap adapta comandos y JSONL. Product mantiene política y degradación, sin trasladar SQL al adaptador. [Decisión 0053](../decisions/0053-local-source-verification.md).

`retrieval::lifecycle` posee calendario por revisión, retirada/restauración/borrado y cola durable por ámbito/perfil (migración 12). La publicación E5 completa comparte transacción con finalización del trabajo; bootstrap solo adapta CLI/JSONL. No se añade app, dependencia ni proceso de fondo. [Contrato](../architecture/memory-lifecycle.md).

`retrieval::reconciliation` posee los contratos locales de escritura, propuestas, relaciones y auditoría; `write` expone internamente su transacción reutilizable. Bootstrap adapta JSON y un clasificador de relaciones optativo en `openrouter/write_relations.rs`, sin dependencia nueva. La migración 11 pertenece a retrieval y se registra antes de lifecycle12. [Contrato](../architecture/write-reconciliation.md).
