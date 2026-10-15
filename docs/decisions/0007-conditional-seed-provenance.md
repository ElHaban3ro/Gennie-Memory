# 0007 — Condiciones y fuente declarada en semillas

Estado: decisión técnica provisional, 2026-10-15. Motivo: el banco v4 medía la selección de ID y hecho, pero no podía verificar si una semilla conservaba las condiciones de aplicación ni su referencia de origen.

Se añadió la migración 4 a `memory_revisions` con `conditions`, `source_kind`, `source_ref` y `source_locator` opcionales. Los fixtures antiguos siguen siendo válidos y sus campos nuevos quedan en `null`. El cargador valida longitud, tipo de fuente y fechas; las búsquedas textual e híbrida y `compose-seed` preservan los metadatos de la revisión vigente. La fuente aparece como **declarada**: un fixture sintético no demuestra que el documento exista ni que su afirmación sea verdadera.

Se versionó v5 con dos recuerdos activos de la misma tarea por proyecto, separados por condición remota/presencial, valores distintos, 1.000 formas de pregunta y etiquetas de hecho, condición y fuente. Los casos sin modalidad decidida se etiquetan como abstención. Un piloto local de 60 casos y cuatro casos remotos por modelo miden el recorrido sin abrir la partición final.

Alternativas consideradas: meter las condiciones solo en `summary` habría impedido medir su preservación estructurada; reescribir la migración 1 habría invalidado bases existentes. Se eligieron columnas por revisión y una migración aditiva. La verificación de fuente real, la semántica de condiciones, el presupuesto de tokens y un contrato de escritura completo permanecen pendientes. Revisar esta decisión al implementar esos componentes o si la evaluación v5 revela que el formato no permite discriminar aplicaciones.
