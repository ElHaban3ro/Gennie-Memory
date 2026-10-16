# Comparación ampliada de memorias persistentes

Estado: búsqueda local ejecutada el 2026-10-16 sobre la cohorte sintética congelada 014–017. El [paquete de datos](../../outputs/iromem/evaluation/broader-comparison-2026-10-16/) conserva consultas, manifiestos, resultados por caso y resúmenes. Las cifras de iROMEM y Engram proceden de la [campaña original](full-comparison-2026-10-16.md), sin volver a ajustar sus rutas.

## Rendimiento observado

Se cargaron los mismos 2.800 recuerdos y se consultaron las mismas 540 preguntas (460 positivas, 80 negativas), en cuatro órdenes congelados. Top 1 y Top 20 cuentan si aparece el ID esperado entre los candidatos; la lista vacía en negativos mide solamente si el buscador se abstiene. El p50 y p95 son las medianas de los cuatro percentiles de corrida, no percentiles agrupados.

| Motor y ruta | Top 1 / 460 | Top 20 / 460 | Negativas vacías / 80 | p50 | p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| iROMEM FTS5 | 420 (91,30 %) | **457 (99,35 %)** | 0 | **6,93 ms** | **7,77 ms** |
| Engram 3.4.0 MCP | **427 (92,83 %)** | 456 (99,13 %) | 0 | 7,22 ms | 10,71 ms |
| Lodis 0.6.0 híbrido MCP | 47 (10,22 %) | 389 (84,57 %) | 0 | 19,67 ms | 40,94 ms |
| Basic Memory, búsqueda local | 276 (60,00 %) | 280 (60,87 %) | **80** | 24,12 ms | 127,72 ms |

![Aciertos en la cohorte 014–017](assets/broader-comparison-accuracy-2026-10-16.svg)

![Latencia de búsqueda](assets/broader-comparison-latency-2026-10-16.svg)

| P95 por corrida | 1 | 2 | 3 | 4 |
| --- | ---: | ---: | ---: | ---: |
| iROMEM | 7,75 | 7,49 | 7,79 | 10,73 ms |
| Engram | 15,55 | 8,27 | 9,64 | 11,78 ms |
| Lodis | 21,04 | 176,35 | 54,57 | 27,30 ms |
| Basic Memory | 436,76 | 107,09 | 148,36 | 23,81 ms |

![P95 de cada corrida](assets/broader-comparison-p95-2026-10-16.svg)

**Comparabilidad:** iROMEM, Engram y Lodis incluyen ida y vuelta por un proceso persistente y serialización; Basic Memory se llamó por su API ASGI dentro del proceso porque su CLI `search-notes` falló en este entorno. Su latencia incluye API y serialización, pero no el mismo transporte ni arranque MCP. La consulta de Basic Memory usa palabras clave normalizadas y filtros exactos de ámbito. Lodis usa dominio exacto y búsqueda híbrida con 2.800 embeddings persistidos; su salida ordenada contiene 20 candidatos. Por ello los números comparan estas configuraciones y este banco, no la velocidad intrínseca de los productos. Las cuatro repeticiones de cada motor usan los mismos casos: no son cuatro muestras de exactitud independientes.

Basic Memory devuelve lista vacía para todas las preguntas negativas, mientras los demás motores devuelven candidatos aunque no haya respuesta válida. Eso **no equivale** a la abstención de la semilla de iROMEM: iROMEM + Decisions acertó 441/460 positivos y dejó vacías 80/80 negativas en su [campaña separada](full-comparison-2026-10-16.md). No hay clasificadores de semilla equivalentes para Basic Memory ni Lodis en este ensayo.

## Preparación y trazabilidad

- Basic Memory: revisión `a72751c5d626380dedeed30f55f7a80ba3dc710d`; 2.800 notas Markdown generadas con ID, texto original, síntesis y ámbito, indexadas en un proyecto aislado. Adaptador: [basic_memory_benchmark.py](../../tools/evaluation/basic_memory_benchmark.py). Los cuatro `search-basic-memory-*-summary.json` acompañan los resultados por caso.
- Lodis: revisión `bf3d89197321a10f6bb845528cfa567d6fd95021`; 2.800 entradas con síntesis, detalle original e ID, dominio por ámbito y 2.800/2.800 embeddings persistidos. El checkout temporal solo cambió la ruta de datos a un directorio aislado. Adaptador: [lodis_benchmark.py](../../tools/evaluation/lodis_benchmark.py). Los cuatro `search-lodis-semantic-*-summary.json` y el mapa de IDs están en el paquete.
- El primer import de Lodis tenía cero embeddings y cayó a búsqueda léxica; sus salidas, con etiqueta original incorrecta `lodis_hybrid`, se conservan en [diagnostic-lodis-no-embeddings](../../outputs/iromem/evaluation/broader-comparison-2026-10-16/diagnostic-lodis-no-embeddings/) y **no entran** en la tabla.
- No hubo llamadas de pago nuevas para las cuatro campañas locales. El coste informado de Decisions y Jev de la campaña previa fue US$0,05130749 en conjunto; no se ha vuelto a medir.

## Otros candidatos investigados

