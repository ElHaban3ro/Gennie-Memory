# Diagnóstico del smoke r11

- Resultado: `degraded`, fallo cerrado `slot_evidence_query_slot_ids_invalid`, sin IDs seleccionados.
- Búsqueda: candidate recall 1/1; el único recuerdo sintético llegó al clasificador.
- Modelo servido: `google/gemma-3-4b-it`.
- Recibo: 684 tokens de entrada, 265 de salida, US$0,00004061.
- Latencia: clasificador 8386,876 ms; búsqueda 0,428 ms; total servicio 8390,207 ms; total campaña 8390,207 ms.
- El recibo no conserva la salida cruda. El código estable solo demuestra que los IDs de slots están vacíos o duplicados; no se puede distinguir cuál condición produjo el fallo.
- La llamada fue única. No se ejecutaron smokes 2/3 ni TRAIN.
- Revisión local siguiente: instruir IDs cortos, únicos y no vacíos, conservando validación estricta y roster. Requiere nueva revisión del prompt y caso sintético nuevo; no se reintenta.
