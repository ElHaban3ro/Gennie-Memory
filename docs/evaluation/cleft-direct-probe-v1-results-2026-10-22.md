# Construcción escindida/directa: sonda controlada

Fecha:2026-10-22. Diagnóstico del paso3. Solo cambió «quien lo/la [verbo] fue [persona]» por «lo/la [verbo] [persona]», conservando query,prefijo factual,nombres,títulos,identidad,negación yverbo. Son16nominalesTRAINf17originales y16contrapartes directas,8escenarios. No nuevo entrenamiento ni validación fresca. [Protocolo](cleft-direct-probe-v1-protocol-2026-10-22.md).

| Estado | Aciertos/32 | Útiles/16 | Ajenos/16 | Parejas con cambio útil/8 | Parejas con cambio ajeno/8 |
| --- | ---: | ---: | ---: | ---: | ---: |
|untrained|18|16|14|0|0|
|parent|32|16|0|0|0|
|reweight|22|6|0|2|0|
|diverse|32|16|0|0|0|

Cambios de selección entre expresiones equivalentes identifican sensibilidad en el mismo escenario. Variar margen sin cambiar decisión no cuenta como pérdida. Se conservaron todos los32casos y cuatroestados, sin buscar umbral o excluir celdas.

## Todas las decisiones

| ID | Construcción | Identidad | Gold útil | Sin adaptar:score/sel | Padre:score/sel | Repetición:score/sel | Diverso:score/sel |
| --- | --- | --- | ---: | --- | --- | --- | --- |
|cdp_01|cleft|same|1|0.995056/1|0.976490/1|0.422505/0|0.997949/1|
|cdp_02|direct|same|1|0.995993/1|0.994488/1|0.851953/1|0.999398/1|
|cdp_03|cleft|other|0|0.872347/1|0.012528/0|0.000386/0|0.006290/0|
|cdp_04|direct|other|0|0.894789/1|0.011597/0|0.000395/0|0.004682/0|
|cdp_05|cleft|same|1|0.992304/1|0.947381/1|0.300746/0|0.997469/1|
|cdp_06|direct|same|1|0.993902/1|0.988403/1|0.757794/1|0.999082/1|
|cdp_07|cleft|other|0|0.171061/0|0.000119/0|0.000007/0|0.000024/0|
|cdp_08|direct|other|0|0.196826/0|0.000197/0|0.000015/0|0.000034/0|
|cdp_09|cleft|same|1|0.997965/1|0.997748/1|0.972831/1|0.999884/1|
|cdp_10|direct|same|1|0.998043/1|0.999534/1|0.996055/1|0.999971/1|
|cdp_11|cleft|other|0|0.950411/1|0.008916/0|0.000095/0|0.004434/0|
|cdp_12|direct|other|0|0.924142/1|0.004538/0|0.000088/0|0.001245/0|
|cdp_13|cleft|same|1|0.996205/1|0.969324/1|0.334589/0|0.998716/1|
|cdp_14|direct|same|1|0.996378/1|0.976669/1|0.236516/0|0.998611/1|
|cdp_15|cleft|other|0|0.916110/1|0.002360/0|0.000027/0|0.000792/0|
|cdp_16|direct|other|0|0.888759/1|0.002132/0|0.000041/0|0.000499/0|
|cdp_17|cleft|same|1|0.991422/1|0.908618/1|0.129403/0|0.987277/1|
|cdp_18|direct|same|1|0.992879/1|0.967163/1|0.320821/0|0.995318/1|
|cdp_19|cleft|other|0|0.974043/1|0.300746/0|0.006641/0|0.142232/0|
|cdp_20|direct|other|0|0.982557/1|0.492188/0|0.014282/0|0.259826/0|
|cdp_21|cleft|same|1|0.992000/1|0.841826/1|0.033337/0|0.980581/1|
|cdp_22|direct|same|1|0.991875/1|0.831143/1|0.039937/0|0.958847/1|
|cdp_23|cleft|other|0|0.944995/1|0.046034/0|0.000602/0|0.040541/0|
|cdp_24|direct|other|0|0.951142/1|0.060087/0|0.001782/0|0.039639/0|
|cdp_25|cleft|same|1|0.987375/1|0.777300/1|0.075858/0|0.959459/1|
|cdp_26|direct|same|1|0.985936/1|0.808067/1|0.086942/0|0.945399/1|
|cdp_27|cleft|other|0|0.746139/1|0.001226/0|0.000041/0|0.000377/0|
|cdp_28|direct|other|0|0.754915/1|0.001741/0|0.000089/0|0.000477/0|
|cdp_29|cleft|same|1|0.996901/1|0.995354/1|0.953622/1|0.999787/1|
|cdp_30|direct|same|1|0.997221/1|0.994820/1|0.934873/1|0.999657/1|
|cdp_31|cleft|other|0|0.950411/1|0.017849/0|0.000121/0|0.008062/0|
|cdp_32|direct|other|0|0.927363/1|0.013325/0|0.000280/0|0.004503/0|

