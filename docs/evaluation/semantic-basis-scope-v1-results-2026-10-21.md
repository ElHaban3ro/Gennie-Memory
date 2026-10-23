# Fundamento y alcance F1: ronda parcial detenida por límite

Paso 3 del objetivo activo. **24 operaciones intentadas: 20 válidas, cuatro HTTP403 sin recibo y 22 no enviadas. Solo hay 80 dimensiones comparables de 184 previstas. No se acredita calidad completa ni se reanuda el diario.** [Protocolo](semantic-basis-scope-v1-protocol-2026-10-21.md), [reporte Rust](../../outputs/iromem/evaluation/semantic-basis-scope-v1-2026-10-21/report.json), [diagnóstico](../../outputs/iromem/evaluation/semantic-basis-scope-v1-2026-10-21/provider-limit-diagnosis.json).

## Implementación y ejecución

F1 conserva todos los inputs/IDs y etiquetas congeladas, parámetros, modelo/ruta Opus, mensaje de datos, esquema y validador V1. Añade exclusivamente una lista general al system para fundamento de evidencia, cobertura semántica y alcance de flujo, sin respuestas por caso. La crítica estática pidió permitir soporte por paráfrasis/contexto y no exigir terminología literal; se incorporó antes de inferencia. Después no encontró bloqueos. Pasaron 509 pruebas con/sin HTTP2 en la fuente ejecutada, y el preflight reprodujo preparación, anotaciones, mapa/paquete y payloads con fuentes/binario ligados.

La ronda se detuvo tras cinco lotes válidos y un lote con cuatro rechazos del proveedor. OpenRouter informó límite mensual de clave USD 5 y uso USD 5,34162262, reinicio anunciado 2026-11-01T00:00:00.000Z. Ese contador puede incluir consumo ajeno al libro de este chat; no se iguala a su coste conocido. Los HTTP403 no incluyen usage. No se transforman en coste cero ni en errores semánticos. El runner drenó y persistió el lote y detuvo el siguiente; no hubo reintentos, fallback ni cambio del límite por el agente.

| Medida | Resultado |
| --- | --- |
| Planificadas / iniciadas / terminales | 46 / 24 / 24 |
| Válidas / HTTP403 / no enviadas | 20 / 4 / 22 |
| Dimensiones concordantes / comparadas / previstas | 78 / 80 / 184 |
| Controles fieles conservados en subconjunto válido | 15/15 |
| No aceptables bloqueados en subconjunto válido | 5/5 |
| Entrada / salida conocidas | 190.267 / 14.224 tokens |
| Coste nuevo conocido / operaciones sin recibo | USD 1,045548 / 4 |
| Intento terminal p50 / p95; muestra de 24 | 8.343 / 14.121 ms |
| Duración del runner | 64,712 s |

Los 20 inputs válidos ya tenían las mismas aceptaciones con V5. F1 corrige dos dimensiones: completitud temporal en s001 y preservación del calificador en s019. Siguen dos discrepancias de fidelidad/atribución en s019. Ninguno de los cuatro casos que V5 aceptó indebidamente fue probado válidamente: s021 recibió HTTP403 y s029/s039/s040 no se enviaron. El cero de aceptaciones indebidas del subconjunto no demuestra que se hayan corregido esos fallos. [Comparación parcial por ID](../../outputs/iromem/evaluation/semantic-basis-scope-v1-2026-10-21/partial-historical-comparison.json).

Un crítico posterior recibió las 80 razones junto con 184 de otro candidato anónimo, sin referencias, scores ni identidad. Señaló los dos problemas de fidelidad/atribución en s019. Veía predicciones: no es predicción ciega ni revisión humana. No se repararon respuestas, etiquetas o métricas. [Revisión](../../outputs/iromem/evaluation/semantic-codex-agents-v1-2026-10-21/rationale-critical-review.json).

## Coste y cierre

La estimación anunciada era USD 2,55 y 3–8 minutos para hasta 46 HTTP. La ilustración conservadora de bytes como tokens y salida máxima era USD 12,42, sin garantía de factura. Solo 20 respuestas tienen recibos: USD 1,045548. Las cuatro operaciones rechazadas no se contabilizan como no enviadas ni se les inventa recibo. El GET público de catálogo se registra aparte. No se afirma negociación HTTP2 observada ni latencia de recall/Codex/producción.

El usuario informó después que amplió el límite de la clave y precisó la política: API para Decisions, Jev y otros clasificadores y sus comparativas; LLM potentes/de frontera mediante subagentes Codex. No se verificó aquí el nuevo saldo ni se cambió la clave por parte del agente. El runner histórico de estos pilotos LLM queda detenido antes de cargar secretos. F1 permanece cerrado y no se volverá a ejecutar por Opus. [Continuación dentro de Codex](semantic-codex-agents-v1-results-2026-10-21.md).

Acumulado proveedor tras F1: 6.129 operaciones/23 campañas emparejadas completas, USD 4,565416508 conocidos más 20 operaciones históricas sin coste documentado; excluye Codex. Los cuatro bancos manuales, diez informes de extracción y reportes anteriores se conservan; F1 se reproduce en bytes con el evaluador actual. La fuente ejecutada está archivada y difiere del código posterior del reporte/guardia. Baseline y V6 intactos. Sin promoción; objetivo completo aún activo.
