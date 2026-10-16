# Engram a través de Codex frente a iROMEM directo

Estado: medición realizada el 2026-10-15 en esta Mac con seis preguntas sintéticas emparejadas. Esta prueba corrige el alcance de la [comparación anterior](engram-comparison-v1.md), que solo cronometró la búsqueda local. Los [resultados y eventos por caso](../../outputs/iromem/evaluation/engram-codex-end-to-end-v1/) permiten auditar cuándo Codex llamó a Engram y cuándo entregó la respuesta.

Una [medición posterior](initial-seed-speed-and-tokens-v1.md) simuló el flujo distinto de entregar la semilla en el primer prompt de Codex, sin llamada MCP, y registró tiempo y tokens. Esta página conserva la medición original de Engram.

## Resultado principal

| Recorrido, seis preguntas | Tiempo hasta respuesta/semilla p50 | p95 | Positivos correctos | Negativo vacío |
| --- | ---: | ---: | ---: | ---: |
| Codex CLI → Engram MCP → respuesta de Codex | **9,49 s** | 13,94 s | 4/5 | 1/1 |
| iROMEM directo: Laya → híbrido → Decisions → semilla validada | **0,73 s** | 0,93 s | **5/5** | 1/1 |

En esta tanda pequeña, Codex tardó unas **13 veces más** en la mediana. Si se cuentan solo los cuatro positivos que ambos resolvieron, Codex tardó una mediana de **9,19 s** en responder. En los seis casos, el primer resultado de Engram llegó al proceso de Codex a los **7,95 s de mediana** desde el inicio; la respuesta final llegó a los 9,49 s. Las nueve llamadas MCP individuales duraron entre **2,2 y 9,4 ms** (mediana **5,0 ms**). El tiempo dominante fue la orquestación y generación de Codex antes y después de la herramienta, no SQLite de Engram.

Codex hizo respectivamente **1, 1, 1, 1, 3 y 2** búsquedas. En la quinta pregunta, sobre un supuesto abandonado en una conversación, tres búsquedas léxicas no localizaron el recuerdo y Codex respondió que no lo encontró. iROMEM sí entregó la semilla correcta. La sexta pregunta no tenía respuesta en el corpus y ambos se abstuvieron. Estos aciertos son observaciones de seis casos, no estimaciones de una tasa general.

## Qué se midió

- **Codex + Engram:** se inició una sesión nueva de `codex exec --json` por pregunta, con el CLI incluido en la app (`0.162.0-alpha.17.2`), modelo `gpt-6.1-sol`, configuración de usuario ignorada y solo el MCP `mem_search` de Engram 3.4.0 disponible. El directorio de trabajo era temporal, el sandbox de Codex era de solo lectura y el prompt prohibía archivos y shell. Se indicó el proyecto sintético correcto y se pidió responder usando Engram. El cronómetro comenzó antes de iniciar cada proceso; los eventos `item.started`, `item.completed` y el mensaje final se guardaron con tiempo monotónico. La respuesta final se cronometró cuando apareció el último `agent_message`; la salida completa del proceso tardó algo más. No se modificó la configuración MCP del usuario ni se inició un servicio con puerto. La [documentación oficial de OpenAI](https://developers.openai.com/codex/mcp) describe esta conexión MCP local por stdio.
- **iROMEM directo:** se mantuvieron abiertos el encoder Laya en MPS y la sesión Rust sobre la base sintética de 2.800 recuerdos. Cada cronómetro empezó al recibir la pregunta y terminó tras generar el vector en vivo, buscar con FTS5 + coseno/RRF, llamar directamente a OpenRouter Decisions con el perfil `direct` y hasta veinte candidatos, y validar/componer la semilla. El modelo Laya tardó **2,80 s** en cargar y **0,34 s** en calentarse, ambos costes previos a los seis cronómetros; si se inicia desde cero para una sola pregunta, deben añadirse. El tiempo de decisión remota fue p50 **663 ms**; codificación p50 **18,6 ms**. El coste informado por las seis llamadas a Decisions fue **US$0,0003175**. La cuenta ChatGPT usada por Codex no informó un coste monetario por consulta en estos eventos.
- Las seis preguntas proceden de la partición de desarrollo de `general-memory-v1`: cinco positivas y una negativa, con ámbitos `user`, `project`, `conversation` y `agent`. Son las mismas preguntas y recuerdos sintéticos en ambos recorridos. Engram representó cada identidad/ámbito como proyecto; iROMEM recibió el ámbito estructurado. No se usaron datos personales reales.

## Límites de la interpretación

Esta es una **comparación del recorrido solicitado**, no una prueba idéntica de motores de búsqueda. Codex usó un modelo general para decidir herramientas y redactar la respuesta; iROMEM usó Decisions como clasificador acotado y devolvió una semilla con ID, revisión y fuente declarada. El Codex CLI inició un proceso nuevo por pregunta; iROMEM reutilizó Laya y SQLite ya cargados. El CLI aislado tampoco reproduce exactamente una conversación larga en la app de Codex, con más herramientas e historial. Las latencias de red y del modelo pueden cambiar. Para comparar operación continua habría que medir sesiones persistentes de ambos en el mismo host y repetir más preguntas; los datos actuales sí muestran el coste observado de pasar por Codex en este experimento.
