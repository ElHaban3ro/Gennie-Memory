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

[0014 — Dashboard local de revisión](0014-local-review-dashboard.md) registra la app HTTP React/Vite, el feedback local reutilizable y el límite de la exportación optativa.

[0015 — Selector experimental por evidencia](0015-experimental-evidence-selector.md) registra la variante explícita `evidence_v1`, su prueba emparejada en desarrollo y la decisión de no promoverla.

[0016 — Comprobación de respaldo y diagnóstico entre proveedores](0016-status-gate-provider-diagnostic.md) registra el piloto descartado de `support_gate_v1` y la señal exploratoria de Jev en conversaciones nuevas.

[0017 — Cierre de M5](0017-m5-closeout.md) documenta la viabilidad general no demostrada, la corrección del contrato de negativos y las condiciones para otra decisión de calidad.

[0018 — Selección experimental por utilidad de evidencia](0018-experimental-utility-selection.md) registra la variante explícita `utility_v1`, su ganancia emparejada en desarrollo y la validación independiente aún pendiente.

[0019 — Compatibilidad del referente antes de utilidad](0019-experimental-referent-utility.md) registra `utility_v2`, sus dos selecciones de otro referente en desarrollo conocido y la decisión de no promoverla.

[0020 — Testigo de aplicabilidad con citas](0020-grounded-applicability-witness.md) registra el contrato, la extracción/comprobación y su fallo en casos nuevos por pérdida de evidencia útil.

[0021 — Roles de evidencia y recuperación tras abstención](0021-evidence-roles-and-conditional-rescue.md) documenta la distinción de roles, el fallo de la primera instrucción y la revisión condicional con puertas de calidad y recursos.

Cada decisión nueva debe indicar contexto, alternativas, motivo, estado y condición de revisión. Una sustitución conserva el documento anterior y enlaza el nuevo; no reescribe silenciosamente la historia. Los registros de [memoria de trabajo](../memory/index.md) aportan evidencia, pero no modifican reglas ni decisiones por sí solos.

[0022 — Transporte HTTP/2 optativo para evaluación](0022-opt-in-http2-evaluation.md) documenta disponibilidad actualizada, feature y dependencias fijadas para medir la hipótesis de transporte sin alterar el predeterminado.

[0023 — Cláusulas tipadas para diagnóstico de extracción](0023-typed-clause-evaluation.md) registra la traducción experimental, identidad estructurada y reutilización de serde en el evaluador, sin cambiar producto.

[0024 — Referencias con mención y clase declarada](0024-grounded-reference-evaluation.md) registra identidad léxica separada del tipo ontológico, ausencia explícita y límites de literalidad, solo en evaluación.

[0025 — Revisión documental automática única](0025-bounded-documentary-revision.md) registra una etapa adicional obligatoria de evaluación, sin fallback y con contabilidad de ambas etapas.

[0026 — Referencias con evidencia distribuida](0026-distributed-reference-evaluation.md) registra el adaptador local, equivalencias de clase afirmadas y dos contraejemplos que bloquean adopción.

[0027 — Comprobación semántica aislada](0027-isolated-semantic-check-evaluation.md) registra el piloto de cuatro juicios por representación, su fallo y las condiciones para otra versión.

[0028 — Conclusión existencial local en evaluación](0028-local-existential-conclusion.md) separa el testigo local de respuesta común y la identidad global, con canal tipado optativo, pruebas por lectura, límites y adopción pendiente.

[0029 — Plazo total HTTP](0029-total-http-request-deadline.md) registra la reproducción local, el arreglo por petición y los límites de verificación remota.

[0030 — Viabilidad de reranker local](0030-local-reranker-feasibility.md) registra el ensayo aislado, assets/runtime fijados y límites de calidad, recursos y dependencias antes de cualquier integración.

[0031 — Ejecución Metal MLX](0031-local-metal-reranker-execution.md) registra el controlstandalone, runtime aislado y equivalencia previa para los mismos pesos, sin integración o mejora semántica presumida.

[0032 — Adaptación local a utilidad](0032-local-utility-adapter-pilot.md) registra una corrida pequeña con datos nuevos revisados, base congelada y puertas previas a reserva; sin integración o mejora presumida.

[0033 — Ficha semántica automática solo en evaluación](0033-semantic-link-classifier-research.md) registra la candidata de referente/foco/aporte con citas, su incertidumbre explícita y la adopción condicionada a las puertas originales.

[0034 — Gate técnico de extracción de evidencia por slot](0034-slot-evidence-gemma-pilot-gate.md) registra el fallo estructural del smoke de `slot_evidence_v1`, el coste conocido y las condiciones previas a abrir TRAIN.

[0035 — Esquema compacto de evidencia por slot V2](0035-compact-slot-evidence-v2.md) registra el contrato compacto, el gate local de referentes ajenos, los smokes fallidos conservados y las condiciones pendientes para revisión ciega y adopción.

[0036 — Ruta experimental de evidencia por slot V3](0036-slot-evidence-v3-experimental-route.md) separa la identidad ejecutable V3 del historial V2, mantiene el baseline y fija el gate antes de smokes comparativos.

[0037 — Demover sujetos ajenos en oraciones posteriores](0037-demote-foreign-subject-in-later-sentence.md) registra una guarda local por proposición que conserva evidencia útil del mismo recuerdo y sus límites prospectivos.

[0038 — Derivar localmente el sujeto del ancla compacta](0038-derive-compact-subject-locally.md) registra la retirada experimental de la cita redundante, la validación léxica conservadora y las puertas ciegas pendientes.

[0039 — Derivar localmente las citas de slot de la pregunta](0039-derive-query-slots-locally.md) retira del wire experimental las citas de slot generadas, fija el binding de objetivos únicos y registra cobertura, regresiones y límites.

[0040 — Restringir el esquema compacto V3 desde señales locales de la consulta](0040-local-slot-schema-boundaries.md) registra límites dinámicos para slots, anclas, premisas y destinos, la regresión de extracción observada y la decisión de no adoptar.

[0041 — Cascada optativa de rescate de estado parcial](0041-epistemic-partial-rescue-experiment.md) registra la mejora de calidad V1, su fallo de coste/latencia y su retirada del runtime.

[0042 — Habilitar single-pass parcial como política optativa](0042-opt-in-epistemic-partial-single-pass.md) registra la mejora confirmada en V11/V14, coste/latencia dentro del gate y la decisión de conservar `baseline` como predeterminado.

[0043 — Adoptar single-pass epistémico como predeterminado](0043-epistemic-single-pass-default.md) registra la promoción inicial tras V11/V14, posteriormente sustituida por la decisión 0044.

[0044 — Restaurar baseline como predeterminado](0044-restore-baseline-default.md) revierte la promoción tras el contraste completo de 540 preguntas y conserva single-pass como opción experimental explícita.
