# V6: diversidad frente a repetición de ejemplos antiguos

Fecha: 2026-10-22. Contraste prospectivo del paso 3. Ambos brazos parten del mismo padre V5 matched, continúan una época de208presentaciones y52updates con optimizador nuevo, y conservan176TRAIN de fondo. Uno añade32presentaciones antiguas f05; el otro32ejemplos nuevos revisados a ciegas. [Protocolo](utility-learning-v6-protocol-2026-10-22.md).

| Conjunto | Padre inicial: aciertos · útiles · ajenos | Repetición: aciertos · útiles · ajenos | Diversidad: aciertos · útiles · ajenos |
| --- | --- | --- | --- |
|background|155/176 · 70/72 · 19|173/176 · 69/72 · 0|168/176 · 70/72 · 6|
|augmentation|28/32 · 16/16 · 4|23/32 · 10/16 · 3|31/32 · 16/16 · 1|
|development|14/24 · 10/12 · 8|14/24 · 7/12 · 5|16/24 · 10/12 · 6|
|known|59/64 · 13/17 · 1|53/64 · 7/17 · 1|56/64 · 10/17 · 1|
|probe|7/8 · 3/4 · 0|7/8 · 3/4 · 0|7/8 · 3/4 · 0|

El suplemento nuevo augmentation32 es TRAIN en diversidad; en repetición no se entrenó. Su acierto no demuestra generalización. Fondo176 entrenado en ambos, desarrollo24/conocidos64 y sonda8 previamente observados; ninguno es cohorte fresca. La base sin adaptar es una referencia histórica del mismo banco: desarrollo13/24,12/12útiles,11ajenos; conocidos53/64,16/17útiles,10ajenos. El padre inicial se midió contemporáneamente en ambos brazos y se verificaron predicciones iguales, diferencia de score≤0,02.

## Puerta original y cambios por caso

- reweight: puerta original aprobada=False; desarrollo14/24,conocidos53/64.
  - development: frente al padre ganancias=3,degradaciones=3,nuevas pérdidas útiles=['d_f08_06', 'd_f09_05', 'd_f09_08'],nuevos ajenos=[]. Frente a sin adaptar nuevas pérdidas útiles=['d_f08_06', 'd_f09_01', 'd_f09_04', 'd_f09_05', 'd_f09_08'],nuevos ajenos=[].
  - known: frente al padre ganancias=0,degradaciones=6,nuevas pérdidas útiles=['cv3_03_00', 'cv3_08_01', 'cv3_08_02', 'cv3_10_00', 'cv3_14_00', 'cv3_16_00'],nuevos ajenos=[]. Frente a sin adaptar nuevas pérdidas útiles=['cv3_03_00', 'cv3_07_00', 'cv3_08_01', 'cv3_08_02', 'cv3_10_00', 'cv3_12_00', 'cv3_13_00', 'cv3_14_00', 'cv3_16_00'],nuevos ajenos=[].
- diverse: puerta original aprobada=False; desarrollo16/24,conocidos56/64.
  - development: frente al padre ganancias=2,degradaciones=0,nuevas pérdidas útiles=[],nuevos ajenos=[]. Frente a sin adaptar nuevas pérdidas útiles=['d_f09_01', 'd_f09_04'],nuevos ajenos=[].
  - known: frente al padre ganancias=0,degradaciones=3,nuevas pérdidas útiles=['cv3_08_02', 'cv3_14_00', 'cv3_16_00'],nuevos ajenos=[]. Frente a sin adaptar nuevas pérdidas útiles=['cv3_07_00', 'cv3_08_02', 'cv3_12_00', 'cv3_13_00', 'cv3_14_00', 'cv3_16_00'],nuevos ajenos=[].

Comparación emparejada diversidad contra repetición, completa en paired-comparison-root.json:
- development: 3correcciones y1degradaciones.
- known: 3correcciones y0degradaciones.
- probe: 0correcciones y0degradaciones.

Un beneficio frente al control no sustituye superar al padre ni la puerta original24/24+64/64,cero pérdidas/ajenos/degradaciones y≥1ganancia. No se elige un checkpoint, umbral, época o versión por estas cifras. La comparación prueba el paquete definido, no un efecto puro de sintaxis o una garantía sobre capacidad/población.

## Integridad, recursos y coste

Datos:32/32etiquetas coinciden entre autor nuevo y revisor con etiquetas/predicciones ocultas;16equivalencias pertinentes,sin ambigüedades. Ocho escenarios repetidos con cuatro variantes, dos plantillas: diversidad limitada. No datos humanos ni independencia entre familias de modelos. Copias de datos, fuentes, tokens y críticas conservadas. Suplementos16B/16none, fondo176yroles/targets idénticos en cada posición; controlochoantiguos×4. MT19937Python con índices congelados, sin score en ordenamiento.

