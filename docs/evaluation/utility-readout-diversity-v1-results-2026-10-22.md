# Diversidad de entrenamiento: resultado del contraste fijo

Paso3 activo. La ampliación de128TRAIN falla el filtro preliminar y no se adopta. Una candidata, mismo encoder base congelado, receta y umbral; no hay barrido ni ajuste retrospectivo. [Protocolo](utility-readout-diversity-v1-protocol-2026-10-22.md).

## Calidad en los432 originales observados

| Medición | Padre histórico | Control contemporáneo | TRAIN ampliado |
| --- | ---: | ---: | ---: |
| Decisiones correctas | 374 | 343 | 328 |
| Útiles conservados (de200) | 171 | 157 | 165 |
| Ajenos seleccionados (de232) | 29 | 46 | 69 |

Frente al padre: 16 pérdidas útiles nuevas, 45 ajenos nuevos y 7 ganancias C. Frente al control: 8 pérdidas útiles nuevas y 32 ajenos nuevos. Los IDs y probabilidades completos quedan en rows.json, sin compensar pérdidas con ganancias.

| Rol | Control: útiles conservados o ajenos seleccionados | Ampliado |
| --- | ---: | ---: |
| A | 92 | 94 |
| B | 25 | 24 |
| C | 40 | 47 |
| none | 46 | 69 |

## Qué se comprobó

El control reproduce151 con diferencia máxima de probabilidad 0 y decisiones idénticas. Esto permite atribuir la diferencia entre los dos brazos a la ampliación elegible de TRAIN bajo esta receta, no al cambio de encoder del experimento anterior. No demuestra causalidad universal ni generalización fresca.

Los128TRAIN nuevos conservan el borrador original:16A,24B,24C,64none. Un revisor separado recibió solo preguntas/textos:128/128 concordancias binarias y de rol, citas literales verificadas. Es la misma familia Codex con historia del proyecto; no equivale a validación humana ni revisión independiente de familia. No se usaron puntuaciones para cambiar etiquetas.

Mapa extendido560:15componentes, cinco con targets originales312/72/32/8/8;48nuevos homologados se excluyen con su componente,80en componentes nuevos pueden entrenar los otros folds. Media, escala y cabeza usan solo TRAIN permitido. Compartir rol no basta para unir familias; el mapa no prueba independencia semántica completa.

Preflight separado antes de inferencia, cuatro controles CPU sintéticos y auditoría posterior aprobada sobre los284archivos congelados y los10ajustes. Se verifican artefactos, hashes, exclusiones, normalización, gradientes finales y predicciones sin volver a entrenar ni ejecutar modelo. La integridad no convierte una puerta de calidad fallida en aprobada.

## Tiempo y coste

| Recurso de esta ejecución | Medición |
| --- | ---: |
| Duración total runner | 19.805s |
| Captura nueva, carga y verificaciones incluidas | 19.196s |
| Diez ajustes CPU | 0.389s |
| Forwards nuevos / cacheados | 128 /432 |
| Backwards /actualizaciones transformer | 0 /0 |
| Pasos CPU de cabezas | 81 |
| Pico RSS | 3582476288bytes |
| Pico MLX | 1596309728bytes |
| Nuevas llamadas/coste API | 0 /USD0 |

Son tiempos instrumentales de investigación, no p50/p95 del recorrido productivo. Energía/amortización y coste Codex desconocidos. El proceso propio es terminal, exit0 observado, sin servicio ni puerto. No se cuentan de nuevo los432vectores cacheados como inferencias nuevas. Acumulado externo permanece6319operaciones,USD4,582739128 conocidos y25operaciones con coste desconocido.

Las218fuentes Rust/producto siguen idénticas al checkpoint151. Las928pruebas default y930HTTP2 se heredan del150; no se volvieron a ejecutar para este cambio Python de investigación. La comprobación de formato actual queda registrada por separado. No se incorporó dependencia productiva ni cambió la base del usuario.

## Diagnóstico posterior sin nuevas inferencias

Se reconstruyeron predicciones de TRAIN y test con las cabezas persistidas, sin reajustarlas. Los cinco controles aciertan todo su TRAIN; dos ampliados también y tres omiten un útil cada uno (519/520,551/552,551/552), con cero ajenos TRAIN. La convergencia numérica no explica la magnitud del fallo fuera del componente.

El componente mayor312 aporta diez de los quince aciertos netos perdidos:254→244; conserva124→133útiles pero selecciona42→61ajenos. Allí los logits medios suben tanto para útiles (+0,786) como para ajenos (+1,101). El componente de72pierde otros cuatro aciertos52→48 y uno de8pierde uno5→4. Los otros dos no cambian decisiones. Es evidencia de peor separación con esta ampliación, no prueba de una causa lingüística única ni de incapacidad del encoder. Todos los cambios, textos y resultados por fold se preservan en residual-diagnostic.json. No se buscan umbrales alternativos para rescatar la candidata.

## Decisión y continuación

Se conservan todas las puertas originales:24/24desarrollo,64/64conocidos, seis conjuntos preservados, recursos≤1,5×base, estabilidad, recibos y V6ciega. Este OOF sobreTRAINobservado no satisface por sí mismo ninguna validación fresca. No se abre reserva ni se cambia el producto con una mejora del promedio. El objetivo sigue activo:2/5etapas cerradas.

La ampliación no alcanza la preservación exigida. La siguiente acción es inspeccionar los residuos y la geometría por componente con los vectores ya capturados, sin otra inferencia o barrido de umbrales, para distinguir falta de cobertura de una limitación de la representación/lectura. No se promete que más datos solucionen el problema.

Una crítica retrospectiva acotada del autor del nuevo TRAIN inspeccionó6ajenos y4pérdidas con fuente y citas, sin detectar motivo para cambiar gold. No fue ciega ni estima prevalencia. Identifica confusión de magnitud, encabezado frente a valor, regla frente a instancia, condiciones y referente; las pérdidas incluyen correcciones, ausencia explícita y dato directo.

Siguiente contraste causal único preparado como hipótesis, todavía sin ejecutar: mantener datos ampliados, features, exclusiones, λ y umbral, pero usar media/escala del control de cada fold. Recalcular la normalización también cambia la penalización efectiva L2 (incluido sesgo regularizado) en coordenadas del hidden. Cinco ajustes CPU bastan para separar esa mediación del cambio en la muestra del objetivo, sin nuevos forwards. Si no reduce daños, conservar el resultado y no rescatarlo con otro umbral. Esta hipótesis no demuestra que el problema sea la normalización.
