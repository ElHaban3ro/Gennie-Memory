# Evento nominal con contextos ligados a la ocurrencia fuente

Fecha: 2026-10-22. Estado: especificación prospectiva fuente-only; requiere crítica antes de implementar. No sustituye [el diseño anterior](documentary-v3-obligation-nominal-source-design-2026-10-22.md), cuyos límites de adjunción permanecen. Fuente única: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.» El alcance presente es su primera cláusula.

## Dictamen semántico

Una referencia rígida al contexto original puede conservar la **dependencia** de la adjunción sin elegir su valor. Eso es distinto de representar una proposición autónoma con adjunción ya resuelta. La ampliación propuesta conserva una expresión estructurada y su interpretación dependiente de la ocurrencia fuente. No demuestra, por nombrar c0, cuál es esa interpretación; tampoco acredita fidelidad si el grafo omite o altera una construcción.

La región compuesta evento+paciente delimita el entorno estructural de cada PP; no afirma que el PP modifique exteriormente el evento. En particular, no aplica `whole_clause_only` a «para Ina». Su contribución se interpreta donde la interpretación original de esa construcción en esa ocurrencia la sitúa, conservando esa dependencia sin publicarla como un nodo resuelto. No hay un parámetro de destino ni elección del consumidor. La indeterminación del transporte no afirma que la fuente sea ambigua.

Esta solución es aceptable como **referencia contextual estructurada** si el contrato admite significado dependiente de la fuente. No cierra una exigencia diferente de resolver y publicar toda la relación nominal/clausal en términos autónomos. Para esa exigencia haría falta resolver la adjunción con evidencia adicional y un carrier revisado; un ID de origen, el texto copiado o una marca de completitud no la suplen.

Solo se usa la fuente indicada y los cuatro documentos autorizados de esta fecha ya leídos, más el extracto de `tools/evaluation/src/documentary_v3.rs` anterior a sus tests. No se leen preguntas, matching, pruebas, resultados, gold, otras revisiones o memoria. No hay candidata completa ni comandos de ejecución.

## Shape mínimo propuesto

Se propone una variante optativa de Body `linguistic_composition` con constructor `nonagentive_nominal_event_with_contexts_v1`. Es una ampliación real de la unión cerrada, no un valor ya admitido por el validador inspeccionado. La feature optativa homónima habilita expresamente ese constructor y su lectura; un lector sin soporte rechaza la variante. No se presupone el estado actual de otros constructores que no se inspeccionaron.

Notación cerrada de esquema, no raw completo. `ID`, `Evidence` y `LocalReferenceId` conservan los límites de los contratos autorizados. Todos los objetos rechazan campos extra, ausentes o duplicados:

```text
Body = {
  kind: "linguistic_composition",
  constructor: "nonagentive_nominal_event_with_contexts_v1",
  composition_id: ID,
  evidence: Evidence,
  core: {
    id: ID,
    action_id: RegisteredActionId,
    action_evidence: Evidence,
    construction: "nonagentive_se",
    construction_evidence: Evidence,
    actuality: "occurred",
    temporal_resolution: "source_past",
    patient: {kind: "ref", id: LocalReferenceId}
  },
  region: {
    id: ID,
    kind: "core_with_patient_nominal_closure",
    core_id: ID
  },
  contexts: [{
    id: ID,
    kind: "source_bound_prepositional_context",
    preposition: Evidence,
    complement: {kind: "ref", id: LocalReferenceId},
    complement_evidence: Evidence,
    phrase_evidence: Evidence,
    region_id: ID,
    attachment: {kind: "original_source_context", resolution: "not_published"}
  }][1..2]
}
```

El primer subset admite de uno a dos PPs y exactamente una región, siempre la raíz compuesta indicada. Son límites operativos del constructor, no un inventario de interpretaciones. No hay campos `agent`, `beneficiary`, `possessor`, `attachment_target`, alternativas ni valores de adjunción posibles. `source_past` conserva el tiempo pasado expresado y no inventa una fecha; no publica un intervalo actual de vigencia. Una forma no pasada o con agente expresado queda fuera de este constructor.

`composition_id`, el ID del core, el ID de región y los IDs de contextos son localizadores internos tipados, únicos en su espacio dentro de este Body. No son References V3, Terms, variables del mundo o valores de respuesta. Ningún `ref` puede resolver a ellos. `patient` y los complementos sí resuelven a References V3 del mismo documento y ámbito. El core y la región no se convierten en dos eventos: la región es una vista estructural del único core y su cierre. Su ID no incorpora semántica oculta.

