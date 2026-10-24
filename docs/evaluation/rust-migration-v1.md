# Medición Rust de la ruta rápida y de la semilla inicial

Estado: corrida sintética del 2026-10-15/16 en esta Mac. La orquestación, los cálculos de recuperación y el render de las figuras se ejecutaron con `gennie-memory-eval` en Rust. Las [consultas exactas, resultados por caso y manifiestos](../../outputs/iromem/evaluation/rust-migration-v1/) están versionados; los intentos descartados no forman parte de las cifras. Los vectores heredados dentro del JSON de consultas se conservaron por huella, pero la ruta rápida no los lee.

## Banco y resultados

El banco final contiene **2.800 recuerdos** en 100 ámbitos sintéticos y **540 preguntas** de sujetos reservados: 460 positivas y 80 negativas. Gennie Memory usó la ruta predeterminada FTS5 por palabras clave, máximo veinte candidatos, y `recall` completo en Rust con Decisions o Jev por OpenRouter. Engram 3.4.0 usó `mem_search`, `match_mode=any`, palabras clave y el mismo máximo; se importó el mismo contenido sintético a un almacén aislado. El título fue la síntesis y el cuerpo el texto original. Engram no recibió las condiciones ni fuentes declaradas de Gennie Memory, por lo que la comparación local es de candidatos, no de contratos de memoria equivalentes.

| Medición final | Gennie Memory rápido | Engram MCP | Gennie Memory + Decisions | Gennie Memory + Jev |
| --- | ---: | ---: | ---: | ---: |
| Recuerdo correcto entre 20 candidatos | 452/460 | 456/460 | 452/460 | 452/460 |
| Semilla exacta en positivos | — | — | 424/460 | 400/460 |
| Negativos con semilla vacía | — | — | 80/80 | 80/80 |
| Mediana de búsqueda local | 5,72 ms | 6,02 ms | — | — |
| Mediana de consulta → semilla | — | — | 376,07 ms | 381,31 ms |
| p95 de consulta → semilla | — | — | 557,50 ms | 458,38 ms |
| Tokens de entrada informados por el clasificador, mediana | — | — | 535 | 938 |

La ventaja de búsqueda de Gennie Memory es **0,30 ms en esta corrida**, mientras que Engram encontró cuatro positivos más entre los veinte candidatos. No se infiere una superioridad estable por una diferencia tan pequeña ni se equiparan candidatos con semillas finales. Los dos clasificadores dejaron los 80 negativos vacíos; Decisions acertó 24 positivos más que Jev. Los proveedores informaron un total de US$0,0283857 para Decisions y US$0,02096829 para Jev en las 540 llamadas de cada uno. El campo de tokens de salida llegó como cero en estas respuestas y no permite estimar consumo completo; los archivos por caso conservan los valores informados.

## Seis turnos de Codex

Se repitieron seis preguntas de desarrollo emparejadas: cinco positivas y una negativa, cada una en un proceso efímero del CLI de Codex **0.162.0-alpha.17.2** con `gpt-6.1-sol`, configuración del usuario ignorada y sandbox de solo lectura. En el primer camino, el runner Rust tomó la semilla del diario `recall` y la incluyó en el prompt inicial, sin herramientas. En el segundo configuró únicamente Engram MCP con su almacén sintético aislado. Un tercer camino sin semilla permitió medir los tokens añadidos al prompt.

| Camino de Codex | Respuesta correcta | Tiempo a respuesta p50 | Tokens de entrada informados p50 | Llamadas MCP |
| --- | ---: | ---: | ---: | ---: |
| Semilla de Gennie Memory ya incluida | 4/5 positivos + 1/1 negativo | 5,97 s | 16.912 | 0 |
| Codex → Engram | 4/5 positivos + 1/1 negativo | 11,07 s | 52.586 | 9 en seis turnos |

La quinta pregunta positiva no obtuvo la semilla correcta con la ruta rápida; Codex respondió que faltaba el dato. Engram hizo tres búsquedas en ese caso y tampoco lo encontró. Los otros cuatro positivos y el negativo se revisaron contra el hecho sintético esperado y las respuestas guardadas. En los cuatro casos que realmente recibieron texto de semilla, este añadió **47 tokens de entrada de mediana** frente al control emparejado; los dos casos sin semilla añadieron tres por la frase de ausencia. Los totales de tokens de Codex incluyen contexto e instrucciones en caché y no aíslan el coste marginal de Engram. Las instrucciones también difieren porque uno recibe contexto y el otro debe usar MCP.

