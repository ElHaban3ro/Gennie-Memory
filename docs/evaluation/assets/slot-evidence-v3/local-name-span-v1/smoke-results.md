# Smoke V1: derivación local del nombre

Fecha: 2026-10-23. Revisión del binario: `1ea174403634ef488490f15e5c8bac01874f7a21483dfd1398cff5cbba41e2aa`. Fixture sintético congelado en `manifest-preflight.json`; dos lectores ciegos confirmaron que la consulta es contestable y que Lund corresponde a Nils Rohe, no a Edda Lind.

## Resultado

La puerta se detuvo en la primera invocación, como estaba previsto. Transporte y recibo fueron completos: HTTP 200, proveedor DeepInfra, modelo solicitado y servido `google/gemma-3-4b-it`, 945 tokens de entrada, 185 de salida, USD 0,00006575, 1.878 ms de transporte y 1.926 ms de clasificación. El recibo indicó 513 bytes de respuesta y cuerpo completo. Los artefactos de captura se crearon con permisos `0600`.

La selección no pasó: `classification_status=failed`, `selected_id=null`, `status=degraded`, `error_code=slot_evidence_proposition_source_quote_invalid`. El contenido capturado muestra la causa observable: el campo `propositions[].quote` copió literalmente la pregunta completa, aunque ese campo debe ser una cita exacta del recuerdo. Por ello no hubo traza puntuable, selección positiva ni prueba de que el selector distinga correctamente el recuerdo ajeno. No se ejecutaron las otras dos llamadas ni hubo reintento.

## Iteración local

Se aclaró prospectivamente en la instrucción compacta que la cita de una proposición debe proceder del resumen del recuerdo, nunca de la pregunta; se añadió un ejemplo contrapuesto y una aserción que protege la distinción. La prueba `compact_prompt_uses_literal_quotes_without_offsets` pasa. `cargo fmt --all` pasa. Es evidencia de consistencia local del prompt, no de que el modelo obedezca la instrucción.

La nueva instrucción cambia el hash del prompt y constituye otra revisión experimental. Este resultado no se mezcla con las 14 revisiones V3 anteriores ni con la revisión V1 registrada aquí. Hace falta un fixture y manifiesto nuevos, revisión ciega independiente y repetir la puerta completa de tres invocaciones idénticas, deteniéndose ante cualquier fallo. No se modifica el baseline ni se abre una cohorte comparativa.

## Coste

- Esta invocación fallida: USD 0,00006575; 1.130 tokens; 1.926 ms de clasificación.
- Tres invocaciones previstas: coste no incurrido; dos llamadas canceladas por la regla de parada.
- Acumulado de esta nueva revisión: USD 0,00006575, 1 llamada.
- Historial V3 anterior: separado; no sumado a este resultado.
- Coste de trabajo Codex: no disponible.
