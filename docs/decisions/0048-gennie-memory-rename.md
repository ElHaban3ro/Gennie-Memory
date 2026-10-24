# 0048 — Nombre Gennie Memory y remoto nuevo

Fecha: 2026-10-23. Estado: aceptada por solicitud explícita del usuario.

El producto pasa de iROMEM a **Gennie Memory**. El remoto `origin` de fetch y push es `git@github.com:ElHaban3ro/Gennie-Memory.git`. Esta decisión sustituye los nombres y el remoto de la inicialización, sin cambiar funcionalidad ni arquitectura.

## Nombres vigentes

| Uso | Nombre |
| --- | --- |
| Producto | Gennie Memory |
| Paquete y ejecutable CLI | `gennie-memory` |
| Evaluador | `gennie-memory-eval` |
| Crates internos | `gennie-memory-kernel`, `gennie-memory-retrieval` |
| Imports Rust | `gennie_memory_kernel`, `gennie_memory_retrieval` |
| Variables de entorno | `GENNIE_MEMORY_*` |

El `.env` local se actualiza únicamente en los nombres de sus variables; conserva valores y ruta a la base existente. Quien exporte variables desde su shell o desde herramientas externas debe actualizar el prefijo; no se introduce un alias silencioso al nombre anterior.

## Excepciones históricas autorizadas

El usuario confirmó conservar evidencia histórica y sus rutas. No se reescriben `outputs/`, `.local/`, bases SQLite, binarios archivados, fixtures de datos, SVG históricos, SQL de migraciones ni el historial de Git. `target/` conserva productos antiguos de compilación; los ejecutables nuevos se generan con el nombre vigente. La carpeta raíz continúa llamándose `iROMEM`.

Se preservan rutas a esos archivos, IDs de esquemas documentales congelados, claves JSON históricas como `/search/iromem` y claves de hashes de binarios antiguos. Son contratos de lectura de evidencia, no nombres del producto actual. Los registros originales de `.creation/` y `docs/memory/` son históricos; esta decisión tiene precedencia. Los gráficos del README conservan sus etiquetas históricas y sus mediciones originales.

Los cambios se aplican sobre la rama y los archivos de trabajo existentes. Actualizar `origin` no publica código ni acredita acceso remoto; no se realiza commit, push ni despliegue.

## Verificación local

- `cargo fmt --all --check` y `git diff --check`: correctos.
- `cargo test --offline --workspace`: 1.058 pruebas correctas, cero fallidas y 25 omitidas.
- Compilación de `gennie-memory` y `gennie-memory-eval`: correcta.
- CLI nueva: inicialización, carga del fixture pequeño y búsqueda con candidatos en SQLite temporal, eliminada al finalizar.
- `gennie-memory-eval validate-benchmark tests/benchmark/v5`: correcto.
- Cargo.lock conserva versiones, checksums y dependencias previos al renombrado; solo cambian nombres de paquetes propios.
- Evidencia archivada preservada; no se ejecutaron campañas remotas ni se modificó la base del usuario.

La suite y la ejecución CLI se realizaron sobre el árbol de trabajo completo, que incluía cambios previos del usuario; el commit selectivo no se sometió de nuevo a pruebas avanzadas. Estas comprobaciones validan el renombrado local; no acreditan publicación, despliegue ni una medición nueva de calidad o rendimiento.
