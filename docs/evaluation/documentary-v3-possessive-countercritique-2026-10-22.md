# Contracrítica semántica del posesivo nominal

Fecha: 2026-10-22. Estado: crítica independiente de la dirección de diseño comunicada; no aprobación de un esquema final, implementación ni fidelidad automática. El archivo de diseño aún no existía al iniciar esta crítica. No se ejecutaron Rust, Cargo, APIs, clasificación contra preguntas ni campañas. Solo se modifica este informe.

## Material y límites

Se leyeron `AGENTS.md`, reglas backend/testing, índice de memoria y decisiones del proyecto, contrato local documental y los archivos propios `credential-term-correction-v1/author/candidate-input.json` y `author/rationale.md`. No se abrieron preguntas, contrapartes, gold, guía V6 ni dictámenes de otros críticos. La crítica parte del texto fuente y de una propuesta general: enlace dirigido `nominal_possessive_determiner`, desde el núcleo hacia un uso léxico de determinante, con slot contextual intrínseco `source_indeterminate`.

La fuente propia dice: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.» La candidata reconoce que `owned_by` es una hipótesis no aceptada y que la identidad del referente de «su» no queda demostrada. Este informe no convierte esa hipótesis en error de ejecución observado: no se ha ejecutado el motor.

## Dictamen previo

La dirección propuesta puede conservar la fuente sin forzar una referencia personal y sin declarar parcial toda representación de una fuente indeterminada. Es condición necesaria que el carrier represente una operación nominal posesiva con dependencia anafórica/deíctica abierta, y que esa operación permanezca en todos sus consumidores. Un token lexical citado más una bandera `source_indeterminate` no basta. Tampoco basta sustituir `owned_by` por una asociación incidental.

La construcción puede quedar representada por completo mientras su slot contextual permanece indeterminado. La completitud de representación significa que la fuente y sus restricciones fueron preservadas; no significa que la fuente determine el referente. Esta distinción debe ser expresa tanto en el esquema como en el informe del validador y en la elegibilidad para responder.

## Semántica mínima que debe cerrar el diseño

1. El objetivo léxico es una **ocurrencia** de determinante posesivo, no el poseedor físico ni un nombre global de la palabra «su». Su mención tiene intervalo exacto en la fuente y su alcance nominal se vincula al núcleo concreto.
2. Cada ocurrencia introduce un slot contextual local. `source_indeterminate` declara que la fuente no fija su valor; no declara que la construcción carezca de relación posesiva, que el extractor omitiera un antecedente conocido o que todas las personas sean antecedentes admisibles demostrados.
3. El enlace restringe la interpretación del término nominal: el término con posesivo no se sustituye por el núcleo desnudo. Debe quedar disponible una lectura inversa del carrier equivalente a «su [núcleo y complementos]», conservando la dependencia contextual. Esta lectura inversa sirve para auditar fuerza, no es la representación de ejecución mediante texto libre.
4. La relación nominal posesiva es primitiva en este alcance. No se define como propiedad legal, tenencia física, emisión, uso, pertenencia institucional ni beneficiario. Tampoco es `associated_with`: este último puede ser verdadero entre dos entidades sin que una construcción posesiva haya sido afirmada.
5. La anáfora/deixis se conserva en ese slot intrínseco aunque no se reutilice el campo histórico `anaphora`. Un `anaphora:null` en el carrier solo puede significar que ese DTO histórico no corresponde a este uso léxico; nunca ausencia de la dependencia semántica.
6. Slots de ocurrencias distintas no prueban poseedores distintos. Slots con igual forma «su» no prueban poseedores iguales. El esquema debe evitar tanto fusión por token como desigualdad por ID. Su estado entre ocurrencias es desconocido salvo evidencia adicional expresamente soportada.
7. La forma «su» no permite inferir género o número del poseedor; tampoco permite elegir entre referente en tercera persona y tratamiento formal por mera morfología. Si el alcance inicial conserva solo la forma y el slot sin desambiguación, no debe anunciar rasgos adicionales resueltos.
8. «De alumna» y el posesivo son restricciones coexistentes sobre el mismo término. No se puede conservar una y borrar la otra, ni convertir «alumna» en persona identificada con Ina. La denotación léxica del complemento no se vuelve física por la llegada de este nuevo enlace.

