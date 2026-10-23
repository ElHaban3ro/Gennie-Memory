# Readout congelado: ajuste correcto, transferencia insuficiente

Fecha:2026-10-22. Paso3 activo. [Protocolo](frozen-utility-readout-v1-protocol-2026-10-22.md). Una candidata, sin barrido ni reintentos. La cabeza logística aprende sus ejemplos de entrenamiento pero falla al excluir familias de plantillas. **No supera preservación y no se incorpora al producto.**

## Calidad sobre los432 pares existentes

| Medición | Yes/no base actual | Padre histórico fijo | Cabeza OOF |
| --- | ---: | ---: | ---: |
| Decisiones correctas | 272/432 | 374/432 | 343/432 |
| Útiles conservados | 197/200 | 171/200 | 157/200 |
| Ajenos seleccionados | 157/232 | 29/232 | 46/232 |

La cabeza reduce111 ajenos de la base pero pierde40 útiles respecto a ella. Frente al padre tiene22 pérdidas útiles nuevas y24 ajenos nuevos; recupera4 útiles C omitidos, insuficientes para compensar daños. El agregado no oculta cambios por caso: todos quedan en rows.json. No se ajusta una cabeza final ni se cambia regularización, umbral o partición después del resultado.

| Rol | Útiles conservados o ajenos rechazados por OOF |
| --- | ---: |
| A: apoyo | 92/100 útiles |
| B: corrección | 25/45 útiles |
| C: aporte parcial | 40/55 útiles |
| none | 186/232 rechazados;46 seleccionados |

Es OOF agrupado sobre TRAIN observado, no generalización fresca ni comparación completa del producto. Yes/no base procede del mismo hidden y la misma pasada que las features. El padre tiene adaptación distinta y sus scores son históricos ligados por hash a los mismos textos/tokens: no es control contemporáneo de recursos ni permite atribuir toda diferencia exclusivamente a la cabeza.

## Por qué el ajuste no basta

Se usan1024 componentes finales del encoder base FP16 sin LoRA. El padre ajustado ya había entrenado176 de estos ejemplos; separar folds de su cabeza no habría eliminado la exposición del encoder. El cambio a base se decidió antes de cualquier extracción, preservando el diseño inicial no ejecutado. No se prueba ausencia absoluta de exposición en pretraining.

El mapa final tiene cinco componentes312/72/32/8/8. Cada fila recibe una predicción de una cabeza que excluyó todo su componente; normalización y ajuste usan solo el TRAIN del fold. Se agrupan familias, pools, duplicados y construcciones homologadas por fuente. Compartir una etiqueta o función semántica general no demuestra identidad de plantilla. Se conservaron mapas borradores: la revisión añadió cuatro puentes concretos de f14/f16 y documentó seis similitudes genéricas como propuestas, antes de features. Esto no demuestra independencia semántica amplia.

| Filas del componente excluido | Ajuste en las otras filas | Resultado OOF |
| --- | ---: | ---: |
| 312 | 120/120 | 254/312 |
| 72 | 360/360 | 52/72 |
| 32 | 400/400 | 27/32 |
| 8 | 424/424 | 5/8 |
| 8 | 424/424 | 5/8 |

Los ajustes convergen al objetivo convexo fijo, con gradiente infinito≤1e-7. El100% en entrenamiento es un diagnóstico de ajuste, no aprobación de calidad. La brecha al excluir plantillas muestra transferencia insuficiente para esta representación, receta y banco. No prueba imposibilidad de cualquier lectura lineal o incapacidad global del modelo; tampoco demuestra que aumentar diversidad vaya a resolverla.

## Verificación, tiempo y coste

Se completan432 forwards y cero backwards/actualizaciones del transformer; cinco cabezas CPU y41 pasos Newton aceptados. Los310 tensores cargados coinciden bit a bit con los bytes BF16 persistidos convertidos a FP16 en CPU. Fingerprints antes/después iguales, sin LoRA o parámetros entrenables del encoder. Todas las features finitas432×1024 FP32; tokens completos sin truncamiento. Se verifica proyección de los mismos vectores hacia no/yes en CPU con tolerancias fijadas antes de ejecutar.

