# Protocolo V1 de selección de estado parcial en una llamada

Estado: protocolo congelado antes de llamar al proveedor. V10 es una cohorte de desarrollo nueva y sintética. No es reserva confirmatoria.

## Hipótesis y mecanismo

V9 mostró que la cascada rescata dos parciales pero incumple coste y p95 por llamadas adicionales. Esta prueba elimina la cascada: una sola elección tipada usa una instrucción que distingue referente/atributo antes de utilidad y que permite un valor provisional explícito como evidencia parcial, nunca como resultado confirmado. No hay preselección, reintentos, fallback ni llamadas condicionales.

Instrucción de la candidata:

> Select the one saved memory that is useful for the user's exact requested fact. First require the same specific referent (including any record, order, sample, person, place, or event ID) and the same requested attribute. A different ID or a different attribute is unrelated, even in the same conversation. Then accept an explicit direct answer, an explicit correction of a premise about that same referent, or a concrete provisional/estimated/pending value or status for that same requested attribute as partial evidence. Preserve its limitation: an estimate is not a confirmed value, a forecast is not an observed result, and a diagnosis is not a completed repair. A memory about a different attribute is not evidence that the requested fact is absent. Choose none when no memory provides one of those forms of evidence. Never infer or invent missing facts.

## Banco congelado

Dieciséis preguntas V10, cuatro por relación, cuatro conversaciones sintéticas. Revisión ciega antes de llamadas; mismo modelo familiar Codex, no humana. La primera revisión encontró ambigüedad en tres distractores que afirmaban explícitamente que el campo no constaba; se corrigieron antes de inferencia y la segunda revisión confirmó la versión final sin ambigüedad material. Hashes exactos en `assets/quality-goal-dev-v10/hashes.json`; dictamen en `assets/quality-goal-dev-v10/blind-adjudication.md`.

## Puertas previas y de decisión

Primero validar fixtures y búsqueda local sin proveedor; los doce IDs útiles deben aparecer en candidatos y no se debe crear evidencia mediante búsqueda. Luego comparar Decisions `baseline` y `epistemic_partial_single_pass_v1`, con 16 casos cada una, máximo 32 peticiones HTTP y máximo USD 0,003 combinado, sin reintentos. El payload no incluye answer key ni etiquetas.

La candidata solo pasa si logra 16/16 exactas (4/4 apoyo, 4/4 refutación, 4/4 parcial, 4/4 negativos vacíos), no pierde ningún ID correcto que baseline seleccione, no añade ID ajeno, no tiene degradaciones y presenta recibo de coste por caso completo. Coste total y p95 de servicio deben ser ≤1,5× baseline. Cualquier incumplimiento rechaza esta versión sin retoques ni segunda corrida en V10. Un pase solo autoriza diseñar y revisar una cohorte confirmatoria disjunta; no autoriza adopción por sí mismo.