Raw scores/márgenes/logits completos en rows.json; selección>=0,5,con números de tabla redondeados. Las64comparaciones pareadas y cambios entre estados se guardan en reporte/auditoría.

## Procedencia y límites

Los16originales se entrenaron en diverse de V6; ninguna contraparte directa se usó en ese entrenamiento. Son contrapartes de los mismos hechos/escenarios,noproblemas independientes. Reweight no entrenó los32f17nuevos. Un pase en esta sonda no prueba transferencia al banco88o a usuarios; una diferencia en ella no demuestra causa universal del entrenamiento ni capacidad global. Las32etiquetas y16equivalencias tienen revisión nativa nueva con etiquetas ypredicciones ocultas, sin discrepancias; no independencia humana/familiar. La transformación acotada solo genera fixtures y no se añade al producto.

La puerta local última sigue fallida: padreconocidos59/64,13/17útiles;V6diverse56/64,10/17útiles,1ajeno. No se reevalúa aquí; no atribuirle ganancias por resultados de la sonda. También siguen pendientes las pérdidas A/C, sin sustituir el objetivo por la familia B.

## Ejecución e integridad

Un solo CLI,26.297s,128forward,0backward,0updates. Orden sin adaptar→padre→reweight→diverse, misma instancia/tokenizador/instrucción. BaseFP16/LoRAFP32, tres checkpoints exactos, modeloeval y caché nueva; fingerprints dentro de estado iguales. Los16originales de cada checkpoint adaptado coinciden con sus decisiones anteriores y scores≤0,02de diferencia. Preflight independiente previo y auditor offline all_passed=True,counts={'checks': 6505, 'passed': 6505, 'cells': 32, 'arms': 4, 'forwards': 128, 'backwards': 0, 'optimizer_updates': 0}. El código terminal real y ausencia de proceso están registrados. Integridad no prueba calidad ni reconstruye los bytes finales de base no persistidos.

API nueva0llamadas/USD0; coste económico local/Codex no cuantificado. Un contexto nuevo para crítica ciega, técnico existente reutilizado. Remoto acumulado6155operaciones/23campañas completas,USD4,566379428conocidos+24operaciones sin coste conocido,sin cambio. No servicios/puertos/descargas/dependencias/DB/secretos/ramas/commits/cambiosdeproducto. FuenteRust213hashes+binarioverificados:heredadas879default/881HTTP2,24/75ignoradas,noCargo nuevo. Tiempos de esta sonda no miden ruta completa/recursos≤1,5×.

Objetivo de cinco pasos activo en paso3,reservas24ycohorte finalV6cerradas,sin adopción.

## Qué cambia este resultado

El padre y el estado diverso aciertan32/32, sin cambio de selección entre las16parejas. La repetición pierde10/16útiles y solo cambia de selección en dos parejas útiles: en ambas rechaza la escindida y conserva la directa. La hipótesis de rechazo general de la forma directa queda contradicha aquí; no se justifica convertirla en una regla del producto ni entrenar otra variación de esa plantilla para aparentar resolver los88casos.

Revisión retrospectiva del V6 anterior: antes de entrenar, el padre ya conservaba16/16útiles de la ampliaciónf17 y acertaba28/32en total. Sus cuatro errores eran ajenos pasivos (q05/q06/q07/q08_m03). Diversidad llevó ese fit a31/32corrigiendo tres ajenos, pero no aportó ejemplos positivos inicialmente fallidos y redujo conocidos13→10/17útiles. Este contraste no establece una causa única; sí muestra que su buen ajuste no prueba aprendizaje de las pérdidas útiles que motivan el objetivo.

Un primer checker técnico falló por un importcollections omitido; fuente/fallo quedaron conservados, se corrigió solo el checker antes de inferencia y el preflight final619/619 fue previo al journal. No se repitió modelo. Auditoría offline6505/6505;128scores y265eventos,64contrastes y192cambios de estado reconstruidos. Fingerprints registrados y archivos de adaptadores verificables no equivalen a persistir/reconstruir toda la base final.

## Decisión siguiente, conservando el objetivo completo

La revisión causal independiente considera razonable pasar de añadir ejemplos sin medir su dificultad a un único contraste de selección activa de TRAIN difícil frente a selección uniforme. Primero debe crearse un pool nuevo con apoyo A,refutación B,contexto parcial C e irrelevancia, con etiquetas revisadas a ciegas y congeladas ANTESde puntuarlo con el padre. La estrategia de minería,cuotas por rol/clase,reglas de grupo/desempate,control y presupuesto se fijan antes de scores. Después se selecciona únicamente material TRAIN; no se convierte ese pool en validación ni se modifican sus etiquetas según el modelo. Deben informarse los fallos útiles realmente disponibles, sin llamar «difícil» a ejemplos ya correctos por conveniencia.

La comparación posterior mantendrá exposición,roles/targets y presupuesto iguales; no presupone éxito. Los88observados,las sondas yreservas quedan fuera de TRAIN. Repetir puerta original después de la intervención, preservar todas las pérdidas A/C y no reducir el objetivo a la familia de atribución. Todavía no se generó ese pool ni se ejecutó otra optimización en esta sonda.
