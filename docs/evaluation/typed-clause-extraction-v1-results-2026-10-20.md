# Cláusulas tipadas V1: resultado negativo del piloto

UTC 2026-10-20. El mecanismo de extracción por cláusula está implementado y probado, pero **no supera la puerta de calidad**. El objetivo de cinco pasos continúa activo en el paso 3; no se amplía el banco ni se incorpora la variante. [Protocolo congelado](typed-clause-extraction-v1-protocol-2026-10-20.md), [artefactos](../../outputs/iromem/evaluation/typed-clause-extraction-v1-2026-10-20/verification.json).

## Qué cambió

El modelo genera estructuras específicas de requisito, propiedad, cantidad, procedimiento, asignación, flujo, disponibilidad y conjunto, en lugar de rellenar todos los campos del DTO V5. Un módulo puro valida y traduce cada cláusula; el parser original verifica después la representación documental. Mantiene varias cláusulas, sujetos, condiciones y lecturas correlacionadas. Las preguntas se extraen sin recibir recuerdos ni respuestas manuales.

La crítica previa encontró cuatro fallos del traductor. Se corrigieron antes de congelar peticiones: identidad de componentes incluye el padre recursivo; las solicitudes conservan ámbito total/común/máximo/inespecífico; números se conservan como decimal exacto sin redondeo `f64`; el máximo de ocho premisas se valida localmente. Las regresiones incluyen componentes homónimos, entidad como valor, padres anidados, enteros alrededor de 2^53 y ocho/nueve premisas. Estas mejoras de código no prueban calidad de extracción. [Crítica inicial](../../outputs/iromem/evaluation/typed-clause-extraction-v1-2026-10-20/code-critic.json), [revisión tras corregir](../../outputs/iromem/evaluation/typed-clause-extraction-v1-2026-10-20/code-critic-final.json).

El evaluador reutiliza `serde` ya fijado en el workspace como dependencia directa; no se añade versión o paquete externo nuevo. [Decisión 0023](../decisions/0023-typed-clause-evaluation.md). No cambia la política del producto.

## Resultado comparable del filtro conocido

Mismas cuatro posiciones de fuente y cuatro de pregunta, sin seleccionar por etiquetas. Todos los contenidos recibieron dos críticas nuevas sin historial, gold, roles ni frames manuales. Los revisores son agentes de la misma familia, no humanos ni una validación independiente poblacional.

| Indicador | Guiado V5 anterior | JSON J1 anterior | Cláusulas V1 |
| --- | ---: | ---: | ---: |
| Peticiones terminadas HTTP 200 | 8/8 | 8/8 | 8/8 |
| DTO completos aceptados | 5/8 | 3/8 | **1/8** |
| Fuentes aceptadas | 4/4 | 2/4 | 1/4 |
| Preguntas aceptadas | 1/4 | 1/4 | 0/4 |
| Raw fiel según ambos críticos | 2/8 | 1/8 | **0/8** |
| Coste por recibos USD | 0,0093804 | 0,0116604 | **0,0091212** |

Los contratos son versiones distintas: la tabla describe aceptación de cada variante, no igualdad de dificultad ni una estimación estadística de rendimiento general. La reducción de coste no compensa el fallo de calidad. Las tasas de fidelidad son decisiones de revisión con discrepancias conservadas.

En cláusulas V1, un revisor considera cinco raw fieles y tres defectuosos; el adversarial considera cinco inciertos y tres defectuosos. Coinciden en los tres defectos y discrepan en las otras cinco referencias. El primero tolera el referente recuperable pese al tipo incorrecto; el segundo no certifica equivalencia de una clase con un objeto/grupo concreto. No se convierten esos inciertos en aciertos ni en errores confirmados. El único normalizado disponible conserva el raw, pero hereda su defecto de sujeto; siete normalizados no están disponibles. **Ninguna salida reúne DTO válido y fidelidad confirmada por ambos.** [Ambas revisiones y resumen](../../outputs/iromem/evaluation/typed-clause-extraction-v1-2026-10-20/blind-review-summary.json).

Fallos observados, sin corregir las salidas:

