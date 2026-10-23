# Aislar la tarea no resolvió la pertinencia; Decisions también falla

Fecha UTC2026-10-20. La ablación completa **no apoya que separar tarea/campo fuente sea suficiente**:32/36roles correctos en conjunto,31/36solo rol con fuente y29/36sin el campo fuente. El filtro adicional con Decisions también falla:60/72juicios y31/36selecciones. No se construye un extractor ni se amplían campañas para estas combinaciones. Goal activo, paso3pendiente; baseline y V6 se conservan.

## Tres brazos contemporáneos

Mismas18preguntas/36proposiciones manuales revisadas, modelo Perplexity/revisión exacta y etiquetas. `joint` reproduce literalmenteV1; `role_source` retira solo las dos preguntas de respaldo; `role_only` retira después solo `state.source`. Pregunta/proposiciones/reglas/criterios/instrucciones/namespaces/modelo permanecen idénticos donde corresponda. No se evalúan solo los cuatro fallos. Las54solicitudes se comprobaron independientemente y las seis permutaciones se repitieron tres veces; un cliente secuencial, sin calentamientos excluidos.

Los brazos de rol no contienen respuestas de respaldo: no se fabrican ni se mezclan con votos históricos para declarar selección real. Excluir el campo fuente no borra texto que forma parte legítima de la proposición, que puede coincidir con la fuente; se estudia contexto adicional, no ausencia total de información de fuente.

| Diagnóstico sobre36roles | Conjunto | Solo rol con fuente | Solo rol sin campo fuente |
| --- | ---: | ---: | ---: |
| Roles correctos | 32 | 31 | 29 |
| Proposiciones no pertinentes aceptadas condicionalmente | 3 | 4 | 5 |
| Contribuciones útiles omitidas | 1 | 1 | 1 |
| Rol útil equivocado | 0 | 0 | 1 |
| Respaldo correcto | 36/36 | No se pregunta | No se pregunta |
| CosteUSD | 0,000412860 | 0,000322860 | 0,000312980 |
| Mediana ms | 419,643 | 409,797 | 402,336 |
| p95 ms | 11568,264 | 588,508 | 1129,721 |

Los cuatro errores originales persisten en los tres brazos: política de B3 frente a política común de clase; realización frente a fecha prevista; cita programada frente a ejecución; contraste C Duna/Loma descartado. Retirar la tarea añade una aceptación equivocada de cota ante pregunta por protocolo de medición. Retirar además la fuente añade otra aceptación de cota y confunde A/Ben la fuente exclusiva de agua. Hay1y2regresiones nuevas respectivamente, ninguna señal favorable y ninguna capacidad aprobada.

54/54respuestas válidas,0recibos ausentes/incertidumbres/reintentos. p95nearest-rank18es máximo; no se excluye la operación lenta `pcv1_control_18__joint` de11568,264ms, que respondió correctamente porHTTP2. Esto no identifica si la espera corresponde a red o proveedor. Coste totalUSD0,001048700. Las latencias son de llamadas de diagnóstico, sin baseline; no prueban recursos de `recall` ni del mecanismo completo. Una tanda no demuestra estabilidad ni un mecanismo causal interno.

## Cambio solo de modelo

La revisión del historial no encontró esta tarea conjunta con Decisions. Conserva sus antecedentes negativos como baseline y crítico de otros contratos. Se fijó antes de red un filtro de18preguntas/72juicios cambiando solo `model` al alias `openai/gpt-6-luna-decisions`, revisión exigida `openai/gpt-6-luna-decisions-20261006`. Estado/preguntas/proposiciones/instrucciones/criterios/orden/parser y puertas permanecen iguales. La prueba de payload restaura solo model y obtiene los18originales; el parser compartido cambia únicamente alias/revisión esperados, rechaza revisiones cruzadas y conserva la independencia de `unsupported + A`.

