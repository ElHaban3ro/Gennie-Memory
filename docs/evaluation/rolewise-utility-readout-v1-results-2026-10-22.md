# Tres cabezas de utilidad: capacidad adicional sin mejora demostrada

Paso 3 activo. Un único ensayo prospectivo con tres cabezas A/B/C y decisión OR sobre features congelados. **Falla calidad y no se adopta.** No se cambian etiquetas, corte, agregación o regularización tras el resultado. [Protocolo](rolewise-utility-readout-v1-protocol-2026-10-22.md).

| Medición | Padre histórico | Binario original | Binario ampliado | Tres cabezas OR |
| --- | ---: | ---: | ---: | ---: |
| Aciertos /432 | 374 | 343 | 328 | 322 |
| Útiles conservados /200 | 171 | 157 | 165 | 150 |
| Ajenos seleccionados /232 | 29 | 46 | 69 | 60 |

Respecto al binario ampliado, rechaza nueve ajenos netos más, pero pierde quince útiles netos: el total baja328→322. Frente al padre aparecen33pérdidas útiles nuevas y41ajenos nuevos, pese a11ganancias C. Cada ID/probabilidad/rol/lector queda en rows.json; no se compensan daños con ganancias.

| Rol | Binario ampliado152 | Tres cabezas |
| --- | ---: | ---: |
| A, útiles /100 | 94 | 86 |
| B, útiles /45 | 24 | 18 |
| C, útiles /55 | 47 | 46 |
| none, ajenos seleccionados /232 | 69 | 60 |

## Qué se probó

Mismos560pares/features/roles y grupos del152; cinco folds excluyen el componente completo, incluidos los nuevos homólogos. Una normalización por TRAIN compartida por tres objetivos one-vs-rest, exacta a la del ampliado152. A/B/C solo son etiquetas objetivo de TRAIN; el predictor runtime recibe features y parámetros, nunca rol gold. Las432filas originales reciben tres probabilidades y se seleccionan si cualquiera alcanza0,5. No se suman scores como probabilidades de utilidad calibradas.

Los15objetivos tienen ambas clases; mínimos de soporte A16,B33,C26. Sigue siendo una intervención conjunta en capacidad, targets, desequilibrio y penalización agregada, no una prueba causal de que mezclar roles originara los fallos anteriores. Las pérdidas predominan enA/B. No se ensaya otra agregación,λ o balanceo para rescatar la candidata.

## Evidencia y coste

Seis controles sintéticos, con12fits pequeños separados de los ajustes reales, verifican aislamiento de heldout, normalización compartida, ausencia de rol runtime, OR frente a suma y rechazo por rol sin soporte antes de ajustar. Preflight independiente350/350 previo a datos reales y248archivos congelados intactos. Auditor independiente reconstruye los15targets, objetivos/gradientes,432×3probabilidades,normalización,exclusiones y métricas sin refit o encoder. La primera versión del auditor se preservó antes de ejecutarla y pasó.

Quince fits reales,100pasos CPU; duración instrumental del runner 0.282720s y picoRSS 94142464bytes. Cero encoder forwards/backwards/updates y cero API/USD0 nuevos. Los560features reutilizados no son nuevas inferencias. CosteCodex,energía y amortización desconocidos. No es latencia productiva ni pase de recursos. Proceso CLI terminal,exit0observado,sinpuerto/servicio.

Las218fuentes Rust/producto siguen idénticas. Las928pruebas default y930HTTP2 se heredan del150; no se repiten para este ensayo CPU. Formato actual verificado por separado. No cambian DB,secretos,dependencias o producto. Acumulado externo6319operaciones,USD4,582739128conocidos y25sinimporte, sin nuevas operaciones externas.

## Siguiente hipótesis de datos, todavía sin ejecutar

La inspección de cobertura encuentra560filas y546documentos:14documentos aparecen con dos consultas de utilidad opuesta y532con una sola. Los128documentos de la ampliación nueva aparecen una vez cada uno. Por tanto, sería falso afirmar ausencia total de contrafactuales o que un lookup de identidad documental explica todas las etiquetas. La cobertura emparejada sí es escasa; tampoco demuestra que el lector aprendido haya usado un atajo documental.

Se prepara un contraste de supervisión condicionada a la consulta sobre32fuentes exclusivamente del TRAIN128:una útil y una ajena por cada una de16familias, seleccionadas determinísticamente antes de nuevas puntuaciones. Cada texto conservará su pregunta original y una contrapregunta con utilidad opuesta, con revisión separada previa. Los dos brazos añadirán64filas a las560de fondo:control con dos repeticiones del original por fuente;tratamiento con original+contrapregunta. Así se igualan frecuencia de documento, filas y cantidades de clases por familia y componente permitido. Serían624filas por brazo,con32features nuevos, sin cambiar la evaluación sobre432originales. La normalización se fijará al fondo permitido común por fold. No se ha lanzado esa comparación ni se afirma mejora; sigue pendiente resolver cualquier nuevo parentesco de plantillas antes de features.

Esta hipótesis cambia la forma de supervisión sin negar los contrafactuales existentes. Puede fallar por preguntas demasiado fáciles, atajos de redacción o insuficiente transferencia. Se vuelve al lector binario fijo, sin encadenar variantes de OR. No es validación fresca del paso4.

## Puertas intactas

Filtro preliminar:gananciaC frente al padre y cero pérdidas útiles nuevas/ajenos nuevos. Permanecen24/24desarrollo,64/64conocidos,seis conjuntos,recursos coste/p50/p95≤1,5×base,recibos,estabilidad yV6ciega. No se abren reservas tras este fallo, no se reevalúan los18casos154para seleccionar variante y no se ajusta cabeza final de despliegue. Objetivo activo con2/5etapas cerradas.

Límite adicional del contraste en preparación: aunque se igualan utilidad binaria y familia, los nuevos positivos redactados son A. En las64filas añadidas, tratamiento tiene20A/6B/6C frente a8A/12B/12Cdelcontrol;ambos tienen32none. Cualquier diferencia deberá atribuirse a esa intervención de datos completa, sin afirmar que aisló únicamente el condicionamiento por pregunta. La crítica de pistas de redacción y el cierre de nuevos parentescos permanecen pendientes antes de inferencias.

Diagnóstico posterior sin refit: cada una de las quince cabezas acierta todos sus propios targets TRAIN, y OR conserva toda la utilidad TRAIN de los cinco componentes. La brecha al componente excluido permanece. Esto descarta atribuir aquí el fallo a falta de convergencia o a que esta receta no haya ajustado sus ejemplos; no prueba transferencia, calibración o independencia de familia.
