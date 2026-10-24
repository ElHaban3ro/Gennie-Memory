# Roadmap resumido de Gennie Memory

Estado revisado: **2026-10-23**. Una casilla marcada indica que se completó el alcance descrito; una casilla abierta puede incluir avances parciales. El [roadmap detallado](ROADMAP.md) conserva los hitos, informes e historial experimental. Actualizar este resumen cuando cambie el estado de un punto, manteniendo su evidencia en el documento detallado.

## Completado

- [x] **Núcleo local en Rust:** workspace, SQLite compartido, migraciones y carga de fixtures sintéticos; búsqueda FTS5 por palabras clave y filtros de ámbito, entidad opcional, revisión, estado y tiempo.
- [x] **Recorrido inicial de recuperación:** `recall` encadena búsqueda, revisión local compatible o clasificación remota y composición determinista de una semilla con referencias; `session` permite reutilizar el cliente HTTP.
- [x] **Dashboard local de revisión:** consultar recuerdos, guardar feedback reutilizable y exportarlo de forma optativa desde la UI React/Vite. Su beneficio con usuarios reales sigue sin medirse.
- [x] **Evaluación y gráficas en Rust:** runner de campañas, validación de bancos, métricas y comparaciones reproducibles; migración de las herramientas activas de Python a Rust completada.
- [x] **Cierre experimental de M5:** resultados, costes, latencias y límites documentados. La conclusión fue viabilidad general no demostrada; este cierre no aprueba uso autónomo general.
- [x] **Investigación de calidad de cinco etapas:** contrato, controles, mecanismo, contraste nuevo e informe cerrados. Tras la comparación amplia se restauró **Decisions + baseline**; single-pass queda experimental, sin mejora general demostrada. [Decisión vigente](docs/decisions/0044-restore-baseline-default.md).

## Pendiente

- [ ] **Consolidar el contrato del producto:** completar consulta, resultado, cobertura, degradación y trazas; documentar capacidades y límites por proveedor, sin fabricar justificaciones ni equiparar puntuaciones; separar desarrollo, calibración y prueba final del producto completo.
- [ ] **Completar recuperación y aislamiento:** incorporar filtros por actor y tipo, resolución real de entidades y completar el contrato de FTS5, deduplicación y cobertura; aplicar los filtros antes de limitar candidatos.
- [ ] **Definir los embeddings definitivos:** elegir y medir un codificador independiente del clasificador, registrar artefactos y perfil, fragmentar por tokens sin truncamiento silencioso y publicar índices completos por revisión y espacio semántico; medir plazos y escalado del coseno exacto.
- [ ] **Completar `recall`:** añadir presupuestos de tokens de entrada y semilla, verificación de fuentes, aplicabilidad y relevancia por candidato, puntuaciones calibradas y degradación del híbrido; completar su integración si vuelve a ser una ruta activa.
- [ ] **Ampliar cobertura y pruebas del producto:** cubrir multilingüismo, textos largos, homónimos, contradicciones, fuentes inciertas, duplicados, permisos cruzados, datos sensibles y fallos parciales; completar ablaciones, medición de concurrencia y mantenimiento sostenido.
- [ ] **Validar calidad para uso real:** obtener preguntas naturales y revisión humana independiente, evaluar las afirmaciones de la respuesta final y completar calibración y reserva nuevas antes de aprobar una política de uso general. Los resultados sintéticos actuales no cierran este punto.
- [ ] **Implementar escritura y reconciliación:** completar `mem_write`, `entity_resolve`, huellas, clasificación de relaciones, propuestas, `mem_resolve`, confirmación y auditoría de evidencia.
- [ ] **Completar el ciclo de vida:** cola persistente de embeddings, revisión de vigencia, `mem_review_list`, retirada, restauración, borrado del almacén activo y cancelación de trabajos.
- [ ] **Integrar con Codex:** exponer MCP y, si se decide, un hook; resolver continuidad tras compactación y medir desde el mensaje del usuario hasta la respuesta final, incluyendo recuperación y entrega automática de la semilla.
- [ ] **Preparar operación sostenida:** observabilidad, migración entre perfiles, respaldo, recuperación, instalación, documentación y otras plataformas; incorporar búsqueda vectorial aproximada solo si el coseno exacto incumple un objetivo medido.
