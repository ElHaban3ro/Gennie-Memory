# Credencial: evidencia pendiente del transporte contextual

Estado: análisis técnico conceptual del caso conocido, sin dictamen nuevo ni adjudicación del antecedente. No se inspeccionó código, pruebas, preguntas, gold, resultados ni otros críticos. No se ejecutaron Cargo, fmt, builds, CLI del proyecto, APIs o servicios. Solo se escribe esta nota.

## Evidencia disponible y pregunta acotada

Fuente: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.» Se consultaron la [nota lingüística](documentary-v3-possessive-linguistic-evidence-2026-10-22.md), el [diseño nominal posesivo](documentary-v3-possessive-design-2026-10-22.md), `review-packet.json` y `followup-linguistic-b.json` de `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/nominal-possessive-contract-v1/obligation-content-author-v2/`. También se leyeron AGENTS.md, las reglas de verificación y los índices de memoria/decisiones del repositorio; no se usó memoria externa.

B retira la inferencia general «referente desconocido implica referente libre». Su condición residual es establecer que el operador conserva exactamente esta ocurrencia U en el mismo contexto original c0. Esta nota aborda esa condición; no solicita otro voto ni considera que una autoridad gramatical tenga que certificar el DTO.

El paquete permite observar los siguientes enlaces documentales:

| Componente | Ubicación y dato presente |
| --- | --- |
| Fuente | `/entries/0/text`; hash declarado en `/entries/0/raw/text_sha256` |
| U | `/entries/0/raw/references/5`: `possessive_use`, `lexical_use`, «su», bytes UTF-8 [60,62), `scope_id:root` |
| H | `/entries/0/raw/references/3`: `credential`, «su credencial de alumna», [60,83), con enlace `nominal_possessive` a U |
| Complemento | H conserva `nominal_complement_de` hacia `student_descriptor` |
| Consumo nominal | `/entries/0/raw/claims/2`: `condition_suffices`, condición H, beneficiario `ina`, base normativa y ámbito `faro` |
| Dependencia | `/entries/0/raw/dependencies`: claim→H y H→U; `/entries/0/raw/identity` vacío |

Esto permite auditar la ocurrencia textual y la inclusión del posesivo en la condición. No demuestra el transporte por un consumidor: no hay en estos materiales un par de entrada/salida de esa operación. Tampoco basta `root` para identificar c0: es un ámbito local reutilizable, no una identidad de emisión o de entorno interpretativo. El hash del texto identifica contenido; dos emisiones pueden compartir contenido y diferir de contexto.

La guía define `PossNom(H,kappa(U))`, prohíbe elegir el referente y declara conservarlo. La nota precisa κ(U,c0). Ambas son especificaciones de la interpretación pretendida. Escribir `S[κ] iff R[κ]` como regla o repetir κ en dos lados postula la equivalencia; no aporta por sí solo evidencia de que la operación concreta respete la dependencia.

## Una prueba local que sí puede discriminar

La acción útil es fijar una operación estrecha: extraer y trasladar **como contexto parcial** la condición de suficiencia, con su construcción nominal íntegra. No hace falta decidir quién designa «su». Hace falta distinguir conservar una dependencia de la fuente de producir una interpretación autónoma en el contexto de destino.

El expediente comprobable tendría tres piezas, aún ausentes como conjunto:

1. **Identificación de origen.** Un registro inmutable que ligue el paquete y su texto a una ocurrencia de documento/emisión, y U a [60,62) de ese origen. Registrar qué significa c0: el entorno de interpretación de esa ocurrencia, incluyendo su cotexto completo disponible. Si hay contexto extratextual desconocido, conservar su dependencia como opaca; no afirmar que el texto y su hash lo reconstruyen. Puede usarse un identificador o handle existente de origen; no se presupone que deba añadirse un campo al raw. Una alternativa precisa es un token opaco nuevo por ocurrencia de entrada, ligado de manera estable a U, su evidencia y el documento congelado. No es un hash global del texto, ni el token resuelve o reconstruye c0.
2. **Operación definida y resultado visible.** Una regla de transformación que copie el cierre claim→H→U y el complemento, mantenga el origen y no vuelva a evaluar U bajo el destino. Su salida debe permitir reconstruir la misma construcción, evidencia, ámbito normativo y vínculo de origen, o remitir de forma verificable al original inmutable. Se comparan entrada y salida por una correspondencia explícita de nodos; los IDs locales pueden cambiar si el mapa retiene la identidad de ocurrencia. Una cita suelta o un flag `preserved:true` no satisfacen esta comparación.
3. **Comprobación adversaria del transporte.** Mostrar la salida del mismo origen al cambiar el entorno del consumidor. La ocurrencia y la dependencia interpretativa deben seguir siendo las del origen. Una salida que sustituya κ por una persona del destino, permita escogerla o publique una suficiencia sobre una credencial sin posesivo debe fallar. Probar también dos orígenes separados con el mismo texto y scopes llamados `root`: no pueden fusionarse sus U por igualdad de bytes, hash de contenido, nombre local o palabra «su».

La tercera pieza no pretende enumerar lecturas legítimas de la oración. Los dos orígenes son controles de transporte, no afirmaciones sobre posibles antecedentes en la fuente estudiada. Un único control pasado tampoco demuestra fidelidad general; permite detectar una clase concreta de sustitución o colisión.

