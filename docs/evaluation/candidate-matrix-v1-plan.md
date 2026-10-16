# Matriz de candidatos y clasificación v1

Estado: evaluación sintética final completada. [Resultados y conclusión](candidate-matrix-v1-results.md).

## Pregunta y criterio

Medir la recuperación híbrida FTS5 + coseno exacto + RRF en corpus de **100, 200 y 2.000 recuerdos activos**, cruzados con **máximos** de 5, 10, 15 y 20 candidatos presentados. La métrica principal es la proporción de preguntas positivas cuya **semilla final** contiene el recuerdo esperado; el objetivo exploratorio indicado por el usuario es al menos 95 %. Los negativos se informan aparte como proporción de semillas vacías. También se separan cobertura de candidatos, elección del clasificador y fidelidad de hecho, condición y fuente declarada.

La matriz distingue peticiones con identidad de entidad conocida y peticiones sin ella. Cada entidad tiene veinte recuerdos activos de diez tareas de reunión y dos modalidades. Con entidad conocida, el total del corpus no altera la cantidad elegible por consulta; sin ella, los 100, 200 o 2.000 recuerdos del ámbito compiten. Esto evita atribuir al tamaño global un cambio causado por el filtro.

## Datos y medición

- `tests/benchmark/candidate-matrix-v1/` contiene tres corpus anidados y consultas positivas y negativas de desarrollo y calibración. Son datos sintéticos generados a partir de entidades y reglas v5, con tareas nuevas deterministas. No representan tráfico humano ni tienen etiquetas revisadas por una persona.
- El encoder multilingüe del snapshot Laya compartido por SOC generó los vectores en MPS. Los lotes y vectores completos permanecen en `.local/pilots/candidate-matrix-v1/`, fuera de Git; un manifiesto liga archivos y perfil. La carga del modelo y la indexación se miden por separado del tiempo por consulta.
- La muestra de recuperación congelada contiene 100 positivos por escenario, además de 20 o 40 negativos. Se ejecutó una sola vez por tamaño y por presencia de entidad. Sobre cada resultado se calculan los cuatro topes sin repetir la búsqueda. [Resumen validado](../../outputs/iromem/evaluation/candidate-matrix-v1/retrieval-summary.json).
- Para clasificadores se usarán los mismos candidatos congelados. Se compararán Laya local, Jev 1.13 y OpenAI GPT-6 Luna Decisions en versiones fijadas, con varias instrucciones y estructuras de respuesta. El desarrollo servirá para explorar; la calibración comprobará la variante elegida sin reajustarla. Se congeló además una partición final propia con veinte entidades no usadas en la pantalla, 400 positivos y 80 negativos por ámbito; sus consultas y vectores están en `.local/pilots/candidate-matrix-v1/final/` y aún no se han buscado ni clasificado. La partición final v5 también permanece cerrada.
- Medir p50, p95 y p99 de generación de vector, búsqueda, decisión y composición; informar también carga/calentamiento, tokens y coste remoto informado o reservado. No presentar la suma de tiempos aislados como medición real de una integración Codex que todavía no existe.

## Límites de la campaña

No se activa un hook, servidor ni memoria real. Las llamadas remotas usarán solo casos sintéticos y diario reanudable. Por indicación expresa del usuario, esta investigación no tiene tope monetario: se registra el coste informado o reservado y se comunica junto a los resultados. Cada invocación tiene un límite técnico de tiempo y número de llamadas para poder reanudarla y auditarla, sin imponer una duración total a la investigación. Una respuesta incierta detiene la fase hasta resolverla. La comparación de formulaciones comienza con una muestra de desarrollo antes de ampliar llamadas. Los escenarios cuyo recuerdo correcto no llega al tope tienen un techo de exactitud conocido; la clasificación se medirá allí como diagnóstico, sin atribuirle las omisiones del buscador.

## Lectura inicial de recuperación

| Recuerdos | Entidad | Correcto en top 5 / 10 / 15 / 20 (de 100 positivos) | p50 búsqueda |
| ---: | --- | --- | ---: |
| 100 | conocida | 50 / 74 / 87 / 100 | 4,74 ms |
| 100 | desconocida | 39 / 71 / 78 / 80 | 20,14 ms |
| 200 | conocida | 54 / 80 / 88 / 100 | 5,03 ms |
| 200 | desconocida | 39 / 69 / 76 / 81 | 39,84 ms |
| 2.000 | conocida | 50 / 78 / 88 / 100 | 8,85 ms |
| 2.000 | desconocida | 46 / 54 / 70 / 79 | 393,15 ms |

