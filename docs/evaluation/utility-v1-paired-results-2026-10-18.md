# Selección por utilidad v1: comparación emparejada

El 2026-10-18, `utility_v1` recuperó **once evidencias útiles adicionales sin perder ninguna seleccionada por la base emparejada** en 64 preguntas sintéticas conocidas. Las refutaciones pasaron de 8/16 a 15/16 y los contextos parciales de 12/16 a 16/16; se conservaron las 16/16 respuestas y las 16/16 abstenciones por ausencia. **Pasa el filtro de desarrollo fijado previamente; no se promueve a predeterminada ni se declara validación general.** Permanece una refutación omitida.

## Cambio y método

La política es explícita mediante `selection_policy: "utility_v1"` en `recall`. Cambian la instrucción y la descripción de `none`: admiten respuesta, corrección de premisa o parte concreta de la información pedida, exigen preservar negación, tiempo y estado, y rechazan coincidencia temática sola o hechos inferidos. Búsqueda, recuerdos, condiciones, proveedor y composición siguen iguales. Se hace una sola elección por consulta; no hay gate ni otra llamada. Como se cambiaron instrucción y texto de `none` juntos, el experimento no identifica cuál de ambos componentes explica cada cambio.

Se respetó el [protocolo anterior a la campaña](utility-v1-paired-protocol-2026-10-18.md): 128 operaciones en un proceso `session`, con cada par adyacente y orden alternado. Decisions, `keywords`, límite 20, SQLite temporal y los mismos 64 casos de `conversation-evidence-v1`; no se transmiten anotaciones al proveedor. Los candidatos de ambas políticas fueron idénticos en cada par. No se cambiaron etiquetas, prompt o modelo después de observar resultados y no se repitieron casos. El [preflight](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/preflight.json) conserva hashes de protocolo, rúbrica, datos, código y binarios anteriores a las llamadas; la [copia de instrucciones](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/instructions.json) fija el texto exacto.

## Selección observada

| Relación | Base emparejada | `utility_v1` | Cambio |
| --- | ---: | ---: | ---: |
| Respuesta completa: ID permitido | 16/16 | 16/16 | 0 |
| Refutación: ID permitido | 8/16 | 15/16 | +7 |
| Contexto parcial: ID permitido | 12/16 | 16/16 | +4 |
| Ausencia: semilla vacía | 16/16 | 16/16 | 0 |
| Selecciones de ID no permitido | 0 | 0 | 0 |
| Casos degradados | 0 | 0 | 0 |

Las once diferencias fueron abstención de la base frente a selección del ID anotado por la variante. No hubo pérdida de evidencia previamente seleccionada, selección no permitida ni degradación nueva: no se activó ningún bloqueo. La base nueva reprodujo exactamente los IDs y abstenciones de la [línea base histórica del 2026-10-17](conversation-evidence-v1-results-2026-10-17.md), no solo sus totales. La comparación relevante para el filtro es la emparejada de hoy. No se agrega una puntuación global que compense pérdidas entre categorías.

Las siete refutaciones recuperadas fueron `cev1q_035_03`, `cev1q_036_04`, `cev1q_037_03`, `cev1q_037_04`, `cev1q_038_04`, `cev1q_039_03` y `cev1q_041_03`. Los cuatro contextos parciales recuperados fueron `cev1q_037_05`, `cev1q_037_06`, `cev1q_040_05` y `cev1q_040_06`. El [reporte JSON](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/report.json) conserva pregunta, IDs y textos de semilla de cada transición.

La omisión restante es `cev1q_040_03`: «¿Quién ganó el primer premio de la exposición de cerámica?». El candidato `cev1_040_03` dice «La exposición de cerámica no tendrá concurso ni premios». Ambas políticas se abstuvieron pese a tenerlo entre candidatos. No se modificó la instrucción para rescatarlo. La semilla vacía omite una corrección útil conforme a la rúbrica; no demuestra que se haya inventado un ganador.

## Recursos y tiempos

| Medición | Base | `utility_v1` |
| --- | ---: | ---: |
| Llamadas / recibos de coste | 64/64 | 64/64 |
| Tokens de entrada reportados | 21.918 | 28.638 |
| Coste reportado, USD | 0,0021918 | 0,0028638 |
| Runner p50, ms | 364,33 | 370,96 |
| Runner p95, ms | 715,75 | 602,11 |
| Runner p99, ms | 1.033,78 | 643,07 |

Modelo servido en las 128 llamadas: `openai/gpt-6-luna-decisions-20261006`. La variante usó 6.720 tokens adicionales (+30,66 %, 105 por consulta) y USD 0,000672 más en sus 64 consultas. El coste reportado total de esta campaña fue USD 0,0050556; no hay recibos ausentes. Es consumo informado por el proveedor de elección tipada, no auditoría de factura ni tokens de semilla.

La mediana de la diferencia temporal por par (variante menos base) fue +1,94 ms; p95 +170,57 ms y p99 +280,90 ms. Las colas marginales menores no prueban aceleración general: solo hay una observación por configuración y pregunta, con red. Los tiempos del runner tampoco son latencia de integración con Codex.

## Verificación, alcance y decisión

La auditoría Rust verificó hashes, eventos completos, 1.024 apariciones de candidatos y 83 recuerdos de semillas (36 de la base y 47 de la variante), incluidos texto, condiciones, fuente declarada, ámbito, revisión, estado y vigencia. Las semillas vacías no contienen recuerdos que auditar. Una [comprobación mecánica separada](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/verification.json) recontó eventos, orden, IDs, transiciones, consumo y hashes previos desde los diarios. No representa revisión semántica independiente.

Pasaron las 45 pruebas del workspace, formato y comprobación de espacios del diff. Las pruebas cubren propagación de política al clasificador, búsqueda vacía sin llamada, fallos, validación de política por caso, conservación de etiquetas y bloqueos por regresión. La campaña real confirma deserialización de `utility_v1`, la ruta remota y composición; no confirma fuentes externas ni verdad de una respuesta final generada.

El banco es desarrollo sintético conocido: la línea base anterior motivó la variante, las etiquetas fueron redactadas y revisadas por el mismo autor, cada ámbito tiene solo ocho recuerdos, y las preguntas parciales son de dos partes. No hay revisión independiente, calibración nueva, reserva ni preguntas humanas. Conservar 16/16 ausencias en estos ejemplos no demuestra tasa cero de selecciones irrelevantes en uso real. Una selección parcial tampoco acredita que el agente posterior reconocerá lo desconocido al responder.

Se conserva `baseline` y límite 20 como predeterminados. `utility_v1` queda como candidata explícita para contrastar con **otra cohorte y anotación independiente**, incluyendo distractores difíciles, contexto condicional, negaciones y listas mayores. No se sigue afinando sobre la omisión conocida ni se promociona por este banco. El README conserva sus métricas vigentes y M5 mantiene su cierre limitado. Esta ganancia corresponde a conservar evidencia útil en desarrollo, no a una mejora demostrada de respuestas finales o integración.

Archivos: [preparación](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/preparation.json), [casos derivados](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/paired-cases.jsonl), [diario](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/journal.jsonl), [auditoría](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/audit.json), [reporte](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/report.json) y [manifiesto de integridad](../../outputs/iromem/evaluation/utility-v1-paired-2026-10-18/manifest.json).
