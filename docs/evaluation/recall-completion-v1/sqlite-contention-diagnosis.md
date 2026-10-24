# Diagnóstico de contención SQLite

Fecha: 2026-10-24. Hallazgo incidental del banco M4, sin base personal o proveedor remoto.

Síntoma: sesión CLI inicializada de antemano y solicitud válida `mem_write create` fallan inmediatamente con `database is locked` cuando otra conexión mantiene `BEGIN IMMEDIATE`150 ms, pese al `busy_timeout`5 s configurado por kernel. Repro mínimo preservado en `bootstrap/tests/product_coverage_m4_concurrency.rs::a_cli_writer_waits_for_an_existing_reserved_lock_then_commits_once`.

Comando rojo ejecutado por implementación:

```sh
cargo test --offline -p gennie-memory --test product_coverage_m4_concurrency a_cli_writer_waits -- --nocapture
```

Resultado previo: `writer completed before reserved lock released`, respuesta `status:failed`, `elapsed_ms:0.463709`, `error:database is locked`; 0 aprobadas/1 fallida,0.40 s. El agente M4 reprodujo antes el mismo fallo dos veces (~0.36/0.53ms).

Hipótesis contrastadas: (1)promoción de transacción deferred tras lectura no puede esperar por el escritor reservado, (2) timeout no aplicado a esta conexión CLI, (3) montaje/cliente/bloqueo del test no representa la ruta productiva. Un probe cambió únicamente el bloqueo externo a `BEGIN EXCLUSIVE`: la misma CLI esperó la liberación150 ms y aprobó (1/1,0.58 s), sin cambios productivos. Se restauró el test a `BEGIN IMMEDIATE`. Esto demuestra espera efectiva en la ruta CLI y acota el fallo a adquirir escritura tras lectura, no a falta general de busy_timeout.

Arreglo dirigido: `reconciliation::write` adquiere `TransactionBehavior::Immediate` justo donde empezaba su transacción, después de llamadas remotas y antes de replay/revalidaciones. No se trasladó I/O de proveedor dentro del bloqueo, no hay reintento oculto, segundo SQLite o cambio de timeout. Mismo repro original con `BEGIN IMMEDIATE` aprobó (1/1,0.64 s). Las lecturas siguen sin reserva de escritor. No se hizo sustitución global de transacciones.

La observación original con escritores/lectores/mantenimiento y su verificación final pertenecen al informe M4. Un pase150 ms no demuestra estabilidad de meses o throughput máximo.

Confirmación adicional: mínimo lifecycle separado falló con `elapsed_ms:0.288875` antes de liberar el RESERVED150 ms. El mínimo `embedding_job enqueue` aprobó sin cambios productivos, por lo que no se modificaron sus transacciones. Se corrigió únicamente `lifecycle::apply` con Immediate antes de replay/revisión vigente. Filtro `reserved_lock` final:3 aprobadas/0 fallos,0.83 s, incluyendo escritura, transición de ciclo de vida y cola. La transacción de resolución y otras rutas no se cambiaron preventivamente.
