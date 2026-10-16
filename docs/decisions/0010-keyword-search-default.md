# 0010 — Búsqueda rápida predeterminada

Fecha: 2026-10-15. Estado: aceptada por decisión explícita del usuario; sustituye la política de activación de [0009](0009-optional-keyword-preflight.md).

## Decisión

La búsqueda FTS5 por palabras clave (`fts5_keywords`) es la ruta base de `search`, `session` y planes nuevos de clasificación cuando no se especifica otra. `query_mode: "natural"` conserva la búsqueda textual anterior y `hybrid-search` mantiene disponible la búsqueda híbrida natural. Decisions `direct` continúa como clasificador principal; Jev es secundario de selección explícita, sin cascada. El perfil de campañas v2 fija esta combinación; v1 permanece inmutable para las corridas históricas. La CLI Rust aún no ejecuta la clasificación.

## Evidencia y coste aceptado

En 460 positivos sintéticos de un banco de 2.800 recuerdos, la ruta rápida incluyó el recuerdo esperado en top veinte en 452 casos, p50 de búsqueda 5,23 ms. La híbrida natural logró 459, p50 11,93 ms. En el piloto de cinco positivos con Decisions, la ruta rápida compuso cuatro semillas correctas frente a cinco con híbrida; sus p50 hasta semilla fueron 0,53 y 0,73 s en corridas distintas. Esta decisión prioriza la menor latencia local aun perdiendo cobertura hoy. No demuestra equivalencia de calidad ni la velocidad de Codex con semilla rápida, todavía no medida. [Informe y límites](../evaluation/keyword-preflight-v1.md).

## Siguiente revisión

Analizar los casos perdidos y probar mejoras pequeñas de la consulta o del índice sobre sujetos nuevos, con negativos, exactitud final de semilla y latencia. No activar un respaldo semántico por lista vacía como solución: en la cohorte final todas las listas fueron no vacías y esa regla no recuperó los ocho positivos omitidos.
