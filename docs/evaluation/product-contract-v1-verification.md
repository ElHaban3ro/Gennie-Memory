# Verificación local del contrato de producto v1

Fecha: 2026-10-23 (America/Bogota). Implementación optativa de `recall-v1`, segmentación literal, capacidades, diagnóstico del transporte y validador prospectivo. No representa resultados de calidad nuevos.

| Comprobación | Resultado |
| --- | --- |
| `cargo test --offline --workspace` | 1053 pasan, 0 fallan, 25 ignoradas |
| `cargo test --offline --workspace --features gennie-memory/http2-transport` | 1055 pasan, 0 fallan, 80 ignoradas |
| Segmentación focal tras optimizar secuencias largas de espacios | 3 pasan; conservación de entrada de 256 KiB |
| Plazo focal con feature HTTP/2 | 6 pasan; servidores de prueba propios cerrados |
| CLI/session/validador, binarios compilados offline | 9 controles pasan con base temporal propia y credencial vacía |
| Formato y whitespace | `cargo fmt --all --check`, `git diff --check` pasan |

Los nueve controles verifican: falta de credencial con candidatos conservados; búsqueda vacía; aclaración sin llamadas; reconstrucción literal por CLI; catálogo; recall_v1, segment_text y provider_capabilities por session; manifiesto de tres particiones mediante ruta relativa. La base temporal se elimina al terminar. La compilación final vuelve a incorporar la optimización del segmentador; su comprobación CLI reconstruye también la entrada de tamaño máximo.

La suite default se ejecutó antes de la última optimización de recorrido de espacios; esa optimización se verificó después tanto focalmente como en el workspace completo con HTTP/2. No cambió el contrato ni los resultados literales. Se conservaron advertencias preexistentes por import/símbolos no usados. Pruebas ignoradas no se habilitaron: no hubo inferencias remotas ni gasto API. Los mocks HTTP usaron loopback y puertos libres.

Las pruebas cubren estructura, límites, errores y composición con respuestas simuladas. No prueban disponibilidad real, precio, exactitud semántica ni mejora de recuperación. No se ejecutó nueva calibración ni prueba final; el validador rechaza fugas declaradas y manipulación de hashes, pero no adjudica datos ni verifica por sí solo toda la coherencia semántica del banco. Persisten los límites del [contrato](../architecture/product-contract-v1.md) y el [protocolo prospectivo](product-contract-v1-protocol.md).
