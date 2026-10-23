# Diagnóstico de críticos rápidos, citas y utilidad de la evidencia

Fecha UTC: 2026-10-20. Desarrollo sobre preguntas conocidas, no validación nueva. Ninguna variante cumple todas las puertas; `baseline` sigue vigente. El objetivo de cinco pasos permanece activo en el paso 3. V6 conserva sus hashes y cero llamadas. No se actualizan las gráficas del README con resultados de desarrollo fallidos.

| Experimento | Operaciones / HTTP | Resultado contra etiquetas originales | Recursos y coste informado total |
| --- | --- | --- | --- |
| Sustituir Microsoft por Decisions, manteniendo Jev | 12 / 12 | Conserva 4/5 propuestas útiles, rechaza 5/6 no permitidas; pierde M4/Umbral y acepta carné como respuesta a avalista | USD 0,000769170; filtro de juicios, sin latencia integrada medida |
| Perplexity directo con RoleV5 | 36 / 36, 18 pares | Conserva útiles históricos y contemporáneos, gana 7 IDs frente a base; añade 3 no permitidos. Resuelve las cinco regresiones iniciales, pero no pasa calidad | Candidata/base: coste 0,427×, p50 1,144×, p95 **3,143×**. USD 0,001598440 incluyendo controles |
| Gemini Flash Lite con roles y citas literales | 1 / 1; otras 5 sin ejecutar | Primera respuesta técnicamente inválida: cita no literal, y pruebas para roles `none`. Se detiene; no se inventa porcentaje semántico | USD 0,000141700; coste conocido pese al rechazo. Sin comparación de recursos |

Total de esta tanda: **49 operaciones / 49 HTTP, USD 0,002509310**, sin nuevos importes desconocidos. Acumulado: **5.341 operaciones, 23 campañas emparejadas completas, USD 0,552124172 conocidos y 4 operaciones históricas de coste desconocido**. Las operaciones incluyen fallos técnicos cobrados; no son todas respuestas válidas. La revisión/programación de Codex queda fuera de esta contabilidad de proveedor.

En Perplexity, p50 candidata/base es 461,727/403,749 ms y p95 2.229,679/709,473 ms. El p95 del piloto de 18 es el máximo por rango más próximo: no se elimina la petición lenta. Las tres selecciones no permitidas son `cev4_060_09` (detalle del ensayo ante recuento), `cev5_071_03` (carné ante avalista) y `cev5_073_17` (depósito con tapa verde ante acometida). La última requiere distinguir selección de rol, como se explica a continuación. No se llama falsedad a una discrepancia de IDs.

## Auditoría ciega retrospectiva

Dos agentes separados recibieron únicamente seis preguntas y sus siete recuerdos, con IDs anónimos: nunca gold, respuestas, probabilidades ni resultados. El primer encargo preguntó por utilidad para responder **o aclarar**; seleccionó 8/42. El segundo recibió además la rúbrica explícita `evidence-context-rubric-v2`, sin la primera revisión; seleccionó 6/42. El gold histórico selecciona 5. Se verificaron por código los 42 pares, su correspondencia con ID/revisión y las 84 citas literales de cada revisión. Esto verifica procedencia textual, no semántica. Los agentes comparten familia de modelo; no equivalen a revisión humana independiente.

Adjudicación retrospectiva del agente principal, conservando ambas revisiones:

