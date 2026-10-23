# Coordenadas mostradas V3: capacidad todavía no demostrada

Paso 3 del objetivo activo. **El intento se detuvo tras cuatro respuestas: tres válidas y una inválida por mezclar namespaces. Quedaron 42 entradas sin enviar. La tabla de coordenadas no resolvió la fiabilidad del modelo.** [Protocolo previo](semantic-coordinates-v3-protocol-2026-10-21.md), [informe completo](../../outputs/iromem/evaluation/semantic-coordinates-v3-2026-10-21/report.json), [diagnóstico](../../outputs/iromem/evaluation/semantic-coordinates-v3-2026-10-21/failure-diagnosis.json).

## Implementación y pruebas locales

V3 conserva los 46 inputs e IDs, anotaciones congeladas V1, separación reversible V2 y parámetros ajenos a mensajes. Añade tablas de rutas mostradas y su correspondencia con rutas canónicas. Las referencias del modelo solo se traducen si coinciden exactamente con una entrada predeclarada; después se aplica el validador V1 sin cambios. El diario conserva siempre el contenido recibido. No se corrigen prefijos, se aproximan referencias o se reanudan diarios históricos.

Se verificaron 3.373 correspondencias de nodos sobre los dos árboles de las 46 entradas: cobertura exhaustiva, unicidad, existencia de rutas, valores hoja exactos y reconstrucción íntegra. Las pruebas incluyen evidencia movida con valores anidados, escapes, Unicode, null/ausencia, rutas a metadatos y formas JSON incorrectas. El crítico estático detectó un acceso mutable que podía provocar panic ante JSON mal formado; se sustituyó por accesos comprobados y se añadió regresión antes de inferencia. La revisión posterior no encontró bloqueos dentro de su alcance; no fue revisión ciega de calidad ni validación humana.

Pasaron **505 pruebas con y sin HTTP/2**, formato y diff. Antes de HTTP, el runner reprodujo preparación, requests, paquete, mapa, cuatro anotaciones y vínculos al padre en un temporal propio; verificó ejecutable y fuentes. Cuatro bancos manuales, diez informes de extracción, cinco reportes/auditores locales anteriores y el reporte V3 se reproducen idénticos. Las fuentes históricas archivadas se conservan; los archivos compartidos actuales sí cambiaron.

## Resultado y denominadores

| Medida | Resultado |
| --- | --- |
| Planificadas / iniciadas / HTTP200 terminales | 46 / 4 / 4 |
| Respuestas válidas / inválidas / no enviadas | 3 / 1 / 42 |
| Comparaciones de dimensión válidas / previstas | 12 / 184 |
| Discrepancias entre las doce válidas | 0 |
| Tokens informados de entrada / salida | 17.868 / 1.271 |
| Coste informado nuevo / operaciones con coste desconocido | USD 0,045904 / 0 |
| Intento terminal p50 / p95, muestra de cuatro | 2.184 / 3.203 ms |
| Duración del runner | 3,228 s |

Las tres respuestas válidas son controles fieles. Ese subconjunto no acredita un 100 % del cribado. La salida inválida no se contabiliza como bloqueo semántico correcto. El harness Cargo terminó correctamente porque persistió y drenó el fallo; la puerta experimental no pasó. No se ejecutaron los controles posteriores de negación ni el resto del banco.

En s001 el original dice ayer, mientras raw contiene una condición con quote=ayer y value=hoy; el normalizado conserva hoy. La respuesta usa raw_paths=/document/certain/0/context/0/value, una ruta canónica fuera de la tabla mostrada, junto a una ruta normalizada /fields/… válida. El validador la rechaza. Además, su razón afirma que raw dice ayer y atribuye el cambio al normalizador: confunde la cita con el valor propuesto. Las dimensiones emitidas tampoco coinciden con las anotaciones congeladas. No se eliminó ese fallo ni se recalcularon aciertos reparando las rutas.

## Coste, límites y siguiente decisión

La consulta pública confirmó OpenAI mediante OpenRouter y precedió a una estimación orientativa de USD 0,72 para hasta 46 peticiones, duración 1–3 minutos. La ilustración conservadora de bytes como tokens más salida máxima fue USD 2,58, sin garantía de factura. La tabla aumenta la entrada; no se interpreta como ganancia de eficiencia. Solo se realizaron cuatro inferencias y un GET de catálogo separado. Alias solicitado/servido openai/gpt-4.1; revisión interna no verificada independientemente. No se afirma negociación HTTP/2 observada.

Se conservan todos los recibos: **USD 0,045904 nuevos**, sin cargos desconocidos nuevos ni reintentos. Acumulado: **6.055 operaciones lógicas, 23 campañas emparejadas completas, USD 1,128020508 conocidos más 16 operaciones históricas de coste desconocido**. Codex queda excluido. Los tiempos/tokens pertenecen al verificador parcial, no a recall, entrada de Codex o una tubería completa. V1/V2 son comparadores históricos, no controles contemporáneos; no se comparan percentiles parciales con campañas completas.

La siguiente versión debe restringir las referencias mediante el esquema de salida, en vez de confiar solo en instrucciones y tablas. También sigue pendiente resolver la confusión entre texto original, cita y valor operativo: una respuesta de formato válido no basta. No está implementada esa siguiente versión. Se mantiene el objetivo completo: cribado semántico, extracción/tubería automática, conocidos100/126, baseline, recursos/estabilidad y finalmente casos nuevos con revisión ciega. No se abre V6 ni se promueve una política, ni se actualizan gráficas del producto con métricas parciales.
