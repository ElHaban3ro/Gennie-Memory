# Contraste Qwen9B reasoning: ejecución incompleta y plazo HTTP corregido

El contraste controlado OFF/ON no permite medir mejora de calidad. Se iniciaron 2 de 32 peticiones: una terminó y otra quedó sin respuesta terminal; 30 no se ejecutaron. El journal original no se modifica ni se completa artificialmente.

| Medida observada | Resultado |
| --- | --- |
| OFF, preguntas válidas | 1 de 16 previstas |
| OFF, roles correctos | 3 de 4 en esa pregunta |
| OFF, evidencia útil conservada | 0 de 1 |
| ON, clasificación | No evaluable |
| Peticiones iniciadas / terminadas / sin terminal | 2 / 1 / 1 |
| Coste API conocido nuevo | USD 0,000052950 |
| Operaciones con coste desconocido nuevas | 1 |
| Coste Codex | No cuantificado |
| Adopción / cohorte V6 abierta | No / no |

Las dos configuraciones mantenían modelo Qwen3.5-9B/DeepInfra, mensajes, esquema, temperatura, orden y presupuesto de 2048 tokens; solo cambiaba reasoning.enabled. No se interpreta una única pregunta como rendimiento global ni se compara con los 64 juicios completos de la campaña anterior. El informe JSON conserva null para métricas ON no evaluables. Cero es únicamente el subtotal conocido de ese brazo, no su coste total.

La segunda petición seguía activa después de 102 segundos pese al timeout del cliente. Se identificaron y terminaron exclusivamente Cargo y el test propios; se conserva registro de interrupción y salida 143. No se observaron chunks del proveedor. El diagnóstico posterior reprodujo localmente un escape del plazo en reqwest blocking; aplicar timeout también a RequestBuilder corrige la reproducción HTTP/1 y HTTP/2. [Decisión y límites](../decisions/0029-total-http-request-deadline.md). No hubo inferencias remotas posteriores a la interrupción. Reproducir la comparación requiere otro protocolo prospectivo; no se reinicia esta tanda.

El objetivo de cinco pasos sigue activo en el paso 3, sin candidata que supere las puertas. El contrato y banco están implementados, pero no equivalen a mejora de recuperación. Se conserva el rechazo documental 64/116 y el resultado previo Qwen 49/64, 6/17 evidencias conservadas, 11 pérdidas y 3 selecciones ajenas. No se relajan las puertas para cerrar M5.

Acumulado de contabilidad: 6154 operaciones externas, 23 campañas emparejadas completas; USD 4,566379428 conocido y 23 operaciones de importe desconocido. Las llamadas de clasificación de esta tanda usan un modelo pequeño; cero llamadas nuevas a modelos frontera por API. No hay commit, push, cambios en la base del usuario o reinicio del dashboard.

Evidencia: `outputs/iromem/evaluation/qwen9b-reasoning-pair-v1-2026-10-22/` contiene preflight/código ejecutado anterior al arreglo, solicitudes, journal, interrupción, informe y diagnóstico local. El snapshot 126 vincula verificación final y fuentes actuales por separado. La latencia del producto y las puertas de recursos/calidad siguen sin nueva validación remota.

Verificación final del código actual: **879 pruebas default y 881 con HTTP/2 aprobadas**, cero fallos; 24 y 74 ignoradas respectivamente. Formato correcto. Quince informes históricos reproducidos byte por byte; informe interrumpido reproducido exacto; alteraciones del inicio pendiente y de la fuente del cliente rechazadas. Quitar el coste del único recibo deja dos importes desconocidos y ninguna métrica de calidad evaluable, sin pase. Estas comprobaciones locales no acreditan mejora semántica ni rendimiento remoto.
