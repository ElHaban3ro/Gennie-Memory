# Rescate contextual por tokens V1: resultado

La variante se rechaza. Cambia embeddings léxicos estáticos por estados de token del padre V5 congelado y conserva la receta 161: arquitectura, folds, calendario, pesos balanceados, optimizador, 12 épocas y umbral 0,5.

## Paridad y captura

La secuencia canónica del padre se reprodujo en los 232 casos residuales: cero flips de decisión y delta máximo cero en logits y puntuaciones. El adaptador cargado corresponde al hash `7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74`; la base, al hash `e2c342cf3096f931b2f5d2023496b5cb15b68cc6d9c4e2c454bc120344e62e5f`.

La captura generó estados finitos para 5.724 tokens de pregunta y 7.261 de documento. Captura 24,26 s; ajuste 1,14 s; tres fits y 708 updates; cero entrenamiento del Transformer y cero llamadas API.

## Calidad

| Evaluación | Aciertos | Útiles retenidos | Extras | Rescates C | `none` nuevos |
| --- | ---: | ---: | ---: | ---: | ---: |
| Padre V5 | 374/432 | 171/200 | 29 | — | — |
| Control balanceado 161 | 339/432 | 171/200 | 64 | 0 | 35 |
| Contextual V1 | 356/432 | 171/200 | 47 | 0 | 18 |

La variante queda 18 aciertos por debajo del padre y añade 18 `none` respecto a este. Conserva los mismos 171 útiles y no rescata C. Falla el criterio conjunto de al menos un rescate C y cero `none` nuevos; no se promueve.

## Límites y artefactos

Es desarrollo con datos conocidos; el padre V5 ya estuvo expuesto a 176 filas. No es transferencia independiente ni validación ciega/fresca. Tampoco valida la integración con FTS/Decisions ni el límite de recursos de la ruta de producto. Los estados incluyen instrucciones y contexto de secuencia; los spans incorporan tokens completos que cruzan límites de puntuación/espacios.

Los logits OOF y composición están en `outputs/iromem/evaluation/contextual-token-rescue-v1-2026-10-23/result-contextual.json` y `oof-contextual.json`. La captura está en `capture-manifest.json`, `contextual-states.safetensors` y `contextual-state-index.json`; la paridad y los spans, en `parity-preflight.json` y `span-offset-audit.json`. Coste API USD0; coste Codex desconocido.

## Revisión retrospectiva de separabilidad

En el fold de 160 filas, el AUC par-a-par útil frente a `none` sube de 0,379 con 161 a 0,669 con la representación contextual. Aun así, con el umbral fijo 0,5 selecciona 0 útiles y 18 `none`; los 18 quedan entre 0,503 y 0,518. La evidencia de ranking no supera la puerta operativa congelada. En el conjunto OOF mezclado, el AUC es solo descriptivo por combinar folds/modelos.

Una heurística exploratoria de intersección de nombres capitalizados halla entidad/nombre compartido en varios falsos positivos: la identidad por sí sola no distingue el atributo pedido. Esto motiva investigar alineación entre slot solicitado y evidencia, pero no demuestra que mejore recall. No se cambió el umbral ni se alteraron juicios. [Auditoría de separabilidad](../outputs/iromem/evaluation/contextual-token-rescue-v1-2026-10-23/feature-separability-audit.json).
