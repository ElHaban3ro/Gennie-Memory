# 0005 — Filtro estructurado de entidad para la ablación

Fecha: 2026-10-15. Estado: implementado como filtro experimental; resolución real de identidad pendiente.

En 50 casos de desarrollo del corpus v2, el híbrido sin identidad de entidad mantuvo recall@20, pero cayó de 29/30 top-1 de FTS5 a 12/30. El encoder provisional de Laya tendía a priorizar otro proyecto con el mismo nombre base y la misma tarea, distinto número. Ajustar RRF solo con esa muestra ocultaría el problema de identidad y sobreajustaría la fusión.

Se añadió la migración `0003_entity_filter.sql` sin editar las anteriores. `entity_id` opcional se conserva como identidad estable por memoria; las búsquedas FTS5 y semántica lo filtran junto a ámbito, revisión, estado y tiempo antes de limitar candidatos. v3 copia los textos y etiquetas de v2 y agrega un identificador resuelto por un host sintético. La prueba Rust usa dos memorias del mismo ámbito y demuestra que un distractor semánticamente más próximo de otra entidad no entra en el ranking; también rechaza cambiar la entidad de una memoria al cargar una revisión.

En los mismos 50 casos v3, FTS5 e híbrido obtuvieron 30/30 positivos top-1. Este corpus solo tiene una memoria activa por entidad y ámbito, así que el filtro simplifica la tarea de forma importante. No existe todavía un resolver de nombres/alias ni evaluación de múltiples memorias dentro de la misma entidad. El resultado es una ablación de contexto estructurado **ya proporcionado**, no una medida extremo a extremo del producto. Ver [piloto](../evaluation/pilot-2026-10-15.md) y [roadmap](../../ROADMAP.md).
