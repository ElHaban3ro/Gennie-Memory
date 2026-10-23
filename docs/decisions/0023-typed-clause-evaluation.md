# 0023 — Cláusulas tipadas para diagnóstico de extracción

Fecha UTC: 2026-10-20. Estado: experimento de evaluación, sin adopción de producto.

Los pilotos anteriores generaban muchos campos no aplicables e incluso invertían una negación pasando el esquema. Se prueba una representación por cláusula y tipo de hecho, con traducción pura al contrato V5 y validación posterior del parser original. La interface recibe tarea, texto, ontología y salida tipada; no recibe etiquetas ni frames manuales. Se mantienen varias cláusulas y lecturas correlacionadas para no perder parciales/refutaciones. No se presume que tipado o cita prueben fidelidad.

El módulo compartido pertenece a `tools/evaluation/src/documentary_extraction_contract/typed_clauses.rs`; el runner de pruebas de bootstrap lo incluye junto al contrato remoto. No es política runtime ni app nueva. El grafo de crates de producto no cambia. El evaluador añade una dependencia directa `serde.workspace=true`, reutilizando exactamente `serde=1.0.229` con `derive`, ya fijada y presente en bootstrap/kernel. No se introduce una versión o paquete externo nuevo ni se cambian sus features globales. `Cargo.lock` registra esa referencia directa del evaluador.

Versión, origen/checksum y uso previo comprobados en el lockfile y [decisión inicial](0001-local-baseline.md). Se volvió a consultar la [ficha oficial docs.rs de 1.0.229](https://docs.rs/crate/serde/1.0.229), que declara `serde_core=1.0.229`, derive opcional y repositorio serde-rs/serde. La página RustSec del paquete no fue accesible en esta comprobación: no se afirma ausencia de avisos ni auditoría exhaustiva. Las builds/pruebas offline comprueban compatibilidad con la toolchain actual; no sustituyen una auditoría de cadena.

La identidad calculada conserva nombre, tipo, propietarios y grupo fijo explícito; `all` y referencia a clase no cambian la clave de esa clase. No se añaden aliases obtenidos de gold para igualar IDs manuales. Si una cláusula o alcance no tiene traducción fiel a V5 debe degradar/ser unsupported, sin descartar solo la parte difícil.

Condición de revisión: fidelidad y cobertura completas en piloto con críticas separadas antes de ampliar; luego banco completo, casos difíciles, recursos/estabilidad y reserva nueva. El [protocolo](../evaluation/typed-clause-extraction-v1-protocol-2026-10-20.md) define límites. Ningún pase manual cierra por sí solo el objetivo ni autoriza promoción.
