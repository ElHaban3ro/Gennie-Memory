# Proyección de afirmaciones: resultado de desarrollo — 2026-10-19

La extracción conjunta de pregunta y recuerdo **no supera calidad y queda descartada como candidata de adopción**. El censo completo encuentra 24 extracciones inválidas, tres aceptaciones incorrectas y siete omisiones entre las extracciones válidas. Once fallos de extracción afectan además a recuerdos útiles: de los 22 útiles presentes en el prefijo, solo cuatro obtienen aceptación válida. No se presenta un porcentaje calculado únicamente sobre los frames válidos.

## Qué se implementó

Se prototiparon contratos puros de evaluación con identidad, partes solicitadas y premisas de pregunta, y afirmaciones del recuerdo con sujeto, propiedad, polaridad, negación y destino. El primer formato exigía citas literales; los siguientes usan referencias al texto original. La versión final restringe cada referencia por un enum dinámico a los IDs reales de su fuente: qN para pregunta, mN para recuerdo. Los grupos internos se definen como conjuntos; los rangos y el orden de los grupos enlazados se conservan.

El programa reconstruye las citas desde offsets UTF8 originales. Rechaza referencias inventadas, de otra fuente, duplicadas, rangos inválidos y destinos inexistentes. Esas garantías de formato no establecen correferencia, completitud, alcance semántico o verdad de la relación. El juicio de mismo caso y utilidad sigue siendo del extractor. No hay una nueva política runtime ni cambios de SQL, dependencias, secretos o puertos. Bootstrap compila los comprobadores de herramientas únicamente bajo `cfg(test)` para runners manuales ignorados.

Se usó GPT‑4.1 mini por OpenRouter, con esquema estricto, temperatura cero, sin fallback y máximo 1400 tokens de salida. Se conservaron el modelo/proveedor servidos, respuestas y recibos. El alias no prueba una revisión interna inmutable. Cambian modelo y representación frente a los deciders: no se atribuye el resultado exclusivamente a la estructura.

## Intentos y censo completo

| Ejecución | Operaciones / HTTP | Extracciones inválidas | Coste USD | Resultado |
| --- | ---: | ---: | ---: | --- |
| Citas v1 | 4 | 3 | 0,002404000 | Reformuló partes de la pregunta en campos de cita literal. |
| Índices numéricos v2 | 4 | 1 | 0,002617600 | Señaló un índice inexistente de pregunta. |
| IDs restringidos v3 | 4 | 1 | 0,003199200 | Grupo válido de referencias rechazado por su orden. |
| Conjuntos de IDs v4 | 8 | 2 | 0,007096800 | Alcance de negación sin predicado y sujeto fuera de afirmación. |
| Censo completo del contrato v4 | 128 | 24 | 0,107496000 | Completo en transporte; falla calidad y extracción. |
| **Total facturado de esta fase** | **148** | — | **0,122813600** | Todas las operaciones tienen recibo. |

Las cuatro primeras ejecuciones se detuvieron según su protocolo; no se repararon ni se presentan como completas. El [censo posterior](assertion-projection-census-protocol-2026-10-19.md) fijó antes de red continuar ante errores de extracción para medir su frecuencia en los 128 pares, manteniendo el rechazo de frames inválidos y la puerta de cero errores. Se detendría ante transporte/HTTP/modelo incorrectos. No hubo tales fallos: completó en 68,25 segundos, con cuatro llamadas simultáneas como máximo. No hubo reintentos ni elección de la mejor repetición por caso. Son variantes y ejecuciones sobre desarrollo conocido, no casos nuevos.

El censo mantuvo exactamente el payload/contrato v4 y sus enums, sobre 126 candidatos de 18 preguntas conocidas más dos controles de autor. Las peticiones se congelaron antes de red con todas las dependencias transitivas de checker y fixtures. El auditor reconstruyó la población desde pilotos CLI y gold originales, verificó cada lote y cargo; comparte el comprobador puro de integridad con el runner, sin constituir verificación semántica independiente.

## Resultado del censo

| Resultado por candidato | Cantidad |
| --- | ---: |
| Útil aceptado con frame válido | 4 |
| No útil rechazado con frame válido | 88 |
| No útil aceptado | 3 |
| Útil omitido con frame válido | 7 |
| Fallo de extracción | 24 |
| De esos fallos, sobre evidencia útil | 11 |
| Controles de autor correctos, aparte | 2/2 |

Los 24 fallos no se cuentan como abstenciones fiables ni verdaderos negativos. Son 14 casos de negación sin predicado vinculado, cinco de negación incoherente con la polaridad, cuatro de sujeto/propiedad fuera de afirmación y uno de destino incompatible con contexto. No aparecieron referencias a IDs inexistentes o de otra fuente en este censo: restringir el esquema resolvió esa parte del formato, sin resolver los enlaces entre afirmaciones.

