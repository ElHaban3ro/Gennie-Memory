# Crítica prospectiva del contrato de evento nominal contextual

Fecha: 2026-10-22. Dictamen: **sin bloqueo conceptual para el objetivo local declarado**, condicionado a conservar las obligaciones negativas del contrato durante su implementación. No es una aceptación de un DTO todavía inexistente ni de su fidelidad fuente; tampoco acredita código, ejecución, utilidad o extracción autónoma.

Se revisaron el diseño de evento nominal contextual, su contrato de implementación y el contrato de referencia a contexto fuente. Se consultaron las reglas backend/testing para delimitar esta revisión documental. No se inspeccionaron código de matching, pruebas, campañas, preguntas de evaluación, gold, otras críticas o memoria. No se ejecutaron comandos de comportamiento.

## Semántica y correlación

`C_o(K,PPs)` es admisible como expresión cuyo contexto original es un parámetro rígido de la ocurrencia externa, no una variable libre satisfecha por el consumidor. El manifiesto no demuestra cuál es la adjunción ni la verdad de la fuente; el contrato tampoco se lo atribuye. Cambiar la pregunta exige otra entrega y conserva la ocurrencia. La distinción entre dependencia contextual conservada y resolución publicada está formulada expresamente.

La región `core_with_patient_nominal_closure` contiene el nominal y sus enlaces. No establece un host exterior para los PP. Por ello no corresponde reducir esta propuesta a `whole_clause_only`: hacerlo descartaría precisamente la posibilidad de que «para Ina» contribuya dentro del nominal. Tampoco corresponde exigir que resuelva contextos desconocidos: el objetivo acepta una referencia a la interpretación original, sin publicarla como proposición autónoma.

El core, el paciente compuesto y ambos PP forman una sola expresión. El contrato impide publicar `remove(H)` separadamente o convertir los PP en notas descartables. Esa indivisibilidad conserva la correlación relevante: las contribuciones pertenecen al mismo core, al mismo paciente y a la misma ocurrencia; no son resultados combinables entre sí ni con otro evento. Sigue pendiente que una candidata concreta conserve todos esos operandos y su incidencia correcta.

## Selección parcial y deuda de rol

El target `action_beneficiary` puede servir de petición para seleccionar contexto sin que el selector afirme esa relación. La coincidencia de acción, patrón nominal y ancla nombrada de un complemento PP acredita únicamente la relevancia limitada que declara el contrato. La identidad del ancla debe demostrarse con las restricciones existentes; la correspondencia de patrones nominales no identifica dos obligaciones físicas.

Esta selección solo conserva su significado si mantiene vacíos términos, proposiciones y bindings de respuesta; `full_answer_supported:false`; adjunción no publicada; y, como mínimo, las deudas `answer_slot`, `patient_identity`, `context_attachment` y `requested_role`. Encontrar el ancla en un PP no puede saldar `requested_role`, aunque su preposición sea «para». Tampoco la acción remove acredita ausencia actual ni una conclusión normativa. Las comprobaciones de configuración, tiempo, guards y dependencias efectivas no pueden convertirse en comodines para obtener ese parcial.

No hay un defecto por no acreditar el rol solicitado: esa ausencia es parte explícita de la salida autorizada. Sí habría un cambio de objetivo si un consumidor tratara la selección como apoyo al beneficiario, a la identidad del paciente o a una respuesta completa.

## Transporte, compatibilidad y presupuesto

V2 conserva las obligaciones útiles de V1: identidad externa de ocurrencia, entrega ligada a pregunta, documento íntegro, selección por la ruta real y reclasificación con comparación completa en consume. Copiar otros claims no les otorga valor de respuesta. Permitir dos tipos de testigo mediante permisos separados es consistente; no autoriza aceptar un testigo nuevo bajo el permiso histórico ni modificar V1.

La activación OR de los dos constructores composicionales evita que el nuevo dependa accidentalmente de duration. El presupuesto único, incluida evidencia escalar sin feature10, recorre validación, cierre, copia, canonicalización, hash, reclasificación y comparación. El agotamiento posterior a un candidato positivo invalida la operación entera. El contrato no propone reinicios por PP ni cotas aumentadas, y por ello no presenta una excepción conceptual de recursos.

Antes de codificar conviene fijar en la definición tipada del testigo nuevo la lista exacta de campos/flags y la regla de deudas: las cuatro citadas son obligatorias; las adicionales deben reflejar las dependencias efectivas que el contrato ya exige conservar. «Mismas formas cerradas» no debe implementarse como un objeto genérico tolerante ni «deudas exactas» como una lista fija que borre deuda adicional. Es una precisión de esquema de la obligación existente, no un mecanismo semántico nuevo ni un bloqueo del diseño.

## Cierre y límites

No se solicita rediseñar la región, resolver la adjunción o añadir una proposición autónoma. El cambio mínimo necesario es materializar las restricciones ya declaradas sin ampliar su lectura: compuesto indivisible, referencia rígida al origen, selección realmente parcial y testigo cerrado con deudas conservadas.

El dictamen habilita avanzar hacia una implementación revisable de ese mecanismo local. Su aceptación posterior requiere comprobar los cortes de consumidores, la selección parcial positiva y el transporte íntegro. La fidelidad de una representación concreta sigue requiriendo crítica fuente separada; esta revisión no la anticipa ni modifica las puertas posteriores de extracción y adopción.

## Adenda: revisión de la precisión cerrada anterior al código

Revisión adicional del apartado «Precisión cerrada anterior al código» del contrato de implementación, sin inspeccionar código, pruebas, gold o resultados. **No hay bloqueo conceptual en este addendum.** El dictamen anterior se conserva.

La enumeración cerrada del testigo y de `nominal_event_context` resuelve la precisión pendiente sobre campos y flags. Mantiene cuatro deudas mínimas más las efectivas adicionales, sin reemplazarlas por una constante; una incompatibilidad o cobertura externa no representada impide el positivo. Los IDs y arrays deben derivarse y coincidir con el Body/documentos validados. Así, los flags no se convierten en declaraciones de confianza aportadas por el consumidor ni el esquema cerrado borra deuda real.

Sembrar el corte con el Claim y las referencias concretas H y T cierra una vía de pérdida: otro Claim que usa ese mismo H no puede recuperar el nominal desnudo solo por omitir una dependencia explícita a la composición. La propagación por aliases/dependencias preserva esa obligación. Se trata de incidencia e identidad dentro del grafo, no de bloquear otros nominales que compartan palabras con H o T.

Excluir de esa siembra indiscriminada los complementos PP nombrados es coherente con la indivisibilidad del compuesto: el PP sigue retenido dentro del testigo, mientras un hecho independiente sobre Ina o Faro conserva su tratamiento propio. No hay contradicción entre bloquear el consumo del mismo paciente compuesto y permitir evidencia independiente sobre una referencia nombrada que también aparece como complemento.

El contraste exigido entre consumidor del mismo H y hecho independiente sobre un complemento, junto con aliases y proyección existencial, corresponde al límite que se quiere preservar. Es una obligación de implementación y verificación posterior, no evidencia de que el corte ya funcione. La adenda no convierte la selección en respuesta, no resuelve el rol ni la adjunción y no cambia c0(o).
