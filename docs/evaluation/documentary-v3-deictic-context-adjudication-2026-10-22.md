# Adjudicación semántica: posesivo y contexto deíctico original

Fecha: 2026-10-22. Estado: análisis acotado del contrato; no aceptación de una extracción ni revisión de fidelidad por dimensiones.

## Fuentes y límites

Se consultaron exclusivamente:

- `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/nominal-possessive-contract-v1/obligation-content-author-v2/review-packet.json`: vocabulario, guía y su única entrada de texto/raw.
- `docs/evaluation/documentary-v3-possessive-design-2026-10-22.md`: definición de `lexical_use` y del operador posesivo.

Texto único: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.» No se consultaron otras revisiones, código, pruebas, resultados, preguntas, contrapartes, gold, V6 ni fuentes lingüísticas externas. No se ejecutaron Cargo, APIs o campañas. Las fórmulas siguientes son instrumentos de razonamiento, no nuevos datos extraídos, autoría raw ni una especificación ejecutada.

## Conclusión acotada

Un valor deíctico desconocido para quien lee puede conservarse sin resolverlo, siempre que la representación siga dependiendo del **mismo uso y del mismo contexto original**, con todas las restricciones que ese contexto imponga. Desconocimiento epistémico no equivale a libertad semántica para sustituir el referente.

Esta posibilidad formal no prueba que la construcción concreta carezca de una vinculación obligatoria con Ina. Tampoco la cercanía discursiva de Ina prueba por sí sola esa vinculación. Con estos materiales no queda acreditado un contramodelo legítimo de raw verdadero y fuente falsa que mantenga contexto original, deíxis y admisibilidad. La ausencia de tal contramodelo aquí tampoco acredita equivalencia lingüística completa: la identificación de las lecturas y contextos admisibles de la frase sigue sin demostrarse independientemente.

## Qué conserva el raw y qué dice la guía

El raw enlaza `credential` con `possessive_use` mediante `nominal_possessive`. Este último nodo cita la ocurrencia «su» y tiene `denotation:lexical_use`; no es una persona. El núcleo conserva además `nominal_complement_de` hacia el uso nominal «alumna». La suficiencia normativa tiene beneficiario `ina`, condición `credential` y ámbito `faro`. No hay una igualdad entre el referente de «su» e Ina. Que Ina sea beneficiaria de la suficiencia es una relación distinta de ser referente del posesivo.

La guía define `PossNom(H,kappa(U))`, prohíbe elegir `kappa` para satisfacer una pregunta y exige conservar la dependencia contextual. Su `possessive_model_rule` pide preservar `S[kappa] iff R[kappa]`; aclara que dos valoraciones no prueban fidelidad general. El diseño también prohíbe convertir el slot en testigo existencial, universal o identidad global. Estas son restricciones semánticas relevantes del operador.

Sin embargo, «In this source no antecedent is selected by the new raw» describe una elección del raw; no demuestra que el texto no imponga un antecedente. Las afirmaciones del diseño según las cuales esta fuente no fija el valor tampoco son evidencia independiente de ese hecho lingüístico. La regla sobre un antecedente fijado por la fuente es pertinente, pero su aplicación no queda resuelta por repetir la conclusión deseada en la guía.

## Hipótesis formal con contexto fijo

Sean `T` el texto, `U` su ocurrencia posesiva, `c₀` el contexto de emisión original y `w` un mundo evaluado. Definimos como notación de análisis:

```text
k₀ = κ(U,c₀)
S(w,c₀) = interpretación de la fuente en c₀
R(w,c₀) = interpretación del raw en ese mismo c₀
Adm(T,c₀) = c₀ satisface las condiciones contextuales de interpretación de T
```

`k₀` no es una variable libre que un consumidor pueda instanciar. Que su valor no esté disponible en el documento no permite cambiarlo entre fuente y raw, entre consultas, o según el resultado buscado. La cuantificación metalingüística sobre contextos posibles para estudiar el contrato no es cuantificación existencial o universal afirmada por la fuente acerca del poseedor.