Las tres aceptaciones incorrectas incluyen nuevamente el nombre de un buzón como contraseña y una negación sobre Neblina transferida al libreto Luciérnaga; otra pertenece a otro trazado. Entre las pérdidas están datos parciales de autonomía y de estado, además de los casos de contexto discutidos del gold original. Resolver esas discrepancias no elimina los demás errores. No se cambió ninguna etiqueta ni se recalcularon resultados históricos con otro criterio.

Una fuente con un resultado de frote seco y una prueba húmeda todavía no realizada produjo una afirmación de polaridad positiva que incorporaba el alcance negativo de la otra cláusula. Otra perdió el sujeto porque lo citó en una cláusula anterior fuera de su rango. Son problemas de descomposición y referencia de las afirmaciones; no se solucionan haciendo pasar la literalidad por prueba semántica ni descartando información parcial válida.

Como diagnóstico **posterior**, se compararon los campos de pregunta de los siete candidatos por consulta, ignorando solo el orden de conjuntos y grupos. En 17 de las 18 preguntas con múltiples candidatos cambió la pertenencia de los tokens de identidad. Esto no prueba que toda diferencia sintáctica cambie el significado, pero el extractor conjunto no entrega una proyección fija de pregunta independiente del candidato. La medición no modifica selección, puertas, etiquetas o costes: el informe original se conserva y el ampliado coincide en todos los demás campos.

## Coste, latencia y decisión

Los 128 juicios del censo costaron **USD 0,107496000**, con cero importes desconocidos. Mediana por extracción: **1693,82 ms**; p95: **2280,50 ms**. Incluyen red, servidor y parsing del cliente; no son latencia CLI de `recall` ni de integración con Codex. No se probó agrupación ni se asumió que compartir reglas preserve calidad. No hay pase de recursos ni promoción.

Esta fase suma **148 operaciones / USD 0,122813600**. El acumulado pasa a **4209 operaciones**, **21 campañas emparejadas completas** y **USD 0,373368858 conocidos**, más **tres operaciones históricas con coste desconocido**. La programación y revisión con Codex quedan fuera de estos importes. Las solicitudes preparadas y no ejecutadas no se contabilizan como operaciones facturadas.

La siguiente realización deberá separar la proyección de pregunta, calculada una vez sin candidatos, de la extracción de afirmaciones de memoria sin pregunta. Tendrá que conservar antecedentes, cláusulas positivas/negativas y condiciones antes de contrastar relaciones. Esta es una hipótesis de arquitectura, todavía no implementada ni una mejora medida; añadirá etapas y solo se justificará con calidad y coste reales. El censo no apoya seguir añadiendo campos al mismo extractor conjunto.

## Estado de los cinco pasos

1. **Contrato:** distingue identidad, premisa y dato solicitado; los prototipos muestran que su extracción estable y el vínculo semántico siguen pendientes.
2. **Banco:** fuentes, preguntas, revisiones y contraejemplos están trazados; se conservan las discrepancias y las etiquetas históricas. Esta fase no añade una revisión ciega ni una cohorte nueva.
3. **Mecanismo:** esta candidata falla; sigue pendiente rechazar evidencia ajena sin perder partes útiles.
4. **Casos nuevos y revisión ciega:** no se abre V6 ni se llama al proveedor hasta superar las puertas de desarrollo. Sus hashes permanecen intactos.
5. **Reporte y adopción:** quedan métricas y costes de cada intento; el informe final y una posible incorporación dependen de una candidata que cumpla calidad y recursos. La configuración predeterminada permanece igual y el goal sigue activo.

Verificación: **172 pruebas del workspace**, formato, compilación del workspace y diff. No hubo nueva DB, servicio, reinicio, commit o push. El snapshot027 conserva la contabilidad y un archivo parcial de fuentes; no archiva binarios.

Artefactos: [censo original](../../outputs/iromem/evaluation/assertion-projection-sets-v4-census-2026-10-19/report.json), [diagnóstico ampliado](../../outputs/iromem/evaluation/assertion-projection-sets-v4-census-2026-10-19/report-with-query-variation.json), [preflight](../../outputs/iromem/evaluation/assertion-projection-sets-v4-census-2026-10-19/preflight.json), [citas v1](../../outputs/iromem/evaluation/assertion-projection-probe-2026-10-19/report.json), [índices v2](../../outputs/iromem/evaluation/assertion-projection-tokens-v2-probe-2026-10-19/report.json), [IDs v3](../../outputs/iromem/evaluation/assertion-projection-ids-v3-probe-2026-10-19/report.json) y [conjuntos v4](../../outputs/iromem/evaluation/assertion-projection-sets-v4-probe-2026-10-19/report.json).
