# Respaldo correcto no basta para seleccionar evidencia pertinente

Fecha UTC2026-10-20. El diagnóstico de proposiciones suministradas **falla**:36/36juicios de respaldo correctos,32/36roles correctos y33/36selecciones correctas. La selección acepta dos proposiciones no pertinentes y pierde un contraste útil, incluso sin errores de extracción. **No se construye un extractor ni se amplían campañas para esta versión.** Goal activo, paso3pendiente; no cambia baseline ni se abre V6.

## Hipótesis y revisión previa

El historial crítico descartó otro ajuste de redacción sobre alcance/planes y otra extracción genérica de campos: ya se habían probado. Se separaron dos obligaciones sobre una proposición concreta preparada por el autor: la fuente debe respaldarla íntegramente y la proposición, suponiéndola verdadera, debe aportar a la pregunta según A/B/C/none. Una afirmación puede ser verdadera e irrelevante o ser una buena respuesta hipotética sin respaldo. El pase exige ambos juicios correctos; errores compensados en la selección también lo bloquearían.

El banco nuevo contiene18preguntas/36proposiciones:16pares de cuantificación, atributo, modalidad, límites, referente, refutación, contraste y exclusividad, más dos controles con ambas proposiciones respaldadas o ambas sin respaldo. No se declara cuota al modelo. Los IDs, etiquetas, factor alterado y revisiones no entran al payload. Ninguna proposición procede de un extractor automático; no se mide su capacidad ni coste.

Un crítico separado recibió solo fuentes/preguntas/proposiciones y contratos, sin etiquetas ni resultados. La primera revisión encontró cinco ambigüedades y un desacuerdo de rol: política común frente a lista de ítems, verificar frente a medir, duración de una ejecución frente a autonomía, alcance de exclusividad y pertinencia normativa de una fuente intermedia. Se aclararon los cuatro primeros antes de red; para la fuente intermedia se mantuvo la regla explícita A del contrato v3, sin exclusividad ni tubería conocida. La revisión final concuerda en72juicios, incluyendo los controles. Se conservan borradores, objeciones, cambios y revisiones. Es revisión automatizada de la misma familia, no humana; las restricciones de lectura son declaradas, no aislamiento de archivos. La concordancia y los hashes no prueban verdad semántica.

## Ejecución y resultado completo

Modelo `perplexity/pplx-decider-v1.2`, revisión `perplexity/pplx-decider-v1.2-20261016`. Una petición por pregunta, dos afirmaciones y cuatro choices independientes de respaldo/rol. Las preguntas comparten fuente/proposiciones en el mismo contexto: la separación de obligaciones no garantiza aislamiento semántico. Código, solicitudes, banco y protocolo quedaron ligados por hashes antes de llamar. Se ejecutaron18/18respuestas válidas sin reintentos, fallos, incertidumbres o recibos ausentes; cada recibo se suma una sola vez.

| Métrica de desarrollo conocido | Resultado |
| --- | ---: |
| Juicios de respaldo correctos | 36/36 |
| Proposiciones sin respaldo aceptadas como respaldadas | 0 |
| Proposiciones respaldadas rechazadas | 0 |
| Roles correctos, suponiendo verdadera la proposición | 32/36 |
| Juicios independientes correctos | 68/72 |
| Selecciones correctas `grounded && role!=none` | 33/36 |
| Selecciones no pertinentes / contribuciones omitidas | 2 / 1 |
| Errores compensados | 0 |
| Coste informado | USD0,000412860 |
| Mediana / p95 de operación | 417,187 / 912,505ms |

El p95nearest-rank de18observaciones es el máximo; se incluyen todos los intentos terminados. No hay control baseline: no se declara mejora de velocidad, coste relativo ni cumplimiento de recursos del mecanismo completo. Las cifras pertenecen al comprobador con proposiciones manuales, no a recuperación automática, `recall`, respuesta final o generalización.

