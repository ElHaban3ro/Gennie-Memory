# Crítica semántica del diseño de medición negada con «hasta»

Fecha: 2026-10-22. Alcance: revisión previa a código de `documentary-v3-until-measurement-design-2026-10-22.md` y de las dos fuentes exactas reproducidas allí. Se consultaron únicamente fuentes lingüísticas primarias para contrastar construcciones. No se inspeccionaron código, contratos operativos, pruebas, resultados, preguntas, gold ni otras críticas. Las comparaciones con el carrier anterior son condicionales a la descripción que hace el diseño; esta revisión no verifica ese contrato.

## 1. Decisión

La dirección composicional es defendible: conserva una relación de límite, un predicado de apagado, una medición negada y su objeto sin afirmar eventos independientes. El contexto de Lino tiene una selección parcial positiva posible por su propietario explícito. No corresponde sustituir todo por una cita opaca ni exigir un motor temporal completo.

El documento necesita tres reparaciones previas a declarar cerrado el diseño: retirar la elección gramatical `impersonal_se`, especificar el estatuto y los tipos de la expresión abierta y hacer explícita la operación positiva de selección por propietario. La justificación de una variante de Body frente a `Claim.context_bindings` sigue siendo condicional: falta describir por qué el carrier existente no puede conservar una obligación inseparable de este contenido sin permitir ejecutar su núcleo por separado. Esta crítica no autoriza implementación ni adopción.

## 2. Adjunción y alcance: diferencia material, no lista de posibilidades

Las dos interpretaciones relevantes tienen consecuencias distintas:

- Lectura de la descripción: la magnitud no medida es la duración hasta el apagado. Que se haya medido otra duración antes del apagado no contradice esta lectura.
- Lectura temporal de la cláusula: no se midió la duración antes del límite de apagado. Una medición anterior de esa misma duración contradice esta lectura; una medición en el límite o después no se excluye por negar el tramo anterior.

El contraste importa en la fuente de Lino. Una situación con una duración ya medida antes del apagado, pero sin medir la duración delimitada por este, distingue ambas interpretaciones. Es un contramodelo de la equivalencia de las lecturas, no evidencia de que la fuente afirme ambas. No se debe convertir en material nuevo del corpus ni en una alternativa afirmada.

El análisis temporal es coherente con el mecanismo general de `hasta + infinitivo` y con la interacción entre negación y límite. La RAE describe esta forma como introductora de subordinadas «de significado temporal». Esa regla general no acredita por sí sola cada control de sujeto ni demuestra una lectura nominal concreta de estas dos frases. La admisibilidad específica de las adjunciones es aquí juicio lingüístico sobre las fuentes, con distinta naturalidad, no una lista exhaustiva derivada de la RAE. Conviene decirlo en §2: sustituir la afirmación sin matiz de «incertidumbre interpretativa real» por una explicación de las dos paráfrasis y de la diferencia de condiciones que las hace relevantes. No hace falta exigir otro mecanismo o otra campaña para conservar el adjunto abierto.

La negación adscrita a medir puede mantenerse como esqueleto fijo `c0`, siempre que `c0` sea una subexpresión no ejecutable. La estructura abierta debe poder restringir el dominio temporal de esa negación cuando la adjunción sea alta y restringir la descripción de cantidad cuando sea baja. No basta con que un futuro consumidor añada un modificador al objeto manteniendo siempre una negación global ya evaluada. El diseño ya prohíbe esa evaluación; conviene expresar la consecuencia positiva: la unidad interpretada es `Compose(c0, Until(a, SwitchOff(s)))`, con `a` abierto, y nunca `Assert(c0)` más un contexto prescindible.

No hay contramodelo que obligue a introducir un evento posterior afirmado. Tampoco corresponde deducir la ausencia absoluta de apagado: mantenerlo como predicado no ocurrido es suficiente. Las restricciones de §7 sobre ochenta minutos y Nacar siguen justificadas por la separación de incidencias.

## 3. Elección gramatical adicional que debe retirarse

`voice: {kind:"impersonal_se"}` y `negative_impersonal_measurement_with_until_v1` van más allá de conservar el `se` de la fuente. «Se midió la duración» admite el análisis de pasiva refleja, con el nominal de cosa como sujeto paciente. La forma singular no demuestra impersonalidad sintáctica. La distinción no es meramente terminológica si luego se usa el sujeto gramatical para controlar «apagarse».

La RAE distingue la pasiva refleja, cuyo nominal es «su sujeto gramatical», de la impersonal, que carece de él. El hecho común útil al diseño es que no se expresa quién mide. Reparación mínima:

- Renombrar la familia como `negative_se_measurement_with_until_v1`.
- Sustituir la etiqueta por `se_marker` o una etiqueta declaradamente descriptiva de agente no expresado, con la misma evidencia.
- Declarar que `object` es el rol semántico de magnitud medida, sin resolver la función de sujeto/complemento directo del nominal.

No hace falta incorporar dos árboles de voz, un agente anónimo ni otro argumento. La corrección reduce compromisos manteniendo los operandos actuales.

## 4. Composición positiva y contrato de tipos

El árbol supera una cita opaca: identifica cuál predicado está negado, cuál es su objeto, qué entidad porta la duración cuando se expresa, qué conector aporta límite y qué predicado ocupa su complemento. Una consulta estructural puede distinguirlo de una negación de funcionamiento o de una descripción de peso aun conservando todas las deudas. Es contenido composicional positivo; no equivale a afirmar que existe una medición ni un apagado.

La gramática está cerrada en claves y cardinalidad, pero cerrar JSON no termina el contrato semántico. Dos precisiones mínimas evitan interpretaciones incompatibles:

1. `quantity_description` debe declararse como tipo interno de expresión, admisible en `measure.object`, cuyo atributo fijo es `duration` y cuyo propietario es exactamente una de las dos variantes. No es por aparecer una cantidad medida, un valor numérico, una referencia V3 ni una entidad existencial. Si el vocabulario usa tipos para validar firmas, esta declaración debe entrar en el perfil/hash junto a la firma. Nombrar el tipo en una firma sin declarar su naturaleza deja abierto un contrato nuevo.
2. La adjunción es un argumento abierto de la composición cuya región es exactamente `core`. Su dominio no incluye el propietario nominal como entidad, otro Claim, las evidencias ni el ID del propio operador. No es necesario enumerar todas las lecturas, pero sí separar el dominio semántico de sitios de la colección indiscriminada de IDs. Mientras sea `null`, no hay resolución ni cuantificación existencial sobre un sitio elegido por el consumidor.

La suficiencia actual es de representación abierta, no de evaluación temporal. Sujeto implícito y posesivo son dos posiciones distintas; `null` no las unifica. Mantener sus evidencias preserva el contenido sin crear un portador físico anónimo. No hay fundamento para exigir que el sujeto implícito de Lino quede abierto en todos los usos futuros: el perfil actual lo conserva abierto y una versión de resolución podrá acreditar el vínculo. Tampoco hay fundamento aquí para enlazarlo automáticamente con Lino o con quien mide.

## 5. Consumo parcial que sí hace algo

§6 ya permite anclar el contexto de Lino. Falta dar una regla concreta que muestre utilidad sin escoger argumentos pendientes:

> Si una selección solicita contexto documental de duración para un referente `r`, esta composición puede seleccionarse cuando `owner.kind = explicit_owner`, `owner.term` identifica acreditadamente `r` bajo su cierre efectivo y `attribute = duration`. Se devuelve el Claim completo con su mismo ID, evidencias, operador y posiciones abiertas. La selección acredita pertinencia contextual al portador de la duración; no acredita sujeto del apagado, episodio, valor ni ausencia global de medición.

Esta operación es aplicable a Lino con el mismo carrier propuesto. No necesita resolver adjunción ni sujeto. La segunda cláusula de Cobre no satisface esa condición por medio de `su`: puede seleccionarse por pertenencia documental o de cláusula, mostrando el posesivo abierto, pero no como contexto ya acreditado para Cobre. Esto conserva una distinción funcional positiva y evita que «consumo parcial» signifique únicamente rechazar salidas.

## 6. Coste de Body frente a contexto

La nueva variante es razonable si el cuerpo ordinario afirmaría una negación evaluable por sí sola y `context_bindings` no puede cambiar ese estatuto. En ese caso se necesita una unidad que contenga el núcleo y su operador antes de toda ejecución; añadir un contexto descartable dejaría disponible un hecho más fuerte que la fuente. Este es el contramodelo arquitectónico pertinente: retirar el contexto y conservar una negación global cambia el contenido.

No basta, sin embargo, con decir que la variante actual de contexto solo admite una preposición y un complemento referencial. Eso demuestra que su wire actual no sirve sin extensión; no demuestra que extender Body cueste menos o preserve mejor la unidad que extender el contrato de contexto. La reparación mínima del diseño es formular la condición de decisión anterior y reconocer la alternativa: contexto solo sería suficiente si su presencia transforma la unidad afirmada, es inseparable para todos los consumidores y no deja un núcleo ejecutable. No se propone introducir ambos mecanismos ni rediseñar contexto. Sin inspeccionar su contrato, no puede cerrarse aquí la comparación como hecho vigente.

## 7. Cierre de la revisión

Quedan resueltos en el plano del argumento: utilidad del árbol frente a cita opaca; necesidad de no evaluar `c0` separado; distinción entre propietario y sujeto implícito; selección parcial válida para Lino; prohibición de resolver `su` por proximidad; suficiencia de un predicado de límite sin evento ocurrido.

