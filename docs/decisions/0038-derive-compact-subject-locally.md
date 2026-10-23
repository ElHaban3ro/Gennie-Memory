# 0038 — Derivar localmente el sujeto del ancla compacta

## Contexto

La sonda sintética de V3 del 2026-10-23 conservó `kind_quote` correcto, pero el modelo devolvió `subject_name_quote` con el ID estable incluido. El validador lo rechazó, como exige el contrato. Pedir otra cita libre y redundante añade una oportunidad de error a un nombre que ya aparece delimitado en la consulta.

## Alternativas

- Mantener el campo `subject_name_quote` y seguir ajustando el prompt: descartada por la salida observada y por no resolver la redundancia.
- Recortar el texto sin validar su forma: descartada porque puede convertir relaciones o aclaraciones en sujeto.
- Retirar el campo del wire experimental y derivar localmente el tramo entre tipo e ID: elegida solo para esta revisión experimental de V3.

## Motivo y estado

El parser exige tipo e ID literales, únicos y en ese orden, un tramo intermedio único, y una forma léxica conservadora de nombre: hasta seis tokens alfabéticos con mayúscula inicial, partículas de nombre conocidas, y guiones/apóstrofos internos. Separadores narrativos, coordinaciones, puntuación final y spans ambiguos fallan cerrados. La rama V2 y el baseline no cambian. La heurística puede abstenerse ante nombres válidos poco comunes; tampoco constituye verificación semántica de identidad.

Los tests locales cubren partículas españolas, apóstrofos, acentos precompuestos, frases relacionales, coordinación, puntuación, orden y repetición. Pasarlos no habilita adopción: se requiere congelar una revisión/casos con hash nuevo, pasar los tres smokes consecutivos requeridos por el gate experimental y luego superar una cohorte sintética nueva con etiquetas ciegas y baseline emparejado, sin falsos positivos nuevos ni degradación de calidad. Detener la tanda ante el primer fallo y conservar su respuesta sin reparar.

## Condición de revisión

Revisar o retirar el derivador si la cohorte muestra spans mal segmentados, identidades falsas, abstenciones en nombres comunes o coste/latencia que no compense el cambio. No combinar estas observaciones con resultados de anteriores revisiones de V3.

## Primer smoke de esta revisión

El fixture `local-name-span-v1` obtuvo dictamen ciego 2/2 de que era contestable y de que Lund pertenece a Nils Rohe, no a Edda Lind. La primera llamada remota tuvo recibo completo (HTTP 200, DeepInfra; 945/185 tokens; USD 0,00006575; 1.926 ms de clasificación), pero falló antes de seleccionar: el modelo puso el texto de la pregunta en `propositions[].quote`, rechazado por `slot_evidence_proposition_source_quote_invalid`. La puerta se detuvo sin reintentar ni hacer las otras dos llamadas. Véase el [reporte](../evaluation/assets/slot-evidence-v3/local-name-span-v1/smoke-results.md).

La siguiente iteración aclara que las citas de proposición proceden del resumen de memoria, mientras que las citas de slot/premisa proceden de la pregunta; añade un ejemplo local y una aserción de prompt. La prueba focal pasa. Esto es solo revisión del prompt, no evidencia de obediencia ni de mejora del modelo. Su hash cambia, por lo que requiere fixture/manifiesto nuevos y superar de nuevo la puerta completa de tres invocaciones idénticas antes de cualquier cohorte. Baseline y adopción siguen sin cambios.

## Segunda revisión y estado actual

Un caso sintético nuevo fue revisado a ciegas por dos lectores (2/2: Port Arlen; Bellford pertenece a Tomas Rell T-28). Con el prompt actualizado, el modelo produjo la cita de proposición desde el recuerdo, pero usó `from Seagrass Research Pier` —texto de memoria— como cita del slot; el parser rechazó la salida (`slot_evidence_v3_slot_quote_not_literal`). Transporte completo: HTTP 200, 1.070/185 tokens, USD 0,000072, 2.786 ms de clasificación. Se detuvo tras una llamada sin reintento; no hubo selección ni comparación baseline. Véase el [reporte V2](../evaluation/assets/slot-evidence-v3/local-name-span-v2/smoke-results.md).

Resultado: la instrucción nueva corrigió el tipo de cita anterior en esta muestra, pero no la calidad global; persiste la confusión entre citas de pregunta y recuerdo. No se abre cohorte ni se adopta. Para evitar una secuencia de retoques de prompt por errores de campo, la siguiente iteración debe cambiar la representación/mecanismo local de obtención de citas de consulta, con controles de múltiples slots y abstención, antes de otra llamada remota. Baseline intacto.
