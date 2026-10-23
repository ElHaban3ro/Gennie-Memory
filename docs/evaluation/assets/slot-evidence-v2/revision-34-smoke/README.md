# Revisión 34 — smoke sintético fallido

Caso nuevo, con revisión ciega sin etiquetas y FTS top-1 1/1. La revisora dictaminó respuesta directa: “Surveyor Elena Park Q-62 is assigned to Juniper Annex.”

El único HTTP recibió 200 y sirvió `google/gemma-3-4b-it`; la clasificación falló cerrada con `slot_evidence_v2_kind_range_splits_identifier`. Uso: 1.079/250 tokens, coste reportado USD 0,00005566; clasificador 2.536,719 ms, total 2.539,884 ms. No se seleccionó ningún candidato. No se hicieron reintentos ni llamadas restantes. Se conserva la respuesta normalizada, recibo/hash, búsqueda, inputs congelados y crítica ciega. Raw del proveedor no conservado.

Este fallo muestra que eliminar solo los offsets del sujeto era insuficiente: también fallaba el índice generado para el tipo. La siguiente revisión elimina índices de todos los textos copiados desde la consulta: tipo, sujeto, slots y premisas. Rust exige una única cita literal y delimitada; no amplía spans ni corrige citas.
