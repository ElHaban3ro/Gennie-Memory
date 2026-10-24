# Consulta rápida por palabras clave frente al híbrido y Engram

Estado: experimento sintético ejecutado el 2026-10-15 en esta Mac. Este informe conserva la política de su fecha: entonces `query_mode: "keywords"` era optativo. La [decisión posterior](../decisions/0010-keyword-search-default.md) lo convirtió en predeterminado; las mediciones y conclusiones de esta corrida no se reescriben. [Resultados por caso](../../outputs/iromem/evaluation/keyword-preflight-v1/).

## Implementación

La nueva modalidad extrae palabras clave de **la pregunta entrante** antes de consultar FTS5. Conserva los documentos indexados y los filtros de ámbito, entidad, revisión, estado y fecha. Usa términos Unicode minúsculos de al menos tres caracteres, sin duplicados ni las palabras funcionales fijadas para la comparación Engram, unidos por OR. No se añadieron etiquetas a los recuerdos ni se modificó SQLite. El texto original de la pregunta sigue disponible para Decisions.

El experimento también simuló una política de preselección: busca por palabras clave y usa el híbrido si la lista está vacía. Fue una simulación del orquestador con vectores ya calculados; **esa regla no se activó**. En final no hubo listas vacías, aunque ocho positivos perdieron el recuerdo correcto entre veinte candidatos. Una lista no vacía no acredita cobertura. El script histórico fue retirado.

## Búsqueda local: 2.800 recuerdos, 540 preguntas finales

Son 460 positivas y 80 negativas, límite veinte. El binario anterior y el nuevo leyeron la misma base, con un proceso caliente y llamadas seriales por `session`. La latencia es ida y vuelta cliente–sesión. El vector del híbrido ya estaba calculado. Engram v3.4.0 es la [corrida local previa](engram-comparison-v1.md) con las mismas preguntas y recuerdos, pero otro proceso y almacenamiento; no se repitió en esta corrida.

| Ruta | Correcto top 1 | Top 20 | Negativos con candidatos | p50 / p95 búsqueda |
| --- | ---: | ---: | ---: | ---: |
| Gennie Memory anterior, híbrido natural | 241/460 | **459/460** | 80/80 | 11,93 / 12,54 ms |
| Gennie Memory nuevo, FTS5 palabras clave | 409/460 | 452/460 | 80/80 | **5,23 / 5,44 ms** |
| Gennie Memory nuevo, híbrido con palabras clave | 244/460 | 456/460 | 80/80 | 11,36 / 11,74 ms |
| Engram `any` con palabras clave, corrida previa | **426/460** | 454/460 | 80/80 | 5,52 / 6,48 ms |

En desarrollo, el híbrido anterior cubrió 115/115 y la ruta rápida 112/115; los tres fallos tenían listas no vacías. El modo rápido ahorró 6,70 ms de mediana de búsqueda respecto al híbrido anterior, pero perdió siete positivos en top veinte en final. Engram y el nuevo FTS5 de Gennie Memory quedan en el mismo orden de milisegundos; una diferencia de 0,30 ms entre corridas no establece superioridad estable.

La pregunta ilustrativa `Hola, ¿cómo me llamo?` produjo **cero candidatos léxicos** en el ámbito sintético `synthetic_user_001` de esta base. Eso no prueba ausencia del recuerdo: las palabras indexadas son `nombre completo`. Esta frase exacta no está en la cohorte de seis casos ni se midió hasta respuesta; requeriría el respaldo semántico y una decisión posterior.

## Recorrido completo: seis preguntas sintéticas

Se reutilizaron los seis IDs del [piloto Codex–Engram](engram-codex-end-to-end-v1.md): cinco positivos y un negativo. Gennie Memory nuevo buscó con palabras clave, llamó a OpenRouter Decisions `direct` y validó/compuso la semilla mediante `compose-seed`; no generó vector porque las seis listas léxicas fueron no vacías. [Resultado nuevo por caso](../../outputs/iromem/evaluation/keyword-preflight-v1/direct-keywords-six.json). Las latencias de proveedor son variables entre campañas.

| Recorrido | p50 / p95 hasta semilla o respuesta | Positivos correctos | Negativo vacío |
| --- | ---: | ---: | ---: |
| Gennie Memory anterior, Laya + híbrido + Decisions, corrida previa | 0,73 / 0,93 s | **5/5** | 1/1 |
| Gennie Memory nuevo, palabras clave + Decisions | **0,53 / 1,41 s** | 4/5 | 1/1 |
| Codex CLI + Engram MCP, corrida previa | 9,49 / 13,94 s | 4/5 | 1/1 |

El positivo perdido por Gennie Memory nuevo trata un supuesto abandonado: el recuerdo esperado no apareció entre sus veinte candidatos. Codex + Engram tampoco lo localizó en el piloto anterior. La búsqueda local completa nueva tuvo p50 8,05 ms frente a 24,20 ms del piloto híbrido anterior; se evitó la codificación de consulta, antes p50 18,59 ms. Decisions tuvo p50 504,63 ms en la corrida nueva frente a 663,38 ms en la anterior. Así, la diferencia observada de 0,20 s en el total **no se atribuye íntegramente a la búsqueda**. Seis casos no estiman una tasa general de acierto ni un p95 estable. Las seis llamadas nuevas informaron US$0,0003133.

## Semilla entregada a Codex antes del turno

Gennie Memory **no tiene hook ni herramienta MCP para Codex**. El escenario relevante es preparar la semilla antes de iniciar Codex y adjuntarla al primer prompt, sin una decisión posterior de buscar por MCP. Una [simulación posterior con el mismo Codex CLI y seis preguntas sintéticas](initial-seed-speed-and-tokens-v1.md) midió **5,12 s** de mediana hasta la respuesta de Codex con la semilla híbrida inicial. La suma por caso con los tiempos previos de preparación de esa semilla da **5,79 s de mediana**, calculada entre corridas separadas; aún no es una traza de integración. La sustitución aritmética de milisegundos de una llamada MCP, considerada inicialmente, describe otra arquitectura y no predice la inyección inicial.

## Decisión y reproducción

Esta fue la decisión **de aquel experimento**. Una decisión posterior fijó la ruta rápida como predeterminada, aceptando su menor cobertura; la [medición Rust vigente](rust-migration-v1.md) evalúa esa política. El salto semántico automático por lista vacía continúa descartado.

```sh
cargo fmt --all --check
cargo test --offline --workspace
cargo run --offline -p gennie-memory-eval -- validate-general-memory tests/fixtures/general-memory-v1
```

La comparación histórica original ya no se regenera con su script retirado. Huellas SHA-256 de esa corrida: binario anterior `24652c759b8237ea679de8ffb1148d1528eab74fc87561a5e93c93fb774ad11b`; binario nuevo `f91056f060022f85341cd6ae9e73d02e063c69fdfb2a917e7a6b3c85d2b5aeef`; base `fd2eb78aff4021660ac9bf0f1972e14014cf17e9c64b9b7b6009aa4fc2a0eae9`; preguntas finales `75ca1feaf830a30e84cf8ccb01bb1c5d55535ad50b8b6e4e305b35bc6ff8ff06`.
