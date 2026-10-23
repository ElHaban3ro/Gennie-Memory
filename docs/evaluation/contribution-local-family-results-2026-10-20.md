# LocalV5 entre familias: ahorro sin pase de calidad

Fecha UTC: 2026-10-20. Se cambió únicamente `model` en las peticiones LocalV5 congeladas, con familias vinculadas a alias y revisión esperada. Ninguna variante pasa todas las puertas. La configuración predeterminada sigue siendo Decisions/baseline; el objetivo permanece activo en el paso3 y V6 conserva cero llamadas.

## Filtro mínimo

Mismas16 preguntas,64 juicios y17 contribuciones útiles anotadas antes de red. Fuentes, instrucciones, ejemplos, criterios, orden y etiquetas idénticos a LocalV5/Jev. Son casos de desarrollo conocidos, no validación nueva.

| Familia | Roles correctos | Útiles conservados | Selecciones ajenas | Coste informado |
| --- | ---: | ---: | ---: | ---: |
| Microsoft | 62/64 | 17/17 | 1 | USD0,000601482 |
| Perplexity | 64/64 | 17/17 | 0 | USD0,000316180 |

Microsoft confunde un teléfono de contacto opcional con refutación y una incompatibilidad explícita con contraste. No se amplió a los18 pares. Se comprobó además que el preparador rechaza esa ampliación antes de crear archivos. Perplexity pasa el mínimo y el límite de coste predeclarado; solo esa familia continúa. Los antecedentes fallidos con otros contratos siguen intactos.

## Contraste difícil de Perplexity

18 pares,36 respuestas válidas HTTP/2, orden alternado, baseline real contemporáneo con hasta20 candidatos frente a7 de la candidata. Banco conciliado de126 roles/17 contribuciones, separado del gold histórico. Ninguna selección quedó sin revisar.

| Métrica | Base contemporánea | Perplexity/LocalV5 |
| --- | ---: | ---: |
| Contribuciones útiles conservadas | 11/17 | 17/17 |
| Selecciones ajenas revisadas | 1 | 4 |
| Errores de rol canónico | No clasifica roles | 5/126 |
| Mediana de clasificación | 545,94ms | 429,57ms |
| p95 de clasificación | 1.217,89ms | 530,22ms |
| Coste informado | USD0,001119900 | USD0,000457460 |

La candidata gana seis contribuciones sin perder las once de su base, pero las cuatro selecciones ajenas y cinco errores de rol bloquean calidad. Coste/mediana/p95 relativos:0,408×/0,787×/0,435×; recursos pasan. El coste baja59,15% frente a su control. Los tiempos incluyen petición/lectura/decodificación/validación, no `recall` ni la respuesta final. p95 por rango más próximo con18 observaciones es el máximo; no se retiraron valores lentos.

Los cuatro extras son `cev4_061_10` (archivo individual frente a política M4), `cev5_068_07` (libreto de otro sujeto) y `cev5_069_10`/`cev5_069_12` (chequeo genérico y cita futura frente a autonomía). Los dos últimos conservan la ambigüedad residual de la conciliación, sin retocar etiquetas después de medir. El quinto desacuerdo es `cev5_073_17`: selecciona correctamente el depósito parcial que alimenta Laurel, pero lo llama refutación B en vez de aporte A. No se infiere exclusividad de esa conexión.

Frente al gold histórico original hay tres extras; uno es el depósito ahora anotado como parcial. Los resultados históricos no se recalculan. Incluso esa lectura conserva errores y no vuelve elegible la candidata.

El Jev/LocalV5 anterior conservó16/17 útiles con5 extras y6 errores. Perplexity conserva17/17 con4 extras y5 errores en esta tanda. Es una señal favorable sobre desarrollo conocido, no prueba de superioridad estable: las tandas no son un emparejamiento contemporáneo entre Jev y Perplexity. Tampoco demuestra corrección de respuestas finales.

## Decisión y trabajo pendiente

Cambiar de familia no resuelve suficientemente las confusiones semánticas. No se mezcla la mejor respuesta de cada ejecución ni se baja la puerta de calidad por ahorro. Antes de otra expansión corresponde investigar una representación que compruebe explícitamente la relación sujeto/atributo y el estado de la evidencia —medida, chequeo genérico o actividad futura— conservando el aporte parcial. Una nueva variante debe congelar ese mecanismo y pasar de nuevo mínimo, casos difíciles y recursos; no basta repetir este piloto esperando otra salida. El ensayo generativo previo con citas inventadas continúa fallido y exige un contrato estructural distinto si se retoma esa vía.

Faltan mecanismo elegible, integración runtime/CLI comprobada, campañas completas, estabilidad y casos nuevos con revisión ciega. No hay adopción, nueva política de producto ni apertura de V6. README y gráficas conservan la ruta vigente.

## Coste y verificación

Esta tanda:68 operaciones/68 HTTP, **USD0,002495022**, incluidos ambos mínimos y los18 controles. Cero importes nuevos desconocidos, reintentos o llamadas inciertas. Acumulado:5.574 operaciones,23 campañas emparejadas completas, **USD0,567370628 conocidos +4 operaciones históricas de importe desconocido**. La programación y revisión de Codex no están incluidas.

245 pruebas pasan con y sin HTTP/2; formato y compilación pasan. Once reportes se reconstruyen exactamente desde sus artefactos, incluidos los ocho anteriores. Los bancos y las fuentes permanecen congelados; se verifican todos los hashes de V6 sin consumir sus preguntas. Los runners son manuales ignorados bajo `cfg(test)`; no se iniciaron servicios, modificaron procesos del usuario, crearon conexiones SQLite ni credenciales.

Artefactos: `outputs/iromem/evaluation/contribution-local-microsoft-minimal-v1-2026-10-20`, `contribution-local-perplexity-minimal-v1-2026-10-20`, `contribution-local-perplexity-known18-paired-v1-2026-10-20`. Protocolo: [contraste congelado](contribution-local-family-protocol-2026-10-20.md). Cierre de contabilidad y huellas: `quality-goal-2026-10-19/snapshots/038`.
