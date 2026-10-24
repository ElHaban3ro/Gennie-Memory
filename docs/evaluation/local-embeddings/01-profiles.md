# 1. Perfiles y artefactos

La comparación se congeló en `tests/fixtures/local-embeddings-v1/suite.json` antes de inferir: 25 recuerdos, 25 consultas positivas y cuatro ajenas. Ocho positivas cruzan idiomas, nueve son españolas, siete inglesas y una exige recuperar el final de un documento largo. Las etiquetas pertenecen al autor; no hay revisión independiente.

| Perfil | Revisión | Dimensión / límite | Licencia |
| --- | --- | --- | --- |
| multilingual-e5-small | `614241f622f53c4eeff9890bdc4f31cfecc418b3` | 384 / 512 | MIT |
| paraphrase-multilingual-MiniLM-L12-v2 | `e8f8c211226b894fcb81acc59f3b34ba3efd5f42` | 384 / 128 | Apache 2.0 |

Fuentes primarias: [E5](https://huggingface.co/intfloat/multilingual-e5-small), [MiniLM](https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2), [Candle BERT](https://docs.rs/candle-transformers/0.11.0/candle_transformers/models/bert/index.html), [tokenizers](https://docs.rs/tokenizers/0.23.2/tokenizers/). Versiones, licencia, compatibilidad y alcance de la revisión de seguridad: [decisión 0051](../../decisions/0051-independent-local-embeddings.md).

Los dos pesos descargados coinciden con el SHA-256 LFS publicado oficialmente; configuración y tokenizer tienen hashes locales fijados en los settings. [Artefactos E5](e5-small-artifacts.json) · [Artefactos MiniLM](minilm-artifacts.json). Los artefactos viven bajo `.local/embedding-models/`; no se modificó el almacén compartido de AIModels.

Puerta fijada: E5 con al menos 90% top-5, sin pérdida frente al control MiniLM y recuperación del texto largo; en empate, mayor ventana. Ambos perfiles lograron 25/25 top-5 y 24/25 top-1 en búsqueda semántica. Se selecciona **E5** conforme a ese desempate. Sus 53 fragmentos cubren ambos campos de los 25 recuerdos; MiniLM necesita 64. El original largo se divide en cuatro frente a quince fragmentos. No se seleccionó un umbral a partir de los cuatro negativos.
