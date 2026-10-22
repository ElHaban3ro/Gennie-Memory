# Cohorte sintética V7 — paquete de autoría

Estado: datos sintéticos de desarrollo, redactados por un solo autor. Dos revisiones independientes ciegas ya compararon las 64 preguntas contra las fuentes y la clave: concordancia de relación y citas **64/64**. Como la preparación ocurrió antes de superar la puerta de smokes y la matriz previa, V7 queda congelada como exploratoria; no puede ser la cohorte confirmatoria de adopción. La revisión no hace que el banco sea evidencia de conversaciones reales ni acredita calidad del clasificador; véase [el informe](../../../../docs/evaluation/conversation-evidence-v7-blind-review-results-2026-10-23.md).

- Ámbitos de conversación: 8, con 16 recuerdos por ámbito (128 total).
- Preguntas: 64, con 8 por ámbito. IDs y consultas solamente en `blind-questions.json`; la identidad de entidad aparece literalmente en cada consulta y en los mensajes fuente. `scope_id` es metadato de conversación, no identidad.
- Relaciones: 16 `supports_answer`, 16 `refutes_premise`, 16 `pending_or_partial` y 8 casos `explicit_unmeasured`, 8 `unrelated`.
- `memory-corpus.json` conserva la forma de `load-fixtures`; cada resumen es texto fuente literal y `source.locator`/`source.ref` apunta al mensaje. IDs de mensaje y de recuerdo usan espacios de nombres diferentes.
- `answer-key.json` conserva las etiquetas, IDs esperados, relación por evidencia, respuesta esperada, cita literal de fuente, razón y límite de respuesta.

## Protocolo de distribución

Crear una copia limpia para cada revisor con **solo** `source.json` y `blind-questions.json`. No copiar `answer-key.json` ni este README de autoría. El revisor debe registrar sus etiquetas por separado. Congelar primero esos dos archivos y sus SHA-256; mantener la clave separada hasta completar revisión y conciliación. No llamar a proveedores con las etiquetas o la clave.

Hashes SHA-256 (bytes exactos, incluidos saltos de línea):

```json
{
  "source.json": "7132dfc1f33809a86abc586d2cfdb72c686eda1ced8f692b15a21c3b0222a006",
  "memory-corpus.json": "b7564bb398d23c5ce07e728b95aa670e56ba360c01eef92ad52310cc1235ab32",
  "blind-questions.json": "09c777e71cb6a03f773da0cbf826722dcaaa26b5f08b11f2db3a3fbd36f68ffb",
  "answer-key.json": "e202745c880a11a2b9147bf6232866cd2bddd8600b7ed9fc7dbb57c69bef1bfc"
}
```
