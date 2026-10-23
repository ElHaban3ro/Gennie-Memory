# Crítica de reparación del diseño preposicional V2

Estado: reparación conceptual sustancial, todavía no aceptable como contrato cerrado para implementar. La calificación contextual positiva y la propiedad estructural del host resuelven el defecto central de V1. Persisten ajustes de procedencia, vocabulario y elegibilidad; envolver los hosts recibidos sin corregirlos no completa ninguna de las tres fuentes. No se propone una candidata ni se declara ejecución.

## 1. Frontera de revisión

Se releen únicamente la crítica original de este revisor y `nacar-source-design-v1/design-v2.md` con `receipt-v2.json`. Se usa el conocimiento fuente ya recibido de los tres textos/raw; no se abren otros inputs, preguntas, contrapartes, gold, V6, código, pruebas ni matching. Se conserva la incidencia de lectura del vocabulario embebido declarada en la crítica original; esta revisión no restablece retrospectivamente una frontera ciega. El recibo V2 declara además que su autor vio incidentalmente pruebas inline: esa incidencia permanece visible y no acredita independencia ni fuente-only estricto.

## 2. Dictamen por bloqueante original

| Bloqueante | Dictamen sobre V2 | Condición pendiente |
| --- | --- | --- |
| B1: sintaxis sin restricción positiva | Reparado conceptualmente. `QualifiedOccurrence(H,P,C)` distingue el compuesto calificado del host desnudo y prohíbe borrado/transferencia silenciosos. La publicación conserva el host y el adjunto inseparables. | No convertir la nueva etiqueta de witness en prueba de fidelidad. El carrier debe conservar el contenido bajo los operadores reales del host; su nombre no basta. |
| B2: deuda autoral confundida con indeterminación fuente | Reparado en forma. `whole_host_only`, `source_open` y `author_assessment` separan tres afirmaciones diferentes. La reserva permanece autoralmente pendiente. | `whole_host_only` no puede eludir una adscripción nominal más precisa efectivamente expresada por la fuente. La revisión fuente pendiente debe acreditar que la representación conserva el compromiso completo que reclama. |
| B3: región sin host/incidencias | Reparado para el alcance externo propuesto: el Body poseído conserva roles e incidencias y no inventa una entidad para la cláusula. | Corregir el intervalo externo de la reserva y no confundir preservación estructural con corrección semántica del Body original. Las adscripciones internas resueltas siguen fuera del alcance. |
| B4: dependencia sin ownership/cierre | Reparado como arquitectura conceptual. El claim exterior posee wrapper y host; qualifier_ids externos e internos y cierre efectivo se conservan. El backpointer no crea ciclo de requisitos. | Toda lectura de host debe pasar por ese claim exterior. Reservar únicamente serialización final no bastaría; V2 ya exige la reserva antes de rutas ordinarias. No hay evidencia de implementación. |
| B5: cuantificación/guards/contextos borrados | Reparado para preservación y exclusión del alcance inicial. V2 conserva operadores y no promueve hosts guardados/negativos/cuántificados a hechos positivos. | No llamar soporte general al rechazo conservador de esos contextos. Debe existir un contexto operativo explícito para guardar y publicar los positivos admitidos, sin descargar sus deudas. |
| B6: bloquear todo como única utilidad | Reparado parcialmente. La salida de evidencia contextual afirmada ofrece una operación positiva definida y no solo un diagnóstico. | Ninguna de las tres fuentes tiene ahora acreditada elegibilidad completa. Hace falta demostrar utilidad sobre hosts fieles; renombrar el estado de salida y conservar respuestas vacías no acredita ese resultado. |

La reparación de B1–B5 es una conclusión de lectura del diseño, no de forma ejecutada, fidelidad de los tres inputs o preservación por todas las rutas reales.

## 3. Ajustes imprescindibles antes de implementar

### 3.1. El envelope verifica integridad del original aportado, no su congelación independiente

