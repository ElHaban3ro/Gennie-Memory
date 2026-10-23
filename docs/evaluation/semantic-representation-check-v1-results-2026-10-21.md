# Verificador semántico aislado V1: capacidad no aprobada

UTC 2026-10-21. Paso 3 del objetivo activo. **El verificador acepta diez construcciones que debía bloquear; no se amplía ni se incorpora al producto.** Esta prueba usa construcciones manuales y no mide extracción o mejora de recuperación. [Protocolo previo](semantic-representation-check-v1-protocol-2026-10-21.md), [informe Rust](../../outputs/iromem/evaluation/semantic-representation-check-v1-2026-10-21/report.json), [etiquetas congeladas](../../outputs/iromem/evaluation/semantic-representation-check-v1-2026-10-21/frozen-labels.json) y [diagnóstico con respuestas](../../outputs/iromem/evaluation/semantic-representation-check-v1-2026-10-21/failure-diagnosis.json).

## Implementación y preparación

El módulo puro prepara una petición con un único texto fuente O pregunta, raw materializado no confiable, normalización operativa, ontología y registro. El juicio es independiente por fidelidad, completitud, conservación al normalizar y atribución de evidencia. No propone reparación, respuesta o relevancia. La aceptación se calcula localmente; citas y paths válidos no certifican semántica.

Se conservaron ambos lados de los 19 pares anteriores (38), más ocho controles nuevos con versiones correctas e incorrectas de calificadores, tiempos por solicitud, omisiones y negación: 46 operaciones. No se eligieron únicamente lados favorables ni se trasladaron etiquetas por pareja. Dos críticos nuevos recibieron solo sobres anónimos; produjeron 184 juicios cada uno, con cuatro discrepancias adjudicadas antes de llamar al proveedor. Los dictámenes iniciales y la conciliación quedan preservados. Una categoría semántica uncertain adjudicada no es un desacuerdo pendiente: debe reconocerse y bloquear aceptación. No es revisión humana ni independencia de familia. Los críticos recibieron todos los sobres en un mismo paquete: evaluarlos por lado fue una instrucción, no una separación estructural de su contexto. El modelo remoto sí recibió una sola entrada por petición, sin la contraparte.

La revisión de código corrigió tres fallos del preflight: congelación incompleta del normalizador, vínculo insuficiente del roster con fuentes y conciliación solo hasheada. El runner verifica el binario preparador y vuelve a generar preparación/requests/packet/map en un directorio temporal propio, comparándolos en bytes antes de HTTP; comprueba además la derivación completa de las etiquetas. El paquete y los payloads nuevos coinciden exactamente con los usados por los críticos. El transporte por lotes existente se reutiliza; su entrada antigua conserva el comportamiento anterior. No hay dependencias o credenciales nuevas.

Se conservan dos fallos iniciales de preparación de la ontología (intermediate_types/additive ausentes), dos errores de compilación de los nuevos tests/reporte y sus correcciones. Ocurrieron antes de la inferencia y no se ocultan como resultados del modelo.

## Métricas del piloto completo

| Medida | Resultado |
| --- | --- |
| Operaciones previstas/iniciadas/completas | 46/46/46 |
| Respuestas con modelo, formato y recibo válidos | 46/46 |
| Construcciones fieles aceptadas | 35/35 |
| Construcciones que debían bloquearse | 11: nueve defectuosas y dos inciertas |
| Bloqueos correctos | 1/11 |
| Aceptaciones indebidas | 10: ocho defectuosas y dos inciertas |
| Rechazos de construcciones fieles | 0 |
| Concordancia con etiquetas por dimensión | 153/184; 31 discrepancias |
| Fallos técnicos, no enviados, reintentos | 0, 0, 0 |
| Tokens de entrada/salida informados | 71.230 / 11.590 |
| Coste nuevo informado | USD 0,210604; cero importes nuevos desconocidos |
| Duración de intento terminal p50/p95 | 2.477 / 3.649 ms |
| Duración del runner con concurrencia máxima cuatro | 47,068 s |

