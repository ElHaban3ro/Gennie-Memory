# Auditoría local de minimalidad de portadores documentales

Seguimiento posterior: tres atributos ensayados con autoría y doble revisión en el [informe de resultados](documentary-v3-known-local-corrections-results-2026-10-22.md). La auditoría original siguiente permanece estática y no acredita por sí sola esos resultados.

Fecha: 2026-10-22. Estado: propuesta semántica estática, sin implementación, ejecución Cargo, API de modelo ni modificación de raws. Se revisan exclusivamente cinco fuentes del diagnóstico, sus raws/vocabularios y el contrato/código del evaluador. No se consultaron preguntas, contrapartes, gold, resultados de utilidad ni V6. Las correcciones propuestas no alteran los dictámenes históricos.

## Dictamen

No se demuestra necesidad de DTO nuevo para conservar estos cinco textos. `property(subject:ref,attribute:attribute_id,value:term)`, `described_ref`, referencias con asociaciones y `reading_set` ya suministran la forma necesaria. Sí falta autorizar sentidos precisos: agregar una fila de atributo no convierte automáticamente el uso en fiel. Sala y papel requieren atributos acotados; confirmación requiere estado de progreso; batería admite el atributo actual; Muelle admite una alternativa de atributo normativo contextual, pero exige una decisión semántica explícita que actualmente la guía no concede. Ninguna de estas propuestas prueba que un documento resultante sea complete.

La solución mínima mantiene `value` como texto escalar citado, nunca como cláusula arbitraria o JSON. Un atributo identifica una dimensión, su sujeto identifica el portador y la evidencia identifica la atribución. El validador comprueba pertenencia del ID y forma, no decide si el atributo significa fielmente la fuente. Los límites semánticos siguientes tendrían que quedar en guía y crítica; un guard ejecutable sería trabajo adicional, no una propiedad ya probada del código.

## 1. Sala baja — d-edc2cc8137740eb42150d567

Fuente: «Arce ensaya en la sala baja.» El raw conserva `action_actor(actor=Arce,action_kind=rehearse)` con `configuration(rehearsal_location,ref room)` y deuda sobre «baja».

**Mínimo defendible:** conservar esos roles y añadir una proposición `property(subject=ref room,attribute=nominal_qualitative_descriptor,value=text "baja")`, `fact/state/description`, sin ámbito normativo. La referencia tiene mención «la sala baja» y clase «sala»; el atributo se cita en «baja», la proposición en «la sala baja». Esta proposición significa únicamente que el nominal de ESE referente incorpora ese descriptor cualitativo explícito. No afirma nivel, planta, altura, posición vertical, orientación ni una relación comparativa.

**Definición acotada propuesta:** `nominal_qualitative_descriptor` conserva un modificador cualitativo lexical expresamente atribuido dentro de un nominal referencial. Valor: adjetivo o sintagma adjetival mínimo, íntegro y citado; no admite una frase proposicional, acción, requisito, relación, estado informativo ni cantidad. Se aplica únicamente cuando ninguna dimensión específica está acreditada inequívocamente. No produce aliases con atributos especializados ni inferencias a partir de su texto. Este atributo conserva la atribución lexical; no resuelve la interpretación física o identificadora de «baja».

La vía `described_ref(room,constraints=[property(...)])` sería local al operando, pero el portador actual es `Configuration.value`, que no admite `described_ref`: el recorrido del validador solo lo admite en Argument (y members habilitados). No se debe afirmar que el perfil de roles term corrige eso. La proposición separada es defendible aquí porque se atribuye una descripción al nominal referenciado inequívocamente, no una propiedad física general ni un hecho independiente inferido. Si una crítica exige que el descriptor solo exista dentro del parámetro de esta ocurrencia, queda una brecha real: ampliar específicamente `Configuration.value` bajo perfil nuevo, con propagación de deuda, o declarar un predicado relacional de ubicación cuyo rol permita descripción. Eso sí cuesta código/contrato y no es necesario para ensayar la alternativa lexical propuesta.

**Costo mínimo:** una fila nueva de atributo, definición y regla explícita de selección/atribución lexical, nueva versión/hash de vocabulario y documento derivado. Sin DTO ni rol nuevo. No usar `orientation`, `relative_level` o `vertical_position` sin decidir antes el sentido; no cambiar unsupported a represented hasta crítica de la nueva propuesta.

## 2. Confirmación pendiente — d-26600b62b77b674c11cdf612

Fuente: «Arce tiene diez músicos de cuerda confirmados; la confirmación de viento sigue pendiente.» La segunda proposición actual especializa indebidamente `documentary_state`; no existe documento afirmado. La primera cuenta de músicos de cuerda se conserva independiente.