`frozen_input_utf8`, `expected_frozen_input_sha256` y el hash del wrapper proceden todos del llamador. Si este reemplaza simultáneamente los bytes y ambos hashes, las igualdades previstas siguen pasando. El recibo inverso puede demostrar que no se alteró el raw aportado, pero no que sea el original previamente congelado autorizado. Tampoco un hash demuestra autoría fuente-only o ausencia de exposición a contrapartes.

Ajuste mínimo: describir exactamente esa garantía como integridad y preservación respecto del input aportado. Cuando se reclame pertenencia al paquete congelado, comparar con el hash esperado obtenido de un manifiesto o contexto de campaña ya confiable y ajeno a la entrega candidata; no aceptar como ancla independiente el mismo campo controlado por quien envía los bytes. No hace falta añadir otra cadena de recibos o servicio. Si no hay tal ancla, queda procedencia no acreditada para esa reclamación, aunque la igualdad inversa sea correcta. No bloquear evidencia semántica por motivos de autoría bajo una etiqueta que mezcle ambas cosas: son puertas diferentes y deben informarse separadas.

### 3.2. La preservación del raw no limita las modificaciones del vocabulario

V2 restaura únicamente `vocabulary_sha256` al proyectar el raw hacia atrás, y conserva el vocabulario original dentro del envelope. Eso no exige que los predicados, tipos, atributos, perfiles o relevancias del vocabulario nuevo mantengan sus valores anteriores. Se podría cambiar el significado declarado de operadores previos y aun obtener `inverse_raw_sha256 == original_raw_sha256`.

Ajuste mínimo: en esta adaptación comparar el vocabulario nuevo con el original, permitiendo exclusivamente la adición del perfil preposicional especificado y, solo si faltaba, el perfil de presupuesto previo con forma exacta declarada. No reemplazar un perfil ya presente. Cualquier otro delta es otra corrección explícita con revisión propia. Comprobar hashes canónicos de ambos vocabularios, forma válida de cada uno bajo su contrato y que el raw original apunte al original. El recibo de preservación debe identificar ese delta realmente comprobado; no llamar preservación semántica a igualdad de host JSON bajo vocabulario libremente modificado.

La necesidad de compartir presupuesto no demuestra por sí sola que un perfil de contexto de requests deba ser una dependencia semántica de una representación fuente. Si se reutiliza el contador existente, documentar que se trata de infraestructura de ejecución; no habilitar otros comportamientos por accidente para conseguirlo.

### 3.3. Preservar un host incorrecto conserva también su error

El raw original de Cobre tiene `basis=assignment`, mientras V2 admite solo `description` u `observation` para publicar el host. El propio diseño fuente interpretó el funcionamiento observado como descripción, sin derivar asignación del pretérito. El raw histórico no es autoridad semántica: la fuente manda. La igualdad inversa respecto de ese raw obligaría a conservar assignment en esa adaptación estricta; esa limitación no debe convertirse en obligación de mantener una interpretación equivocada. La reparación es autorar y revisar primero el host base corregido contra la fuente, registrar el delta respecto del antecedente, congelar esa nueva base y después verificar que la envoltura la preserve. No se corrige silenciosamente dentro del wrapper ni se amplía basis para hacer pasar el antecedente. Esa fila queda fuera del alcance positivo mientras falte esa corrección explícita y revisada.

En ese mismo raw, `sin apagarse` es un valor de configuración textual. Mantenerlo idéntico no establece todavía ausencia de apagado durante el intervalo del mismo funcionamiento. El límite relativo de la medición negada pertenece a otra cláusula y sigue necesitando su propia representación. En Zinc, conservar `duration_until_shutdown=100` tampoco acredita que se haya representado correctamente «tras cien minutos» como intervalo anterior con origen no dado; no debe convertirse en duración de funcionamiento o autonomía. En reserva, la mención `batería nueva` y su coverage unsupported no son por sí mismas semántica acreditada de «nueva».

