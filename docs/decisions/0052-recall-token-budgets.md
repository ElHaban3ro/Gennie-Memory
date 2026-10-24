# 0052 — Presupuestos locales de tokens en recall-v1

Fecha: 2026-10-24. Estado: implementada; verificación registrada en el informe enlazado.

Contexto: las cotas de bytes y plazo no limitaban tokens de las peticiones ni de la semilla. Decisions y Jev usan protocolos personalizados de OpenRouter cuyo tokenizer nativo no está verificado. La decisión del usuario conserva E5 como investigación y centra el producto en FTS5.

Decisión: usar `o200k_base` ordinary mediante tiktoken-rs fijado a 0.12.0, con vocabulario local, como unidad reproducible de aplicación. Contar JSON exacto antes de cada POST, acumular clasificación/revisión y contar texto de semilla después de componerlo. Valores por defecto 16384/2048, rango 1–65536; bytes y plazo siguen independientes. No alterar prompts, políticas o recuerdos para hacerlos caber. Mantener consumo del proveedor separado.

El adaptador sin soporte falla cerrado. Un rechazo de la revisión conserva selección y recibo principales con degradación explícita; un rechazo de semilla conserva selección y omite todo el texto. El recibo de un envío fallido sigue siendo desconocido si no fue comunicado. La admisión local no prueba envío ni coste. La ruta experimental heredada conserva compatibilidad para reproducción. Se corrige además la marca de inclusión: los equivalentes solo figuran incluidos si se publica su semilla.

Coste: nuevo vocabulario/tokenizador local con inicialización única y trabajo CPU por petición/semilla; no hay segundo servicio o inferencia. La dependencia directa se verificó en [crates.io](https://crates.io/crates/tiktoken-rs/0.12.0) (no retirada, MIT, Rust >=1.85) y [docs.rs](https://docs.rs/tiktoken-rs/0.12.0/). La revisión del directorio RustSec de la dependencia devolvió 404; no es auditoría integral de transitivas ni garantía de seguridad. La implementación de referencia de [OpenAI tiktoken](https://github.com/openai/tiktoken) describe o200k_base, sin acreditar su equivalencia con estos modelos personalizados.

Alternativas: contar solo consulta/candidatos omite esquema e instrucciones; bytes no son tokens; consultar un endpoint remoto de otro proveedor no mide este protocolo y añade red/coste. Adoptar tokenización nativa exige fuente verificable y una nueva versión del perfil. Este paso no verifica fuentes ni calidad/relevancia semántica y no cierra todo recall.

[Plan](../architecture/recall-token-budgets-plan.md), [contrato](../architecture/product-contract-v1.md), [pruebas y límites](../evaluation/recall-token-budgets/README.md).