Para aislar el posesivo, manténganse constantes los restantes compromisos, sus roles, ámbito, complemento nominal y modalidad. Si la fuente y el raw expresan la misma condición nominal restrictiva dependiente de `κ(U,c₀)`, entonces variar el mundo sin variar `c₀` no crea por sí solo una diferencia entre ellos. En particular, la falta de una etiqueta que diga «Ina» no elimina automáticamente el valor que la función contextual ya tiene. Un término puede tener valor semántico sin que el sistema sepa nombrarlo.

Esto es una equivalencia **condicional**: presupone precisamente que la interpretación del operador coincide con la construcción fuente y conserva su admisibilidad contextual. No demuestra esa premisa por definición. Si `κ` o el operador omiten una restricción obligatoria de la fuente, escribir la misma letra en dos fórmulas no repara la omisión.

## Saliencia, igualdad y admisibilidad

**Saliencia pragmática.** Ina es la persona nombrada y beneficiaria en el discurso inmediato. Esto da una razón textual para considerar su prominencia al interpretar «su». No establece por sí solo una regla de igualdad obligatoria; tampoco autoriza a declarar igualmente admisible cualquier otro referente. No se atribuyen probabilidades ni se construye una lista cerrada de antecedentes.

**Igualdad semántica obligatoria.** Si una interpretación acreditada de la fuente impone `κ(U,c₀)=Ina`, esa igualdad debe permanecer operativa en el raw. Puede estar preservada por una dependencia contextual efectivamente conservada, aunque el consumidor no la haya resuelto; no puede quedar sustituida por una libertad para elegir `κ`. Si la representación admite ignorar esa restricción, pierde contenido. Mantener un hecho independiente acerca de Ina no basta para preservar un enlace obligatorio del posesivo.

**Admisibilidad del contexto.** No basta decir que un referente es de un tipo compatible o imaginable. El contexto debe ser admisible para esa ocurrencia dentro de ese discurso. Si la fuente solo admite contextos que fijan el referente a Ina y el raw admite además contextos incompatibles, existe una ampliación indebida. Pero esa ampliación debe demostrarse; no se sigue de que el campo `anaphora` sea nulo. Tampoco `scope_id:root` demuestra por sí solo que se conserve todo el contexto de emisión: su función documental no especifica la interpretación de `κ`.

La frase aislada suministra cotexto, pero no documenta todo un contexto de emisión ni presenta una regla explícita «su se refiere a Ina». Los materiales no ofrecen un criterio independiente suficiente para decidir si la prominencia de Ina es aquí preferencia pragmática, vinculación obligatoria o restricción de admisibilidad. Declarar abierta la forma no decide cuál de esas posibilidades describe fielmente la fuente.

## Búsqueda de contramodelos

Un contramodelo decisivo debería exhibir `w,c₀` tales que:

```text
Adm(T,c₀)
R(w,c₀) = verdadero
S(w,c₀) = falso
```

La interpretación de `κ(U,c₀)`, el vínculo de su ocurrencia, el ámbito normativo y la modalidad deben ser los mismos en ambas evaluaciones. No se dispone aquí de un testigo acreditado con esas propiedades.

