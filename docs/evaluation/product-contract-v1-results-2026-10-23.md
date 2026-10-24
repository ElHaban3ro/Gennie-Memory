# Cierre del contrato de producto v1 y validación sintética

Fecha: 2026-10-23 (America/Bogota).

## Decisión y alcance

Se cierra la implementación inicial del contrato optativo v1 y su protocolo sintético de 18 casos. El perfil Decisions + per_memory_v1 superó desarrollo, calibración y final sin ajustes entre etapas. La calibración comprobó la receta congelada; no se ajustó ningún umbral ni se equipararon puntuaciones. Las tres particiones y etiquetas originales conservaron sus hashes. La prueba final está consumida: cualquier cambio de política requiere otra reserva.

Esta aprobación cubre el contrato local, fidelidad estructural y el perfil en este banco pequeño. No promueve per_memory_v1 como predeterminado ni demuestra calidad poblacional o respuesta final de Codex. El baseline predeterminado conserva un límite observado descrito abajo. Revisión humana y verificación externa de fuentes siguen fuera de la evidencia obtenida.

## Resultados reales

| Partición | Resultados esperados | Evidencias útiles | Ajenas | Degradaciones | p50 ms | p95 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| development | 6/6 | 3/3 | 0 | 0 | 638.22 | 1111.72 |
| calibration | 6/6 | 3/3 | 0 | 0 | 460.70 | 870.28 |
| final | 6/6 | 3/3 | 0 | 0 | 460.73 | 983.25 |

Cada partición contiene seis consultas y solo tres evidencias útiles: apoyo, refutación y dato parcial. Los otros controles son abstención, aclaración y búsqueda vacía. Nueve evidencias útiles en total; las nueve relaciones y semillas coincidieron con las anotadas. No extrapolar estos denominadores a los 540 casos del benchmark histórico. p95 usa rango más cercano sobre seis observaciones por partición y corresponde al recorrido de producto, sin integración Codex.

La campaña hizo **12 llamadas** con recibos completos: **USD 0,0006135**, bajo los límites registrados de 12 llamadas, 300 segundos y USD 0,10. Coste por etapa: desarrollo USD 0,0002087000; calibración USD 0,0002036000; final USD 0,0002012000. La duración de campaña incluye carga, preflights de búsqueda local y control del runner; sus milisegundos exactos están en summary.json. Los tiempos de cada caso incluyen segmentación, búsqueda, selección y composición del producto. Los subagentes de revisión no tienen coste USD medido.

El diario se sincroniza antes de cada solicitud y después de cada respuesta. El runner exige directorio nuevo, liga suite/política/binario/revisiones/precio por hash, reserva presupuesto por operación, limita llamadas/tiempo y se detiene sin reintento ante consumo desconocido o puerta fallida. La reserva es una salvaguarda contable local; no obliga al proveedor a respetar un precio máximo por petición. Casos sin candidatos o con aclaración no consumen llamadas. Una búsqueda local previa determina si reservar, sin consultar gold.

La auditoría offline posterior reconstruyó las tres puertas desde el diario: secuencia de inicios/terminales, solicitudes congeladas, IDs/revisiones/ámbitos, metadatos y **texto exacto de la semilla**, recibos y coste. No efectuó llamadas nuevas. El producto no cambió después de abrir final; la ampliación del auditor comprueba fidelidad del texto emitido con el formato existente.

## Proveedores: documentación y comprobación real

