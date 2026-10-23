# Consultas opuestas con documento fijo: contraste controlado V1

La candidata no pasa calidad y no se adopta. La comparación emparejada mejora de323/432 a331/432, pero introduce dos pérdidas útiles y dos ajenos nuevos frente a su propio control. El padre histórico conserva374/432; frente a este, la candidata pierde12recuerdos útiles y añade43ajenos nuevos, pese a seis ganancias C. No se ejecutaron reservas ni validación nueva.

| Variante | Correctos /432 | Útiles conservados /200 | Ajenos seleccionados /232 |
| --- | --- | --- | --- |
| Padre histórico fijo |374|171|29|
| Control: repeticiones del original |323|168|77|
| Candidata: original y consulta opuesta |331|168|69|

Son doce errores corregidos y cuatro nuevos respecto al control: ganancia neta ocho. Igual número de útiles conservados no implica conservar los mismos IDs. Las nuevas pérdidas son `hpc_q07_m02` y `hpa2_q02_m02`; los nuevos ajenos son `x_f15_q04_m03` y `hpb_q07_m03`. Los resultados históricos152(343/432control y328/432ampliado) usan otra partición y son descriptivos, no controles emparejados de este ensayo.

## Diseño y límites

Se seleccionaron antes de inferir32fuentes, una útil y una irrelevante por cada16familias TRAIN. Las64filas(original más consulta opuesta) recibieron dictamen fuente64/64coincidente en utilidad y rol. Es revisión separada entre agentes de la misma familia con historia previa, no humana ni validación fresca. Las etiquetas originales y sus textos permanecen intactos.

Ambos brazos tienen624ocurrencias, mismos documentos, exposición por documento y marginales binarios por familia/componente. El control duplica dos veces cada original; la candidata añade original y contrapregunta, además del original ya existente. La exposición por documento queda2:1en etiquetas originales/opuestas. Los roles positivos y el estilo no están equilibrados entre brazos: no atribuir causalmente toda diferencia solo a condicionalidad. Tampoco concluir que el modelo usa un atajo documental.

El cierre semántico previo a inferencia contiene14componentes, cuatro con432objetivos(320/72/32/8). Una homología concreta nueva une dos grupos históricos. Ambos brazos usan ese mismo cierre; las estadísticas de normalización se calculan exclusivamente sobre background560fuera de cada componente(200/488/520/552filas). Modelo Qwen3-Reranker0.6Bpreentrenado FP16 congelado, sin LoRA; lector binario y receta fijos, sin barrido de umbral, regularización, ponderaciones ni rescate por control.

El diagnóstico posterior con lectores persistidos demuestra clasificación correcta de todas las contrapreguntas presentes en el TRAIN de cada fold. No implica transferencia a familias excluidas, donde falla calidad. Este diagnóstico no hizo fits ni forwards adicionales.

## Verificación y recursos

Preflight961/961previo a inferencia. Auditoría CPU separada aprobada:264archivos congelados, ocho objetivos, gradientes, normalización y predicciones recomputados sin refit; diferencia de predicciones cero y gradiente máximo8,651e-9. Los218archivos fuente Rust conservan sus hashes. `cargo fmt --all --check` se ejecuta en este cierre; las suites928default/930HTTP2son heredadas de150, no repetidas.

Una ejecución local terminal: sesión78059, PID5232, exit0 y proceso ausente.32forwards nuevos,560features reutilizados, ocho fits CPU y64pasos Newton; cero backwards o actualizaciones del transformer. Duración total8,937681s(captura8,730036s, lectores0,127241s); RSS pico3.309.944.832bytes. Son recursos instrumentales del ensayo, no latencia end-to-end ni prueba de la puerta original de recursos. Cero llamadas API y USD0externos adicionales; coste económico local/Codex sin cuantificar.

## Próxima decisión

Completar una ampliación controlada a las128fuentes TRAIN, conservando receta y dos ocurrencias añadidas por fuente.688contenidos únicos y816ocurrencias por brazo;592features cacheados y96consultas nuevas. Reutilizar64dictámenes V1y96dictámenes de originales, revisar solo96contrapreguntas nuevas; declarar cada procedencia. Resolver cierre semántico antes de features, sin imponer cuatro folds. Si quedan menos de dos componentes objetivo o un TRAINsin ambas clases, detener antes de inferir.

Esta ampliación cierra esta variante: si no supera los criterios originales, descartarla como candidata. No seguir barriendo variantes del mismo lector para perseguir porcentajes. Conservar configuración vigente y usar el fallo documentado para plantear una hipótesis distinta. Solo un pase permite abrir la validación fresca del paso4. Objetivo activo:2/5etapas cerradas; no adopción ni cierre del objetivo.
