# 4. Recorrido local reproducible

Los comandos siguientes necesitan GENNIE_MEMORY_DATABASE_PATH apuntando a una base temporal propia. La referencia no es una ruta que el producto lea; el operador aporta el contenido explícitamente.

Registro de ejemplo:

```json
{"schema_version":1,"scope":{"kind":"project","id":"demo"},"source":{"kind":"document","ref":"synthetic-note"},"version":"v1","text":"No enviar si llueve en Bogotá 😀."}
```

Escribir un recuerdo con ese texto en memory.original_text, source del mismo tipo/ref y locator omitido o rango UTF-8 exacto. El resumen puede tener otro texto; la comprobación no lo certifica. Verificación explícita:

```json
{"schema_version":1,"scope":{"kind":"project","id":"demo"},"memory_id":"<id devuelto por mem-write>","revision":1}
```

CLI: `source-register <registro.json>`, `source-verify <verificación.json>`. Session usa líneas `{op:source_register,request:...}` y `{op:source_verify,request:...}`. Para exigir procedencia en una recuperación: añadir `source_policy:"require_verified"` a una solicitud recall-v1 válida. Si hay un locator heredado no soportado, revisar el recuerdo mediante el contrato existente con locator canónico; no editar SQL de una base personal para hacer pasar la verificación.

`bootstrap/tests/source_verification_cli.rs` prueba procesos separados, registro/escritura/verificación/reapertura y mismo recibo, además de sesión con nueva versión, rechazo, actor ajeno, conflicto y replay viejo sin reactivación. Una recuperación con límite de un token comprueba que recall-v1 ve el mismo recibo antes de bloquear el envío. La clave es sintética y ese control impide contactar al proveedor. Bases/directorios se retiran al terminar, sin puerto o proceso persistente.
