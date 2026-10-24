# 0049 — Presentar evidencia explícita al baseline

Fecha: 2026-10-23. Estado: aceptada e implementada por solicitud explícita del usuario.

El caso de Aliso recuperaba el recuerdo correcto, pero Decisions se abstenía al recibir un contexto cauteloso. El diagnóstico aisló el efecto del payload: los textos aparecían solo como opciones de relevancia. Añadirlos a `state.memories` resolvió el caso sin retirar el contexto. El usuario autorizó aplicar la corrección y ejecutar 30 recuperaciones.

El adaptador baseline de Decisions/Jev/Jev114 añade `state.memories`, con las mismas etiquetas/textos y excluyendo `none`. Mantiene instrucciones, opciones y parser. Se aplica tanto a recall heredado como a recall-v1/session; no cambia la política predeterminada, la ruta Gemma ni las políticas experimentales. `roles_fallback_v1` mantiene su payload inicial histórico. La revisión se identifica como `baseline_explicit_evidence_v1` en la evidencia de evaluación; no se inventa una política nueva ni roles/razones ausentes.

El módulo de payload tiene regresión que falló antes del cambio y pasó después. El evaluador añade `run-baseline-regression`, aislado fuera del producto, para un único banco de desarrollo ligado por hashes, con DB propia, presupuestos, recibos y diarios durables. Reutiliza el runner/comprobador de semillas; no fuerza tres particiones artificiales ni abre la reserva final consumida.

[Los 30 casos](../evaluation/baseline-evidence-30-results-2026-10-23.md) pasaron con Decisions: 20 positivos y 10 negativos, incluido Aliso; 28 llamadas, USD 0,0008273. Las etiquetas son sintéticas del mismo autor y no están revisadas independientemente. No se establece calidad general ni rendimiento de Jev/Jev114. El payload crece al duplicar evidencia; las métricas anteriores de 540 preguntas y las figuras históricas no miden esta revisión.
