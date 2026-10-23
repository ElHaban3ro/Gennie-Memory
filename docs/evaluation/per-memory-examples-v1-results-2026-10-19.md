# Ejemplos contrastivos: sin mejora medida

Estado2026-10-19: desarrollo sintético conocido; calidad fallida, recursos aprobados, sin ampliación ni promoción. [Protocolo previo](per-memory-examples-v1-protocol-2026-10-19.md).

`per_memory_examples_v1` añade seis demostraciones generales aV5 para distinguir atributo solicitado de requisito asociado, corrección explícita, parcial útil, preparación sin resultado y contexto distinto. Mantiene Jev1.13, candidatos, tarea binaria, una petición y composición de todos los positivos. La prueba compara payloads completos tras retirar exclusivamente ejemplos/regla añadidos. No hay cambio de defaults, dependencias, SQL ni servicios.98pruebas workspace pasan y formato correcto.

Piloto20operaciones/diez pares alternados, mismas listas de candidatos y etiquetas originales. Ambas políticas seleccionaron exactamente los mismos recuerdos: cero IDs útiles ganados, cero perdidos y cero selecciones no permitidas eliminadas. Persiste `cev5_071_03`, carné presentado ante la pregunta por avalista; sí se conserva la negación explícita correcta `cev5_071_08`. Un dato asociado no se convierte en corrección por coexistir con otro recuerdo correctivo. No se cambiaron gold, fuentes o preguntas ni se descontaron las discrepancias retrospectivas de contexto.

| Medición del runner | ControlV5 | Con ejemplos |
| --- | ---: | ---: |
| Operaciones terminadas |10|10|
| Degradaciones |0|0|
| Coste reportado USD |0,000566076|0,000762216|
| Mediana ms |395,26|358,40|
| p95 ms |975,67|402,51|

La candidata cuesta1,3465× y pasa recursos diagnósticos, pero **no mejora calidad**. Su menor cola en esta muestra no demuestra superioridad general de latencia ni compensa el error persistente. No es comparación contra Decisions/baseline, cohorte nueva ni respuesta final de Codex.

Se añadió `report-policy-probe` en Rust: exige20casos exactos ligados al banco original, orden alternado, auditoría ordinaria sin proyección, hashes vigentes, binario/base previos, mismos candidatos, modelo/semilla/trazas completas y diferencias de todos los IDs. Las ganancias no compensan pérdidas ni contaminación. El primer análisis se detuvo por asumir que seed.classifier era la política; el contrato actual guarda el modelo. Se corrigió solo el reporte, se conservó el fallo de análisis y se usó el mismo diario, sin repetir red. La política por caso queda vinculada mediante casos y binario congelados; el producto no expone una etiqueta independiente de política en la respuesta.

[Reporte Rust](../../outputs/iromem/evaluation/per-memory-examples-v1-pilot-2026-10-19/report.json), [auditoría](../../outputs/iromem/evaluation/per-memory-examples-v1-pilot-2026-10-19/audit.json), [corrección del generador](../../outputs/iromem/evaluation/per-memory-examples-v1-pilot-2026-10-19/reporting-failure.json).

Coste nuevoUSD0,001328292, recibos completos. Acumulado2585operaciones/17campañas emparejadas, USD0,181793416 conocidos y dos operaciones con importe desconocido, excluido Codex. Las parejas de este diagnóstico no se cuentan como otra campaña completa. DB temporal propia eliminada, sin puertos ni cambios al dashboard. V6 permanece congelada y sin llamadas.

La versión se detiene antes de campaña completa. El fallo debilita seguir añadiendo instrucciones abstractas o ejemplos al mismo juicio global de utilidad. Una próxima prueba deberá aislar preguntas de significado más acotado y otro tipo de tarea, medirlas antes de incorporarlas y conservar las puertas; no bastará cambiar etiquetas, exigir más confianza ni repetir la misma política hasta que acierte. El objetivo sigue abierto: faltan mecanismo aprobado, contraste nuevo y adopción con mejora demostrada.
