# Solicitudes sin etiquetas: matriz de revisiones V1

Instantánea 4: conserva las 24 solicitudes idénticas y refresca las huellas de fuente tras añadir un fallback conservador cuando la extracción de slots omite calificadores temporales. Si algún slot conserva explícitamente el calificador, la salvaguarda se limita a ese slot; si ninguno lo conserva, se mantiene el guard global para no dejar pasar evidencia temporalmente insuficiente. `manifest.json` liga esta instantánea a la anterior mediante SHA-256. No contiene corpus, resultados de búsqueda, anotaciones, gold ni respuestas. El hash del binario, prompt/esquema efectivo y recibos por operación siguen pendientes.

Este paquete contiene únicamente los 24 descriptores de consulta y proveedor de G0/G2/D0 y un manifiesto sin hashes de gold. No incluye corpus, resultados de búsqueda, anotaciones ni respuestas esperadas. Los fixtures operativos y anotaciones siguen en el preflight privado; no se debe compartir ese directorio completo con revisores o proveedores.

G0 y G2 usan el mismo modelo solicitado y difieren en el paquete completo de política/prompt/esquema/adaptador. D0 es una comparación descriptiva con otro backend. Cada solicitud debe ejecutarse en el harness local que ya carga el corpus y guarda recibos; no se envían etiquetas al proveedor.

Los hashes de `bootstrap/src/openrouter.rs`, `bootstrap/src/openrouter/slot_evidence.rs`, `apps/retrieval/src/slot_evidence.rs`, `apps/retrieval/src/service.rs` y `Cargo.lock` fijan el código fuente relevante de esta preparación. La respuesta servida, parámetros efectivos, cuerpo/esquema serializados, hash del binario, recibo y tiempos deben quedar registrados por cada ejecución; los hashes de fuente por sí solos no prueban qué binario se ejecutó.
