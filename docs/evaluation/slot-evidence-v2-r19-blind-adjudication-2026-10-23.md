# Adjudicación ciega local del banco slot-evidence-v2 r19

Fecha: 2026-10-23

## Alcance y aislamiento

Un crítico independiente recibió solo 50 pares sintéticos de pregunta y resumen de memoria. El paquete omitió las etiquetas esperadas, resultados de selección, IDs de memoria, implementación, protocolo y resultados anteriores. La instrucción prohibió inspeccionar esos materiales. El crítico etiquetó cada par como útil para la pregunta, evidencia para evaluar una premisa, o irrelevante, con una razón breve.

Esta es una auditoría ciega de la semántica de los pares del banco, no una evaluación ciega del clasificador ni una campaña de recuperación: el crítico no ejecutó el código ni vio resultados del mecanismo. Tampoco es una cohorte prospectiva independiente; el banco r19 ya existía antes de la adjudicación.

## Resultado conciliado

| Juicio ciego por fila | Casos | Etiqueta congelada del banco | Concordancia |
| --- | ---: | ---: | --- |
| Útil para la pregunta, incluida información parcial | 20 | Seleccionar | 20/20 |
| Permite evaluar una premisa (una la apoya y otra la refuta) | 2 | Seleccionar | 2/2 |
| Irrelevante por identidad, sujeto, destinatario, dirección o falta de evidencia | 28 | No seleccionar | 28/28 |
| Total | 50 | — | 50/50 |

La etiqueta congelada coincide con el juicio fila por fila: 22/22 candidatos seleccionados son evidencia útil y 0/28 negativos se consideran útiles. La crítica observó que algunas preguntas generales sobre “qué ciudad” no especifican si se busca residencia o ubicación de viaje; las memorias correspondientes sí expresan explícitamente residencia o estancia. Esto limita la precisión de la pregunta, pero el crítico no marcó esas relaciones como ambiguas al leer pregunta y memoria.

## Corrección de conteo

El crítico reportó inicialmente 19 casos útiles, 2 de premisa y 29 irrelevantes. Ese agregado sumaba 50, pero omitía del listado la fila 18 (`useful-spanish-complement-about-foreign-person`), que su propia tabla había clasificado como útil. La fila por fila da 20 útiles, 2 de premisa y 28 irrelevantes. El conteo del informe se corrigió después de comparar la tabla completa, sin cambiar etiquetas del banco ni la respuesta del crítico.

## Límites y coste

El resultado respalda la coherencia semántica de este banco pequeño y dirigido; no demuestra mejora poblacional, precisión en nuevas conversaciones ni superioridad frente a la configuración vigente. No se llamó a OpenRouter: no hay tokens, coste ni latencia de proveedor que reportar. El coste del uso de subagente Codex no está medido aquí. Se conserva el gate remoto fallido por HTTP 429; TRAIN, la comparación con Decisions y la adopción siguen cerrados.

## Addendum retrospectivo

Una revisión posterior detectó seis exclusiones explícitas del destinatario que la tabla anterior contó como irrelevantes. La evidencia corrige la premisa de destinatario pero no responde qué dijo Ana a Bob. Se conserva el informe y sus conteos como registro histórico; la clasificación corregida está separada y ligada por hash en [la revisión retrospectiva r19](slot-evidence-v2-r19-retrospective-review-2026-10-23.md) y `tests/fixtures/slot-evidence-v2/local-bank-r19-retrospective-overlay.json`. Por tanto, la concordancia 50/50 declarada arriba queda reemplazada para interpretar la semántica completa: ahora son 28 útiles para pregunta o corrección de premisa y 22 irrelevantes. La selección histórica de 22 no incorpora las seis correcciones omitidas y no debe presentarse como recuperación completa.
