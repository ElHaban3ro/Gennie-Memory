# 1. Registro local de fuentes

Registro explícito `source-register` / `source_register`: schema_version 1, actor_id opcional (`local`), scope, source {kind,ref}, version y text UTF-8. El source de registro no admite locator: el snapshot contiene todo el texto aportado. Kind usa el vocabulario existente user/document/tool/agent_discovery; ninguno implica autenticidad externa. Texto no vacío hasta 256 KiB; JSON CLI hasta 2 MiB; session conserva su cota de 1 MB por línea, también al escapar caracteres. IDs/versión/ámbito se validan con el contrato local.

La migración aditiva 10 crea snapshots inmutables y recibos de comprobación en la conexión existente. La identidad SHA-256 liga perfil, actor, ámbito, tipo/ref, versión y hash textual. La última versión **nueva registrada** es la vigente: el orden es local de registro, no orden alfabético ni una fecha de publicación externa. Repetir la misma versión/texto es idempotente; cambiar su contenido se rechaza; repetir una versión anterior no la reactiva.

El comando confirma la transacción antes de devolver snapshot_id, versión, hash, momento y trust_basis=operator_registered_snapshot. No descarga documentos, sigue URLs o abre rutas de la referencia. Los snapshots retienen texto y crecen con las versiones; no hay GC o sincronización automática con archivos externos.

Pruebas: registro/replay/conflicto de versiones, límites y ausencia de escritura ante entrada inválida; aislamiento de actor y ámbito. Las fuentes se suministran por el operador local: no son evidencia autenticada del host.
