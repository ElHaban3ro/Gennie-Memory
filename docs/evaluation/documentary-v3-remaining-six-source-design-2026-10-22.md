# Diseño fuente→raw de las seis entradas pendientes

Fecha: 2026-10-22. Alcance: lectura y diseño documental; no se ejecutó Cargo, proveedor, extracción ni comparación de pares. Este documento no modifica raws, revisiones, puntuaciones ni la puerta: siguen siendo necesarias las 116 representaciones completas y la aceptación de fidelidad por ambos críticos antes de utilidad automática. No acredita un cierre del 100 %.

Se leyeron exclusivamente texto, raw y vocabulario propios de estas seis entradas en `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-author-corrections-v1/known116-bundle/`. No se consultaron sus contrapartes, gold, guías V6, informes de calidad ni otras revisiones. Se contrastaron `AGENTS.md`, la regla de pruebas, el contrato local y declaraciones operativas del código. Las propuestas de lowering de abajo son diseños, no documentos validados.

## Criterio de conservación

El compromiso expresado y la precisión del texto son cosas diferentes. Una mención contextual puede estar íntegramente conservada aunque el texto no decida su tipo; una negación temporal no autoriza una negación global; una finalidad no acredita que su acción haya ocurrido. `unsupported` por carencia del lenguaje y una referencia o interpretación indeterminada en la fuente deben diagnosticarse por separado.

Eso no permite cambiar retrospectivamente un `partial` por `complete`. El adaptador de lecturas existente marca parcial toda envoltura abierta o con dimensiones pendientes; el contexto normativo distingue `unspecified` de `ambiguous_control` y `extraction_unresolved`. Por tanto, la fidelidad de una ausencia explícitamente transportada no garantiza hoy completitud estructural. Si esa distinción requiere una representación nueva, hay que incorporarla y verificarla prospectivamente; nunca quitar el diagnóstico para pasar la puerta.

Las posibilidades operativas existentes son `documentary_evidence_v3`, referencias con enlaces restrictivos, `configuration`, términos `described_ref`, los contextos normativos optativos, `relative_nonoccurrence_bound` y `documentary_reading_set_v1`. Las alternativas documentales o interpretativas no prueban por sí mismas que una lista de interpretaciones sea exhaustiva. No se propone otro sobre ni una lista cerrada fabricada.

## 1. QFaro — `d-1dc7dd73ded28e0da27a8682`

Texto: «¿Qué tutor debe acompañar a Ina para matricularse en el curso Faro?».

Hay que conservar conjuntamente: solicitud de un tutor, acompañamiento de Ina, modalidad de obligación, curso Faro y relación con matricularse. No se afirma matrícula ni acompañamiento efectivos. El raw conserva el primer segmento pero marca `para matricularse en el curso Faro` como `unsupported`; `normative_scope_filter:course` no sustituye la acción subordinada ni su relación.

El carrier existente `normative_action_relation` puede alojar `action_kind:enroll`, destino Faro, relación citada y participante subordinado. Es preferible a una premisa de matrícula ocurrida, a un requisito textual sin estructura o a un nuevo envelope. La acción principal conserva su actor abierto y filtro tutor; Ina permanece como objeto del acompañamiento.

Quedan dos decisiones que el extractor no debe resolver por conveniencia: valor de la relación introducida por «para» y control del sujeto de «matricularse». Si la lectura adoptada es finalidad, se registra `purpose`, sin convertirla en suficiencia o ejecución. Si fuente y análisis no acreditan un actor único, no se asigna por intuición al tutor ni a Ina. `unresolved:unspecified` solo sirve para ausencia realmente no especificada; no puede reemplazar `ambiguous_control` para evitar el diagnóstico parcial. La forma existente de relación `unresolved` enumera `purpose/necessary_condition` y queda parcial: no demuestra que esas dos lecturas agoten el texto.

Preservación útil: el consumidor recibe pregunta completa, acción principal, acción subordinada y deudas de relación/control; no una pregunta reducida a «quién acompaña». Hasta resolver esas deudas con una representación que las conserve, no se publica tutor ni se acredita cumplimiento de matrícula. Si se necesita indeterminación completa sin inventariar lecturas, la brecha es local al vínculo subordinado; no justifica rehacer el documento entero.

## 2. Lino — `d-eeb265867d41ca3c7ebc7f3b`

Texto: «No se midió la duración del reproductor Lino hasta apagarse.».

Hay que conservar negación, medición, duración, sujeto Lino y vínculo de «hasta apagarse». No se afirma duración numérica, autonomía máxima, apagado ocurrido ni medición posterior. El raw actual conserva el segmento como `terminal_condition` sobre un evento negado; su cita no decide por sí sola si modifica el aspecto medido o el intervalo de no realización.

