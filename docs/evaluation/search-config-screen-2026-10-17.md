# Configuraciones antes del siguiente paso del roadmap

Fecha: 2026-10-17. Estado: cribado local de desarrollo terminado; **`keywords` con límite 10 es candidata, sin cambio del predeterminado 20**. Este seguimiento no modifica el cierre limitado de M5 ni convierte los resultados en validación independiente. No se llamó a proveedores.

## Método fijado antes de ejecutar

La [suite](../../outputs/iromem/evaluation/search-config-screen-2026-10-17/suite.json) define seis configuraciones: `keywords` y `natural` con 5, 10 y 20 candidatos. Se usaron exclusivamente los bancos de desarrollo v2 (100 casos, 150 recuerdos) y v3 (36 casos, 36 recuerdos), cada uno en su propia SQLite temporal. No se utilizó 018–020 ni otra reserva para elegir ajustes. Se generaron copias de las consultas cambiando solo `limit`; los originales y sus etiquetas permanecen intactos.

El runner Rust ejecutó tres rondas seriales por configuración y banco, invirtiendo y rotando el orden: **2.448 búsquedas en 36 sesiones**, con una consulta de calentamiento adicional por sesión fuera de las métricas. Todos los tiempos son ida y vuelta por el mismo transporte `session`, sobre el mismo binario. Las bases no cambiaron de hash durante la búsqueda y se eliminaron al terminar. El [reporte con salidas y manifiestos](../../outputs/iromem/evaluation/search-config-screen-2026-10-17/runs/report.json) conserva los hashes y el orden de cada ejecución.

El filtro previo a una promoción exige conservar la cobertura de positivos y de evidencia revisada de refutación o contexto parcial que tenía la referencia `keywords`/20. Se utiliza la [anotación retrospectiva del cierre](m5-closeout-label-audit-2026-10-17.md) para distinguir 13 refutaciones, 9 contextos parciales y 15 casos sin evidencia. No se calcula una nueva exactitud semántica ni se llama falsedad a recuperar evidencia que contradice la premisa. La anotación y los bancos ya son conocidos; esto es diagnóstico de desarrollo.

## Cobertura y volumen

| Configuración | Positivos cubiertos, v2 + v3 | Refutaciones cubiertas | Contexto parcial cubierto | Bytes de resúmenes frente a referencia, v2 / v3 | Decisión local |
| --- | ---: | ---: | ---: | ---: | --- |
| `keywords` / 20, referencia | 99/99 | 13/13 | 9/9 | 100 % / 100 % | Referencia |
| `keywords` / 10 | 99/99 | 13/13 | 9/9 | 76,5 % / 82,9 % | Candidata para comprobar selección |
| `keywords` / 5 | 99/99 | 12/13 | 9/9 | 62,1 % / 42,7 % | Descartada por pérdida de evidencia |
| `natural` / 20 | 99/99 | 13/13 | 9/9 | 419,4 % / 111,8 % | Sin ventaja de volumen |
| `natural` / 10 | 99/99 | 13/13 | 9/9 | 214,9 % / 92,3 % | Sin ventaja común a ambos bancos |
| `natural` / 5 | 98/99 | 12/13 | 9/9 | 109,2 % / 44,8 % | Descartada por pérdida de evidencia |

En las 136 consultas emparejadas, `keywords`/10 emitió **71.400 bytes UTF-8 de resúmenes frente a 90.632** de la referencia: **21,2 % menos**. Son bytes de resúmenes sumados, no tokens, precio ni contexto completo del proveedor. El número de candidatos pasó de 842 a 682. No se estima ahorro monetario ni de tiempo de inferencia con esas diferencias.

Ambas configuraciones con cinco candidatos perdieron en v3 el recuerdo que dice que el jurado todavía no eligió ganador (`cm5v3q_058f2b3995069e2b561b`). Con `keywords`/20 esa evidencia estaba en posición 9. Una puntuación que mirase solo los positivos habría pasado por alto esta pérdida de información correctiva.

## Latencia local comparable

Medianas de los tres p50 y de los tres p95 por corrida, en milisegundos:

| Configuración | v2 p50 / p95 | v3 p50 / p95 |
| --- | ---: | ---: |
| `keywords` / 20 | 0,476 / 1,007 | 0,961 / 1,629 |
| `keywords` / 10 | 0,494 / 0,815 | 0,743 / 1,285 |
| `keywords` / 5 | 0,476 / 0,736 | 1,175 / 1,695 |
| `natural` / 20 | 1,130 / 1,371 | 1,159 / 1,926 |
| `natural` / 10 | 0,907 / 1,117 | 0,772 / 1,242 |
| `natural` / 5 | 0,876 / 1,443 | 0,635 / 0,879 |

La variación entre rondas es visible: el p95 de referencia v3 fue 0,840, 1,629 y 3,525 ms; el de `keywords`/10 fue 0,841, 1,285 y 1,634 ms. No se concluye una aceleración general ni significación estadística con tres rondas. La ventaja comprobada de la candidata es el menor volumen conservando esta cobertura. Los corpus son pequeños y distintos del banco de 2.800 recuerdos; sus tiempos no sustituyen las gráficas reservadas ni las comparaciones de motores del README.

## Decisión antes de avanzar

La única candidata que reduce contexto y conserva toda la evidencia medida en ambos bancos es **`keywords`/10**. El predeterminado del producto permanece en 20. Una lista más corta puede alterar la elección o la abstención del clasificador aunque conserve el recuerdo útil; este ensayo solo midió recuperación.

Antes de adoptar el ajuste corresponde una comparación emparejada de selección 20 frente a 10 con el mismo proveedor y política, anotando apoyo, refutación, contexto parcial e irrelevancia del texto seleccionado. Debe comprobar que no pierde positivos antes correctos ni introduce selecciones irrelevantes; recuperar una refutación no cuenta automáticamente como error. Hay que medir tokens, coste y tiempo remoto reales en esa prueba y mantener separados los fallos de transporte. Después, una nueva aprobación de calidad sigue requiriendo datos y revisión independientes según el cierre de M5.

Verificación: los 17 tests del evaluador pasaron, incluida la prueba que rechaza un recorte que pierda una refutación aunque reduzca bytes y conserve positivos. No se modificó el producto ni se hicieron cambios en el README para presentar esta candidata como mejora validada.
