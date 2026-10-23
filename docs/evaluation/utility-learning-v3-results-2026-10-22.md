# Adaptación local V3: expansión y control con actualizaciones iguales

Las dos candidatas finales **fallan la puerta de calidad**. La ampliación mejora el ajuste a sus 128 ejemplos de entrenamiento, pero empeora la preservación de evidencia útil en los 64 pares conocidos. No se adopta ningún adaptador ni se abren los 24 casos reservados o V6. El objetivo permanece activo en el paso 3, con los cinco compromisos originales intactos y sin permisos pendientes.

[Protocolo previo](utility-learning-v3-protocol-2026-10-22.md). Evidencia: `outputs/iromem/evaluation/utility-learning-v3-2026-10-22/`, con plan, órdenes, fuentes, eventos, checkpoints, resultados y auditorías conservados. Plan SHA-256: `38e9602874e05855ab95c2c3121a5c2a1719b0c13bca2a784d888fdbbc9a4e07`.

## Comparación de calidad

Cada decisión evalúa si un recuerdo aporta evidencia útil para una pregunta. «Aciertos» incluye rechazos correctos; debe leerse junto a útiles preservados y ajenos seleccionados. Estos 64 pares ya eran conocidos: son regresión, no validación fresca.

| Versión | Aciertos /64 | Útiles preservados /17 | Ajenos seleccionados | Pérdidas útiles nuevas frente a base |
| --- | ---: | ---: | ---: | ---: |
| Reranker sin adaptar | 53 | 16 | 10 | 0 |
| V2, 72 actualizaciones | 59 | 14 | 2 | 2 |
| V3 original, 264 actualizaciones | 56 | 10 | 1 | 6 |
| V3 ampliado, 264 actualizaciones | 55 | 9 | 1 | 7 |

El comparador de esta tabla es el mismo reranker sin adaptar; **no** mide todavía la ruta vigente FTS5/Decisions ni permite afirmar una mejora del producto.

| Brazo | Entrenamiento original: aciertos /48; útiles /24; ajenos | Extensión: aciertos /128; útiles /48; ajenos | Desarrollo observado: aciertos /24; útiles /12; ajenos |
| --- | --- | --- | --- |
| Original | 48; 24; 0 | 108; 40; 12 | 15; 10; 7 |
| Ampliado | 42; 22; 4 | 124; 45; 1 | 16; 8; 4 |

En desarrollo, el original preserva A 4/4, B 2/2 y C 4/6; el ampliado A 4/4, B 2/2 y C 2/6. En regresión conocida, el ampliado preserva A 5/10, B 3/4 y C 1/3. La concentración en información parcial de desarrollo es una pista, no explica por sí sola todos los errores de regresión.

Los 48 originales forman entrenamiento en ambos brazos. La extensión de 128 solo forma entrenamiento en el ampliado; por eso su 124/128 es ajuste dentro de muestra. En conjunto, el ampliado acierta 166/176 pares entrenados, preserva 67/72 útiles y selecciona 5/104 ajenos. Aun dentro del entrenamiento quedan cinco pérdidas útiles y cinco ajenos. Los 24 de desarrollo están observados y no pueden presentarse como ciegos.

Exactitud por conjunto de cuatro candidatos de la extensión (ocho consultas por cardinalidad útil):

| Brazo | 0 útiles | 1 útil | 2 útiles | 3 útiles |
| --- | ---: | ---: | ---: | ---: |
| original | 4/8 | 3/8 | 4/8 | 6/8 |
| expanded | 8/8 | 5/8 | 7/8 | 8/8 |

El ampliado sigue fallando en consultas con uno y dos recuerdos útiles. La mejor selección de conjuntos entrenados no acredita generalización a casos nuevos.

## Qué se contrastó y qué permite concluir

Misma base Qwen3-Reranker-0.6B FP16 congelada, mismos 32 tensores LoRA q/v, rango 8, escala 8, últimas ocho capas, semilla y estado inicial. Misma pérdida V2 `CE + y*relu(2-margin)^2`, corte 0,5, AdamW y acumulación de cuatro. Cada brazo realizó 1056 pasos de entrenamiento y 264 actualizaciones: 48 pares por 22 épocas frente a 176 por seis épocas. Se fijaron ambos planes antes del primer score y se ejecutaron en procesos separados, en serie; solo se evaluaron los checkpoints finales previstos.

