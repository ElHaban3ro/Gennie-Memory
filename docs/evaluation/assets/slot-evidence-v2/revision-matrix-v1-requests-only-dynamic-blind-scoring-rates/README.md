# Solicitudes sin etiquetas: matriz de revisiones V1

Instantánea 10: conserva las 24 solicitudes idénticas de la revisión 9. Actualiza el hash del evaluador porque el reporte ahora expone numerador, denominador y tasa de selección útil adjudicada, junto con los conteos irrelevante/ambiguo. Una prueba end-to-end con 64 casos sintéticos verifica 63/64 cuando se incluye una selección irrelevante. No contiene corpus, resultados de búsqueda, anotaciones, gold ni respuestas; esta prueba no constituye una cohorte de calidad. `manifest.json` liga esta instantánea a la anterior mediante SHA-256. El hash del binario, prompt/esquema efectivo y recibos por operación siguen pendientes.

Este paquete contiene únicamente los 24 descriptores de consulta y proveedor de G0/G2/D0 y un manifiesto sin hashes de gold. No incluye corpus, resultados de búsqueda, anotaciones ni respuestas esperadas. Los fixtures operativos y anotaciones siguen en el preflight privado; no se debe compartir ese directorio completo con revisores o proveedores.

G0 y G2 usan el mismo modelo solicitado y difieren en el paquete completo de política/prompt/esquema/adaptador. D0 es una comparación descriptiva con otro backend. Cada solicitud debe ejecutarse en el harness local que ya carga el corpus y guarda recibos; no se envían etiquetas al proveedor.

Los hashes de `bootstrap/src/openrouter.rs`, `bootstrap/src/openrouter/slot_evidence.rs`, `apps/retrieval/src/slot_evidence.rs`, `apps/retrieval/src/service.rs` y `Cargo.lock` fijan el código fuente relevante de esta preparación. La respuesta servida, parámetros efectivos, cuerpo/esquema serializados, hash del binario, recibo y tiempos deben quedar registrados por cada ejecución; los hashes de fuente por sí solos no prueban qué binario se ejecutó.


Esta revisión liga también el código del preparador/scorer de revisión ciega (`tools/evaluation/src/revision_matrix.rs`). El harness acepta N tripletes completos; su prueba integral usa 64 casos sintéticos y comprueba la tasa de selección útil adjudicada.
