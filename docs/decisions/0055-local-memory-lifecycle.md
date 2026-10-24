# 0055 — Ciclo de vida y cola local persistente

Fecha: 2026-10-24. Estado: implementada localmente; comprobaciones en el contrato [memory-lifecycle](../architecture/memory-lifecycle.md).

## Contexto y decisión

El usuario autorizó desarrollar cola de embeddings, revisión de vigencia, retirada, restauración, borrado activo y cancelación, coordinando con los chats de escritura, fuentes y cobertura. Se mantiene una conexión por proceso y CLI/JSONL. `retrieval::lifecycle` posee una migración nueva, recibos transaccionales idempotentes y calendario por revisión. La retirada cambia visibilidad sin destruir historial. El borrado elimina contenido activo y conserva solo recibos mínimos; las fuentes independientes no se borran implícitamente.

La cola suscribe ámbitos/perfiles E5 explícitos y reconstruye índices completos. Generación y token de concesión cercan la publicación y su cancelación dentro de la transacción del índice. La inferencia ocurre fuera de la transacción. No se inicia procesamiento automático ni se integra E5 en recall.

## Alternativas y coste

Una cola efímera pierde trabajo al cerrar el proceso. Una cola por recuerdo no representa el contrato actual de publicación de índices E5 completos. Se elige una fila durable por ámbito/perfil, con generación, estado, intentos, error y concesión recuperable. El coste es reconstruir el ámbito completo y perder índices completos que contienen contenido borrado. No se añade un scheduler ni heartbeat.

## Condición de revisión

Revisar cuando exista indexación incremental validada, actor autenticado o worker persistente. Para trabajos mayores de diez minutos hace falta renovación de concesión o segmentación. El borrado activo no promete eliminación en backups/WAL ni en fuentes independientes. Recuperar trabajos tras un crash no implica continuidad del contexto del agente tras compactación.
