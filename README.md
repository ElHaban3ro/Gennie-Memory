# Gennie Memory

Gennie Memory prepara recuerdos relevantes para que un agente de IA responda con contexto breve, sin tener que recorrer todo el historial en cada consulta. Corre localmente mediante CLI y sesiones JSONL.

## Índice

- [Rendimiento](#rendimiento)
- [Comparación con otros motores](#comparación-con-otros-motores)
- [Contexto y respuesta del agente](#contexto-y-respuesta-del-agente)
- [Cómo funciona](#cómo-funciona)
- [Contrato de producto v1 y comandos nuevos](docs/architecture/product-contract-v1.md)
- [Funcionamiento técnico y diseño de la memoria](docs/architecture/memory-design.md)
- [Roadmap resumido](ROADMAP-SUMMARY.md)
- [Roadmap detallado e historial](ROADMAP.md)

## Rendimiento

Configuración principal: **Decisions + baseline**. La corrección de evidencia explícita pasó [30 recuperaciones de desarrollo, incluida Aliso](docs/evaluation/baseline-evidence-30-results-2026-10-23.md).

La tabla y las gráficas siguientes conservan mediciones **anteriores a esa corrección**, con **2.800 recuerdos y 540 consultas sintéticas**; no miden el payload vigente.

| Indicador | Gennie Memory |
| --- | ---: |
| Recuerdo esperado entre los 20 candidatos | **457/460 · 99,35 %** |
| Selección correcta de la semilla | **441/460 · 95,87 %** |
| Semilla vacía cuando no corresponde seleccionar | **80/80 · 100 %** |
| Tiempo hasta la semilla: mediana / p95 | **409 / 863 ms** |
| Coste informado por Decisions para las 540 consultas | **USD 0,029742** |

La semilla es el contexto seleccionado para el agente; su acierto no equivale a la exactitud de la respuesta final. [Datos y alcance](docs/evaluation/epistemic-single-pass-competitor-comparison-2026-10-23.md).

## Comparación con otros motores

Recuperación de candidatos sobre los mismos 2.800 recuerdos y 540 consultas. Estos tiempos incluyen la búsqueda; la selección de la semilla de Gennie Memory se muestra arriba.

| Búsqueda de candidatos | Gennie Memory | Engram 3.4.0 | Lodis 0.6.0 | Basic Memory |
| --- | ---: | ---: | ---: | ---: |
| Recuerdo esperado entre 20 | **99,35 %** | 99,13 % | 84,57 % | 60,87 % |
| Recuerdo esperado en primera posición | 91,30 % | **92,83 %** | 10,22 % | 60,00 % |
| Latencia mediana | **6,93 ms** | 7,22 ms | 19,67 ms | 24,12 ms |
| Latencia p95 | **7,77 ms** | 10,71 ms | 40,94 ms | 127,72 ms |

Gráficas de referencia del **16 de octubre de 2026**. Las rutas de acceso de cada motor son distintas; los detalles están en la [comparación completa](docs/evaluation/broader-comparison-2026-10-16.md).

![Recuerdos esperados entre los candidatos de cada motor](docs/evaluation/assets/broader-comparison-accuracy-2026-10-16.svg)

![Latencia de búsqueda de los cuatro motores](docs/evaluation/assets/broader-comparison-latency-2026-10-16.svg)

![Variación de la latencia p95 de los cuatro motores](docs/evaluation/assets/broader-comparison-p95-2026-10-16.svg)

## Contexto y respuesta del agente

Con la semilla de Gennie Memory ya preparada, el turno de Codex tuvo una mediana de **4,99 s**, frente a **10,65 s** con Engram, **12,66 s** con Lodis y **15,29 s** con Basic Memory. La referencia es pequeña: seis turnos sintéticos por ruta, con instrucciones y contextos distintos.

La preparación de la semilla se mide aparte. La gráfica siguiente conserva la referencia del **16 de octubre de 2026**: **370 ms** de mediana y **567 ms** en p95; la tabla superior conserva una referencia posterior, también anterior a la corrección del payload.

![Tiempo de preparación de la semilla: referencia del 16 de octubre de 2026](docs/evaluation/assets/full-comparison-seed-preparation-2026-10-16.svg)

Sumar preparación y respuesta por consulta dio medianas de **5,20–5,80 s** para Gennie Memory. Es un tiempo calculado: la entrega automática al agente sigue pendiente.

![Preparación más respuesta: tiempo calculado frente a los turnos con otros motores](docs/evaluation/assets/broader-comparison-calculated-flow-2026-10-16.svg)

La siguiente gráfica muestra el tiempo del turno y los tokens de entrada del agente, con la semilla de Gennie Memory ya lista. Incluye instrucciones y caché.

![Tiempo del turno de Codex y tokens de entrada con cada motor](docs/evaluation/assets/broader-comparison-codex-2026-10-16.svg)

## Cómo funciona

1. **Busca** recuerdos relacionados con la consulta dentro del ámbito indicado: usuario, agente, proyecto o conversación.
2. **Selecciona** los candidatos que sirven para responder, reutilizando una revisión compatible o consultando al clasificador.
3. **Prepara** una semilla breve con el contenido seleccionado y sus referencias, para que el agente la use como contexto.

El núcleo está escrito en Rust y los recuerdos se almacenan en SQLite. La interfaz actual es CLI/JSONL. La búsqueda predeterminada es por palabras clave; la búsqueda semántica sigue siendo experimental. La integración automática con agentes todavía está pendiente.

La CLI incluye `mem-write` para crear una memoria y `mem-write-batch` para escribir lotes de hasta 2.500 mediante la misma operación idempotente. Esta primera ruta es local y no ofrece todavía autenticación multiusuario, ACL ni resolución de duplicados. [Contrato de escritura](docs/architecture/memory-design.md#21-contrato-de-escritura).

Para contratos, almacenamiento, índices y ciclo de vida de los recuerdos, consulta el [funcionamiento técnico de la memoria](docs/architecture/memory-design.md). Para la distribución del código, consulta la [arquitectura vigente](docs/architecture/overview.md) y el [mapa de módulos](docs/architecture/modules.md).
