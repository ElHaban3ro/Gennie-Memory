# 2. Comprobación literal y recibos

`source-verify` / `source_verify` recibe schema_version, actor_id, scope, memory_id y revision. Solo accede a una revisión vigente con ese propietario/ámbito. Usa original_text, no el resumen, contra el snapshot vigente del tipo/ref exactos. Un registro ajeno no satisface la fuente.

Locator admitido `utf8:start:end`: offsets de bytes con fin exclusivo, ambos en fronteras UTF-8 y texto íntegro exactamente igual al original. Sin locator se requiere una ocurrencia única, contando también coincidencias solapadas. No normaliza acentos, espacios o negaciones. Locators opacos como hmid=4 o line=2 quedan no resueltos; no se reinterpretan como rangos.

Estados: verified_literal/literal_match; declared_unverified con source_not_registered, locator_unsupported, source_quote_ambiguous u original_empty; not_provided si falta fuente; rejected con source_quote_not_found, source_quote_mismatch, source_range_invalid o snapshot_integrity_mismatch. Ausencia de snapshot no prueba que la fuente externa no exista. El contenido y la identidad del snapshot se vuelven a comprobar por hash.

Cada resultado tiene perfil, hashes, versión/rango, motivo y checked_at UTC RFC3339 con milisegundos. Los recibos de DB son idempotentes por evidencia observada, conservan resultados anteriores y solo se devuelven después de commit. Reutilizar un recibo no evita comparar el texto actual; checked_at conserva el primer momento de ese mismo resultado. La integridad del JSON persistido se contrasta con el resultado calculado; los flags históricos de memory_source_evidence no se usan como prueba.

Pruebas: Unicode/emoji/rangos/negación, coincidencias solapadas, fuente ausente/ambigua, integridad de snapshot y recibo, revisión vieja, versiones nuevas, rollback de un conjunto que contiene un candidato alterado. Esto comprueba procedencia literal local, no el respaldo semántico del resumen o de condiciones añadidas ni veracidad factual.

Para propuestas aún sin memoria, check_evidence ofrece la misma comparación sobre Connection/Transaction prestada, sin iniciar transacción o guardar recibo propio. Su prueba confirma el uso dentro de una transacción del llamante, ausencia de recibos y separación de actor; la capa de escritura decide su persistencia.
