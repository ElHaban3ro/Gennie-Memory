# Contrato experimental: utilidad por recuerdo y semilla múltiple

Fecha: 2026-10-19. Estado: implementación optativa, sin promoción. Política `per_memory_v1`. Responde al fallo de vetar IDs distintos aunque aporten partes compatibles y al acuerdo sobre requisito relacionado sin refutar la premisa.

## Datos y método

Entradas obligatorias: `RecallRequest.search` con pregunta, ámbito y fecha; opcionalmente proveedor y política. Para activar esta variante se envía `provider: jev`, `selection_policy: per_memory_v1`; omitirlos conserva Decisions/baseline. Recuperación20 igual que antes; se clasifican los primeros16, no se generan índices nuevos. Se mantienen DB, revisiones, origen y texto literal.

Una petición contiene hasta16 preguntas `memory_00`…`memory_15`, una por cada recuerdo del prefijo. `state` lleva pregunta, ámbito y reglas comunes; cada instrucción contiene la síntesis literal correspondiente. La [API documentada](https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request) acepta varias preguntas tipadas y un recibo global. Esto no garantiza límite16, coste o latencia: el piloto real debe comprobarlos.

Cada pregunta devuelve choice entre A=respuesta explícita, B=corrección explícita de premisa, C=parte concreta solicitada o estado útil, none=sin dato solicitado/corrección. Se exige mismo referente/contexto, preservar negación/tiempo/condición/estado y no inferir que un requisito sea exclusivo. Las memorias no compiten entre sí. Las preguntas comparten petición/contexto: no se afirma independencia estadística.

Bootstrap adapta formato y cuenta el recibo global una sola vez. `retrieval::per_memory` posee tipos y validación estructural: un juicio por ID/revisión del prefijo, en su orden, sin omisiones, sustituciones, extras o reordenamientos. Etiquetas/probabilidades inválidas o juicio ausente degradan la operación entera sin semilla parcial fiable y conservando consumo conocido. Una búsqueda vacía no llama.

## Resultado y consumidores

Todos los juicios distintos de none se componen, en orden de búsqueda, copiando las síntesis y referencias originales. No se truncan ni se llaman contradicciones por tener IDs distintos. `per_memory.items` contiene ID, revisión y relación asignada. El servicio revalida antes de componer; no comprueba verdad semántica de las etiquetas ni resuelve contradicciones reales entre fuentes.

`selected_ids` es la lista íntegra de IDs en `seed.memories`, también en respuestas de rutas anteriores. En esta variante `selected_id` es el primero o null como alias de compatibilidad; **no representa por sí solo toda la selección**. Las rutas históricas de revisión humana pueden conservar su alias previo cuando hay múltiples recuerdos. El dashboard conserva revisiones y su composición literal, sin cambios de interfaz ni datos persistidos. Auditores y evaluadores deben examinar todos los elementos, no solo el alias.

Hasta16 positivos aumentan superficie de falso positivo y contexto posterior. Registrar número de recuerdos y bytes de semilla, no llamarlos tokens ni coste de Codex. No hay presupuesto completo de semilla o integración automática en esta prueba.

## Verificación y puertas

Pruebas deterministas: partes compatibles preservadas, abstención total, búsqueda vacía sin llamada, coincidencia de lista completa y primera selección, ID/revisión cambiado, juicio ausente/adicional/inválido y recibo único. Validación estructural no verifica atribución correcta del modelo; el caso de avalista solo se puede contrastar semánticamente con fuente/rúbrica, no aprobar con mocks.

El evaluador debe bloquear cualquier ID no permitido en el conjunto, aunque el primero sea correcto, y conservar toda evidencia útil de la base. En bancos conocidos sin cobertura por partes anotada prospectivamente, exigir inclusión exacta del ID útil de base es una puerta conservadora: puede rechazar alternativas equivalentes; no llamarla completitud semántica. Un banco nuevo deberá fijar partes solicitadas y cobertura por ID con revisión ciega antes de llamadas. La clasificación recibirá textos sin etiquetas gold.

Se mantienen cero regresiones, cero ID no permitido y ninguna degradación nueva; al menos una ganancia de utilidad o evidencia útil adicional. Coste y p50/p95 ≤1,5× base, recibos completos. Una propuesta de conjunto no promueve automáticamente la ruta. Los bancos v2/v4/v5 observados son desarrollo; validación nueva requiere otra cohorte.
