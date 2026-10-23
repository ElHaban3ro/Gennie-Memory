# V3: correcciones operativas y vocabulario completo aceptado

El vocabulario domain3 ya supera la validación global real: se retiraron los cuatro tipos sin citas de clase admisibles, sin inventar equivalencias. Se corrigieron cotas, granularidad temporal y aplicación de contexto en el mecanismo local. **La traducción completa del banco a frames ejecutables y la calidad automática siguen pendientes.** Goal activo en el paso 3; baseline y V6 conservadas.

[Informe estructurado](../../outputs/iromem/evaluation/documentary-v3-domain-preflight-2026-10-21/report.json), [procedencia](../../outputs/iromem/evaluation/documentary-v3-domain-preflight-2026-10-21/execution-provenance.json), [controles CLI finales](../../outputs/iromem/evaluation/documentary-v3-domain-preflight-2026-10-21/cli-verification-corrected.json) y [revisión del inventario](../../outputs/iromem/evaluation/documentary-v3-domain-preflight-2026-10-21/inventory-review.json). Los fallos, raws y críticas anteriores permanecen intactos.

## Comportamiento corregido

Una cantidad con cota superior/inferior ya no completa una solicitud de valor exacto: conserva la cantidad y su cota como parcial. Un máximo explícito exacto sí puede completar el control correspondiente. Esto no implementa álgebra general de intervalos ni compatibilidad arbitraria entre atributos y unidades.

Los slots `occurred_year`, `planned_year`, `scheduled_year` y `holds_year` permiten pedir explícitamente un año. Exigen término temporal `format:year`, cuatro cifras ASCII y una cita exacta de esas cifras. No convierten fechas ni expresiones relativas a años. Los slots anteriores conservan su significado de tiempo general: **el raw antiguo `occurred_at` de una pregunta por año sigue sin expresar esa restricción y no se repara automáticamente**. La extracción futura debe producir el slot correcto; no se ha medido todavía.

Varias acciones temporales compatibles mantienen pendiente la identidad de ocurrencia, incluso si una tiene año y otra solo fecha relativa. El contador distingue conclusiones de sus citas de procedencia y agrupa las ramas cubiertas por un invariante demostrado; no convierte una regla con varias citas o alternativas equivalentes en varias ejecuciones.

Una configuración demostrablemente distinta se excluye también en rutas de relevancia, eventos y reglas con condiciones pendientes. Los enlaces de relevancia conservan desconocimiento como candidato pendiente, sin contarlo como utilidad seleccionada. Se comprueban ámbito normativo, fundamento, dependencias, ramas y premisas. Esto requiere aún la puerta completa de conservación de utilidad: ser más conservador no demuestra mejora general.

## Evidencia y verificación

| Comprobación | Resultado y límite |
| --- | --- |
| Workspace Rust | 565 pruebas pasan, 24 omitidas, cero fallos; 45 pruebas V3 totales, tres nuevas en esta tanda |
| Controles manuales por CLI | 15 conductas esperadas comprobadas; incluye un rechazo intencional de año mal formado, no son 15 extracciones fieles |
| Vocabulario domain3 completo | Aceptado en ambos lados de un par manual de propiedad; respuesta no vacua con cita |
| Crítica operativa | Ocho controles iniciales conservados; fixes y tres regresiones revisadas/corregidas, con pruebas de ocurrencias e incompatibilidad de contexto |
| Historia | Cuatro bancos manuales/puntuaciones, diez informes de extracción, seis semánticos, tres auditores y cuatro informes Codex reproducidos byte a byte |
| Reserva y producto | Sin abrir V6, llamadas remotas, cambios de baseline, SQL ni servicios |

No se repitió HTTP/2. El binario CLI contiene el código operativo actual; el último cambio posterior a su compilación solo corrigió el texto de un test. Suite final, hashes, controles y críticas se vinculan en la procedencia. No se midieron latencia de producción, recursos, estabilidad de modelo ni recall.

La crítica revisó también los controles derivados: corrigió la redacción de cota inferior, la pregunta temporal y el fundamento observado; aisló la ablación del enlace de relevancia y explicitó registros en los contextos. Un offset inicialmente contado por caracteres en vez de bytes UTF-8 fue rechazado correctamente por el validador y corregido sin cambiar expectativas. Se conservan las versiones iniciales y los diagnósticos. Esta revisión conoce las expectativas y no se presenta como evaluación ciega de extracción.

## Banco completo: avance y lagunas reales

La matriz cubre 116 entradas —83 fuentes y 33 preguntas—, 135 unidades y 701 citas por bytes UTF-8, verificadas por un segundo agente. No usa gold para añadir argumentos o clases. Domain3 añade dos aridades para conservar beneficiario y propósito expresos sin inventar actor ni posesión; aún faltan sus controles ejecutables.

La matriz lexical **no es un conjunto de DTO validados**. Hay 24 filas con lagunas anotadas y ocho unidades de preguntas cuyos roles lexicales todavía no corresponden a los roles operativos. Esas cifras pueden solaparse; no se suman como errores independientes. Faltan especialmente traducción de cantidades asociadas a modos, compatibilidad atributo/unidad, clases ausentes, utilidad neutral y exenciones sin regla identificada. Las notas de operaciones propuestas no se ejecutan por tener hash.

La siguiente puerta es traducir todas las unidades a representaciones tipadas, resolver los roles sin rellenar actores/sustancias/documentos inventados y comprobar las operaciones pendientes. Solo después corresponde repetir extracción completa con revisión, utilidad en conocidos126, baseline emparejada, recursos/estabilidad y casos nuevos ciegos. La validez global del vocabulario por sí sola no abre esas puertas.

## Calidad y coste

No hay medición automática nueva. La última permanece en **102/116 DTO válidos (87,93%) y 93/116 fieles en las cuatro dimensiones para ambos críticos originales (80,17%)**; la puerta sigue fallida. Sus 928 juicios y 29 desacuerdos se preservan.

Esta tanda añade **cero llamadas a proveedores y USD 0 de API**. Coste y tokens Codex desconocidos. Acumulado sin cambios: 6129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos y 20 operaciones con coste desconocido; Codex excluido. No se afirma ahorro ni promoción.
