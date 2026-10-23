# Catálogo de referencias V1: conserva objetos, no supera suficiencia

UTC 2026-10-21. **La sustitución exacta de referencias funciona, pero no resuelve dos límites del contrato vigente. No se prepara ni ejecuta el extractor de referencias.** Goal activo en paso 3, sin promoción. Cero llamadas y coste nuevo de proveedor **USD 0**. [Protocolo](reference-catalog-v1-protocol-2026-10-21.md).

## Qué se implementó

Un prototipo puro de catálogo permite que las cláusulas autor-manuales usen un ID local en cada subject/reference. El materializador copia exactamente el objeto guardado y rechaza IDs desconocidos/duplicados, referencias inline editadas y markers fuera de slots. Conserva nombre, clase, propietarios, scope, cardinalidad y enlaces tal como fueron suministrados. Los IDs son locales a cada fuente o pregunta: no prueban correferencia. Referencias no usadas se muestran sin exigir hechos inventados sobre cada propietario.

El prototipo no es un extractor ni una tubería automática. Puede copiar perfectamente un catálogo incorrecto o un ID permitido elegido para el portador equivocado. Tampoco valida el predicado, supuestos o completitud de cláusulas. La matriz separa materialización, traducción grounded V1, normalización tipada y proyección V5; conserva cada etapa incluso si falla la siguiente. No se cambian el traductor, parser ni resolver históricos.

## Matriz completa conservada

Ocho controles autor-manuales LA01–08, con ambos lados intentados y todas las condiciones necesarias declaradas antes del informe. [Fixture](../../tests/fixtures/reference-catalog-controls-v1.json), [informe final](../../outputs/iromem/evaluation/reference-catalog-v1-2026-10-21/matrix-final.json).

| Control | Resultado del contrato original |
| --- | --- |
| LA01 — propietario distribuido entre frases | La copia conserva todos los campos, pero el traductor rechaza el propietario que no aparece dentro de la mención del formulario. |
| LA02 — adjetivo como premisa corregible | Identidad y supuesto se conservan; selecciona el color fuente mediante A. Existe discrepancia sobre calificación/corrección explícita, descrita abajo. |
| LA03 — propietarios en otra formulación | Selecciona A; la revisión cuestiona que «para el pedido» equivalga siempre a la misma cadena de pertenencia. |
| LA04 — universal plural y miembro singular | Ambos lados se normalizan; sensores y sensor generan clases distintas, por lo que pierde evidencia útil y devuelve none. |
| LA05 — homónimo con propietario distinto | Conserva X/Y y devuelve none. |
| LA06 — componente aditivo | Conserva el dato parcial y exige calificación, sin publicar una masa exacta del equipo completo. La formulación manual tiene ambigüedad de vínculo/basis. |
| LA07 — depósito intermedio frente a red | Conserva evidencia parcial con calificación; no inventa una red ni el enlace aguas arriba. La revisión cuestiona parte de la interpretación de dirección. |
| LA08 — La Paz frente a Paz | Mantiene los nombres literales distintos y devuelve none. |

Se cumplen **6/8 condiciones mecánicas necesarias**, no un 75% de calidad ni seis extracciones correctas. Los hechos, catálogos y preguntas fueron escritos manualmente. Un control falla por contención de evidencia y otro por compatibilidad de clase, no por pérdida durante la copia. La puerta local íntegra falla.

LA01 no demuestra imposibilidad sintáctica absoluta: el parser acepta una mención artificialmente ampliada a todo el texto. Eso no certifica un sintagma nominal fiel y no se utilizó para aprobar. LA04 puede representar fielmente ambas clases literales y aun así fallar matching; preservar palabras no conserva por sí solo utilidad.

## Revisión independiente y discrepancias

