# Referencia a contexto fuente: revisión estática del adaptador

Estado: inspección estática del adaptador nuevo `tools/evaluation/src/documentary_v3_io/source_context_reference.rs`, su declaración en `documentary_v3_io.rs`, la rama nueva de `main.rs` y el [contrato corregido](documentary-v3-source-context-reference-contract-2026-10-22.md). Se comprobaron también los helpers directamente invocados de presupuesto, parseo estricto, validación de lado, forma cerrada y escritura para establecer sus fronteras. No se ejecutaron Cargo, fmt, builds, API, CLI o servicios. No se usaron resultados de pruebas, gold o campañas para el dictamen.

## Dictamen

No encuentro un defecto concreto que bloquee la implementación dentro del alcance contractual revisado. Esto permite pasar a la verificación ejecutada por el coordinador; no acredita todavía que compile ni que los controles positivos, negativos y de agotamiento pasen.

## Evidencia de los mecanismos

**Origen y contenido separados.** `manifest_valid` exige tablas cerradas y acotadas, IDs únicos y origen existente por entrega. `bind` (líneas 168–186) calcula por separado el digest fuente y el digest de la entrega y los contrasta con las filas encontradas por `origin_id` y `delivery_id`. Ninguna ruta asigna o fusiona el origen a partir del hash. Dos filas con texto idéntico conservan IDs distintos bajo la autoridad externa del runner, como permite el contrato. Esto no autentica una emisión histórica ni detecta una autoridad que sustituya coherentemente manifiesto e inputs; ambas propiedades quedan fuera del alcance.

**Bind y consume comprobables.** Bind valida fuente y pregunta completas antes de clasificar (188–199), busca un solo testigo del tipo y selección indicados (200–212) y rechaza ausencia o multiplicidad. Consume obtiene otra vez la referencia esperada y ejecuta `require_same` sobre el `Value` completo (259–263). La igualdad recorre objetos y arrays, compara claves y valores y cobra el recorrido; no se reduce a hashes o flags afirmados por el caller. Los campos ajenos de una referencia recibida también la vuelven desigual. Reejecutar una entrega lógica conserva su ID sin prometer frescura.

**Documento, redundancias y deuda.** Las líneas 215–228 comparan los dos documentos con sus raw originales, Claim/request con el owner seleccionado y los arrays de referencias, ámbitos, qualifiers y dependencias con su lado correspondiente. Las líneas 230–235 exigen arrays de respuesta y bindings vacíos y resolución posesiva no probada. La referencia publicada contiene el testigo real, el source input íntegro y el origen; consume exige su igualdad total. La operación no recibe un resolver de κ ni transforma el owner de la pregunta en un binding fuente. La corrección semántica del extractor sigue siendo una cuestión distinta.

**Presupuesto y lectura.** Run lee input y manifiesto con la segunda cota reducida por el tamaño real del primero (276–280), usando un byte centinela para detectar exceso sin cargar el archivo entero. Usa el parser estricto que conserva la detección de duplicados en profundidad. La preinspección propia acumula ambos árboles en el mismo `Counts`, limita profundidad, nodos, owners y evidencia y recurre sin construir una pila proporcional a un array del caller. Los árboles parseados quedan previamente acotados por la lectura conjunta.

Run abre la operación de presupuesto y execute la anida (245–291). El helper `context_budget::with_operation` conserva el contador exterior y propaga agotamiento; la reclasificación ocurre dentro de esa misma vida útil. La activación forzada solo sirve al presupuesto del adaptador cuando falta el perfil: la validación y clasificación reciben el vocabulario original. No se incorpora un perfil semántico al input.

`Meter` carga bytes antes de procesarlos para medida/hash; `reserve` reserva las copias antes de las materializaciones explícitas del adaptador. Se reservan pair, informes serializados, fuente y testigo; la comparación completa cobra nodos y cadenas. Run mide y reserva la salida pretty con su salto final antes de invocar el escritor existente. El agotamiento impide llegar a esa escritura. Estas son unidades conservadoras del evaluador, sin promesa de medir consumo real de CPU o RAM.

**Superficie y rutas históricas.** La integración observada añade una declaración de módulo y una rama de comando con aridad cerrada. El escritor compartido exige archivo nuevo. No se observa una alteración del matcher, la validación histórica ni las ramas anteriores por ese registro. La conservación byte a byte de replays históricos requiere ejecución posterior; esta inspección no la sustituye.

## Límite de la conclusión

La revisión acredita correspondencia estática entre los mecanismos visibles y el contrato local. No atribuye éxito a pruebas aún no ejecutadas, no mide recursos o estabilidad y no extiende el resultado a un consumidor que descarte el wrapper. No solicita cambios de arquitectura, autenticación, estado persistente o producto: no hay un fallo concreto que los justifique en este alcance.
