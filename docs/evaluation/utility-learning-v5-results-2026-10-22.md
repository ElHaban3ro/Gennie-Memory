# Utilidad V5: composición igualada por actualización

Ambos brazos fallan la puerta local. El objetivo original de cinco pasos sigue activo, con configuración vigente intacta.

[Protocolo previo](utility-learning-v5-protocol-2026-10-22.md). Evidencia en `outputs/iromem/evaluation/utility-learning-v5-2026-10-22/`. Plan SHA-256 `42bac91fbf312813a7890be3d8a32267c1f19a255305f2d85cfbd8b5c8b0abe2`.

## Calidad

Los conocidos 64 y desarrollo 24 son regresión observada, nunca validación fresca. La base de este ensayo es el reranker sin adaptar, no FTS5/Decisions productivo. Aciertos incluye rechazos correctos; debe leerse junto a útiles preservados y ajenos seleccionados.

| Versión | Conocidos correctos /64 | Útiles preservados /17 | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Reranker sin adaptar | 53 | 16 | 10 |
| V2 | 59 | 14 | 2 |
| V4 agrupado | 59 | 13 | 1 |
| V5 queries | 59 | 13 | 1 |
| V5 matched | 59 | 13 | 1 |

| Brazo | Entrenado original: aciertos /48; útiles /24; ajenos | Extensión entrenada: aciertos /128; útiles /48; ajenos | Desarrollo observado: aciertos /24; útiles /12; ajenos |
| --- | --- | --- | --- |
| queries | 38; 24; 10 | 108; 47; 19 | 15; 11; 8 |
| matched | 38; 24; 10 | 117; 46; 9 | 14; 10; 8 |

Ambos entrenan los mismos 176 pares. Un ajuste alto dentro de muestra no acredita calidad general. Errores íntegros, ganancias y pérdidas nuevas se conservan sin cambiar gold.

| Brazo | Pools exactos con 0 útiles /8 | Con 1 /8 | Con 2 /8 | Con 3 /8 |
| --- | ---: | ---: | ---: | ---: |
| queries | 3 | 5 | 6 | 7 |
| matched | 4 | 6 | 6 | 8 |

## Contraste y límites

Queries conserva la agrupación y órdenes de V4. Matched permuta sin reemplazo los cuatro estratos de entrenamiento y rellena el mismo patrón de target binario y origen original/extensión en cada posición de cada actualización. Cambia la asignación semántica bajo ese patrón, con coocurrencias por azar registradas sin rerolls. Mantiene cada fila una vez por época.

Matched usa expresamente targets binarios de entrenamiento para organizar las órdenes. Roles, razones y datos de desarrollo/conocidos quedan fuera del sampler; ningún target o metadato se agrega al input del modelo. Misma base, inicialización LoRA, instrucción, tokenización, pérdida V2, umbral 0,5, optimizador, seis épocas, 1056 forward/backward y 264 updates por brazo. Solo checkpoints finales prefijados, procesos seriales.

La comparación permite estimar el efecto de esa asignación en este banco/presupuesto condicionado a composición. No demuestra una explicación universal, alineamiento o cancelación de gradientes ni generalización fresca. Clipping y magnitudes reales pueden variar al cambiar los ejemplos. Ranking o umbrales alternativos no rescatan la puerta fallida.

## Ejecución, coste y verificación

queries: 503.891 s totales; 364.582 s de entrenamiento.
matched: 418.408 s totales; 283.040 s de entrenamiento.

Total 922.299 s. Es tiempo del experimento, no latencia de producción ni prueba de recursos ≤1,5×. Dos brazos: 3296 forward transformer, 2112 backward, 528 updates y 1184 scores repetidos sobre 264 pares distintos.

API nueva USD 0, cero llamadas a proveedores; coste económico local y Codex desconocido. Acumulado externo intacto: 6155 operaciones, 23 campañas emparejadas completas, USD 4,566379428 conocidos más 24 operaciones de importe desconocido, excluyendo Codex.

Integridad posterior aprobada: 1927 queries + 1927 matched + 102 pareja = 3956 comprobaciones. Preflight independiente all_passed=True. La integridad no demuestra calidad semántica: las métricas y puertas se reconstruyen por separado. La auditoría no repite entrenamiento; el hash de base posterior es evidencia registrada sin segunda copia íntegra independiente.

