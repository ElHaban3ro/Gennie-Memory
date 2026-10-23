# Revisión estática del evento nominal contextual

Fecha: 2026-10-22. Revisión adversarial acotada del código Rust de evaluación y de los contratos de implementación del evento nominal y `source-context-reference-v1`. Se leyeron las reglas backend, módulos y pruebas. No se modificó código, no se ejecutó Cargo, formato, CLI ni API y no se consultaron autorías, credencial real, revisiones semánticas o gold. Las observaciones siguientes son inferencias estáticas; requieren controles ejecutados antes de atribuir un fallo o su corrección al binario.

## Hallazgos

### P2 — Un PP puede absorber las evidencias del core

Ubicación: `tools/evaluation/src/documentary_v3/nominal_event_context.rs:254–287` y `:355–365`.

La validación comprueba que los PP no se solapen entre sí, que sus spans sean exactos, que estén dentro del Body y que no invadan la mención del paciente. No comprueba que un PP sea disjunto de `core.construction_evidence` y `core.action_evidence`. El contrato exige el orden se→verbo→paciente y permite PP anteriores o posteriores a esos operandos; absorber el core dentro del complemento nombrado no cumple esa partición.

Contraejemplo estático preciso sobre el fixture sintético del módulo: en `En Orbe se eliminó la restricción de acceso para Paz.`, cambiar `place.mention` y `place.name`, y el `complement_evidence` del primer PP, de `Orbe` `[3,7)` a `Orbe se eliminó` `[3,19)`; cambiar su `phrase_evidence` a `En Orbe se eliminó` `[0,19)`. Todos son rangos UTF-8 exactos. El primer PP sigue antes del segundo y no toca al paciente `[23,45)`, pero invade `se` `[8,10)` y `eliminó` `[11,19)`. El segundo PP/Paz permanece intacto, por lo que la relevancia hacia Paz no depende del nombre alterado. Por lectura, ni el validador nominal ni el selector añaden el rechazo que falta.

Impacto: una partición estructuralmente incoherente puede presentarse como documento completo y conservarse en un testigo parcial V2 ligado a origen; la igualdad del transporte preserva esa partición, no la corrige. No se infiere de ello una respuesta completa ni una resolución de adjunción.

Corrección propuesta: exigir no solape de cada `phrase_evidence` con ambas evidencias del core, manteniendo la comprobación existente del paciente. Ejecutar luego el contraejemplo completo y conservar los positivos PP anterior/posterior. No se ejecutó ese control durante esta revisión.

### P2 — La entrada directa del clasificador copia ambos documentos antes del presupuesto

Ubicación: `tools/evaluation/src/documentary_v3/pair.rs:5620–5624`.

Cuando `classify_pair_context` se invoca sin una operación activa, construye `json!({"source": source.report["document"], "question": question.report["document"]})` antes de llamar a `with_operation`. La macro materializa copias completas de los dos documentos antes del preflight y de cualquier reserva de sus bytes. La cota del presupuesto puede rechazar después, pero ya no precede a esa asignación.

Alcance: defecto heredado de la ruta composicional anterior que también usa feature12; afecta al punto de entrada Rust público sin operación externa. En `source-context-reference-v2`, el transporte activa primero su operación, así que esta rama concreta no demuestra un bypass de la CLI. El contrato nuevo exige precobrar copias y la API no debe atribuirse esa garantía hasta corregir esta entrada.

Corrección propuesta: activar primero la operación sin construir un árbol combinado y hacer preflight/reserva acumulados sobre las raíces prestadas antes de cualquier copia. Verificar la entrada directa además de la CLI. No se ejecutó un control de asignación en esta revisión.

## Comprobaciones estáticas sin bloqueo adicional identificado

