# Cohorte nueva para testigos de aplicabilidad v1

Ocho conversaciones ficticias 050–057, 224 recuerdos y 64 preguntas; 16 por relación. Fuente, preguntas y etiquetas nuevas para el mecanismo, sin usar salidas del proveedor para diseñarlas. Los temas, plantillas y distractores son repetitivos: no se atribuye dificultad humana o generalización amplia a un resultado.

La variación cubre equipo, sede, edición, entorno, colección, modalidad, ciclo y responsable. Cada conversación contiene 28 recuerdos, incluidos distractores de otra variante y registros auxiliares repetidos. Los calificadores están en el texto, no en `entity_id` ni `conditions` estructurados. Resumen y fuente coinciden literalmente, sin compresión. IDs `cev3_...`, consultas `cev3q_...`; orden mezclado con semillas 1850/1851/1852.

`required_query_bindings` es anotación de evaluación de referentes que deben extraerse; no se transmite al clasificador o extractor. La revisión ciega separada solo recibió `source.json` y `blind-questions.json`, sin etiquetas, cuota o resultados. Confirmó las 64 categorías/citas y detectó que faltaba «presencial» en ocho anotaciones. Se añadió antes de las llamadas y se conservó la anotación original. Esa revisión es entre agentes de la misma familia, no humana ni independencia de familia; `independent_review` permanece falso.

El [contrato](../../../docs/evaluation/applicability-witness-v1-contract.md) distingue citas válidas de completitud de extracción. La conciliación y manifiesto congelan los datos antes del proveedor. No se reetiquetan resultados después de ejecutarlos. Esta cohorte es un contraste sintético nuevo, no una reserva humana o prueba de respuestas finales.
