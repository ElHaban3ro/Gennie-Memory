# Ampliación final de consultas opuestas: V2

La candidata queda descartada para adopción. Mejora el control contemporáneo, pero incumple preservación de evidencia útil y rechazo de ajenos. El objetivo completo sigue activo en el paso 3, con dos de cinco etapas cerradas.

| Variante | Aciertos / 432 | Útiles conservados / 200 | Ajenos seleccionados / 232 |
| --- | --- | --- | --- |
| Referencia histórica fija | 374 (86,57 %) | 171 | 29 |
| Control contemporáneo: repetir originales | 324 (75,00 %) | 172 | 80 |
| Candidata: consultas opuestas | 336 (77,78 %) | 172 | 68 |

Frente al control corrige 20 errores e introduce ocho: ganancia neta 12, pero cinco pérdidas útiles y tres ajenos nuevos. La igualdad de 172 útiles no es preservación de los mismos IDs. Frente a la referencia histórica registra diez pérdidas útiles y 42 ajenos nuevos, pese a seis ganancias C. No satisface las puertas originales.

Nuevas pérdidas útiles frente al control: `x_f15_q02_m03`, `hpc_q06_m02`, `hpc_q07_m02`, `hpc_q08_m02`, `hpa2_q06_m02`. Nuevos ajenos: `x_f16_q03_m03`, `hpb_q11_m03`, `hpb_q13_m03`. Todos los resultados e IDs completos están en `report.json` y `rows.json` del directorio del experimento.

| Evidencia útil | Referencia histórica | Control contemporáneo | Candidata |
| --- | --- | --- | --- |
| A: apoyo / 100 | 98 | 97 | 95 |
| B: refutación / 45 | 31 | 26 | 30 |
| C: parcial / 55 | 42 | 49 | 47 |

La mejora agregada no elimina los daños particulares. Estos son objetivos TRAIN previamente observados evaluados fuera de su componente; no una validación fresca ni una estimación poblacional.

## Qué se contrastó

Se amplió de 32 a las 128 fuentes del banco TRAIN, sin cambiar el lector, umbral, regularización o modelo. Las 256 filas contienen 128 originales y 128 consultas de utilidad opuesta; se conservaron las 64 filas V1 exactamente. Hubo 96 revisiones fuente nuevas, todas coincidentes en utilidad y rol; otras 160 anotaciones son reutilizaciones verificadas (64 V1 y 96 originales). Los agentes pertenecen a la misma familia y tienen historia previa: no es revisión humana ni independencia de familia.

Ambos brazos tienen 816 ocurrencias, iguales documentos, posiciones, exposición y marginales binarios por familia y fold. La frecuencia de etiquetas por documento es 2:1 original/opuesta. Los roles positivos y el estilo difieren: no atribuir la diferencia exclusivamente a condicionalidad ni afirmar un atajo documental no demostrado.

El cierre fuente quedó en 13 componentes, cuatro con objetivos originales: 320/72/32/8. Conserva los enlaces V1 y corrige una homología concreta entre negación de ocurrencia en intervalo solicitado, ya visible antes de V2. Por ello los resultados V1 y 152 son históricos descriptivos, no controles contemporáneos de V2. No se reescribieron esos resultados. La normalización es común a ambos brazos y usa solamente background560 fuera del componente: 200/488/520/544 filas.

688 contenidos únicos; 592 representaciones reutilizadas por ID y hashes (560 originales y 32 V1), 96 nuevas. Modelo Qwen3-Reranker 0.6B FP16 base congelado, sin LoRA; representación final 1024 y lector binario fijo151. Ocho fits CPU, sin barridos ni rescate por control.

## Integridad y coste

El primer preflight rechazó una referencia de tokens V1 sin congelar. Un segundo plan pasó un preflight incompleto, pero el runner se detuvo antes del marker, carga del modelo o captura porque faltaban referencias históricas V1 de informe y resultado. Se conservaron planes, verificaciones y fallo; se añadieron las referencias sin cambiar datos ni receta. El plan final `1accd51661051e5ed302b4d13aa9a3cf6dd874d752e3a8baf146bc57a2cd91fd` pasó 393 verificaciones antes de inferir. El primer intento fallido no efectuó inferencias ni fits y no se cuenta como corrida de modelo.

La ejecución real terminó con exit 0, sesión 37696 y PID ausente, según observación terminal. 96 forwards nuevos, ocho fits CPU y 64 pasos Newton; cero backwards o actualizaciones del transformer. Tiempo total 16,201695 s: captura 15,696499 s y lectores 0,313405 s. RSS pico 3.633.692.672 bytes. Estos recursos son instrumentales del experimento: no son latencia end-to-end ni pase de la puerta original de recursos.

Cero llamadas API y USD 0 externos adicionales. Coste económico local y Codex desconocido. Los 218 archivos fuente Rust permanecen sin cambios; suites 928 default/930 HTTP2 heredadas de150, sin repetir. `cargo fmt --all --check` se ejecuta en este cierre. La auditoría CPU posterior pasó: verificó los 279 archivos congelados y recompuso normalización, objetivos, gradientes y predicciones desde pesos persistidos, con diferencia máxima de probabilidades 0, sin refit ni encoder. No reconstruye cada búsqueda Armijo intermedia; sus límites constan en el artefacto de auditoría.

## Decisión y siguiente trabajo

Se cierra esta variante como rechazada, tal como se fijó antes del resultado. No seguir aumentando el mismo banco ni barriendo otro corte o lector para intentar rescatarla. No se adopta el control como sustituto. La configuración vigente y las puertas originales se mantienen; las reservas y V6 permanecen cerradas.

El siguiente trabajo del paso 3 debe fundamentar un mecanismo distinto en la relación entre pregunta y evidencia. La revisión posterior de las ocho fuentes ya consta en `source-error-census.json`: tres ajenos nuevos pertenecen a otro referente explícito; tres pérdidas útiles aportan una faceta de dos, una aporta el valor pedido y otra su indisponibilidad explícita. No prueba por sí sola un defecto causal del encoder. Antes de otra corrida hace falta una intervención distinta sobre la interacción pregunta-documento y un criterio prospectivo de descarte, no etiquetas revisadas ni un selector manual. Evitar también repetir las fichas y cambios de esquema ya fallidos de ADR0033. Si no surge una intervención concreta, no lanzar más ajustes del mismo experimento. Los pasos 4 y 5 siguen pendientes del pase de calidad local.
