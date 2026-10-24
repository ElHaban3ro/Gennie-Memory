# 3. Integración de recall-v1

`source_policy:report` por defecto conserva el recorrido para fuentes no resueltas con sus estados visibles; `require_verified` exige coincidencia literal de todos los recuerdos que finalmente aparecen en la semilla. Una fuente incluida rechazada bloquea la semilla en ambos modos. Las políticas no son filtros de red: el clasificador puede recibir fuentes no verificadas y producir un recibo antes del bloqueo final. No hay fallback, nueva elección ni selección por scores de procedencia.

Antes de clasificación se comprueba el conjunto de candidatos y se registran sus resultados. Tras componer y antes de publicar/contar la semilla se vuelven a comprobar los incluidos, también los posibles equivalentes presentados al agente. Un cambio de snapshot durante el clasificador invalida el resultado previo; un cambio de memoria/candidato produce fallo de comprobación. El proveedor sigue sin tener autoridad para marcar una fuente verificada.

SourceCheck separa coincidencia literal de semantic_support:not_assessed. Assessments y SeedMemory publican provenance_status; el texto final advierte que original, resumen, condiciones y verdad tienen alcances distintos. Esa advertencia participa del presupuesto de tokens/bytes, al igual que IDs y aviso de equivalencias. Si se bloquea, selection/assessments/recibos se conservan; seed es null y la ejecución degradada con source_verification_blocked o source_check_failed. included_in_seed permanece falso.

Una abstención sin recuerdos incluidos no exige inventar una fuente verificada. Comprobar candidatos no prueba cobertura de pregunta. La ruta recall heredada y el compositor experimental conservan el contrato histórico; estas garantías son de recall-v1/session recall_v1.

Pruebas de producto: report frente a require_verified, semilla verificada con equivalente, bloqueo de rechazo en report, hashes/recibos idempotentes y revalidación tras registrar una versión nueva durante clasificación. No se ejecuta inferencia pagada.

La procedencia no es el estado semántico del recuerdo: un Candidate.state derivado como uncertain por propuestas pendientes puede conservar coincidencia literal. Source no consulta tablas de reconciliación ni certifica que esa incertidumbre esté resuelta; el repositorio/compositor son dueños de mostrarla y de los filtros de ciclo de vida. Una prueba contrasta esa separación sin alterar revisión, original o fuente.
