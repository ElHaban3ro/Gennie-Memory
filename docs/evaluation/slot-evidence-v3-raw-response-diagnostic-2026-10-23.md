# Diagnóstico de respuesta estructurada V3

Fecha: 2026-10-23. Una única petición sintética; no es una métrica de calidad ni una campaña de cohorte.

## Propósito y preparación

Se añadió `GENNIE_MEMORY_EVALUATION_RESPONSE_CONTENT_PATH`, opt-in independiente de la captura de hashes. Con la variable ausente no se guarda texto de respuesta. Cuando se activa, conserva únicamente `choices[0].message.content` completo, hasta 24 KiB, en JSONL nuevo con permisos `0600`; se empareja por SHA-256 de la solicitud. No captura credenciales, cabeceras ni cuerpo HTTP. La ruta solo debe usarse con fixtures sintéticos porque el texto generado puede repetir la entrada. La lectura de configuración, límites, permisos y comportamiento están cubiertos por pruebas.

Dos revisores ciegos recibieron solo la pregunta y el recuerdo sintético y coincidieron en que Mara Sol E-40 trabaja actualmente desde Bergen. Búsqueda FTS5 local encontró el recuerdo esperado como candidato único. Hashes de consulta y corpus: `4ac5d939ab82f643eafaf0a14581f95b218077027ffce93773c7c1d64330dc84` y `7f99ab8691f3469cefa9aa84f2757b7f8dc093f93038e6bf81a09afb4e169560`.

## Resultado observado

La única petición a Gemma 3 4B falló en validación local con `slot_evidence_v3_query_subject_name_quote_contains_stable_id`; no seleccionó ningún recuerdo. El contenido capturado muestra que `kind_quote` era exactamente `technician`, con la misma capitalización que la pregunta. Sin embargo, el modelo puso `Mara Sol E-40` en `subject_name_quote`, aunque el contrato exige solo `Mara Sol`. El fallo ocurrió en ese campo antes de demostrar cualquier problema con la extracción literal del tipo.

Uso/coste informado: 1.112 tokens de entrada, 255 de salida, USD 0,00008110. Latencia del clasificador 3.065,09 ms; operación completa 3.067,65 ms. No se reintentó.

Esta sonda **no explica** el error `slot_evidence_v2_kind_not_literal` de las cohortes anteriores: en este caso el tipo sí era literal, pero la validación se detuvo antes en otro campo. El raw previo no existe, así que no se atribuye ese error a mayúsculas/minúsculas.

## Decisión

La nueva captura permite distinguir los campos defectuosos y confirma que el prompt estricto sigue produciendo anclas inválidas en ejemplos sencillos. No se aceptará ni corregirá silenciosamente `subject_name_quote`: la regla exige una cita literal solo del nombre. Una dirección de desarrollo segura es quitar ese campo redundante del wire experimental y derivar localmente el nombre únicamente cuando `kind_quote` y `stable_id` únicos delimitan un tramo contiguo inequívoco de la propia consulta. Antes de tocar el parser, esa regla requiere tests positivos y negativos para repeticiones, separadores, puntuación y ambigüedad. No se vuelve a llamar a este caso ni se adopta V3.

Artefactos sintéticos y raw: `assets/slot-evidence-v3/kind-diagnostic-v1/`. El archivo crudo contiene solo la respuesta de esta sonda ficticia; mantenerlo local y no añadir datos reales.
