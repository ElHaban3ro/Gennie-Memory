# Instrucciones españolas: mejora parcial, variante no elegible

Fecha UTC: 2026-10-20. La traducción revisada pasa el mínimo conocido, pero **no supera calidad ni recursos del contraste difícil**. La ejecución se detuvo ante un HTTP503 de Decisions/control, sin reintento:46 respuestas válidas,1fallo sin recibo y7operaciones no ejecutadas. No se rellena el resultado con una segunda corrida. El objetivo continúa en el paso3; baseline y V6 permanecen intactos.

## Cambio y comprobaciones

Se implementó únicamente para evaluación `perplexity-spanish`: el mismo modelo `perplexity/pplx-decider-v1.2`, revisión `perplexity/pplx-decider-v1.2-20261016`, y las mismas nueve alternativas que la referencia inglesa. Solo se traducen15tipos de campo revisados (reglas,4definiciones,plantilla,9criterios). Pregunta, fuentes,8analogías,IDs,claves,orden y parámetros se conservan. Restaurar esos campos reproduce exactamente los16payloads mínimos y los54difíciles preparados; la referencia es literalmente la candidata inglesa de nueve alternativas anterior, no la de cuatro.

La revisión bilingüe separada es falible y de la misma familia de modelo, no humana. Se conservan borrador, rechazo inicial y correcciones de equivalencia previas a red. Las pruebas detectaron y corrigieron antes de llamar un escape incorrecto de comillas al incrustar fuentes; el JSON resultante conserva su texto literal. Preparador, runner y reporte comparten la referencia correcta de nueve alternativas; una prueba con recuerdos no vacíos comprueba esa distinción. La preparación de dos brazos rechaza esta variante antes de escribir. No hay política nueva de producto, SQL, dependencias, credenciales ni servicio.

## Métricas

Mínimo conocido:16/16respuestas válidas,64/64roles correctos,17/17evidencias útiles,0extras/pérdidas; costeUSD0,000692820. Pasar este filtro permite el contraste difícil, no demuestra generalización.

El contraste difícil preparó18casos/54operaciones equilibradas entre baseline real, referencia inglesa y candidata española. Llegó a15triples completos más una referencia del caso16antes del503del control. Los últimos7requests no se ejecutaron. No llamar14/17a la candidata como si hubiera omitido las fuentes de casos sin ejecutar: conserva14/14IDs útiles en los15casos comparables, sin pérdida frente al control.

| Métrica en15casos completos | Baseline | Inglés9 | Español9 |
| --- | ---: | ---: | ---: |
| Evidencias útiles conservadas | 9/14 | 14/14 | 14/14 |
| Selecciones no permitidas por desarrollo revisado | 1 | 4 | 3 |
| Roles canónicos correctos de105 | No mide roles | 101/105 | 102/105 |

La diferencia es `cev5_068_07`: inglés transfiere a Luciérnaga una afirmación sobre el libreto de Neblina; español la rechaza. Persisten `cev4_061_10` (archivo de un ítem frente a política global) y `cev5_069_10`/`cev5_069_12` (chequeo genérico/cita futura frente a autonomía actual). Las dos últimas anotaciones conservan ambigüedad residual explícita; no se relabelan tras medir. No se cambia gold histórico ni se presenta esta reanotación de desarrollo como verdad humana. Una diferencia de una pasada no demuestra estabilidad ni que una palabra concreta sea la causa. El campo automático de señal de idioma sigue falso porque exige las54operaciones válidas.

| Medida de operaciones válidas disponibles | Baseline | Inglés9 | Español9 |
| --- | ---: | ---: | ---: |
| Respuestas válidas /18planeadas | 15 | 16 | 15 |
| Coste conocidoUSD | 0,000970300 | 0,000880800 | 0,000978000 |
| Mediana ms | 327,490 | 518,097 | 536,927 |
| p95 ms | 1043,534 | 1067,039 | 559,732 |

Los denominadores difieren: esta tabla no constituye una comparación completa. El p95 es nearest-rank y aquí coincide con el máximo de cada serie válida. El fallo503tardó10269,191ms y queda fuera de esas latencias válidas; su importe es desconocido. Incluyéndolo, el máximo/p95de16intentos del control es10269,191ms, sin que eso pruebe rapidez de la candidata. El reporte conserva todos los intentos y bloquea ambas puertas por incompletitud; además persisten3extras y la mediana española es1,640×la base frente al límite1,5×. El cociente de costes conocidos1,008×excluye el cargo desconocido y no permite declarar pase. No se midió `recall`, estabilidad, campañas completas ni casos nuevos.

## Coste, verificación y decisión

Tanda:63operaciones/63HTTPintentados, USD0,003521920conocidos +1cargo desconocido (16mínimos+47difíciles). Las63operaciones incluyen el503; solo62tienen respuesta válida. Suma todos los brazos, no solo la candidata. Acumulado: **5755operaciones,23campañas completas,USD0,577250278conocidos +5operaciones históricas o actuales de importe desconocido**. Codex de programación/revisión no está incluido.

265pruebas pasan con y sin HTTP/2; formato y compilación pasan. Se reconstruyen18informes y la auditoría de frames con dimensiones; permanecen íntegros bancos, resultados originales y snapshots001–042. Se preservan todos los hashes declarados de V6 sin leer semánticamente ni llamar al proveedor. No cambia README/gráficas porque no hay nueva ruta validada.

No se adopta la traducción ni se encadenan variantes de idioma. El siguiente trabajo deberá atacar alcance/estado con contrastes mínimos que diferencien una pieza de una regla global y preparación de ejecución; exigirá fuentes, revisión separada y protocolo previo antes de otra llamada. No basta otro cambio de redacción ni repetir esta tanda hasta que salga favorable. Siguen pendientes un mecanismo elegible, estabilidad, integración y casos nuevos ciegos.

[Protocolo mínimo](instruction-language-es-v1-protocol-2026-10-20.md), [contraste difícil](instruction-language-es-known18-v1-protocol-2026-10-20.md). Artefactos: `outputs/iromem/evaluation/contribution-local-perplexity-spanish-{minimal-v1,known18-triple-v1}-2026-10-20`. Contabilidad y huellas: `quality-goal-2026-10-19/snapshots/043`.
