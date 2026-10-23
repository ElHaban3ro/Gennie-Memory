# Ablación de la contribución del adaptador LoRA

Fecha: 2026-10-22. Paso 3 del objetivo activo. [Protocolo congelado](adapter-contribution-ablation-v1-protocol-2026-10-22.md). Diagnóstico sobre 432 pares TRAIN ya observados; no validación nueva ni cambio de producto.

## Resultado observado

| Conjunto | Padre: aciertos / útiles preservados / ajenos | Residual desactivado |
| --- | --- | --- |
| background | 155/176 · 70/72 · 19 | 96/176 · 72/72 · 80 |
| pool | 219/256 · 101/128 · 10 | 176/256 · 125/128 · 77 |
| Total | 374/432 · 171/200 · 29 | 272/432 · 197/200 · 157 |

Desactivar el residual admite 26 útiles adicionales: 2 de apoyo (A), 11 de refutación (B) y 13 parciales (C). También admite 128 ajenos adicionales. Todas las selecciones del padre están incluidas en las del estado desactivado a este corte fijo. Ambos omiten los mismos tres útiles B; comparten 29 selecciones ajenas. Una unión simple hereda los errores del estado desactivado. No se implementó ni se midió un selector combinado.

El testigo descriptivo de complementariedad C es positivo (13 casos exclusivos), pero no demuestra una regla que pueda aprovecharlo sin daños. La puerta original 24/24 + 64/64 y la preservación de seis conjuntos **no se reevaluaron**. Tampoco se abrieron las reservas originales ni V6. No hay candidata promovida.

## Límite de combinar los dos márgenes

Comprobación posterior al contraste, sin nuevas inferencias, ajuste ni selección de umbral: para cada uno de los 13 C exclusivos existe al menos un ajeno rechazado por el padre cuyo margen es mayor o igual en ambos estados. Los 13/13 tienen testigos explícitos en `dominance-diagnostic.json`, contrastados independientemente contra los logits guardados.

Si una puntuación global F(margen_padre, margen_off) es no decreciente en cada coordenada, ese ajeno obtiene una puntuación mayor o igual que el útil. Cualquier corte que rescate el útil admite también al menos un ajeno nuevo. Por tanto, esta familia de reglas no puede rescatar ninguno de esos 13 casos con cero ajenos nuevos en este TRAIN. No descarta reglas no monótonas, información semántica adicional, otros modelos ni resultados en otros datos. No es una prueba de imposibilidad general.

Como dato secundario, 148/200 puntos útiles están dominados por algún ajeno del conjunto completo. No se presenta una precisión de ensemble u oráculo ni se convierte este diagnóstico adaptativo en generalización.

## Control y verificación

Mismo Qwen3-Reranker-0.6B y padre V5matched congelado; mismas preguntas, documentos, instrucción original, tokens y corte 0,5. Una sola instancia Metal. El estado off conserva las 16 matrices A del padre y pone a cero las 16 B, con iguales wrappers, formas y dtype. Es una ablación del residual en el mismo grafo, no una ejecución de la arquitectura nativa sin wrappers. Se restauraron los 32 parámetros del padre al terminar; no hubo un nuevo ensayo funcional posterior a la restauración.

Antes de inferencia: preflight independiente 1.671/1.671. Sonda acotada: 32 llamadas directas a módulos, 224 comprobaciones finitas y salidas exactamente iguales a la rama base. Estas llamadas no son forwards del transformer ni una prueba universal sobre entradas futuras. Durante el modelo se registraron 96.768 comprobaciones de finitud de entradas, productos, residual y salida con la misma instrumentación en ambos estados. Los tensores intermedios no se guardaron: la auditoría verifica código y resúmenes registrados, no reconstruye esos tensores.

Auditoría independiente offline: 84358/84.358 comprobaciones; 1.735 eventos y 864 pares reconstruidos desde logits, tokens, referencias y estados. Sin nuevas inferencias en auditoría o revisión causal. Base intacta y adaptadores estables dentro de cada estado; se cambiaron intencionalmente entre estados.

Los 213 hashes de implementación Rust y el binario permanecen idénticos. Se heredan 879 pruebas default / 881 HTTP2 (24 / 75 ignoradas) de esas mismas fuentes; no hubo Cargo nuevo. Producto, DB, secretos, dependencias, ramas y gráficas README sin cambios. Ningún servicio o puerto iniciado.

## Tiempo y coste

Modelo: **104.284 s**, 864 forwards/scores, 0 backward y 0 actualizaciones. Las 32 llamadas directas de sonda se contabilizan aparte. Preparación, auditoría y revisión excluidas del tiempo del modelo. No es latencia de la ruta productiva ni prueba del límite p50/p95 ≤ 1,5×.

API nueva: **0 llamadas, USD 0**. Coste económico local y Codex desconocidos. Acumulado remoto conservado: 6.155 operaciones, 23 campañas emparejadas completas, USD 4,566379428 conocidos y 24 operaciones de coste desconocido. No se declara gratuito el trabajo local o de agentes.

## Continuación

La ablación encuentra señal C complementaria, pero el estado off añade 128 ajenos. Los 13 útiles C exclusivos están dominados por ajenos rechazados por el padre en ambos márgenes: se descarta su rescate sin daño mediante una combinación global monótona. El siguiente experimento debe obtener una señal semántica adicional desde pregunta y recuerdo, separando compatibilidad de identidad y aportación del dato solicitado; no utilizará IDs, roles gold o testigos de este diagnóstico como entradas. Se fijará una única candidata y un filtro TRAIN antes de ejecutar los 88 casos conocidos. No se propone otra mezcla de puntuaciones ni se presume mejora antes de medirla.

El objetivo íntegro sigue activo. Los casos nuevos y su revisión ciega, comparación productiva, recursos, recibos y estabilidad siguen condicionados a una mejora local demostrada. Artefactos: `outputs/iromem/evaluation/adapter-contribution-ablation-v1-2026-10-22/`.
