# Roadmap resumido de Gennie Memory

Estado revisado: **2026-10-24**. Una casilla marcada indica que se completó el alcance descrito; una casilla abierta puede incluir avances parciales. El [roadmap detallado](ROADMAP.md) conserva los hitos, informes e historial experimental. Actualizar este resumen cuando cambie el estado de un punto, manteniendo su evidencia en el documento detallado.

## Completado

- [x] **Núcleo local en Rust:** workspace, SQLite compartido, migraciones y carga de fixtures sintéticos; búsqueda FTS5 por palabras clave y filtros de ámbito, entidad opcional, revisión, estado y tiempo.
- [x] **Recorrido inicial de recuperación:** `recall` encadena búsqueda, clasificación remota y composición determinista de una semilla con referencias; `session` permite reutilizar el cliente HTTP.
- [x] **Retirada del dashboard:** eliminados UI, servidor y reutilización de feedback por petición del usuario. CLI/JSONL sigue como interfaz; datos históricos conservados.
- [x] **Evaluación y gráficas en Rust:** runner de campañas, validación de bancos, métricas y comparaciones reproducibles; migración de las herramientas activas de Python a Rust completada.
- [x] **Cierre experimental de M5:** resultados, costes, latencias y límites documentados. La conclusión fue viabilidad general no demostrada; este cierre no aprueba uso autónomo general.
- [x] **Investigación de calidad de cinco etapas:** contrato, controles, mecanismo, contraste nuevo e informe cerrados. Tras la comparación amplia se restauró **Decisions + baseline**; single-pass queda experimental, sin mejora general demostrada. [Decisión vigente](docs/decisions/0044-restore-baseline-default.md).

- [x] **Contrato local v1 consolidado:** consulta, resultado, cobertura, degradación, trazas, segmentación sin modelo y proveedores documentados; desarrollo/calibración/final del perfil congelado superados en 18 casos sintéticos. [Alcance y límites](docs/evaluation/product-contract-v1-results-2026-10-23.md).
- [x] **Completar recuperación y aislamiento local:** actor simulado y tipo filtrados antes del límite en FTS5, semántica y composición; registro persistente de entidades/alias con desambiguación explícita, frases FTS5, pesos BM25, deduplicación y cobertura. No incorpora autenticación de host ni correferencia automática. [Contrato](docs/architecture/retrieval-isolation.md) · [Cuatro páginas de pruebas](docs/evaluation/retrieval-isolation/README.md).
- [x] **Definir embeddings locales v1:** E5 multilingual small independiente del clasificador, comparado con MiniLM; artefactos/hashes/runtime fijados, fragmentos sin truncamiento, índices completos por revisión/espacio y coseno exacto con presupuestos. Verificado hasta 10.000 recuerdos/20.000 fragmentos; **conservado solo como investigación por decisión del usuario**, sin adopción en producto. FTS5 continúa como predeterminado. [Contrato](docs/architecture/local-embeddings.md) · [Pruebas y límites](docs/evaluation/local-embeddings/README.md).
- [x] **Completar `recall` local:** mecanismos de la ruta activa implementados y evaluados. [Contrato](docs/architecture/recall-completion-v1.md) · [Resultados y límites](docs/evaluation/recall-completion-v1/README.md).
  - [x] Presupuestos acumulados de entrada y semilla y procedencia literal local.
  - [x] Adquisición HTTPS explícita de texto con hash esperado; importación y replay comprobados realmente. No certifica autoría ni verdad externa.
  - [x] Aplicabilidad y relevancia por candidato en `per_memory_factors_v1`, conservando condiciones, estado, fuente y scores separados.
  - [x] Calibrador optativo ligado a perfil y modelo servido, ajustado en calibración y comprobado en reserva independiente. **Resultado negativo:** los intervalos ensayados empeoran Brier y se rechazan para adopción; no se promueve un umbral ni se cambia baseline.
  E5 y el híbrido permanecen como investigación; su integración queda condicionada a una futura decisión de activarlos. Este cierre local no aprueba calidad autónoma general.
- [x] **Ampliar cobertura y pruebas del producto — M4:** controles adversariales y evaluación experimental completados. [Informe y reproducción](docs/evaluation/m4-completion-v1/README.md).
  - [x] Matriz determinista multilingüe/adversarial, degradación y aislamiento; 24 casos semánticos nuevos, revisión independiente y ocho etiquetas confirmadas por Fernando.
  - [x] Comparación Decisions/Jev sobre los mismos casos, con desarrollo/calibración/final separados y errores conservados.
  - [x] Ablación actual de filtros en 160 búsquedas y replay de embeddings/RRF/pisos de coseno sobre 1.600 + 200 casos históricos, sin encoder activo.
  - [x] Concurrencia y mantenimiento durante 30 segundos: 960 operaciones, cinco procesos, contención real, CPU/RSS e integridad comprobadas. Es una observación local acotada, no operación productiva prolongada.
- [x] **Validar recuperación y selección con un benchmark externo:** evaluación completada sobre LoCoMo, con 272 recuerdos y 1.982 preguntas evaluables; búsqueda y selección de semilla medidas. [Resultados y límites](docs/evaluation/locomo-results-2026-10-24.md). Cierre de este alcance por decisión del usuario (2026-10-24); la revisión humana independiente, la respuesta final y la calibración/reserva nuevas para aprobar uso general siguen aplazadas. LoCoMo contiene conversaciones generadas, no recuerdos personales reales.
- [x] **Escritura y reconciliación local:** `mem-write` crea/revisa/confirma con huellas y tema único; entidades/alias sin fusión por nombre; clasificación de relaciones remota optativa, propuestas fuera de FTS, `mem-resolve`, recibos y auditoría atómicos. Conserva fuentes declaradas por compatibilidad y ofrece política estricta literal; sin ACL de host ni equivalencia semántica calibrada. [Contrato y límites](docs/architecture/write-reconciliation.md) · [Pruebas](docs/evaluation/write-reconciliation/README.md).
- [x] **Ciclo de vida local:** cola persistente E5 por ámbito/perfil, revisión de vigencia declarada, `mem_review_list`, retirada/restauración con revisiones de control, borrado activo y cancelación transaccional; procesamiento explícito sin daemon. [Contrato y límites](docs/architecture/memory-lifecycle.md).
## Pendiente

- [ ] **Integrar con Codex:** exponer MCP y, si se decide, un hook; resolver continuidad tras compactación y medir desde el mensaje del usuario hasta la respuesta final, incluyendo recuperación y entrega automática de la semilla.
- [ ] **Preparar operación sostenida:** observabilidad, migración entre perfiles, respaldo, recuperación, instalación, documentación y otras plataformas; incorporar búsqueda vectorial aproximada solo si el coseno exacto incumple un objetivo medido.