Controles CPU:11 comprobaciones sintéticas, sin features reales. Revisión de grupos3.828/3.828; preflight3.724/3.724 antes de inferencia; auditoría posterior7.451/7.451. El auditor reconstituye casts, features, normalizaciones TRAIN, predicciones y estacionariedad final del objetivo. No reejecuta el encoder ni optimiza cabezas. No hay pesos intermedios para probar cada trayectoria Newton/Armijo ni tensors finales del encoder persistidos; sus fingerprints finales son evidencia registrada de ejecución, no una segunda captura externa del modelo. Integridad aprobada no implica calidad aprobada.

Dos agentes prepararon un archivo temporal de auditor con el mismo nombre. Se detectó antes de ejecutar auditorías, se preservaron versiones propias y se ejecutó solo la del autor con SHA a83ca3ab4e2c92db78550ec70526fb0a59563d598fcd3197ea79a0eff758d2e4. La versión del crítico no se cuenta como segunda auditoría ejecutada. El runner, sus archivos congelados y sus resultados no cambiaron. Detalle en auditor-provenance.json.

| Recurso experimental | Observado |
| --- | ---: |
| Duración total del runner | 50,049308 s |
| Tiempo acumulado de los432 forwards | 42.231,611 ms |
| Mediana descriptiva por forward | 94,248 ms |
| Pico RSS del proceso | 3.521.036.288 bytes |
| Pico MLX reportado | 1.596.309.728 bytes |
| Nuevas llamadas API | 0 |
| Nuevo coste API | USD0 |

La mediana es instrumental y no p50/p95 del recorrido productivo. Energía/amortización/Codex desconocidos; USD0 de API no es coste económico total cero. No se usan tokens de Codex como métrica del clasificador local. El proceso PID75959/session67011 fue observado en ejecución y después ausente, con exit0, sin reinicios. Los procesos CLI propios son terminales; no hubo servicio o puerto.

No cambió ninguna de las218 fuentes de producto/evaluador Rust ligadas a150. Se heredan, sin volver a ejecutarlas, las928 pruebas predeterminadas y930 HTTP2 de esa versión; se ejecutó de nuevo cargo fmt --all --check con salida0. Los controles de esta implementación experimental y su auditoría sí son nuevos. Acumulado externo permanece6.319 operaciones yUSD4,582739128 conocidos más25 sin importe. Se corrige un alias agregado obsoleto24→25 para hacerlo coherente con el contador principal; no añade operaciones.

## Continuación

La lectura logística reduce los ajenos de la base157→46, pero pierde útiles197→157 y queda por debajo del padre374→343/432. Las cinco cabezas convergen y aciertan todos sus ejemplos de entrenamiento; la transferencia al componente excluido falla. No se modifica umbral/regularización, no se ajusta cabeza final y no se abren88/reservas. La siguiente prueba mantendrá encoder/receta/umbral fijos y cambiará únicamente cobertura del TRAIN:128 pares nuevos en16familias con construcciones distintas, revisión de fuentes sin etiquetas y conciliación antes de features. No basta cambiar nombres o repetir plantillas. El mapa de parentesco con el banco anterior se resolverá antes de OOF; ningún par homologado podrá entrenar su fold de evaluación. La mejora de datos es una hipótesis, no una conclusión garantizada.

La autoría de ese nuevo TRAIN está iniciada; aún no es banco validado ni resultado de otra prueba. La revisión separada recibirá solo preguntas/documentos sin etiquetas. Modelos, pérdida, umbral y errores actuales quedan congelados. Los nuevos datos se destinan a entrenamiento; no se anuncian como la validación fresca del paso4. Se reutilizarán las432 features ya capturadas cuando corresponda, sin repetir el encoder para cerrar observaciones.

Paso3 sigue sin cerrar. Original88, seis conjuntos, fidelidad, recursos≤1,5×, estabilidad, reserva pequeña y V6 permanecen pendientes/cerrados según sus puertas. Producto/default, DB, secretos, dependencias, dashboard, rama, commit y push sin cambios. No se actualizan gráficas como mejora de producto. Objetivo íntegro activo,2/5 etapas cerradas; no bloqueado por permisos.

Artefactos: outputs/iromem/evaluation/frozen-utility-readout-v1-2026-10-22/.
