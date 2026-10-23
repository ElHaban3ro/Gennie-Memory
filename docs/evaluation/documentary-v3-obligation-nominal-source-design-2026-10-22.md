# Eliminación de un nominal «obligación de tutor»: diseño fuente-only

Fecha: 2026-10-22. Estado: diseño semántico acotado, sin candidata completa, implementación, ejecución o aceptación de fidelidad. Fuente única: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.» Este documento diseña únicamente la primera cláusula; no cambia el análisis de la suficiencia ni resuelve el posesivo de la segunda.

## Dictamen y frontera

La solución semántica mínima conserva un solo host compuesto: la eliminación ocurrida del nominal local «obligación de tutor», calificada por «En Faro» y «para Ina». `nominal_complement_de` permite descomponer ese nominal sin elegir qué relación normativa expresa el genitivo. Los contextos léxicos conservan ambas restricciones sobre ese mismo host sin convertirlas en papeles más estrechos. No hacen falta dos lecturas enumeradas ni una entidad que desempeñe el papel de tutor.

Esto no cabe sin cambios en la admisión cerrada de `lexical_clause_context`: el contrato consultado admite un único contexto por owner. La ampliación mínima propuesta conserva su shape y permite dos incidencias distintas sobre el mismo Claim; exige revisión conjunta de su correlación. Además, debe acreditarse que el carrier de eliminación existente admite como argumento el nominal íntegro, en lugar de imponer una clase o un requisito de acompañamiento. Sin esa acreditación no hay candidata completa autorizable: es un límite expresivo pendiente, no un positivo manual.

Se leyeron solamente los diseños de referencia a contexto fuente, complemento nominal «de», contexto léxico de cláusula y posesivo de esta fecha, y `tools/evaluation/src/documentary_v3.rs` cortado antes del primer `#[cfg(test)]`. Los antecedentes mencionados dentro de esos documentos no se abrieron ni se usan como evidencia de esta fuente. No se leyeron matching, tests, preguntas, gold, reportes, otras revisiones o memoria. El extracto Rust expone la entrada `validate` y sus módulos, no las firmas de átomos/eventos ni las comprobaciones internas: no permite certificar soporte actual de esta combinación. No se ejecutaron Cargo, fmt, APIs o servicios.

## Nominal y evento conservados

N es la ocurrencia nominal cuyo núcleo explícito es «obligación»; T es el descriptor nominal local «tutor». N conserva su núcleo, mención y enlace dirigido `nominal_complement_de` hacia T con evidencia exacta «obligación de tutor». No se guarda todo el sintagma como una clase opaca ni se reemplaza su estructura por una cadena. La clase declarada acredita el núcleo léxico de esta ocurrencia; no cuantifica sobre todas las obligaciones.

T sigue las restricciones del descriptor nominal del diseño «de»: núcleo explícito, uso local ligado a esta incidencia, sin nombre, anáfora, binder, links o identidad de persona. La denotación `entity` de ese subset no acredita un tutor individual existente. No se crea un agente, poseedor o acompañante, ni se añade `owned_by`, una relación de asistencia o un rol normativo para T. La relación nominal es positiva y dirigida; la falta de especialización de su genitivo no significa que cualquier relación propuesta después quede demostrada.

El host E afirma la eliminación ocurrida de N. En notación semántica explicativa, no wire nuevo:

```text
E = EliminationOccurred(target=N)
N = NominalDe(head_use=obligación, tail_descriptor=tutor)
LexicalClauseContext(E, En, Faro)
LexicalClauseContext(E, para, Ina)
```

`target` aquí nombra el papel semántico del argumento y no afirma que exista un campo V3 con ese nombre. El carrier existente solo sirve si conserva ese argumento con todas sus restricciones y expresa eliminación pasada sin fecha inventada. Si su contrato exige una clase, `requires_class`, acompañamiento o un agente identificado, no se reutiliza por semejanza verbal. Haría falta una relación tipada de eliminación de término nominal con esos mismos compromisos mínimos; su firma sería una ampliación separada a revisar antes de producir raw. Este diseño no añade un predicado al vocabulario ni afirma haber comprobado esa firma.

«Se eliminó» no suministra agente explícito. La eliminación pasada tampoco demuestra por sí sola la ausencia actual de un requisito, su no reintroducción, una negación de clase o un permiso. No se publica `not requires_class(Ina,tutor)` ni se deriva que Ina puede realizar una acción no expresada. La segunda cláusula no se convierte en condición retrospectiva de E.

## «Para Ina» y «En Faro» sin adjunción inventada

Ambos contextos pertenecen al Claim completo de E, que incluye el evento y su argumento nominal N con T. «Para Ina» no se omite ni se conserva como texto sin vínculo. Es una restricción léxica positiva con complemento Ina en ese mismo compuesto. No se tipa automáticamente como beneficiaria de un requisito, sujeto obligado, tutora, poseedora del nominal o destinataria de un permiso. «En Faro» conserva la restricción léxica correspondiente, sin resolver Faro como institución, edificio o régimen normativo.

