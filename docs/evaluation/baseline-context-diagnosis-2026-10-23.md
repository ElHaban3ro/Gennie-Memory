# Diagnóstico de omisión de baseline con contexto

Fecha: 2026-10-23 (America/Bogota). Producto vigente: Gennie Memory.
Estado del diagnóstico original: desencadenante y corrección candidata aislados, todavía sin parche en esa etapa.

Actualización posterior: el usuario autorizó aplicar la corrección al baseline. Ya está implementada y pasó [30 recuperaciones de desarrollo, incluida Aliso](baseline-evidence-30-results-2026-10-23.md). Los ensayos de este documento conservan el estado y los resultados anteriores al parche.

## Hallazgo

La búsqueda recupera y presenta el recuerdo correcto. El proveedor devuelve literalmente `none`; el parser lo conserva correctamente. Con la presentación actual, una instrucción de cautela incluida en el contexto activa la omisión en este caso de desarrollo. Los recuerdos solo aparecen como descripciones de las opciones de respuesta (`questions.relevance.criteria`), mientras `state` contiene consulta/contexto y ámbito, sin recuerdos como evidencia explícita.

Añadir esos mismos recuerdos en `state.memories`, dejando intactos consulta, contexto, opciones e instrucciones baseline, hizo elegir la evidencia en 2/2 ensayos. Esto localiza una corrección candidata en la presentación de evidencia. No permite conocer el razonamiento interno del modelo ni demuestra que sea la única causa de cualquier abstención.

## Reproducción real y minimización

Pregunta: «¿Qué formato debe usar el informe semanal de Aliso?».
Recuerdo: «El informe semanal de Aliso debe usar una tabla de tres columnas: tarea, responsable y fecha.».
Contexto original: «Estamos preparando el informe de Aliso. Si no existe un formato registrado, no inventes uno. Conserva las condiciones del documento.».

Se compiló `gennie-memory` offline y se ejecutó CLI recall-v1 con DB temporal propia. El comando `python3 /tmp/gennie-baseline-loop.py original 3` terminó con código 1: `assert expected memory selected: 0/3`. Cada respuesta recuperó candidatos y terminó abstained. El binario y las funciones del producto son los reales; no se simuló la elección del modelo.

Sin contexto: 2/2 selecciones correctas con ambos recuerdos. Con un único recuerdo y contexto: 0/1. Con ese mismo único recuerdo sin contexto: 1/1. Así se descartó el distractor como condición necesaria del fallo. No se redujo la pregunta hasta perder el dato solicitado.

## Hipótesis contrastadas

Las hipótesis se comunicaron antes de los probes: codificación JSON doble, instrucción de cautela, falta de anclaje de pregunta y parser descartando la elección. Tras localizar el efecto de cautela, se ensayó como refinamiento la distinción entre opciones y evidencia explícita en state.

| Contraste | Selección del recuerdo correcto |
| --- | ---: |
| CLI original | 0/3 |
| CLI sin contexto | 2/2 |
| CLI con un único candidato y contexto | 0/1 |
| CLI con un único candidato sin contexto | 1/1 |
| Replay HTTP del payload original exacto | 0/2 |
| Contexto neutral, conservando envoltorio JSON | 1/1 |
| user_prompt como objeto JSON nativo, mismo contexto | 0/1 |
| Pregunta y contexto en campos separados, mismos textos | 0/2 |
| Instrucción adicional de anclaje a query, mismo contexto | 0/1 |
| Quitar solo «Si no existe un formato registrado, no inventes uno» | 2/2 |
| Quitar solo «Conserva las condiciones del documento» | 0/2 |
| Sustituir la cautela por «No inventes información» | 0/1 |
| Añadir recuerdos explícitos a state.memories, original intacto | 2/2 |

Control negativo del payload candidato: se preguntó quién autorizó el informe, dato que los recuerdos no contienen. Devolvió `none` (1/1), sin forzar una selección por haber añadido evidencia a state. Después se volvió al payload original: devolvió `none` de nuevo; ese control está incluido en los dos replays de la tabla.

Las reconstrucciones del payload de referencia coinciden por SHA-256 con request_payload_sha256 capturado por el transporte Rust. Se variaron campos concretos manteniendo los demás. Los probes HTTP guardaron respuesta cruda antes del parser y recibos. Esto descarta error de mapeo A→ID/none para el caso; tampoco basta cambiar de string a objeto o dividir campos para corregirlo.

## Interpretación y corrección propuesta

La cautela del contexto no afirma que falte el formato; solo dice qué hacer si faltara. Sin embargo, baseline se abstiene con ese contexto y representación. Las opciones contienen el texto correcto, pero no hay una colección de evidencia explícita en state. Añadir esa colección resuelve el caso sin retirar la cautela. Por tanto, la corrección candidata es hacer explícitos los recuerdos como datos de entrada al clasificador y mantener separados los criterios/etiquetas de salida.

No se elimina «no inventes», no se fuerza selección ante abstención y no se reemplaza baseline por per_memory para ocultar la omisión. El efecto observado no equivale a probar una cadena de razonamiento interna del modelo. Los probes de este diagnóstico usaron Decisions; los fallos previos de Jev se conservan, pero esta corrección candidata aún no se contrastó aquí con Jev.

## Límites y estado del arreglo

No se modificó código de producción: el cambio de payload solo se aplicó al ensayo. Falta implementar el arreglo en una frontera apropiada y medir regresiones en desarrollo: ausencia real de respuesta, condiciones, referentes distintos y contexto que sí cambia el dato solicitado. El control negativo único no acredita precisión general. Debe comprobarse coste/tamaño adicional y conservar el comportamiento histórico donde no hay contexto. No cambiar el predeterminado ni reutilizar la reserva final consumida para ajustar esta corrección.

No se escribió una prueba unitaria que finja la decisión del proveedor: eso comprobaría el mock, no esta omisión semántica. El bucle real y los payloads/receipts capturados son la prueba roja actual; la futura prueba estructural debe asegurar que la colección explícita de evidencia llega al payload real, junto con una comprobación semántica separada.

## Recursos y evidencia

22 llamadas sintéticas, **USD 0,0006034** informados, recibos completos y cero llamadas inciertas. Tiempo de peticiones activas 13,763 s; construcción y análisis se miden aparte. Límites registrados: 30 llamadas, USD 0,02 y 180 s activos. Se eliminó cada DB temporal y no se inició servidor ni se usó la base del usuario. No se abrieron calibración/final ni se cambiaron etiquetas congeladas. La revisión humana permanece aplazada.

- [Resumen y resultados por variante](../../outputs/gennie-memory/evaluation/baseline-context-20261024T035733Z/summary.json).
- [Diario sincronizado](../../outputs/gennie-memory/evaluation/baseline-context-20261024T035733Z/loop.jsonl).
- [Hashes del código/binario observado](../../outputs/gennie-memory/evaluation/baseline-context-20261024T035733Z/source-snapshot.json).
- [Prototipos de diagnóstico preservados](../../outputs/gennie-memory/evaluation/baseline-context-20261024T035733Z/debug/README.md).

La instrumentación está aislada en debug; no se añadieron logs al producto ni dependencias. Las únicas modificaciones del repositorio de esta tarea son documentación de diagnóstico y seguimiento; se preservó el renombrado y demás cambios existentes. No se ejecutaron suites largas al no cambiar implementación. El build offline y los bucles CLI/HTTP son evidencia de ejecución, no despliegue ni aprobación de una política nueva.
