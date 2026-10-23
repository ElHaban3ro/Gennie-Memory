# Adaptación local V4: agrupación de gradientes por consulta

Ambas candidatas fallan la puerta local. El objetivo original permanece activo. No cambia la configuración vigente.

[Protocolo previo](utility-learning-v4-protocol-2026-10-22.md). Artefactos en `outputs/iromem/evaluation/utility-learning-v4-2026-10-22/`. Plan SHA-256: `e916f847740197329332519afe884a4fe9ac4bf2c6f14c1a033216c2fc866659`.

## Calidad y límites

Los 64 pares conocidos y 24 de desarrollo ya están observados: son regresión, nunca validación fresca. La base es el reranker sin adaptar, no la ruta productiva FTS5/Decisions. Aciertos incluye rechazos correctos y no sustituye preservar evidencia útil.

| Versión | Conocidos correctos /64 | Útiles preservados /17 | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Reranker sin adaptar | 53 | 16 | 10 |
| V2 | 59 | 14 | 2 |
| V3 ampliado | 55 | 9 | 1 |
| V4 rows | 55 | 9 | 1 |
| V4 queries | 59 | 13 | 1 |

| Brazo | Original entrenado: aciertos /48; útiles /24; ajenos | Extensión entrenada: aciertos /128; útiles /48; ajenos | Desarrollo observado: aciertos /24; útiles /12; ajenos |
| --- | --- | --- | --- |
| rows | 42; 22; 4 | 124; 45; 1 | 16; 8; 4 |
| queries | 38; 24; 10 | 108; 47; 19 | 15; 11; 8 |

Los 176 ejemplos son entrenamiento en ambos brazos. Los porcentajes de ajuste no acreditan generalización. Los errores y pérdidas nuevas están íntegros en los reportes.

| Brazo | Pools exactos con 0 útiles /8 | Con 1 /8 | Con 2 /8 | Con 3 /8 |
| --- | ---: | ---: | ---: | ---: |
| rows | 8 | 5 | 7 | 8 |
| queries | 3 | 5 | 6 | 7 |

El agrupado mejora preservación respecto al control, pero sigue perdiendo tres útiles conservados por la base. En entrenamiento combinado preserva 71/72 útiles y selecciona 29/104 ajenos, frente a 67/72 y 5/104 en el control: no es una mejora general de discriminación. En desarrollo también suben útiles (8→11/12) y ajenos (4→8).

## Interpretación del contraste

Mismos 176 pares, seis épocas, 1056 pasos forward/backward y 264 actualizaciones por brazo. Misma base, inicialización LoRA, tokenización, instrucción, pérdida CE más penalización positiva V2, umbral 0,5 y optimizador. Control contemporáneo rows con orden idéntico a V3 expanded; candidata queries agrupa por consulta, sin ranking, top-1, cuotas o etiquetas en el sampler. Son dos procesos seriales y solo se evalúa su checkpoint final prefijado.

La hipótesis de interferencia entre contribuciones de preguntas distintas sigue siendo una hipótesis; un cambio de calidad en este ensayo mide el efecto de ese algoritmo de orden y empaquetado en este banco y presupuesto. No demostraría por sí solo un mecanismo universal de interferencia, capacidad suficiente o generalización a familias nuevas. No se ajusta retrospectivamente umbral, agrupación, etiquetas o época para rescatar resultados.

## Ejecución, coste y verificación

rows: 716.731 s totales, 508.600 s de entrenamiento.
queries: 400.929 s totales, 303.031 s de entrenamiento.

Total 1117.660 s. Son duraciones del experimento; no son latencias de producción ni prueban p50/p95 dentro de 1,5×. Dos brazos: 3296 forward transformer, 2112 backward, 528 updates y 1184 scores repetidos sobre 264 pares distintos.

API nueva: USD 0, cero llamadas a proveedores. Coste económico local y Codex desconocido. Acumulado externo intacto: 6155 operaciones, 23 campañas emparejadas completas, USD 4,566379428 conocidos más 24 operaciones de importe desconocido, excluyendo Codex.

La revisión estática independiente pasó 65 comprobaciones antes de inferencia, incluida reconstrucción independiente de órdenes y exclusión de labels/role/rationale del sampler. Las auditorías posteriores de eventos, fórmulas, contadores, arrays, hashes y selección separan integridad de calidad; no repiten entrenamiento ni verifican semántica poblacional. El hash de base antes/después es registrado por runner, sin segunda copia completa posterior independiente.

