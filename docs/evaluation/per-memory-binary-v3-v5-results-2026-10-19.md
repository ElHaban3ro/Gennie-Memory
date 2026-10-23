# Tarea explícita e incompatibilidad: V3–V5

2026-10-19. **Goal activo. Ninguna variante nueva se aprueba.** No se amplían las fallidas ni se usa V6. Baseline y dashboard intactos. Se conserva el snapshot007 y se añade008.

| Intento conocido v5 | Operaciones completadas/previstas | Resultado | USD conocidos | Coste desconocido |
| --- | ---: | --- | ---: | --- |
| Binaria V3, tarea explícita |18/18 |17respuestas completas; una selección ajena y un fallo técnico |0.000904428 |1operación |
| Binaria V4, incompatibilidad ambigua |1/34 |Ampliación detenida por revisión estática;33no ejecutadas |0.000056406 |0 |
| Binaria V5, OR explícito |34/34 |Cero pérdidas útiles de base, un ID ajeno; no pasa calidad |0.001920072 |0 |

Total53operaciones, USD0,002880906 conocidos más una operación de importe desconocido. No son53respuestas correctas ni53recibos completos. No incluye tokens/coste de Codex.

## Qué se aprendió

V3 añadió la tarea explícita a cada recuerdo sin cambiar state/criterios de V2. Recuperó las ocho pérdidas anteriores que pudieron evaluarse; la novena falló tras20.011,42ms, sin estatusHTTP ni recibo registrados, y no se repitió en esa campaña. Es compatible con el límite de20s del cliente, pero el código genérico no identifica la causa exacta. La petición HEAD sin credenciales devolvióHTTP/2 404: demuestra conectividad TLS/HTTP, no salud de inferencia ni importe del fallo. Las17consultas restantes se completaron. El carné volvió a aceptarse como evidencia sobre avalista (`cev5_071_03`), así que también falla semánticamente y no se amplía. Un formato más claro no prueba una causa interna del proveedor.

V4 añadió prueba de incompatibilidad para una refutación. La revisión crítica detectó después de lanzar una primera operación que «si falta dato solicitado, none» podía vetar una refutación válida. Se detuvo la ampliación, preservando instrucciones/coste y auditoría parcial explícita: una completada,33no ejecutadas. Fue un error de coordinación iniciar antes de concluir esa revisión. No se midió esa regresión semántica ni se atribuye un pase al único caso.

V5 corrige el OR: dato/parte solicitados o refutación válida; none solo si ambos fallan. El resto del payload deriva V3 sin cambios. Revisión crítica favorable antes de llamadas. Las34consultas (18anteriores más16unrelated) completaron con todos sus recibos;34HTTP. El coste es1,0431× base histórica del subconjunto, pero persiste el ID ajeno del carné. Eso bloquea aunque no se pierdan IDs de base. Semillas sin ID ajeno33/34, no precisión de respuesta final. No se hizo comparación completa pareada porque el piloto falla.

Auditorías ordinarias V3/V5 y auditoría parcial declaradaV4 validan eventos/candidatos/semillas. Los juicios y etiquetas originales permanecen. Revisión estática de ruta y recibos no prueba calidad semántica.89pruebas workspace pasan, fmt check pasa. Sin dependencias, tablas, configuración secreta o servicios nuevos.

## Revisión del siguiente mecanismo

Se exploró sin llamadas una composición de decisiones históricas: conservar V2 si selecciona algo y rescatar con V5 solo allnone.25de26abstenciones V2 tienen respuesta V5 conocida; la consulta06605 no se observó con V5. Esa proyección no es una ejecución nueva ni pase. La revisión crítica señaló que un primario positivo puede estar incompleto. Se comprobó el ejemplo06706: pide humedad/aplastamiento/sellado; V2elige013 y omite014con el sellado. Rescatar solo ante vacío cerraría esa parte disponible. No se implementa como atajo al objetivo ni se modifican las anotaciones para hacerlo pasar.

Siguiente diagnóstico preparado: registrar scores A/none que devuelve la API con el mismo payload y elección V5; verificar si existe un umbral global con margen predefinido, sin usar V6 para ajustarlo. Si los grupos se solapan, se descarta esa hipótesis. Sigue pendiente implementar la instrumentación y recoger la cohorte completa. [Protocolo congelado del diagnóstico](per-memory-score-diagnostic-protocol-2026-10-19.md).

Artefactos: [V3](../../outputs/iromem/evaluation/per-memory-binary-v3-pilot-2026-10-19/report.json), [V4 detenido](../../outputs/iromem/evaluation/per-memory-binary-v4-pilot-2026-10-19/aborted.json), [V4 auditoría parcial](../../outputs/iromem/evaluation/per-memory-binary-v4-pilot-2026-10-19/audit-partial.json), [V5](../../outputs/iromem/evaluation/per-memory-binary-v5-pilot-2026-10-19/report.json). Protocolos anteriores a llamadas: [V3](per-memory-binary-v3-protocol-2026-10-19.md), [V4](per-memory-binary-v4-protocol-2026-10-19.md), [V5](per-memory-binary-v5-protocol-2026-10-19.md).
