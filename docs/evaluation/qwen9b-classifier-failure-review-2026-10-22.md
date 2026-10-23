# Qwen9B: fallo semántico del intento completo

Fecha: 2026-10-22. Revisión estática de fuentes, gold, payloads, cuerpos y reporte en `outputs/iromem/evaluation/contribution-qwen9b-minimal-attempt2-2026-10-22/`. Sin red, Cargo o cambios de código. El primer intento429permanece separado, sin causa identificada.

**El segundo intento falla calidad con transporte y recibos completos:**16respuestas válidas/64juicios,49roles correctos y15errores,11IDs útiles omitidos y3selecciones ajenas. El error restante es un ID útil con rol equivocado, no otra pérdida de ID. Solo2/16sets de roles son exactos. CosteUSD0,000823150; todos los cuerpos declaran Qwen/DeepInfra y cero tokens de razonamiento. No hubo429. Este resultado no mide baseline contemporáneo, estabilidad o recuperación completa.

## Patrón que importa para la siguiente acción

No es solo dificultad con contrastes o con una construcción normativa ambigua. Hay omisiones incluso de respuesta explícita y de negación inequívoca; y también aceptaciones de otro objeto/atributo. Por ello no basta arreglar la letraC, acortar instrucciones o cambiar transporte para explicar el fallo observado.

| Patrón | Ejemplos fuente/pregunta y salida |
| --- | --- |
| Evidencia solicitada descartada por no ser valor completo | `cv3_01_01`: teléfono «no consta en el registro», A→none; `cv3_12_00`: cantidad de sal «ilegible», A→none. La regla A permite expresamente ambos estados. `cv3_04_00`:80minutos observados sin apagado y máximo no medido, A→none; debía conservar límite/estado sin inventar el máximo. |
| Respuesta directa o refutación explícita omitida | `cv3_07_00`: el depósito que alimenta Azucena mezcla lluvia y agua de red de **T6**, A→none. La mezcla no borra la tubería identificada para la parte de red. `cv3_06_00`: «Malva no usa agua de la red», B→none. `cv3_15_00`: Risco «no admite el modo Fino», B→none. Estos casos no necesitan resolver el problema nominal de Faro para contradecir la presuposición consultada. |
| Comparación explícita no retenida | Los tres casos de cámara/perfiles, impresoras/modos y grabadores/programas (`cv3q_03/13/16`) pierdenC→none. No prueban exclusividad, pero la rúbrica C pide conservar exactamente ese contraste entre asignaciones. |
| Rol/alcance/identidad transferidos | `cv3_08_02`, formato de Delta no registrado, A→C pese a que no compara asignaciones entre casos. `cv3_09_00`, borrado de un archivo ante política de toda la colección, none→B: la pregunta no afirma una política incompatible que ese evento refute. `cv3_10_01`, revisar fotos de Vado, none→A; `cv3_10_02`, inaugurar **Puerto**, none→A ante fecha de inauguración de **Vado**. La corrección real de autoría en `cv3_10_00` sí recibióB: acertar esa cláusula no evitó aceptar las dos ajenas. |

Además, Faro (`cv3_02_01`) pierdeB→none bajo su lectura anotada de eliminación del requisito de tutor. Sus cautelas lingüísticas históricas se mantienen; no son necesarias para sostener este fallo. La evidencia parcial de alimentación de Nardo (`cv3_05_00`) también pierdeA→none; la regla declara explícitamente su aporte parcial y prohíbe inferir exclusividad.

## ¿Problema de gold o límite demostrado del clasificador?

**No encontré un defecto nuevo concreto del gold que explique este patrón.** La regla suministrada permite explícitamente estado desconocido/ilegible, límites, fuente intermedia yCsin exclusividad; exige mismo objeto/atributo y no generalizar instancia a clase. Los ejemplos anteriores incumplen esas reglas sin requerir un cambio de contrato. No hay truncamiento, salida inválida, diferencias de orden o ID ni recibos ausentes que los conviertan en errores del parser. Todos los payloads llevan las mismas instrucciones completas; se mantuvieron los textos originales.

Eso demuestra el límite de **esta combinación de modelo, endpoint, instrucciones, esquema y ejecución** en el mínimo conocido. No distingue capacidad semántica, comprensión del idioma, seguimiento de reglas, sesgo hacia respuesta completa o efectos de contexto: son explicaciones posibles, no causas internas medidas. Hay44nonecorrectos, pero también11pérdidas y3extras; no describir el comportamiento como abstención uniformemente segura. El49/64agregado oculta que solo6/17IDs útiles quedaron seleccionados, uno con rol incorrecto.

No se prueba que toda lectura lingüística del banco sea única ni que otra anotación nunca sea defendible. Una objeción puntual a Faro o al concepto prospectivo de utilidad de un contraste se registra separadamente; no elimina las pérdidas deT6/no-red/no-Fino ni las aceptaciones de otro puente/acción. Gold histórico y métricas permanecen intactos.

## Consecuencia

Descartar esta versión completa y conservar baseline. El cuello observado ya incluye selección de relaciones elementales con texto íntegro; volver a otra firma de modelo o formato sin una hipótesis adicional no explica qué error corregiría. Antes de invertir en extracción, segmentación o un grafo, exigir una señal verificable sobre **objeto/atributo y fundamento solicitado** que preserve a la vez valores, estados y refutaciones: restringir el ensayo aCsolo o a negativos ocultaría pérdidas ya demostradas. Esto delimita la siguiente investigación; no propone otra campaña ni un nuevo protocolo.

El pase de coste del mínimo no compensa calidad y no acredita el límite relativo1,5×. El429del primer intento y las sondas neutras se mantienen en la contabilidad de investigación, con sus importes desconocidos donde corresponda; no forman parte de los64juicios del segundo. Sin adopción, cambio de all116/64fallido, reapertura de reserva o cierre del objetivo.
