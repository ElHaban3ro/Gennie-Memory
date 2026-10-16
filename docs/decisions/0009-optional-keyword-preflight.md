# 0009 — Consulta léxica rápida optativa

Fecha: 2026-10-15. Estado: decisión histórica; sustituida por [0010](0010-keyword-search-default.md) para el valor predeterminado.

## Contexto y alternativas

El usuario pidió probar la idea de búsqueda rápida de Engram, comparar iROMEM anterior y nuevo, y separar el tiempo local de la recuperación mediada por Codex. Se consideraron: conservar solo el híbrido natural; sustituirlo por FTS5 de palabras clave; usar palabras clave como primera fase con respaldo semántico cuando la lista estuviese vacía; usar palabras clave dentro del RRF híbrido.

## Decisión y motivo

Se añadió `query_mode: "keywords"` como opción de `SearchRequest` para experimentar sin cambiar el comportamiento predeterminado. La transformación se aplica a la consulta entrante, no a los recuerdos. La prueba final midió 5,23 ms p50 y 452/460 positivos cubiertos en top veinte, frente a 11,93 ms y 459/460 del híbrido anterior. La regla de respaldo por lista vacía no recuperó ninguno de los ocho positivos omitidos porque todas las listas finales contenían candidatos. El híbrido con consulta normalizada tampoco mejoró la cobertura ni el orden suficiente para reemplazar el natural. [Datos y límites](../evaluation/keyword-preflight-v1.md).

## Condición de revisión

Mantener el híbrido natural como ruta de referencia hasta fijar una política de confianza en desarrollo y validarla en sujetos nuevos con semilla final, negativos y latencia completa. No inferir equivalencia de Codex + iROMEM a partir del reemplazo aritmético de unos milisegundos de herramienta.
