# Fragmentación textual: preflight sin implementación

Una revisión crítica nativa separada, sin inferencias ni cambios de etiquetas, contrastó la [propuesta source_text_edges_v1](source-text-evidence-candidate-protocol-2026-10-22.md) con las16preguntas/64fuentes conocidas de `evidence-contribution-v3`. Raíz reprodujo los conteos manteniendo delimitadores y comprobando concatenación exacta.

La regla corta en salto de línea o después de `.?!;` seguido de whitespace; no divide por «y». Las16preguntas tienen1segmento. De64fuentes,56tienen1segmento y8tienen2:72aristas y216juicios binarios por clasificador en este banco. Las fuentes divididas son cv3_02_01,03_00,04_00,08_01,11_00,13_00,15_00y16_00.

Por tanto esta segmentación no modifica la unidad de los errores unicláusula de teléfono ilegible, T6, no-red y otro puente; tampoco separa las partes de la pregunta multipartes. Añadir consenso y tres juicios por arista no demuestra una causa que corrija esos errores. Decisions ya falló31/36selecciones con proposiciones manuales; aislar preguntas LocalV5 tampoco eliminó sus fallos. Los controles correctos mínimos no acreditan la generalización difícil.

**Decisión de inversión: detener esta propuesta en preflight por falta de justificación causal suficiente para esos fallos.** No se implementa, no se paga una campaña y no se afirma haber medido su calidad o demostrado inviabilidad universal. Las8fuentes multisegmento podrían permitir otra hipótesis acotada, pero eso exigiría evidencia específica y presupuesto, no promesas por fragmentar.

La revisión no cambia el banco, puertas, resultados históricos o reservaV6. Cero llamadas/coste API nuevo en este análisis; Codex no cuantificado. Evidencia reproducible y hashes de las fuentes: `outputs/iromem/evaluation/qwen9b-reasoning-total-deadline-v2-2026-10-22/source-edges-offline-preflight.json`. No constituye un paso3aprobado.
