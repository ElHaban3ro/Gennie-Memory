# 0050 — Completar recuperación y aislamiento local

Fecha: 2026-10-24. Estado: implementada para CLI personal y actores simulados.

Contexto: M1 tenía filtros por ámbito/entidad/revisión/tiempo, pero faltaban actor/tipo, resolución persistente de entidades, frases FTS5 y contabilidad de cobertura. La escritura exacta e idempotente existente debía conservarse.

Decisión: migración aditiva 8; actor `local` predeterminado, filtros anteriores al límite en todas las rutas y registro local de entidades por actor/ámbito con alias y fuente declarada. Coincidir por nombre devuelve candidatos sin atribuir identidad; ID explícito resuelve y `create_new` permite homónimos. Escritura enlaza entidades ya registradas, conserva identidad en revisiones y deduplica contenido sin cruzar actor/entidad. No se introducen ACL o autenticación de host.

FTS conserva su tokenizer y pesos anteriores como predeterminados; admite frases literales y pesos explícitos, mantiene proyección por triggers y devuelve cobertura cuantitativa bajo una instantánea consistente. El híbrido expone sus cotas. La operación de producto falla cerrada ante índice incompleto y distingue consulta sin términos buscables de ausencia de candidatos.

Alternativas: fusionar automáticamente el único nombre coincidente confundiría homónimos; usar modelo para resolver identidad añadiría inferencia no necesaria; implementar cuentas/ACL ampliaría el producto personal; reconstruir todas las huellas en cada escritura penalizaría lotes. Se usan selección explícita y actualización perezosa de claves derivadas.

Costes y límites: las consultas añaden conteos de cobertura; no se midió rendimiento nuevo en LoCoMo ni concurrencia. El registro resuelve identidad declarada, no pronombres, fuentes externas o verdad semántica. Fuente conservada como `declared_unverified`; etiquetas de pruebas son del mismo autor.

Verificación: [cuatro páginas de pruebas](../evaluation/retrieval-isolation/README.md), [contrato](../architecture/retrieval-isolation.md). Reabrir antes de autenticar un host, admitir entidades múltiples por memoria, fusionar identidades automáticamente o cambiar cotas/rendimiento objetivo.
