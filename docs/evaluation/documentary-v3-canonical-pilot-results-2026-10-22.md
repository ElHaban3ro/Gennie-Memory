# Piloto canónico independiente: rechazo de la candidata

La guía canónica V4 no pasa la puerta de calidad. Dos autores separados extrajeron desde cero 12 fuentes y 10 preguntas, sin recibir emparejamientos, etiquetas ni resultados. Dos críticos nuevos revisaron los 22 textos contra sus representaciones y validaciones, sin resultados del clasificador. La selección de 12 pares se fijó previamente por hash y pertenece al desarrollo conocido; no es la cohorte reservada V6.

| Medida | Resultado |
| --- | ---: |
| Textos entregados | 22/22 |
| Validaciones completas | 21/22 |
| Validaciones parciales, sin ocultar deuda | 1/22 |
| Aceptación en cuatro dimensiones, crítico A | 18/22 |
| Aceptación en cuatro dimensiones, crítico B | 18/22 |
| Aceptación conjunta | 17/22 |
| Puerta para ampliar a 116 | Falla |

Se verificaron los 44 dictámenes, sus paquetes/contratos, IDs, hashes y todas sus rutas y citas. No hubo correcciones semánticas ni reintentos favorables. La validación parcial reconoce honestamente que no puede representar para qué tarea están pendientes unas páginas; conservar la deuda no acredita completitud.

Los hallazgos concretos afectan tarea pendiente de digitalización en tres representaciones, dimensión genérica frente al formato específico de entrega en otra, y `quantity_extent` aplicado indebidamente a «toda la colección» en una pregunta no cuantitativa. Los críticos no coinciden en todas las entradas; se conservan ambos dictámenes sin conciliarlos a favor de la candidata.

Se ejecutaron los 12 pares con ambas representaciones y el mismo binario/contexto experimental. Los 12 resultados antiguos son idénticos por bytes a los congelados. Al fallar fidelidad, las cifras nuevas son exclusivamente diagnósticas y no sustituyen las métricas automáticas del banco completo.

| Diagnóstico sobre los mismos 12 pares | Representación anterior | Extracción canónica nueva |
| --- | ---: | ---: |
| Útiles seleccionados | 5/5 | 4/5 |
| Selecciones ajenas | 0/7 | 0/7 |
| Respuestas completas | 0 | 0 |
| CLI p50, sin extracción/revisión | 16,755 ms | 16,923 ms |
| CLI p95 de 12 ejecuciones | 36,312 ms | 42,634 ms |

Hay una pérdida previa, ninguna ganancia y ningún ajeno nuevo. El resultado se rechaza. No prueba latencia de producción, estabilidad ni coste completo. La calidad automática anterior permanece en 23/39 útiles y cero ajenos/60. No hay adopción, contraste contemporáneo con baseline de producto ni apertura de V6.

El muestreo aleatorio por hash dejó los cinco útiles del subconjunto ya seleccionados antes: no ofrecía margen para acreditar una ganancia. Esto no excusa su regresión ni su fallo de fidelidad. Se ha congelado separadamente la selección de un próximo piloto dirigido de desarrollo: cuatro omisiones útiles conocidas, cuatro útiles conservados y cuatro ajenos, por hash dentro de cada estrato. No se compararán porcentajes poblacionales entre esos subconjuntos; sus autores y críticos volverán a recibir solo su propio material. La guía V5 sigue en preparación y requiere revisión previa. El primer piloto y sus criterios permanecen intactos.

La revisión independiente de preflights también rechazó una falsa solución para «recibe sobres cada martes»: el binder existencial podía fijar los mismos sobres a través de la recurrencia. El pase mecánico no se adopta como fidelidad. La afirmación positiva de permiso normativo tiene apoyo en la lectura ordinaria y una reserva presuposicional; no acredita uso ni existencia física. Esas revisiones se conservan aparte y no mejoran las cifras del piloto.

Código Rust sin cambios respecto al getter verificado: 715 pruebas pasadas, 24 omitidas y 326 informes idénticos. Esta tanda ejecutó 22 validaciones y 24 clasificaciones locales. Cero llamadas nuevas a OpenRouter y USD 0 nuevos en el ledger externo; coste, tokens y latencia de producción de Codex no cuantificados. El objetivo sigue activo en el paso 3.

Evidencia: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/canonical-pilot-v1/`, guía/crítica en `canonical-contract-v1/`, próxima selección congelada en `canonical-pilot-v2/`. Binario: `3ef55fb0da656d123aa8d9fa50dbe67a986e9e3cf4159f918d2da09044dc5557`.
