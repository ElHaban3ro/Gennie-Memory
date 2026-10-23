# Disposición representacional de indeterminación normativa

Contrato optativo de evaluación implementado y verificado localmente; una pregunta conocida recibió doble revisión fuente. La puerta completa116 y las pruebas de utilidad/producto siguen pendientes. Sigue la crítica de Faro: el agente subordinado ya tiene posición tipada; no se inventa otro operador o referente. No cambia producto, banco, etiquetas o dictámenes históricos.

## Forma y semántica

Feature nuevo `normative_question_indeterminacy` dentro de `documentary_request_context_v1`, que requiere además `normative_action_relation`. Forma inicial solo para requests de pregunta: acción positiva requerida/state, answer/actor, roles action_kind/object, filtro de clase, fundamento exacto normative y un único contexto normative_action_relation de propósito o condición necesaria resuelta. La acción subordinada conserva símbolo declarado, destino referencial, posición de agente y evidencia íntegros. El scope normativo puede ser string o null; null significa no explicitado, nunca igualdad con destino, ausencia demostrada de autoridad o wildcard. La denotación y semántica del curso permanecen sin cambios.

Se admite como estructura la posición del actor `unresolved/ambiguous_control`; sigue sin asignar identidad, existencia o inventario cerrado de candidatos. `extraction_unresolved` no recibe esta disposición. `unspecified`, referencia y owner_slot conservan significados y diferencias existentes; el feature no los sustituye. Relación unresolved, fragmentos perdidos, cobertura unresolved/unsupported o errores de forma siguen parciales/invalid según el contrato existente. Los contextos de fuente no se amplían por este feature.

El constructor debe satisfacer todas las restricciones actuales excepto (a) la deuda tipada del agente y (b) que el filtro de scope no explicitado sea null. No basta filtrar códigos de issues: una condición de soporte distinta sigue rechazando. Binder, rama, guard, premisas y otras formas fuera del alcance inicial se conservan y no reciben la disposición completa. La evidencia exacta y el grafo/dependencias se validan como antes. Toda elegibilidad se deriva del contenido validado, no de un flag de confianza del extractor.

## Salida

El reporte optativo separa representación de resolución con campos cerrados `representation_disposition` y `resolution_disposition`. Debe transportar la lista completa de deudas, sus paths y su procedencia; los issues originales no desaparecen ni cambian de ID. Puede declarar `status:complete` representacional únicamente cuando todas las unidades estén representadas por el constructor admitido, todos los otros controles pasen y los únicos issues sean las dos deudas exactas reconocidas del request. Esa disposición no prueba fidelidad semántica, respuesta ni aceptación del banco116. Un campo tipado explícito informa que la resolución/elegibilidad continúa bloqueada. El formato exacto se congelará en el código y fixtures del módulo antes de ejecutar controles; no se serializa JSON como texto.

Fuera del opt-in se conserva el informe histórico por bytes. No se trasladan estas reglas a otros kinds unresolved. Quitar evidencia, relación, destino, clase, papel o cobertura impide completitud. El cambio de cobertura del verbo solo se admite si su constructor y posición están presentes; no soluciona ningún fallo por sí solo. La doble crítica fuente sigue siendo requisito independiente para contabilizar una corrección.

## Consumo obligatorio

Todo request admitido con actor ambiguo o scope null conserva deuda efectiva antes de legacy y de cualquier respuesta/proyección. Todos sus canales de soporte, refutación y respuestas/bindings quedan vacíos; se conserva un candidato pendiente con request y documento original íntegros y deudas tipadas. No se permite completar scope desde destino ni actor desde una respuesta atractiva. No se reutiliza un subconjunto sin finalidad o un término de tutor como respuesta. Las conclusiones existenciales y agregaciones entre lecturas no eluden este corte. Los otros requests y hechos independientes mantienen su comportamiento.

El corte se aplica incluso si los inputs se pasaron a classify mediante una API interna con informe parcial: la deuda se deriva del request, no de la etiqueta complete ni de que falte un issue. Un input inválido no recibe una disposición favorable. No hay matcher nuevo ni selección de una contraparte aquí.

## Presupuesto y controles

Se usan límites y contador contextual acumulado existentes. Precobrar recorridos y copias del request/documento/deudas; un fallo agota la operación completa. El feature ausente no inicia el nuevo recorrido. Evitar una mutación de documentos originales o informes previos.

Controles prospectivos obligatorios: constructor con actor ambiguo y scope null completo representacional/bloqueado operacional; extracción fallida parcial; evidencia/roles/contexto perdido no pasan; scope null no igual a destino ni wildcard con actor explícito; purpose no equivale necessary_condition; referencia/owner_slot no se eligen para el ambiguo; todos canales/proyección y dependencias conservan deuda; hecho/request independiente mantiene respuesta; presupuesto falla global; feature ausente exacto. Reproducir el conjunto de informes previo y verificar ambos documentos completos en controles semánticos, separando fixtures inválidos. No llamar rendimiento autónomo a estas pruebas.

La implementación solo prepara una candidata fiel comprobable. No relaja el gate completo116, la doble crítica ni las puertas de utilidad, comparación, recursos y V6. Los seis pendientes siguen pendientes hasta que exista evidencia de aceptación.
