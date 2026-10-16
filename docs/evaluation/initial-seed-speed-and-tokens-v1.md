# Velocidad y tokens de la semilla inicial: evidencia y límites

Estado: **piloto histórico** sintético medido el 2026-10-15 en esta Mac, con figuras [originales de velocidad](assets/speed-comparison-v1.svg) y [aciertos](assets/accuracy-comparison-v1.svg). Simuló la entrega de una semilla **híbrida** en el primer prompt de Codex; no instaló un hook. La [decisión posterior](../decisions/0010-keyword-search-default.md) cambió el método predeterminado a palabras clave. Su medición equivalente actual, ejecutada desde Rust, está en [rust-migration-v1.md](rust-migration-v1.md); no se deben mezclar sus cifras.

## Dos bancos, dos preguntas distintas

**Búsqueda local de candidatos.** Una base de 2.800 recuerdos sintéticos y 540 consultas finales congeladas (460 positivas, 80 negativas) compara iROMEM híbrido, iROMEM por palabras clave y Engram 3.4.0. El vector de consulta del híbrido ya estaba calculado. Las cifras son tiempo de ida y vuelta al proceso local, no tiempo de respuesta del agente. La [comparación detallada](keyword-preflight-v1.md) conserva resultados por caso.

| Búsqueda local | p50 / p95 | Recuerdo correcto entre 20, positivos |
| --- | ---: | ---: |
| iROMEM híbrido natural | 11,93 / 12,54 ms | **459/460** |
| iROMEM FTS5 de palabras clave, optativo | **5,23 / 5,44 ms** | 452/460 |
| Engram `any` con palabras clave, corrida previa | 5,52 / 6,48 ms | 454/460 |

iROMEM híbrido no es más rápido que Engram en la búsqueda local aislada. La opción léxica está en el mismo rango de tiempo, pero pierde siete recuerdos adicionales frente al híbrido. No se activó como política base.

**Recorrido de seis preguntas.** Se usaron cinco preguntas positivas y una negativa de la partición de desarrollo, con recuerdos sintéticos de usuario, proyecto, conversación y agente. La [corrida Codex + Engram](engram-codex-end-to-end-v1.md) inició un proceso Codex nuevo por pregunta y midió hasta la respuesta. La [corrida iROMEM directa](../../outputs/iromem/evaluation/engram-codex-end-to-end-v1/direct-iromem.json) usó Laya, búsqueda híbrida, Decisions y composición validada, con los procesos calientes, hasta producir la semilla. Después, una [corrida Codex con semilla inicial](../../outputs/iromem/evaluation/codex-initial-seed-simulation-v1/summary.json) inició el mismo Codex CLI y modelo con el texto de la semilla incluida en el primer prompt, sin MCP ni herramientas. Las selecciones de recuerdos de iROMEM quedaron congeladas antes de ese turno.

La simulación histórica usó el CLI incluido en la aplicación (`0.162.0-alpha.17.2`), `gpt-6.1-sol`, una sesión nueva y efímera por pregunta, configuración del usuario ignorada y sandbox de solo lectura. El prompt incluyó pregunta, semilla y una instrucción breve de responder sin herramientas. El runner antiguo guardó los eventos de cada turno y rechazó corridas con herramientas; fue retirado. La prueba Engram tenía MCP disponible y una instrucción distinta para consultarlo; por ello la comparación incluye tanto el cambio de entrega de memoria como el cambio de herramientas/contexto.

| Recorrido medido | p50 / p95 | Resultado de seis casos |
| --- | ---: | --- |
| Codex + Engram MCP → respuesta | 9,49 / 13,94 s | 4/5 positivos y 1/1 negativo |
| iROMEM híbrido directo → semilla, **sin Codex** | 0,73 / 0,93 s | 5/5 positivos y 1/1 negativo |
| Codex con semilla preparada en el prompt inicial → respuesta, **sin contar preparación** | 5,12 / 6,61 s | 5/5 positivos y 1/1 negativo |

