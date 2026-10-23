# Revisión única V1: mismo resultado, mayor coste

UTC 2026-10-21. Se implementó una segunda revisión obligatoria de todos los ocho borradores del piloto anterior. **Los ocho JSON finales son iguales a sus borradores: cero reparaciones y cero regresiones de contenido.** No se amplía ni adopta esta variante. Goal activo en paso 3. [Protocolo](grounded-revision-v1-protocol-2026-10-21.md), [verificación](../../outputs/iromem/evaluation/grounded-revision-v1-2026-10-21/verification.json).

El preparador y runner vinculan cada borrador a su petición y recibo terminal mediante hashes. La entrada conserva el texto fuente o pregunta por separado, ontología y esquema, y añade únicamente el borrador propio y el error local recalculado, o null. Se revisan también los cuatro DTO válidos. La segunda salida siempre es final, aunque empeore; no hay selección del mejor intento, fallback al borrador ni reparación desde gold. El traductor y parser anteriores quedan intactos. [Decisión 0025](../decisions/0025-bounded-documentary-revision.md), [contraste de payloads](../../outputs/iromem/evaluation/grounded-revision-v1-2026-10-21/payload-contrast.json).

| Medida sobre los mismos ocho casos conocidos | Borrador histórico | Revisión final |
| --- | ---: | ---: |
| HTTP 200 | 8/8 | 8/8 |
| DTO completos | 4/8 | 4/8 |
| Fuentes / preguntas válidas | 2/4 y 2/4 | 2/4 y 2/4 |
| Semántica fiel según ambos críticos | 2/8 | 2/8 |
| Fidelidad incluyendo evidencia según ambos | 1/8 | 1/8 |
| Contenidos con defectos semánticos acordados | 6/8 | 6/8 |
| Coste de esa etapa, USD | 0,059038 | 0,091816 |

Dos críticos nuevos sin historial revisaron los ocho originales, finales y borradores, más los cuatro normalizados disponibles. El borrador no era gold. Coinciden en seis defectos semánticos persistentes: O7/pedido, Verbena/huerto, Cima y Atlas/expediente siguen perdidos en campos de identidad. Arce conserva significado y evidencia; la pregunta de autonomía conserva significado, pero la cita del propietario Bruma refiere a sus faros. Los dos rechazan esa atribución. Una observación adicional del adversarial sobre dirección del flujo en op_054 queda registrada sin adjudicarla como regresión: el JSON no cambió. Las críticas son de la misma familia de agentes, no humanas ni independencia poblacional. [16 juicios y resumen](../../outputs/iromem/evaluation/grounded-revision-v1-2026-10-21/blind-review-summary.json).

La igualdad de los ocho JSON se comprobó automáticamente, además de las revisiones. No se infiere corrección porque pase el esquema. Los brazos calculados tampoco cambian: manual 100/100; fuente automática 2 calculados/98 degradados; pregunta automática 6/94; ambos automáticos 0/100. No son cifras de precisión poblacional ni recuperación fiable de 100 casos.

## Coste y latencia

Gasto nuevo por recibos: **USD 0,091816**, sin cargos nuevos desconocidos. Coste lógico de las dos etapas: **USD 0,150854**, 2,5553 veces el borrador solo, sin mejora observada. El padre ya estaba contabilizado y no se suma otra vez al acumulado. Son ejecuciones históricas separadas, no una prueba contemporánea completa de la tubería. [Auditoría de dos etapas](../../outputs/iromem/evaluation/grounded-revision-v1-2026-10-21/stage-cost-audit.json).

Ocho HTTP/2, proveedor OpenAI y alias openai/gpt-4.1, cuatro concurrentes, sin reintentos. Revisión fechada del modelo no comprobada por separado. Estimación anunciada USD 0,21754 y cota conservadora USD 0,741964. Recibos: 47.884 tokens de entrada, 2314 de salida y 14.976 de entrada en caché; son tokens del proveedor experimental, no de Codex.

| Revisión | p50 ms | p95 ms |
| --- | ---: | ---: |
| Fuentes, 4 intentos | 3326,646 | 6026,465 |
| Preguntas, 4 intentos | 1699,301 | 2490,303 |

Runner: 9413,583 ms. No mide recall, hook o respuesta final; no se suman medianas de etapas para inventar latencia end-to-end. No hay pase de recursos ni estabilidad.

## Verificación y continuidad

447 pruebas pasan con y sin HTTP/2; once ejecuciones unitarias nuevas prueban vinculación, aislamiento, inclusión de borradores válidos, rechazo de padres incompletos y ausencia de fallback. La compilación HTTP/2 conserva un aviso de método compartido no utilizado por el crate bootstrap; el evaluador sí lo usa. Compilación, formato y diff-check pasan. Cuatro bancos manuales y nueve informes anteriores, más este, se reproducen byte por byte. Fuente ejecutada y binario CLI con hashes conservados. Snapshots001–056 intactos; cierre nuevo057. Sin cambios de default, etiquetas, README/gráficas, dependencias, SQL, dashboard o servicios. V6 conserva su manifest sin lectura semántica ni llamadas.

Acumulado: **6001 operaciones, 23 campañas emparejadas completas, USD 0,845298508 conocidos más 16 operaciones de coste desconocido**, sin Codex. La nueva tanda añade ocho operaciones; no una campaña completa.

Esta revisión del mismo modelo no aporta evidencia favorable y no justifica más revisiones iguales. La siguiente investigación debe aislar, sin red inicialmente, si la proyección de referencias puede construirse con garantías comprobables de conservación desde menciones, en vez de pedir al modelo duplicar la misma identidad en varios campos. Debe enfrentar antes los contraejemplos ya registrados: propietarios distribuidos, clases declaradas/ausentes, premisas corregibles, cardinalidad, homónimos y relaciones entre miembro y universal. No borrar campos del raw ni reducir el alcance para aprobar. Si no existe una transformación con conservación verificable, registrar la falta y cambiar de hipótesis antes de otro piloto. Calidad completa, banco difícil, recursos/estabilidad y V6 siguen pendientes; goal activo, sin promoción.