Una derivación local suficiente **para la dependencia contextual de esta operación** mostraría: la selección preserva la construcción nominal completa; el mapa de origen conserva la misma incidencia U; la interpretación del contenido trasladado remite al origen y no aplica resolución del destino; y el consumidor publica solo ese contenido restringido. La justificación de cada paso debe apuntar al mapa o a la regla efectiva de transformación. Una demostración sobre una operación definida establece la propiedad de ese diseño; para atribuírsela al software haría falta además una traza de su operación real. Aquí no se ha visto ninguna de las dos.

## Suficiencia semántica de una ruta opaca bien delimitada

Las condiciones anteriores son suficientes para conservar la dependencia, si la salida funciona como una referencia al contenido de origen y el consumidor la respeta. Puede definirse una operación muy pequeña sin atribuirle comportamiento al software actual:

```text
origin_token := fresh(entry_occurrence)
origin[origin_token] := immutable(source_packet, U_span, H_span, source_context_dependency)
transport(origin_token, claim_id) := source_context_reference(origin_token, claim_id)
```

`source_context_reference` significa «el contenido de esa afirmación tal como depende de su fuente». No evalúa el posesivo en el destino. El lector obtiene la misma ocurrencia del registro inmutable, con la construcción nominal completa y su dependencia de c0. La función de transporte solo crea una referencia; no contiene una operación capaz de sustituir el parámetro contextual. Por composición, cambiar el contexto de destino no puede cambiar la dependencia de la construcción referida. Esto conserva también cualquier vínculo discursivo que pudiera operar en c0, sin decidirlo ni convertirlo en un hecho `Same` del raw.

La prueba no consiste en declarar verdadera una equivalencia semántica global. Se apoya en operaciones limitadas que pueden auditarse: creación de token por ocurrencia, registro inmutable, selección de una afirmación identificada, resolución de la referencia al mismo registro y ausencia de reinterpretación deíctica. Son falsables mediante colisión de tokens, mutación del registro, reconstrucción con pérdida de H→U, resolución en destino o publicación de una condición despojada de esa dependencia. Si una operación adicional interpreta o normaliza el contenido nominal, esta derivación deja de cubrirla y debe examinarse esa operación adicional.

El resultado basta para **retener contenido como contexto de fuente**. No establece que el DTO por sí solo codifique toda la semántica discursiva ni habilita respuesta final, antecedente identificado o inferencia normativa adicional. Tampoco convierte automáticamente la formulación del contrato en esta ruta: el contrato y la implementación tendrían que adoptar esa disciplina, y el expediente debe mostrar que realmente la siguen. No se exige obtener una observación completa del entorno externo.

## Contraejemplo concreto al criterio insuficiente

Supónganse dos emisiones e1 y e2 del mismo texto, cada una con U en [60,62) y scope `root`. En un control semántico se asignan entornos c1 y c2 tales que κ(U1,c1)=a y κ(U2,c2)=b, con a≠b. Esta asignación es un dispositivo de prueba del mecanismo, no una propuesta de antecedentes para Ina.

Un transporte que use solo `(text_sha256, root, start_byte, end_byte)` como identidad contextual produce la misma clave para ambas emisiones. Si después interpreta la construcción en el entorno del destino, puede convertir la condición procedente de e1 en una condición dependiente de b. Conservó todas las letras, el enlace H→U y los offsets, pero sustituyó la dependencia. La igualdad de esos campos no prueba preservación de c0.

El contraejemplo se bloquea manteniendo separada la procedencia de emisión y transportando el vínculo al entorno original. Si la salida conserva una referencia opaca a e1 y no descarga su posesivo, no necesita conocer a. Esto demuestra por qué «no resolver» y «perder» no son equivalentes, y por qué conservar texto sin su procedencia tampoco basta.

## Qué cambia y qué sigue sin establecerse

Una ruta opaca con origen preservado ofrece un objetivo alcanzable: conservar **la expresión fuente con su dependencia contextual**, sin afirmar un valor de κ ni sustituirla por una clase. Puede comprobarse con comparación de estructuras y controles de no sustitución. No exige reconstruir todos los hechos del mundo de c0, ni una adjudicación lingüística de quién es el antecedente. Si c0 no puede materializarse, el resultado debe seguir remitiendo a la fuente y no prometer una interpretación resuelta o autosuficiente.

Para usar esa ruta, la definición operativa necesita precisar origen, mapa y comportamiento del consumidor. Si el transporte ya cuenta con un envelope inmutable capaz de preservar estos datos, bastaría especificarlo y aportar la evidencia correspondiente. Si el formato de salida solo admite H/U y scopes locales sin procedencia recuperable, la propuesta requiere ampliar ese transporte o mantener el consumidor pendiente. No se puede afirmar cuál de estas situaciones existe sin inspeccionar su contrato o implementación; hacerlo quedaría fuera de esta tarea. Esta nota no incorpora ningún mecanismo al producto ni cambia el raw V2.

Ni siquiera un transporte mecánico correcto prueba por sí solo que toda la traducción semántica de la oración es adecuada. Preservar de forma opaca la construcción original evita resolverla de nuevo; convertirla en una representación autónoma exigiría justificar aparte la correspondencia composicional y cualquier restricción discursiva que se afirme haber codificado. Esta nota no adjudica esas restricciones, no declara aceptación integral y no demuestra un defecto del raw.

Acción comprobable siguiente: preparar, antes de otro juicio semántico, un expediente pequeño de entrada, salida y mapa del transporte propuesto, con los dos controles anteriores y una declaración exacta de su dependencia de origen. En el estado observado solo hay especificación, anclas textuales y una condición de aceptación; falta esa evidencia de operación. No se cuantifica calidad ni se modifica ningún dictamen.
