# Identidad parcial de medición: extensión de representación V5

UTC 2026-10-20. Desarrollo prospectivo sobre el contrato documental V5, sin alterar V4, sus bancos o resultados. El objetivo es representar textos que declaran una magnitud medida sin nombrar un procedimiento. No se añade un método inventado ni se interpreta identidad ausente como comodín.

Un hecho con `attribute` de tipo measurement puede declarar `measured_attribute` y `measurement_method_scope: "unspecified"`, sin `measurement_method`. Su dato conocido usa el identificador neutral `measurement`: acredita el estado de esa medición según polaridad y modalidad, no un método determinado. El ámbito explícito se conserva en cualquier hecho publicado. Es incompatible con un método conocido, otro atributo o un dato conocido que identifique un método específico. No se permite omitir silenciosamente el ámbito.

Una pregunta genérica declara `measurement_method_scope: "any"`; una pregunta por magnitud genérica declara además `measured_attribute_scope: "any"`. Un hecho de método no especificado puede informar medición genérica de su magnitud; no responde sobre un método concreto ni refuta una premisa sobre ese método. Las premisas conservan la identidad completa requerida por V4. `unspecified` de fuente y `any` de pregunta no son el mismo concepto.

La vía de estado de ejecución hacia una pregunta por el valor de una magnitud conserva la exigencia V4 de un procedimiento identificado. La extensión no hace que «se midió autonomía» revele una autonomía numérica ni que implique desconocimiento del valor. Tampoco convierte una medición planificada en realizada. Una prueba hasta apagarse conserva su método explícito y magnitud correspondiente cuando la fuente los permite; su ausencia no es `unknown` de la magnitud.

La migración del desarrollo conocido se revisa semánticamente antes de ejecutar y conserva originales, preguntas, fuentes y roles históricos por separado. Donde un texto o pregunta sea ambiguo, se conservan las lecturas o revisión; no se escoge el rol que favorezca el pase. Medir sufiencia manual no demuestra extracción automática ni mejora de una respuesta final. El diagnóstico no abre V6.
