# 0045 — Segmentación programática para el contrato de producto

Fecha: 2026-10-23 (America/Bogota).
Estado: aceptado e implementado en `apps/retrieval/src/segmentation.rs`, CLI y session.

## Contexto y origen

Al planificar la consolidación del contrato de producto, el usuario pidió convertir textos largos en frases más pequeñas y conservar una estructura detallada de lo que sucede con los recuerdos. Aclaró: «Primero segmentación fiel y verificable de manera automatizable y programatica. La idea es que no pase por un modelo porque añade gasto de tokens y tiempo.»

## Decisión

Implementar segmentación local determinista, sin inferencia de modelos ni llamadas remotas. Conservar original y rangos UTF-8 verificables; proteger condiciones, negaciones y referencias mediante conservación literal y contexto. No presentar los fragmentos como afirmaciones independientes ni modificar recuerdos persistidos.

La elección técnica implementada es una partición íntegra con advertencias y límites explícitos. Si una oración no admite división conservadora, conservarla larga o rechazar por límite absoluto; no truncar para aparentar reducción. El [contrato vigente](../architecture/product-contract-v1.md) documenta los límites, campos y conservación literal.

## Consecuencias y límites

No consume tokens de un proveedor, pero sí tiempo y memoria locales que deben medirse. Segmentar no reduce por sí mismo el volumen total de texto ni resuelve ambigüedad semántica. Las unidades pueden depender del contexto anterior, y las reglas de puntuación requieren controles para abreviaturas, URLs, código y comillas.

Queda descartado usar un resumen generativo en esta primera versión. Cualquier futura reescritura mediante modelos requiere una decisión nueva; no se habilita como fallback.

## Verificación

Reconstrucción exacta del original, rangos ordenados válidos, Unicode, casos adversariales de puntuación y límites. Evaluar por separado la utilidad de integrar segmentos en recuperación. El [plan de consolidación](../architecture/product-contract-v1-plan.md) define fases, salidas y criterios; la implementación tiene pruebas estructurales, pero no acredita una mejora de calidad semántica.
