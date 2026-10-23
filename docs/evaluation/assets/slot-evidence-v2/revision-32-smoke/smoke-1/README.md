# Smoke 1 — fallo conservado

HTTP 200; modelo servido `google/gemma-3-4b-it`, proveedor `DeepInfra`. FTS recuperó 1/1. La clasificación falló antes de seleccionar con el código congelado `slot_evidence_v2_subject_quote_not_in_query`; recibo de 1.629 tokens de entrada, 262 de salida, USD 0,00010765. Latencia del clasificador 5.643,936 ms; total 5.645,934 ms.

El cuerpo bruto no se retuvo; solo se guardó su hash. El código entonces usado asignaba ese motivo cuando el span reconstruido no correspondía a un identificador delimitado completo. No conocemos el fragmento exacto que propuso Gemma. Smokes 2/3 detenidos y sin reintento. La misma salida se conserva en `recall-output.json` y el recibo en `evaluation-capture.jsonl`.
