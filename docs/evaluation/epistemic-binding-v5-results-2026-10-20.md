# Proyección de conocimiento documental V5: límites preservados

UTC 2026-10-20. Avance del paso 3 del objetivo de calidad; no cierre del goal, nueva política runtime ni mejora de recuperación automática demostrada. Contrato prospectivo [V5](epistemic-binding-v5-contract.md), con V3/V4 históricos conservados.

El mecanismo anterior trataba cada lectura de una etiqueta como una posibilidad y quedaba sin decisión ante A/none. Esta versión distingue esas lecturas de lo que permite afirmar el registro. Puede conservar «esta etiqueta podría aportar un teléfono a K6» sin publicar ese número como teléfono de K6. Si otra cláusula cierta registra un número, lo conserva con el texto y alcance de su fuente; no acredita vigencia ni verdad externa.

## Qué se implementó y revisó

Módulo puro de evaluación `tools/evaluation/src/typed_facts/v2/record_projection.rs`, reutilizando compatibilidad y roles V4. Los hechos de entrada siguen siendo manuales e independientes de la pregunta; no hay extractor ni campos de relevancia preasignada. Para cada pregunta se proyectan los hechos que aportarían directamente el campo solicitado. Solo se publican como `record_asserts` los hechos ciertos o comunes a todas las lecturas cerradas, con todos sus calificadores y testigos de entrada. La proyección no transforma una cantidad de componente en total ni una anotación en verdad del mundo.

Una lista abierta no permite afirmar un hecho concreto desde una única lectura enumerada. Un vínculo explícitamente posible puede conservarse como metainformación, con cobertura incompleta y revisión obligatoria. B/none y C/none sin campo solicitado permanecen sin selección y con revisión: no se fabrica A a partir de cualquier pertinencia posible. La invariancia es proposicional sobre la afirmación completa; no identifica qué segunda cláusula aportó un duplicado.

Dos críticos de la misma familia de modelo revisaron concepto/fuentes y código. El crítico de etiquetas no recibió gold ni implementación/resultados. No es revisión humana ni validación externa. Antes de ejecutar se corrigieron dos defectos de procedencia del borrador, se explicitó el cierre de las listas y se fijaron las 18 etiquetas/calificaciones. El crítico de código detectó prioridad incorrecta de una B condicional en listas abiertas y falta de verificación de calificación; ambos tienen pruebas de regresión. Un auditor adicional comprueba los valores y calificadores publicados contra hechos ciertos o todas las lecturas cerradas.

La primera ejecución fue **rechazada antes de puntuar**: dos citas usaban «el identificador» y el texto decía «su identificador». Autor y críticos habían omitido esa diferencia literal. Se conservan banco/preflight rechazados y diagnóstico, se corrigieron solo esas citas y se volvió a congelar el banco; hechos, preguntas y etiquetas semánticas no cambiaron. El rechazo no se convirtió en acierto, abstención ni fallo semántico. Es un límite concreto de la revisión semántica y la razón para mantener validación literal separada.

## Resultados de desarrollo manual

| Medición | Resultado | Interpretación |
| --- | ---: | --- |
| Comportamientos previstos | 18/18 | Concordancia con límites anotados antes de ejecución; no exactitud de un extractor |
| Pares seleccionados | 10/18 | Seis con hechos invariantes, cuatro solo con metainformación |
| Pares con hechos invariantes | 6 | Cinco con número registrado, uno con ausencia acotada al registro de mantenimiento |
| Hechos publicados con testigo comprobado | 6/6 | Ningún hecho variable publicado como invariante en este banco |
| Solo incertidumbre documental seleccionada | 4 | No se recuperó un teléfono concreto en esos cuatro casos |
| Incertidumbre documental junto con dato cierto | 1 | Dato cierto preservado sin transferir el otro número |
| Revisión obligatoria | 5 | Incluye una metainformación seleccionada con cobertura abierta |
| Revisiones sin selección | 4 | B/none, C/none y dos controles abiertos |
| Ajenos cerrados sin selección | 4 | Otro sujeto, atributo, contexto y requisito distinto del teléfono |
| Pares con proyección solicitada variable | 6 | No se afirma que su identidad esté resuelta |
| Comparaciones de nombres/orden | 54/54 estables | Conservan también las revisiones y las salidas negativas |
| Pruebas workspace | 344 con/sin HTTP2 | Quince unitarias nuevas; pruebas remotas manuales ignoradas |

`passes_boundary_behavior=true`, **`passes_autonomous_quality=false`**. No se mide latencia ni ahorro del recorrido automático. El replay reproduce idénticos en bytes los 100 pares V1 (98/100) y los 32 V4 (32 comportamientos previstos, autonomía falsa), sin recalcular etiquetas o métricas. El scorer rechaza una revisión adulterada que retira calificación aunque se actualicen sus hashes. Las listas de datos ciertos e interpretaciones permanecen en las salidas para inspección; una cita literal no prueba fidelidad semántica.

## Coste y siguiente puerta

Esta etapa: **0 llamadas remotas, USD 0 de proveedor nuevo**. Acumulado del objetivo: **5881 operaciones lógicas, 23 campañas completas, USD 0,582095598 conocidos y cinco operaciones de coste desconocido**. No incluye desarrollo ni revisión de Codex; no equivale a coste total cero.

Falta comprobar suficiencia sobre el desarrollo conocido completo y representaciones de contexto/método/alcance más amplias, antes de construir una extracción automática independiente de la pregunta. El próximo avance debe medir fidelidad de esa extracción y selección final sobre fuentes, no solo sobre hechos suministrados. Solo un mecanismo completo que preserve apoyo, refutación y contexto parcial, y pase recursos/estabilidad, podrá abrir V6 con revisión ciega. Esta ampliación explícita de A no rehabilita resultados V4 ni transforma selección de incertidumbre en respuesta concreta. Baseline, dashboard y V6 permanecen sin cambios de política o llamadas.

Artefactos: `tests/fixtures/epistemic-binding-v5/` y `outputs/iromem/evaluation/epistemic-binding-v5-2026-10-20/`, incluyendo borrador, críticas, adjudicaciones, rechazo literal, preflight, salida, score y verificación. CLI: `audit-epistemic-binding-v5 <entrada> <salida-nueva>` y `score-epistemic-binding-v5 <banco> <resultado> <score-nuevo>`.
