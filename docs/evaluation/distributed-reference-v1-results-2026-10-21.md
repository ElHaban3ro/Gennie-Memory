# Referencias distribuidas V1: resultado local

UTC 2026-10-21. Paso 3 del objetivo activo. **El prototipo resuelve dos límites de representación, pero falla la puerta semántica y no se adopta.** No se ha medido mejora automática de recuperación. [Protocolo](distributed-reference-v1-protocol-2026-10-21.md), [matriz ejecutada](../../outputs/iromem/evaluation/distributed-reference-v1-2026-10-21/matrix-final.json) y [conciliación ciega](../../outputs/iromem/evaluation/distributed-reference-v1-2026-10-21/review-reconciliation.json).

## Qué se implementó

Un adaptador exclusivamente de evaluación conserva la mención propia y permite evidencia separada de cada vínculo de propietario. Comprueba portador, orden de eslabones, estado afirmado, rol fijo, condiciones y aplicabilidad declaradas. Una equivalencia de clase explícita ligada a un registro por hash permite representar sensor/sensores sin recortar palabras ni derivar alias de gold. La afirmación de equivalencia sigue siendo falible.

Se extrajo un callback en el traductor compartido; su entrada antigua usa el reescritor original. El nuevo resultado conserva el raw completo en un sidecar y declara semantic_assertions_verified=false. El catálogo sustituye referencias exactamente; el parser y el resolver de hechos se reutilizan. No se implementó un extractor de modelo ni un runner remoto nuevo.

## Resultados y límites

| Comprobación | Resultado | Interpretación |
| --- | --- | --- |
| Ocho controles originales adaptados | 8/8 condiciones mecánicas | Antes eran 6/8 con el contrato previo; cambia la representación manual, no demuestra mejor extracción |
| Ocho controles aclarados prospectivamente | 8/8 condiciones mecánicas | Cambian textos ambiguos explícitamente; no es la misma cohorte |
| Controles de alcance declarados | 13/13 resultados estructurales | Doce errores en grounded_translation y una observación acotada aceptada; motivos comprobados por separado |
| Contraejemplos semánticos | 1/3 cumple su condición necesaria | Dos falsos positivos deliberados permanecen |
| Revisión ciega por crítico | 14 fieles, 4 defectuosos, 1 incierto, de 19 | Ambos críticos coinciden en los veredictos; discrepan en calificación en tres casos |
| Suite workspace | 476 pruebas con y sin HTTP/2 | Ocho pruebas nuevas; comprueba código, no calidad poblacional |
| Conservación histórica | 4 bancos y 10 informes idénticos en bytes | Auditores previos de cobertura y catálogo también idénticos |

Los dos falsos positivos son una equivalencia falsa entre sensores de presión y sensores generales, y un contexto que declara hoy aunque su cita dice ayer. Citas literales y acuerdo de campos no prueban que el contenido esté respaldado. Ambos casos se conservan en el agregado, que falla. No se ocultan como errores de esquema ni se seleccionan únicamente los casos que pasan.

Los críticos recibieron solo 19 pares con IDs anónimos, textos, registro y construcciones manuales. No vieron expectativas, resultados, mapa, V6 ni historial. Son agentes de la misma familia, no revisión humana ni independencia de familia. En los originales señalan atribución de propiedad no respaldada (LA03), observación/componente no explícitos (LA06) e interpretación incierta del recorrido del agua (LA07). El lote aclarado tiene ocho acuerdos fieles; ello valida esas construcciones manuales, no la habilidad de un extractor. Se conserva la discusión previa de azul frente a roja sin exclusividad declarada.

La revisión de código halló una reducción incidental de claves de contexto de 96 a 64 bytes: corregida separando IDs de clase y claves de condiciones, con regresión para 65/96/97 bytes. Los dos informes estáticos anteriores quedan archivados. No quedan bloqueos incidentales identificados por el crítico en su alcance; sí límites que bloquean adopción: calificadores de propietarios homónimos pueden perder efecto en la identidad normalizada, intervalos de varias solicitudes usan el scope global y la semántica de citas/valores no está certificada. Los guards del evaluador solo comparan normalización/ausencia; la verificación de esta corrida también exige etapa y motivo concretos, sin afirmar que su implementación tenga esa cobertura general.

Se conservan los primeros fixtures e informe fallidos. Una expectativa nueva redundante sobre qualification_required de LA02 aclarado fue adjudicada antes de revisión, sin cambiar B ni el resolver. El fallo del generador que alteraba conjuntamente intervalo de soporte y uso se corrigió y su caso original se preserva como contraejemplo falso, no como rechazo aprobado.

## Coste, estado y siguiente trabajo

Cero operaciones/HTTP nuevos y **USD 0 de proveedor** en esta ronda local. Acumulado de investigación: 6001 operaciones lógicas, 23 campañas emparejadas completas, USD 0,845298508 conocidos y 16 operaciones con coste desconocido. El importe completo no puede calcularse; Codex no está incluido. No hay medición nueva de tokens, latencia de recall, recursos o recorrido completo.

El paso 3 continúa abierto. La siguiente versión necesita comprobar la correspondencia entre clase/intervalo y cita, preservar calificadores e intervalos por uso, y añadir controles de homónimos/solicitudes múltiples. Debe mantener salida fuera de cobertura explícita y conservar evidencia parcial/correctiva. No basta añadir otra revisión que repita el mismo JSON. Antes de red: diseño criticado, matriz completa y revisión; después, extracción en conocidos, contraste con base, recursos/estabilidad y únicamente entonces cohorte nueva ciega. Esta hipótesis aún no está implementada.

No se modifica el predeterminado, README/gráficas, producto, dashboard, SQL, dependencias ni servicios. V6 permanece sin consumir y snapshots 001–059 intactos. El objetivo de cinco pasos sigue activo; esta ronda no cumple su objetivo final de mejorar calidad automáticamente.
