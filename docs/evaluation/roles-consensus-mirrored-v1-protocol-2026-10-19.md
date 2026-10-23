# Protocolo previo: dos votos con orden invertido

Fecha: 2026-10-19. Desarrollo conocido y opción experimental `roles_consensus_mirrored_v1`. Frente a `roles_consensus_batch_v1`, el único cambio de entradas es invertir el orden de los mismos primeros 16 candidatos en los criterios de la segunda pregunta. Se mantienen ambas instrucciones congeladas, resúmenes literales, modelo Jev, opción `none`, estado y una petición con dos preguntas. La búsqueda sigue recuperando hasta 20; baseline Decisions clasifica 20. No se cambia el predeterminado.

## Hipótesis y límites

El recorte a cuatro conservó cobertura pero introdujo pérdidas y un acuerdo incorrecto. El lote original de 16 falló por una pérdida de refutación. Invertir la segunda lista puede reducir errores de posición compartidos conservando contexto; también puede crear otros. Es un contraste defendible de sensibilidad al orden, no una corrección semántica garantizada. Ambas preguntas comparten petición y contexto, sin independencia. La comparación con base mide el conjunto modelo/instrucción/mecanismo; comparar con el lote previo solo es contraste retrospectivo, sin atribución causal aislada entre campañas en momentos distintos.

Cada voto tiene su mapa: el primero A–P corresponde al orden recuperado; el segundo A–P al orden inverso de esos mismos 16. Se normalizan ambos a memory_id antes de exigir acuerdo. A/P puede acordar el primer ID; A/A representa distintos IDs salvo lista de uno. `classified_candidate_count` cuenta 16, no 32. Los votos guardados están normalizados; el orden se identifica por la política de la petición. Dos `none` o desacuerdo producen semilla vacía; elección inválida, voto ausente o error técnico degrada y conserva recibo global conocido, una sola vez.

Pruebas previas: listas de uno, siete y veinte candidatos; reversión solo del prefijo 16; criterios y texto completos; acuerdo por ID, discrepancia de misma etiqueta, dos abstenciones, elección fuera de rango, voto ausente y raíz inválida. Deserialización y propagación hasta servicio, límites anteriores y recibos permanecen cubiertos. Suite workspace: 63 pruebas.

## Secuencia congelada

Primero v4 conocida completa; después v2 conocida solo si pasa todas las puertas. Cada cohorte: 128 operaciones/peticiones, 64 base y 64 candidata, orden alternado por posición sin usar gold. Estimación 1–2 min/cohorte. Top16 conserva utilidad y mejor nivel en ambas según cribado local previo, ligado por hash. Los candidatos recuperados deben ser idénticos, aunque candidata clasifica 16 y base 20. No hay reintentos inciertos, servidores, nuevas dependencias o base ajena; SQLite temporal propia. Registrar huellas de código/binarios/preguntas/gold/revisión/protocolo antes de llamadas.

Puertas intactas: ninguna selección no permitida, ninguna pérdida por caso de utilidad/completitud de base, ninguna degradación nueva y al menos una ganancia; recibos completos y coste/p50/p95 ≤1,5× base. No compensar pérdidas con ganancias. Si falla v4, detener sin v2/v5; conservar cada error y diagnosticar sin retocar etiquetas.

Solo si pasan v4 y v2, verificar cobertura útil y mejor nivel de top16 en v5 nueva, ya redactada, revisada a ciegas y conciliada antes de cualquier llamada. No modificar preguntas, gold, instrucciones o límite según ese cribado: si falla cobertura, conservarlo y no abrir llamadas. Si pasa, otras 128 operaciones/peticiones con las mismas puertas. La revisión ciega usa agentes separados de la misma familia, sin etiquetas/resultados/cuotas, no revisión humana ni independencia de familia. El coste comunicado no incluye Codex; medir selección de recuerdos no mide respuestas finales ni integración automática.

Incluso un pase sintético exige reporte de errores, latencia, consumo y límites antes de incorporar. El goal no se cierra mientras falte alguna de las cinco puertas.
