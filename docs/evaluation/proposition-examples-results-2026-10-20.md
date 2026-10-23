# Las analogías recuperan C, pero no superan la puerta completa

Fecha UTC2026-10-20. La candidata mejora de68/72a69/72juicios correctos, recuperando el contraste Csin regresiones nuevas. **Falla capacidad**: persisten3errores de rol y2selecciones no pertinentes. La señal exploratoria sobre Cno se convierte en pase de calidad del producto ni permite construir un extractor para esta versión. Se cierra esta vía de ejemplos congelados. Goal activo, paso3pendiente; baseline y V6 intactos.

## Cambio controlado y ejecución

18preguntas/36proposiciones manuales del mismo banco revisado, sin cambiar fuentes, preguntas, afirmaciones o etiquetas. Control=V1conjunto literal con Perplexity; candidata añade únicamente `state.contribution_examples` con las ocho tripletasLocalV5literales y `state.contribution_examples_scope` para aclarar que enseñan utilidad condicional, no respaldo ni hechos de la consulta. La restauración de esos dos campos iguala cada candidata a su control. Las tripletas se compararon además contra el artefactoLocalV5anterior, no solo contra el builder actual.

Mismo alias `perplexity/pplx-decider-v1.2`, revisión requerida `perplexity/pplx-decider-v1.2-20261016`, parser de cuatro choices, reglas, namespaces y probabilidades. Se ejecutaron18pares contemporáneos alternando nueve veces cada orden, un cliente secuencial sin warmups excluidos. Las36solicitudes/código/protocolo/ejecutablehash se ligaron antes de red; no se pasaron IDs, etiquetas, factores ni resultados del banco.36/36respuestas válidas, sin reintentos, fallos, incertidumbres o cargos ausentes.

La revisión estática crítica no encontró bloqueos; no se presenta como ejecución de pruebas. El preparador reconstruyeV1y exige los mismos bytes del informe original antes de escribir. El auditor recompone todos los payloads, orden, respuestas, selecciones y recibos desde los archivos ejecutados. No se cambia gold tras medir ni se atribuye una mejora a otras diferencias entre tandas históricas.

## Métricas completas del diagnóstico

| Métrica conocida | Control | Con ocho analogías |
| --- | ---: | ---: |
| Juicios independientes correctos | 68/72 | 69/72 |
| Respaldo correcto | 36/36 | 36/36 |
| Roles correctos | 32/36 | 33/36 |
| Errores en el C esperado | 1 | 0 |
| Selecciones correctas | 33/36 | 34/36 |
| Selecciones no pertinentes | 2 | 2 |
| Contribuciones útiles omitidas | 1 | 0 |
| Regresiones nuevas por juicio frente a control | — | 0 |
| CosteUSD | 0,000412860 | 0,000509700 |
| Mediana ms | 416,007 | 421,628 |
| p95 ms | 1891,323 | 489,442 |

p95nearest-rank18es el máximo; se incluyen todos los intentos terminados, sin excluir el control lento. No hay baseline de producto ni medición de `recall`: los cocientes entre estos dos brazos no prueban el presupuesto de una cadena automática. Coste totalUSD0,000922560, con ambos brazos y todos sus recibos una sola vez.

La única diferencia de juicio es `pcv1_07_01/claim_00`: las asignaciones Duna/Borrador y Loma/Fino pasan de noneaCpara la consulta Duna/Fino. Eso conserva el contraste explícito sin afirmar exclusividad. Cumple la señal estrecha predeclarada: menos errores de Csin ninguna regresión en los72juicios. No demuestra estabilidad, causalidad interna, superioridad de familia o generalización. El antecedente negativo de añadir estas analogías en GeminiV2→V3permanece intacto; no se presenta como una solución general.

Persisten exactamente los otros tres desacuerdos de ambos brazos:

- `pcv1_01_02/claim_01`: política de un archivo B3ante política común de toda la clase,none→A,con respaldo correcto. Selección no pertinente por ampliar alcance.
- `pcv1_03_01/claim_01`: evento realizado ante fecha programada,none→B,con falta de respaldo bien detectada. El filtro de respaldo evita seleccionarlo, pero el rol incorrecto cuenta.
- `pcv1_03_02/claim_01`: evento programado ante si se realizó,none→A,con respaldo correcto. Selección no pertinente por tratar un plan como ejecución.

Los dos componentes se miden por separado; que una proposición sin respaldo no entre en selección no perdona un rol equivocado. La candidata necesitaba72/72y36/36selecciones, no solo recuperar C. **No pasa** y no se continúa ajustando ejemplos/modelos para esta explicación.

## Verificación, coste y siguiente vía

289pruebas de workspace pasan con y sinHTTP2, formato y compilación pasan. Los21informes anteriores y este nuevo informe se reconstruyen; también la auditoría de frames con dimensiones. Bancos históricos, snapshots001–045y todos los hashes de V6 permanecen íntegros, con cero llamadas aV6. El banco de proposiciones sigue siendo manual, de desarrollo conocido y revisado por agentes de la misma familia, no por humanos; fuentes/hashes/concordancia no prueban verdad semántica ni extracción automática.

Tanda: **36operaciones/36HTTP,USD0,000922560,0importes desconocidos nuevos**. Acumulado: **5881operaciones,23campañas completas,USD0,582095598conocidos +5operaciones de importe desconocido**. Codex de programación/revisión no está incluido. No hay nueva política runtime, dependencia, SQL, conexión, credencial, servicio/puerto o proceso del usuario modificado. README/gráficas mantienen la ruta vigente.

La auditoría histórica adicional encuentra un límite entre módulos aún no ensayado: extracción de hechos tipados → resolver que calcula la relación sin un matcherLLM. Las proyecciones previas componían relaciones/same_case/destinos ya declarados por un modelo; cambiar IDs/conjuntos no añadía una lógica de cuantificación o modalidad. La igualdad literal de witnessV1perdió correcciones y partes y sigue siendo un antecedente adverso.

Para que sea una vía distinta, los hechos no podrán contener role/useful/same_case/target_requested_slotni otros equivalentes que escondan la decisión final. El código deberá derivar el rol de referente, propiedad, cuantificador, modalidad, polaridad, condiciones/tiempo y límites. Primero corresponde probar **sin API** suficiencia de gramática y resolver sobre frames manuales revisados: cubrir todas las familias (incluidos parcial/pendiente,B,C), contar como falta de cobertura lo no representable, buscar colisiones de frames con roles diferentes y comprobar invariantes de renombrado/orden. Una colisión o fallo con frames correctos refuta temprano esa representación; ningún ID del banco puede intervenir en reglas. Un pase manual tampoco probaría extracción automática, recursos o mejora del producto. Es una propuesta de siguiente diagnóstico, todavía sin implementación ni resultado.

[Protocolo previo](proposition-examples-v3-protocol-2026-10-20.md). Artefactos: `outputs/iromem/evaluation/proposition-examples-v3-2026-10-20`, incluido el historial crítico del posible resolver. Contabilidad y huellas: `quality-goal-2026-10-19/snapshots/046`.
