# Diagnóstico de latencia del consenso paralelo

Fecha: 2026-10-19. Desarrollo conocido, sin promoción ni cambio de baseline. El objetivo de cinco puertas sigue activo.

## Pregunta y método

El consenso que pasó calidad falló recursos: p95 906,11 frente a474,00ms, razón1,912 y límite1,5. La reproducción del cálculo sobre el informe conservado volvió a fallar. Antes de otra instrucción se añadieron duración y versión HTTP por voto, sin cambiar modelo, instrucciones, top16, aceptación por acuerdo o tratamiento de errores.

Hipótesis previas: una petición rezagada domina el máximo; apertura de conexiones; variabilidad/concurrencia del proveedor; espera local de los hilos. Se eligieron por tiempo las tres operaciones más rápidas y las tres más lentas del diario original, sin usar gold para seleccionarlas. Tres rondas en orden alternado, rotado e invertido:18 operaciones,36 votos HTTP. El [preflight](../../outputs/iromem/evaluation/roles-consensus-latency-2026-10-19/preflight.json) precedió las llamadas; el [diario](../../outputs/iromem/evaluation/roles-consensus-latency-2026-10-19/journal.jsonl) conserva recibos y trazas.

`consensus.transport`, opcional, conserva `http_elapsed_ms[2]` y `http_versions[2]` solo cuando ambos votos completan y se interpretan. Cada intervalo empieza antes de construir/enviar la petición y acaba tras leer y decodificar JSON: incluye trabajo del cliente, TLS/red, espera/procesamiento remoto y lectura; no separa sus componentes. El total de clasificación conserva su medición anterior. Las trazas antiguas y por lote pueden no tener transporte. Un fallo sigue degradando y conservando consumo conocido; no se fabrica duración de la etapa fallida. Los tiempos son diagnósticos y no intervienen en aceptación o validación semántica.

## Resultado

| Medida | Resultado |
| --- | ---: |
| Operaciones completas |18/18|
| Votos HTTP exitosos |36/36|
| Versiones observadas |HTTP/1.1 en36/36|
| p50 operación |379,50ms|
| p95 operación |875,08ms|
| Mediana de clasificación menos máximo HTTP |0,373ms|
| Esa diferencia por debajo de1ms |17/18|
| Máxima diferencia |4,457ms|
| Primera operación |840,97ms|
| Operación posterior más lenta |1.068,32ms|
| Coste comunicado |USD0,001783656|

Todas las selecciones de estas seis preguntas conocidas fueron permitidas en las tres repeticiones. Esto verifica la ruta instrumentada, no generalización de calidad. No se compara este p95 seleccionado con el de una campaña completa ni se sustituye la puerta fallida. Los datos completos están en el [informe JSON](../../outputs/iromem/evaluation/roles-consensus-latency-2026-10-19/report.json).

La espera fuera de las peticiones medidas no explica los picos de esta muestra. El primer arranque tampoco los explica por sí solo: otra petición tardó1.061ms en la tercera ronda. Sigue sin resolverse si la espera interior pertenece al cliente HTTP, la red o el proveedor. Que se observe HTTP/1.1 no demuestra que HTTP/2 lo mejore.

## Contraste HTTP/2 no ejecutado

Se preparó y retiró una feature optativa `http2-transport` sin llamadas remotas con ella. Cargo no pudo resolver `h2 ^0.4.14` exigido por el hyper1.12.0 existente: el índice accesible solo ofrecía0.4.13 y anteriores, tanto offline como con acceso conectado explícito. No se sustituyeron versiones del workspace ni se mantuvieron dependencias o feature nuevas. Este impedimento afecta esa alternativa, no demuestra inviabilidad del objetivo.

La [feature oficial de reqwest0.12.28](https://docs.rs/crate/reqwest/0.12.28/features) requiere h2; la comprobación puntual de [RustSec2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258.html) exige h2≥0.4.16 para ese aviso. [Docs.rs](https://docs.rs/crate/h2/0.4.16) lista la versión corregida y su procedencia hyperium/h2, sin que el índice disponible la resolviera. No se usaron fuentes alternativas ni versiones antiguas. Estas consultas de metadatos y avisos no son auditoría exhaustiva de la cadena o garantía contra compromiso. No se atribuye un rendimiento a HTTP/2 sin ejecución.

## Decisión

Retener la observabilidad del consenso para siguientes contrastes y conservar la puerta de recursos. No cambiar más prompts por el síntoma de latencia ni promover por esta muestra. El siguiente experimento debe declarar una modificación concreta que reduzca las esperas medidas y comprobarla contra baseline en desarrollo completo antes de usar v5. No hay servicios iniciados ni cambios del dashboard. Se eliminó la DB temporal propia y el binario temporal; se conservan fuentes, hashes, diarios y recibos, sin prometer reproducción bit a bit.
