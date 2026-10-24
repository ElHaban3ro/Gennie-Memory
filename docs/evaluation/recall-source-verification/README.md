# Procedencia literal local en recall-v1

Fecha: 2026-10-24. Subpaso: verificar fuentes registradas localmente antes de publicar la semilla. E5 permanece como investigación.

| Paso | Implementación y evidencia |
| --- | --- |
| 1 | [Snapshots versionados e inmutables](01-registry.md) |
| 2 | [Comparación literal, rangos y recibos](02-literal-check.md) |
| 3 | [Políticas y revalidación de recall-v1](03-recall.md) |
| 4 | [Recorrido CLI y JSONL](04-cli.md) |
| 5 | [Verificación y cierre acotado](05-verification.md) |

Cierre: **18 controles propios aprobados**; suites conjuntas **1153** con configuración predeterminada y **1169** con embeddings, cero fallos y 25 ignoradas en cada una. Formato y revisión de espacios correctos. [Resultados y hashes](verification.json).

Se implementaron `source-register`/`source-verify` y sus operaciones JSONL; las referencias se resuelven contra texto aportado explícitamente al registro, separado por actor y ámbito. Las versiones son inmutables y no se reactiva una anterior al repetir su registro. La comprobación compara el texto original íntegro con un rango UTF-8 exacto o una ocurrencia única, conserva hashes, versión, momento y motivo, y detecta cambios del snapshot.

`source_policy:report` conserva fuentes no resueltas con su estado visible. `require_verified` exige coincidencia de todo lo incluido en la semilla, también posibles equivalentes. Una fuente incluida rechazada bloquea en ambos modos. Se conserva selección y recibo del proveedor; no hay fallback o texto truncado. Las fuentes se vuelven a comprobar después del clasificador y el aviso final cuenta dentro de los presupuestos de semilla.

**Verified_literal no certifica verdad, resumen, condiciones añadidas o autenticidad externa.** Solo acredita coincidencia del original contra el snapshot del operador. Las políticas controlan publicación de semilla, no el envío previo de candidatos al clasificador. Los conectores externos y la relevancia/aplicabilidad calibradas continúan pendientes.

[Plan](../../architecture/recall-source-verification-plan.md), [contrato](../../architecture/product-contract-v1.md), [decisión 0053](../../decisions/0053-local-source-verification.md). Pruebas con datos sintéticos y bases temporales, sin inferencia pagada, commit o push. La verificación conjunta incluye cambios compartidos ya registrados por los chats de cobertura/ciclo de vida/escritura y no acredita modificaciones posteriores de esos chats.
