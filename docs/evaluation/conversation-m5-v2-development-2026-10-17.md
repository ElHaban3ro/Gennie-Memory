# M5: segundo banco sintético de desarrollo de `conversation`

> Corrección de cierre, 2026-10-17: se conservan el protocolo, los datos y las puntuaciones históricas. Las selecciones discordantes con una etiqueta negativa no prueban una afirmación falsa; algunas preservan evidencia que refuta la pregunta. Las conclusiones de mejora de verdad o superioridad derivadas solo de esa abstención quedan retiradas. Véase el [cierre de M5](m5-closeout-2026-10-17.md).

Estado: medido el 2026-10-17; **solo desarrollo**. Este informe no modifica el clasificador, el buscador predeterminado ni la decisión de viabilidad. El [protocolo](conversation-m5-v2-development-protocol.md) y la [revisión adversarial ciega](conversation-m5-v2-agent-audit-2026-10-17.md) precedieron a las ejecuciones. Los datos y sus hashes están en el [fixture](../../tests/fixtures/conversation-m5-v2-development/README.md); los diarios y reportes reproducibles están en [`outputs/iromem/evaluation/conversation-m5-v2-development-2026-10-17`](../../outputs/iromem/evaluation/conversation-m5-v2-development-2026-10-17/).

## Banco y búsqueda local

Se crearon cinco conversaciones ficticias nuevas (`026`–`030`), cada una con treinta recuerdos, quince preguntas positivas y cinco negativas: **150 recuerdos y 100 preguntas**. Las etiquetas son `agent_synthetic_unreviewed`: un agente redactó los casos y otro intentó refutarlos sin ver salidas de iROMEM. Se corrigió una pregunta semánticamente inválida y se reforzaron diez negativos antes de buscar. Cinco preguntas ambiguas quedaron fuera de puntuación. El validador comprueba referencias, estructura y SHA-256. Las preguntas positivas siguen siendo relativamente literales y el banco no contiene revisiones sucesivas, condiciones, vigencia ni permisos.

| Ruta local | ID esperado top 1 | Top 5 | Top 20 | Negativos sin candidatos | Búsqueda p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `keywords`, ruta vigente | 74/75 | 75/75 | 75/75 | 2/25 | 0,470 / 0,964 ms |
| `natural`, diagnóstico | 74/75 | 74/75 | 75/75 | 0/25 | 1,036 / 1,314 ms |

Que una búsqueda devuelva candidatos para una pregunta negativa **no** significa que haya una respuesta válida. En este banco la cobertura de la ruta vigente ya es completa a top 5, de modo que la intervención prioritaria es la selección y la abstención, no ampliar la búsqueda. Estas latencias son de la búsqueda local en una SQLite temporal, no del trayecto remoto completo.

## Campaña Decisions de 100 casos

Con la ruta `keywords` y el clasificador Decisions vigente: **96/100 correctos**; 74/75 positivos correctos y 22/25 negativos vacíos. Los 75 positivos tenían el ID esperado entre candidatos. Uno de los 75 positivos se degradó por `transport_error` antes de una respuesta del proveedor; se conserva como fallo de campaña y no se reintentó. En los otros 74 positivos hubo 74 aciertos. Los tres fallos de selección verificables son falsos recuerdos en negativos:

| Caso | Pregunta pide | Recuerdo elegido afirma |
| --- | --- | --- |
| `cm5v2q_ee0d52649ceeffc138e1` | Recuento final de aves migratorias | Recuento preliminar de dos |
| `cm5v2q_83eac9f1cc94898beab7` | Licencia de emisión comercial concedida | Se pidió información sobre la licencia |
| `cm5v2q_c8df100909c8daf7fbc0` | Proveedor adjudicado | Se compararon dos proveedores sin elegir uno |

No hubo fugas de IDs prohibidos. El tiempo total registrado fue p50 **368,8 ms** y p95 **555,6 ms**. El coste informado por el proveedor fue **US$0,0032522** en 97 casos con coste reportado; no se debe imputar coste cero a los demás. La calificación original de estas tres selecciones como «falsos recuerdos» era más fuerte que la evidencia: una auditoría retrospectiva ciega de dos revisores las clasifica como dos casos de contexto parcial y una refutación acotada a la reunión. Las puntuaciones originales contra `expected_ids: []` se mantienen intactas, pero no miden si la evidencia era útil como parcial/refutación. Véase la [auditoría retrospectiva de relaciones](conversation-m5-v2-relation-audit-2026-10-23.md). El resultado no prueba que una regla léxica sobre palabras como «final» sea suficiente.

## Comparación Jev focalizada

Después de observar esos tres fallos se seleccionaron 16 preguntas del mismo banco: los tres errores y controles deterministas de los cinco ámbitos. La [selección y los hashes](../../outputs/iromem/evaluation/conversation-m5-v2-development-2026-10-17/jev-probe-selection.json) declaran expresamente que **no es una validación independiente**. Sobre esos mismos 16 casos Decisions acertó 13/16 y Jev 15/16; ambos acertaron los diez positivos. Jev se abstuvo correctamente ante el recuento preliminar y la licencia solicitada, pero también eligió el recuerdo de proveedores comparados cuando se preguntaba quién ganó. En los seis negativos Jev dejó cinco vacíos. Su tiempo total fue p50 **371,1 ms**, p95 **500,9 ms**; coste informado **US$0,000357588** en 15 casos. Esta muestra elegida después de mirar Decisions no justifica cambiar el proveedor ni estimar una tasa poblacional.

## Decisión de desarrollo

Conservar `keywords` y Decisions como referencia. El próximo experimento útil es una variante explícita de selección que distinga hecho documentado de propuesta, solicitud, prueba o dato preliminar, con la pregunta y los candidatos congelados y una sola variable modificada. Se debe comprobar caso a caso si corrige los tres falsos recuerdos sin perder positivos ni crear otros falsos recuerdos, y después contrastarla en conversaciones de calibración separadas antes de congelar una ruta. Las etiquetas sintéticas necesitan revisión independiente para validar el banco; el usuario no tiene que clasificar cien preguntas manualmente. M5 permanece abierto: faltan calibración, reserva y preguntas humanas revisadas.
