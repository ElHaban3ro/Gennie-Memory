# Diagnóstico de scores y auditoría de fuentes

Estado: objetivo activo, sin promoción. Desarrollo sintético conocido, no validación nueva. La política optativa `per_memory_scored_v1` conserva el payload y la elección de `per_memory_binary_v5`: instrumenta scores y hash de petición sin aplicar umbrales ni cambiar la selección predeterminada.

## Resultado medido

64 operaciones/64 HTTP completadas, cero degradaciones, recibos completos: **USD0,003622542**. Mediana370,73ms y p95523,95ms del runner; no comparación emparejada, latencia poblacional ni tiempo de respuesta final de Codex. Modelo solicitado `typesafe/jev-1.13`, servido `typesafe/jev-1.13-20260917`.

512 juicios agrupados en64preguntas:69 positivos y443 negativos según las etiquetas originales. No son512observaciones independientes. Cero discrepancias entre elección y máximo score. La elección conserva dos IDs no permitidos por ese gold y omite14IDs anotados útiles dentro del prefijo. Otro ID anotado útil queda en posición13, fuera del prefijo8. Estos conteos describen el gold original y no resuelven sus discrepancias semánticas.

Mínimo scoreA positivo0,01; máximo negativo1,00; separación total−0,99 frente al margen previo0,05. **Se rechaza la hipótesis del umbral global**; no se elige uno ajustado por caso. El carné por sí solo no responde qué avalista llevar, pero obtiene1,00. Incluso comparándolo con el control útil inequívoco de horario con0,99, no existe la banda requerida. Esta conclusión no depende de aceptar como definitivas las etiquetas de contexto discutidas.

[Protocolo previo](per-memory-score-diagnostic-protocol-2026-10-19.md), [reporte original](../../outputs/iromem/evaluation/per-memory-score-diagnostic-2026-10-19/report.json), [tiempos del runner](../../outputs/iromem/evaluation/per-memory-score-diagnostic-2026-10-19/campaign-report.json).

## Auditoría retrospectiva sin etiquetas ni scores

Un subagente separado recibió las conversaciones completas y64preguntas, sin gold ni resultados. Antes había revisado cuatro preguntas fuente y recibido una objeción al criterio; por eso no se presenta como revisor completamente nuevo. Su primera inclusión del carné se corrigió tras esa objeción y ambas versiones se conservan. Misma familia de modelos, sin revisión humana ni validación nueva.

La reconciliación detecta **10preguntas con discrepancias de IDs** y **dos discrepancias de relación en IDs compartidos**. El revisor propone60vínculos útiles y16preguntas sin evidencia. Las diez diferencias de IDs retiran contexto relacionado del conjunto útil; no son diez errores demostrados del autor. Por ejemplo, recibir un mapa no prueba aprobación y preparar una muestra no aporta el grado de una prueba. Otras diferencias afectan información de preparación o planificación cuya utilidad exige fijar una rúbrica explícita.

El auditor Rust comprueba cobertura64/64, identidad/ámbito de cada ID, separación entre contexto y evidencia, roles, razones y hashes del banco anterior a la revisión. Estas comprobaciones acreditan integridad estructural, no verdad semántica. Las condiciones de ceguera son procedencia declarada. La v1 de reconciliación omitía diferencias de relación; se preserva junto con la v2 corregida. **No se alteraron etiquetas, se recalcularon aciertos ni se reescribieron resultados.**

[Reconciliación completa](../../outputs/iromem/evaluation/score-source-audit-2026-10-19/reconciliation-v2.json).

## Próxima puerta

Antes de otro ajuste, separar prospectivamente evidencia que responde, refuta explícitamente o comunica un estado parcial solicitado, de contexto relacionado que solo describe preparación o trámites. Conservar también ese contexto como categoría propia: no eliminarlo para aparentar precisión. Resolver por fuente las discrepancias, versionar cualquier banco corregido y mantener sus resultados separados del original; nunca usar esa reconciliación como ganancia retrospectiva. Después congelar una única variante y contrastarla en desarrollo antes de abrir V6, que sigue con cero llamadas.

95pruebas workspace pasan y `cargo fmt --all --check` pasa. Revisión estática separada corrigió contabilidad de recibos y reporte de relaciones. Base temporal propia eliminada, sin servidores iniciados ni cambios en el dashboard.

Acumulado del objetivo: **2545operaciones,17campañas emparejadas, USD0,179332972 conocidos y dos operaciones de coste desconocido**. Excluye Codex para programación/revisión; el total completo no se conoce. Ninguna candidata supera todavía todas las puertas. No se cambia README ni configuración predeterminada.
