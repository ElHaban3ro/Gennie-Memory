# Ablación retrospectiva de recorte por oración

Fecha: 2026-10-23. Estado: análisis local de desarrollo; no cambia el selector ni la política predeterminada.

## Pregunta

¿Basta con dividir cada recuerdo por puntuación terminal y conservar las oraciones que contienen el ID estable preguntado para evitar que la semilla mezcle un dato útil con hechos de otra persona?

## Método

Se usaron los artefactos congelados de `smoke-14` y las 14 entradas de `tests/fixtures/slot-evidence-v3-attribution-blind-v1.json`. Para cada consulta se extrajo el ID estable escrito literalmente en ella; luego se dividió `summary` después de `.`, `!` o `?`, y se conservaron solo las oraciones que contienen ese ID. La regla no consulta etiquetas, relaciones ni IDs gold para decidir qué conservar. No se alteraron los fixtures ni se llamó a ningún proveedor.

## Resultado

- En `smoke-14`, el resumen tiene dos oraciones. La regla conserva `Engineer Marta Solis M-58 currently works from Valencia.` y descarta la oración de Teo Rivas. Para este ejemplo consigue separar el dato revisado como útil del distractor.
- En el banco ciego adversarial, **14/14** resúmenes constan de una sola oración que contiene B-29 y también al menos un ID de otra persona. La regla conserva las 14 oraciones completas: **0/14** quedan sin el distractor de otra identidad. La adjudicación previa del banco fue 7 respuestas y 7 casos ajenos; el recorte no decide cuál relación es correcta ni resuelve los siete casos ajenos.

## Decisión y límites

Se descarta el recorte por oración como mecanismo general de atribución: su acierto 1/1 en `smoke-14` no se reproduce en las construcciones de una sola oración del banco. No se promueve ni se incorpora a `recall`. El experimento tampoco invalida un análisis de cláusulas: no lo implementa ni lo mide. La siguiente prueba local debe trabajar sobre el alcance de sujeto/predicado que ya valida `slot_evidence`, y demostrar separación sin perder las 7 respuestas antes de abrir otra comparación remota. No se deben recalcular etiquetas ni llamar a esto validación nueva.

Coste API: USD 0. Latencia de modelo: no medida. La evidencia de entrada permanece en [smoke 14](assets/slot-evidence-v3/smoke-14/) y en la [adjudicación ciega del banco](slot-evidence-v3-blind-adjudication-2026-10-23.md).
