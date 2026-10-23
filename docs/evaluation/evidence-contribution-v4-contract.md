# Contribución V4: estado de evidencia y alcance explícito

UTC 2026-10-20. Contrato prospectivo de desarrollo. No sustituye gold ni resultados V1–V3, incluidos 98/100 del resolver manual. Los cinco objetivos, preservación de evidencia, comparación de recursos ≤1,5×, campañas completas, estabilidad y cohorte nueva siguen pendientes. No activa una política de producto.

## Identidad, pregunta y afirmación

La pregunta declara sujeto/colección, condiciones fijas, configuración supuesta, atributos solicitados y premisas necesarias. Una atribución corregible no cambia el sujeto. Cada afirmación se formaliza sin la pregunta: sujeto, atributo, modalidad, condiciones, valor/límite/estado, polaridad, alcance y fuente del estado. Hechos de textos idénticos son idénticos. Los nombres, IDs y resultados anteriores no participan en reglas.

Mantener por separado: valor de una magnitud; disponibilidad del dato en un registro/copia; ejecución o ausencia de ejecución de un procedimiento que mide esa magnitud; planificación; admisión/capacidad; asignación/uso y observación de un ensayo. No se deriva desconocimiento desde no-medición, ni ejecución desde intención, ni uso desde capacidad, ni máximo general desde un ensayo. Los calificadores se conservan al transmitir evidencia.

## Relaciones y límites

B, A de valor/límite/parte/disponibilidad y C conservan precedencia y exigencias V3. B requiere incompatibilidad bajo la misma identidad, condiciones y alcance. Un miembro puede refutar una afirmación universal factual, pero no establecer una política común. Una configuración alternativa no es exclusiva por defecto. C exige ambas asignaciones explícitas y un valor alternativo real en una clave supuesta.

V4 amplía expresamente A: **estado de ejecución de una medición** de la magnitud solicitada, documentado como realizado o no realizado, para el mismo sujeto y condiciones. Es información sobre evidencia, no sobre el valor ni su disponibilidad global. Deben constar magnitud y procedimiento: la pregunta no los aporta a un texto genérico. Una relación independiente método→magnitud es un hecho de dominio; no expresa utilidad, rol ni coincidencia. Una mención meramente futura o una medición de otro atributo, sujeto, componente no aditivo o condiciones incompatibles no satisface esta vía.

Ejemplo: al preguntar por autonomía máxima, «la prueba de autonomía por descarga hasta apagarse no se realizó» permite conservar ese estado con el límite «no acredita desconocimiento del máximo ni ausencia de especificación». «Se prevé esa prueba» aislado no informa ejecución. «No se midió la masa» no aporta autonomía. Si el texto además especifica máximo 90, ambas cláusulas se conservan sin hacer que la ausencia de prueba contradiga el máximo. Una medición realizada sin publicar su resultado acredita solo realización, no su valor.

Esta extensión cambia el alcance de A; no es una reparación de anotaciones ni prueba de cumplimiento histórico V3. Se medirá bajo esta versión y sus propios casos revisados.

## Ambigüedad de relación y de utilidad

Cuando un texto admite lecturas semánticas explícitas, se evalúa cada una y se conserva **el conjunto completo** de roles, no el rol que más convenga. Si todas las lecturas son útiles (A/B/C), puede conservarse el texto con relación no resuelta y calificación obligatoria, sin afirmar una refutación. Por ejemplo, exclusividad inmediata de depósito frente a exclusión de cualquier recorrido de red puede producir {A,B}; no se publica B como hecho demostrado. Si alguna lectura es none, la selección automática queda sin resolver y se deriva a revisión; no se cuenta como rechazo seguro ni como recuperación completa.

Para una relación resuelta el conjunto es unitario. Para una ambigua, el banco exige el conjunto exacto, además de selección y necesidad de calificación: acertar una alternativa no basta. Se publican separadamente ambigüedades de relación, utilidad sin resolver y pérdidas frente a un criterio de utilidad en cualquiera de las lecturas. No se transforma una falta de cobertura V3 en acierto histórico V3. La puerta de calidad del mecanismo completo seguirá exigiendo que no haya evidencia útil perdida ni selección ajena, además de relaciones/límites correctos.

## Implementación y evaluación previstas

El diagnóstico local reutiliza los hechos V1 y añade una relación factual de medición con `measured_attribute` y `measurement_method`. La ontología distingue propiedades, magnitudes y mediciones. El código calcula relación; el extractor no recibe ni devuelve role/useful/same_case. Se conservan fuente original, interpretaciones y límites. No se genera una respuesta final.

Primero se revisan casos límite sintéticos y sus hechos sin etiquetas ni resultados. Después se ejecuta la prueba manual, invariancia de nombres/orden y regresiones V1. Solo un pase local permite preparar evaluación de extracción; un pase manual no prueba extracción, recursos, estabilidad ni generalización. Las métricas históricas y V6 permanecen congeladas. La calidad de degradación ante ambigüedad se informa aparte; un resultado estructural correcto que requiere revisión no demuestra autonomía.


Precisiones conciliadas antes de ejecutar: «pendiente» de completar un procedimiento es un estado actual de ejecución, distinto de una mera previsión; aporta A con calificación cuando magnitud, sujeto y condiciones corresponden. No acredita si se inició, cuánto se avanzó, si se completará o si falta el dato por otras vías. No se convierte en unknown de la cantidad. «No publicado» no informa realización del procedimiento; la no publicación de un resultado de ensayo no identificado como el máximo pedido no satisface por sí sola ejecución ni ausencia documental del dato máximo. Una ausencia explícita del propio máximo en un registro sí conserva la vía de disponibilidad acotada. El banco contrasta ambos textos.

Measurement designa un procedimiento explícito para obtener o estimar una magnitud, incluidas estimaciones y cálculos declarados como tales. Su estado no prueba validez física, precisión ni equivalencia a observación empírica. Se conservan método, fundamento y calificadores. Si la propia pregunta consulta realización de una prueba, su identidad incluye magnitud y método, no solo el nombre del método. La representación diferencia máximo especificado (`basis=specification`), norma (`basis=normative`) y observación; no se eleva un resultado de ensayo a máximo general. Estos campos son hechos de fuente, nunca juicios de utilidad.

Identidad formal de medición: premisas y preguntas que mencionan una prueba conservan tanto magnitud como método, incluso cuando su dato es pending sin texto numérico. Una pregunta que cuantifica métodos o magnitudes en general declara el alcance any explícitamente; una omisión no es un comodín. La identidad no se completa desde la afirmación candidata. El puntuador exige también concordancia de roles/límites con la revisión ciega final y adjudicación ligada a fuente/etiquetas, no solo existencia de archivos.