Los cuatro desacuerdos son:

| Caso/proposición | Esperado → observado | Consecuencia |
| --- | --- | --- |
| `pcv1_01_02/claim_01`: política de retención de B3 ante plazo común de toda la clase | none→A; respaldo correcto | Selección no pertinente por ampliar alcance |
| `pcv1_03_01/claim_01`: medición realizada ante fecha programada | none→B; falta de respaldo correcta | Rol incorrecto aunque el filtro de respaldo evita seleccionarla; realización y programación pueden coexistir |
| `pcv1_03_02/claim_01`: medición programada ante si se realizó | none→A; respaldo correcto | Selección no pertinente por confundir plan y ejecución |
| `pcv1_07_01/claim_00`: Duna/Borrador y Loma/Fino ante Duna/Fino | C→none; respaldo correcto | Pérdida del contraste útil sin exclusividad |

Los dos controles sin cuota se clasificaron correctamente. Pasar respaldo36/36no demuestra que ese componente sea perfecto fuera de este banco. La evidencia sí refuta la suficiencia de esta combinación: los errores de utilidad persisten con afirmaciones ya suministradas y con respaldo acertado. Un nuevo extractor no resolvería por sí solo esa puerta. No se cambia ningún texto, etiqueta o rol después de esta ejecución.

## Implementación, verificación y coste

El código nuevo vive en la herramienta de evaluación y un runner manual ignorado bajo `cfg(test)`. `validate-proposition-contribution` comprueba manifiesto, contratos, cobertura de revisiones, IDs, citas, enums y ausencia de discrepancias pendientes. `prepare-proposition-contribution` genera solicitudes sin etiquetas. `report-proposition-contribution` recompone respuestas/selección y desglosa ambos errores de respaldo y los roles. Ninguno es prueba determinista de implicación del lenguaje natural.

La revisión estática detectó cobertura incompleta preparation→preflight en el primer auditor; se corrigió antes de red y una prueba rechaza mapas de fuentes vacíos, parciales y hashes cambiados. Otras pruebas rechazan namespace, revisión, probabilidades y citas alteradas y muestran que errores de respaldo/rol compensados bloquean éxito.274pruebas pasan con y sin HTTP2; formato y compilación pasan. El informe nuevo y los18anteriores se reproducen desde diarios; también la auditoría de frames con dimensiones. Bancos históricos y snapshots001–043permanecen íntegros. V6 conserva todos sus hashes y cero llamadas.

Tanda:18operaciones/18HTTP,USD0,000412860,0costes nuevos desconocidos. Acumulado: **5773operaciones,23campañas completas,USD0,577663138conocidos +5operaciones de importe desconocido**. Codex de programación/revisión queda fuera. No hubo servicio/puerto, SQL, conexión, dependencia, credencial nueva ni proceso del usuario modificado. README/gráficas mantienen la ruta vigente porque no existe promoción.

El siguiente diagnóstico razonable es comprobar si disponer de la fuente y de la tarea de respaldo contamina la clasificación condicional de la proposición. Exigiría un contraste predeclarado sobre todo el banco, con controles contemporáneos y aislamiento real de datos/tareas, sin repetir hasta acertar ni ajustar los cuatro fallos. Todavía no es un mecanismo implementado ni una mejora validada; sus llamadas y costes se sumarían. Si se ensaya otra vía, debe justificar qué obligación cambia respecto de estos fallos y cómo preserva C y los parciales. No abrir V6 ni adoptar antes de todas las puertas originales.

[Contrato](proposition-contribution-v1-contract.md), [protocolo previo](proposition-contribution-v1-protocol-2026-10-20.md). Banco: `tests/fixtures/proposition-contribution-v1`. Artefactos: `outputs/iromem/evaluation/proposition-contribution-v1-2026-10-20`. Cierre de huellas/contabilidad: `quality-goal-2026-10-19/snapshots/044`.
