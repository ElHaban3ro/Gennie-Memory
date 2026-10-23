# Recuperación de contexto sin respuesta inventada

La política experimental `classify-documentary-context-v2` recupera 23/39 recuerdos útiles frente a 16/39 del mismo recorrido automático anterior. Conserva todas las selecciones previas y selecciona cero de los 60 recuerdos ajenos. Es una mejora local de siete recuerdos, no el cumplimiento del objetivo: aún hay 16 omisiones, ninguna respuesta completa y una referencia ambigua separada. El control con representación manual conserva 36/39.

Se añaden rutas parciales para eventos con sujetos anónimos asociados a una entidad anclada y para estados documentales con fundamento pendiente. Conservan los eventos/proposiciones y calificadores fuente, explicitan las deudas de identidad, evento, fundamento y valor, y no producen términos de respuesta. La comparación estricta existente permanece disponible con comportamiento idéntico; no cambia el producto ni su configuración vigente.

La primera variante recuperó cinco útiles pero seleccionó indebidamente una medición programada ante una pregunta sobre realización. Se conservan sus resultados como fallo. La corrección aplica el alcance tipado de truth_status. Una revisión crítica separada reprodujo además dos mecanismos: atajo alfa que omitía enlaces de propietario y dominios nombrados comparados solo por clase. Se conservaron tres contraejemplos válidos y un control. El binario corregido rechaza los cuatro y la revisión de seguimiento confirma ambos arreglos.

| Medida de desarrollo | Comparador anterior, misma extracción | Contexto corregido |
| --- | ---: | ---: |
| Útiles seleccionados | 16/39 (41,0%) | 23/39 (59,0%) |
| Selecciones ajenas | 0/60 | 0/60 |
| Respuestas completas emitidas | 0 | 0 |
| Concordancia histórica, incluida ambigua | 76/100 | 83/100 |
| CLI p50 | 11,588 ms | 11,597 ms |
| CLI p95 | 19,699 ms | 21,278 ms |

La segunda revisión conserva los cinco útiles recuperados y añade otros dos: normalización del portador cardinal permitido (clase del dominio frente a variable ligada), solo para contexto existencial exacto anónimo sin restricciones adicionales, y transporte del mismo modo/atributo numérico ante una pregunta con forma de propiedad. No afirma identidad de grupos, equivalencia de predicados ni satisfacción de la premisa; conserva ambas representaciones y sus deudas. La revisión separada ejecuta diez controles: cuatro fallos previos, portadores anónimos/nombrados, asignación real y tres negativos de condiciones/negación. Todos los veinte documentos validan completos y los diez controles pasan.

Los tiempos excluyen extracción/revisión/red; son observaciones de CLI y no latencia de producción ni pase de recursos. Los 100 pares se ejecutaron en cada brazo alternando orden, con entradas automáticas y referencias congeladas. Los 100 informes del brazo anterior son idénticos por bytes a su ejecución anterior.

Pasan 705 pruebas del workspace (24 omitidas), 41 enfocadas en contexto, formato y compilación. Los 226 informes históricos permanecen idénticos. Se conservan fuentes, hashes del binario, entradas/salidas y fallos de fixtures. Cero nuevas llamadas externas y USD 0 en ese ledger; coste Codex desconocido. No hay adopción, contraste con baseline de producto, recursos completos ni apertura de V6. El objetivo sigue activo en el paso 3.

Evidencia: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/context-utility-v3/`, con `report.json`, `verification.json`, fuentes e informes. La primera variante fallida permanece en `context-utility-v1/` y la corrección intermedia de 21/39 en `context-utility-v2/`; los contraejemplos y seguimiento en `context-route-critical-review-v1/`.
