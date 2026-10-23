# Comparación prospectiva V3 de alcance de sujeto — resultados

Fecha: 2026-10-23. Cohorte sintética, desarrollo; no es validación poblacional.

## Resultado

El gate V3 falló en su primer smoke, por lo que la variante se detuvo tras una llamada. La búsqueda local recuperó el único candidato esperado; baseline lo seleccionó correctamente (**1/1**). V3 terminó degradado y sin selección con `slot_evidence_v2_kind_not_literal` (**0/1**). El diario no conserva el texto generado, así que no se puede afirmar qué cita emitió ni si fue capitalización, paráfrasis o ambigüedad. La raíz `v2` del código de error es histórica del decoder compartido; la política pedida fue V3.

El baseline completó los nueve casos: **8/9** aciertos exactos, con **7/7** positivos y **1/2** abstenciones correctas; seleccionó erróneamente la memoria de Carlos para responder por la ciudad de Bob en `sentence-05`. No se degradó en ningún caso. La búsqueda encontró el candidato esperado en **7/7** positivos y devolvió un candidato en ambos negativos; eso no equivale por sí mismo a seleccionar una respuesta. El resumen de campaña cuenta candidatos prohibidos expuestos además de selecciones, por lo que su `forbidden_leak_count=2` no debe leerse como dos respuestas falsas: la auditoría de `seed.memories` encuentra **1** selección falsa.

| Brazo | n | Aciertos exactos | Positivos | Negativos abstendidos | Degradados | Coste informado | Latencia clasificador p50 / p95 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Baseline | 9 | 8/9 | 7/7 | 1/2 | 0 | USD 0,00006755 | 771 / 1.175 ms |
| V3 | 1 de 9 (gate detenido) | 0/1 | 0/1 | — | 1 | USD 0,00007875 | 3.108 / 3.108 ms |

Tokens informados: baseline 1.243 de entrada y 54 de salida en total; V3 1.049/263. Coste conocido combinado: **USD 0,00014630** en diez llamadas. Mediana de servicio total: baseline 775 ms; V3 3.111 ms (n=1). Son tiempos de este servicio sintético, no latencia de integración completa.

## Integridad y límites

La fixture sin etiquetas se revisó ciegamente antes de esta campaña: dos lectores coincidieron en 10/10; nueve casos se puntúan y el octavo queda fuera porque contiene ubicación pasada que podría ser evidencia parcial. Las nueve búsquedas locales terminaron completas y recuperaron exactamente un candidato por ámbito. El request de runtime se construye de `request.search` y proveedor; la etiqueta, el segmento, el razonamiento y la adjudicación no se envían. Se usó `blind_synthetic_adjudication`; casos, corpus, búsquedas, base y binarios tienen hashes en `manifest-preflight.json` y en los manifiestos de diario.

No se hicieron reintentos. Al fallar el primer smoke, no se gastaron las ocho llamadas restantes de V3. El baseline completo muestra que hay margen de mejora en abstención ante sujetos ajenos; el resultado V3 no puede compararse como una cohorte completa y falla la puerta antes de cualquier afirmación de mejora. El guard local de la tanda anterior no puede reparar una cita de tipo que falla antes de construir una traza.

Decisión: **V3 no se adopta y baseline no cambia**. Siguiente iteración local: hacer más explícita la extracción literal de `kind_quote` para el patrón de pregunta `Where does <kind> <name> <ID> work now?`; revisar el cambio sin alterar parser ni etiquetas y luego usar casos nuevos, no reintentar este holdout como validación.
