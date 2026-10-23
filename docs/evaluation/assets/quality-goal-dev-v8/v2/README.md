# Cohorte v8 — revisión ciega previa a cualquier inferencia

Iteración v2 del paquete de desarrollo. Preserva la primera autoría y sus dictámenes bajo `../`; la versión anterior y sus dictámenes ciegos se conservan en [el reporte de revisión](../v1-blind-review.md) porque cuatro consultas supuestamente irrelevantes contenían negaciones explícitas de que el registro tuviera el dato. Dos revisores ciegos las trataron como evidencia parcial o de estado, no como irrelevancia. En esta revisión se sustituyen por preguntas distintas cuyos datos no aparecen en los mensajes; no se cambia después de ver salidas de modelos (no se han ejecutado).

Alcance actual: 4 conversaciones, 24 mensajes/recuerdos, 16 preguntas; balance previsto de 4 casos por relación. Entregar a dos revisores copias de `source.json` y `blind-questions.json` únicamente. No entregar `answer-key.json`, README ni hashes de la clave. Esperar y conciliar los dictámenes antes de congelar el banco. El conjunto sigue siendo sintético y de desarrollo; no es holdout ni evaluación de producto.

Hashes SHA-256 (bytes exactos):

```json
{
  "source.json": "61e4e2d8ceaf3a70ad0fedc49d0113a6c18b36180631060011bae3f5a459cf7c",
  "memory-corpus.json": "2cb010b064b66cc6a7b4f9dfa72599961d85e037ad36544e5bbe32720b4ef6c8",
  "blind-questions.json": "97267d550d857df1b842df0ff0858768ca2e80b7a56d2a16e87b062ce3298fad",
  "answer-key.json": "526523b22ec29dca79edc174cc0573a3625eb17ea43c22b9198e4762bf78715b"
}
```