La modalidad propuesta para cada contexto es `whole_clause_only`: se afirma su adscripción al compuesto completo y se rehúsa publicar una adscripción interior más estrecha. No se usa `source_underspecified` para elevar la duda del diseñador a una afirmación de ambigüedad de la fuente. Que el sintagma aparezca junto al nominal no acredita, por mera contención, un enlace nominal a Ina; tampoco basta para convertirlo en un argumento obligatorio del evento.

En este contrato, el host es el Claim compuesto, no el VP «se eliminó» tomado aisladamente. Por ello `whole_clause_only` no adjudica «para Ina» al verbo. Tampoco es una referencia opaca a toda la expresión fuente en c0 que garantice, por guardar la fuente, cualquier adjunción original. Afirma una restricción contextual positiva exterior y deja de publicar una distribución interna: esa abstracción necesita aceptación semántica propia. Si la fuente exige que «para Ina» restrinja precisamente N, el contexto exterior solo conserva una versión más débil y no acredita completitud de esa construcción.

`source_underspecified` no permite eludir este problema: además de requerir evidencia de indeterminación fuente, el carrier existente sigue siendo propietario de Claim. No contiene una región interna tipada N/E ni un selector de subhosts. Si conservar la indeterminación interna precisa exigiera tal región, habría una ampliación real del DTO/semántica y su revisión; no se disimula como un nuevo valor de flag ni como dos lecturas cerradas. Ninguna de estas ampliaciones queda implementada o autorizada para implementación por este diseño.

Esta granularidad está ya definida por el diseño de contexto léxico y tiene un coste explícito: conserva una relación contextual de la cláusula y no autoriza conclusiones que necesiten una adjunción interior determinada. La revisión fuente debe confirmar que eso preserva el contenido de «para Ina» a esta granularidad. Si exige una restricción interna específica de N, esta solución queda semánticamente pendiente; no se declara completa por llevar ambas citas. En tal caso la ampliación mínima sería un contexto léxico propietario de N, con cierre y correlación N→T→E preservados, en vez de reasignar arbitrariamente el contexto existente. Ese carrier nominal no está definido ni autorizado por los documentos leídos y no se introduce aquí como si ya existiera.

Dos contextos en E no se interpretan como hechos independientes transferibles. No se distribuye «para Ina» sobre todas las obligaciones ni «En Faro» sobre otros claims. La salida que dependa de E conserva simultáneamente evento, N, T, ambas incidencias y sus complementos. Eliminar una incidencia o moverla al otro Claim cambia la representación. Conservarlas en claims separados sin un host correlacionado tampoco basta.

## Ampliación mínima y límites de suficiencia

El cambio mínimo del contrato léxico es de admisión y consumo conjunto: máximo dos contextos por owner en este subset, IDs de incidencia distintos dentro de E y comprobación independiente de evidencia/complemento más aceptación conjunta del mismo host. No cambia el wire de `LexicalClauseContext`, no añade un wrapper y no afloja los demás límites de scope, forma, presupuesto, dependencias o procedencia. Si el evento se codifica en un Body que no sea la proposition admitida actualmente, la admisión sigue bloqueada hasta una ampliación explícita de esa variante; no basta cambiarle la etiqueta a proposition.

Los dos contextos comparten el Claim, pero conservan lexema y evidencia propios. La mayúscula de «En» se mantiene bajo el contrato que no normaliza lexemas. La frontera externa contiene la primera cláusula y no incorpora la suficiencia posterior al punto y coma. Las evidencias del nominal, el evento y cada sintagma deben poder reconstruirse contra la fuente exacta; este diseño no produce offsets o un raw para aparentar validación.

La admisión doble requeriría adaptar la revisión ligada a incidencias para que ambas sean revisadas y ninguna desaparezca de su universo efectivo. La aceptación aislada de una no acepta el conjunto. Cierre, selección y transporte deben recorrer ambos contextos y los enlaces de N; una ruta que solo admita uno queda sin soporte para E íntegro. Se reutiliza el presupuesto acumulado vigente y se cobran las dos incidencias y sus copias; no se propone otro sistema de recursos ni una ampliación de cotas.

El contrato de referencia a contexto fuente no subsana estas brechas: su primera forma selecciona un testigo posesivo y vuelve a comprobarlo. No es un carrier genérico que vuelva fiel un host de eliminación incompleto. Su disciplina de conservar ocurrencia y documento puede orientar un transporte posterior, pero el transporte no decide la adjunción ni la semántica del evento.

Por tanto, `nominal_complement_de` más eliminación alcanzan el núcleo de la primera cláusula solo si la eliminación consume el término nominal restrictivo completo. Para conservar además Faro e Ina mediante el carrier léxico diseñado hacen falta la admisión doble y la revisión conjunta indicadas. La fidelidad de la adscripción externa de «para Ina» y la firma real de eliminación permanecen como puertas concretas; si alguna falla, existe una brecha expresiva visible. No se borra Ina, no se fuerza acompañamiento y no se declara resuelta por enumerar interpretaciones.

No se comunica ningún resultado de rendimiento, extracción, matching o calidad. Una candidata futura tendría que conservar la segunda cláusula y sus deudas por separado antes de evaluar la completitud de la fuente entera.
