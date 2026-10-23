# Vías de evidencia: pase puntual de calidad, sin mejora aprobada

Estado 2026-10-19: goal activo, baseline conservada, V6 sin llamadas. Se implementaron vías separadas de dato y corrección para el mismo referente/contexto, sin veto global ni identidad inventada. Cada variante está disponible explícitamente; ninguna supera todas las puertas. Se conservaron consultas, gold, versiones, fallos y recibos.

## Qué cambió y qué se midió

Scoped V1 hace dos choices por cada top8 en un HTTP y compone todos los positivos. La traza scoped_checks conserva dato_con_contexto y corrección_con_contexto, revalidados por servicio y auditor JSON independiente. Son juicios fallables del modelo, no prueba de verdad. Scoped V2 solo acorta descripciones de opciones; Scoped V3 solo acorta/aplana alias de memorias/preguntas y remapea claves de respuesta. Pruebas de reversión exigen igualdad del resto del payload. No cambian SQL, dependencias, puertos o proveedor/política predeterminados.

| Versión | Prueba | Calidad observada | Coste/puerta | Decisión |
| --- | --- | --- | --- | --- |
| Scoped V1 | Piloto20 operaciones contra binaryV5 | Gana1 ID útil anotado, elimina2 nopermitidos; cero pérdidas/degradaciones |1,493×; pasa piloto | Abrió comparación completa |
| Scoped V1 | V5 completa,64 pares/128 operaciones contra Decisions/base | Pasa esa corrida: cero ID ajeno/pérdida de la base, dos abstenciones útiles resueltas y nueve casos con evidencia adicional |1,554×>1,5×; falla | No amplía ni promueve |
| Scoped V2 | Piloto20 operaciones contra V1 | Agrega la tapa de depósito ante pregunta por acometida; cero pérdidas |−6,36%de coste y recursos pasan; calidad falla | Descarta ampliación |
| Scoped V3 | Piloto20 operaciones contra V1 | Mismas selecciones útiles, cero ID ajeno/pérdida/degradación |−4,32%de coste; pasa piloto | Abrió comparación completa |
| Scoped V3 | V5 completa,64 pares/128 operaciones contra Decisions/base | Un ID ajeno; pierde además un ID anotado útil frente aV1 | Un HTTP 504 de la base sin recibo capturado impide ratio completo | No amplía ni promueve |

Los pilotos son conocidos y no son otra validación nueva. Cambios de formato coinciden con regresiones, pero no se atribuye toda diferencia a ese cambio: el diagnóstico A/A posterior encontró variación también en V1 sin cambiar formato. No se elige la mejor ejecución para sustituir otra.

## Scoped V1: cifras completas del pase puntual

| CategoríaV5 | Base:casos con semilla permitida/vacía correcta | Scoped V1 |
| --- | ---: | ---: |
| Apoyo |16/16|16/16|
| Refutación |15/16|16/16|
| Parcial |15/16|16/16|
| Ausencia |15/16|16/16|

En positivos significa al menos un ID permitido y ninguno ajeno; no que todas las partes de una respuesta estén completas. Ausencia exige semilla vacía. V5 no anota cobertura requerida por parte como V6; conservar los IDs útiles seleccionados por la base no prueba recuperar todo el gold. Las discrepancias retrospectivas de contexto de V5 siguen aparte, sin reetiquetar o recalcular los resultados originales. No se midió verdad de una respuesta final ni generalización poblacional.

| Recursos del runner | Base | Scoped V1 |
| --- | ---: | ---: |
| Coste reportado USD |0,003480200|0,005407836|
| Mediana ms |413,90|358,36|
| p95 ms |593,28|408,81|
| p99 ms |781,05|461,14|
| Tokens de entrada del clasificador |34802|128758|
| Bytes de semilla:mediana/p95 |174,5/208,85|180/397,75|

Mejora la latencia en esa corrida y aumenta tamaño de semilla al conservar más recuerdos. Esos tokens son del clasificador y los bytes no son tokens enviados aCodex. No se midió integración o recorrido completo de respuesta. La puerta exige coste y p50/p95<=1,5×; el coste falló y no se compensó con latencia/calidad.

[Reporte completoV1](../../outputs/iromem/evaluation/per-memory-scoped-v1-development-2026-10-19/paired-v5/report.json).

## Scoped V3: errores conservados

EnV3 la semilla de06803 incluye el ID correcto06806 y también06807: el banco permite la corrección explícita de autoría deLuciérnaga, mientras06807 habla deNeblina. No se transfieren referentes. El modelo marcó la vía de corrección de ambos como verdadera; la composición local fue fiel a juicios incorrectos. Se informa ID no permitido, no una respuesta generada falsa: el producto compone evidencia.

La preservación adicional contra V1 detectó pérdida de06912 en06905. Es la cita futura para probar faros: el goldoriginal la anota útil y la revisión retrospectiva la cuestiona como contexto. Se mantiene la pérdida según la puerta fijada y no se retira ese IDpara aprobar. No hay pérdida de ID útil seleccionado por la base contemporánea, pero sí respecto de V1.