- **Domicilio:** que no sea obligatorio registrarlo no da su valor ni prueba que no exista registro. Es contexto relacionado, fuera de la selección de evidencia solicitada. La primera revisión lo admitió por el alcance amplio de «aclarar»; la segunda lo excluyó.
- **Avalista:** exigir carné no excluye exigir avalista. El recuerdo que elimina expresamente el requisito sí refuta la premisa. Mismo desacuerdo de alcance en la primera revisión; no hay nueva refutación en el carné aislado.
- **M4/Umbral y M8/Silente:** la frase conjunta aporta un contraste explícito pertinente. No demuestra por lógica que M4 jamás pueda usar Silente. Conservar su utilidad no autoriza presentar exclusividad como hecho. La segunda revisión lo llama refutación, pero su propia objeción reconoce esta suposición; la primera distingue contraste de incompatibilidad estricta. La etiqueta histórica se conserva.
- **Depósito de Laurel:** «el depósito de lluvia L que alimenta Laurel» aporta una parte concreta sobre origen/recorrido del agua. El color no elimina esa cláusula útil. Ambos revisores lo conservan; ninguno aporta prueba de exclusividad. La segunda revisión lo llama refutación, pero reconoce posible alimentación mixta: adjudicación principal **parcial**, no negación de conexión a red. Es una discrepancia plausible del gold histórico, registrada sin reetiquetar ni mejorar retrospectivamente porcentajes.

Resultado: hay dos problemas distintos que no se arreglan agregando votos: confundir un requisito compatible con exclusión, y confundir evidencia útil de contraste/parte con refutación estricta. El acuerdo en seleccionar un ID no certifica que el rol asignado sea correcto. Incluso descontando hipotéticamente la discrepancia del depósito, Perplexity conserva otros dos extras y falla p95; no sería elegible.

## Consecuencia para los cinco pasos

Reabrir la precisión semántica de los pasos 1/2 antes de otro contraste de familias. La siguiente versión **de desarrollo** debe mantener por separado: dato/parte/estado solicitado, refutación incompatible, contraste explícito del mismo atributo/caso y contexto meramente relacionado. Un contraste no debe imponer exclusividad ni transferir atributos de otro sujeto. Evaluar por separado selección útil y corrección del rol; no ocultar errores de rol detrás de un ID acertado.

Antes de más red: preparar pares mínimos nuevos con fuentes que hagan explícita la coexistencia o exclusividad (carné con/sin avalista; depósito exclusivo/mixto; M4 con uno/varios perfiles), incluir negativos de otro atributo y revisar a ciegas sin respuestas ni gold. No tocar V1–V6 ni recalificar campañas anteriores. Esta recomendación todavía no es un mecanismo aprobado, un banco nuevo ejecutado ni mejora medida. No volver a lanzar las tres variantes fallidas sin una hipótesis y un cambio falsables. Siguen pendientes mecanismo real, campañas completas, estabilidad y V6 nueva bajo puertas originales de calidad/coste/p50/p95.

## Reproducción y límites

Los tres auditores reconstruyeron exactamente los informes desde los diarios, payloads y código ejecutado archivados. `cargo test --offline --workspace` y la misma suite con `--features iromem/http2-transport`: **220 pruebas pasan en cada configuración**, sin red. Formato y compilación de ambos binarios pasan. El contrato de citas compartido prueba rechazo de citas inventadas/prestadas, cobertura, duplicados y campos; una cita literal válida seguiría sin demostrar pertinencia. Los runners nuevos son manuales ignorados bajo `cfg(test)`, sin política de producto, SQL, secretos nuevos o servicios. No se midió `recall` completo ni integración Codex.

Artefactos desde la raíz del repositorio:

- `outputs/iromem/evaluation/fast-critic-replacement-screen-v1-2026-10-20/`: protocolo ligado, inputs, raw, recibos y `report.json`.
- `outputs/iromem/evaluation/direct-role-perplexity-v1-2026-10-20/`: 18 pares, trazas H2 y guardia de las once propuestas archivadas.
- `outputs/iromem/evaluation/quoted-role-generative-v1-2026-10-20/`: primera respuesta inválida conservada, cinco preguntas sin ejecutar; el informe deja errores semánticos en `null`.
- `outputs/iromem/evaluation/known-source-utility-blind-review-2026-10-20/`: entrada anónima, mapa privado, ambas revisiones y `audit-reviews.py` reproducible sin red. La adjudicación semántica es el texto de este informe, no una conclusión determinista del script.
- `outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/034/`: cierre de contabilidad y huellas; snapshots anteriores inmutables.
