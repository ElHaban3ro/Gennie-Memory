# Extracción V3 estricta: mejora de aceptación, puerta global fallida

Se ejecutaron los116textos conocidos completos con la guíaV3y el contrato de dominios explícitos. Hubo10tareas de extracción y20críticas separadas, todas con contexto nuevo y sin historial compartido. Los críticos no recibieron etiquetas, frames manuales, pares, resultados antiguos ni la crítica del otro. La evaluación es desarrollo conocido de una misma familia; no revisión humana, independencia de familia ni cohorte nueva.

Se conserva una entrega por lote sin reparación tras feedback, todas las ausencias, los raws originales y los dictámenes. El vocabulario, la guía, el validador y el orden cambiaron respecto de la tanda anterior: la comparación es descriptiva, no causal de un único factor.

## Métricas sobre el mismo banco de116textos

| Medida | Tanda anterior | V3estricta |
| --- | --- | --- |
| Estructuralmente completas |112/116|101/116|
| Parciales |4|1|
| Inválidas |0|2|
| No disponibles |0|12|
| Aceptadas en las4dimensiones por ambos críticos |65/116|98/116|
| Fuentes aceptadas por ambos |45/83|65/83|
| Preguntas aceptadas por ambos |20/33|33/33|
| Desacuerdos entre críticos por dimensión |46|18|

Las98aceptaciones conjuntas son un resultado de revisión, no98aciertos de recuperación ni precisión poblacional. Los20dictámenes superan los controles de integridad; hay403/464dimensiones faithful en el críticoA y405/464enB. La puerta exige116completas y464dimensiones faithful por cada crítico: **no pasa**.

Hay59entradas aceptadas en ambas tandas,39aceptadas nuevas y6antes aceptadas que ahora no lo están. Cinco de esas pérdidas corresponden al lote ausente y una a una representación inválida. Las ganancias no borran las pérdidas. El conjunto y los dictámenes anteriores permanecen intactos; no se eligió al crítico favorable ni se adjudicaron discrepancias retrospectivamente.

## Causas preservadas

Un lote fuente de12textos no creó su entrega: una variable con bytes del paquete se reutilizó como referencia antes de calcular SHA256. La ausencia fue confirmada al terminar el agente y el AST del script muestra la asignación inicial de bytes, su sobrescritura y la llamada posterior. Se preservó el script sin ejecutarlo ni repararlo para rescatar sus decisiones. Las12filas siguen en el denominador y ambos críticos registraron su indisponibilidad.

Las dos entradas inválidas confunden el targetAtom de una restricción con unaProposition. También tienen defectos semánticos: el miembro exclusivo pierde su resolución y una descripción bajo uso negado se transforma en procedencia factual positiva. Corregir únicamente el JSON no resolvería esas críticas.

Otras cuatro fuentes estructuralmente completas quedan sin aceptación conjunta: cita de clase tomada de otra cláusula, procedencia explícita reducida a asociación genérica, atributo usado como clase del evento y condición nueva sin el ámbito de una reserva pasada. En el último caso los críticos discrepan; ambos dictámenes permanecen visibles. La entrada parcial sobre horario y objeto recibido sí fue aceptada semánticamente por ambos, pero conserva su marca unsupported y no se cuenta como estructuralmente completa.

El [triage íntegro](../../outputs/iromem/evaluation/documentary-v3-strict-extraction-2026-10-21/fidelity-triage.json) enlaza las18entradas sin aceptación conjunta y la entrada parcial aceptada con sus textos, raws, diagnósticos, citas y juicios originales. Se conservan tanto problemas de entrega/formato como problemas semánticos; no se reduce el banco a las104entregadas ni a las98aceptadas.

## Coste, tiempo y estado

Cero operaciones nuevas de proveedor, cero peticionesAPI yUSD0nuevos de OpenRouter. Las30tareas de subagente Codex tienen coste/tokens desconocidos y no se presentan como gratuitas ni como consumo de producción. El intervalo observado entre el primer registro de despacho y la última revisión recibida fue1141,304s, unos19minutos, con hasta3subagentes simultáneos. Incluye coordinación y trabajo local, no mide latencia por modelo ni del producto. Los agentes de esta tanda terminaron; no quedó un proceso esperando autorización.

El acumulado histórico de proveedor sigue en6129operaciones lógicas,23campañas emparejadas completas,USD4,565416508conocidos y20operaciones con coste desconocido. La confirmación adicional de agentes resultó innecesaria: la autorización previa seguía vigente y el cierre de los anteriores ya se había cumplido. Se usaron contextos nuevos, sin reactivar los antiguos ni enviar modelos frontera porAPI.

Código y binario idénticos a los verificados con619pruebas/24omitidas; no se repitió Cargo. Se verificaron hashes congelados, recepción de todos los lotes,20críticas y10controles negativos del comprobador de revisiones. Esas comprobaciones no sustituyen la revisión semántica.

No se ejecutó utilidad automática sobre raws fallidos, baseline, recursos/estabilidad de producción niV6. Las cifras manuales36/39útiles,3respuestas completas y0selecciones en60none siguen siendo históricas; no se recalcularon como mejora automática. No hubo cambios de producto ni de predeterminado, y no hay adopción.

## Siguiente iteración del paso3

El objetivo permanece activo. Primero se debe separar la serialización mecánica de las decisiones semánticas y probar constructores genéricosAtom/Proposition. En paralelo, resolver capacidad y reglas para miembros restringidos, descriptores bajo negación, procedencia explícita, atribución local de clases y condiciones de eventos pasados. Se fijaron contraejemplos para cada familia en `retrospective-next-action.json`. No se entrenará el matcher contra raws defectuosos ni se quitará unsupported por conveniencia.

La siguiente versión debe predefinir su mecanismo y validar el banco completo con revisión separada antes de utilidad automática; la cohorte ciega continúa reservada. Los resultados negativos de esta tanda no se sustituyen por un reintento favorable.

Artefactos: `outputs/iromem/evaluation/documentary-v3-strict-extraction-2026-10-21/`. Preparación y protocolo originales conservados, además de entregas, recibos, críticas, matriz, discrepancias, causas y manifiestos.