Si una versión inicial excluye coordinación, elipsis, antecedentes explícitamente resueltos, cuantificación o posesivos con otra forma, debe declararlo. Una fuente fuera de ese alcance permanece no soportada; el feature no redefine esas construcciones para obtener completitud.

## Prueba formal de fuerza

Sea `h` el término nominal íntegro, `o` la ocurrencia posesiva y `κ(o)` el referente contextual todavía no determinado por el documento. Escribimos `PossNom(h, κ(o))` para la restricción nominal posesiva contextual; no es `OwnedBy(h, κ(o))`. La representación debe conservar `PossNom` dentro del argumento de la proposición de suficiencia, junto con `nominal_complement_de`, beneficiario, fundamento normativo, estado y scope fuente.

`κ` es un parámetro de interpretación contextual, no un individuo nuevo cuya existencia física afirme el carrier, ni una lista cerrada de candidatos. La representación tampoco introduce una universal «para cualquier poseedor». El cuantificador de la siguiente prueba está en el metalenguaje de auditoría.

Para cada interpretación contextual `κ` compatible con lo expresado en la fuente, y cada modelo `M` del vocabulario relevante:

- **No adición:** si `M` satisface la fuente interpretada bajo `κ`, debe satisfacer el contenido tipado proyectado bajo ese mismo `κ`.
- **No pérdida:** al volver a interpretar el contenido tipado en el fragmento expresivo declarado, debe conservarse la restricción posesiva y su alcance bajo ese mismo `κ`. No basta que ambas formas impliquen el núcleo desnudo.
- **Dependencia:** cambiar `κ(o)` debe cambiar, cuando corresponda, la selección contextual del término. Una implementación cuyo resultado es invariante porque borró `o` falla esta condición.
- **No cierre artificial:** ninguna igualdad, desigualdad, identidad con Ina o conjunto exhaustivo de candidatos puede aparecer salvo que esté respaldado por la fuente.

En forma abreviada se exige equivalencia en el fragmento relevante `Sκ ↔ Rκ`, con traducción explícita del operador nominal y sin exigir equivalencia sobre detalles ajenos al alcance. La implicación unilateral `Sκ → Rκ` solo prueba ausencia de sobreafirmación; permite pérdida arbitraria, incluida la representación vacía.

No se admite reemplazar esta prueba por `∃κ Rκ`: cerrar existencialmente el slot borra la dependencia de contexto y permite confundir «su credencial» con «alguna credencial de alguien». Tampoco se admite cerrar universalmente el slot dentro del contenido, porque convertiría la suficiencia contextual en suficiencia para todos los poseedores. Mantener el slot abierto con procedencia preserva la indeterminación sin enumerar lecturas cerradas.

Esta es una obligación formal de diseño y controles, no una prueba ejecutada de fidelidad de un extractor.

## Contraejemplos que distinguen pérdida y sobreafirmación

| Control manual nuevo | Falla que debe detectar |
| --- | --- |
| «Basta su credencial» frente a «basta una credencial» | Borrar el posesivo amplía el término admisible; igualdad de núcleo no es equivalencia de restricción. |
| «Su médico» | Una relación personal posesiva no acredita propiedad sobre una persona. |
| «Su foto» | La forma aislada no fija autoría, persona retratada, propiedad ni tenencia. No enumerar esas posibilidades como lista exhaustiva. |
| «Ina vio a Lea; su credencial basta» | Proximidad y sujeto previo no acreditan el antecedente. No crear Same con Ina ni Lea. |
| «Su credencial basta; su autorización caducó» | Dos ocurrencias pueden compartir referente o no. Ni fusión por forma ni Different por ID. |
| Misma forma «su» en dos documentos | La igualdad de token no establece correferencia entre documentos, namespaces ni scopes. |
| Mismo carrier reutilizado para dos núcleos no coordinados | Debe rechazarse la reutilización estructural que pierde el alcance de ocurrencia. No probarlo con un documento inválido por otra causa. |
| «Su credencial de alumna» | Deben sobrevivir juntos posesivo, núcleo y complemento nominal sin identificar la alumna con una persona. |
| «Su destrucción» fuera del alcance inicial | No imponer propiedad física al nombre eventual ni aprobarlo automáticamente por contener «su». |
| «La credencial de Ina» frente a «su credencial» sin antecedente | Relación explícitamente anclada y slot abierto no son el mismo objeto semántico. |
| «Su propia credencial» fuera del alcance inicial | No borrar un refuerzo restrictivo no representado para obtener complete. |

