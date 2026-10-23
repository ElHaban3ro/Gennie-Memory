# Extracción documental V3 congelada: resultado fallido

Se completaron los 116 textos conocidos (83 fuentes y 33 preguntas), con una entrega por extractor y sin reparar respuestas tras ver el validador. Diez tareas de extracción y veinte revisiones separadas produjeron 928 juicios. Las tareas usaron contextos separados de la misma familia de modelos; no constituyen revisión humana ni independencia de familia. No recibieron frames manuales, etiquetas ni resultados como entrada de extracción.

| Medida | Resultado |
| --- | ---: |
| Documentos estructuralmente completos | 112/116 (96,55 %) |
| Parciales / inválidos / ausentes | 4 / 0 / 0 |
| Cuatro dimensiones fieles para ambos críticos | 65/116 (56,03 %) |
| Fuentes aceptadas por ambos | 45/83 |
| Preguntas aceptadas por ambos | 20/33 |
| Desacuerdos de etiqueta entre críticos | 46 |
| Revisiones con integridad de entrega correcta | 20/20 |

**La puerta de fidelidad y la puerta completa fallan.** No se elige el crítico favorable ni se sustituyen sus juicios mediante arbitraje. Ambos revisores consideran preservada la operación raw→documento validado en las 116 entradas: las pérdidas principales aparecen al traducir texto a raw. Esto no demuestra corrección del significado elegido ni utilidad de recuperación.

Se observaron genitivos ambiguos convertidos en propiedad, ubicación o membresía; usos concretos elevados a perfiles; sustancias confundidas con métodos; restricciones conservadas solo como menciones; historia omitida; origen confundido con vía de entrada y portadores de acciones disponibles pero no utilizados. Los cuatro parciales son question022 y source013/037/047. Las causas y discrepancias completas permanecen en artefactos separados.

El intento automático anterior normalizaba 102/116 y recibía aceptación conjunta en 93/116. Cambian representación y contrato de revisión: no es una comparación causal ni poblacional. El resultado actual no acredita una mejora. Tampoco el cierre manual previo de nueve carencias equivale a extracción automática.

La coordinación duró aproximadamente 24,74 minutos; no mide latencia de producto. Cero llamadas OpenRouter nuevas y USD 0 de API. Tokens y coste de Codex desconocidos. El acumulado de proveedor sigue en 6129 operaciones, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 operaciones históricas de coste desconocido, excluyendo Codex.

Se conservan protocolo, entregas originales, revisión doble, matriz y manifest previo. [Informe medido](../../outputs/iromem/evaluation/documentary-v3-frozen-extraction-2026-10-21/report.json), [hallazgos](../../outputs/iromem/evaluation/documentary-v3-frozen-extraction-2026-10-21/findings.json), [desacuerdos](../../outputs/iromem/evaluation/documentary-v3-frozen-extraction-2026-10-21/disagreements.json) y [protocolo](documentary-v3-frozen-extraction-protocol-2026-10-21.md).

La corrección posterior de asociaciones y guía semántica es prospectiva; no altera estos datos ni mejora retroactivamente sus porcentajes. Sigue abierta la validación de extracción de esa revisión, conservación global de evidencia útil, comparación con baseline, recursos y estabilidad. V6 permanece sin abrir y no se adopta política nueva.
