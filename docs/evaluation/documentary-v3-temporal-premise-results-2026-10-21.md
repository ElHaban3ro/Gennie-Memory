# Premisa de ocurrencia y coordenada temporal solicitada

Se corrigió una pérdida local demostrada: una pregunta por el año de un hecho ocurrido volvía a exigir ese mismo año para acreditar su premisa de ocurrencia. La CLI original reconocía el año, pero quedaba parcial por premise. Con exactamente la misma entrada manual y sin reparar su pregunta raw, ahora publica 2019 como respuesta completa y conserva el calificador year original. Es mejora del mecanismo en ese control; no una nueva métrica automática de recuperación.

## Contrato y corrección

La excepción solo sirve para una petición temporal occurred_at/occurred_year y una premisa fact/occurred del mismo átomo completo. La proposición fuente debe satisfacer las mismas comprobaciones existentes de identidad, polaridad, fundamento, juicio, estado, ámbito normativo, dependencias y elegibilidad. Las cadenas completas de scopes de fuente, premisa y petición deben ser documentales sin binding ni rama; no basta que el scope inmediato diga document.

Se admite exactamente una coordenada temporal conocida del rol solicitado; el año requiere format:year explícito. La pregunta y la premisa no pueden imponer otro calificador temporal. Si pair_scope fija un tiempo, debe coincidir exactamente en rol y término. Se omite esa coordenada solo de la comparación local de la premisa; los claims, testigos, calificadores y evidencias originales no se modifican. Otras configuraciones, guards, tiempos, incógnitas, planes y restricciones se conservan. No se proyecta una fecha o literal a año ni se instancian cuantificadores.

La primera corrección seguía fallando porque proposition volvía a comparar sus qualifier_ids internos. Se corrigió la vista local de esa comparación además del contexto exterior. El crítico señaló después el selector externo: se reprodujo una aceptación indebida de 2019 bajo selector 2020 y se corrigió, conservando el positivo 2019/2019. También señaló que un documento puede tener binder ancestral; se verifican ahora todos sus ancestros. Dos controles de ese escenario ya rechazaban antes del último cambio: no se afirma un bypass ejecutado. Se conserva como cierre de una precondición estática con límites explícitos.

## Evidencia ejecutada

| Verificación | Resultado |
| --- | --- |
| Workspace offline | 661 pasan, 24 omitidas, cero fallos |
| Nuevas pruebas del mecanismo | 5 pasan, con positivos y matriz de negativos |
| Motor V3 | 125 pruebas pasan |
| Controles CLI previos | 18 pasan |
| Entrada manual del diagnóstico anterior | parcial → completa; año 2019 y calificador conservados |
| Informes históricos | 226/226 idénticos en bytes |
| Build offline y formato | pasan |

Los controles cubren contexto A conservado, A/B distinto, tiempo desconocido, otro rol, formato literal en una petición de año, premisa negativa/distinta, año explícito diferente en la pregunta, plan frente a ocurrencia, rama y documento bajo binder. También comprueban selector externo coincidente/distinto. No son extracción ni revisión ciega de textos nuevos. Los fallos previos, la corrección intermedia insuficiente y las dos críticas se conservan.

Un contexto crítico separado produjo dos revisiones estáticas sin leer fixtures o resultados ni ejecutar pruebas. Recibió del coordinador descripciones de cambios y reproducciones, declaradas en sus informes. El dictamen final no identifica otro bloqueo concreto en el helper; no aprueba calidad global, semántica de extracción ni rendimiento de producción. Se conserva el informe inicial y se distingue el hallazgo ancestral no reproducido del selector ejecutado.

## Límites y siguiente trabajo

Esta corrección no resuelve la discusión sobre dependencias de referencias dentro de calificadores. Hay que definir y comprobar el requisito directo/transitivo del propietario y su resolución efectiva antes de cambiar validador o criterios de revisión. Tampoco resuelve la selección de atributos demasiado específicos, el cierre de configuraciones ni las citas de otra cláusula de la última extracción. Una configuración/guard satisfecho solo en el ámbito externo todavía puede perderse conservadoramente en la comparación interna de proposiciones. Fechas completas, múltiples coordenadas y planes no reciben esta excepción de ocurrencia; quedan bajo sus reglas anteriores.

La última tanda automática conserva íntegros 80/116 documentos completos, 36 unavailable tras interrupción y 72/116 aceptados por ambos críticos; cinco desacuerdos y puerta fallida. No se reescribe su diagnóstico temporal original ni se cuenta el control manual como recuperación de fidelidad. No hay nueva utilidad automática, baseline, recursos/estabilidad o V6 y no se promueve el mecanismo al producto. El objetivo continúa en paso 3 con el alcance completo de los cinco puntos.

## Coste y trazabilidad

Cero llamadas nuevas de proveedor y USD 0 de API. Un contexto crítico, dos turnos; coste y tokens Codex desconocidos, no gratuitos ni parte del ledger. Acumulado sin cambios: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos, excluyendo Codex. Los tiempos de tests/CLI no son latencia de producción.

Artefactos: outputs/iromem/evaluation/documentary-v3-temporal-premise-2026-10-21/. Snapshot 096 conserva fuente, pruebas, revisiones, fallos y verificaciones. Los paquetes, raws, etiquetas, críticas y snapshots anteriores se verificaron por hash e inventario y permanecen intactos.