- `op_000`: conserva «no exige teléfono», pero elimina el formulario como portador y lo predica del pedido; pasa estructura con un alcance semántico discutible/incorrecto.
- `op_054`: inventa `network_provenance` como nombre del sujeto y copia la oración en el valor, sin estructurar el enlace por T2.
- `op_088`: pierde Cima y «cada mes», sustituyendo una frecuencia de revisión por un requisito.
- Las cuatro preguntas omiten `status`. Hay además referencias que confunden el tipo de entidad con la categoría `class_reference`. Añadir la envoltura no resolvería estos problemas semánticos.
- `op_115`: representa cero pendientes, pero omite contexto/configuración y repite la cláusula como lectura sin alternativa real.

## Cobertura y control

El control manual sigue en 100/100 comportamientos previstos, 40 selecciones, sin certificar autonomía. No es un resultado del extractor. En las ablaciones con denominador 100, fuente automática tiene 99 pares degradados y pregunta/ambos tienen 100; el piloto dejó la mayoría de operaciones del banco sin ejecutar. No se presentan esos denominadores como precisión medida de 100 extracciones ni se selecciona solo el único DTO válido para anunciar calidad.

Se reproducen byte por byte raw y score de los cuatro bancos manuales previos y los seis informes de extracción anteriores, además del informe nuevo. Gold, anotaciones y casos difíciles no se modificaron. V6 conserva su hash de manifest, sin inspección semántica ni llamadas nuevas.

## Coste y tiempo

Ocho HTTP/2 200, OpenRouter con proveedor OpenAI fijo; alias servido `openai/gpt-4.1-mini`. El catálogo declara el endpoint fechado, pero los recibos no verifican por separado una revisión fechada del modelo. Cero reintentos y cero recibos nuevos desconocidos. Estimación anunciada USD 0,0394084; cota conservadora por bytes y salida máxima USD 0,136094. Coste real **USD 0,0091212**, incluidos fallos de contrato. No incluye agentes/Codex.

41.887 tokens de entrada, 917 de salida y 30.336 de entrada en caché según recibos. No se infiere coste solo de tarifas nominales ni de bytes. Son tokens del proveedor del experimento, no uso de Codex.

| Peticiones | p50 ms | p95 ms |
| --- | ---: | ---: |
| Fuentes, 4 intentos | 1756,317 | 1954,838 |
| Preguntas, 4 intentos | 1327,221 | 1725,091 |

El runner mide 3799,163 ms con hasta cuatro peticiones concurrentes; Cargo reporta 6,58 s para el test. Ninguna de estas cifras mide `recall`, integración ni respuesta final. No hay comparación contemporánea emparejada de recursos ni pase de su puerta.

Acumulado conciliado del objetivo: **5977 operaciones, 23 campañas emparejadas completas, USD 0,639238508 conocidos y 16 operaciones de coste desconocido**. Este piloto añade ocho operaciones, no una campaña completa. El gasto de Codex no está incluido.

## Verificación y siguiente decisión

413 pruebas pasan con y sin HTTP/2; formato y `git diff --check` pasan. Se ejecutaron 31 pruebas unitarias nuevas entre las dos inclusiones del contrato y el adaptador; ninguna sustituye la revisión semántica. Crítica de código con cuatro hallazgos corregidos y 16 juicios de contenido conservados. Snapshots 001–053 intactos antes del cierre; el estado nuevo se congela en 054. No hay servicios, puertos, bases temporales ni procesos pendientes de esta tanda.

La hipótesis «una estructura por tipo basta para extraer fielmente» no queda confirmada. No conviene ampliar los ocho ni reparar `status` para fingir un pase. El próximo diagnóstico debe separar errores de serialización de errores de referencia/predicado, con tareas de alcance reducido, citas y rechazo explícito de información no representable; comparar una capacidad de extracción más fuerte solo si mantiene el mismo texto/contrato y se conserva el coste de todos los intentos. El fallo de frecuencia demuestra además que el contrato no debe convertir un predicado no representable en `requirement`. Primero cobertura y fidelidad del texto aislado; después alineación automática fuente/pregunta, banco completo conocido y difícil, recursos y estabilidad; V6 únicamente al pasar esas puertas. No añadir aliases manuales ni ejemplos derivados de los ocho para fabricar concordancia.
