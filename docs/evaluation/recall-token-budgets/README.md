# Presupuestos de tokens de recall-v1

Fecha: 2026-10-24. Alcance: subpaso de completar recall sobre FTS5. E5 permanece como investigación. Los límites se aplican a `recall-v1` y session `recall_v1`; la ruta heredada `recall` conserva el protocolo experimental para reproducir campañas.

| Paso | Implementación y evidencia |
| --- | --- |
| 1 | [Contrato y contador local](01-contract.md) |
| 2 | [Entrada completa y acumulación](02-input.md) |
| 3 | [Semilla íntegra y omisión segura](03-seed.md) |
| 4 | [Recorrido CLI, sesión y coste local](04-cli.md) |
| 5 | [Regresiones y cierre acotado](05-verification.md) |

Valores predeterminados: **16384 tokens de entrada acumulada**, **2048 de semilla**, configurables entre 1 y 65536 por `budgets.input_tokens` y `budgets.seed_tokens`. Se cuenta el JSON exacto antes de cada envío y la semilla final después de componer IDs, fuentes, condiciones y aviso de equivalencias. Un exceso no recorta recuerdos: rechaza la llamada o toda la semilla, conservando recuperación y selección disponibles. La segunda revisión puede omitirse con degradación explícita sin perder la selección inicial ni inventar un segundo coste.

Unidad local `o200k_base`, **no tokenizer nativo verificado de Decisions/Jev**. `token_accounting` y los recibos de `usage` tienen significados distintos. La primera carga local es apreciable: 525–676 ms en dos operaciones frías; siguientes operaciones de rechazo en sesión, 1,21–1,87 ms. No es un benchmark de inferencia o recall exitoso. [Datos crudos](cli-measurements.json).

Este cambio no verifica fuentes, aplicabilidad independiente, relevancia calibrada o integración Codex y no cierra todo el hito de recall. No se ejecutó inferencia pagada, se usaron bases temporales y mocks propios. No se hizo commit ni push. [Contrato público](../../architecture/product-contract-v1.md), [decisión 0052](../../decisions/0052-recall-token-budgets.md).
