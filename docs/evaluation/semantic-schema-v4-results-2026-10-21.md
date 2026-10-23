# Esquema estricto V4: rutas válidas, citas no literales

Paso 3 del objetivo activo. **Cuatro respuestas cumplen el esquema de rutas, pero tres incumplen las citas literales. Solo una pasa el contrato completo; 42 entradas quedan sin enviar. No hay pase semántico ni mejora de recuperación.** [Protocolo](semantic-schema-v4-protocol-2026-10-21.md), [reporte Rust](../../outputs/iromem/evaluation/semantic-schema-v4-2026-10-21/report.json), [diagnóstico](../../outputs/iromem/evaluation/semantic-schema-v4-2026-10-21/failure-diagnosis.json).

V4 cambia únicamente response_format respecto a los 46 payloads V3 congelados: json_schema estricto, objetos cerrados y enums de referencias mostradas por lado. Mensajes, modelo GPT-4.1, ruta OpenAI, parámetros, inputs y anotaciones V1 permanecen idénticos. Se conserva la traducción exacta y checked V1. El crítico estático no encontró bloqueos; no revisó datos ni ejecutó red. Pasaron 506 pruebas con/sin HTTP2 en la fuente ejecutada. La reconstrucción previa a HTTP ligó fuentes, ejecutable, payloads, paquete/mapa y anotaciones al padre.

| Medida | Resultado |
| --- | --- |
| Previstas / iniciadas / HTTP200 terminales | 46 / 4 / 4 |
| Esquema estricto conforme / contrato completo válido | 4 / 1 |
| Respuestas con citas no literales / no enviadas | 3 / 42 |
| Dimensiones comparables / previstas | 4 / 184 |
| Discrepancias en el único control válido | 0 |
| Entrada / salida informadas | 22.376 / 2.745 tokens |
| Coste nuevo / operaciones sin coste conocido | USD 0,066712 / 0 |
| Intento terminal p50 / p95; muestra de cuatro | 4.535 / 5.754 ms |
| Duración del runner | 5,772 s |

Se validó todo el subconjunto de JSON Schema que usa el payload: objetos, arrays, strings, enum, required, additionalProperties y referencias locales. Las cuatro salidas usan rutas declaradas y objetos conformes. Esa comprobación no valida significado. El validador semántico original rechaza tres respuestas porque presentan fragmentos del JSON propuesto/normalizado como si fueran citas del texto original. No se corrigieron las citas ni se puntuaron respuestas inválidas como aciertos o abstenciones fiables.

La respuesta s001 distingue parcialmente ayer/hoy, pero una de sus citas proviene del campo estructurado, no del original. s002 llama defectos a renombrados documentados del normalizador —described_fact→description y codificación de entidades— que el contrato permite. s000 cuestiona además una relación identificadora del ejemplo. Son observaciones del autor sobre respuestas inválidas, no nueva revisión ciega ni puntuación recalculada. Solo s003 tiene cuatro juicios válidos concordantes; no se presenta ese subconjunto como 100 % de calidad.

La consulta pública confirmó response_format y structured_outputs antes de la estimación anunciada USD 0,85/1–3 minutos. La ilustración conservadora de bytes más salida máxima era USD 2,96, sin garantía de factura. Se realizaron cuatro inferencias, un GET de catálogo y un GET posterior de descubrimiento de modelos separado; sin reintentos. Coste de las cuatro respuestas USD 0,066712, excluye Codex. Acumulado al terminar V4 y antes del posterior V5: 6.059 operaciones, USD 1,194732508 conocidos y 16 operaciones históricas de coste desconocido, 23 campañas emparejadas completas. Los tokens/latencias son del verificador parcial; no miden recall, Codex ni tubería completa. Alias solicitado/servido GPT-4.1; revisión interna no comprobada y negociación HTTP2 no observada.

La siguiente prueba separada V5 conserva mensajes/esquema y contrasta otro perfil de capacidad; no reanuda V4 ni supone que un esquema correcto resuelva semántica. Los resultados anteriores, baseline y V6 permanecen intactos. Sigue pendiente superar capacidad completa, extracción/tubería, conocidos100/126, recursos/estabilidad y cohorte nueva ciega antes de adoptar cualquier mecanismo. No se actualizan gráficas de producto con este intento parcial.
