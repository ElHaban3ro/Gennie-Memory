# Banco prospectivo de factores M4 v1

24 fuentes y 24 preguntas sintéticas, ocho por partición. Fuentes, nombres y ámbitos separados; familias de plantillas compartidas. Cada pregunta consulta un ámbito con una memoria para aislar los tres factores del clasificador, no la calidad de ranking entre distractores.

`manifest.json` liga corpus, casos, etiquetas y paquete ciego por SHA-256. `labels.json` contiene factores, relación, respuesta permitida, selección y error prohibido; no se envía al proveedor. Solo `cases.json` y `corpus.json` forman sus entradas. La revisión independiente de 24 casos es de agente. Fernando confirmó ocho etiquetas de la muestra y la corrección lógica de la fuente06 antes de inferencia; el registro y la adjudicación preservan el historial.

[Protocolo, revisión y resultados](../../../docs/evaluation/m4-completion-v1/README.md). No usar la partición final para modificar cortes, preguntas o etiquetas después de ver salidas.
