# Banco sintético de evidencia v6

224 recuerdos,64 preguntas y8 ámbitos nuevos074–081. Preparado antes de ejecución remota, con autor separado y revisión ciega de conversaciones/preguntas, seguida de conciliación. Los revisores son de la misma familia de modelos; no es validación humana ni independencia estadística. Los campos técnicos human_review e independent_review siguen false; review_status registra revisión separada.

La revisión inicial, el autor inicial y todas las discrepancias se conservan. Dos consultas se aclararon antes de ejecución y tienen revisión posterior explícita; esa segunda etapa no se llama ciega. 075_05 solicita hora y estado taxonómico: dos evidencias parciales juntas responden ambas partes, y el caso se etiqueta supports_answer por cobertura conjunta. 076_04 pregunta tipo de celdas braille y conserva la corrección de premisa. El manifiesto congela preguntas, etiquetas y cobertura antes de llamadas. No se exige equilibrio a costa del significado:17 apoyo,16 refutación,15 parcial,16 ausencia.

Cada requested_parts declara ID, tipo requested/premise y cita literal de consulta; covers enumera las partes concretas aportadas por cada recuerdo. Es una anotación fuente, no una inferencia del clasificador. Cobertura de partes disponibles se informa separada de cobertura de todos los datos solicitados: lo desconocido no se inventa. Una corrección de premisa no responde necesariamente el atributo cuya existencia niega.

No ejecutar hasta superar las puertas de desarrollo conocidas y congelar la política. La lectura de este banco para validación de etiquetas no equivale a resultado remoto, pero cualquier ajuste de política motivado por él lo convertiría en desarrollo observado. No usar sus textos para afinar el clasificador.
