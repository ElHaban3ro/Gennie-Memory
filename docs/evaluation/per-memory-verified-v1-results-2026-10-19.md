# Piloto de propuesta y verificación aislada v1

2026-10-19. **Falla calidad y coste diagnóstico; no se amplía.** Diez preguntas conocidas v5,27peticiones HTTP, USD0,001057618 informados con todos los recibos; baseline predeterminada intacta. No es validación nueva.

Jev propuso positivos entre ocho recuerdos; Decisions verificó cada positivo aisladamente. Cada semilla retiene exactamente la intersección, con recibos por etapa sin duplicar propuesta. Se conservaron todos los ID permitidos elegidos por base en estas consultas. Nueve de diez semillas no incluyen ID ajeno; una sigue incluyendo `cev5_071_03`, presentar carné, que no establece ausencia de avalista. El verificador también lo aceptó. Por tanto, el acierto aislado observado antes en la matriz no garantizó el mismo juicio al volver a medir.

Se rechazaron tres propuestas incorrectas: horario de otra operación, adquisición inferida de recogida y revisión del objeto ajeno. Los cinco casos antes contaminados en V2 quedan reducidos a uno en este piloto, pero la comparación histórica seleccionada no es un ensayo controlado de mejora. Baseline histórica tenía diez semillas permitidas en este subconjunto; la candidata no la mejora.

Coste2,0296× USD0,000521100 de base histórica para estas diez consultas. Mediana de operación823,26ms; no es suma de tiempos de etapas ni recorrido Codex. El precio y tiempos históricos no son un contraste pareado contemporáneo. Tampoco se midió verdad de respuesta final o cobertura de todas las partes solicitadas. No se compensan el ID ajeno ni coste con las propuestas rechazadas.

La auditoría ordinaria pasó sobre consultas y gold originales (sin proyección auxiliar), candidatos, revisiones, fuentes declaradas, semillas y eventos. El evaluador independiente revalida todas las comprobaciones positivas, los modelos y la agregación de recibos. Una revisión crítica separada detectó búsqueda vacía no admitida y fuentes requeridas no obligatorias en preflight: ambas corregidas con regresiones antes del siguiente piloto; no alteran estas diez respuestas.

[Contrato previo](per-memory-verified-v1-contract.md), [diario](../../outputs/iromem/evaluation/per-memory-verified-v1-pilot-2026-10-19/journal.jsonl), [auditoría](../../outputs/iromem/evaluation/per-memory-verified-v1-pilot-2026-10-19/audit.json), [reporte](../../outputs/iromem/evaluation/per-memory-verified-v1-pilot-2026-10-19/report.json). Los archivos de lanzamiento conservan huellas; el evaluador fue endurecido después de las llamadas y sus huellas de análisis se registran aparte. No se reintentó ninguna llamada.