`RegisteredActionId` se resuelve en un registro de acciones tipado y ligado al vocabulario autorizado. Para esta fuente se necesita la acción genérica `remove`: eliminación ocurrida del término paciente, sin agente expresado, sin convertir el paciente en clase normativa ni afirmar que algo siga ausente ahora. No depende de «obligación», Faro, Ina o un ID de caso. Si ese registro no existe, añadirlo y su definición es parte explícita del coste de esta propuesta; no se reemplaza por un string libre ni se declara un registro existente sin inspeccionarlo. La evidencia «eliminó» acredita el lexema; el registro aporta la semántica y su revisión debe aceptar esa correspondencia.

## Instanciación semántica de esta fuente

El paciente H es la ocurrencia nominal con núcleo «obligación», mención íntegra y enlace `nominal_complement_de` a T, descriptor local «tutor». Se reutiliza el contrato nominal existente, con dirección, evidencia exacta y restricciones de incidencia. T no es una persona inventada ni un tutor físico identificado. H no es la clase de todas las obligaciones y el genitivo no se especializa como acompañamiento, propiedad o deber correspondiente a una persona. El constructor exige conservar el cierre nominal íntegro de H; no permite pasar solo su núcleo como paciente.

El core conserva la construcción «se eliminó» sin agente publicado. Un contexto conserva «En Faro» y el otro «para Ina», cada uno con su complemento V3 completo, lexema original y región compartida. Faro e Ina no adquieren tipos o papeles normativos por su posición. Las dos incidencias mantienen el orden de fuente mediante su evidencia y el orden del array; no se deduplican por texto. El Body termina en la primera cláusula; no absorbe la suficiencia tras el punto y coma ni resuelve «su».

La evidencia del Body contiene el core, la mención del paciente y ambos PPs; las evidencias específicas coinciden con bytes UTF-8 de la misma fuente. Los offsets se producirían en una candidata posterior. Contención, exactitud y cobertura son condiciones necesarias, no prueba de adjunción. Se rechazan PPs solapados ambiguamente identificados, referencias de otra ocurrencia y dependencias nominales omitidas; no se reconstruye desde una quote un enlace ausente.

## Significado de origen fijo

Sea o la ocurrencia fuente registrada, K su core estructurado con el cierre nominal del paciente y p cada incidencia PP. El carrier conserva una expresión `C_o(K, PP_1, …, PP_n)` cuya interpretación requiere el contexto original c0(o). Para cada p, su contribución y su lugar de composición son los de esa incidencia dentro de la interpretación original de la región declarada en c0(o). Esta es la definición del operador contextual, no una función ejecutable ya disponible ni una variable que el matcher pueda satisfacer.

No se afirma `∃a Apply(a,K,p)`, no se universaliza sobre adjunciones y no se ofrece una lista de opciones. La referencia a c0(o) es rígida durante entrega, selección y lectura: cambiar consumidor o intención de selección no cambia su valor. Si la fuente y su contexto original determinaran una adjunción nominal, el carrier la conserva por dependencia; si determinaran otra, no autoriza sustituirla por la nominal. La forma no afirma que existan varias interpretaciones en la fuente.

La región identifica exactamente core+paciente+enlaces nominales y sus restricciones, reconstruidos desde el documento. No equivale a «cualquier cosa que diga la fuente»: el operador tiene estos operandos y estas incidencias acreditadas. El paciente no puede cambiar, `remove` no puede convertirse en suficiencia y un PP no puede moverse a la segunda cláusula manteniendo la misma representación. Las dependencias de scope, qualifiers y referencias que el constructor admita permanecen íntegras; el primer subset exige scope raíz único sin ramas, binders, guards o negaciones heredadas. Otros alcances quedan sin soporte, no se aplanan.

La fuente-boundness requiere identidad de ocurrencia externa. Se reutiliza la disciplina del contrato de referencia fuente: un manifiesto autorizado liga `origin_id` a la triple exacta de vocabulario, texto y documento; dos textos iguales no identifican la misma ocurrencia por hash. Dentro de esa ocurrencia, Claim, composition e incidencia PP identifican el uso. El Body no puede autoproclamar su origen ni aportar su propio contexto de destino. El consumidor verifica el documento exacto y la incidencia; no acepta un wrapper que contenga únicamente IDs y flags.

