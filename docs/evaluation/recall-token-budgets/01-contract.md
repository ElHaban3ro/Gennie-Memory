# 1. Contrato y contador

Estado: implementado. [Plan previo](../../architecture/recall-token-budgets-plan.md), [contrato público](../../architecture/product-contract-v1.md), [decisión 0052](../../decisions/0052-recall-token-budgets.md).

Perfil fijo `o200k_base:tiktoken-rs-0.12.0:ordinary:utf8-json-v1`. Entrada acumulada 16384 y semilla 2048 por defecto, rango 1–65536. Codificación ordinary conserva marcadores especiales como texto, UTF-8, condiciones y negaciones; no interpreta instrucciones del recuerdo. El vocabulario se carga una vez por proceso sin red. Las cotas existentes de bytes/plazo continúan activas.

Pruebas: `token_budget::tests` verifica texto conocido, vacío, marcadores, Unicode, hashes, igualdad exacta de límite y acumulación sin cargar rechazados. `product::tests::invalid_token_budgets_fail_before_classification` valida extremos y `adapters_without_budget_support_fail_closed` demuestra que un adaptador antiguo no recibe la llamada sin límite.

Límite: unidad local de referencia; tokenizer nativo y plantilla de Decisions/Jev no verificados. No sustituye recibos, contexto nativo o dinero. No hay evidencia nueva de relevancia/calidad.
