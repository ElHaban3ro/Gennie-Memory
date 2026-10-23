# Iteración de prompt V3 para extracción de tipo literal

Fecha: 2026-10-23. Estado: evaluación negativa; V3 sigue experimental y no se adopta.

## Diseño

Se preparó una cohorte sintética prospectiva nueva de nueve preguntas: siete contestables y dos distractores de sujeto. Dos revisores ciegos recibieron solo preguntas y resúmenes, sin etiquetas, hipótesis ni resultados; coincidieron en los 9/9 juicios. Las entradas, corpus, etiquetas adjudicadas y resultados de búsqueda local quedaron congelados antes de la campaña. SHA-256 de `cases.jsonl`: `cb7cf3ef281557777b60a57fff212d200eda142edca287cfc6c36684faf0b380`; SHA-256 de `corpus.json`: `a91c66a4fa7e437410144a02915151551f50e03bacab10f8bbb6d1067037a8e6`.

La búsqueda local encontró un único candidato esperado en 9/9. El baseline y V3 recibieron consultas/corpus emparejados. V3 tiene una puerta preregistrada de tres smokes consecutivos; al fallar el primer caso se detuvo esa ruta, sin reintentos ni llamadas adicionales. El baseline completó los nueve para medir el comportamiento de referencia. El prompt recibió un ejemplo que exige copiar literalmente el tipo con capitalización de la consulta; la cohorte de validación no contiene ese ejemplo.

## Resultado

- **Baseline:** 8/9 selecciones exactas; 7/7 positivos; abstuvo en 1/2 negativos y seleccionó indebidamente el recuerdo de Vera/Ivo en `prompt-revision-05`. Cero respuestas degradadas.
- **V3 con prompt revisado:** 0/1; el primer positivo terminó degradado con `slot_evidence_v2_kind_not_literal`, sin selección. La puerta se cerró y no se ejecutaron los otros ocho casos con V3.
- **Coste informado:** baseline USD 0,00006473; V3 USD 0,00008845; total USD 0,00015318 en 10 peticiones. Tokens: 1.221 entrada/54 salida para baseline y 1.121 entrada/324 salida para V3. Latencia del clasificador baseline p50 825,96 ms, p95 1.215,00 ms; servicio total p50 829,92 ms, p95 1.216,97 ms. V3 tardó 4.854,41 ms en clasificar y 4.855,66 ms total.

La salida JSON cruda del proveedor no se conservó, así que el código de error no permite distinguir capitalización distinta de otra cita de tipo inválida. La causa concreta queda sin verificar. El ejemplo añadido al prompt no evitó este fallo. No se cambia el contrato literal ni se ajustan las etiquetas retrospectivamente.

## Decisión

No hay evidencia de mejora de calidad: V3 falló en el único caso ejecutado y el baseline mantiene un falso positivo de sujeto en una pregunta negativa. Se conserva el baseline. No se repiten estas consultas ni se adopta V3. El siguiente experimento requiere preservar de forma segura la respuesta estructurada cruda (sin secretos) y comprobar en un caso nuevo si el error proviene de capitalización; cualquier recuperación local deberá seguir validando que el fragmento corresponde de manera única a la pregunta y conservar exactamente el texto original. Esta campaña por sí sola no autoriza relajar esa condición.

Artefactos: `assets/slot-evidence-v3/prompt-revision-v1/` (`cases.jsonl`, `corpus.json`, `adjudication.json`, `manifest-preflight.json`, `search-results.manifest.json`, `baseline-report.json`, `baseline-journal.jsonl`, `v3-journal.jsonl`). El journal V3 conserva el error, uso de tokens, coste y latencia; no conserva la salida cruda.