Los controles son de diseño redactados por este crítico. No son textos del banco ni contrapartes observadas, ni acreditan ejecución o extracción autónoma.

## Consumo, deudas y no interferencia

El claim consumidor debe conservar su argumento íntegro, el enlace, slot y ocurrencia, complemento nominal, qualifiers, fundamento y ámbitos. La propagación cubre dependencias explícitas e implícitas y todos los aliases del núcleo; no se descarga el slot a través de metadata ni de una ruta de conclusión existencial. Debe ser imposible obtener una respuesta completa por olvidar el carrier antes del matching ordinario.

Un parcial útil puede transportar esa construcción íntegra en `source_constraint`, ligado al claim y ámbito fuente concretos, conservando la petición y deudas. En ese canal, términos/proposiciones de respuesta y bindings permanecen vacíos. La identidad contextual pendiente no debe contaminar un claim independiente que no consuma el término. El informe debe distinguir `source_indeterminate` de `unresolved_target` por pérdida o fallo de extracción; una ausencia no explicada de antecedente no se beneficia automáticamente del nuevo estado.

La canonicalización puede renombrar IDs de ocurrencia y conservar una firma equivalente local, pero no convertir la firma léxica en `Same` entre poseedores. Debe conservar multiplicidad/incidencias y ámbito. La existencia de la variable contextual tampoco autoriza proyección de un individuo físico.

## Puertas y estado

No se concede pase de diseño final hasta leer un esquema cerrado y verificar que satisface estas obligaciones. Para una implementación posterior harán falta controles con documentos válidos completos, controles de rechazo estructural separados, fuentes inherentemente indeterminadas con representación completa, rechazo de respuestas por borrado del posesivo, no interferencia de hechos independientes, aliases, scopes, presupuesto y compatibilidad histórica sin feature.

La puerta sigue siendo 116/116 entradas fuente/pregunta fieles con aceptación de ambos críticos en las dimensiones exigidas, conservando omisiones, desacuerdos, históricos y denominador. Completar este mecanismo o corregir una representación propia no equivale a superar esa puerta ni permite utilidad automática o adopción. Este informe no modifica el contrato operativo ni autoriza implementación por sí solo.

## Segunda revisión: esquema disponible y suficiencia nominal

Se leyó después el diseño `documentary-v3-possessive-design-2026-10-22.md`, SHA-256 `d60c06bddcf3c20eb6ae96074615ee6d9e75fe4a090b19d584e2e12fcd55a4f1`. Ese contenido cierra las objeciones semánticas del carrier para el alcance que declara. Se acepta el diseño del operador, sin afirmar implementación ni revisión de una extracción nueva.

`Reference.denotation:lexical_use` y el enlace optativo `nominal_possessive` hacen explícita la distinción entre uso del determinante y referente contextual. El target conserva ocurrencia y scope, sin campos que permitan presentarlo como persona. El slot intrínseco preserva anáfora/deixis y restricción nominal; `anaphora:null` no se interpreta como ausencia de esa dependencia. El diseño excluye usos físicos, identidades y respuestas con U, conserva restricciones por aliases, corta consumidores antes de todas las rutas ordinarias y no permite convertir el slot en testigo existencial. Se reconoce expresamente la incompatibilidad con lectores anteriores de enums cerrados: un feature no vuelve comprensible una variante nueva para esos lectores.

El testigo parcial propuesto conserva fuente, petición, propietarios, scopes, dependencias, evidencia y ambos operadores nominales. `possessive_referent_proven:false` es resultado derivado del matcher, no permiso de extracción. La completitud representacional puede acreditarse sin resolver el referente. Esto no habilita respuestas completas por el término desnudo ni perjudica claims independientes. Cobro acumulado, agotamiento global y ausencia de recorrido sin feature están incluidos en el diseño.

La comparación de dos modelos con distinto referente es un control necesario de no fijación, pero no demuestra por sí sola conservación. Debe leerse junto a la prueba `Sκ ↔ Rκ` anterior. Por composición, la proyección relevante mantiene la misma estructura de suficiencia normativa sobre `PossNom(H,κ(U))`, con el complemento «de», beneficiario y scope intactos. El cambio de carrier solo sustituye la notación de esa construcción, sin introducir `OwnedBy`, valor fijo para `κ`, existencia física ni cierre cuantificado. El caso sin el operador no pasa la dirección de no pérdida; el caso con propietario impuesto no pasa no adición. Esta comprobación es lógica bajo la semántica declarada, no un resultado ejecutado.

