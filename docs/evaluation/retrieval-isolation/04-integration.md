# 4. Integración y cierre

Fecha: 2026-10-24. Estado: verificado. [Índice](README.md).

Reproducción:

```sh
cargo test --offline -p gennie-memory --test retrieval_isolation_cli
cargo test --offline --workspace
cargo fmt --all --check
git diff --check
```

La prueba `bootstrap/tests/retrieval_isolation_cli.rs` ejecuta el binario con directorio y SQLite temporales, sin clave remota. Comprueba `entity-resolve`, repetición idempotente, `mem-write`, frase FTS con entidad/tipo/pesos, persistencia entre procesos y cinco líneas JSONL: resolución, búsqueda, actor ajeno, JSON inválido y búsqueda posterior válida. Comprueba que la sesión conserva su conexión después del error y elimina sus archivos al terminar.

`upgrade_preserves_history_and_checksums_and_registers_legacy_entities` comprueba migración 7→8, actor `local` para datos anteriores, registro de IDs históricos, migraciones repetibles y rechazo del checksum alterado de la 8. Kernel conserva sus pruebas de rechazo de bytes históricos alterados.

Iteración de integración: el primer intento de compilación de bootstrap detectó un `Ok(())` en un brazo que debía devolver unidad; se corrigió antes de ejecutar el binario.

[Registro de verificación y hashes del código](verification.json).

Resultado final: 1.091 pruebas aprobadas, cero fallos, 25 ignoradas; incluye 20 controles unitarios nuevos y una prueba CLI nueva. Formato y diff sin errores. Las ignoradas son las pruebas manuales ya existentes; no se habilitaron campañas remotas.

Se verificó el código local mediante ejecución. No se modificó la base personal, no se abrió servidor, no hubo inferencia remota ni se publicó o desplegó código. Las métricas LoCoMo anteriores corresponden a otra versión medida; esta entrega no presenta tiempos nuevos de recuperación o calidad.