| Herramienta | Ajuste al objetivo | Decisión para este banco |
| --- | --- | --- |
| [MCP Memory Service](https://github.com/doobidoo/mcp-memory-service) | Almacenamiento y búsqueda semántica persistente por MCP | `retrieve_memory` no expone un filtro de ámbito exacto comparable; requeriría múltiples almacenes o cambiar el protocolo del banco. |
| [Mem0 OSS](https://github.com/mem0ai/mem0) | Memoria persistente con búsqueda y ámbitos | Requiere configurar proveedor de embeddings y, según ruta, LLM; la evaluación precisa fijar ese perfil y sus costes antes de comparar. Su [MCP gestionado](https://github.com/mem0ai/mem0/blob/main/docs/platform/mem0-mcp.mdx) es otra ruta. |
| [Graphiti](https://github.com/getzep/graphiti) | Memoria temporal en grafo | Requiere servicio de grafo y modelos para extracción/embeddings; el contrato de ingestión genera hechos a partir del texto y no conserva necesariamente una correspondencia 1:1 con los 2.800 IDs. |
| [Supermemory](https://github.com/supermemoryai/supermemory) | API y MCP de memoria persistente | Su [despliegue propio](https://github.com/supermemoryai/supermemory/blob/main/apps/docs/self-hosting/quickstart.mdx) exige varios servicios; aún falta una configuración local equivalente y medir ingestión e indexación. |
| [Letta](https://github.com/letta-ai/letta) | Memoria de agentes persistentes | Orientado a agentes completos y bloques de memoria; no ofrece directamente el mismo contrato de recuperación de 20 IDs con ámbito por consulta. |
| [LangMem](https://github.com/langchain-ai/langmem/blob/main/docs/docs/guides/memory_tools.md) | Herramientas para memoria de agentes | Biblioteca que necesita un almacén y una configuración de agente; no es un motor aislado equiparable sin fijar esos componentes. |

Esta es una selección de alternativas pertinentes, no una afirmación de cobertura de todo el mercado. Los candidatos no medidos **no tienen cifra de rendimiento comparable**; no se les asigna posición en la tabla.

## Turno Codex y límites

Con la configuración MCP de aprobación efímera, Lodis y Basic Memory completaron dos rondas de los mismos seis casos preseleccionados. Los dos respondieron correctamente los cinco positivos y la negativa en cada ronda, con una llamada MCP completada por caso. El intento inicial de Lodis, rechazado por aprobación no interactiva, permanece [separado](../../outputs/iromem/evaluation/broader-comparison-2026-10-16/codex-lodis-1/) y no cuenta. Sus [rondas válidas](../../outputs/iromem/evaluation/broader-comparison-2026-10-16/codex-lodis-valid-1/) y [repetición](../../outputs/iromem/evaluation/broader-comparison-2026-10-16/codex-lodis-valid-2/) contienen respuestas, llamadas y tokens informados.

Basic Memory se omitió inicialmente porque su CLI de búsqueda local falló y el primer piloto Codex no efectuó ninguna llamada MCP. En el diagnóstico, `search_notes` sí respondió al fijar proyecto, `search_type="text"`, filtros exactos y aprobación MCP efímera. La disponibilidad de la herramienta resultó intermitente durante el arranque; el piloto válido marcó el servidor como obligatorio y permitió hasta 60 s de inicialización. Las rondas [1](../../outputs/iromem/evaluation/broader-comparison-2026-10-16/codex-basic-memory-valid-1/) y [2](../../outputs/iromem/evaluation/broader-comparison-2026-10-16/codex-basic-memory-valid-2/) tienen seis llamadas completadas cada una. Los intentos incompletos están en directorios `diagnostic-basic-memory-*` y no entran en las cifras.

| Turno Codex de seis casos | Ronda 1 | Ronda 2 | Entrada p50 informada |
| --- | ---: | ---: | ---: |
| iROMEM, semilla ya entregada | **4,79 s** | **5,18 s** | 17.005,5 / 17.005,5 tokens |
| Engram MCP | 9,97 s | 11,34 s | 52.859 / 52.885,5 tokens |
| Lodis MCP | 12,20 s | 13,11 s | 52.717,5 / 52.715 tokens |
| Basic Memory MCP | 15,36 s | 15,23 s | 37.677,5 / 37.673,5 tokens |

![Tiempo del turno Codex y tokens de entrada de cuatro rutas](assets/broader-comparison-codex-2026-10-16.svg)

La preparación de semilla iROMEM + Decisions tomó 0,37 s de mediana en esos seis casos; la **suma calculada por caso** de preparación y turno fue 5,20 y 5,80 s por ronda. Los turnos de Engram, Lodis y Basic Memory incluyen arranque MCP, búsqueda y respuesta. Basic Memory usó una instrucción explícita de llamar `search_notes` incluso en el negativo, y su búsqueda MCP recibió las mismas palabras clave normalizadas y filtros de ámbito que su medición local. Son vías de entrega y prompts distintos; la suma de iROMEM combina ejecuciones separadas. Ninguna cifra mide desde el mensaje del usuario hasta respuesta con un hook iROMEM instalado. El arranque MCP y la variación del modelo impiden atribuir los 15 s de Basic Memory a su búsqueda local de 24 ms.

![Suma calculada de iROMEM y turnos MCP de Engram, Lodis y Basic Memory](assets/broader-comparison-calculated-flow-2026-10-16.svg)

La [figura de preparación de semilla](assets/full-comparison-seed-preparation-2026-10-16.svg) sigue representando únicamente iROMEM + Decisions y Jev. No hay clasificador de semilla equivalente para los motores nuevos, ni coste monetario informado por Codex. El banco usa preguntas sintéticas con ámbito conocido y plantillas emparentadas con cohortes previas. No mide escritura continua, conflictos, fuentes verificadas, tamaño de índice mayor, variación entre máquinas ni respuestas humanas. Los resultados dependen de adaptadores documentados; no se deben extrapolar como clasificación general de producto.
