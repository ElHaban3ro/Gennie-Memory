# Alcance de cantidades comunes universales en V3

El comparador conservaba el plazo de un archivo individual como dato parcial para una pregunta por el plazo común de todos los archivos. Ahora exige una prueba de alcance para publicar ese dato: el cuantificador debe corresponder al dominio solicitado y el operando real del dato debe estar ligado a la variable correspondiente. Un cuantificador alrededor de una afirmación que sigue hablando de B3 no demuestra un plazo para todos.

La comprobación se aplica a las vías positivas de dato `answer`, `event_slot` y `constraint` cuando la petición declara el operador existente `quantity_extent:común`. Una lista de valores individuales conserva sus datos parciales; no requiere un valor común. Las vías de corrección de premisas y contexto conservan su tratamiento separado. El mecanismo trabaja sobre frames validados y declaraciones explícitas; no interpreta directamente el texto natural.

La ausencia de esa prueba deja un candidato pendiente con motivo `common_quantity_scope`, sin valor publicado. No declara falsedad, diferencia entre referentes ni contradicción. El texto fuente y sus dieciocho días permanecen intactos. No implementa un demostrador general de cobertura por enumeración, eliminación de cuantificadores adicionales o contraejemplos universales.

## Comparación emparejada conocida

Los mismos 100 pares manuales, vocabulario, frames y referencias heredadas V5 se ejecutaron con ambos binarios. No se cambiaron entradas ni etiquetas para aprobar.

| Medida | Anterior | Nuevo |
| --- | ---: | ---: |
| Evidencias útiles seleccionadas /39 no ambiguas | 28 | 28 |
| Referencias `none` seleccionadas /60 | 1 | 0 |
| Respuestas completas | 3 | 3 |
| Concordancia con selección heredada /100 | 87 | 88 |

El único cambio de selección es `pcv1_01_02/claim_01`. Se conservan las 28 útiles y las tres respuestas completas, sin nuevas selecciones `none`. La concordancia incluye una referencia ambigua no seleccionada: no es exactitud del producto. Persisten 11 evidencias útiles no ambiguas omitidas (8A, 2B, 1C) y una ambigua separada. La puerta global sigue fallando; Duna conserva sus calificadores pendientes.

## Controles y verificación

Ocho controles manuales prospectivos separan individuo→todos, todos→todos, individuo→individuo, todos→individuo parcial, dominio universal distinto, existencial→todos, lista individual sin operador común y cuantificador con operando constante. La versión nueva cumple los ocho; la anterior falla los cuatro negativos. Además, una prueba de rutas comprueba proposición, evento y restricción con documentos completos y candidato pendiente, evitando un rechazo vacuo por documento inválido.

Se conservan los intentos fallidos: el caso individual inicialmente seleccionado; un fixture de dominio sin clase/cardinalidad que se corrigió como fixture; un error de colocación de una aserción que impedía compilar; y el contraejemplo válido con cuantificador alrededor de un operando constante. Este último motivó exigir el vínculo del operando, además del ámbito. Ningún fallo se elimina del registro ni se presenta como prueba superada por la versión anterior.

El hash final pasa **603 pruebas, 24 omitidas; 83 V3**, compilación y formato. Se reprodujeron **224 informes idénticos**: los 100 pares candidatos, ocho controles y 116 validaciones históricas de extracción. La auditoría del caso rechazado verifica que no publica support/refutation/partial ni valor, conserva la fuente y devuelve el motivo de alcance pendiente.

Estos resultados no miden extracción nueva, latencia, coste de producción, estabilidad, baseline ni generalización. La última extracción automática sigue en **65/116 fieles para ambos críticos**, con puerta fallida. No hubo nueva revisión semántica independiente ni reactivación de subagentes; V6 permanece sin abrir. El siguiente trabajo son las 11 omisiones útiles y la validación automática completa antes de casos nuevos o adopción.

**Coste nuevo: 0 peticiones API, USD 0 de proveedor.** Coste/tokens Codex desconocidos. El acumulado no cambia: 6129 operaciones, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos; excluye Codex. El objetivo continúa activo en el paso 3; no cambia la política del producto.

[Protocolo](../../outputs/iromem/evaluation/documentary-v3-universal-scope-2026-10-21/protocol.json), [comparación](../../outputs/iromem/evaluation/documentary-v3-universal-scope-2026-10-21/report.json), [procedencia ejecutada](../../outputs/iromem/evaluation/documentary-v3-universal-scope-2026-10-21/execution-provenance.json) y [controles](../../tests/fixtures/documentary-v3-universal-scope-controls-v1.json). Los scripts, propuestas y logs fallidos/finales se conservan junto al informe JSON.
