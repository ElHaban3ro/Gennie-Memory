# Contribución generativa: dos filtros fallidos conservados

Fecha UTC:2026-10-20. Se implementó y ejecutó un clasificador generativo de roles A/B/C/none con las fuentes originales. Dos versiones completaron las16 preguntas/64juicios mínimos, ambas con JSON válido y ambas fallidas semánticamente. No se ampliaron a casos difíciles ni se incorporaron al producto. El objetivo permanece activo en el paso3.

## Mecanismo y alcance

V2 usa Gemini2.5FlashLite por Google AI Studio estándar, sin fallback, razonamiento ni reintentos. Conserva literalmente las reglas v3 y añade un procedimiento general de sujeto, atributo, condiciones y estado de evidencia; omite las ocho analogías que recibían los deciders. La salida contiene únicamente roles en orden, sin hechos, citas o probabilidades generadas. Se remapea a IDs/revisiones y textos originales.

El ensayo generativo anterior con citas inventadas sigue fallido. Aquí se elimina el requisito de producir esas citas, pero no se considera el formato válido como prueba de pertinencia. Se verifican modelo/proveedor, finalización completa, ausencia de negativa explícita, JSON con un solo campo, cantidad exacta y enum estricto. Truncamientos, campos extra y respuestas inválidas degradan y conservan recibos, sin convertirse en `none`.

V3 añade solo las ocho tripletas congeladas de LocalV5 al mensaje system de V2. Las16 peticiones se compararon antes de red: ninguna otra diferencia. No se añadieron ejemplos del banco, etiquetas ni respuestas esperadas. Una prueba determinista conserva esa relación entre builders. La comparación no prueba una causa interna del modelo ni estabilidad.

## Resultados completos del mínimo conocido

| Métrica | V2 sin analogías | V3 con analogías |
| --- | ---: | ---: |
| Respuestas técnicamente válidas | 16/16 | 16/16 |
| Roles correctos | 50/64 | 47/64 |
| Escenarios de roles exactos | 4/16 | 6/16 |
| Contribuciones útiles conservadas | 14/17 | 11/17 |
| Selecciones ajenas | 10 | 10 |
| Errores de rol | 14 | 17 |
| Coste informado | USD0,000874500 | USD0,001202100 |

V2 falla por confundir relación temática con aporte: acepta reglas de recogida opcional, requisitos distintos y detalles de otros atributos, y pierde parte de la evidencia parcial y dos contrastes. No hay error técnico que permita atribuir esos desacuerdos al parser.

V3 corrige5juicios de V2 y estropea8que antes eran correctos;13roles cambian, sin mezclar mejores respuestas entre versiones. Pierde seis evidencias útiles y conserva diez selecciones ajenas. Aunque6sets completos superan los4deV2, ese agregado no compensa el empeoramiento de conservación/roles. Los dos contrastes perdidos persisten. El resultado no respalda que la ausencia de analogías fuera una solución suficiente. No se ejecutaron repeticiones para seleccionar una corrida favorable.

Ambas cumplen el límite monetario del mínimo y fallan calidad. Los preparadores rechazaron sus ampliaciones antes de crear archivos. No existen llamadas de los18pares generativos; por tanto no hay comparación contemporánea de recursos contra baseline ni medición de `recall` para estas versiones. Las rutas de artefactos mantienen el sufijo histórico `minimal-v1`; el contrato generativo ejecutado es V2/V3 según su protocolo y código archivado.

## Decisión y siguiente investigación

Ninguna versión es elegible. No se sustituyen las métricas de Perplexity ni las del producto por estos filtros. El procedimiento de sujeto/atributo se expresó como instrucción, pero no se emitió ni comprobó una representación estructurada de esa relación: esta prueba no demuestra que esa representación no pueda funcionar. La siguiente hipótesis deberá comprobar una relación explícita antes de la composición, con conservación de partes y refutaciones; no basta otra variación genérica de instrucciones o otra repetición de esta salida de roles. Debe justificar coste y evitar que un fallo de extracción se interprete como ausencia fiable.

Las puertas originales siguen pendientes: mecanismo que pase desarrollo/recursos, runtime/CLI, campañas completas, estabilidad y casos nuevos con revisión ciega. V6 conserva todos los hashes y cero llamadas. No se cambia default, dashboard, SQL, secretos o servicios. README y gráficas mantienen la ruta vigente.

## Contabilidad y verificación

Tanda:32operaciones/32HTTP, **USD0,002076600**, recibos completos y cero costes nuevos desconocidos. Acumulado:5.606operaciones,23campañas emparejadas completas, **USD0,569447228 conocidos +4 operaciones históricas de importe desconocido**. Codex no está incluido.

251pruebas pasan con y sin HTTP/2; formato y compilación pasan. Se reconstruyeron exactamente los13reportes —los11previos y los2nuevos— y se conservaron hashes de ambos bancos y todos los snapshots anteriores. Los runners son manuales ignorados bajo `cfg(test)`; no se iniciaron servicios ni conexiones SQLite, ni se detuvieron procesos ajenos.

Protocolos congelados: [V2](generative-contribution-v2-protocol-2026-10-20.md) y [V3](generative-contribution-v3-examples-protocol-2026-10-20.md). Artefactos: `outputs/iromem/evaluation/contribution-local-gemini-minimal-v1-2026-10-20` y `contribution-local-gemini-examples-minimal-v1-2026-10-20`. Contabilidad y código del cierre: `quality-goal-2026-10-19/snapshots/039`.

La tarifa estándar consultada para estimar, USD0,10/0,40por millón de entrada/salida, está publicada en [OpenRouter](https://openrouter.ai/google/gemini-2.5-flash-lite); el uso de JSON schema está descrito en [Structured Outputs](https://openrouter.ai/docs/guides/features/structured-outputs). Los importes de este reporte proceden de los recibos reales, no de esa estimación.
