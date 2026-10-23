# Verificador de relaciones entre tokens: sin rescates

La candidata no pasa calidad: conserva la referencia experimental, pero no mejora ninguna decisión. No se adopta ni se abre la validación nueva. El objetivo completo sigue activo en el paso 3, con dos de cinco etapas cerradas.

| Medición conocida | Referencia V5 | Referencia + verificador |
| --- | ---: | ---: |
| Aciertos / 432 | 374 | 374 |
| Útiles conservados / 200 | 171 | 171 |
| Ajenos seleccionados / 232 | 29 | 29 |
| Nuevas pérdidas útiles | — | 0 |
| Nuevos ajenos | — | 0 |
| Rescates útiles / parciales C | — | 0 / 0 |

La conservación de selecciones resulta de la unión con la referencia, no demuestra que el verificador reconozca evidencia útil. Rechazó las 232 abstenciones evaluadas: 29 útiles y 203 ajenos. La referencia es V5 con LoRA, ya entrenada sobre 176 pares; no es la ruta productiva FTS5/Decisions. El resultado no se compara como mejora respecto a los 355/432 de otra candidata con política distinta.

## Qué se implementó y probó

Se ejecutó una sola arquitectura de 36.929 parámetros FP32: proyección compartida de embeddings, posición fija, atención entre tokens y MLP antes de la decisión. Usa 1.924 filas BF16 originales decodificadas exactamente a FP32, fuera de los parámetros entrenables. Pregunta y documento completos, sin truncamiento o instrucciones de chat. No consume IDs, roles gold, scores de los lectores anteriores o reglas por caso como features.

Tres fits excluyen el componente fuente completo, con 12 épocas y la receta congelada de AdamW/BCE sin ponderación. El cuarto componente no contiene abstenciones: sus ocho casos no prueban el verificador ni provocan otro fit. Las 200 selecciones de la referencia pasan sin scorer. No se escogió checkpoint, corte, época o semilla a partir de resultados.

## Diagnóstico posterior: no basta llamarlo falta de transferencia

El auditor recompuso los checkpoints finales sobre sus propios TRAIN, sin reentrenar: también rechazan las 58 ocurrencias útiles de 464 pares-fold evaluados (29 útiles únicos repetidos entre folds), al corte fijo 0,5. Rechazan los 406 ajenos. Por ello no es únicamente un fallo fuera de entrenamiento.

| TRAIN del fold | Útiles seleccionados | BCE final | Mejor BCE constante según prevalencia TRAIN |
| --- | ---: | ---: | ---: |
| 209 pares, 22 útiles | 0/22 | 0,269673 | 0,336496 |
| 183 pares, 15 útiles | 0/15 | 0,231032 | 0,283548 |
| 72 pares, 21 útiles | 0/21 | 0,576823 | 0,603637 |

La pérdida menor que el óptimo constante muestra que no es correcto llamarlo predictor literalmente constante o afirmar ausencia de aprendizaje. Sus scores varían, pero ningún positivo cruza el corte. No demuestra que la causa sea exclusivamente el desbalance, que la arquitectura carezca de capacidad o que más épocas lo arreglen. Esos cambios no se ensayaron.

## Integridad y coste

Preflight anterior al entrenamiento: 259 archivos congelados, todos los paths requeridos, 1.924 filas byte a byte, 432 tokenizaciones y soporte de tres folds. Controles sintéticos previos independientes: paridad NumPy/MLX, padding, gradientes de los cinco arrays y un paso AdamW manual con clipping. Sus tres forwards, un backward y una actualización sintéticos se contabilizan aparte.

Ejecución real: tres fits, 708 batches backward/actualizaciones y 5.568 presentaciones TRAIN, más 232 predicciones OOF y 200 bypass. Cero forwards, backwards o actualizaciones del reranker grande. Pesos iniciales/finales, agendas, pérdidas/normas y estado final de optimizador con pasos y hashes persistidos. Checkpoint final único.

Auditoría independiente NumPy aprobada: las 232 decisiones coinciden; diferencia máxima de probabilidad 5,56e-8 frente a tolerancia previa 5e-5. Verifica fuentes, máscaras, pesos, agendas, contadores, estado final AdamW, decisiones y gate obligatorio de ganancia C sin daños. No reproduce cada uno de los 708 gradientes ni certifica un óptimo de este entrenamiento no convexo. El diagnóstico adicional de 464 pares-fold TRAIN usa CPU, sin MLX/AD o inferencias nuevas del reranker.

Tiempo del runner: 3,872188 s; entrenamiento 2,483590 s y OOF 0,325129 s. RSS pico 235.421.696 bytes. Exit 0, sesión 10114, PID ausente. Son recursos del ensayo, no latencia del producto ni prueba del límite original de recursos ≤1,5×.

API adicional: cero llamadas, USD 0. Coste económico Codex/local sin cuantificar. Las 218 fuentes Rust permanecen idénticas; suites 928 default/930 HTTP2 heredadas de 150, sin repetir. fmt se ejecuta en este cierre. Sin producto, DB, servicios, puertos, descargas o dependencias nuevas.

## Decisión

Rechazar V1 por cero beneficio, manteniendo todos sus artefactos. No bajar el corte ni rescatar otro checkpoint, época o balanceo dentro de esta corrida. La próxima hipótesis debe contrastar por qué el objetivo de aprendizaje admite omitir todos los positivos al corte fijo, distinguiendo calibración de discriminación y sin atribuir una causa no probada. Una aceptación mayor no demuestra mejor calidad: siguen siendo obligatorias ganancia C y cero ajenos nuevos, además de las puertas originales 24/24+64/64, seis conjuntos, casos nuevos ciegos y recursos/recibos/estabilidad. No se abrió ninguna reserva ni V6.
