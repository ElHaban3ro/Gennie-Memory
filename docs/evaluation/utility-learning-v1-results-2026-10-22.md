# Aprendizaje de utilidad V1: menos selecciones ajenas, pérdida útil inaceptable

La primera adaptación local **falla la puerta de calidad y se rechaza**. En los64pares conocidos reduce los ajenos seleccionados de10a1, pero conserva solo8de17útiles frente a16del control. Son ocho pérdidas útiles nuevas. La exactitud agregada53→54/64oculta ese deterioro y no justifica adopción. La partición cerrada24yV6siguen sin abrir.

| Conjunto | Aciertos antes → después | Útiles conservados antes → después | Ajenos seleccionados antes → después | Pérdidas útiles nuevas |
| --- | ---: | ---: | ---: | ---: |
| Entrenamiento48 | 33 → 44 | 24/24 → 22/24 | 15 → 2 | 2 |
| Desarrollo24 | 13 → 17 | 12/12 → 10/12 | 11 → 5 | 2 |
| Regresión conocida64 | 53 → 54 | 16/17 → 8/17 | 10 → 1 | 8 |

Entrenamiento no es validación. Desarrollo se separó por familias y sus textos se redactaron/revisaron antes de scores, pero estos resultados lo convierten en desarrollo observado para investigaciones posteriores. Los64ya eran conocidos; ninguno se usó para entrenar. No se compensan pérdidas con ganancias: la puerta exigía24/24y64/64sin pérdida ni selección ajena, además de una ganancia.

El desglose de útiles conocidos también empeora: A9/10→4/10, B4/4→2/4 yC3/3→2/3. Se pierden tanto datos/estados solicitados como refutación y contexto parcial. Ejemplos de pérdidas nuevas incluyen límite de autonomía medido, negación explícita de agua de red y corrección del autor de una inauguración. Los IDs, textos, scores y gold originales se conservan íntegros en los artefactos; no se cambiaron etiquetas o umbrales para conseguir un pase.

Se ejecutó exactamente una corrida prospectiva:6épocas,288forward/backwardde un par y72actualizaciones AdamW, acumulación4, clipping1. Adaptadores LoRA de rango8/multiplicador8 en q/v de las últimas8capas;32tensoresFP32,327680parámetros, baseFP16congelada. Semilla20261022y seis órdenes PCG64persistidos antes de scores. Los controles sin entrenar y LoRAinicialB=0conservaron las64decisiones originales dentro de la tolerancia fijada. No hubo selección entre checkpoints, ajuste de tasa/umbral o repetición tras ver fallos.

El entrenamiento duró66,649s; la corrida total con carga, controles y evaluación duró135,695s. La pérdida media por época bajó1,188→0,492→0,449→0,448→0,390→0,336. Se verificaron gradientes positivos, Bfinal distinto de cero, parámetros finitos y hash base idéntico antes/después. Eso acredita una adaptación ejecutada, no una mejora de recuperación. El adaptador ocupa1314143bytes y queda guardado, sin activarse en producto. PicoRSS1,571GB y picoMLX1,691GB son métricas de memoria unificada con posibles solapamientos: no se suman.

La evaluación produjo336scores:136de control,64con adaptador cero y136con pesos finales. Son192scores sobre los mismos64conocidos,96sobre48trainy48sobre24desarrollo; no336ejemplos distintos. Contando entrenamiento son624forwards,288backwardy72actualizaciones. Se registraron comienzos y respuestas de cada operación; la ejecución terminó con código0y sin fallo técnico. No se midió aquí latencia batch20 de los pesos aprendidos ni recursos contra el recorrido productivo: la puerta semántica ya falló y no se amplió.

## Diagnóstico retrospectivo, sin rescatar el intento