No hay código Rust/Cargo nuevo; las verificaciones históricas se heredan únicamente tras comprobar mismos 213 hashes y binario: 879 aprobadas/24 ignoradas por defecto, 881 aprobadas/75 ignoradas HTTP/2 y formato previo. No se repitió Cargo para estos scripts aislados. Sin servicios/puertos, dependencias o pesos nuevos, DB, secretos, env, procesos ajenos, rama, commit o push.

Reserva de 24 y V6 permanecen cerradas en este reporte. Solo una candidata elegible congelada podría abrir la reserva conforme al protocolo, nunca para elegir entre brazos. Siguen pendientes casos nuevos con revisión ciega, comparación con configuración vigente, calidad sin pérdidas, recursos/recibos/estabilidad y V6 antes de adoptar.

El control reproduce exactamente los 1056 pasos de V3 ampliado (pérdida, CE, penalización, margen y norma) y el archivo de pesos final es idéntico. La candidata agrupada finalizó durante una interrupción: diario completo, reporte completo y ausencia de proceso confirmados; su handle ya no existe y el código de salida del sistema no se observó. Se conserva como desconocido, sin inventar salida 0 ni repetir entrenamiento.

Un fallo posterior de agregación por dos rutas antiguas de V3 se conserva en `aggregate-coordinator-failed-v1.py/json`; se corrigieron solo las rutas del reporte, sin cambiar scores, pesos, etiquetas o repetir inferencias.

Auditorías posteriores: control 1904/1904, agrupado 1903/1903 y pareja 92/92, total **3899 comprobaciones aprobadas**. Son adicionales a las 65 previas y no equivalen a calidad semántica. La elección independiente coincide con `selection.json`: ninguna candidata, ninguna reserva abierta.

Contabilidad local de entradas: 404531 posiciones de tokens por brazo (260562 de entrenamiento y 143969 de evaluación), 809062 en total. Se calculan de las secuencias tokenizadas completas por forward; no suman backward como nuevas entradas. Son procesamiento local del modelo, no tokens enviados o facturados por un proveedor ni una medición de tokens Codex.

## Revisión retrospectiva y siguiente contraste

La revisión separada `causal-review-independent.json` encuentra que el agrupado aumenta 262 de 264 scores respecto al control, incluidos todos los de entrenamiento y desarrollo. El orden útil frente a ajeno dentro de cada consulta cambia: entrenamiento 24→24/24 comparaciones correctas, extensión 79→78/80, desarrollo 11→10/12 y conocidos 48→49/49. Hay una corrección real de orden en conocidos, pero domina una mayor aceptación; el ranking por pares no permite elegir un único recuerdo ni acredita consultas sin útiles o con varios.

El clipping global se activó en 229/264 actualizaciones por filas y 203/264 agrupadas. También cambió la composición útil/ajeno por actualización. Son consecuencias del sampler probado, pero no se han aislado del efecto de compartir consulta. Solo se conservan normas acumuladas, no vectores de gradientes por parámetro o clase; no se afirma alineamiento o cancelación demostrados.

Siguiente contraste propuesto, aún sin ejecutar: agrupado por consulta frente a un control contemporáneo que iguale el patrón binario de target y origen original/extensión en cada posición de cada actualización. Construir el control permutando sin reemplazo los cuatro estratos de entrenamiento y rellenando las posiciones del patrón agrupado. Mantener exactamente cada fila una vez por época, seis épocas, pérdida, inicialización y 264 updates. Así se examina la asociación semántica condicionada a composición de targets y origen; las coocurrencias casuales se deberán registrar sin buscar otra semilla después de resultados.

Ese nuevo sampler usaría explícitamente targets **solo de entrenamiento** para ordenar, a diferencia del sampler sin etiquetas de V4. Nunca añadiría targets, roles, razones o metadatos al input del modelo. Requiere protocolo y órdenes previos, un solo contraste y las puertas originales completas. Reutilizar V4 como control solo sería comparación histórica. Las reservas siguen cerradas; ninguna de estas observaciones convierte el fallo actual en mejora adoptable.
