# Campaña remota v5: registro histórico

La campaña v5 se ejecutó el 2026-10-15 con 2.800 llamadas intentadas, ocho HTTP 504 de OpenAI, ninguna tarea incierta y 1.600 semillas compuestas por modelo. Los [resultados congelados](../../outputs/iromem/evaluation/v5-campaign-remote/README.md), la [muestra sintética](../../tests/benchmark/v5/development-campaign-1600.jsonl) y los manifiestos conservan el método y sus límites.

El runner Python usado entonces fue retirado durante la migración a Rust. Sus comandos de plan, exportación y reintento ya no son ejecutables y **el diario antiguo no debe reanudarse** con `gennie-memory-eval campaign`, que tiene otro formato y mide la ruta rápida actual. Los resultados históricos siguen siendo evidencia de la ruta híbrida original; no se reinterpretan como una prueba del comando Rust nuevo.

Para campañas actuales, usa el [contrato Rust](contract-v1.md): congela casos, binario y base; ejecuta `gennie-memory-eval campaign` con un diario nuevo por proveedor; verifica `started`/`finished` y genera `campaign-report` solo al terminar. Una llamada incierta no se repite automáticamente. La [medición Rust final](rust-migration-v1.md) documenta la primera ejecución completa de esta ruta con Decisions y Jev, junto a los datos por caso.