**Carrier propuesto:** `property(subject=ref confirmation,attribute=action_progress_state,value=text "sigue pendiente")`, `fact/state/description`, ámbito normativo null. `confirmation` es el nominal explícito «la confirmación de viento», con `associated_with→wind` ya respaldado. No hace falta crear una ocurrencia confirm, actor, documento o conjunto cuantificado. La cita del valor debe abarcar «sigue pendiente», conservando la continuidad, y la proposición debe citar toda la cláusula. No normalizar a «pendiente» perdiendo «sigue».

**Definición propuesta:** `action_progress_state` expresa el estado de progreso explícito de una acción o proceso nominalizado, incluyendo el modificador aspectual contiguo que lo delimita. El valor no es una tarea o descripción arbitraria de la acción. Solo admite estados y aspecto literalmente expresos del mismo portador; no implica realización, programación, obligación, documentación ni desconocimiento de una cantidad. «Sigue pendiente» cabe como estado escalar compuesto sin codificar un calendario. Si se requiere razonar separadamente sobre continuidad temporal, ese razonamiento sigue pendiente; no se adquiere por guardar el literal.

**Guards y costo:** una fila nueva, definición cerrada, propietario nominal exacto y separación de `status:state` (forma de la proposición) del estado lexical del proceso. Mantener asociación a viento; no transferir el diez a viento, inferir cero confirmados ni `confirm planned`. `pending_action` está restringido a `quantity.pending_pages` y no sirve. Sin DTO, sin nueva enumeración de status. El complete estructural previo permanece histórico y no acredita fidelidad.

## 3. Batería nueva — d-b7123caa690427c7c04d4c74

Fuente: «Se reservó una batería nueva para Cobre en Nacar.»

**Carrier existente para nueva:** conservar `action_beneficiary(subject=described_ref battery,action_kind=reserve,beneficiary=ref cobre)` con `occurred/description/fact`; restricción local `property(subject=ref battery,attribute=validity_state,value=text "nueva")`, `fact/state/description`. El atributo actual define información explícita de validez/condición del portador correcto. La lectura de condición literal es defendible y no necesita fila nueva. Cita del descriptor «batería nueva» y del valor «nueva»; la restricción no se publica como hecho autónomo del documento.

No implica carga, funcionamiento, batería sin usar ni fecha de fabricación. Si una crítica entiende «nueva» como novedad respecto a otra batería y excluye esa acepción de condición, la diferencia no se resuelve afirmando el atributo correcto por definición: mantener lectura/deuda o precisar un atributo de novedad relativa, sin inferir cuál relación temporal significa.

**Lo que no cierra:** «en Nacar» no fija si Nacar modifica la reserva o al beneficiario, ni acredita ajuste o lugar por su capitalización. No poner `context=Nacar` ni un link located_at por conveniencia. `reading_set` puede conservar lecturas completas cuando haya portadores semánticamente autorizados; si faltan, mantener el conjunto abierto y esa cobertura unresolved. Corregir nueva no permite complete de toda la fuente.

**Costo mínimo:** reautoría del término subject y su restricción local, sin cambio de código/DTO/vocabulario. La ambigüedad de Nacar tiene costo de revisión de lecturas, separado del descriptor.

## 4. Bandeja de papel grande — d-f4152dc4613a39eccea79714

Fuente: «Senda tiene una bandeja de papel grande.» El raw solo conserva la lectura bandeja grande y la marca representada. La alternativa papel grande es concreta y debe permanecer visible.

**Lectura A:** conservar `has_item(bearer=Senda,item=described_ref tray)` con `property(tray,paper_tray_size,text "grande")` local. No transferir el atributo a papel.

**Lectura B:** conservar `has_item(Senda,described_ref tray)` cuya restricción incluye una relación nominal débil al referente paper, y una proposición que describa `property(paper,qualitative_size,text "grande")` sobre el papel de ese nominal. `Reference.links.associated_with` permite el genitivo sin inferir composición, contenido, admisión ni posesión del papel. Su dirección es tray→paper. El referente paper usa la mención y clase literal «papel», sin inventar hoja individual, cantidad o formato. La descripción de paper puede conservarse como proposición atributiva local al referente del nominal; si debe quedar encapsulada en el descriptor de tray, no anidar described_ref (prohibido): una restricción arraigada en tray con predicado/link existente más property independiente necesita revisión de fidelidad y deudas; no inventar una anidación que el validador rechaza.

**Atributo propuesto:** `qualitative_size` = tamaño cualitativo literal explícitamente atribuido al portador, sin dimensiones numéricas, formato, estándar ni unidad inferidos. Solo valores escalares de tamaño; no forma, capacidad, material o propiedad genérica. Conserva la comparación implícita sin fabricar patrón de comparación. Un atributo paper_size estrecho también cabe, pero agrega especialización innecesaria; `qualitative_size` evita trasladar paper_tray_size y tiene frontera revisable.