La suma **por caso** del tiempo observado para preparar la semilla y el tiempo observado de Codex con esa semilla da p50 **5,79 s** y p95 **7,42 s**. Es una estimación aritmética de dos corridas, **no una traza punta a punta integrada**. Frente a los 9,49 s de Codex + Engram, esta muestra sugiere una ventaja de unos 3,70 s en la mediana del recorrido completo estimado. No demuestra esa ventaja en una integración real ni en conversaciones largas. La semilla se preparó con el híbrido natural, que conservó el recuerdo difícil que la ruta léxica optativa perdió. Laya tardó además 2,80 s en cargar y 0,34 s en calentarse antes de los cronómetros de la corrida directa; una primera consulta fría tendría ese coste adicional.

## Tokens observados

Los proveedores informaron estos tokens durante las seis preguntas. «Entrada total Codex» incluye instrucciones del sistema, esquemas de herramientas cuando existen, contexto, mensajes y resultados de herramientas; **no es el tamaño de la memoria**. El campo de caché es un subconjunto de entrada total. Los totales de Codex con y sin MCP no son una atribución causal limpia solo a Engram.

| Operación | Entrada p50 | Entrada en caché p50 | Entrada fuera de caché p50 | Salida p50 |
| --- | ---: | ---: | ---: | ---: |
| Decisions para elegir recuerdo en iROMEM híbrido | **524** | No informado | No informado | **0** en la respuesta tipada |
| Codex + Engram MCP, turno completo | 49.323 | 43.392 | 7.645 | 112 |
| Codex con semilla inicial, turno completo | 15.598 | 12.288 | 3.310 | 17,5 |

La semilla positiva inyectada tenía entre **160 y 196 bytes UTF-8**, o **15 a 19 palabras separadas por espacios**. Una [corrida de control sin semilla](../../outputs/iromem/evaluation/codex-no-seed-control-v1/summary.json) mantuvo la pregunta y el formato del prompt. Para los cinco positivos, añadir la semilla aumentó la entrada total informada por Codex en **40–51 tokens**, mediana **42**. Ese incremento es la medición más cercana al **coste marginal del texto de memoria** bajo este formato; no equivale a la totalidad de tokens de iROMEM. El negativo no recibió semilla y la diferencia de dos tokens entre corridas muestra variación de instrumentación/contexto, por lo que no se interpreta como coste de memoria. El codificador Laya no registró aquí el número de tokens de su entrada local.

La ruta léxica optativa consumió 518 tokens p50 de entrada en Decisions en una segunda corrida de seis casos, pero omitió un positivo; sus [datos de tokens](../../outputs/iromem/evaluation/keyword-preflight-v1/direct-keywords-six-tokens.json) no justifican sustituir el híbrido.

## Qué sigue sin estar medido

- Tiempo real de un hook que prepare e inyecte la semilla antes de un turno de Codex, incluidos sus errores, permisos, procesos fríos y conversación acumulada.
- Ahorro causal de tokens frente a Engram con igual carga de contexto y herramientas: las configuraciones del modelo difieren porque una corrida tenía MCP y la otra no.
- Resultados en preguntas y recuerdos reales. Seis preguntas templáticas no bastan para estimar p95, calidad general ni tasa de errores.

Las figuras históricas usaron medianas de las observaciones guardadas y separaron candidatos de respuesta final. Los cinco aciertos de Codex con Engram y seis con semilla inicial proceden de revisar las respuestas guardadas contra el hecho sintético esperado. El generador Python fue retirado; las figuras originales y sus datos quedan congelados. Las [figuras vigentes](rust-migration-v1.md) se regeneran en Rust. No se usaron recuerdos reales ni se modificó la configuración de Codex del usuario.

## Estado de reproducción

Los resultados por caso incluyen tiempos y contadores del proveedor. Los comandos Python originales ya no existen. Para repetir el experimento vigente con el runner Rust y salidas nuevas, consulta [el contrato de evaluación](contract-v1.md) y [el informe actual](rust-migration-v1.md); la simulación de Codex requiere la autenticación existente y no modifica su configuración.