| Métrica de capacidad conocida | Decisions |
| --- | ---: |
| Respuestas válidas | 18/18 |
| Juicios de respaldo correctos | 33/36 |
| Respaldo falso aceptado / respaldo real rechazado | 0 / 3 |
| Roles correctos | 27/36 |
| Juicios independientes correctos | 60/72 |
| Selecciones correctas | 31/36 |
| Selecciones no pertinentes / contribuciones omitidas | 2 / 3 |
| Errores compensados | 0 |
| CosteUSD | 0,002461200 |
| Mediana / p95 ms | 502,518 / 1070,904 |

No hay pase aunque se cumpla el presupuesto monetario del filtro. Las pérdidas de selección son una cota inferior válida de autonomía, una refutación de autoría y el contraste C; acepta indebidamente un plazo individual como global y una cota como evidencia de si se midió hasta apagar. Los errores de rol restantes en proposiciones sin respaldo también cuentan, aunque no entren en la selección. No hubo fallos técnicos, importes desconocidos ni reintentos.

Diagnóstico retrospectivo separado: en tres pares con fuente y proposición idénticas, cambia el juicio de respaldo al cambiar la pregunta. La cita de Bruma, la cota inferior de Lino y las páginas pendientes de Atlas tienen respaldo verdadero en ambos miembros bajo la rúbrica; cada una recibe true/false. La corrección de respaldo no debería depender de la utilidad para la pregunta. Esto es evidencia de incumplimiento de esa invariancia en esta tanda, no prueba de una causa interna concreta. Está archivada sin cambiar etiquetas ni porcentajes. Decisions se ejecutó después del triple, sin controles intercalados propios: no se declara superioridad estable de familia ni efecto causal de modelo a partir de comparar tandas.

## Verificación y decisión

Los protocolos y payloads se congelaron antes de red. La revisión estática del crítico no encontró bloqueos, sin presentarla como pruebas ejecutadas. La primera preparación local detectó diferencia de representación float entre Value reconstruido y JSON releído; la serialización reconstruida ya era idéntica byte por byte al informeV1. Se corrigió antes de cualquier llamada comparando exactamente esos bytes originales, sin redondear importes, modificar reportes ni relajar puertas. La prueba también rechaza cambios de errores. El preparador fallido no creó directorio ni gastó API.

285pruebas pasan con y sinHTTP2; formato y compilación pasan. Se reproducen21informes y la auditoría de frames con dimensiones; los tres informes de proposiciones coinciden byte por byte. Los runners son manuales ignorados bajo `cfg(test)`; no cambian producto/default, dependencias, SQL, secretos, servicios, dashboard, README o gráficas. Snapshots001–044, bancos/gold originales y todos los hashes de V6 están intactos, con0llamadas aV6.

Tanda: **72operaciones/72HTTP,USD0,003509900,0costes nuevos desconocidos**. Acumulado: **5845operaciones,23campañas completas,USD0,581173038conocidos +5operaciones de importe desconocido**. Incluye todos los brazos, errores y latencias. Codex de programación/revisión queda fuera. Ningún piloto equivale a una campaña completa o calidad del producto.

Una auditoría conceptual adicional constató que el comprobador conjunto usa RULESv3pero no las ocho analogías de LocalV5. Cestá definido explícitamente: su omisión no invalida los fallos ni permite atribuir diferencias de tarea a ese detalle. La adición de esas analogías ya falló como solución general en GeminiV2→V3, pero no se ensayó en este comprobador. Como máximo justifica una ablación predeclarada con un solo modelo, ejemplos congelados de utilidad (no de respaldo), control contemporáneo y todos los72juicios; no basta recuperarCsi empeora respaldo, una parte o una refutación. Tampoco hay evidencia de que arregle política/planes. Sin un hallazgo distinto no corresponde encadenar más ejemplos ni modelos. El objetivo completo continúa pendiente, incluidas calidad/recursos conjuntos, runtime, estabilidad, campañas y casos nuevos ciegos.

[Protocolo de aislamiento](proposition-isolation-v2-protocol-2026-10-20.md), [filtro Decisions](proposition-decisions-v1-protocol-2026-10-20.md). Artefactos: `outputs/iromem/evaluation/proposition-isolation-v2-2026-10-20` y `proposition-contribution-decisions-v1-2026-10-20`. Contabilidad/huellas: `quality-goal-2026-10-19/snapshots/045`.
