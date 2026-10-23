# Contrato experimental de aplicabilidad con citas v1

Fecha: 2026-10-18. `selection_policy: witness_v1` propone un recuerdo con la instrucción congelada `utility_v2` y, si existe propuesta, solicita una extracción estructurada a `openai/gpt-4.1-mini` por OpenRouter. Decisions conserva su modelo vigente. Hay una llamada adicional por propuesta; una primera abstención no llama al extractor. La base sigue predeterminada.

La interfaz del módulo `apps/retrieval/src/applicability.rs` recibe pregunta, resumen y `Witness`. Devuelve `accepted`, `incompatible` o `not_established` con motivos; un contrato/cita inválido es error técnico. Sus campos obligatorios son:

- `version`: `applicability-witness-v1`.
- `query_bindings` y `memory_bindings`: listas de `{dimension, quote}`. Dimensiones: `subject`, `item`, `edition`, `site`, `team`, `environment`, `time`, `condition`, `person`. Se permiten varios valores por dimensión para condiciones compuestas; no se permite repetir el mismo par dimensión/cita. Máximo dieciséis bindings.
- `quote`: fragmento literal no vacío del texto correspondiente, máximo 512 bytes. No se permiten paráfrasis ni valores canónicos inventados. La cita representa el valor mínimo del referente; no el dato todavía desconocido por el que se pregunta.
- `relation`: `supports_answer`, `refutes_premise`, `pending_or_partial`, `unrelated` o `unknown`.
- `coverage`: `complete` o `uncertain`, estimación del extractor sobre si recogió todos los referentes especificados.

El comprobador exige citas presentes literalmente en su fuente y dimensiones válidas. Normaliza solo mayúsculas y espacios, sin alias, traducción, similitud ni eliminación de calificadores. Exige `subject` en pregunta y correspondencia de cada dimensión de pregunta con la misma dimensión de memoria: algún valor requerido distinto de los valores extraídos de esa dimensión → `incompatible` (desacuerdo de campos, no prueba de contradicción lógica); valor ausente o cobertura incierta → `not_established`. Una relación no útil o desconocida tampoco se acepta. Solo `accepted` conserva el ID propuesto. La corrección de una premisa puede aceptarse bajo el mismo referente; no se compara una premisa negada como si fuera un nuevo sujeto.

El resultado de `recall` añade `applicability`, nulo en políticas sin prueba, con ID propuesto, consumos separados del proponente y extractor, testigo, resultado y motivos. La extracción no recibe etiquetas ni campos gold del banco. El servicio vuelve a comprobar testigo, resultado e ID contra los candidatos antes de componer. Una extracción válida que no demuestra correspondencia termina como abstención completa; fallo HTTP, respuesta inválida o cita inventada degrada y no genera semilla fiable. Se preserva consumo conocido de etapas anteriores y de la extracción cuando ya existe recibo; coste de una llamada fallida sin recibo queda desconocido, no cero.

## Qué comprueba y qué no

Esto comprueba fidelidad literal de las citas y acuerdo entre campos extraídos. **No demuestra completitud semántica.** El extractor puede omitir un calificador importante, confundir su dimensión o citar solo el tema compartido. Si omite una diferencia en ambas listas, el comprobador puede aceptar incorrectamente. El banco nuevo anota referentes requeridos para medir esas omisiones por separado; esos campos no se transmiten al producto. Citas correctas no se presentan como garantía de aplicabilidad.

También puede rechazar paráfrasis/alias legítimos o seleccionar dimensiones distintas para expresiones equivalentes. Esta versión conservadora no mantiene un catálogo de alias. La relación de utilidad sigue siendo un juicio del modelo; no se valida respuesta final, fuente externa o extracción infalible. El testigo no modifica recuerdos, índices, feedback ni estado persistente.

## Adaptación y coste

Se utiliza salida `json_schema` estricta con `additionalProperties: false`, `provider.require_parameters: true`, sin fallback, temperatura cero y máximo 1.000 tokens de salida. Se comprueba además localmente, sin asumir que formato válido implica significado correcto. La compatibilidad de formato se contrastó con la [documentación oficial de OpenRouter](https://openrouter.ai/docs/guides/features/structured-outputs); el modelo y precios publicados se consultaron en su [ficha oficial](https://openrouter.ai/openai/gpt-4.1-mini). La factura y el coste real solo se describen mediante recibos de las llamadas.

El coste del mecanismo incluye selección y extracción cuando procede. Se informan llamadas efectivas, tokens de entrada/salida, recibos por etapa, coste conocido y desconocido, resultados por relación y diagnósticos de extracción. Se compara también el ID propuesto antes del comprobador con su salida para distinguir efecto de la selección inicial y de la comprobación. Cambiar de extractor respecto al clasificador es parte del mecanismo evaluado; no se atribuye una ganancia exclusivamente al contrato.