Antes de inferir quedaron congelados plan d358bc810a10eb16cc9d2c754f7557b9599bf4ab5728e9ba5c421adf764a7037, código, tokens, inputs, hiperparámetros y checkpoint inicial7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74. Preflight nativo1803/1803antesdelprimerjournal. Mismo padre exacto32arraysFP32; baseFP16inmutable. Media de cuatro gradientes antes de clip1,lossV2CE+hingepositivo2,freshAdamWlr1e-4igual. Cada208backward52updates queda ligado a índice/ID/target/tokenhash; estado finito. Checkpoint final guardado antes de evaluar y fingerprints verificados al final.

Auditor offline all_passed=True; counts={'evaluation_scores': 1216, 'training_forward_backward': 416, 'optimizer_updates': 104}. Integridad no equivale a calidad. Ambos procesos tienen salida0observada y ausencia de su PID/script al cierre; no reintentos.

- reweight: entrenamiento32.133s,total94.069s,p50forwardfinal95.380ms,p95nearest-rank110.399ms;608scores_eval+208forward/backward,52updates. Checkpoint7eea01861727fecc3f338a08af09b228c4fd32009f1bc97f476fd8e6d81cf056.
- diverse: entrenamiento32.561s,total95.802s,p50forwardfinal97.958ms,p95nearest-rank119.409ms;608scores_eval+208forward/backward,52updates. Checkpointc7f7eeb216822c64b2dc5e75a92a77de55cae515127ac932f37cd9178ae3a952.

Total de la campaña:1216scores de evaluación+416forward/backward de entrenamiento=1632forward,104updates. Duración de los dos CLI sumada, no latencia de recuperación completa. No mide la puerta de coste/p50/p95≤1,5× contra el producto ni estabilidad entre campañas.

Nueva API0llamadas/USD0. Costes económico local y Codex no cuantificados; un contexto nuevo de autoría y revisores existentes reutilizados. Acumulado remoto6155operaciones/23campañas completas,USD4,566379428conocidos más24operaciones sin importe, sin cambio. Sin servicios, puertos, descargas, dependencias, DB, secretos, ramas, commits o cambios en el producto. Rust879default/881HTTP2 heredados únicamente tras revalidar213hashes fuente y binario; ignoradas24/75. No Cargo nuevo por esta evaluación y documentación.

Reservas original24 y cohorte finalV6cerradas. Este V6 de entrenamiento no abre esa cohorte. Cinco pasos íntegros, objetivo activo; ninguna adopción automática.

## Decisión y diagnóstico retrospectivo

Ambos brazos fallan; ninguno se adopta. Diversidad frente a repetición sube conocidos53→56/64 y útiles7→10/17, pero frente al padre baja59→56/64 y útiles13→10/17. La mejora relativa al control no demuestra mejora aceptable del sistema. La sonda conserva7/8 y sigue perdiendo el nominal útil del atlas. El entrenamiento obtiene31/32en nuevos datos, mientras no resuelve la transferencia buscada. La pérdida del padre frente a sin adaptar continúa; ninguna puerta se rebaja.

La revisión causal independiente descarta que esta exposición/diversidad concreta sea suficiente. No prueba que los datos nunca sirvan o que haya una causa única. La auditoría textual offline siguiente se completó sin modelos: los ocho nominales útiles nuevos usan «quien ... fue» y se seleccionan; los dos nominales útiles de la sonda usan «lo ... [agente]», con atlas perdido(score0,426322) y puente conservado(0,999751). Una regla simple de rechazo de toda forma directa queda contradicha por el puente. Personas, longitud de título, léxico y composición también varían; no afirmar que la construcción sea causa demostrada ni otra receta validada. Fuente y scores íntegros en posthoc-composition-audit.json.

Frente a la candidata fija anterior hay un intercambio que el agregado56/64 oculta: diversidad conserva cv3_10_00 (refutación de atribución) pero pierde cv3_16_00 (contexto parcial de programa/entidad), ambas útiles. Frente al padre, cv3_10_00 ya estaba conservada; no contar esa diferencia como ganancia nueva sobre el padre.

Siguiente decisión antes de otra optimización: comprobar con un único cruce de construcciones/identidades si la sensibilidad al orden de agente persiste manteniendo hechos y léxico, sin entrenar observados. No hay evidencia para otro barrido de épocas, penalización o escala. Los pendientes de cobertura útil A/C y conservación del padre siguen siendo parte del mismo objetivo y no desaparecen porque se focalice una familia.

Auditoría independiente:16949/16949repetición y16824/16824diversidad, más9/9emparejadas;33782comprobaciones de integridad, sin otra inferencia. Se reconstruyen registros, pérdidas escalares y correspondencia de datos; no hay tensores completos de gradientes/estados AdamW o base final persistidos, de modo que su inmutabilidad/operaciones se sustentan en fuente y fingerprints registrados, no en una reconstrucción externa del entrenamiento. Cero calidad aprobada por integridad.
