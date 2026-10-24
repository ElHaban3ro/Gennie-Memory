# 0047 — Retirar el dashboard del alcance actual

Fecha: 2026-10-23 (America/Bogota). Estado: implementado por petición explícita del usuario.

## Decisión

Se elimina apps/dashboard completo: crate, UI React/Vite, servidor HTTP, comando dashboard y dependencia/registro en bootstrap. recall y session pasan directamente por retrieval y no reutilizan revisiones del dashboard. Se retiran Axum y dependencias que quedan sin consumidores; Tokio solo conserva los recursos necesarios para transporte/pruebas existentes.

La interfaz activa es CLI/JSONL, sin puertos de servidor local. No se introduce una UI o mecanismo de revisión alternativo. La revisión humana queda aplazada hasta nueva indicación del usuario.

## Datos e integridad

No se borran bases ni revisiones guardadas. La migración dashboard:1 ya aplicada es inmutable: sus bytes se trasladan a bootstrap/migrations/retired_dashboard/0001_review.sql. Bootstrap verifica versión/checksum si existen registros históricos; rechaza alteración o versión desconocida. No registra un módulo ficticio ni aplica el SQL retirado a bases nuevas. Las tablas existentes permanecen sin uso activo. Kernel no importa recursos de apps.

La nueva respuesta v1 informa local_review:not_available_dashboard_removed. No se modifica la selección baseline ni se hacen llamadas remotas como parte de esta retirada.

## Evidencia anterior y límites

Las campañas previas permanecen históricas y ligadas a su binario anterior. Se archivó ese ejecutable en el directorio de la campaña para poder auditarlo después de reconstruir el producto sin dashboard. Los hashes/etiquetas/resultados previos no cambian.

La omisión observada en baseline significa que devolvió abstención pese a un recuerdo con la respuesta explícita. Que el caso incluyera contexto describe la entrada; aún no demuestra que el contexto causara el fallo. La consulta pedía el formato del informe de Aliso y el recuerdo especificaba una tabla de tarea, responsable y fecha. Las variantes por recuerdo lo seleccionaron; baseline se abstuvo con los tres modelos. No equivale a timeout, ausencia física de recuerdos ni pérdida de datos.

Verificar DB nueva sin tablas de dashboard, conservación y checksum en DB histórica, CLI/session activos y suites offline default/HTTP2. No modificar bases de usuario, detener procesos ajenos ni ejecutar llamadas pagadas para estas comprobaciones.

## Verificación ejecutada

- Suite offline workspace: 1.058 correctas, cero fallos, 25 ignoradas.
- Workspace con HTTP2: 1.060 correctas, cero fallos, 80 ignoradas. Incluye la aserción fija del hash original del SQL retirado.
- Compilación offline del binario actual, formato y git diff --check correctos. No se agregaron versiones de dependencias; se retiraron diez paquetes sin consumidores del lockfile.
- Ocho controles CLI con bases temporales propias: esquema nuevo sin dashboard, comando retirado rechazado, base histórica conservada, recall/session sin revisiones, estado v1, checksum alterado rechazado y auditoría de la campaña previa con su binario archivado. Las bases temporales se eliminaron al terminar.
- Se conservaron las advertencias preexistentes de código no usado. No se ejecutó npm, no hubo llamadas pagadas ni servicios persistentes iniciados/detenidos.

Los resultados sintéticos anteriores siguen siendo evidencia del binario archivado. La retirada actual se verifica por las pruebas y controles anteriores, sin volver a consumir la reserva final ni atribuirle mediciones nuevas.
