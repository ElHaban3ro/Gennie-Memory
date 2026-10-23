# Candidato de corrección fija: puerta local fallida

Fecha: 2026-10-22. Objetivo original activo en paso3. La corrección que mejoró TRAIN155→174/176 falla fuera del entrenamiento: **72/88 correctos,20/29 útiles conservados y7ajenos**. Frente al padre V5 hay una caída neta73→72/88 y tres pérdidas útiles nuevas; no se incorpora al producto ni se abre reserva. [Protocolo previo](utility-gradient-local-gate-v1-protocol-2026-10-22.md).

## Resultados completos

| Banco | Sin adaptar: aciertos / útiles / ajenos | Padre V5: aciertos / útiles / ajenos | Candidato fijo: aciertos / útiles / ajenos |
| --- | --- | --- | --- |
| Desarrollo24 | 13/24 ·12/12 ·11 |14/24 ·10/12 ·8 |16/24 ·10/12 ·6 |
| Conocidos64 |53/64 ·16/17 ·10 |59/64 ·13/17 ·1 |56/64 ·10/17 ·1 |
| Agregado observado88 |66/88 ·28/29 ·21 |73/88 ·23/29 ·9 |72/88 ·20/29 ·7 |

Comparadores históricos del mismo reranker y textos, no corrida contemporánea ni configuración productiva FTS5/Decisions. Desarrollo y conocidos fueron observados en variantes previas, aunque no se usaron como entrenamiento o para calcular la delta. Ningún agregado acredita generalización fresca ni verdad poblacional. La puerta24/24+64/64,cero pérdidas/ajenos/degradaciones y al menos una ganancia falla; se conserva exactamente.

La disminución de ajenos no compensa la pérdida de evidencia útil. Contra sin adaptar corrige14 casos y degrada8, todos útiles; contra el padre corrige2 y degrada3. Conserva roles A9/14,B5/6,C6/9; rechaza52/59ajenos. No se modifican etiquetas, umbral0,5, pesos o documentos después de observar scores.

## Residuos y siguiente decisión

Los tres fallos nuevos frente al padre son cv3_08_02 (ausencia documental del formato),cv3_10_00 (refutación de quién inauguró),cv3_14_00 (atributo correcto entre dos modos). Los dos primeros tienen scores0,241y0,317, pero el tercero0,212; otros errores tienen confianza próxima a cero o uno. La evidencia no avala que bajar un umbral resuelva el contrato y preserve el rechazo de ajenos.

Persisten confusiones entre regla y episodio individual, disponibilidad y uso efectivo, corrección de premisa y falta de respuesta literal, así como información útil sobre ausencia de un dato. Son clases semánticas distintas; un fit98,9% en TRAIN no las resuelve de forma transferible.

Una comprobación retrospectiva offline encontró ejemplos TRAIN aceptados correctamente para **las tres relaciones de las nuevas pérdidas**: ausencia documental x_f13_q04_m04(score0,999996),refutación de atribución t_f05_01(0,996517) y valores de dos perfiles x_f16_q02_m02(0,999434). Sus contrapartes observadas cv3_08_02/cv3_10_00/cv3_14_00 caen a0,241/0,317/0,212. Fuentes y textos íntegros en posthoc-coverage-witnesses.json. Esto descarta una explicación simple de ausencia de esas familias amplias; no demuestra cobertura representativa, equivalencia de construcción ni interferencia causal. Ampliar sin diagnóstico sería otra conjetura.

Siguiente paso: auditoría offline sistemática de cobertura y construcción TRAIN frente a residuos observados, con crítica independiente. Solo si respalda una hipótesis concreta, preparar un contraste nuevo prospectivo para probarla. Mantener los88 observados fuera de entrenamiento, reservas cerradas y evitar barridos de escala/umbral. Diagnóstico retrospectivo y propuesta no se presentan como mejora demostrada.

## Ejecución y coste

Una sola corrida:88forward,0backward,0updates,11,908s completos. Tiempo por forward p50=101,838ms,p95=111,257ms (nearest-rank,88observaciones); no incluye carga ni búsqueda/composición y no es latencia completa del producto o comparación causal de recursos≤1,5×. Sesión98096/PID20026,exit0 observado; proceso ausente al cierre. Candidato32d62088e13b4e6b4b5f63aec22839e01e8db5e633fadaaf6b02f01dc4356ad7 intacto; baseFP16/LoRAFP32 y fingerprints antes/después iguales.

Nueva API:0llamadas,USD0; coste económico local y Codex no cuantificado. Acumulado remoto permanece6155operaciones/23campañas completas,USD4,566379428conocidos más24operaciones de coste desconocido. No nuevos servicios, puertos, descargas, dependencias, DB o cambios de producto. Pruebas Rust heredadas únicamente tras revalidar los213hashes fuente y binario (879default/881HTTP2,ignoradas24/75); esta tanda solo modifica evaluación y documentación.

## Integridad y límites de revisión

PlanSHA c2a38eb14e41a9abe2949ea887ebd69a1830724e1102a9f85cf49ff519e58f8e congelado antes de inferencia; diario178eventos, los88scores y todas las decisiones conservados. Comprobación estática y compilación raíz realizadas antes de iniciar. La revisión nativa independiente terminó después del inicio:566comprobaciones técnicas pasan y1condición temporal falla(no_model_started); **no se presenta como preflight nativo aprobado**, no se oculta el fallo ni se repite la corrida para cambiar su orden. Es una revisión posterior/concurrente del mismo intento. La integridad de artefactos y la calidad son juicios separados.

Un primer auditor falló por KeyError al esperar una clave terminal que no existe en el esquema; fuente y error se conservaron como auditor-failed-v1. Se corrigió solo el lector offline, sin nuevas inferencias. La revisión causal independiente verificó los tres testigos y sus cuatro hashes fuente. Mantener49/49pares de ranking en conocidos no compensa las pérdidas absolutas de selección. Una sensibilidad a la realización léxica/sintáctica es hipótesis, todavía sin contraste que la confirme.

Auditoría offline independiente final: {'checks': 3845, 'passed': 3845, 'rows': 88, 'development': 24, 'known': 64, 'journal_events': 178, 'forwards': 88, 'backwards': 0, 'optimizer_updates': 0}; all_passed=True. Reconstrucción sin inferencias adicionales.

Reserva original24 y V6 cerradas. Sigue pendiente preservar toda evidencia útil en la puerta local, luego casos nuevos ciegos, comparación productiva completa, recursos/recibos/estabilidad y evaluación final. Los cinco puntos permanecen íntegros; no hay permiso pendiente ni adopción.
