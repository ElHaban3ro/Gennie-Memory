# Normalización fija: hipótesis sin apoyo en este contraste

Paso3activo. La comparación prospectiva cambia solo media/escala de los cinco ajustes ampliados, reutilizando las del control. **No reduce daños y no se adopta.** [Protocolo](readout-normalization-mediation-v1-protocol-2026-10-22.md).

| Medición | Padre histórico | Control original | Ampliado con estadísticas propias | Ampliado con estadísticas del control |
| --- | ---: | ---: | ---: | ---: |
| Decisiones correctas (432) | 374 | 343 | 328 | 326 |
| Útiles conservados (200) | 171 | 157 | 165 | 165 |
| Ajenos seleccionados (232) | 29 | 46 | 69 | 71 |

Congelar estadísticas conserva exactamente las mismas decisiones para los200útiles y añade dos ajenos:hpa_q01_m03 y hpa2_q07_m03. Ninguna otra decisión cambia frente al ampliado152; los432resultados y probabilidades están preservados. Frente al padre:16pérdidas útiles nuevas,47ajenos nuevos y7ganancias C. La ganancia parcial no compensa daños.

## Qué se aisló

Los560features ya capturados, datos, etiquetas, orden, mapa de componentes, exclusiones, receta y corte0,5 son los mismos. Cada media/escala del control se reconstruye exactamente desde TRAIN original fuera del componente; se ajusta la cabeza con todo TRAIN ampliado permitido. Cinco folds cubren cada original una vez. No se usa información del componente test para normalización o ajuste.

Cambiar la escala cambia la geometría de L2 en coordenadas del hidden y centrar con sesgo regularizado puede cambiar la penalización del intercepto. Por eso el contraste es informativo, pero el resultado no respalda que recalcular estadísticas cause el deterioro anterior: mantenerlas no lo evita. No prueba que normalización sea irrelevante en todo dataset ni identifica por sí mismo la causa lingüística. No se cambian λ, umbral, datos o checkpoints para rescatarlo.

## Evidencia y coste

Preflight independiente332/332antesdeajuste;246archivos congelados y11controles CPU sintéticos. Auditorposterior independiente reconstruye normalización, objetivo/gradiente incluyendo sesgo, predicciones exactas, métricas y diario sin refit. Convergencia de las cinco cabezas verificada; no equivale a calidad aprobada.

Cinco ajustes CPU y40pasos; duración instrumental del runner0.147910s y picoRSS95223808bytes. Cero forwards/backwards/actualizaciones del transformer y cero llamadasAPI/USD0. CosteCodex,energía/amortización desconocidos. No es latencia productiva ni pase de recursos. Proceso terminal,exit0observado,sinpuerto/servicio.

Las218fuentes Rust/producto son idénticas al152. Las928pruebas default y930HTTP2se heredan del150,no se reejecutan para esta investigaciónCPU. Formato actual comprobado por separado. No cambió producto,dependencias,DB,credenciales o baseline. Acumulado externo permanece6319operaciones,USD4,582739128 conocidos y25operaciones con importe desconocido.

## Continuación y puertas

Se descarta esta hipótesis como explicación útil del deterioro observado y no se ensayan variantes de normalización. La siguiente intervención debe comprobar si la representación/lectura responde al dato solicitado, en vez de añadir otra tanda de ejemplos o mover el umbral. Se prepara un diagnóstico de preguntas contrafactuales sobre diez textos ya observados: conservar el documento y cambiar solamente qué se pregunta, con revisión de etiquetas separada antes de inferencias. No es un nuevo TRAIN ni validación fresca del paso4. Todavía no se ejecutó ese diagnóstico.

Permanecen todas las puertas originales:24/24desarrollo,64/64conocidos,conservación de seis conjuntos,recursos coste/p50/p95≤1,5×base,estabilidad,reciboscompletos yV6ciega. Las reservas siguen cerradas; OOFsobreTRAINobservado no prueba generalización nueva. Objetivo activo,2/5etapas cerradas;paso3pendiente de calidad.
