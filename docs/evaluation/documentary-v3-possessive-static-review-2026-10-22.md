# Revisión estática del posesivo nominal

Fecha: 2026-10-22. Alcance: lectura de implementación y pruebas locales del operador, hooks en validación, pair, presupuesto, perfil, lecturas y proyección existencial. No se ejecutaron Rust, Cargo, APIs ni campañas. No se abrieron benchmarks, nuevas autorías fuente ni revisiones de esas fuentes. Los hallazgos siguientes son de lectura y seguimiento de control; no son reproducciones de ejecución.

## Hallazgos

### P2 — La reserva fallida no detiene las copias de la vista ejecutable

En `tools/evaluation/src/documentary_v3/pair.rs:5621-5627`, el bloque nuevo calcula una reserva de tres veces el documento fuente, llama `context_budget::charge(1, bytes)` e ignora el booleano. Después ejecuta `sd.clone()` y construye más copias de claims. Si el cálculo excede el presupuesto restante, el contador queda agotado, pero las asignaciones siguen ocurriendo. El wrapper acaba convirtiendo el resultado en `request_context_limit`; eso evita un parcial exitoso, pero no satisface el requisito distinto de detenerse antes de materializar sin reserva.

Corrección: retornar inmediatamente al fallar la reserva o detectar agotamiento antes de cualquier clone posterior. El error debe salir por la operación compartida, sin una vista truncada o éxito parcial. Control sugerido: presupuesto restante inferior al tamaño reservado y evidencia de que no se alcanza la materialización, además del resultado técnico global.

### P2 — La atribución de issues confunde el documento fuente íntegro con la pregunta

`nominal_possessive_matching.rs:151` pasa `sd = &self.source_report["document"]` a `dependency_missing`. El helper en `pair.rs:899-903` elige el informe fuente solo si `doc` es el mismo objeto que `self.source`; este último es `executable_source`, un clone distinto. Por tanto esta llamada usa `question_report` para indexar issues de la fuente. `guard_satisfied` en `pair.rs:695-700` usa el mismo selector y puede recibir ese documento íntegro mediante contexto/dependencias.

El recorrido sigue usando las filas de `doc`, pero su mapa de incidencias/candidatos pertenece al otro lado. Puede omitir una incidencia de fuente o atribuirle una de pregunta si los IDs coinciden. Los controles de `profile_pending` y candidates posteriores reducen algunos efectos; no reparan la selección equivocada del informe. No se afirma aquí una respuesta falsa reproducida.

Corrección: declarar el lado o reconocer explícitamente tanto la vista fuente como el documento fuente original, manteniendo la cuarentena de claims al comprobar guards. Control sugerido: misma estructura con IDs fuente/pregunta iguales, una incidencia relevante solo en fuente y otra independiente solo en pregunta, verificando atribución correcta y conservación de hechos independientes. Un guard válido debe comprobarse contra la vista autorizada, nunca recuperar un claim puesto en cuarentena mediante el original.

### P2 — Se mezclan correlaciones de distintas filas de relevancia

En `nominal_possessive_matching.rs:196-211`, `connection` se reemplaza por la última fila cuyo par de predicados coincide. Después, en las líneas 227-238, cada rol busca su binding con `flat_map` sobre **todas** esas filas. La fila que acredita cobertura completa de roles no queda ligada a los bindings consumidos. El resultado depende del orden y puede construir una correlación que ninguna fila declara.

Contraejemplo simbólico de control de flujo: fuente `S(actor=A,condition=Hposesivo)` y pregunta `T(x=A,y=B,z=Hcompatible)`. L1 solo declara `source.actor→target.x`; L2 declara `source.actor→target.y` y `source.condition→target.z`. Los dos enlaces pueden ser estructuralmente válidos bajo la validación genérica: no se exige que cada enlace enlace todos los roles. L2 es la última fila y hace `connection=true`. El mapping posterior toma actor→x de L1 y condition→z de L2; ambos comparan, pese a que L1 está incompleta y L2 sola falla actor A frente a B. La prueba externa nominal y el carrier pueden cumplirse. Se obtiene contexto parcial mediante una correlación inventada, sin necesidad de crear una respuesta completa o resolver κ.

Corrección: evaluar cada fila completa como una alternativa atómica, con sus bindings y contexto; no componer filas. Conservar al menos una fila íntegra que pase y su procedencia. Control sugerido: ese contraejemplo y su permutación deben conservar el mismo rechazo; una fila única completa correcta debe conservar el parcial positivo.

## Propiedades que la lectura sí acredita

- La denotación `lexical_use` se admite solo con feature; el target exige forma léxica, scope, incidencia y campos cerrados. Su uso como término físico o identidad se rechaza en la validación específica.
- `identity_depth` corta la comparación cuando cualquiera de las referencias pertenece a la cuarentena posesiva. El matcher no asigna valor a κ y el resultado derivado declara `possessive_referent_proven:false`.
- La cuarentena recorre enlaces, dependencias declaradas, operandos implícitos, aliases y referencias de claims/premisas. Los claims bloqueados se retiran de la vista fuente para rutas ordinarias. Un claim independiente puede permanecer ejecutable.
- El parcial fija `answer_terms`, `answer_propositions` y `bindings` vacíos y copia ambos documentos originales íntegros, además de constraints y arrays de contexto. Así transporta claims, requests, identidad, namespace y dependencias de pregunta, sin convertirlos en respuesta.
- La lectura de referencia canónica corta referencias bloqueadas; la proyección existencial rechaza `lexical_use` o un descriptor cuyo cierre esté bloqueado, con `not_supported`. No se observó una creación explícita de existencia física o Same para el slot.
- Los nuevos recorridos de componentes y cuarentena retornan antes de caminar referencias cuando no está activo el feature. El perfil cerrado admite ocho features conocidos.
- El presupuesto compartido invalida la operación al agotarse; las principales copias del parcial y candidato tienen reserva previa. La excepción concreta de la vista ejecutable está señalada arriba.

