# 0018 — Selección experimental por utilidad de evidencia

Fecha: 2026-10-18. Estado vigente: promoción descartada tras el contraste v2; permanece experimental para reproducir resultados. El primer desarrollo pasó su filtro, sin validación general.

La rúbrica prospectiva distingue respuesta, refutación, contexto parcial y ausencia. La línea base omitió evidencia correctiva y parcial pese a su cobertura. Se incorpora `selection_policy: utility_v1` explícita, con una sola elección remota e instrucción y opción `none` orientadas a utilidad de evidencia. Preserva las reglas de autorización de candidatos y composición; no infiere ni guarda hechos nuevos. No se modifica `baseline`, no se añade un gate ni se reutiliza como aprobada la política histórica `evidence_v1`.

El [protocolo previo](../evaluation/utility-v1-paired-protocol-2026-10-18.md) bloquea pérdidas, selecciones no permitidas y degradación nueva. El [resultado emparejado](../evaluation/utility-v1-paired-results-2026-10-18.md) en 64 preguntas conocidas obtuvo 16/16 respuestas, 15/16 refutaciones, 16/16 contextos parciales y 16/16 ausencias; la base obtuvo 16/16, 8/16, 12/16 y 16/16. Se recuperaron once evidencias sin regresiones observadas, con 30,66 % más tokens y coste reportado en la variante. Persiste una refutación omitida. No se ajusta para rescatarla.

La mejora es de selección conforme a etiquetas sintéticas del mismo autor, no de verdad de respuesta final. La revisión independiente y validación nueva siguen pendientes. El filtro permite proponer ese siguiente contraste; no promover, actualizar cifras principales del README ni declarar viabilidad general. El cierre de M5 en 0017 permanece vigente.

`tools/evaluation` prepara pares alternados, exige política explícita por caso, conserva etiquetas y genera reporte contra una auditoría íntegra con listas de candidatos idénticas. `per_case` es una opción del runner, no del contrato del producto. No se agregan dependencias, conexiones, servicios, puertos ni tareas en segundo plano.

## Contraste nuevo del 2026-10-18

La [cohorte v2 con revisión ciega entre agentes](../evaluation/utility-v1-cohort-v2-results-2026-10-18.md) mantuvo las instrucciones congeladas y amplió a 224 recuerdos en ocho ámbitos. Base y variante conservaron 16/16 respuestas y 16/16 refutaciones. La variante elevó contexto parcial de 13/16 a 16/16, pero redujo abstenciones por ausencia de 16/16 a 13/16 al seleccionar datos de otra edición, equipo y entorno. Son tres bloqueos fijados antes de ejecutar: no se promueve ni se compensan con las ganancias. Se conservan fuentes, anotaciones iniciales/finales, conciliación, diarios y gráficas separadas. La revisión es de agentes de la misma familia, no humana. Las semillas conservan los calificadores originales; no se midió una respuesta falsa de Codex.

La siguiente hipótesis debe comprobar compatibilidad del referente antes de aceptar evidencia parcial. No se incorpora otro ajuste en esta campaña ni se reabre esta cohorte como reserva. El cierre limitado de M5 y el predeterminado siguen vigentes.
