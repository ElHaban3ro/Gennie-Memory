# iROMEM frente a Engram: búsqueda local con recuerdos sintéticos

Estado: medición de **búsqueda local** realizada el 2026-10-15 en esta Mac. No se consultaron recuerdos reales ni proveedores remotos. Los [resultados por caso](../../outputs/iromem/evaluation/engram-comparison-v1/) conservan tiempos, candidatos y etiquetas. La medición posterior del [recorrido Engram a través de Codex frente a iROMEM directo](engram-codex-end-to-end-v1.md) responde a la latencia de obtención de la memoria completa.

## Resultado

Con 2.800 recuerdos y las 540 preguntas de la cohorte final, iROMEM encontró el recuerdo esperado entre sus veinte candidatos en **459/460 positivos (99,78 %)**. Engram 3.4.0, usando una consulta de palabras clave y `match_mode=any`, lo encontró en **454/460 (98,70 %)**. Engram situó el correcto en primer lugar más a menudo: **426/460 (92,61 %)** frente a **241/460 (52,39 %)** de iROMEM. La lista de Engram llegó aproximadamente dos veces más rápido en este recorrido aislado: p50 **5,52 ms** frente a **11,78 ms** de iROMEM; p95 **6,48 ms** frente a **12,57 ms**. Engram falló seis preguntas que iROMEM cubrió, e iROMEM falló una que Engram cubrió.

| Método, consulta | Correcto top 1 | Top 5 | Top 10 | Top 20 | Búsqueda p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| iROMEM híbrido, pregunta original | 241/460 | 375/460 | 402/460 | **459/460** | 11,78 / 12,57 ms |
| Engram `any`, palabras clave | **426/460** | **444/460** | **452/460** | 454/460 | **5,52 / 6,48 ms** |
| Engram `any`, pregunta original | 108/460 | 148/460 | 220/460 | 332/460 | 0,89 / 5,99 ms |
| Engram `all`, palabras clave | 277/460 | 280/460 | 280/460 | 280/460 | 2,52 / 3,62 ms |
| Engram `all`, pregunta original | 0/460 | 0/460 | 0/460 | 0/460 | 0,50 / 1,04 ms |

El p50 de Engram con la pregunta original y `all` mide sobre todo consultas vacías; no es una ventaja útil. En los 80 negativos, iROMEM y las dos variantes `any` de Engram entregaron candidatos en los 80 casos. La ausencia de candidatos con `all` tampoco demuestra una abstención inteligente.

## Método

- Se cargaron los 2.800 recuerdos activos de `tests/fixtures/general-memory-v1/memories.json` en una base Engram temporal. Cada ámbito e identidad sintética de iROMEM se representó como un proyecto Engram distinto: 100 proyectos en total. `title` recibió el resumen y `content` el texto original. Los IDs de importación se mapearon de vuelta a los IDs del fixture para calcular recall. Engram no recibió las condiciones ni fuentes declaradas de iROMEM; la comparación mide candidatos, no equivalencia de contratos de memoria.
- iROMEM usó su base y sus vectores ya cargados de `.local/pilots/general-memory-v1/`, con FTS5 + coseno exacto + RRF. Engram usó la búsqueda nativa por MCP `mem_search`, límite veinte y proyecto explícito. Ambos procesos permanecieron abiertos por stdio y se calentaron con una consulta excluida de la medición. Los tiempos son ida y vuelta por caso, seriales, e incluyen el transporte y serialización de cada proceso.
- La consulta de palabras clave se fijó tras mirar solo los 135 casos de desarrollo: minúsculas, palabras Unicode, eliminación de tokens de menos de tres caracteres y una lista fija de palabras vacías españolas. No usa etiquetas, memoria esperada ni tema. Se aplicó una vez, sin ajuste posterior, a las 540 preguntas finales. El mismo texto original se pasó a ambos sistemas en la variante de pregunta original. La normalización se aplicó únicamente a Engram en la variante adaptada porque iROMEM usa el vector precalculado de la pregunta original.
- La cohorte final tiene 460 positivos y 80 negativos, con sujetos distintos de desarrollo. En desarrollo, iROMEM obtuvo 115/115 en top veinte; Engram `any` con palabras clave 113/115. Las cuatro tablas de resultados por caso incluyen ambas cohortes y ambas formulaciones.

El ejecutable de Engram es el [release oficial v3.4.0](https://github.com/Gentleman-Programming/engram/releases/tag/v3.4.0) para macOS arm64; el SHA-256 del archivo descargado coincidió con el publicado: `d0aa59c9a83d17ae3fc7f5dd31fdab6a26fd8c00fc5e692195753cd0719294f7`. El binario iROMEM se compiló localmente con `cargo build --offline -p iromem`. Huellas de esta medición: fixture `ee55707918018aece6f687759ef31eac0adc37d85542fae0d2e312bd136c6835`; preguntas finales `75ca1feaf830a30e84cf8ccb01bb1c5d55535ad50b8b6e4e305b35bc6ff8ff06`; base iROMEM `fd2eb78aff4021660ac9bf0f1972e14014cf17e9c64b9b7b6009aa4fc2a0eae9`; ejecutable iROMEM `24652c759b8237ea679de8ffb1148d1528eab74fc87561a5e93c93fb774ad11b`. La rama tenía cambios locales previos; el hash de `HEAD` era `b7dc87d20619617a7dcf435b5785525aa264004d` y por sí solo no identifica el binario.

## Qué significa para una semilla final

Esta tabla evalúa **recuperación y orden de candidatos**. Engram no ejecutó un clasificador que decida entre candidato y `none` con el contrato de iROMEM. Dar automáticamente su primer resultado produciría una respuesta candidata en los 80 negativos; no se probó si un agente consumidor sabría descartarla. La clasificación Decisions de iROMEM, medida antes sobre esta misma cohorte y las listas híbridas congeladas, logró **436/460 positivos con semilla correcta y 80/80 negativos vacíos**, con decisión p50/p95 **473/696 ms**; véase [general-memory-v2-results.md](general-memory-v2-results.md). Ese tiempo es de otra campaña y no se debe sumar a percentiles de esta tabla como si fuera una sola corrida punta a punta.

Tampoco se midió en esta comparación la generación en vivo del vector de la consulta, la carga inicial de Laya, la escritura/indexación de recuerdos, memoria RAM, permisos, ni la latencia dentro de Codex. Por tanto, la ventaja local de tiempo de Engram es válida para esta llamada de búsqueda ya preparada, y el coste completo de iROMEM será mayor cuando deba codificar la consulta y llamar a Decisions. El banco usa plantillas sintéticas y ámbito conocido; no permite extrapolar porcentajes a conversaciones reales.

## Interpretación

Engram es una referencia fuerte para ordenar el primer resultado en este banco **si se le pasa una consulta corta de palabras clave**. iROMEM conserva una pequeña ventaja de cobertura en top veinte y un mecanismo medido de decisión y abstención, a costa de más latencia. La mayor oportunidad concreta para iROMEM es mejorar el orden inicial de su lista híbrida sin sacrificar la cobertura; comparar solo el top veinte oculta esa diferencia. Una comparación punta a punta exigiría configurar en ambos sistemas el mismo selector posterior, la misma política de `none`, el vector generado en vivo en iROMEM y las mismas reglas de ámbito.