- `linguistic_composition::blocked_nodes` siembra el Claim; el constructor nuevo añade H y el cierre nominal T. Los complementos PP nombrados no se siembran indiscriminadamente. El filtro del par aplica ese cierre antes de legacy y los guards usan la vista ejecutable fuente.
- El permiso composicional es OR de features11/12; `request_context_profile` conserva permisos separados y feature12 exige `nominal_complement_de` y `remove`. La ruta histórica de matching excluye el discriminador nuevo.
- El selector exige `truth/status`, acción `remove`, nominal compuesto compatible y ancla Same con las comprobaciones históricas de namespace/restricciones; conserva las cuatro deudas y los arrays de respuesta, bindings y estados vacíos. Evalúa cobertura/dependencias y contexto de ambos lados. La completitud ordinaria se calcula antes de adjuntar contexto parcial.
- V2 obtiene el testigo por reclasificación real, comprueba documentos/Claim/request/arrays redundantes y esquema cerrado del testigo nuevo, conserva flags false y deudas mínimas y recalcula la referencia completa al consumirla. V1 mantiene su filtro de testigo histórico y sus etiquetas de versión. Esto es lectura del flujo, no reproducción byte a byte de salidas V1.
- La canonicalización nominal normaliza los IDs internos explícitos y conserva operandos externos, ambos PP y orden. La separación de espacios de IDs se mantiene por los paths y términos tipados.

## Verificación pendiente y límite de esta revisión

No se acredita compilación, ejecución, replays históricos ni fidelidad semántica autónoma. Se observaron pruebas nuevas en fuente, pero no se ejecutaron. La revisión del precobro de la pila en `context_budget::preflight` estaba siendo afinada por otro agente y queda fuera de cualquier dictamen de corrección aquí: esta lectura vio `stack.extend` de arrays y `stack.push` de objetos antes de reservar la capacidad; debe revisarse de nuevo sobre el archivo final. Los hallazgos de esta nota tampoco deben darse por corregidos por un cambio concurrente sin relectura y prueba focal.

## Addendum: relectura estática de las correcciones

Se conservan arriba los dos hallazgos originales y el estado de su primera revisión. Esta segunda lectura se hizo sobre los archivos corregidos el 2026-10-22, sin ejecutar Cargo, CLI o API y sin consultar paquetes de autoría/crítica fuente.

1. **Solape PP/core: mecanismo corregido por lectura.** `nominal_event_context.rs:274–277` añade la prueba de intersección de intervalos de cada `phrase_evidence` con `construction_evidence` y `action_evidence`, antes de aceptar el PP. Usa la misma semántica de intervalos semiabiertos que el corte del paciente; permite tocar un límite y rechaza una intersección no vacía. El contraejemplo original `[0,19)` ahora dispara ambos rechazos. Se observó además la prueba focal `nominal_event_context_prepositional_phrase_cannot_absorb_se_or_action` en el archivo de pruebas; su presencia no sustituye su ejecución. El coordinador comunicó que había reproducido el defecto anterior por CLI y preservado el informe; esa ejecución pertenece al expediente del coordinador y no se presenta como ejecutada por este revisor.

2. **Copia previa de los documentos: mecanismo corregido por lectura.** `pair.rs:5620–5623` pasa referencias prestadas a `with_pair_operation`, eliminando el `json!` combinado anterior. `context_budget.rs:209–227` activa el presupuesto mediante `with_preflight` y recorre las dos raíces prestadas con contadores acumulados, profundidad equivalente al envoltorio y límites conjuntos de nodos/owners/evidencias, dimensiones y tamaño serializado. El fallback de límite conserva una salida pequeña con el motivo, sin volver a copiar los dos documentos después del agotamiento. No se observó una nueva copia equivalente anterior al preflight en esta entrada.

3. **Pila del preflight: precobro y cota presentes.** `context_budget.rs:104–107` cobra la pila inicial de hasta dos punteros antes de `to_vec`; `:119–127` y `:130–168` comprueban la suma de pendientes y nuevos hijos contra los nodos restantes y cobran unidades conservadoras de 64 bytes por hijo antes de `extend`/`push`. La pila almacena referencias, no árboles `Value` copiados. El límite se comprueba antes de crecer a partir de un array u objeto ancho. Esto cierra estáticamente la observación pendiente de la primera revisión; no es una medición de memoria real del asignador.

Dictamen de esta relectura acotada: no queda un P1/P2 abierto en los tres mecanismos corregidos aquí inspeccionados. La confirmación ejecutada del rechazo nuevo, los positivos preservados, la suite y los replays sigue correspondiendo al coordinador. Esta conclusión no modifica el límite de fidelidad semántica ni demuestra extracción autónoma.
