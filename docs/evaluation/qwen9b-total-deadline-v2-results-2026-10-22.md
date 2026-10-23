# Qwen9B con plazo total: indisponibilidad explícita del proveedor

La prueba terminó tras **1 HTTP y cero clasificaciones**, antes de alcanzar el brazo ON. DeepInfra devolvió HTTP429 con `limit_source=upstream_provider_shared_pool` y `provider_error_code=engine_overloaded`. El mensaje atribuye la limitación temporal al pool compartido upstream. Esta evidencia explica **esta respuesta**, no la interrupción V1 ni los429 históricos sin esa metadata.

El preflight confirmó saldo suficiente y tarifas dentro de la estimación USD0,0229036. El fallo no se atribuye al límite de la clave. Se conservaron respuesta acotada sin secretos, journal y estado terminal; no hubo reintentos ni más sondas de inferencia. Cero subtotal conocido y un importe desconocido **no significan coste total cero**.

| Medida | Resultado |
| --- | --- |
| Preguntas del banco preparadas / solicitudes congeladas | 16 / 32 |
| Solicitudes ejecutadas / terminales / no ejecutadas | 1 / 1 / 31 |
| Clasificaciones válidas OFF / ON | 0 / 0 |
| Calidad de ambos brazos | No evaluable |
| Tiempo del intento HTTP fallido | 807,56 ms |
| Ventana del runner | 0,825 s |
| Subtotal API conocido nuevo | USD 0 |
| Operaciones nuevas sin importe | 1 |
| Coste Codex | No cuantificado |
| Puerta de calidad / adopción / apertura V6 | No / no / no |

El proceso de test terminó normalmente porque registró el error y detuvo nuevas llamadas. Su `ok` **no es un pase del experimento**: `screen_passed=false`. La respuesta llegó por HTTP/2 y terminó dentro del plazo, pero un429rápido no verifica la clasificación ON ni el arreglo frente al comportamiento remoto que bloqueó V1.

Los32payloads,16entradas, orden y etiquetas son byte por byte iguales a V1. Se cambiaron únicamente el transporte ya corregido y la identificación/fuentes del nuevo intento. V1 sigue con su journal original de3eventos y su resultado no evaluable. Ambos intentos fallidos quedan separados; no se reemplazan ni suman respuestas para completar una pareja.

La suite actual pasa **879 pruebas default y 881 HTTP/2**, cero fallos;24y75ignoradas respectivamente. Formato correcto. Quince informes históricos y el informe V1 interrumpido se reproducen byte por byte. No se repite la batería de425informes documentales porque este cambio afecta únicamente preparación/runner del contraste; su comprobación histórica queda identificada como heredada.

El objetivo sigue activo en el paso3: contrato y banco implementados, ninguna mejora nueva demostrada; faltan pase completo del mecanismo, recursos/estabilidad, cohorte nueva ciega y adopción condicionada. Acumulado:6155operaciones externas,23campañas emparejadas completas,USD4,566379428conocidos y24operaciones sin importe; coste Codex separado y desconocido. Sin modelos frontera por API, cambios de producto/DB/secreto, nuevos servicios, commit o push.

Evidencia: `outputs/iromem/evaluation/qwen9b-reasoning-total-deadline-v2-2026-10-22/`; [protocolo prospectivo](qwen9b-total-deadline-v2-protocol-2026-10-22.md), catálogos y saldo archivados, fuentes ejecutadas, auditoría y snapshot127. No se ejecutará un tercer intento de esta configuración en esta continuación.