Estas comprobaciones de lectura no prueban cobertura exhaustiva, comportamiento real ni fidelidad de extracción.

## Pruebas inspeccionadas y límites

La suite local contiene controles de carrier completo, shape/scope/incidencia y uso semántico indebido, feature ausente, cierre de alias/guard/ámbito normativo, actor distinto, presupuesto acumulado, dos posesivos sin respuesta completa y hecho independiente. Se leyó su código sin ejecutarlo.

El control `possessive_implicit_normative_guard_and_alias_closures_are_cut` llama al grafo con un JSON armado y no valida documentos completos ni ejecuta el matcher. Acredita únicamente la intención y aserciones sobre el cierre; no debe presentarse como prueba de rechazo semántico de un guard válido. El control positivo sí exige documentos completos antes de comparar y comprueba igualdad de ambos documentos transportados. Faltan en lo inspeccionado controles explícitos de las tres incidencias descritas arriba.

## Dictamen

Diseño semántico previamente aceptado, implementación con tres hallazgos estáticos que deben resolverse y verificarse antes del cierre del mecanismo. No se modifica código desde esta revisión. La aprobación de diseño, tests del coordinador y este análisis no equivalen a fidelidad fuente-only, puerta 116/116 ni adopción.

## Observación adicional de presupuesto durante la corrección

`nominal_possessive::has` usa `out.iter().any(...)` sin cobrar cada entrada visitada. `put` llama a ese helper antes de cada inserción; los recorridos de edges/component también lo reutilizan. El coste real de N inserciones puede crecer cuadráticamente mientras se cobran las inserciones y solo una parte de las visitas. Se comunicó al implementador antes del cierre. La corrección puede usar búsqueda por clave del BTreeSet con reserva de temporales, o cobrar cada visita de la búsqueda lineal; el objetivo es que el límite de pasos mida esos recorridos efectivos. Es una lectura estática de complejidad, no un agotamiento reproducido.

## Revisión posterior de las correcciones

Se releyó el freeze comunicado por el implementador. Los tres hallazgos principales y la observación adicional de presupuesto quedan **cerrados por inspección estática**:

1. La reserva de la vista ejecutable retorna inmediatamente si falla, antes de `sd.clone()` y copias de claims.
2. `Engine::is_source_document` reconoce tanto la vista ejecutable como `source_report.document`; los dos selectores usan ese helper. Los guards de fuente recorren claims de la vista ejecutable y excluyen adicionalmente los nodos posesivos bloqueados del lado correcto. No vuelven a ejecutar un claim en cuarentena leyendo el original.
3. `possessive_arguments_match` recibe una única fila de relevancia y exige un binding único de esa misma fila para cada rol fuente. El caller evalúa filas completas como alternativas. Desapareció el `flat_map` transversal que fabricaba correlaciones.
4. `nominal_possessive::has` cobra cada entrada visitada y retorna inmediatamente si el presupuesto falla; ya no oculta la exploración lineal ni sigue recorriendo tras agotarse.

Los tests de alias/guard y actor distinto se reescribieron para exigir validación completa de ambos documentos antes del matching. Este crítico solo leyó su código: el coordinador debe ejecutar y confirmar sus resultados.

Se detectó un defecto de preparación en el nuevo test adversarial de relevancia: su fila `complete_but_wrong_actor` contiene `carry_roles:[]`, pero el validador genérico exige una lista no vacía. Así no puede acreditar el rechazo semántico porque `pair()` exige documentos completos y el vocabulario fallaría antes. Se notificó a implementador y coordinador; basta conservar un carry_role fuente válido sin modificar el contraejemplo. También conviene comprobar permutación de las dos filas y una fila única correcta como positivo. Esto es verificación pendiente del fixture, no reapertura del defecto de correlación ya corregido por lectura.

Dictamen estático posterior: no quedan los cuatro defectos de código señalados. El cierre ejecutado del mecanismo depende aún de corregir ese fixture, compilar y completar las comprobaciones del coordinador. Esta revisión no afirmó haber ejecutado ninguna de ellas y no consultó nuevas representaciones fuente ni sus críticas.

### Cierre del fixture y huellas revisadas

Se verificó por lectura la corrección de `complete_but_wrong_actor.carry_roles` a `["source.condition"]`; deja de existir la carencia estructural concreta señalada arriba. La ejecución focal del coordinador es una verificación separada y no se atribuye a este crítico. No quedan hallazgos estáticos bloqueantes de esta revisión.

SHA-256 de las rutas principales al cerrar esta lectura:

- `nominal_possessive.rs`: `155218ade5d7e53c46d2a94443b80bdf8f5aee61a7ee7887259c6e0807935be3`.
- `nominal_possessive_matching.rs`: `79234f8563fa6c7191dc7e15456d73683a09981599c9aeba771515c5e394708e`.
- `pair.rs`: `c12dba94a4d2af97c62e718cab28b42faa9d6e77de4382d66d3cbd5b201a7b88`.
- `nominal_possessive_tests.rs`: `b65bc4a912a92180307fe444b241192b25cb5c905fc49af92a3acce59b812237`.

Las huellas identifican lo leído, sin afirmar compilación, ejecución, freeze inmutable futuro ni fidelidad automática.
