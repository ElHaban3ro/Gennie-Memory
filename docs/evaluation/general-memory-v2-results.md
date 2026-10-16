# Recuerdos generales v2: ajuste de selección y segunda opinión

Estado: evaluación sintética completada el 2026-10-15 en esta Mac. Laya se usa únicamente para los vectores de la búsqueda híbrida; los clasificadores evaluados son OpenAI GPT-6 Luna Decisions y Jev 1.13. No hay hook ni activación automática en Codex. [Resultados por caso y manifiesto](../../outputs/iromem/evaluation/general-memory-v2/).

**Decisión posterior del usuario:** se descarta la cascada Decisions → Jev. Sus cuatro aciertos positivos adicionales no compensan el falso recuerdo ni la latencia extra. Decisions `direct` con máximo veinte candidatos queda como referencia experimental; su 94,78 % de positivos no cumple todavía el objetivo del 95 %. Jev permanece disponible para comparaciones independientes. Los resultados de la cascada se conservan como evidencia de la prueba, no como política recomendada.
La [configuración base v1](../../config/classifier-profile-v1.json) formaliza esa elección para campañas nuevas: Decisions principal, Jev secundario seleccionable, búsqueda híbrida con Laya para vectores, `choice` con `none`, instrucción `direct` y máximo veinte candidatos. No cambia retrospectivamente las campañas de esta página ni integra el clasificador en la CLI Rust.

## Método

- Se reutilizaron los 2.800 recuerdos, vectores y resultados híbridos congelados de [v1](general-memory-v1-results.md). Cada decisión del mismo caso vio los mismos candidatos, con máximo veinte salvo las pruebas explícitas de segunda lista reducida.
- El cribado de prompts reutilizó el desarrollo de v1: 115 positivos y 20 negativos de cuatro sujetos sintéticos. `direct` fue la referencia; `relation`, `explicit` y `relation_context` fueron variantes nuevas. La última añade la condición declarada al texto del candidato cuando existe.
- Una cohorte de comprobación nueva tomó los sujetos sintéticos 006–009 de cada ámbito: 460 positivos y 80 negativos de 16 sujetos, sin solapamiento de sujeto con el desarrollo o final de v1. Los restantes siguen reservados. La selección y los hashes de entrada se guardan en el manifiesto.
- La política de segunda opinión es fija: Decisions con `direct` recibe hasta veinte candidatos; solo si elige `none`, Jev con `direct` recibe los mismos veinte. Si Jev también elige `none`, el resultado sigue vacío. Las pruebas de topes 5 y 10 para Jev se limitaron al desarrollo.
- Métrica principal: recuerdo exacto en positivos. Negativos vacíos, tiempo de decisión p50/p95 y coste remoto informado se registran por separado. Las latencias no incluyen generar el vector en vivo ni componer la semilla.

## Cribado de desarrollo

| Instrucción | Decisions positivos | Negativos vacíos | Jev positivos | Negativos vacíos |
| --- | ---: | ---: | ---: | ---: |
| `direct` (referencia v1) | 109/115 | 20/20 | 105/115 | 20/20 |
| `relation` | 111/115 | 19/20 | 108/115 | 20/20 |
| `explicit` | 89/115 | 20/20 | 74/115 | 20/20 |
| `relation_context` | 111/115 | 19/20 | No medido | No medido |

Las dos variantes que mejoraron dos aciertos de Decisions introdujeron un falso positivo. Se fijó `direct` para la nueva cohorte antes de ver sus decisiones. El cribado de la segunda lista para Jev, solo en las 25 abstenciones de Decisions del desarrollo, produjo **109/115** positivos con máximo cinco, **111/115** con diez y **112/115** con veinte; los tres dejaron **20/20** negativos vacíos. El tope veinte quedó fijado para la comprobación nueva. Estos datos comparten sujetos con la evaluación v1 y solo sirven para escoger la política.

## Comprobación en sujetos nuevos

| Política | Positivos con semilla correcta | Negativos vacíos | Decisión p50 / p95 | Llamadas remotas |
| --- | ---: | ---: | ---: | ---: |
| Decisions `direct` | 436/460 · 94,78 % | 80/80 | 473 / 696 ms | 540 |
| Decisions `direct` → Jev `direct` tras `none` (descartada) | 440/460 · 95,65 % | 79/80 | 488 / 1.064 ms | 540 + 98 |

Decisions se abstuvo en 18 positivos y 80 negativos. Jev respondió con el recuerdo correcto en cuatro de esos positivos, eligió otro en cuatro y siguió absteniéndose en diez; en negativos siguió absteniéndose en 79 y produjo un falso recuerdo. La cascada cambió nueve decisiones finales. El incremento positivo fue **4/460, o 0,87 puntos porcentuales**, a cambio de un falso positivo y **368 ms más en p95** de decisión. Por ámbito, la cascada logró usuario **100/100**, proyecto **119/120**, conversación **103/120** y agente **118/120** positivos; el falso positivo fue de conversación. La concentración de 17 fallos positivos en conversación sigue siendo el principal problema.

El recuerdo correcto estuvo entre los primeros veinte candidatos de **459/460** positivos. `compose-seed-batch` revalidó las **540** selecciones de cada política contra búsqueda híbrida, ID y revisión. La fidelidad de hecho de la semilla coincidió con la selección: **436/460** para Decisions y **440/460** para la cascada. Entre los aciertos de la cascada con condición etiquetada, **36/36** conservaron la condición; la tasa sobre los 40 positivos etiquetados es 36/40 porque cuatro no seleccionaron el recuerdo correcto. La fuente sintética declarada se conservó en todos los aciertos. La composición aislada de la cascada midió p50 **12,01 ms**, p95 **13,01 ms**.

La campaña nueva registró **1.364 intentos** y **US$0,06679584** informados o reservados, incluidos los cribados de desarrollo, las pruebas de tope y la cohorte nueva. Un primer intento de `relation` falló por DNS dentro del entorno restringido, quedó registrado con reserva de US$0,0001 y se reintentó por la ruta de red autorizada; las campañas terminadas no tienen llamadas fallidas ni inciertas. En la cohorte nueva, Decisions informó **US$0,02885450** y la segunda opinión añadió **US$0,00385447**. Es coste de llamadas, no facturación confirmada.

## Interpretación y límites

El objetivo observado del **95 % de positivos** se alcanzó en esta cohorte sintética con cascada, **pero se perdió un negativo**. El usuario descartó ese intercambio; no se implementará como política del MVP. Solo hay 16 sujetos nuevos, las preguntas proceden de plantillas y las etiquetas no tuvieron revisión humana. Los ámbitos se entregaron ya resueltos; no se midieron permisos de un host, consultas sin ámbito, generación del vector de cada pregunta en vivo ni ahorro neto de tiempo de Codex. Los percentiles son de decisión; no se suman a percentiles independientes de búsqueda y composición para presentar una latencia punta a punta.

Los sujetos sintéticos posteriores al 009 permanecen reservados para pruebas futuras. Para reducir los fallos de conversación y el falso positivo hay que diseñar reglas con desarrollo adicional y comprobarlas en esos sujetos intactos; no conviene ajustar un umbral a partir del único falso positivo observado aquí.
