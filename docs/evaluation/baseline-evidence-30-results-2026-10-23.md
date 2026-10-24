# Baseline: evidencia explícita y 30 recuperaciones

Fecha: 2026-10-23 (America/Bogota). Ejecución: 2026-10-24 UTC. Estado: implementado y comprobado en desarrollo sintético.

El usuario autorizó aplicar la corrección del [diagnóstico de Aliso](baseline-context-diagnosis-2026-10-23.md) y ejecutar 30 recuperaciones, incluida la que falló. El baseline de Decisions/Jev/Jev114 ahora presenta los recuerdos como datos en `state.memories`, conservando las mismas etiquetas y textos de sus opciones. `none` no es un recuerdo y se excluye de esa colección. Consulta, contexto, instrucciones, opciones, parser y política predeterminada se conservan. La ruta baseline de Gemma y las políticas experimentales no cambian; `roles_fallback_v1` conserva su payload inicial histórico.

## Banco y protocolo

[Fixture versionado](../../tests/fixtures/baseline-evidence-30-v1/README.md): 30 operaciones `recall_v1` mediante el binario real y una sesión JSONL con SQLite propia. Contiene el caso original conocido de Aliso y 29 casos sintéticos nuevos: 20 positivos y 10 negativos; estos últimos incluyen dos búsquedas sin candidatos. Las 61 revisiones pertenecen a 60 recuerdos en 30 ámbitos separados.

Los casos cubren cautela en el contexto, consultas sin contexto, condiciones, prohibición de producción, observación, aprobación pendiente, cancelación, revisión vigente, entidades homónimas y vigencia temporal; los negativos preguntan datos ausentes o de otro referente. Las etiquetas se fijaron antes de la inferencia y permanecen fuera de la solicitud remota. El manifiesto liga corpus, política y casos por SHA-256. No se reutilizó la reserva final consumida del contrato v1.

Límites previos: 30 llamadas, 180 segundos y USD 0,02, con reserva contable de USD 0,002 antes de cada operación. Sin fallback ni reintentos automáticos. Recibo desconocido interrumpe la ejecución. La reserva contable no impone un límite de facturación al proveedor. Comprobación previa sin credencial: 30 operaciones completadas, 28 degradaciones explícitas por credencial ausente y dos búsquedas vacías, cero llamadas y cero coste. Una degradación técnica no se interpreta como abstención semántica.

## Resultado medido

| Indicador | Resultado |
| --- | ---: |
| Operaciones con resultado y semilla esperados | 30/30 |
| Evidencia positiva recuperada entre candidatos | 20/20 |
| Evidencia positiva seleccionada | 20/20 |
| Negativos sin selección | 10/10 |
| Selecciones ajenas / degradaciones | 0 / 0 |
| Semillas fieles al texto y metadatos del corpus | 30/30 |
| Latencia total por operación, mediana / p95 | 410,14 / 1.055,55 ms |
| Duración de la campaña | 15,23 s |
| Llamadas con recibo completo | 28 |
| Tokens de entrada / salida reportados | 8.273 / 0 |
| Coste total reportado | USD 0,0008273 |

Percentiles por rango más próximo sobre las 30 operaciones; las dos operaciones sin llamada se incluyen. Modelo servido: `openai/gpt-6-luna-decisions-20261006`. La puerta de desarrollo fijada en el manifiesto pasó, incluido p95 ≤ 2.000 ms.

Aliso conservó literalmente «Si no existe un formato registrado, no inventes uno» y el resto de su contexto. Recuperó y seleccionó `pcv1-aliso-1`, con una semilla que conserva la tabla de tarea, responsable y fecha. Tardó 1.337,58 ms; recibo completo de 320 tokens de entrada y USD 0,000032. El diagnóstico anterior preserva los fallos del payload antiguo; esta campaña ejecutó solo el payload corregido, sin un brazo contemporáneo adicional.

## Evidencia y límites

[Resumen y detalle de los 30 casos](../../outputs/gennie-memory/evaluation/baseline-evidence-30-20261024T041340Z/summary.json), [diario de solicitudes y respuestas](../../outputs/gennie-memory/evaluation/baseline-evidence-30-20261024T041340Z/journal.jsonl), [auditoría local](../../outputs/gennie-memory/evaluation/baseline-evidence-30-20261024T041340Z/audit.json) y [hashes del código/binario](../../outputs/gennie-memory/evaluation/baseline-evidence-30-20261024T041340Z/source-snapshot.json). El directorio conserva además capturas HTTP, recibos y la DB sintética. La auditoría comprueba correspondencia de los 30 inicios/finales, hashes de solicitudes/artefactos, suma de recibos y fidelidad de semilla. No verifica fuentes externas.

Es una muestra de desarrollo con etiquetas del mismo agente (`agent_synthetic_unreviewed`), sin doble revisión ni validación humana. No demuestra calidad poblacional, estabilidad entre repeticiones, exactitud de respuestas finales, rendimiento de Jev/Jev114 ni integración con agentes. Las pruebas locales comprueban el payload de esos tres modelos; las llamadas nuevas solo midieron Decisions. La relación de cada selección sigue siendo `unassessed`: baseline no produce roles ni justificaciones.

Duplicar los textos en state aumenta la entrada respecto al payload anterior. El coste anterior del diagnóstico y las métricas históricas de 540 preguntas se mantienen separados; no se atribuyen a esta revisión del baseline. Las gráficas existentes conservan sus cohortes históricas.

## Verificación del código

- Regresión del payload: fallo reproducido antes del cambio (`state.memories` ausente), dos pruebas correctas después.
- Validador offline: 30 casos ligados, 20 positivos y conservación de consulta/contexto original de Aliso.
- `cargo test --offline --workspace`: 1.061 correctas, cero fallidas y 25 omitidas.
- `cargo test --offline -p gennie-memory --features http2-transport`: 243 correctas, cero fallidas y 79 omitidas; las omitidas incluyen ensayos remotos manuales.
- `cargo fmt --all --check` y `git diff --check`: correctos. Compilación de ambos ejecutables: correcta.
- Reutilizar el directorio de ejecución fue rechazado antes de inferencia. Las capturas, entradas congeladas, fuentes ejecutadas y logs de verificación se conservan junto al resultado.

Estas comprobaciones corresponden al árbol local completo, que incluye cambios previos. No se realizó commit, push ni despliegue, ni se modificó la base del usuario.