Los136scores train/desarrollo/conocidos bajan respecto al control. La ordenación entre útil y ajeno de la misma pregunta cambia23/24→23/24en train,10/12→10/12en desarrollo y48/49→49/49en conocidos. La inversión conocida Vado/Puerto queda corregida en orden, pero el útil cae por debajo del corte0,5y se pierde. **49/49comparaciones ordenadas no equivale a seleccionar toda la evidencia útil**: cada una de estas16preguntas conocidas contiene al menos un útil y una contiene dos. Tomar solo el primero ocultaría pérdidas y tampoco prueba abstención cuando todos los candidatos son ajenos.

En cada split el menor score útil sigue siendo inferior al mayor irrelevante: ningún único umbral monótono sobre estos scores separa todos sus pares. Este cálculo es diagnóstico posterior, no validación prospectiva o permiso para cambiar el corte. Los hechos son compatibles con un descenso general de scores que domina esta adaptación corta; no prueban por sí solos si lo causan datos insuficientes, objetivo de entrenamiento, número de pasos o capacidad del adaptador. No se atribuye causalidad exclusiva sin un contraste separado.

Un cálculo posterior del gradiente respecto al margen de salida yes−no obtiene magnitud media0,002313en positivos train y0,673550en negativos, razón291,25×al inicio. Es la derivada p−yde la pérdida binaria, no el gradiente vectorial de parámetros: no prueba alineamiento de actualizaciones ni una causa única. Da una hipótesis concreta para un contraste posterior: conservar CE y añadir una penalización unilateral del margen absoluto de cada positivo, manteniendo datos, orden, presupuesto y corte. También podría aumentar selecciones ajenas, por lo que se conservan todas las puertas.

La siguiente investigación debe conservar la utilidad absoluta de positivos y discriminar los negativos de la misma pregunta, incluyendo refutación y parciales, y probar ausencia de útiles y varios útiles por pregunta. Antes de otra corrida: fijar hipótesis/contraste y protocolo separado; no entrenar estos64, leer reservas, bajar puertas o rescatar epochs del intentoV1. La calidad completa, comparación contemporánea de recursos, estabilidad yV6siguen pendientes. El objetivo original permanece activo en paso3.

## Auditoría y coste

Auditoría independiente offline:466/466comprobaciones de integridad y reconstrucción, `all_passed=true` con `quality_passed=false`. Recalculó métricas, errores/roles, órdenes, conteos y gradientes registrados; rehasheó los32tensores finales desde safetensors y confirmó su finitud y Bnozero. Los hashes base antes/después son registros de instrumentación; no se persistió una copia completa del modelo base posterior para rehashearla de forma independiente. La secuencia checkpoint→evaluación final se contrasta con flujo estático/artefactos/mtime; no hay evento de checkpoint con timestamp absoluto en el diario y no se presenta como prueba cronológica criptográfica. El próximo protocolo debe registrar ese evento antes de puntuar.

Coste nuevo de API de inferencia: **USD0, cero llamadas remotas**. Sin instalaciones, dependencias o pesos base nuevos. Energía/amortización/Codex desconocidos. Acumulado externo intacto:6155operaciones,23campañas emparejadas completas,USD4,566379428conocidos y24importes desconocidos; no incluye Codex. No cambió Rust/Cargo y no se repitió su suite:213hashes y binario vinculados a879pruebas default/881HTTP2anteriores se verifican antes de archivar. Son pruebas heredadas del mismo código, no ejecución nueva.

Evidencia: `outputs/iromem/evaluation/utility-learning-v1-2026-10-22/` conserva runner/protocolo/inputs permitidos/preparación antes de scores, runtime, diario, órdenes, controles, pérdidas, checkpoint, informe, diagnóstico retrospectivo y auditoría. [Protocolo prospectivo](utility-learning-v1-protocol-2026-10-22.md), [datos revisados](utility-learning-data-v1-protocol-2026-10-22.md), [decisión](../decisions/0032-local-utility-adapter-pilot.md). Configuración vigente sin cambios; ningún servicio iniciado, reserva abierta, commit o push.
