# Extracción consolidada de 116 textos — 2026-10-22

Tanda original congelada en snapshot 108. Las [cuatro correcciones conocidas posteriores](documentary-v3-author-corrections-results-2026-10-22.md) se informan por separado y no sustituyen sus métricas.

La extracción de desarrollo terminó: **116/116 textos entregados**. El primer intento tiene 100 completos estructuralmente; una reparación mecánica de IDs eleva la cifra a **102/116**. La doble revisión semántica está cerrada: **100/116 aceptadas por ambos críticos**, de las que **99/116 son además estructuralmente completas**. Quedan 17 entradas que incumplen al menos una condición. La puerta de 116 completos y fieles sigue fallando; no se ejecuta utilidad ni se atribuye una nueva mejora a recall.

La medición anterior de 31/39 útiles, cero ajenos/60 y ninguna pérdida procede de cuatro documentos previamente revisados en la consolidación conocida. Sigue separada de esta nueva extracción íntegra y no es una métrica de la tubería autónoma de esta tanda.

## Métricas y comparación

| Métrica, denominador fijo 116 | Resultado |
| --- | ---: |
| Entregas recibidas | 116 |
| Completas en el primer intento | 100 |
| Completas tras reparación mecánica | 102 |
| Parciales / inválidas | 10 / 4 |
| Aceptadas por ambos críticos | 100 |
| Completas y aceptadas por ambos | 99 |
| Puerta completa superada | No |

Los veinte dictámenes cubren 928 dimensiones: 882 faithful, 36 defective y 10 uncertain. Hay 12 desacuerdos de dimensión, conservados sin reemplazarlos por consenso favorable. Una entrada parcial recibe aceptación de ambos críticos, pero sigue fuera del pase combinado. La revisión es ciega a contrapartes y resultados, con contextos separados de la misma familia Codex; no es revisión humana o independencia de familia.

La unión exacta por tipo y SHA256 verifica los mismos 116 textos que en la [tanda neutral anterior](documentary-v3-neutral-extraction-results-2026-10-21.md): **115→102 completas y 112→99 completas y aceptadas**. Se conservan 96 aceptaciones, se pierden 16, se ganan tres y una sigue sin aceptación. Es una regresión observada entre tandas con guía, vocabulario y revisores distintos; no demuestra una caída causal del modelo o de recall. No se adopta esta candidata. Los originales, etiquetas y juicios anteriores permanecen intactos.

## Protocolo y ejecución

Se usaron los diez paquetes congelados de `consolidated-extraction-contract-v1/candidate-v2`: 83 fuentes y 33 preguntas, sin cambiar textos, IDs o vocabulario. Diez autores Codex nuevos recibieron solo su paquete fuente o pregunta. No recibieron etiquetas, contrapartes, roster, críticas ni resultados previos. Cada ítem conserva un único intento semántico. Los críticos trabajan en contextos nuevos separados por paquete y tipo, con las cuatro dimensiones congeladas.

La recepción detectó una incompatibilidad de metadatos antes de leer los raws: el ensamblador solo admitía el campo histórico `validator_contract_source_without_tests`, mientras los nuevos paquetes contienen `author_guide`. Se añadieron dos formas cerradas alternativas, sin aceptar campos ajenos o mezcla de perfiles, conservando el hash de todos los bytes. No cambió el clasificador ni las representaciones.

En dos ítems de source-04, un claim y una referencia compartían ID `measurement`. La reparación acotada cambia solo el ID tipado del claim y sus enlaces por `measurement_claim`. El renombrado inverso recupera exactamente el JSON original; cuerpos, citas, estados, scopes y qualifiers son idénticos. Primer intento, derivado, recibos y prueba de equivalencia permanecen guardados. Los errores de argumento obligatorio o alcance no se reparan mecánicamente.

## Corrección conocida y siguiente trabajo

El diagnóstico de las tres preguntas distingue un error de autor (Salvia) y dos brechas del contrato: finalidad de la obligación respecto a la matrícula, y magnitud condicionada durante una ruta. No se resuelven creando actores, una operación ocurrida, equivalencias de magnitudes o asociaciones más fuertes.

El diagnóstico separado cubre las 17 entradas: seis causas primarias de autor, cinco brechas de contrato, cinco ambigüedades genuinas y una sin resolver. Es clasificación retrospectiva, con causas secundarias y propuestas pendientes de crítica; no sustituye los veinte dictámenes. Algunas frases no dan información para cerrar un sentido o contexto. Exigir resolución completa puede forzar una afirmación inventada: antes de otra tanda hay que diseñar y criticar prospectivamente la distinción entre incertidumbre fiel y pérdida semántica, sin recalcular esta puerta ni relajar filtros de respuesta o utilidad.

Se comprobó una corrección conocida de Salvia en un artefacto separado: `lights.binding_scope_id=null`, manteniendo `scope_id=six_pairs`, variable contada `pairs`, cardinalidad y asociaciones intactas. La CLI pasa de invalid a complete, con un único campo cambiado. No modifica la tanda, sus críticos o porcentajes; falta nueva revisión de fidelidad y no se ejecutó utilidad.

## Verificación y coste

Pasan 730 pruebas del workspace, cero fallidas y 24 ignoradas; las 35 focales de IO y el chequeo de formato pasan. No se arrancaron servicios, no se cambió base de datos, configuración de producto, dependencias o credenciales.

Se usaron diez contextos de autor y veinte críticos nuevos, más dos contextos de diagnóstico retrospectivo. Desde el registro de despacho de la extracción hasta el cierre de su fidelidad transcurrieron aproximadamente 28 minutos; no incluye terminar el diagnóstico ni es latencia de producción.

**Cero peticiones externas y USD 0 externos nuevos.** Coste y tokens facturables Codex desconocidos. El tiempo de orquestación incluye extracción y crítica, y no mide latencia de producción. Acumulado externo anterior: 6.129 operaciones, 23 campañas, USD 4,565416508 conocidos y 20 importes desconocidos, sin Codex.

M5/goal siguen activos en paso 3, con revisión retroactiva del contrato. Sin adopción ni apertura de V6. La siguiente corrección se centra en las 17 entradas fallidas: errores de autor frente a capacidades neutrales faltantes, con diagnóstico fuente/pregunta separado. No corresponde repetir a ciegas todo el banco ni ejecutar utilidad antes de superar fidelidad. El diagnóstico y cualquier corrección conocida tendrán versiones explícitas y no se contarán como mejora medida por sí solos.

## Evidencia

Directorio: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-source-extraction-v1/`.

- `dispatch.json`, `author-runs.json` y `review-runs.json`: despacho y aislamiento.
- `receipt-summary.json`: originales, hashes, recibos y estados estructurales.
- `source-04-packet/mechanical-repair.json`: renombrado reversible y originales conservados.
- `fidelity-summary.json`, `final-report.json`: veinte revisiones cerradas, cuatro dimensiones, puerta fallida.
- `previous-full116-comparison.json`: unión íntegra con la tanda anterior y todas sus pérdidas/ganancias.
- `../consolidated-failure-diagnosis-v1/`: diagnóstico retrospectivo separado, sin calidad medida.
- `packet-receiver-fix-verification.json`, `workspace-tests.log`: pruebas de la corrección de recepción.