Las fichas oficiales publican [Decisions](https://openrouter.ai/openai/gpt-6-luna-decisions/providers) a USD 0,10/M entrada y cero salida; [Jev 1.13](https://openrouter.ai/typesafe/jev-1.13/providers) y [Jev 1.14](https://openrouter.ai/typesafe/jev-1.14/providers) a USD 0,042/M y cero salida. Datos consultados el 2026-10-23. Decisions publica contexto de 1.050.000 tokens y hasta 200 preguntas; Jev publica 262K de contexto. Son límites documentados, no ensayos de saturación. El catálogo estándar /api/v1/models no incluyó estos modelos de Decisions en esta consulta; se usaron sus fichas oficiales y las respuestas reales. La consulta de solo lectura de la credencial fue aceptada y confirmó presupuesto suficiente; no se guardó la clave ni el saldo completo.

Se comprobó compatibilidad de las nueve combinaciones de tres modelos y tres políticas (baseline, per_memory_v1, per_memory_v2), usando la operación de la campaña para Decisions/per_memory_v1 y ocho llamadas adicionales sobre un caso de desarrollo. Todas devolvieron contrato válido y recibo completo. Modelos servidos observados: openai/gpt-6-luna-decisions-20261006, typesafe/jev-1.13-20260917 y typesafe/jev-1.14-20261014.

**Límite observado:** baseline se abstuvo con los tres modelos en ese caso de desarrollo que incluye contexto y evidencia útil. Las variantes por recuerdo seleccionaron la evidencia. No se ha demostrado la causa de la abstención ni se corrige con una etiqueta inventada. El primer intento se detuvo y quedó preservado como discrepancia semántica; las siete combinaciones restantes se midieron aparte sin repetir esa petición. Compatibilidad técnica no acredita acierto semántico. No se adopta ningún nuevo predeterminado con estos smokes.

Las ocho llamadas adicionales costaron USD 0,000253136, con protocolo separado de hasta ocho llamadas y USD 0,02. **Total API de esta continuación: 20 llamadas y USD 0,000866636.** Las comprobaciones públicas de catálogo, credencial y auditorías locales no fueron inferencias.

## Evidencia reproducible

- [Campaña y diario](../../outputs/iromem/evaluation/product-contract-v1-20261024T034011Z/summary.json).
- [Auditoría exacta](../../outputs/iromem/evaluation/product-contract-v1-20261024T034011Z/audit.json).
- [Primer smoke baseline conservado](../../outputs/iromem/evaluation/product-provider-compatibility-20261024T034105Z/summary.json).
- [Siete combinaciones restantes](../../outputs/iromem/evaluation/product-provider-remaining-20261024T034151Z/summary.json).
- [Proveedor/precio y presupuesto registrados](../../tests/fixtures/product-contract-v1/provider-verification.json).

Desde la raíz:

```sh
cargo run --offline -p gennie-memory-eval -- audit-product-campaign tests/fixtures/product-contract-v1/suite.json outputs/iromem/evaluation/product-contract-v1-20261024T034011Z/bin/iromem outputs/iromem/evaluation/product-contract-v1-20261024T034011Z
```

El auditor exige el mismo hash del binario utilizado. Se conservó el binario exacto en outputs/iromem/evaluation/product-contract-v1-20261024T034011Z/bin/iromem antes de retirar el dashboard. Una compilación distinta no es identidad binaria comprobada.

## Pendientes identificados

- La omisión semántica de baseline tiene [diagnóstico posterior](baseline-context-diagnosis-2026-10-23.md): cautela del contexto y presentación de recuerdos, con corrección candidata ensayada. Falta implementar y validar el arreglo; no se cambiaron defaults.
- Dashboard retirado posteriormente por decisión del usuario: no queda integración pendiente. Se conservan datos y checksums históricos; [retirada](../decisions/0047-remove-dashboard.md).
- Revisión humana aplazada por el usuario. Verificación externa de fuentes e integración Codex permanecen en sus hitos posteriores.

Verificación de esta continuación: `cargo test --offline -p gennie-memory-eval` terminó con **709 pruebas correctas, cero fallos y cero ignoradas**. Se conservaron advertencias preexistentes de símbolos no usados. Las pruebas nuevas rechazan roles incorrectos, evidencia ajena, degradaciones presentadas como abstención válida, exceso de p95 y texto de semilla alterado con metadatos idénticos. El ensayo CLI sin credencial registró seis respuestas y cuatro degradaciones, detuvo avance antes de calibración/final, rechazó reutilizar el directorio y no llamó al proveedor. Compilación offline, `cargo fmt --all --check` y `git diff --check` pasaron.

Esta continuación cambió herramientas de evaluación y documentación, sin modificar el producto ni su transporte después de abrir la reserva; por eso se verificó el crate evaluador completo. Las suites workspace/HTTP2 anteriores del contrato permanecen en su registro original. No se iniciaron servicios con puertos ni se detuvieron procesos ajenos. Los procesos de sesión propios se cerraron al terminar. Las bases y diarios de campaña son artefactos sintéticos propios conservados para auditoría; no se usó la base de desarrollo. Sin commit ni push.
