# Conservación del estado documental en V3

El diagnóstico anterior omitía notas explícitas como «no consta en el registro» o «ilegible en la copia». Se implementó una vía de contexto documental que conserva la proposición completa, incluido el documento al que se refiere, sin publicar un valor solicitado. La evidencia permanece parcial y calificada: no prueba inexistencia global, no equivale a cero y no completa una respuesta.

La vía necesita una relación de relevancia explícita del vocabulario entre `documentary_availability`/`information_state` y una petición `property.value`/`quantity.amount`. Debe ligar sujeto y atributo y declarar `source.state` como contenido transportado. Sin declaración, no hay inferencia nueva. Se añadieron cuatro relaciones genéricas al vocabulario experimental, manteniendo los mismos 31 predicados, 30 acciones y 77 atributos. Los frames originales permanecen intactos; los de la comparación solo cambian su hash de vocabulario.

Se exige mismo atributo y referente acreditado. Si falta clase pero hay nombre documental común, se conserva la advertencia de identidad; la semejanza de referencias anónimas o su coocurrencia no basta. Contexto desconocido, condiciones de rama, dependencias, premisas o fundamento incompatibles conservan candidatos pendientes sin selección. El witness usa `partial_relevance`, `answer_terms:[]` y la proposición original en `answer_propositions`; no transforma el estado documental en el valor preguntado.

El protocolo se congeló antes de comparar los mismos100pares manuales, con las referencias históricas V5 sin readjudicar. Tres variantes separan configuración e implementación:

| Variante | Útiles seleccionadas | Referencias `none` seleccionadas | Respuestas completas | Concordancia /100 |
| --- | ---: | ---: | ---: | ---: |
| Código anterior, vocabulario original | 21 | 1 | 3 | 80 |
| Código anterior, cuatro relaciones declaradas | 22 | 1 | 3 | 81 |
| Código nuevo, esas mismas relaciones | 23 | 1 | 3 | 82 |

Se recuperan `cv3q_01/cv3_01_01` (teléfono no documentado) y `cv3q_12/cv3_12_00` (cantidad ilegible). La primera selección se recupera al declarar relevancia; la segunda requiere además el código nuevo bajo idéntica configuración. Ambas variantes conservan las21selecciones útiles previas, las tres respuestas completas y ninguna selección nueva en `none`. El código nuevo conserva los estados como proposiciones documentales, sin términos de respuesta fabricados.

No se interpreta82/100como exactitud del producto: es concordancia diagnóstica sobre frames manuales y etiquetas heredadas. Siguen16referencias útiles no ambiguas sin seleccionar (9A,4B,3C), una ambigua separada y la discrepancia de alcance universal anterior. La disponibilidad de las imágenes de Delta sigue pendiente: sus anclajes anónimos y vínculos no prueban identidad. No se fuerza ese match para mejorar cifras.

Se conservan el fallo inicial de implementación implícita —violaba el control que exige declaración de relevancia— y un fallo por depender del orden de los bindings. La corrección respeta la declaración y compara esos bindings sin depender de su orden. Los controles también cubren otro nombre, otro atributo, preservación del documento, rama pendiente y fundamento incompatible. No se cambian expectativas históricas para aprobar.

**597 pruebas pasan, 24 omitidas; 77 V3**, dos controles nuevos. Compilación y formato pasan. El hash final reproduce idénticos100informes con vocabulario original,100de la variante nueva y116validaciones de extracción congelada.

La extracción automática anterior sigue en65/116entradas fieles para ambos críticos. No se repite ni repara, no se mide baseline o latencia/coste de producto y no se abre V6. El goal sigue activo en paso3: conservar toda la evidencia útil y resolver alcance antes de extracción congelada, comparación real, recursos/estabilidad y casos nuevos. Cero llamadas de API y USD0nuevos; coste/tokens Codex desconocidos. No se reactivaron subagentes.

[Protocolo y hashes](../../outputs/iromem/evaluation/documentary-v3-documentary-state-2026-10-21/protocol.json), [comparación completa](../../outputs/iromem/evaluation/documentary-v3-documentary-state-2026-10-21/report.json) y [vocabulario experimental](../../outputs/iromem/evaluation/documentary-v3-documentary-state-2026-10-21/vocabulary.json). Las pruebas y reproducciones del hash final se registran aparte en `execution-provenance.json`.
