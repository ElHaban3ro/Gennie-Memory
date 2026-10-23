# Atribución corregida: contraste prospectivo de realización

Fecha:2026-10-22. Diagnóstico aislado del paso3; la última puerta local sigue fallida (desarrollo16/24,conocidos56/64). Esta sonda no reevalúa esa puerta, no mide mejora del producto ni permite abrir reserva. [Protocolo](attribution-realization-probe-v1-protocol-2026-10-22.md).

| Estado | Aciertos/8 | Útiles/4 | Ajenos/4 |
| --- | ---: | ---: | ---: |
|untrained|7|4|1|
|parent|7|3|0|
|candidate|7|3|0|

Ocho casos=atlas/puente × pasiva/nominal × mismo/otro objeto. Personas y nombres fijados en todas las celdas; controles irrelevantes cambian solo Cauce→Rama. No mes en documento; lo útil es refutar la atribución a Helena. Revisor nativo nuevo sin etiquetas ni predicciones:8/8concordantes,sin ambigüedad,y4equivalencias semánticas pertinentes. Primeras versiones y revisión preservadas: se retiró detalle de catálogo solo presente en pasivas, ANTESde modelos, sin cambiar etiquetas. Revisión sintética de mismafamilia,no humana ni independencia de modelo.

## Todas las decisiones y márgenes

| Dominio | Forma | Identidad | Útil | Base margen/selección | Padre margen/selección | Candidato margen/selección |
| --- | --- | --- | --- | --- | --- | --- |
|atlas|passive_correction|same|1|4.375000/1|8.046875/1|5.078125/1|
|atlas|passive_correction|other|0|-2.648438/0|-7.062500/0|-7.843750/0|
|atlas|nominal_correction|same|1|2.906250/1|-0.562500/0|-2.203125/0|
|atlas|nominal_correction|other|0|-2.703125/0|-9.421875/0|-9.468750/0|
|bridge|passive_correction|same|1|4.765625/1|8.195312/1|4.289062/1|
|bridge|passive_correction|other|0|-2.351562/0|-7.007812/0|-8.140625/0|
|bridge|nominal_correction|same|1|5.398438/1|6.070312/1|2.750000/1|
|bridge|nominal_correction|other|0|0.000000/1|-7.296875/0|-6.812500/0|

## Contrastes fijados antes de inferir

- untrained: cambios de selección entre formas equivalentes 1/4.
- parent: cambios de selección entre formas equivalentes 1/4.
- candidate: cambios de selección entre formas equivalentes 1/4.

Todos los contrastes de forma, dominio y cambios entre estados se conservan en auditoría/rows; no elegir celdas ni buscar umbrales. Si hay cambio de decisión, evidencia sensibilidad en esos textos; si solo cambia margen, no es una pérdida de selección. El cruce no aísla sintaxis pura: forma cambia léxico/sintaxis; dominio cambia objeto/verbo. No prueba una causa única, generalización poblacional o que entrenamiento adicional vaya a corregirla.

## Ejecución, integridad y coste

Una corrida24forward,0backward,0updates,10.289s. Misma instancia:baseFP16sinLoRA antes de insertar adaptadores; luego padre y candidato FP32 exactos. Estado modelo eval, cache nueva por forward, corte0,5sin cambio. Fingerprints antes/después iguales dentro de cada estado; base envuelta validada por separado. Proceso/terminal real en terminal-observation.json. Preflight nativo completo antes de journal; no repetir la inversión temporal de la fase138. Auditoría offline all_passed=True,counts={'checks': 1260, 'passed': 1260, 'cells': 8, 'arms': 3, 'forwards': 24, 'backwards': 0, 'optimizer_updates': 0}; integridad no implica calidad.

API nueva0llamadas/USD0. Coste económico local/Codex no cuantificado; un contexto nativo nuevo para revisión ciega y técnico existente reutilizado. Sin servicios/puertos/descargas/dependencias/DB/secretos/cambios de producto. Acumulado remoto6155operaciones/23campañas,USD4,566379428conocidos+24operaciones sin importe; sin cambio. Tiempos de esta sonda no acreditan recursos de ruta completa≤1,5×. Rust/heredados solo tras verificar213fuentes y binario; no se reejecuta Cargo por evaluación/docs.

La puerta original y los cinco puntos permanecen íntegros. Reservas24/V6cerradas; ninguna adopción o modificación del producto.

## Interpretación independiente y siguiente intervención

La revisión causal confirma una pérdida por realización en atlas: base conserva pasiva0,987568/nominal0,948155; padre0,999680/0,362969 y candidato0,993807/0,099470. En puente ambas útiles se conservan. Es una interacción textual concreta, no una regla de que toda nominalización falle; el fallo ya existe en el padre, no se atribuye exclusivamente a la delta. El ajeno nominal del puente en base tiene scoreexacto0,5 y se selecciona por el corte congelado>=0,5; no describirlo como aceptación de alta confianza ni cambiar el criterio tras verlo.

La única intervención propuesta es comparar32ejemplos diversos y emparejados de corrección de atribución frente a32presentaciones antiguas f05, con igual peso de clases y presupuesto de entrenamiento. Se conserva el resto de TRAIN para controlar cambio de exposición. Nuevos nombres/textos, revisión ciega y separación de los88observados,estos8yreservas. No se crearon datos ni se entrenó en esta sonda; protocolo/dataset/control deben congelarse por separado antes de la próxima ejecución. La propuesta prueba diversidad/contrastes, sin garantía de mejora ni pretensión de aislar cada componente de la realización.
