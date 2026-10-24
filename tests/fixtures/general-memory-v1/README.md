# Banco sintético de recuerdos generales v1

Este banco amplía **los ejemplos**, no la funcionalidad medida del recuperador. Contiene 2.800 recuerdos activos y 3.300 consultas etiquetadas en cuatro ámbitos: `user` (1.000), `project` (600), `conversation` (600) y `agent` (600). Hay 2.800 consultas positivas con un recuerdo esperado y 500 negativas que deben quedar sin semilla. Otras 140 consultas sin ámbito explícito ilustran la resolución de contexto que **todavía no implementa** Gennie Memory.

Los doce tipos de recuerdo incluyen identidad, hechos, preferencias, relaciones, rutinas, objetivos, decisiones, procedimientos, restricciones, observaciones, eventos y correcciones. Los temas abarcan nombres, idioma, accesibilidad, comunicación, reuniones, viajes, trabajo, hitos, riesgos, documentación, acuerdos temporales y actividad de agentes. Todas las personas, proyectos, conversaciones, agentes, fuentes y respuestas son ficticios. Las fuentes `synthetic://` son declaraciones de fixture, no documentos comprobados.

`conversation` significa recuerdos cuyo contexto pertenece a **una conversación concreta**: qué se acordó, corrigió, descartó o dejó pendiente allí. `agent` significa recuerdos del **estado operativo de un asistente ficticio**: qué herramienta tenía, qué operación completó, qué error observó y cuál era su siguiente comprobación. Ningún dato de `agent` concede capacidades ni autoridad al asistente real; incluso las preferencias de perfil son hechos sintéticos recuperables, no instrucciones que sustituyan las reglas del sistema o del usuario.

## Archivos

- El generador Python histórico fue retirado. Los datos congelados se verifican con `cargo run --offline -p gennie-memory-eval -- validate-general-memory tests/fixtures/general-memory-v1`; no se promete regeneración byte por byte desde las plantillas originales.
- `memories.json`: lista compatible con `load-fixtures` de la CLI actual.
- `queries.jsonl`: consultas con `scope_kind` y `scope_id` ya dados, `expected_memory_ids` y etiqueta positiva o negativa. Son etiquetas de referencia para un evaluador futuro; `search-batch` no consume este formato directamente.
- `unscoped-queries.jsonl`: preguntas como «¿cómo me llamo?» con `host_context`, ámbito y recuerdo esperados. Requieren un componente futuro que seleccione ámbitos y preserve permisos. La CLI actual exige que se le indique un único ámbito.
- `manifest.json`: conteos y categorías para revisar la generación.

## Alcance

El banco mezcla muchos temas y ámbitos, pero conserva plantillas deterministas; no equivale a preguntas humanas ni valida precisión. Tampoco incorpora embeddings, revisiones históricas, identidades ambiguas o verificación real de fuentes. Se ejecutó una [campaña sintética de búsqueda híbrida y clasificación](../../../docs/evaluation/general-memory-v1-results.md) sobre estos ejemplos; sus resultados son independientes de la matriz anterior.