1. **Cambiar el referente entre fuente y raw.** Hacer que el texto use Ina y que el raw use otro valor, conservando solo el nombre escrito del contexto, cambia la deíxis efectiva. No es un contramodelo del supuesto planteado.
2. **Elegir el valor por pregunta.** Dar al slot el referente que permita responder una consulta equivale a una satisfacción oportunista. Contradice la guía y el contexto fijo; no descubre una consecuencia del operador correctamente interpretado.
3. **Comparar dos contextos.** Un contexto donde el referente sea Ina y otro donde no lo sea pueden ilustrar falta de resolución, si ambos son admisibles. No producen raw verdadero y fuente falsa en el mismo contexto. Su mera construcción tampoco demuestra que ambos sean admisibles para este texto.
4. **Añadir una igualdad obligatoria solo a la fuente.** Bajo la hipótesis adicional `S = R ∧ (κ(U,c₀)=Ina)`, se obtiene algebraicamente raw verdadero y fuente falsa fijando `κ(U,c₀)≠Ina` y verdaderos los demás compromisos. Es un esquema condicional, no un contramodelo acreditado: falta probar que esa igualdad pertenece obligatoriamente al significado de la fuente y que el contexto elegido sigue siendo admisible. Si la igualdad es una condición de admisibilidad, el contexto desigual ya queda excluido y el ejemplo falla antes de comparar valores de verdad. Si se trata de contenido obligatorio evaluable como falso, se necesita justificar esa lectura, sin cambiar deíxis entre las dos fórmulas.
5. **Confundir condición normativa con credencial física.** Hacer verdadera la suficiencia de una condición nominal mientras falta un objeto material no refuta la representación del posesivo. La fuente/raw no afirman aquí presentación o tenencia física mediante esa suficiencia. Un contraste fundado en esa inferencia cambia el compromiso comparado.

Los primeros tres intentos no sirven; el cuarto identifica exactamente la premisa lingüística pendiente; el quinto introduce una lectura adicional no acreditada. Esta búsqueda no prueba que un contramodelo sea imposible para toda interpretación legítima del texto.

## Brecha de contrato y reparación mínima genérica

Los materiales ya prohíben la elección oportunista y exigen la dependencia contextual, pero no fijan expresamente el segundo argumento contextual de `κ`, su constancia entre fuente/raw/consumidores ni el criterio compartido de admisibilidad. La notación unaria y la prueba con valoraciones distintas permiten una lectura incorrecta de «abierto» como libre. La brecha es de precisión normativa; no prueba que una implementación concreta cometa ese error.

Reparación mínima propuesta, sin escribir un nuevo raw ni seleccionar antecedente:

> En `PossNom(H,κ(U,c₀))`, `c₀` es el contexto original de la ocurrencia fuente y se conserva junto con su procedencia y ámbito. «No resuelto» significa valor desconocido para el consumidor, no valor libre. Fuente, representación y consumidores conservan la misma dependencia contextual y las mismas restricciones de admisibilidad; ninguna consulta puede seleccionar, sustituir o ampliar su interpretación. La fidelidad se contrasta en un mismo contexto admisible. Dos contextos posibles ilustran no resolución únicamente tras justificar su admisibilidad. Toda vinculación obligatoria que imponga la fuente, expresa o exigida por su interpretación acreditada, debe seguir operativa; si el contrato no puede conservarla o no puede justificarse su ausencia, queda una brecha semántica visible.

Esta redacción no exige enumerar antecedentes ni añadir una persona al raw. Tampoco certifica que el documento transporte de hecho suficiente contexto. La procedencia identifica la ocurrencia; conservar su identificación no demuestra por sí solo la conservación de todas sus restricciones. Si se necesitase un mecanismo nuevo para transportar o resolver contexto, requeriría diseño y revisión separados; esta adjudicación no lo incorpora.

## Lo que no puede certificarse

No se certifica que Ina sea o no sea el antecedente obligatorio, que cualquier otro contexto sea admisible, que el operador propuesto cubra toda la semántica natural de «su», ni que la candidata completa sea fiel. Tampoco se validan aquí los restantes predicados, relaciones, modalidad o completitud general del raw: se mantuvieron constantes para aislar la cuestión deíctica.

La constancia de `κ(U,c₀)` resuelve el error formal de tratar el slot como variable elegible. No resuelve por sí sola la adjudicación lingüística del texto. Sin esa adjudicación independiente, corresponde conservar la incertidumbre del revisor. Este documento no asigna puntuaciones, no acepta el caso, no borra incertidumbres previas y no mueve 111/116 ni ninguna métrica de calidad.
