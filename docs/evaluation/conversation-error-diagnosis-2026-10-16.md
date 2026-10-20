# Diagnóstico de errores en recuerdos de conversación

Estado: análisis descriptivo de campañas ya observadas. No se modificaron búsqueda, instrucción, clasificador ni umbral. Las cohortes 010–020 permanecen consumidas para cualquier ajuste posterior.

## Evidencia

Los diarios de `recall` con Decisions muestran un fallo persistente en positivos del ámbito `conversation`:

| Cohorte | Semilla correcta | Esperado entre candidatos | Abstención | Elección incorrecta | Negativos vacíos |
| --- | ---: | ---: | ---: | ---: | ---: |
| 002–005, reutilizada | 103/120 | 117/120 | 13 | 4 | 20/20 |
| 010–013 | 101/120 | 116/120 | 17 | 2 | 20/20 |
| 014–017 | 103/120 | 117/120 | 16 | 1 | 20/20 |
| 018–020, final sintética | 77/90 | 88/90 | 12 | 1 | 15/15 |

En 018–020, once de los trece errores tenían el recuerdo esperado entre los veinte candidatos. Cinco errores lo tenían en primera posición. Los dos fallos de cobertura son preguntas sobre el supuesto abandonado (`changed_assumption`). El clasificador informó `complete` en las doce abstenciones; no fueron fallos de transporte. Las preguntas sobre restricción temporal y estilo de respuesta se repiten entre los tipos de fallo, pero el diario no permite atribuirles una causa única. Jev también se abstuvo en nueve de las doce abstenciones de Decisions en esta cohorte; acertó las otras tres.

Por ejemplo, ante «¿Qué estilo de respuesta se pidió en conversation ficticio 018?», la primera candidata decía «se pidió responder con pasos numerados», pero Decisions devolvió `none`. Ante «¿Qué límite temporal se pidió en conversation ficticio 018?», el recuerdo de no publicar durante la conversación estaba en segunda posición y también devolvió `none`. Son casos sintéticos con etiquetas deterministas; no prueban cómo se comportaría una pregunta humana.

## Lectura y siguiente prueba

La falta de cobertura explica una minoría de errores; la abstención con el recuerdo presente es el problema observado más frecuente. Una explicación posible es que la formulación de estas preguntas genéricas y los candidatos del mismo ámbito dificulten la elección. Otra es que algunas etiquetas permitan más de una interpretación. Ninguna queda confirmada por los diarios.

La siguiente prueba debe usar **desarrollo nuevo**: construir preguntas de conversación con hechos y distractores revisados independientemente, variar una sola dimensión por ensayo (formulación de la pregunta o conjunto de candidatos) y comparar cobertura, `none`, elección incorrecta y negativos. Antes de una campaña remota, fijar casos, criterios y presupuesto de tiempo; mantener 018–020 congelada. La [auditoría humana de 48 etiquetas](mvp-final-audit-sample-2026-10-16.md) y una muestra separada de preguntas naturales siguen pendientes para juzgar viabilidad general.

Fuentes: diarios `decisions` de [expansión léxica](../../outputs/iromem/evaluation/keyword-prefix-v1/), [comparación 014–017](../../outputs/iromem/evaluation/full-comparison-2026-10-16/) y [cohorte final](../../outputs/iromem/evaluation/mvp-final-2026-10-16/). Los recuentos se obtuvieron de registros `finished`, comparando `expected_ids`, `retrieval.candidates` y `seed.memories`.
