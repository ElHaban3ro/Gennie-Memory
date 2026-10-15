# 0001 — Base local de recuperación

Fecha: 2026-10-15. Estado: provisional para el MVP local. Origen: propuesta del asistente a partir de los requisitos confirmados en `.creation/answers.md`. Revisar al incorporar embeddings/proveedores o si el benchmark exige otra persistencia.

## Contexto y decisión

Se necesita una primera línea de referencia ejecutable para consultas sintéticas sin hook, MCP ni datos reales. Se eligió workspace Cargo con `kernel`, `apps/retrieval` y `bootstrap`; SQLite/FTS5 y una CLI local. Se omiten crates vacíos, frontend y servidor. La migración experimental contiene solo tablas necesarias para revisiones y texto; el SQL adjunto al README continúa como propuesta del producto completo. La ruta SQLite es explícita mediante `IROMEM_DATABASE_PATH`.

Alternativas: implementar todo el esquema y ciclo de vida antes de medir recuperación, o generar solo documentos sin recorrido ejecutable. La primera ampliaría trabajo ajeno a la validación inicial; la segunda no permitiría probar consultas. Esta base hace explícita la cobertura semántica deshabilitada y conserva la evolución en `ROADMAP.md`.

## Dependencias comprobadas al incorporarlas

Versiones exactas fijadas en `Cargo.toml`: `rusqlite 0.40.2` con SQLite embebido, `serde 1.0.229` con derive, `serde_json 1.0.154`, `dotenvy 0.15.7`, `sha2 0.11.0`. La versión de Rust comprobada localmente fue 1.95.0. Las versiones se contrastaron el 2026-10-15 con [docs.rs rusqlite](https://docs.rs/rusqlite/0.40.2/rusqlite/), [serde](https://docs.rs/serde/1.0.229/serde/), [serde_json](https://docs.rs/serde_json/1.0.154/serde_json/), [dotenvy](https://docs.rs/dotenvy/0.15.7/dotenvy/) y [sha2](https://docs.rs/sha2/0.11.0/sha2/); los paquetes estaban en la caché local salvo tres artefactos transitivos descargados para completar `Cargo.lock`.

Se revisaron las páginas RustSec de [rusqlite](https://rustsec.org/packages/rusqlite.html) y [sha2](https://rustsec.org/packages/sha2.html): los avisos allí mostrados afectan versiones anteriores a las fijadas. La búsqueda de avisos para `serde`, `serde_json` y `dotenvy` no mostró un aviso directo en las fuentes consultadas. Esto es una revisión puntual de dependencias directas, no una auditoría exhaustiva de toda la cadena ni prueba de ausencia de vulnerabilidades o compromiso. `cargo-audit` no estaba disponible localmente; repetir auditoría de lockfile cuando esté disponible.

## Límites conocidos

La consulta actual es FTS5 con términos, no preserva todavía grupos de frase entre comillas. `as_of` admite UTC con milisegundos. El cargador recibe datos sintéticos y no implementa procedencia, idempotencia o autorización de un host real. No hay embeddings, clasificador ni semilla final; `SearchResponse` contiene candidatos y BM25, sin tratar esa puntuación como relevancia calibrada. No se ha medido ahorro de tokens o tiempo del agente.
