# Decisiones

[0001 — Base local de recuperación](0001-local-baseline.md) documenta la elección de crates, esquema reducido y límites de la primera entrega. Las respuestas y decisiones originales de inicialización permanecen en `.creation/` como historial; la fuente vigente resumida está en [requirements.md](../initialization/requirements.md) y [implementation.md](../initialization/implementation.md).

[0002 — Banco sintético y pilotos comparables](0002-synthetic-benchmark.md) registra la generación versionada de consultas y la separación entre experimentos pequeños y campaña final.

[0003 — Espacio semántico versionado](0003-versioned-embeddings.md) describe la migración v2, validaciones del vector y el límite del encoder Laya experimental.

[0004 — Corpus v2 con respuestas variables](0004-benchmark-v2.md) conserva v1 y corrige la fuga de respuesta por tipo de tarea; registra la diversidad y sus límites.

[0005 — Filtro estructurado de entidad](0005-entity-filter.md) registra la migración v3, la ablación emparejada y el límite de resolver la identidad en un host sintético.

[0006 — Varias memorias por entidad](0006-multi-memory-benchmark.md) registra el banco v4 y el fallo de selección positiva observado con Laya local.

[0007 — Condiciones y fuente declarada](0007-conditional-seed-provenance.md) registra la migración 4 y el banco v5 de aplicabilidad.

[0008 — Sesión local de evaluación](0008-local-evaluation-session.md) explica el transporte JSONL y la primera medición completa de FTS5, Laya y semilla.

[0009 — Consulta léxica rápida optativa](0009-optional-keyword-preflight.md) registra la medición y la política inicial, ahora sustituida.

[0010 — Búsqueda rápida predeterminada](0010-keyword-search-default.md) registra la decisión posterior de priorizar FTS5 por palabras clave como base, con el híbrido explícito para comparación.

[0011 — Recorrido textual completo en Rust](0011-rust-recall-cli.md) registra el contrato del comando `recall`, el adaptador OpenRouter y la permanencia de Laya solo para el híbrido experimental.

[0012 — Expansión por prefijo en la búsqueda rápida](0012-keyword-prefix-expansion.md) registra el ajuste léxico y de instrucción de Decisions, con selección en desarrollo, validación final y coste observado.

[0013 — Protocolo previo a la evaluación final del MVP](0013-mvp-final-evaluation-protocol.md) reserva 018–020, fija criterios y enlaza los resultados sintéticos sin confundirlos con la auditoría humana pendiente.

Cada decisión nueva debe indicar contexto, alternativas, motivo, estado y condición de revisión. Una sustitución conserva el documento anterior y enlaza el nuevo; no reescribe silenciosamente la historia. Los registros de [memoria de trabajo](../memory/index.md) aportan evidencia, pero no modifican reglas ni decisiones por sí solos.