Para el límite temporal de una no realización ya existe `relative_nonoccurrence_bound`, con sujeto Lino, acción límite `switch_off` y aspecto de medición `duration`. Su adjunción es al evento negado, con dependencias explícitas; no debe conservarse simultáneamente una configuración terminal competidora. Este carrier impide que la negación acabe como «nunca hubo medición» y no afirma el límite como acontecimiento.

Para una lectura de duración hasta apagado, la representación del aspecto de medición y sus condiciones conserva un compromiso diferente. La disponibilidad de ambos mecanismos no autoriza declarar dos lecturas exhaustivas ni convertir una lectura elegida en hecho incondicional. Si sigue indeterminada la adjunción, una envoltura abierta existente puede mostrar la deuda sin inventar cierre; seguirá parcial bajo el contrato actual. No es evidencia de fallo del extractor por haber reconocido fielmente una indeterminación del texto.

Preservación útil: transportar evento, límite/aspecto, citas y deuda exactos; términos de respuesta vacíos. Es una base de contexto, no una cantidad ni una conclusión de ignorancia global. La primera intervención sobre esta entrada debe comprobar qué compromiso expresa cada carrier, antes de redactar otra extracción.

## 3. Credencial — `d-5b4860f3cf848e9dcbf9985a`

Texto: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.».

Hay que conservar eliminación ocurrida, obligación eliminada ligada a Ina, clase tutor, ámbito Faro, suficiencia normativa y el referente de «su credencial de alumna». No se infiere que Ina tenga físicamente la credencial, que la haya presentado, que esté matriculada ni que toda persona poseedora de cualquier credencial esté exenta.

El raw ya separa eliminación, ausencia de requisito y suficiencia, pero en `class_suffices.class` almacena toda la frase «su credencial de alumna» como texto y marca esa frase `unsupported`. Conservarla en una cadena citada no vuelve operativos el posesivo y la calificación local.

La primera opción es reutilizar un término referencial permitido por el rol `term`: referencia local de credencial, vínculo restrictivo a Ina respaldado por «su» y complemento nominal «de alumna». El código actual dispone de `nominal_complement_de`, optativo, cuyo descriptor es local y no afirma por sí mismo existencia ni pertenencia de Ina a una clase. Una asociación genérica no debe presentarse como propiedad jurídica o sustituir sin más el posesivo. Debe comprobarse que el enlace seleccionado conserva exactamente su fuerza contextual.

Si la representación necesita una restricción interna, `described_ref` está permitido en roles `term` por el perfil ya presente; sus restricciones pertenecen a esa ocurrencia y no se elevan a hechos fuente. No se inventará una propiedad para tener una restricción obligatoria: el término referencial con sus enlaces puede bastar. Si `class_suffices` solo admite semánticamente clases y pierde la suficiencia de esa credencial vinculada a Ina, la brecha sería un portador acotado de suficiencia de término, preservando sujeto y ámbito; no una excusa para tratar el documento específico como clase universal. Esta alternativa requiere justificar primero la incompatibilidad real del portador existente.

Preservación útil: mantener la dispensa y la alternativa suficiente como dos compromisos normativos distintos; no publicar un tutor ni promover suficiencia a permiso universal. No añadir como finalidad una matrícula que esta fuente aislada no menciona.

## 4. Cobre, funcionamiento — `d-38fb24e622a073a293ead090`

Texto: «Cobre funcionó ochenta minutos en Nacar sin apagarse; no se midió su duración hasta apagarse.».

Hay dos cláusulas. La primera afirma funcionamiento ocurrido de Cobre durante 80 minutos, en el contexto literal «en Nacar», sin apagarse durante ese funcionamiento. La segunda expresa una no medición calificada por «hasta apagarse», con la cautela de adjunción descrita en Lino. No se obtiene un máximo ni una duración total de 80 minutos. Tampoco se afirma ausencia de medición de cualquier atributo y en cualquier momento.

El raw ya tiene dos claims y duración exacta. No debe especializar `Nacar` como ruta, modo, lugar o programa sin evidencia. La referencia nombrada y la relación contextual se conocen; lo no determinado es su interpretación y, cuando corresponda, su adjunción. Un `configuration` neutral con clave `context` y valor referencial Nacar puede ser un candidato existente si su contrato expresa solo ese contexto literal, con cita de «en Nacar» y dependencia del propietario. Marcar `resolution:unknown` confundiría desconocimiento del dato Nacar con interpretación no especificada y bloquearía al consumidor.

Esta propuesta necesita verificar que el comparador conserva la relación literal y no equipara automáticamente contextos de distinto sentido por nombre. Tampoco debe extender a la segunda cláusula el contexto de la primera sin evidencia. Si la semántica real de `configuration` presupone un tipo o una adjunción más precisa, no sirve: sería necesario un carrier local para relación contextual literal, con referente, propietario y precisión no especificada, sin lista de sentidos cerrada.

