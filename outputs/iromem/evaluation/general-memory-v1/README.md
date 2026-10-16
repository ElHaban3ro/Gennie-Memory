# Resultados de recuerdos generales v1

El [informe](../../../../docs/evaluation/general-memory-v1-results.md) explica el diseño, la comparación y sus límites. Todos los sujetos y hechos son sintéticos.

- `manifest.json`: hashes del fixture y reparto por sujetos.
- `retrieval-summary.json`: cobertura y tiempo de las 3.300 búsquedas híbridas.
- `laya-oracle-diagnostic.json`: prueba controlada con el recuerdo correcto presente.
- `laya-development-*` y `laya-final-*`: resumen y decisiones locales por caso.
- `remote-development-*-report.json` y `remote-final-direct-report.json`: decisiones, tiempos, modelos servidos y coste informado por llamada. No contienen credenciales.
- `final-summary.json` y `final-*-seed-summary.json`: métricas emparejadas y fidelidad de semillas.

Los vectores, la base SQLite temporal, los planes y los diarios de petición remota permanecen en `.local/pilots/general-memory-v1/` en esta Mac. El fixture versionado se verifica con `iromem-eval validate-general-memory`; los generadores Python originales fueron retirados y esta campaña híbrida queda como resultado histórico congelado. Las llamadas remotas requieren el diario original para reanudarse sin duplicar intentos.
