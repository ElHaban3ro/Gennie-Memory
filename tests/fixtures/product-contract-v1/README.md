# Banco prospectivo de producto v1

18 consultas y 36 recuerdos sintéticos nuevos: seis consultas por partición, con fuentes, entidades y familias declaradas disjuntas. Perfil congelado: Decisions + `per_memory_v1`, FTS keywords, hasta 20 candidatos, entrada 32768 bytes, semilla 16384 bytes y plazo 25000 ms. Es un perfil experimental para observar relaciones por recuerdo; no sustituye Decisions + baseline ni sus métricas históricas.

Cada partición contiene tres selecciones (apoyo, refutación y contexto parcial), una abstención con distractores, una referencia declarada sin resolver y una hipótesis de búsqueda vacía. La campaña posterior de Decisions/per_memory_v1 completó las tres etapas y sus puertas; [resultados](../../../docs/evaluation/product-contract-v1-results-2026-10-23.md). La separación por familias usa los escenarios específicos anotados; las categorías semánticas se repiten para poder evaluarlas. No se afirma ausencia automática de toda semejanza lingüística entre particiones.

## Artefactos

- `corpus.json`: formato de carga del producto, con fuente sintética declarada, entidad e ID/revisión.
- `development.jsonl`, `calibration.jsonl`, `final.jsonl`: solicitudes runtime separadas de etiquetas. No pasar la fila completa al proveedor; usar únicamente `request`.
- `policy.json` y `suite.json`: política y protocolo congelados por hash. Umbrales de control del banco: conservar las nueve evidencias útiles, cero ajenas/degradaciones, p95 ≤1500 ms. Muestra insuficiente para una afirmación estadística de calidad poblacional. El protocolo se ejecutó con máximo 12 llamadas, 300 segundos y USD 0,10; el gasto informado fue USD 0,0006135. El runner aplica reserva contable y no garantiza el precio facturado por el proveedor. Para cualquier nueva campaña, confirmar precios/disponibilidad y usar una reserva adecuada.
- `review-packet.jsonl`: casos en orden mezclado, IDs opacos, solicitudes y recuerdos, sin etiquetas ni nombres de partición.
- `review-rubric.md`: definición de relaciones y precedencia explícita cuando se solapan datos provisionales y refutación.
- `review-a-v2.json`, `review-b-v2.json`: dos revisiones separadas con acceso únicamente al paquete y rúbrica. Juicios completos y preocupaciones preservados; no validación humana ni independencia de familia de modelos.
- `review-manifest.json`, `review-audit.json`: identidad, hashes, concordancia y límites de revisión. El coste de los subagentes es desconocido, no cero.
- `development-preflight.json`: ejecución local real de seis requests, con base temporal propia y credencial vacía. Comprueba esquema, carga, búsqueda, aclaraciones y degradación sin pago; no mide selección remota.
- `history/draft-1/`: primera propuesta y revisiones íntegras, conservadas antes de corregir la ambigüedad de HIN-24 y aclarar la rúbrica. Los archivos `review-a.json`/`review-b.json` de la raíz son también primeras revisiones; solo los `-v2` corresponden al paquete vigente.

## Reproducir la auditoría local

Desde la raíz del repositorio:

```sh
cargo run --offline -p gennie-memory-eval -- validate-product-suite tests/fixtures/product-contract-v1/suite.json
cargo run --offline -p gennie-memory-eval -- audit-product-reviews tests/fixtures/product-contract-v1/suite.json tests/fixtures/product-contract-v1/review-packet.jsonl tests/fixtures/product-contract-v1/review-rubric.md tests/fixtures/product-contract-v1/review-a-v2.json tests/fixtures/product-contract-v1/review-b-v2.json
```

Ambos comandos son locales, sin credenciales, SQLite ni red. El auditor exige paquete idéntico al corpus/solicitudes, hashes vigentes, dos identidades distintas, todos los casos y recuerdos, sin selecciones duplicadas/ajenas. Informa desacuerdos y preocupaciones sin resolverlos automáticamente. Identidades distintas no prueban por sí solas independencia: en esta preparación el coordinador aisló los archivos de entrada de cada subagente.

## Conciliación de observaciones

1. Licencia: la primera pregunta no identificaba solicitud. La versión vigente menciona HIN-24 en pregunta y recuerdos; ambos revisores aceptaron que la refutación se limita a esa solicitud.
2. Provisionalidad: se fija `partial_context` cuando existe un valor provisional/condicionado del dato pedido; `refutes_premise` para negar el evento sin ese valor. Es convención de esta evaluación, no exclusividad semántica universal. Ambos revisores la aceptaron.
3. Aclaraciones: `unrelated` en las revisiones representa selección suspendida; la salida real es aclaración, sin juicio del proveedor. Las asignaciones de área no se convierten en autorización de cambios.
4. Ausencia: las tres hipótesis `no_candidates` requieren búsqueda real. El preflight comprobó primero desarrollo; la campaña posterior verificó también calibración/final. Tener distractores de un mismo tema no aporta el dato ni prueba almacenamiento vacío.
5. Condiciones: no inferir moneda, año no declarado, permisos ni requisitos adicionales por proximidad textual. Conservar literalmente los condicionantes en la semilla.

Las etiquetas se revisaron antes de la campaña. Desarrollo, calibración y final se ejecutaron posteriormente sin ajustar prompts ni parámetros entre etapas. La reserva final está consumida; no reutilizarla para ajustar una política. Una campaña posterior que motive cambios deberá conservar sus resultados y renovar la reserva afectada.

El campo remote_evaluation de review-manifest.json registra el estado al revisar las etiquetas, previo a la campaña; se conserva como evidencia de esa secuencia. Los resultados reales se guardan aparte.
