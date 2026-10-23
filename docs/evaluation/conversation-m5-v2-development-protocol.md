# M5: protocolo del segundo banco sintético de desarrollo

Estado: fijado antes de medir el nuevo banco, 2026-10-17. Esta partición sirve para encontrar y descartar errores; no es calibración, reserva, prueba humana ni cambio de la ruta de producto. Se mantienen intactos los resultados anteriores y las cohortes 018–020.

## Construcción y revisión

Cinco conversaciones ficticias nuevas, separadas por `scope_id`, con treinta recuerdos y veinte preguntas por conversación: quince positivas y cinco negativas. Los autores automáticos redactan hechos y preguntas con formulaciones variadas; un revisor adversarial independiente del autor intenta encontrar otras respuestas admisibles, presuposiciones y negativos que sí tengan respuesta. La procedencia de las etiquetas es `agent_synthetic_unreviewed`. No se describe como revisión humana.

El clasificador vigente devuelve cero o un ID y el evaluador de campaña compara la selección con `expected_ids` por igualdad exacta. Por ello, el conjunto puntuado solo admite preguntas positivas con un único ID defendible. Una pregunta para la que haya varias respuestas razonables se registra como ambigua y se excluye antes de cualquier ejecución; no se fuerza una de ellas como verdad. Los negativos deben tener un distractor verosímil, pero ningún recuerdo aplicable. `forbidden_ids` se reserva para un ID que jamás debería recuperarse por ámbito, estado o revisión; un distractor visible del mismo ámbito no es prohibido.

El autor fija los archivos y sus SHA-256 en el manifiesto; el revisor adversarial trabaja sobre esa versión sin resultados del sistema. Si la crítica exige corregir un caso, se genera una nueva versión de los archivos y hashes antes de buscar. No se cambian etiquetas después de ver los resultados para elevar la puntuación.

## Mediciones predefinidas

Primero se valida estructura, referencias y hashes. En una SQLite temporal nueva se carga el corpus una vez y se ejecuta `search-benchmark` con `keywords` y, como diagnóstico separado, `natural`, sobre los mismos cien casos. Los resultados se guardan con hashes de casos, binario y base. Se informan por conversación y categoría: positivos con ID esperado en top 1, top 5 y top 20; casos sin cobertura; candidatos en negativos; cruces de ámbito/estado/revisión; y tiempos de búsqueda p50/p95/p99. Que un negativo tenga candidatos no implica una respuesta falsa: eso requiere clasificador y semilla.

Los fallos de cobertura se investigan como búsqueda; las abstenciones o elecciones incorrectas con el ID presente se investigan como selección. Una variante nueva se define y registra antes de probarla con proveedor, conserva pregunta, corpus y candidatos comparables, y se enfrenta a la ruta vigente caso a caso. No se usa el ID esperado para ordenar, podar o redactar la entrada en inferencia. Se descarta si crea un falso recuerdo nuevo, depende de un orden artificial, no mejora por conversación o eleva desproporcionadamente latencia y coste. Una comparación de proveedor separada no se presenta como mejora de búsqueda.

Solo después de revisar el banco y la cobertura local se define un piloto remoto acotado con número máximo de llamadas, duración prevista y coste informado o reservado. El resultado remoto es de desarrollo; calibración y reserva permanecen cerradas. El cierre de M5 requiere además una muestra de preguntas y etiquetas revisadas por personas independientes o feedback voluntario real, sin pedir al usuario una clasificación exhaustiva de este banco.