Igualar actualizaciones evita atribuir a los datos una ganancia causada simplemente por más pasos. La frecuencia de repetición, orden y proporción de etiquetas cambian con la ampliación, de modo que tampoco es una prueba de una sola propiedad semántica aislada. Los primeros 288 pasos del control reproducen exactamente las pérdidas, márgenes y componentes de V2; el entrenamiento ampliado presenta pérdida fluctuante. No se ha demostrado una causa única como falta de capacidad, insuficientes épocas o un defecto del optimizador.

La evidencia rechaza estas dos candidatas y la hipótesis de que esta ampliación, bajo este presupuesto, basta para cumplir calidad. No se rescatan umbrales, épocas o etiquetas usando la regresión. El siguiente trabajo es revisar los residuos de entrenamiento y desarrollo, contrastar cobertura conceptual y dinámica de aprendizaje, y fijar una sola intervención antes de nuevas inferencias. No se abre otra reserva para compensar el fallo.

## Revisión retrospectiva y siguiente contraste

La revisión separada `causal-review-independent.json` confirma errores de atributo, selección de planes futuros como hechos e información parcial perdida dentro del entrenamiento. Nueve de los diez errores finales del ampliado ya estaban mal en su última observación de entrenamiento; por tanto, el olvido posterior no explica todo. El margen observado durante entrenamiento precede a su update, no es evaluación posterior.

La hipótesis propuesta es interferencia entre las cuatro contribuciones acumuladas de consultas diferentes, todavía sin medir alineación de gradientes ni descartar optimización o capacidad. El contraste siguiente mantendría los mismos 176 pares, pérdida, inicialización y 264 actualizaciones: control por filas frente a acumulación agrupada por consulta. Los 32 pools nuevos aportan 32 grupos de cuatro; las 24 consultas antiguas de dos filas se emparejan en 12 grupos: 44 actualizaciones por época, seis épocas. Cambiaría únicamente el empaquetado y orden de grupos, sin ranking, top-1, cuotas de seleccionados ni metadatos dentro del modelo.

Esta es una recomendación concreta, aún no ejecutada. Antes de inferencias necesita un protocolo y órdenes deterministas congelados y control contemporáneo; reutilizar V3 como control solo permitiría una comparación histórica. Las puertas y reservas se conservan. Una mejora probaría el efecto de ese sampler en este banco y presupuesto, no una explicación universal ni generalización fresca.

## Ejecución, coste y verificación

Original: 631,039 s totales, 527,879 s de entrenamiento. Ampliado: 696,598 s totales, 559,319 s de entrenamiento. Total observado: **1327,637 s, unos 22 min 8 s**, por encima de la estimación inicial de 8–14 minutos. Ambos acabaron con salida 0 dentro del límite individual de 1200 s. Estas duraciones de experimento no son latencias de producción ni validan la puerta p50/p95.

Dos brazos: **3296 recorridos forward**, 2112 backward, 528 actualizaciones y 1184 scores de evaluación. Los scores repiten base, adaptador cero y adaptador final: no son 1184 ejemplos independientes. Hay 264 pares distintos evaluados en cada brazo. Ninguna llamada nueva a proveedores; **USD 0 de API nuevos**. Coste económico local y Codex desconocido. El acumulado externo permanece en 6155 operaciones, 23 campañas emparejadas completas, USD 4,566379428 conocidos más 24 operaciones de importe desconocido; excluye Codex.

Auditorías independientes aprobadas: original 1943/1943 comprobaciones, ampliado 1878/1878 y pareja 67/67, total 3888. Calidad fallida en ambos. Comprueban por separado integridad y calidad: eventos secuenciales UTC, órdenes y objetivos, contadores, hashes, runtime y fuentes, checkpoint previo a evaluación y selección. Un pase de integridad no convierte el fallo semántico en pase. El hash completo de la base antes y después está registrado por el runner; no existe una segunda copia independiente de todos los tensores posteriores para reauditar esa afirmación.

No cambió código Rust/Cargo ni el binario de evaluación. Se heredan, tras verificar sus 213 hashes de fuente y binario, 879 pruebas aprobadas/24 ignoradas por defecto y 881 aprobadas/75 ignoradas con HTTP/2, junto al formato previamente comprobado. No se repitió Cargo para estos scripts aislados. No hay nuevos servicios, puertos, dependencias, descargas de pesos, cambios de base de datos, credenciales, rama, commit o push.

La adopción aún requiere calidad local completa sin pérdidas nuevas, casos nuevos con revisión ciega, comparación con la configuración vigente, recibos completos, coste y p50/p95 dentro de 1,5×, estabilidad y V6 al final. El trabajo técnico realizado no equivale a porcentaje verificable de objetivo cumplido.