**La entrega automática de Gennie Memory a Codex todavía no existe.** Los 5,97 s son una ejecución real de Codex con la semilla incluida por el runner Rust. `gennie-memory-eval paired-flow-report` emparejó esos seis turnos con sus preparaciones Rust: la mediana de preparación fue **0,49 s** y la mediana de las **seis sumas por caso** fue **6,72 s**, frente a 11,07 s medidos de Codex→Engram. La [suma por caso](../../outputs/iromem/evaluation/rust-migration-v1/paired-flow-summary.json) es calculada a partir de dos ejecuciones; no se presenta como tiempo de integración instalada. Los seis turnos son un piloto de flujo, no una estimación de acierto general.

## Método reproducible y límites

- `gennie-memory-eval search-benchmark` mantuvo abierto un proceso `gennie-memory session`, calentó una consulta excluida y cronometró 540 búsquedas seriales. `gennie-memory-eval engram-benchmark` importó el fixture con Rust, mantuvo abierto Engram MCP, calentó una consulta y cronometró las mismas 540 solicitudes seriales. `gennie-memory-eval retrieval` calculó ambas coberturas y percentiles. Los tiempos incluyen IPC y serialización, con condiciones locales y orden de corrida observados; no son microbenchmarks de SQLite puro.
- `gennie-memory-eval campaign` envió las 540 preguntas a `recall` por una sesión Rust persistente para cada proveedor, guardando un diario sincronizado antes y después de cada llamada. Ambas campañas acabaron sin respuestas degradadas ni filtración de IDs prohibidos. `campaign-report` calculó los aciertos, tokens informados y latencias por fase.
- `gennie-memory-eval codex-prompt` y `codex-engram` guardaron los eventos JSONL de cada turno; `paired-flow-report` sumó por identificador la preparación y el turno con semilla, sin sumar medianas independientes. El CLI de Codex de esta aplicación ejecutó las llamadas; Rust controló procesos, entradas, límites y cronómetros. Una primera prueba con un CLI independiente desactualizado fue rechazada antes de generar respuesta. Una primera corrida Codex→Engram con el almacén predeterminado se descartó al detectar que el MCP no recibía `ENGRAM_DATA_DIR`; se repitió con la ruta explícita y un caso previo válido.
- El archivo de preguntas finales tiene SHA-256 `75ca1feaf830a30e84cf8ccb01bb1c5d55535ad50b8b6e4e305b35bc6ff8ff06`; el fixture de recuerdos, `ee55707918018aece6f687759ef31eac0adc37d85542fae0d2e312bd136c6835`; la base Gennie Memory, `fd2eb78aff4021660ac9bf0f1972e14014cf17e9c64b9b7b6009aa4fc2a0eae9`; y el binario Gennie Memory, `607f1f8de28cc41380af2b74cfc4940526bbb9530ff98cf4b87b51b08e8b0597`. El [release oficial de Engram 3.4.0](https://github.com/Gentleman-Programming/engram/releases/tag/v3.4.0) para macOS arm64 se descargó a `/tmp` y su archivo coincidió con el SHA-256 publicado `d0aa59c9a83d17ae3fc7f5dd31fdab6a26fd8c00fc5e692195753cd0719294f7`. No se tocó la base personal de Engram.
- Las figuras históricas se generan con `gennie-memory-eval render-charts` desde los seis informes JSON de este paquete. La [figura de tiempo](assets/speed-comparison-rust.svg) compara Gennie Memory y Engram en búsqueda y respuesta de Codex, con ejes desde cero y escalas separadas. La [figura de aciertos](assets/accuracy-comparison-rust.svg) mide recuperación y clasificación, no respuestas de Codex. No se extrapola el piloto de seis turnos a las 540 preguntas finales.

Los resultados históricos de búsqueda híbrida y Laya siguen documentados en sus informes; esta corrida evalúa la ruta rápida vigente. El banco sintético, los ámbitos explícitos, la red y la carga de esta Mac limitan la generalización a conversaciones reales.