Preservación útil: funcionamiento observado y no medición permanecen separados; cada testigo conserva sus propios qualifiers. La falta de interpretación de Nacar no puede borrar 80 minutos, pero tampoco permite usar ese valor en un contexto pedido cuya correspondencia no esté acreditada.

## 5. Batería reservada — `d-b7123caa690427c7c04d4c74`

Texto: «Se reservó una batería nueva para Cobre en Nacar.».

Hay que conservar reserva ocurrida, paciente batería, calificación «nueva», beneficiario Cobre y relación contextual «en Nacar». Agente ausente no significa agente desconocido nombrado. Reserva no afirma instalación, uso, entrega, capacidad ni aumento de autonomía.

`action_beneficiary` conserva reserva y beneficiario sin inventar agente. Para «nueva», la primera opción es un sujeto `described_ref` que mantenga la propiedad local de esa batería; basta una extensión declarativa de vocabulario con atributo neutral de condición/novedad si no existe uno adecuado. Su significado debe seguir la calificación literal: no equivale automáticamente a nueva fabricación, compra reciente o carga completa. No se debe colgar el adjetivo del evento de reserva ni elevar una restricción local a afirmación sobre todas las baterías.

El contexto Nacar tiene la misma cautela tipológica que el caso anterior, pero debe resolverse desde esta oración. No se hereda que sea un modo, ruta o lugar porque otro documento tenga el mismo nombre. Si la adjunción a reserva, batería o beneficiario no queda acreditada, una configuración del evento no puede esconder esa diferencia. Primero se comprueba la suficiencia de contexto literal con alcance local; si no basta, la necesidad de carrier se limita al vínculo y su adjunción pendiente.

Preservación útil: reserva y descripción íntegra de batería, con beneficiario, contexto y deudas. Cualquier selección futura debe mantener que es una reserva; nunca responder autonomía o batería instalada a partir de esta evidencia.

## 6. Zinc — `d-5d8e6d45d1ee52311cdfb8b4`

Texto: «El reproductor Zinc se apagó tras cien minutos en Nacar.».

Hay que conservar apagado ocurrido de Zinc, sucesión expresada por «tras», cantidad 100 minutos y contexto Nacar. No se afirma medición por un método, máximo de autonomía, comparación con Cobre ni identidad de episodios entre textos.

El raw conserva acción sin agente y `duration_until_shutdown:100 minutos`, pero el diseño debe comprobar que ese atributo no introduce un inicio operativo ausente: «tras cien minutos» da un intervalo relativo cuyo punto inicial no aparece explícito. La expresión completa, incluido «tras», debe permanecer ligada al apagado; una cantidad suelta no sustituye la relación. Los portadores temporales literales y `configuration` existentes son candidatos antes de crear otro operador; no se promete que permitan publicar una duración numérica calculada.

Para Nacar se aplica el examen de contexto literal ya descrito, conservando el alcance de esta oración. No se declara `operates_mode(Zinc,Nacar)` ni ubicación por intuición. Si queda pendiente el inicio del intervalo o su interpretación, se registra esa precisión ausente sin calificarla como omisión de una información que la fuente no ofrece.

Preservación útil: apagado y relación temporal íntegros como evidencia de este episodio, sin afirmar máximo, medición o condición de otro episodio.

## Siguiente intervención concreta

Preparar, en un archivo derivado nuevo, una representación de la entrada de credencial usando referencia local, complemento nominal y vínculo posesivo debidamente justificado, manteniendo los tres compromisos originales. Es el ensayo más delimitado para comprobar si los carriers ya presentes resuelven la pérdida sin mecanismo nuevo. Primero revisar estáticamente fuerza del vínculo y dominio semántico de `class_suffices`; después, bajo el flujo general de trabajo, validar forma y pedir doble crítica fuente-only. Si el rol existente pierde la distinción entre credencial específica y clase, documentar esa objeción antes de ampliar el contrato.

En paralelo lógico, los tres Nacar necesitan una decisión común sobre preservación de contexto literal con tipo no especificado, pero cada adjunción se justifica por su propia fuente. No propongo añadir otro sobre: el trabajo consiste en probar la suficiencia de `configuration` neutral y, solo si falla por semántica operativa, diseñar un carrier local para ese vínculo. QFaro y Lino requieren conservar incertidumbre interpretativa sin adjudicar cierre a un inventario arbitrario.

Esta revisión no produce resultados de ejecución ni declara resuelta ninguna de las seis filas. Conserva todos los compromisos y evita resolver la puerta mediante exclusión, cambio de etiquetas, eliminación de calificadores o confusión entre indeterminación de fuente y fallo técnico de extracción.