La salida ligada conserva el grafo, no solo texto. El texto y evidencia permiten auditoría, pero el núcleo semántico sigue siendo acción registrada, paciente nominal compuesto, PPs y dependencias rígidas por ocurrencia. El registro de origen acredita identidad/integridad, no la fidelidad de ese grafo. Una crítica fuente separada tiene que aceptar acción, estructura nominal, identificación de PPs, pertenencia a la región y conservación contextual; sin ella hay referencia candidata pendiente, no fidelidad demostrada.

## Límite concreto y corrección mínima

Si un lector exporta `remove(H)` y después muestra «para Ina» como nota, ha perdido el operador: puede concluir una eliminación irrestricta que el compuesto no publica. Si exporta `requires_class(Ina,tutor)=false`, además introduce una lectura normativa y actualidad no expresadas. Conservar los bytes de «para Ina» al lado no corrige ninguna de esas dos pérdidas.

Otro fallo sería interpretar la región como host exterior ya demostrado. Eso identificaría esta propuesta con `whole_clause_only` y volvería a introducir la decisión que se quiere evitar. La corrección mínima es la semántica `original_source_context` especificada: región como dominio estructural, resolución no publicada y prohibición de proyección desnuda. Si un sistema exige una proposición independiente de c0, esta corrección no basta: el bloqueo conceptual permanece hasta resolver y representar la relación interna con evidencia. No se anuncia cierre de ese objetivo más fuerte.

## Consumo parcial positivo

Una ruta conocedora del constructor puede seleccionar y devolver el compuesto como evidencia contextual por coincidencia de la acción registrada y un ancla exterior acreditada del paciente o de un complemento. Esa selección compara únicamente términos cerrados ya acreditados en la misma entrega; no usa un PP para asignar un papel semántico interno, no prueba identidad entre fuentes por nombre y no elige un valor contextual para satisfacer una petición. La selección no afirma una respuesta a quién tiene obligación, quién acompaña o qué se requiere actualmente.

La salida positiva lleva `kind: "source_bound_linguistic_context"`, la referencia de origen/entrega verificada, Claim y Body íntegros, documento/vocabulario fuente íntegros, cierre efectivo reconstruido y los criterios de selección realmente acreditados. Es un testigo tipado; se necesita añadir esa variante de salida al contrato, sin fingir que el testigo posesivo existente la admite. Conserva `answer_terms:[]`, `answer_propositions:[]`, `bindings:[]`, `attachment_resolution:"not_published"` y `full_answer_supported:false`. No se da un wire completo de runner/CLI ni una candidata en esta etapa de diseño.

Seleccionar es positivo sobre relevancia de una construcción contextual íntegra, no sobre una conclusión normativa ni sobre adjunción resuelta. Los contextos no constituyen dos resultados combinables: el testigo transporta ambos con el único core. Falta de soporte, origen no verificado o crítica pendiente impide ese positivo y conserva la causa correspondiente; nunca produce un testigo truncado.

Todas las rutas ordinarias, de índice, proyección, guard, cierre, respuesta o exportación que no conozcan el constructor deben retener el compuesto pendiente y no acceder al core como proposición utilizable. Esta prohibición se aplica antes de seleccionar o derivar, no solo al serializar. La mera presencia de una feature no valida a un consumidor. Los claims independientes conservan su tratamiento propio.

## Coste y alcance del diseño

La ampliación añade una variante Body, un constructor cerrado, soporte de acciones registradas si falta, localizadores internos tipados, dos incidencias contextuales y un testigo específico de referencia estructurada. Requiere validación de región/cierre, ligadura externa a origen, consumo indivisible y crítica semántica; no es solo aumentar un contador de contextos. Cada recorrido, comparación, hash y copia usa el presupuesto acumulado existente, sin reiniciarlo por PP y sin ampliar cotas por conveniencia.

Esta especificación conserva una dependencia semántica de origen en una estructura explícita y ofrece selección parcial del conjunto. No demuestra que esa referencia pueda consumirse como una proposición autónoma, que la crítica de fidelidad vaya a aceptarla, que el motor la implemente ni que mejore rendimiento o calidad. No autoriza implementación: la siguiente puerta es la crítica de este contrato.