Ajuste mínimo: mantener separadas adaptación preservadora y corrección de autoría, como V2 ya propone. La congelación/procedencia acredita trazabilidad de una versión, nunca sustituye su contraste con la fuente. Hasta recibir esa corrección, registrar las deudas efectivas y no publicar `qualified_contextual_evidence` como si la completitud semántica del host hubiera sido comprobada por validar su forma. Un indicador `complete_for_declared_form` del autor no puede descargar dichas deudas. Si una deuda semántica no es detectable por el validador estructural, su aceptación debe venir de revisión fuente acreditada; no inventar que el cierre del grafo la detectará automáticamente.

### 3.4. El intervalo externo de la reserva contradice la regla de contención literal

La tabla V2 fija external_host en `[0,50)`. El átomo del raw original de la reserva cita la oración completa `[0,51)`, incluido el punto final. La regla V2 exige que external_host contenga toda evidencia del cuerpo. No cabe esa cita conservando los dos valores.

Ajuste mínimo: usar evidencia externa que incluya el punto `[0,51)` para esta cláusula, sin cambiar el host original ni el intervalo del PP `[42,50)`. No hace falta redefinir semántica de adjunción ni añadir excepciones al validador por puntuación. Este es un fallo concreto de coherencia del diseño, independiente de la frontera interna pendiente.

## 4. Mínimo arquitectónico y utilidad real

Un wrapper que posea el Body existente es una respuesta proporcionada a la pérdida de host compuesto; evita reificar cláusulas y una API abierta de localizadores. El único adjunto y la ausencia de wrappers anidados delimitan un primer alcance comprensible. El owner literal duplicado es validación de coherencia y podría computarse; no debe convertirse en una segunda fuente de verdad. El cierre debe reconstruirse y los DTO de recibo/testigo son salidas del mismo cálculo, no nuevos depósitos que requieran conciliación independiente.

No está justificado añadir un framework general de parsing, sentidos preposicionales, incidencias serializadas o identidades. Tampoco hace falta implementar este carrier para corregir cualquier otra pérdida de la campaña: cada caso que quepa fielmente en un carrier existente debe repararse allí. Esta crítica no revisó Faro y no afirma cuál es su forma correcta; es coherente priorizar su reparación con operadores existentes antes de ampliar arquitectura.

Para cerrar B6 no se exige inventar correspondencia con preguntas en este diseño fuente-only. Se exige una operación positiva real: conservar y publicar un host afirmado fiel con su restricción íntegra, distinguiéndolo del desnudo y conservando hechos independientes. El diseño ya define esa salida. Su funcionamiento queda por verificar y la elegibilidad de estas tres fuentes por acreditar; ambas deudas deben seguir separadas. Si todos los hosts relevantes terminan pending/not_supported debido a errores no reparados, eso es evidencia de trabajo pendiente, no éxito del nuevo carrier.

## 5. Cotas y decisión

Las cotas numéricas V2 cierran la indeterminación de configuración, pero son decisiones de diseño sin calibración. Un wrapper por documento puede excluir textos con dos adjuntos; profundidad32 o4.096 endpoints pueden excluir un host legítimo que el contrato general acepte. Es admisible delimitar un primer alcance, pero cada exclusión conserva su caso y denominador. Error de límite de operación, forma no soportada y falta de aceptación semántica son resultados distintos; ninguno cuenta como acierto ni desaparece de los116. Un límite compartido y copias precobradas son obligaciones correctas; la cifra16MiB no demuestra RSS ni estabilidad.

Dictamen: aceptar la dirección de reparación conceptual, devolver V2 para los cuatro ajustes concretos anteriores y mantener implementación y aceptación de fuentes abiertas. No se necesita otra capa de maquinaria antes de corregir esos puntos. No hay fuente completa nueva, extracción autónoma, prueba de matching, medición de recursos ni cambio a la puerta116. Esta revisión escribe únicamente este documento.
