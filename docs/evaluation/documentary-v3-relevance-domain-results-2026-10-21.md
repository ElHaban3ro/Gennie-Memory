# Aplicabilidad de dominio en enlaces de contexto V3

Los enlaces nuevos seleccionaban recuerdos sobre el mismo destino aunque hablaran de paquetes, energía o un medio no acreditado. También conservaban el contexto de Azucena sin comprobar el apoyo que lo relaciona con el agua. Los contraejemplos se fijaron y ejecutaron antes de cambiar el mecanismo: ambos documentos eran completos en los primeros diez controles, sin rechazos vacuos del validador.

Se implementó `relevance_profiles`, optativo y versionado, en el vocabulario del evaluador. Cada fila identifica un enlace existente y admite alternativas explícitas:

- `class_context`: el rol fuente y el rol objetivo tienen clases declaradas dentro de las listas configuradas. La declaración concreta relaciona depósito de lluvia con agua como contexto; no descubre una ontología ni iguala referentes.
- `answer_filter`: los términos fuente, incluidos **todos** los miembros si el rol corresponde a una restricción, cumplen el filtro léxico, semántico o patrón referencial solicitado. No se afirman miembros por separado.
- `corroborating_claim`: una única proposición fuente positiva fact/state acredita bindings referenciales locales de fuente y objetivo. Debe ser elegible, sin rama, binder, dependencia o contexto pendiente, con ámbito y fundamento compatibles y sin una proposición negativa equivalente en el mismo contexto. No hay recursión, regla transitiva ni conclusión de origen.

La forma es cerrada: versión, enlace, roles referenciales, alternativas, clases y bindings se validan; una prueba auxiliar necesita anclaje a fuente y objetivo. Ocho mutaciones comprueban rechazo de versión/enlace/rol desconocidos, clases vacías, anclaje incompleto, fila duplicada, campo extra y rol auxiliar inexistente, con hashes recalculados para evitar falsos rechazos solo por hash.

Si el perfil no acredita aplicabilidad, se conserva candidato con `relevance_profile`, sin selección y sin demostrar Difference. Ausencia del perfil conserva el comportamiento anterior. Los perfiles son configuración declarada del experimento, no configuración runtime ni cambios del producto.

## Resultados

| Medida | Anterior sin perfil | Nuevo sin perfil | Nuevo con perfil |
| --- | ---: | ---: | ---: |
| Útiles no ambiguas del banco /39 | 33 | 33 | 33 |
| None seleccionadas del banco /60 | 0 | 0 | 0 |
| Respuestas completas del banco | 3 | 3 | 3 |
| Referencia ambigua seleccionada /1 | 1 | 1 | 1 |
| Concordancia heredada del banco /100 | 94 | 94 | 94 |

Los 100 pares conservan textos, frames y referencias originales; solo cambia el vocabulario y su hash en la variante protegida. Ambas ejecuciones sin perfil reproducen exactamente los 100 informes previos. Se preservan todas las 33 selecciones útiles y las tres respuestas completas, sin nuevas selecciones none. Quedan seis omisiones útiles (3A,2B,1C) y la interpretación ambigua continúa separada. La concordancia 94/100 incluye esa referencia y no es exactitud automática; sobre los 99 no ambiguos es 93/99.

| Controles prospectivos de desarrollo | Anterior sin perfil | Nuevo con perfil |
| --- | ---: | ---: |
| Negativos seleccionados /10 | 10 | 0 |
| Positivos conservados /3 | 3 | 3 |
| Comportamientos previstos cumplidos /13 | 3 | 13 |

Los diez negativos incluyen paquetes, energía, medio no especificado, auxiliar con otro anclaje, negado, en rama, ausente, contradictorio, con calificador pendiente y clase objetivo ausente. El auxiliar con calificador pendiente produce documento parcial deliberadamente; los otros doce documentos fuente y todas las preguntas son completos. Ninguno es inválido. Son controles manuales y mutaciones mecánicas, no revisión ciega, extracción fiel ni prueba de generalización poblacional.

Nardo conserva su relación original mediante la declaración explícita de clases. Azucena requiere c3 como apoyo del contexto de c1; el witness conserva `support_claim_ids:[c3]` y sus citas, pero publica solo la proposición original c1. Sustancia y origen siguen pendientes; no publica T6 como respuesta por esta vía. Los dos casos de Olmo acreditan el filtro pedido y conservan proposición o restricción íntegra, sin términos nuevos ni proposiciones por miembro.

## Verificación y límites

Pasan **607 pruebas, 24 omitidas**: 85 del motor/validador V3 y dos del adaptador CLI. Compilación y formato pasan. El primer control positivo detectó un error de implementación: el comprobador trataba todos los registros de candidatos como pendientes, incluidos los elegibles. Se conserva el fallo y se corrige comprobando su estado; no se debilitaron las expectativas. Los primeros diez controles permanecen intactos al añadir tres negativos adicionales y el vocabulario propuesto a la envoltura.

El hash final reproduce **357 informes idénticos**: 200 pares candidatos, 13 controles propios, 28 controles previos y 116 validaciones congeladas de extracción. Cuatro witnesses con perfil se auditaron para conservar estructura fuente y no publicar origen ni miembros. La redacción del protocolo confundía el total de controles con su número de negativos; un addendum vinculado por hash aclara 13 totales, diez negativos y tres positivos, sin modificar el protocolo, fixture ni resultados.

Esta mejora elimina selecciones indebidas demostradas en desarrollo y conserva la cobertura manual conocida. No acredita que las declaraciones semánticas cubran otros dominios, tiempos o juicios normativos; sigue haciendo falta revisión independiente y más contrastes. La última extracción automática continúa en **65/116 fieles para ambos críticos**, sin nueva medición. Baseline, recursos, latencia, estabilidad y V6 siguen pendientes. No se reactivaron subagentes ni se modificó la política del producto.

Coste nuevo: **0 peticiones API y USD 0 de proveedor**; coste/tokens Codex desconocidos. Acumulado intacto: 6129 operaciones, 23 campañas, USD 4,565416508 conocidos más 20 importes desconocidos, excluyendo Codex. Objetivo activo en paso 3, puerta global fallida.

[Propuesta](../../outputs/iromem/evaluation/documentary-v3-relevance-domain-2026-10-21/proposal-final.json), [protocolo](../../outputs/iromem/evaluation/documentary-v3-relevance-domain-2026-10-21/protocol.json), [comparación](../../outputs/iromem/evaluation/documentary-v3-relevance-domain-2026-10-21/report.json), [ejecución](../../outputs/iromem/evaluation/documentary-v3-relevance-domain-2026-10-21/execution-provenance.json), [vocabulario](../../outputs/iromem/evaluation/documentary-v3-relevance-domain-2026-10-21/vocabulary.json) y [controles](../../tests/fixtures/documentary-v3-relevance-domain-controls-v1.json).
