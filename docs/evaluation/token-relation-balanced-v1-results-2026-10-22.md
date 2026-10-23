# Objetivo balanceado: aprende TRAIN, falla el rescate

La candidata se rechaza: no rescata útiles y añade 35 recuerdos ajenos. La configuración vigente del producto se mantiene; el objetivo sigue activo en el paso 3, con dos de cinco etapas cerradas.

| Medición conocida | Control V160 | Candidata balanceada |
| --- | ---: | ---: |
| Aciertos / 432 | 374 | 339 |
| Útiles conservados / 200 | 171 | 171 |
| Ajenos seleccionados / 232 | 29 | 64 |
| Rescates útiles / C | 0 / 0 | 0 / 0 |
| Nuevas pérdidas útiles | 0 | 0 |
| Nuevos ajenos | 0 | 35 |

La unión conserva los 200 IDs iniciales por construcción; no evita adiciones irrelevantes. En las 232 abstenciones de la referencia, el nuevo verificador selecciona 35 ajenos y ninguno de los 29 útiles. La precisión de esas 35 adiciones es 0; en el conjunto vacío del control es indefinida, no 100 %. El padre histórico V5 ya vio 176 pares y no es la ruta productiva FTS5/Decisions; no llamar a este resultado calidad del producto o validación fresca.

## Qué cambió

Únicamente el objetivo de entrenamiento: pesos n/(2npos) y n/(2nneg) calculados con el TRAIN residual de cada fold, promedio de peso×BCE por minibatch. Inicialización y agendas coinciden exactamente con el control160. Se mantuvieron datos, embeddings, arquitectura de 36.929 parámetros, semilla, épocas, batches, AdamW, clipping, checkpoint final y corte numérico 0,5. El control no se reentrenó. Las probabilidades ponderadas no son confianza calibrada bajo las frecuencias reales; cambiar el objetivo puede cambiar el punto operativo aun con el mismo corte.

## La mejora de entrenamiento no basta

En los 464 pares-fold TRAIN, pasa de seleccionar 0/58 útiles a 53/58, pero también acepta 87/406 ajenos. Los tres folds cumplen el diagnóstico prospectivo de aprendizaje de ambas clases; esto no sustituye la puerta de calidad. Las ocurrencias repiten filas entre folds y no son 464 casos independientes.

| Fold | AUC TRAIN control → candidata | AUC fuera del componente control → candidata | TP / FP fuera del componente |
| --- | ---: | ---: | ---: |
| 1 | 0.853 → 0.950 | 0.571 → 0.661 | 0 / 0 |
| 2 | 0.852 → 0.977 | 0.494 → 0.459 | 0 / 0 |
| 3 | 0.700 → 0.710 | 0.254 → 0.379 | 0 / 35 |

El ranking TRAIN mejora en los tres grupos, de modo que el efecto observado no se describe solo como un desplazamiento uniforme del intercept. Esa discriminación no se transfiere de manera útil al componente excluido: ningún útil supera el corte y los 35 errores nuevos se concentran en el componente mayor. No atribuir causalmente todo el fallo al desbalance ni afirmar que más aceptación demuestra mejor selección. El AUC OOF agrupado 0,218→0,245 mezcla priors/modelos de distintos folds y es descriptivo; se prioriza el desglose por fold.

## Verificación y recursos

Plan y preflight originales conservados como rechazados por un UnboundLocalError en el propio script de verificación: una importación interna sombreaba importlib.metadata. Ocurrió antes de cualquier modelo o entrenamiento; se corrigió solo ese import y se conservó la primera fuente y el error. El preflight final pasa y liga los 307 archivos congelados al plan39c8d028bd04026499a63bf22098335d04b96a72c32f1b89dbd968468a84709a. Datos, objetivo y receta permanecieron iguales.

Controles sintéticos de loss/gradiente ponderados: batch mixta, monoclase y última parcial; masa total TRAIN n/2 por clase y peso medio1, sin renormalización por lote o doble factor. Sus cuatro backwards de logits se cuentan aparte: cero instancias/updates del verificador reales en esos controles. Arquitectura y AdamW reutilizan controles160 por hash.

Ejecución real completa: tres fits, 708 updates/backwards, 5.568 presentaciones TRAIN, 232 OOF y 464 evaluaciones finales TRAIN; 200 bypass de referencia. Cero cargas/forwards/AD del reranker grande. Tiempo total 2.836281 s, entrenamiento 1.648524 s, OOF 0.027575 s; RSS pico 241008640 bytes. Exit0, sesión42767 y PID ausente. Son recursos instrumentales, no latencia del producto ni pase del límite≤1,5×.

Auditoría NumPy independiente aprobada: 232 OOF y 464 TRAIN, decisiones idénticas y diferencias máximas de probabilidad 8.96e-08 / 2.05e-07, frente a tolerancia previa5e-5. Pesos TRAIN, inicialización/agendas160, estado final AdamW, 708 recibos, AUC con media de empates, losses, precisión vacía y gate C comprobados. No replay de los 708 gradientes ni garantía de óptimo en este entrenamiento no convexo.

API adicional: cero llamadas y USD0. Coste Codex/local desconocido. Fuentes Rust218 idénticas; suites928default/930HTTP2 heredadas de150, no repetidas; fmt ejecutado en el cierre. Sin producto, DB, servicios, puertos, descargas o dependencias nuevas.

## Decisión y siguiente trabajo

Rechazar este objetivo balanceado como candidata y conservar160 también rechazado. No ajustar pesos, corte, épocas o checkpoint a estos errores. El diagnóstico respalda que el módulo puede aprender parcialmente sus datos, pero necesita demostrar una señal que transfiera la relación pregunta–evidencia. Se revisa una hipótesis distinta de representación contextual por token, con crítica previa y declaración de exposición del encoder: todavía no se ha implementado ni medido esa propuesta.

Las puertas originales24/24+64/64, seis conjuntos, ganancia C sin daños nuevos, casos nuevos con revisión ciega, recibos/recursos≤1,5× y estabilidad permanecen intactas. No se abrió original88, reserva o V6 y no hay adopción.
