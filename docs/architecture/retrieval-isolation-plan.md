# Recuperación y aislamiento local

Fecha: 2026-10-24. Estado: cuatro etapas implementadas y verificadas. Alcance: primer pendiente de ROADMAP-SUMMARY.md.

## Plan y puertas

1. Filtros de actor y tipo: migración aditiva, validación, FTS5, semántica y listado de ámbito. Pruebas: distractores mejor puntuados y límite 1; composición rechaza IDs excluidos. [Página 1](../evaluation/retrieval-isolation/01-filters.md).
2. Entidades persistentes: registro por actor/ámbito, alias explícitos, idempotencia y enlace desde escritura. Un nombre no demuestra identidad; coincidencias requieren selección explícita. Pruebas de homónimos, fuentes, reintentos y rollback. [Página 2](../evaluation/retrieval-isolation/02-entities.md).
3. FTS5 y cobertura: frases entrecomilladas literales, operadores tratados como texto, pesos BM25 opcionales, identidad única por memoria/revisión y contabilidad del conjunto elegible, indexado, coincidente y limitado. [Página 3](../evaluation/retrieval-isolation/03-fts-coverage.md).
4. Integración y cierre: CLI/JSONL, DB nueva e histórica, checksum, suite offline, documentación vigente. [Página 4](../evaluation/retrieval-isolation/04-integration.md).

## Decisiones de alcance

Se mantiene la conexión SQLite única, rama actual, migraciones históricas y deduplicación exacta existente. No se requiere dependencia nueva, servidor, puerto o inferencia. El actor omitido es `local`; los IDs aportados a búsqueda/fixtures/resolución son filtros simulados del operador local, no autenticación o ACL. La escritura personal pertenece a `local`. Una integración futura deberá suministrar el actor desde una conexión autenticada.

Las entidades históricas se registran sin inferir nombres reales (`kind=subject`, label=ID). Resolver significa registrar identidades y alias declarados con procedencia; no extraer correferencia con un modelo ni certificar fuentes. No se fusionan homónimos por nombre. La deduplicación de contenido debe conservar actor y entidad además del ámbito.

Las páginas de prueba son documentos del repositorio con contratos, comandos y evidencia; no reintroducen el dashboard retirado. Se ejecutan pruebas focales por etapa y una suite offline del workspace al cierre, sin campañas pagadas ni bases personales. El usuario autorizó implementar, probar e iterar todas las etapas en esta sesión.