Los denominadores son construcciones y juicios, no preguntas nuevas de recuperación. Pasar JSON y las 491 pruebas del workspace no compensa estos fallos. La matriz conserva todas las predicciones y los motivos, incluso cuando coinciden en el bloqueo pero difieren de la dimensión anotada.

## Qué falló

La negación invertida es un caso inequívoco: el texto dice «El formulario F no es verde.», mientras raw declara affirmed=true y la normalización positive=true. El modelo aprueba y describe la construcción como negativa. También aprueba metadatos hoy con una cita ayer, equipara la clase restringida sensores de presión con sensor general y pierde la restricción del propietario del norte. Acepta la pregunta con dos intervalos distintos cuando ambos usos quedan bajo ayer.

Solo bloquea la omisión de la masa: encuentra el dato faltante en el texto, pero lo atribuye además a una pérdida de normalización aunque ya falta en la propuesta raw. Otras discrepancias abarcan propiedad inferida desde finalidad, atribución de observación no expresada y alcance del suministro de agua. Las razones guardadas confunden en varios casos texto original, propuesta raw y normalización. Esta es una hipótesis sobre el fallo observable; no conocemos la causa interna del modelo. No se atribuye a ausencia de entradas, error HTTP o recibos faltantes: los inputs y el recorrido previo están ligados por hash.

Los dos falsos positivos que motivaron la ronda persisten. El resultado impide tratar una segunda aprobación textual como prueba de veracidad. La capacidad de esta versión falla sin necesidad de adjudicar a su favor interpretaciones inciertas. Se conserva la puerta cuantitativa fallida y la revisión semántica completa de motivos como pendiente; no se finge que esa auditoría adicional podría convertir el piloto en pase.

## Coste, conservación y siguiente trabajo

Se consultó una vez el catálogo público (HTTP de metadatos, separado de las 46 operaciones de inferencia); la primera consulta restringida falló DNS antes de HTTP. El catálogo informó la ruta OpenAI y precios de USD 0,000002 por token de entrada y 0,000008 por salida. La estimación previa fue USD 0,37 y 2–4 minutos; una ilustración conservadora con bytes de payload más salida máxima fue USD 1,53, no una garantía de factura. La suma exacta de los 46 recibos es USD 0,210604. Modelo solicitado/servido: alias openai/gpt-4.1, proveedor informado OpenAI; la revisión interna fechada no fue verificada independientemente. No se recomienda este alias para producción.

Acumulado de investigación: **6047 operaciones lógicas, 23 campañas emparejadas completas, USD 1,055902508 conocidos más 16 operaciones de coste desconocido**. Codex no está incluido; los tokens anteriores son del verificador remoto. Esta etapa sobre construcciones manuales no mide coste de una tubería completa, recursos comparados con baseline, estabilidad ni latencia de recall/end-to-end. Tampoco se afirma negociación HTTP/2: se compiló la feature, pero el transporte reutilizado no guardó esa versión por respuesta.

Pasaron 491 pruebas con/sin HTTP2, formato y diff. Se reprodujeron idénticos cuatro bancos manuales, diez informes previos de extracción y los tres auditores de cobertura/catálogo/referencias distribuidas; el informe nuevo también se reproduce en bytes. Snapshots001–060 y gold se preservan. No cambia producto, baseline, dashboard, SQL, servicios, README/gráficas o V6. No existe una nueva mejora de rendimiento del producto que justifique reemplazar sus figuras vigentes.

La siguiente hipótesis requiere separar un inventario independiente del texto de las afirmaciones operativas explícitas de la propuesta, y comprobar cada correspondencia antes de comparar normalizaciones. Sigue sin implementarse; no se convierte este piloto observado en validación ajustando su prompt. Debe mantener el alcance general, datos parciales/correctivos, calificadores y múltiples solicitudes, sin vocabulario finito como sustituto ni abstención universal. Después faltan la tubería automática completa, conocidos100/126, comparación con baseline, calidad/recursos/estabilidad y cohorte nueva ciega. El objetivo de cinco pasos sigue activo, no logrado ni bloqueado por falta de autorización.