El coordinador propuso además, de forma explícita y separada del texto del diseño leído, una fila de vocabulario `condition_suffices(beneficiary:ref,condition:term)`, `distributive:false`, para declarar una condición nominal concreta suficiente para ese beneficiario en el scope normativo fuente. Se acepta esa semántica como alternativa general a sobrecargar `class_suffices`. Debe incorporarse al diseño antes de implementar y preservarse como predicado distinto: sin equivalencia automática, backmapping, distribución sobre clases ni transformación de un término nominal en regla universal. La fila requiere rol `condition` íntegro; no autoriza desenvolver el posesivo ni el complemento «de». Ni la fila ni su estructura afirman existencia actual del objeto, ejecución, permiso, necesidad o quién es el referente contextual. La modalidad normativa, estado y scope pertenecen a la proposición y deben conservarse, no inferirse desde el nombre del predicado.

Declarar una fila genérica no demuestra que la fuente la sostenga ni que una pregunta concreta corresponda a ese predicado. Esta crítica no inspeccionó preguntas y no promete utilidad de un par. El matcher genérico puede comprobar identidad de predicado y roles sin ramas específicas; esa posibilidad estructural no permite igualar `condition_suffices` con `class_suffices` ni garantiza utilidad. La extracción nueva deberá citar y justificar también el beneficiario y la suficiencia, además del carrier posesivo.

No quedan objeciones semánticas bloqueantes al operador y a esa fila con estas definiciones. Antes de código siguen debiendo quedar concretos los diagnósticos cerrados y el shape de salida cuando el DTO actual no pueda conservar el testigo completo; el propio diseño exige ampliar ese DTO antes de implementar en ese caso. Este pase de diseño delimitado mantiene intactas la puerta 116/116, doble crítica de entregas, procedencia y verificación posterior.

## Revisión del addendum normativo

El addendum leído incorpora la fila exacta `condition_suffices`, su separación histórica y los diagnósticos cerrados. Esa precondición queda satisfecha. Se mantiene la aprobación semántica del operador.

La primera enumeración exacta de campos de transporte todavía omite nodos necesarios: `source_constraint` copia un Claim y `request_constraint` una Request, mientras que los arrays completos de dependencias pueden referir otros claims y requests. `premise_ids` requiere claims de pregunta que esa enumeración no transporta. Además, `identity[]` conserva los aliases Same/Different y no está contenido en `references[]`; también debe preservarse el namespace documental. Por tanto esa enumeración no satisface aún la promesa de resolución íntegra de IDs y restricciones. Se comunicó el hallazgo antes de código. La forma preferida para cerrar esta única objeción de transporte es incluir ambos documentos tipados íntegros, manteniendo los owners source/request explícitos, con cobro previo de copia; una alternativa por arrays debe demostrar inventario realmente completo. No es una objeción a la semántica posesiva ni permite relajar la puerta de calidad.

## Cierre de revisión del diseño

Se verificó el addendum corregido, SHA-256 del diseño `9936050f3e3757c44fd0238ac1f8c5bed338e9539815f9d93f34e5d03aa1e091`. La inclusión de `source_document` y `question_document`, ambos `DocumentaryV3` tipados íntegros y validados antes de las vistas privadas, cierra la objeción de transporte: permanecen claims, requests, identity, namespace y demás nodos/restricciones. Se exige coherencia exacta de constraints y arrays redundantes, con cobro de todas las copias antes de materializarlas.

Quedan satisfechas las dos precondiciones señaladas: fila normativa de suficiencia distinta y salida íntegra con diagnósticos cerrados. El dictamen final de este crítico es **diseño semántico aceptado dentro de su alcance declarado, sin objeciones de diseño bloqueantes pendientes**. Esto permite implementar el diseño revisado; no acredita que la implementación lo cumpla, que una entrega fuente sea fiel ni que la puerta 116/116 esté superada. No se leyó ningún resultado fuente nuevo ni dictamen de otro crítico, y no se ejecutaron Rust, Cargo, APIs ni campañas durante este cierre.
