# 0054 — Escritura y reconciliación local

Fecha: 2026-10-24. Estado: implementada localmente. Verificación y límites en el informe enlazado. Ámbito: completar el paso de escritura/reconciliación del roadmap solicitado por Fernando.

## Decisión

Extender la superficie local sin romper el JSON ni los recibos históricos de `create`/`revise`: nuevo envelope en `retrieval::reconciliation`, misma transacción de `write` para revisión/FTS/huella, propuestas separadas y resolución explícita con control de revisión. `confirm` conserva contenido y añade evidencia. La migración 0011 es aditiva; no se agregan crates, dependencias, conexiones ni servicios.

`source_policy:report` mantiene compatibilidad con importadores locales. El modo explícito `require_verified` abre propuesta si no hay snapshot literal autorizado. Aceptar una propuesta exige evidencia literal de decisión y contenido. El clasificador de relaciones remoto es optativo, limitado y fallable: nunca autoriza fusión/sobrescritura. Se revalidan versiones después del transporte, fuera de la transacción. Las citas, recibos y diagnósticos se conservan con su significado real.

## Alternativas

Hacer estrictas todas las escrituras existentes rompería fixtures/importación y mezclaría fuente declarada con verificación externa inexistente. Fusionar por BM25/coseno o un juicio remoto permitiría cambios destructivos sin autorización concreta. Mantener toda escritura en bootstrap duplicaría validación/SQL y perdería atomicidad. Una conexión nueva o un servicio HTTP no resuelven este problema local.

## Límites y revisión

El operador `local` sigue siendo la autoridad personal, sin ACL multiusuario ni identidad autenticada de host. Solo metadatos del contrato local están implementados; el diseño v0.1 completo sigue siendo más amplio. La clasificación requiere calibración semántica separada; el roster FTS no demuestra cobertura de paráfrasis. Revisar esta decisión al integrar host/MCP, verificación externa o deduplicación semántica calibrada. El ciclo de vida tiene decisión y migración propias, coordinadas con esta entrega.

[Contrato y verificación](../architecture/write-reconciliation.md).