Bloqueos documentales antes de dar el diseño por cerrado: etiqueta de `se` sobredeterminada; contrato explícito de tipo interno y dominio de adjunción; operación de selección positiva; justificación condicional y verificable de Body frente a contexto. Son reparaciones del texto y del contrato propuesto. No se requieren cuantificadores nuevos, motor temporal, elección de antecedente, enumeración exhaustiva de lecturas ni pruebas de calidad para resolverlas.

## Fuentes lingüísticas consultadas

- RAE/ASALE, [DPD, «hasta»](https://www.rae.es/dpd/hasta), §1. Cita breve: «introduce oraciones subordinadas de significado temporal». Sustenta el mecanismo general; no certifica las dos adjunciones específicas.
- RAE, [«Se venden casas», «Se buscan actores» frente a «Se busca a los culpables»](https://www.rae.es/espanol-al-dia/se-venden-casas-se-buscan-actores-frente-se-busca-los-culpables), apartados a/b. Cita breve: «su sujeto gramatical». Sustenta la distinción entre pasiva refleja e impersonal.
- RAE/ASALE, [Nueva gramática, 28.7a](https://www.rae.es/gram%C3%A1tica/sintaxis/per%C3%ADfrasis-de-infinitivo-ii-negaci%C3%B3n-y-tiempo-en-las-per%C3%ADfrasis-modales-de-infinitivo). Contraste consultado sobre alcance de negación y límite temporal; no se traslada la negación anticipada de verbos de creencia a medir.

Consultadas el 2026-10-22. Las búsquedas externas se limitaron a construcciones gramaticales; no se enviaron las fuentes de Lino/Cobre a modelos externos.

## Dictamen delta sobre las reparaciones

Revisión del diseño actualizado el 2026-10-22, limitada a los puntos de esta crítica. No se consultaron nuevas fuentes ni otros archivos.

- **Voz con «se»: cerrado.** `negative_se_measurement_with_until_v1` y `nonagentive_se`, definidos expresamente sin decidir pasiva refleja frente a impersonal, eliminan el compromiso adicional. `object` queda como rol semántico, por lo que no impone función gramatical ni control del infinitivo.
- **Tipos internos: cerrado.** El diseño declara `quantity_description` como nodo descriptivo del AST y fija sus posiciones, atributo y propietarios admisibles. No lo convierte en valor, referencia o entidad existente. Declarar `measure` y `switch_off` como discriminantes internos versionados evita exigir nuevas filas globales de predicados para expresar estos operandos. La definición de la variante y su feature fijan el contrato de tipos pertinente.
- **Adjunción, alcance y `c0`: cerrado.** La resolución conserva el esqueleto y compone su alcance; no puede ejecutar primero su negación. El dominio del hueco incluye posiciones de predicación/descripción y excluye entidades, evidencias y otros Claims. La prohibición explícita de lectura existencial impide que el consumidor escoja una adjunción conveniente. La aclaración de las paráfrasis distingue incertidumbre local de una supuesta lista gramatical exhaustiva.
- **Body inseparable frente a contexto: cerrado como decisión de diseño.** Se explica el contramodelo: quitar `Until` no preserva necesariamente la verdad de la construcción y por ello `c0` no debe publicarse como proposición separada. Se reconoce que un contexto inseparable podría satisfacer el requisito, sin presentar la elección de Body como resultado de una auditoría o comparación de costes inexistente. No queda un argumento arquitectónico circular en este punto.
- **Selección parcial positiva: cerrado en el contrato propuesto.** La regla nueva tiene un caso habilitante concreto: request referencial `quantity`/`duration` con slot `amount` pendiente, propietario fuente explícito y prueba `Same` bajo el contexto efectivo completo. Devuelve `partial_relevance` transportando el mismo Claim y la composición íntegra con sus deudas; no publica valor ni bindings. No necesita resolver sujeto de apagado, adjunción ni episodio. El posesivo abierto no puede pasar por el propietario explícito, de modo que se cierra el contramodelo de seleccionar Cobre por cercanía. No es una ruta que solo bloquee: existe una condición positiva satisfacible sin resolver los argumentos pendientes.

**Decisión delta:** los bloqueos documentales señalados por esta crítica quedan cerrados en el diseño actualizado. No identifico otra brecha concreta dentro de estos mismos puntos. Esto confirma suficiencia semántica/composicional de las reparaciones propuestas, no admisión real del wire, funcionamiento del transporte V2 ni corte efectivo de consumidores; esos hechos requieren su verificación correspondiente y no fueron objeto de esta relectura. No se abre otra ronda general ni se añaden requisitos.
