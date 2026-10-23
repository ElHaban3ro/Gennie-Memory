# Diagnóstico factorial de utilidad por recuerdo

2026-10-19. **Goal activo; ningún pase de calidad ni promoción.** Se conserva Decisions/baseline y V6 sigue sin llamadas. El turno anterior produjo progreso comprobado (snapshot005): composición múltiple y variantes fallidas preservadas; esta tanda añade una prueba controlada de sus errores.

## Pregunta y método

Se compararon ocho pares deliberadamente elegidos entre datos conocidos: cuatro negativos difíciles y cuatro positivos (dos parciales, un apoyo y una refutación). Reutilizan seis preguntas, por lo que no son muestras independientes. Tareas: cuatro relaciones o útil/irrelevante. Contextos: un recuerdo aislado o los ocho del diario conocido. Proveedores: Jev y Decisions. Son64operaciones,288juicios sobre recuerdos,64peticiones y un recibo por petición. Una observación por celda; ninguna es reserva nueva.

La variante optativa `per_memory_binary_v1` conserva state e instructions de V2 y cambia solamente los criterios a utilidad/none. Devuelve `useful_evidence`: no inventa una categoría de apoyo/refutación/parcial. Cada resultado se revalida frente a ID/revisión y tarea. El evaluador exige fuente y texto originales, proyección exacta de etiquetas, pregunta constante dentro de cada par, las64celdas completas y candidatos idénticos entre tareas/proveedores de cada contexto. El cambio de contexto también cambia posición, otras preguntas y ámbito diagnóstico; los contrastes no identifican una causa interna.

## Resultados diagnósticos

«Objetivo correcto» evalúa utilidad del recuerdo objetivo; no acierto de la subcategoría ni respuesta generada. «Contaminadas» cuenta operaciones con cualquier ID no permitido en toda la semilla, aunque el objetivo sea correcto. Cada fila tiene8operaciones.

| Proveedor | Tarea | Contexto | Objetivos correctos | Falsos positivos del objetivo | Semillas contaminadas | USD |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Jev | Relaciones | Aislado1 |4/8 |4 |4/8 |0.000186774 |
| Jev | Relaciones | Con8 |4/8 |4 |6/8 |0.000532224 |
| Jev | Binaria | Aislado1 |5/8 |3 |3/8 |0.000174006 |
| Jev | Binaria | Con8 |5/8 |3 |4/8 |0.000430080 |
| Decisions | Relaciones | Aislado1 |7/8 |1 |1/8 |0.000261300 |
| Decisions | Relaciones | Con8 |7/8 |1 |1/8 |0.001317800 |
| Decisions | Binaria | Aislado1 |8/8 |0 |0/8 |0.000242100 |
| Decisions | Binaria | Con8 |7/8 |1 |1/8 |0.001164200 |

Los cuatro objetivos positivos se conservaron en todas las condiciones. Jev mantiene errores de sujeto/objeto/atributo incluso con recuerdo aislado y juicio binario, lo que contradice que todo se explique por acompañantes o por las cuatro categorías. La binaria corrigió el caso de ubicación de contador frente a sonda; quedaron horario de recogida frente a adquisición, carné frente a avalista y revisión de otro objeto. Decisions aislado/binario distingue los ocho objetivos observados; al acompañar vuelve a admitir el carné como evidencia sobre el avalista. Es una señal diagnóstica, no una garantía ni una prueba de generalización; no atribuir esos cambios exclusivamente al contexto con una sola observación.

## Coste, tiempo y alcance

Coste informado **USD0,004308484**, con64recibos completos y61356tokens de entrada reportados. Tiempos de operación de esta mezcla diagnóstica: mediana381,88ms, p95524,20ms y p99831,94ms. No son tiempos de una candidata completa ni una comparación de latencia contra base; cada celda tiene carga y modelo distintos. Aislado1 clasifica ocho recuerdos en ocho operaciones por fila; Con8 clasifica64recuerdos en ocho operaciones por fila. El importe de Aislado1 no representa el precio de examinar todos los candidatos de una consulta. No incluye Codex ni tiempo de una respuesta final.

## Fallo de preparación y auditoría separada

La preparación dejó null los auxiliares `expected_fact/expected_condition/expected_source` incluso cuando la proyección dejaba un solo ID permitido. **La auditoría ordinaria falló y sigue fallando** sobre ese archivo original. Fue un error del coordinador, no del clasificador. Se preservan matriz, casos, corpus, diario y huellas originales; no se cambió ningún conjunto de ID permitido ni consulta, ni se repitieron llamadas para corregirlo.

Se añadió una vía diagnóstica explícita: `report-per-memory-projection`. Primero exige hashes fuente, clones literales salvo ID/ámbito, consultas originales y etiquetas proyectadas exactamente; luego deriva en memoria los tres auxiliares all-null desde el único ID permitido y ejecuta las comprobaciones normales de eventos, ámbito, revisión, estado, fuente y semilla. Se auditaron288candidatos y79memorias de semilla. Un auxiliar no nulo incorrecto sigue siendo error; los casos/diario en disco no se reescriben. El informe y `audit-projection.json` señalan `strict_original_audit_passed:false`; se documentan36proyecciones auxiliares. Esto permite interpretar el diagnóstico, pero no transforma el intento original en una auditoría ordinaria válida ni en pase de calidad. Revisión estática separada no encontró debilitamiento de esas comprobaciones.

El preflight ordinario ya rechaza este defecto antes de futuras llamadas. Pruebas verifican null rechazado, proyección explícita sin mutar originales, fact no nulo incorrecto rechazado, matrices incompletas/duplicadas, cambios de pregunta/texto/gold, mezcla de tareas y contaminación escondida detrás de un objetivo correcto. **79pruebas workspace pasan**; fmt check pasa.

## Decisión y siguiente paso

No ampliar Jev/binario como solución: sigue fallando tres negativos del diagnóstico. Tampoco promover Decisions aislado por sus8/8: falta un mecanismo completo, contraste pareado, recursos y nueva validación. La siguiente hipótesis es propuesta económica más verificación aislada del recuerdo propuesto, conservando las partes compatibles; antes de implementarla hay que fijar recibos/modelos por etapa y medir el coste de todos los candidatos realmente verificados. Verificar cada uno de los ocho recuerdos por separado podría multiplicar llamadas y coste; esta matriz no demuestra que cumpla los límites.

V6 permanece congelada, con cobertura por partes y revisión separada, sin llamadas. No utilizarla para ajustar instrucciones. El goal conserva los cinco pasos y las puertas originales; quedan pendientes mejora de calidad demostrada, recursos, contraste nuevo y adopción.

Artefactos: [protocolo previo](per-memory-binary-v1-protocol-2026-10-19.md), [diseño congelado](../../outputs/iromem/evaluation/per-memory-matrix-2026-10-19/matrix.json), [rechazo original](../../outputs/iromem/evaluation/per-memory-matrix-2026-10-19/audit-error.json), [auditoría proyectada](../../outputs/iromem/evaluation/per-memory-matrix-2026-10-19/audit-projection.json), [resultados por operación](../../outputs/iromem/evaluation/per-memory-matrix-2026-10-19/report-projected.json), [procedencia de ejecución/reporte](../../outputs/iromem/evaluation/per-memory-matrix-2026-10-19/analysis-provenance.json).
