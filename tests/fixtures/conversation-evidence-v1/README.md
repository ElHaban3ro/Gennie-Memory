# Desarrollo sintético de pertinencia v1

Ocho conversaciones 034–041, 64 recuerdos y 64 preguntas: 16 de respuesta, 16 de refutación, 16 de contexto parcial y 16 de ausencia. Fuente y etiquetas redactadas por un agente, sin revisión independiente ni humana. No es reserva ni validación general.

`source.json` conserva frases fuente; `memories.json` usa esas frases sin compresión; `development.jsonl` contiene consultas y anotaciones. Los IDs `cev1_<conversación>_<frase>` y `cev1q_<conversación>_<caso>` son estables. Las listas de corpus y casos se mezclaron con semillas fijas 1741 y 1742, respectivamente. Ningún ID esperado se calcula con una puntuación de recuperación. Los archivos JSON congelados son la referencia; no se requiere un generador Python.

La [rúbrica](../../../docs/evaluation/evidence-rubric-v1.md) define las cuatro relaciones, lo afirmable y las limitaciones. `expected_ids` incluye evidencia correctiva o parcial, no solo respuestas completas. Las citas y fuentes son sintéticas. `agent-review.md` documenta la revisión del propio autor sin atribuir independencia.

Validar: `cargo run --offline -p iromem-eval -- validate-evidence-benchmark tests/fixtures/conversation-evidence-v1`. Para obtener cobertura por relación de una búsqueda congelada: `report-evidence-retrieval <banco> <search.jsonl> <salida-nueva.json>`; exige el manifiesto producido por `search-benchmark`. No ejecuta proveedor ni modifica el producto.