Un crítico nuevo sin historial recibió los ocho textos y construcciones, sin condiciones esperadas ni resultados. Considera cinco representaciones fieles y tres inciertas: LA03 por finalidad/pertenencia, LA06 por componente físico y procedencia observacional no explícitos, LA07 por inmediatez del flujo y camino interpretado. No se corrigieron las construcciones después de esa revisión ni se recalculó un porcentaje favorable. Son controles manuales en desarrollo, no una cohorte de extracción ciega validada. El crítico estático separado tiene historial y no se presenta como ciego. Ambos son agentes de la misma familia, no humanos. [Conciliación sin alterar etiquetas](../../outputs/iromem/evaluation/reference-catalog-v1-2026-10-21/manual-review-reconciliation.json).

En LA02, el crítico pide corregir/calificar expresamente «roja», mientras el motor devuelve A sin calificación. El código no declara incompatibles dos valores positivos de propiedad: refuta negación explícita del mismo valor o valores excluidos por un conjunto exhaustivo. «Azul» no trae automáticamente una regla de color único. Se registra esta diferencia entre lectura natural y contrato sin añadir una regla de exclusividad ni anunciar como refutación probada lo que el motor no demuestra. La referencia fija y la premisa se conservan, pero la condición de selección usada en esta matriz no cubre toda esa discrepancia.

## Verificación y errores del prototipo

El materializador tiene cinco pruebas nuevas; la matriz añade una que conserva expresamente los dos fallos necesarios. Las comprobaciones cubren valores/premisas/lecturas, namespaces separados, referencias añadidas/editadas, entradas no usadas y el caso donde conservar un objeto no valida el ID o predicado escogido. Pasan 468 pruebas por suite con y sin HTTP/2, cuatro bancos manuales y diez informes históricos idénticos, además del auditor previo y la matriz final reproducibles byte por byte. Formato y diff-check pasan. El aviso histórico de método compartido sin uso en bootstrap test permanece sin impedir compilación. Los detalles constan en la [verificación](../../outputs/iromem/evaluation/reference-catalog-v1-2026-10-21/verification.json).

Se conservan el fallo inicial de compilación por un helper de escritura inexistente y el fallo inicial de fixture por metadatos obligatorios de ontología. Antes de la revisión ciega se corrigió la ubicación de extent, se hizo explícita la membresía/clase en su propio pasaje, se citaron los nombres La Paz/Paz sin una clase genérica añadida y se completaron campos de ontología exigidos. Ninguna condición necesaria ni gold histórico cambió; borradores iniciales, logs y hashes quedan preservados. La crítica del código detectó que la primera matriz descartaba la materialización al fallar el traductor; el informe final conserva las etapas y liga el fixture por SHA-256. El primer informe permanece separado. [Procedencia](../../outputs/iromem/evaluation/reference-catalog-v1-2026-10-21/fixture-provenance.json), [crítica final](../../outputs/iromem/evaluation/reference-catalog-v1-2026-10-21/code-critic-final.json).

## Siguiente corrección requerida

El siguiente cambio debe versionar la evidencia de referencias: frase nominal propia y vínculo de propietario con cita separada, preservando portador, alcance temporal/condicional y distinción fijo/supuesto. Esto permitiría representar evidencia distribuida sin ensanchar artificialmente la mención. No resuelve correferencia ni singular/plural: la compatibilidad de clase necesita una hipótesis semántica explícita y controles aparte, sin cortar palabras, borrar clases o introducir aliases desde gold. Las tres construcciones manuales inciertas deben aclararse en una nueva versión prospectiva, conservando esta.

No tiene sentido ejecutar ahora dos llamadas por caso sobre un contrato que falla localmente. Aunque una versión posterior pase suficiencia, tendrá que probar extracción completa, alineación, banco difícil, baseline, coste/p50/p95 de todas las etapas, estabilidad y solo entonces V6 ciega. La conservación exacta es un requisito, no una redefinición del objetivo de mejorar calidad.

Sin llamadas ni cargos nuevos: acumulado **6001 operaciones, 23 campañas emparejadas completas, USD 0,845298508 conocidos más 16 operaciones de coste desconocido**, sin Codex. Las métricas automáticas siguen siendo las del piloto histórico, sin mejora nueva. No se mide latencia de recall/end-to-end ni pase de recursos. V6, etiquetas, defaults, README/gráficas, dependencias, SQL, dashboard y servicios se conservan. Goal activo.
