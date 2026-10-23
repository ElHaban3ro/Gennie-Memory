# Asignación cuantitativa ligada al uso en V3

Se implementó una proyección optativa de afirmaciones explícitas `uses(subject,object)` con cantidades en calificadores del mismo claim. El diagnóstico anterior conservaba el uso y su resolución/tasa/exposición, pero no conectaba ese valor con una pregunta cuantitativa sobre el mismo equipo y modo.

El vocabulario admite `assignment_projection:{version:"uses_qualifier_assignment_v1",context_keys:[...]}`. Su esquema es cerrado y comprueba los roles de `uses` y `quantity`. La versión declara que una cantidad literal conocida, adjunta a un uso factual actual y con clave igual al atributo solicitado, es una asignación ligada a ese uso. No añade predicados, acciones, atributos, dependencias, APIs o política de producto.

Se exige sujeto acreditado, afirmación positiva factual en estado efectivo, valor literal del mismo claim, contexto restante resuelto y ausencia de rama/dependencia/premisa pendiente. No se toma un valor de un equipo o claim vecino, ni se deriva uso de admisión, permiso, negación o plan. El valor mantiene decimal, unidad y cota originales. Un nombre compartido con clase no acreditada sigue siendo evidencia calificada.

La correspondencia entre nombre de referencia y texto del modo solo compara la etiqueta de configuración local a esa asignación; no prueba identidad global. Si el modo observado difiere del solicitado, solo se conserva la proposición original como contexto parcial, con `answer_terms:[]`: no se afirma que el equipo jamás use el modo solicitado. Si existe evidencia del modo solicitado, ese contraste adicional no vuelve incompleta su respuesta. Cantidades incompatibles o no equivalentes bajo el mismo modo conservan sus witnesses como parciales y bloquean la respuesta única mediante conflicto potencial/ocurrencia pendiente.

El protocolo congeló los100pares manuales antes de ejecutar. Los textos, etiquetas y frames permanecen iguales; para activar el perfil solo cambia el vocabulario y su hash en raw. La revisión modifica código y metadatos conjuntamente: no se atribuye el efecto a un único factor. El esquema cerrado anterior no puede ejecutar la clave nueva.

| Variante | Referencias útiles seleccionadas | Referencias `none` seleccionadas | Respuestas completas | Concordancia heredada /100 |
| --- | ---: | ---: | ---: | ---: |
| Código anterior, vocabulario anterior | 23 | 1 | 3 | 82 |
| Código nuevo, vocabulario anterior | 23 | 1 | 3 | 82 |
| Código nuevo, proyección activada | 26 | 1 | 3 | 85 |

Los100informes con vocabulario anterior se reproducen byte por byte en ambos binarios. La candidata recupera `cv3q_13/cv3_13_00`, `cv3q_14/cv3_14_00` y `cv3q_16/cv3_16_00`: contexto real de Senda/Nube sin usar valores de Loma/Valle y la resolución600ppp de Cima ligada a Fino. Conserva las23selecciones útiles y tres respuestas completas previas, sin selección nueva entre las referencias `none`.

Pasan **599 pruebas, 24 omitidas; 79 V3**, dos controles nuevos con variantes. Se conservaron la prueba inicial fallida y tres fallos posteriores: el modo alternativo volvía incompleta una respuesta válida y dos cantidades de claims distintos podían publicarse como respuesta única, y una petición negativa podía recibir una asignación positiva. Se restringió la proyección a peticiones positivas con slot omitido. Los controles cubren desactivación, plan, admisión, negación, rama, valor ausente, conflictos dentro/entre claims, metadatos mal formados y acoplamiento correcto de sujeto/modo/valor. Las mutaciones mecánicas se identifican como tales, sin presentarlas como fidelidad de extracción.

El hash final reproduce316informes idénticos:100con configuración anterior,100con proyección y116validaciones de extracción congelada. La auditoría de los tres witnesses nuevos confirma valor ligado al claim y ausencia de términos de respuesta en los dos contrastes de modo.

La puerta global sigue fallando: faltan13referencias útiles no ambiguas (8A,4B,1C), una ambigua permanece separada y sigue una discrepancia de alcance universal. El caso P2/Ópalo/Eclipse conserva anclajes/contexto pendientes; no se fuerza para subir el porcentaje. No se revisó independientemente la nueva regla semántica ni se repitió extracción automática: la última conserva65/116entradas fieles para ambos críticos. Esta concordancia manual85/100no es exactitud del producto ni comparación con baseline.

Cero llamadas API y USD0nuevos; tokens/coste Codex desconocidos. No se reactivaron agentes. Latencia de producción, recursos, estabilidad y V6 siguen sin evaluar. Sin promoción; goal activo en paso3.

[Propuesta](../../outputs/iromem/evaluation/documentary-v3-scoped-assignment-2026-10-21/proposal.json), [protocolo](../../outputs/iromem/evaluation/documentary-v3-scoped-assignment-2026-10-21/protocol.json), [comparación completa](../../outputs/iromem/evaluation/documentary-v3-scoped-assignment-2026-10-21/report.json) y [vocabulario](../../outputs/iromem/evaluation/documentary-v3-scoped-assignment-2026-10-21/vocabulary.json). La comprobación del hash final, trazas y reproducciones se guarda en `execution-provenance-v2.json`.