La base devolvió HTTP 504 en07105 tras7247,69 ms. El transporte anterior retornaba antes de leer el cuerpo de error: sabemos que no capturó recibo, no si el proveedor lo envió ni si cobró. Su coste no se supone cero. La candidata completó64/64 y la base63/64. El reporte cuenta tres casos que pasan de ausencia/degradación a evidencia: dos eran abstenciones y uno fue el fallo técnico; no llamar a los tres una mejora semántica.

CosteV3:candidata USD0,005173980 y base USD0,003428200 conocidos más un importe desconocido. Ratio de coste completo:null, no1,49. Medianas base/candidata383,03/361,71 ms; p95 1367,24/439,73 ms; p99 8189,71/4936,98 ms. Muestra pequeña con colas reales y un error; no garantiza latencia futura. Falla calidad y recursos, sin abrir V4/V2 o V6.

[ReporteV3](../../outputs/iromem/evaluation/per-memory-scoped-v3-development-2026-10-19/paired-v5/report.json), [preservación de V1](../../outputs/iromem/evaluation/per-memory-scoped-v3-development-2026-10-19/paired-v5/preservation.json).

## A/A: el pase de V1 no demostró estabilidad

Se fijaron ocho consultas conocidas, tres rondas y24 llamadasV1 sin cambios de parámetros, candidatos o constructor. Todos los resultados se conservaron, sin escoger la mejor respuesta ni votar; mismo modelo servidoJev 1.13-20260917. Las24 completaron con recibos, ninguna abstención completa ante evidencia anotada. Siete consultas conservaron su conjunto seleccionado;06803 varió:

| Repetición de06803 | IDs seleccionados |
| --- | --- |
|0|06806|
|1|06806,06807 no permitido|
|2|06806|

Mismos candidatos comprobados y mismo número de entrada2051 tokens en las tres. Igualdad de payload por construcción de consulta/candidatos/proveedor/política; no se capturaron bytes de red. Un ID no permitido en una de estas tres repeticiones muestra que el error aparece sin optimizar formato. No identifica su causa interna ni estima una tasa de error poblacional. Tres repeticiones tampoco bastan para demostrar determinismo en los otros casos. El pase anterior de V1 se conserva como un resultado de aquella corrida, no como garantía estable.

[Protocolo previoA/A](scoped-stability-protocol-2026-10-19.md), [todas las repeticiones](../../outputs/iromem/evaluation/scoped-stability-2026-10-19/report.json).

## Arreglo de registro y siguiente puerta

El transporte ahora lee cuerpos HTTP de error completos hasta el límite existente. Conserva campos válidos de consumo cuando hay JSON, manteniendo el fallo/status y ninguna semilla fiable. Pruebas cubren coste positivo/cero, HTML, cuerpo ausente/excedido. No se registran cuerpos crudos, no hay reintento automático y no se recupera retroactivamente el504. Prueba local/compilación no demuestra qué contenía aquella respuesta real.

108 pruebas workspace pasan y formato correcto. El contrato anotado también pasa 26/26; ese comprobador no mide extracción automática ni calidad de recuperación. La base temporal propia compartida se eliminó; ningún servidor iniciado ni dashboard reiniciado. `README` y configuración predeterminada se conservan. V6 permanece congelada, con revisión separada y cero llamadas.

Siguiente investigación: verificación selectiva con otro modelo sobre las propuestas que solo se apoyan en corrección de premisa, aisladas de otros recuerdos; conservar hechos/partes directas y medir cada etapa antes de adoptar. No asumir que acuerdo o identidad literal garantizan verdad. Añadir una petición por corrección tiene coste/latencia: el piloto deberá medirlo completo y conservar las puertas. Un recorte global a7 tiene cobertura retrospectiva suficiente para los IDs seleccionados porV1 en V5, pero no es una política implementada ni un pase: las experiencias previas impiden promover por cobertura sola. Ninguna reserva se abre para ajustar.

## Contabilidad reconciliada

Esta tanda:340 operaciones de consultas conocidas, dos campañas emparejadas completas, pilotos y diagnósticoA/A. **USD0,024224412 conocidos más una operación de coste desconocido**. No son340 casos nuevos o independientes. Acumulado del objetivo: **2945 operaciones,19 campañas emparejadas y USD0,207637306 conocidos más tres operaciones de importe desconocido**. Excluye Codex de programación/revisión; son consumos reportados, no auditoría de factura.

Se reconciliaron diarios únicos, costes y contadores contra snapshot012; no se descartaron la corrida fallida ni el504. [Reconciliación contable](../../outputs/iromem/evaluation/quality-goal-2026-10-19/accounting-reconciliation.json), [nota del recibo HTTP no capturado](../../outputs/iromem/evaluation/quality-goal-2026-10-19/http-error-receipt-gap.json). El goal sigue activo: faltan mecanismo estable/aprobado, contraste nuevo y adopción demostrada.

Actualización posterior: esa hipótesis ya está implementada como variante optativa y supera112pruebas locales; el piloto sigue sin llamadas por autorización explícita pendiente tras rechazo automático. No se presenta como mejora medida. [Estado vigente](per-memory-correction-verified-v1-results-2026-10-19.md).
