# Revisión 32 — smoke sintético de parser y ruteo

Caso nuevo, distinto de r31 en identidad, valor, ámbito y consulta. La búsqueda local recuperó el único candidato correcto. Una revisión ciega de consulta y recuerdo confirmó que responde directamente, sin ver la etiqueta esperada.

La única llamada ejecutada obtuvo HTTP 200 de DeepInfra, pero falló al convertir el span de sujeto a un identificador delimitado completo (`slot_evidence_v2_subject_quote_not_in_query`). Coste USD 0,00010765; 1.629 tokens de entrada, 262 de salida; 5.643,936 ms de clasificador. El cuerpo bruto no se conserva; el texto exacto propuesto es desconocido. Se detuvo la secuencia sin reintentar; smokes 2 y 3 no se enviaron.

La revisión no pasa el gate de tres smokes. En una nueva versión, el prompt indicará explícitamente los límites de token requeridos y el parser distinguirá este error. Esto no cambia ni explica el resultado congelado r32. Los hashes de origen del manifest son los del binario ejecutado antes de esa revisión.