Evidencia de terminación y códigos observados se conserva en `terminal-observation.json`; cualquier código no observado queda desconocido. No se reinicia por pérdida de handle.

Mismos 213 hashes de fuente Rust/Cargo y binario de evaluación: se heredan 879 pruebas aprobadas/24 ignoradas por defecto, 881/75 con HTTP/2 y formato previo. No se repitió Cargo para scripts aislados. Ningún servicio/puerto, dependencia, peso nuevo, DB, secreto, env, proceso ajeno, rama, commit o push.

Las reservas de 24 y V6 siguen cerradas en este reporte. Una candidata elegible congelada solo podría abrir la primera conforme a selección previa; nunca para elegir entre brazos. Después siguen casos nuevos con revisión ciega/pools mixtos, comparación contra configuración vigente, calidad sin pérdidas, recibos completos, coste/p50/p95 ≤1,5×, estabilidad y V6 final antes de adoptar.

El igualado mantiene exactamente las decisiones conocidas del agrupado y sus tres pérdidas útiles nuevas respecto a la base. En entrenamiento combinado pasa de 146/176 a 155/176 aciertos, pero preserva 70/72 útiles frente a 71/72; ajenos 29→19. En desarrollo preserva 10/12 frente a 11/12, con ocho ajenos en ambos. Esa mejora de ajuste no supera la puerta ni demuestra transferencia.

El agrupado reproduce exactamente los 1056 pasos de entrenamiento y bytes de pesos de V4. Ambos procesos de V5 terminaron con código 0 observado; se verificó ausencia por script y argumento de brazo. El guardado de PID detectó que macOS muestra Python resuelto fuera del venv: se corrigió el filtro de observación y se conservó `process-inventory-correction.json`. La observación anterior por prefijo ejecutable de V4 podía omitir un proceso; ahora se revalidó su ausencia con el criterio correcto, sin inventar su código de salida histórico desconocido ni modificar snapshots anteriores.

Contabilidad local: 404531 posiciones de tokens por brazo, 809062 total (entradas completas por forward, sin sumar backward como nueva entrada). Es procesamiento local, no tokens facturados por proveedores ni una medición de Codex.

## Revisión causal y siguiente diagnóstico

La revisión independiente `causal-review-independent.json` no encuentra una ventaja consistente de agrupar por consulta: ambos brazos coinciden en las 64 decisiones conocidas y sus 49/49 comparaciones útil/ajeno dentro de consulta. El agrupado conserva un útil adicional de desarrollo con los mismos ocho ajenos; el igualado elimina diez ajenos de extensión y pierde un útil adicional. Una semilla y regresiones observadas no demuestran equivalencia poblacional ni que composición explique por sí sola todo V4.

Ambos siguen seleccionando los mismos diez negativos del entrenamiento original, entre ellos atributos ajenos, planes futuros y hechos tangenciales a preguntas parciales. El clipping difiere pese al patrón de targets idéntico (203 actualizaciones agrupadas frente a 212 igualadas, de 264). Las normas acumuladas no revelan direcciones ni cancelación. No se selecciona ningún sampler para reservas ni se abre otra ronda de variantes.

Siguiente paso propuesto, aún no ejecutado: diagnóstico del checkpoint final matched congelado sobre los **176 ejemplos de entrenamiento completos**, sin actualizaciones de optimizador. Medir logits, margen y Jacobiano respecto a los mismos 32 tensores LoRA; separar derivadas escalares de CE/penalización de la sensibilidad paramétrica y comparar direcciones firmadas de corrección dentro de cada consulta. Conservar todas las filas, ceros y pools sin útiles; ningún conocido, desarrollo o reserva entra en gradientes o elección de muestra.

Este análisis requiere protocolo y presupuesto propios antes de inferencia. La reconstrucción analítica expresa la regla de la cadena ideal: la aritmética mixta FP16/FP32 exige verificar sus límites numéricos antes de tratarla como gradiente ejecutado exacto. Ni norma pequeña ni conflicto local prueban capacidad global o la trayectoria de AdamW; no se persistieron estados finales de ese optimizador. El diagnóstico no modifica pesos, umbrales o etiquetas y no rescata este checkpoint fallido.
