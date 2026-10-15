# Análisis de errores de la campaña local v5

Fuente: [1.600 consultas sintéticas de desarrollo](../../tests/benchmark/v5/development-campaign-1600.jsonl) y los tres [informes emparejados](../../outputs/iromem/evaluation/v5-campaign-development/README.md). Cada forma positiva aparece una vez; 600 consultas son negativas. Los tres métodos vieron los mismos candidatos y la cobertura semántica fue completa. Este análisis usa etiquetas del generador; no estima comportamiento en preguntas humanas ni abre calibración o final.

## Dónde se pierde la semilla

| Método | Positivos correctos | Positivos omitidos (`none`) | Positivos con otra memoria | Negativos con falsa memoria |
| --- | ---: | ---: | ---: | ---: |
| Top‑1 | 719 | 0 | 281 | 400 |
| Laya `choice` | 136 | 685 | 179 | 32 |
| Laya `noul` condicionado, 0,5 | 494 | 206 | 300 | 400 |

El recuerdo etiquetado estuvo entre los cuatro candidatos en los **1.000 positivos**. En este corpus el cuello de botella medido es la elección/aplicabilidad de la memoria, no su presencia en la lista. Cuando la elección fue correcta, la semilla conservó el hecho, la condición y la referencia sintética de fuente. `choice` reduce los falsos recuerdos negativos a 32/600, pero omite 685/1.000 positivos. Top‑1 y `noul` condicionado aceptan una memoria en 400/600 negativos; los 200 restantes se resuelven por ausencia de candidatos.

Los negativos que requieren abstenerse **pese a haber candidatos** explican gran parte de la diferencia. Para `explicit_no_recall`, `mode_unknown` y `mode_unspecified` (100 casos cada uno), `choice` acertó 99, 100 y 100; top‑1 y `noul` condicionado acertaron cero en los tres segmentos. En `wrong_task`, `choice` acertó 69/100 y los otros cero. Los tres métodos acertaron 100/100 en `absent_entity` y `wrong_scope`, donde los filtros estructurados del host sintético excluyen los candidatos.

En positivos hay un intercambio distinto: `noul` condicionado acertó 98/100 en `long_context`, frente a 59/100 de top‑1 y 1/100 de `choice`. En los otros segmentos positivos top‑1 acertó más; por ejemplo, 203/250 en `direct` frente a 105/250 de `noul`, y 130/150 en `indirect` frente a 74/150. Un enrutador que use la longitud de la pregunta podría explotar el patrón de desarrollo, pero esa hipótesis requiere diseño previo y prueba fuera de estas mismas plantillas.

## Umbral binario: límite de las puntuaciones actuales

[El barrido reproducible](../../outputs/iromem/evaluation/v5-campaign-development/threshold-analysis.json) vuelve a aplicar cada umbral a las puntuaciones **ya guardadas** de Laya `noul` condicionado, sin otra inferencia. Mantiene el candidato con puntuación máxima y devuelve `none` si ese máximo cae por debajo del umbral.

| Umbral | Positivos exactos / 1.000 | Negativos vacíos / 600 | Total exacto / 1.600 |
| ---: | ---: | ---: | ---: |
| 0 | 607 | 200 | 807 |
| 0,5 usado en la corrida | 494 | 200 | 694 |
| 0,75 | 397 | 229 | 626 |
| 0,9 | 187 | 396 | 583 |
| 0,95 | 89 | 566 | 655 |
| 0,99 | 0 | 600 | 600 |

Hay 876 valores de puntuación máxima distintos. Al evaluar todos sus puntos de cambio, el máximo **exploratorio en desarrollo** es 807/1.600 con umbral 0, por debajo de 919/1.600 de top‑1 en esta muestra. El recuerdo correcto ni siquiera ocupa el primer lugar de Laya en 393/1.000 positivos; cambiar solo el umbral no corrige esos órdenes. El mejor umbral depende de cuánto cuesten una omisión y un falso recuerdo; el total exacto de esta muestra asigna el mismo peso a ambos y **no fija un umbral de producto**.

## Siguiente decisión de evaluación

Antes de abrir la partición final, hace falta definir una política de abstención/aplicabilidad independiente de las etiquetas del generador y calibrarla solo en `calibration`. También falta comparar Jev y OpenAI Decisions en las mismas 1.600 consultas con el presupuesto remoto aprobado. Las diferencias de latencia y tokens del [informe local](../../outputs/iromem/evaluation/v5-campaign-development/README.md) no prueban ahorro neto del agente principal sin una integración posterior.