**Extras que impiden cerrar:** la ambigüedad del alcance de grande requiere `reading_set`, nunca `body.alternatives` (la fuente no afirma una disyunción). «De papel» no autoriza una relación fuerte; si las lecturas materiales/funcionales no quedan íntegramente representadas por asociación débil, conjunto open y deuda explícita. Las hojas deben preservar el mismo texto y el descriptor en su propietario respectivo. No declarar closed solo porque se enumeraron dos alcances de grande si quedan otras diferencias semánticas no cubiertas.

**Costo mínimo:** una fila de atributo, definición, reautoría de dos lecturas y crítica del enlace nominal. Cero DTO nuevos. El complete histórico no decide esta atribución.

## 5. Requisito Muelle — d-a54874431854cbe517a24364

Fuente: «El curso Muelle exige tutor para matricularse.» `body.rule(direction=necessary)` expresa necesidad, pero exige acción como Proposition completa. `action_actor` requiere actor; `action_undergoer` requiere afectado; `action_beneficiary` requiere sujeto/beneficiario. Ni el curso es un alumno ni el tutor es el matriculado. No crear ref student, actor omitido, regla con Atom incompleto o acción accompany.

**Opción mínima a criticar antes de DTO:** `property(subject=ref course,attribute=enrollment_required_class,value=text "tutor")`, `fact/state/normative`, `normative_scope=course`. El atributo significa exclusivamente «clase exigida como condición necesaria para matricularse en este curso». La evidencia del atributo/propiedad abarca «exige tutor para matricularse»; el valor cita «tutor»; el portador course conserva nombre Muelle y clase curso. El juicio fact afirma la existencia del requisito como propiedad normativa del curso, sin afirmar que el curso deba matricularse ni que tutor se haya cumplido. La necesidad está en la semántica del atributo, la acción en su dimensión explícita; no se pierde `para matricularse` ni se convierte en purpose genérico.

Esta es una dimensión específica como otros atributos declarados; su neutralidad consiste en no inventar participante, ocurrencia o suficiencia. No es un atributo genérico requirements con la frase completa dentro de value. Si el requisito exige adjuntos del tutor, condiciones o actor expreso que este escalar no liga, queda fuera y debe conservar deuda.

**Cambio semántico obligatorio:** la guía actual da prioridad a `body.rule` cuando hay acción explícita. Hay que admitir explícitamente la atribución de clase necesaria para matrícula al curso sin actor nominal, y delimitar que esta nueva propiedad describe el requisito del curso. No basta agregar un ID y adjudicar complete. La propuesta usa roles existentes, pero no está autorizada por la guía vigente.

**Por qué no usar requirement_class tal cual:** su subject es el beneficiario/sujeto del requisito; usar course lo convierte sin declaración en el sujeto de una exigencia que se aplica a quien se matricula. Agregar un qualifier acción no corrige ese cambio de rol. Otra opción sería ampliar expresamente requirement_class para un titular contextual y agregar key de acción prospectiva; también evita DTO, pero cambia más semántica y abre una forma genérica de acciones. La opción del atributo acotado tiene menor superficie para este texto.

**Límite:** esta propiedad no ofrece por sí sola matching de reglas, dispensa, compartición de participante entre condiciones y acto, ni inferencia sobre matrícula. Si se requiere esa operación futura, sí conviene estudiar un patrón normativo impersonal con participante ligado; el hecho de necesitar esa futura operación no vuelve obligatorio implementarlo para conservar esta fuente. El patrón normativo abierto existente tampoco rellena actor desconocido dentro de rule.action.

**Costo mínimo:** una fila estrecha, definición y excepción de precedencia en guía, reautoría y crítica. Sin DTO ni cambios de código para proponer la conservación escalar. Si la crítica rechaza esta atribución normativa al curso, conservar unsupported; no redefinir complete ni fabricar alumno.

## Evidencia y verificación pendiente

Base consultada: `consolidated-failure-diagnosis-v1/source-diagnosis.json` (solo esos cinco items), `source-only-package.json` (vocabulario, notation y guía; solo material de esas fuentes), y cinco `consolidated-author-corrections-v1/known116-bundle/<id>-input.json` (text/raw/vocabulary). Ambos directorios están bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`.

Evidencia estática: [contrato](documentary-v3-local-contract-2026-10-21.md), `tools/evaluation/src/documentary_v3/validation.rs` (forma de Configuration/Body, contexto permitido de described_ref, validación de roles y comprobación de rule), `tools/evaluation/src/documentary_v3/descriptions.rs` (arraigo, no recursión y límites de restricciones locales). La pertenencia de un atributo nuevo a fixed_attribute_rows habilita estructura, no su significado. Los controles futuros deben comprobar portador, continuidad, alcance, modalidad, asociación y conservación de ambigüedad, además de forma.

No se ejecutaron pruebas ni se generaron validaciones derivadas. Esta auditoría no reclama fidelidad automática, aceptación de críticos, utilidad, latencia ni despliegue. Las nuevas versiones de vocabulario/guía, documentos candidatos y críticas son trabajo posterior; no se sustituyen raws ni dictámenes congelados.
