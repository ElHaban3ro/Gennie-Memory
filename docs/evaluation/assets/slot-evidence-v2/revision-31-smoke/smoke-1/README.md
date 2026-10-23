# Smoke 1 — fallo conservado

La request sintética obtuvo HTTP 200 de `google/gemma-3-4b-it`, servida por Parasail. FTS recuperó 1/1 candidato. La salida fue rechazada localmente con `slot_evidence_v2_subject_not_literal`; no hubo selección. El recibo indica 1.629 tokens de entrada, 250 de salida y USD 0,00007766. Latencia del clasificador 2.412,441 ms; total 2.413,385 ms.

La respuesta bruta no se guardó, conforme al diseño; el hash de respuesta se conserva en `evaluation-capture.jsonl`. Por ello no es posible saber retrospectivamente cuál de los controles internos devolvió el código compartido. El resultado r31 queda inalterado y no se reintenta. Smokes 2 y 3 no se ejecutaron por la regla de detenerse en la primera salida inválida.
