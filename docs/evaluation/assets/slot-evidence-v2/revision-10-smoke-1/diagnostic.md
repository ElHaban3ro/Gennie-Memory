# Diagnóstico del smoke r10

- Resultado: fallo cerrado `slot_evidence_query_target_quote_invalid`; no seleccionó recuerdos.
- Modelo servido: `google/gemma-3-4b-it`.
- Recibo: 611 tokens de entrada, 359 de salida, US$0,00004239.
- Latencia: clasificador 2701,696 ms; búsqueda 0,428 ms; total del servicio 2703,252 ms; total campaña 2704,335 ms.
- El recibo no conserva la respuesta estructurada cruda. La traza tenía una sola consulta con un slot solicitado y cero premisas; por ello el código `query_target_quote_not_literal` indica que la cita del slot no coincidió literalmente. No se puede determinar si cambió capitalización, puntuación o palabras.
- La campaña no repitió la respuesta. No se ejecutaron smokes 2/3. La observación pertenece a `smoke`, no a TRAIN.
- Corrección local siguiente: reforzar únicamente la instrucción de copiar citas de slot/premisa byte por byte y con capitalización/puntuación originales. El validador sigue exacto. La nueva instrucción recibió crítica estática independiente favorable y tiene prueba local.