Son mediciones de la muestra sintética y del perfil experimental Laya en esta Mac, no estimaciones de producción. El coste del coseno exacto sin filtro de entidad crece mucho a 2.000 recuerdos; el perfil de embedding definitivo sigue pendiente.

## Pantalla local de Laya

La [pantalla de clasificación](../../outputs/iromem/evaluation/candidate-matrix-v1/laya-choice-screen-summary.json) registró 2.016 decisiones: seis escenarios × cuatro topes × tres formulaciones × 28 preguntas. En los 480 resultados positivos por formulación, acertó 1 con `plain`, 1 con `conditional` y 8 con `structured`; en los 192 negativos por formulación eligió `none` en 146, 161 y 116 casos respectivamente. Los mismos casos se repiten entre topes y tamaños, de modo que esas cifras son diagnóstico emparejado, no 480 preguntas independientes. Los tiempos p50 de decisión fueron aproximadamente 47–50 ms después de cargar el modelo (2,19 s) y calentarlo (130 ms). El desempeño positivo es insuficiente incluso en escenarios donde el recuerdo correcto estaba entre los candidatos; se revisarán formulación y adecuación del modelo antes de cualquier conclusión sobre uso real.

## Pantalla remota de desarrollo

El [resumen de desarrollo](../../outputs/iromem/evaluation/candidate-matrix-v1/remote-development-summary.json) contiene 144 celdas de modelo (72 cruces × dos modelos), 2.232 llamadas planificadas y 2.235 intentos registrados tras tres reintentos explícitos. El coste informado o reservado fue US$0,133294. Los diarios conservan un HTTP 503 y dos pérdidas de respuesta como intentos separados; no se contó una decisión fallida como correcta.

Con entidad conocida, veinte candidatos y formulación `plain`, OpenAI acertó 11/12, 12/12 y 12/12 positivos para 100, 200 y 2.000 recuerdos; Jev acertó 11/12 en los tres tamaños. OpenAI eligió `none` en 3/4 negativos y Jev en 4/4 por tamaño. Esas son doce preguntas positivas distintas repetidas entre tamaños. En la muestra de desarrollo sin entidad, el recuerdo correcto estaba entre los veinte candidatos solo en 6/12 positivos por tamaño, lo que impide alcanzar 95 % final allí con cualquier clasificador.

La [política provisional congelada tras desarrollo](../../outputs/iromem/evaluation/candidate-matrix-v1/development-policy.json) usa búsqueda híbrida con filtro de entidad cuando se conoce, máximo veinte candidatos y OpenAI `plain`. Se eligió antes de revisar calibración. La validación de composición en entidades inéditas sigue pendiente.

Como comprobación de integración sobre los 16 casos de desarrollo del corpus de 2.000 con entidad conocida, `compose-seed-batch` volvió a ejecutar la búsqueda y validó que la elección se había presentado con la revisión vigente. Compuso el recuerdo correcto en 12/12 positivos, semilla vacía en 3/4 negativos y conservó hecho, condición y fuente declarada en los 12 aciertos. La composición midió p50 de 9,18 ms y máximo de 27,65 ms en esta corrida. Aún falta medir el tiempo total de una petición real que incluya vectorización y llamada remota.

El [resumen de calibración](../../outputs/iromem/evaluation/candidate-matrix-v1/remote-calibration-summary.json) registra 1.658 intentos para 1.656 llamadas planificadas y US$0,097491 informado o reservado. Con entidad conocida, tope veinte y `plain`, OpenAI y Jev acertaron 8/8 positivos en cada tamaño; Jev eligió `none` en 4/4 negativos y OpenAI en 3/4. Son ocho preguntas positivas distintas repetidas entre tamaños. Se mantiene congelada la política provisional; la prueba final usará entidades no vistas en esta pantalla y comparará ambos modelos.

## Búsqueda en la partición final

En las veinte entidades finales inéditas, el [resumen de recuperación](../../outputs/iromem/evaluation/candidate-matrix-v1/final-retrieval-summary.json) muestra cobertura de 190/295/342/400 de 400 positivos para topes 5/10/15/20 con entidad conocida. Sin entidad, la cobertura fue 221/252/290/320; el tope veinte se queda en 80 % antes de clasificar. La búsqueda con entidad midió p50 8,45 ms y p95 11,06 ms; sin entidad, p50 453,56 ms y p95 937,80 ms. La clasificación, composición y latencia total finales se detallan en el [informe de resultados](candidate-matrix-v1-results.md).
