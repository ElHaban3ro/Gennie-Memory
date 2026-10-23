# Resolver sobre hechos manuales: 100 pares conocidos

UTC 2026-10-20. **La puerta de suficiencia sigue fallando.** El resolver local coincide con 98 de 100 roles históricos, discrepa en uno y deja uno explícitamente ambiguo. Son hechos anotados manualmente, no un 98 % de recuperación automática ni una mejora del modelo en uso. Baseline continúa vigente; V6 no se abre.

## Qué se implementó

`tools/evaluation/src/typed_facts.rs` separa hechos de cálculo de relación: sujetos individuales/colecciones/componentes, modalidad, condiciones fijas, configuración supuesta, valores/límites/estados, polaridad y conjuntos exhaustivos. El código calcula B/A/C/none sin recibir etiquetas ni IDs de evaluación. Los hechos no contienen role/useful/same_case. Dos lecturas con roles distintos devuelven falta de cobertura. El adaptador separado de puntuación reproduce resultados, verifica hashes y exige que sus etiquetas coincidan con el gold de las fuentes proporcionadas.

El banco de esta prueba contiene 33 preguntas únicas, 83 textos de afirmaciones y 100 pares: 64 mínimos y 36 roles condicionales de proposiciones ya conocidas. Textos idénticos comparten hechos. No se comprueba aquí respaldo fuente→proposición ni extracción automática.

Antes de la ejecución, el crítico de anotaciones revisó los 116 textos y detectó doce objeciones. Se conservaron borrador, revisión y adjudicación; entre las correcciones: retirar desconocimiento inferido desde no-medición, ligar fechas al aspecto medido, separar admisión de uso, conservar ámbito de registro/copia, ruta, ensayo, lluvia y proveedor inmediato. La segunda revisión no detectó infidelidades bloqueantes para el diagnóstico. El crítico no recibió gold ni resultados; sí conocía un encargo anterior de Olmo. Son agentes de la misma familia, sin revisión humana ni independencia entre modelos.

La crítica del código detectó aceptación de negaciones de estados como positivos, ausencia de contraejemplo individual frente a una premisa universal factual, contraste generado por una clave adicional sin valor alternativo y pérdida de estados explícitos sin tipo del valor. Se corrigieron con regresiones. La revisión posterior detectó alcance de flujo y registro ignorados en refutación: se corrigieron y probaron. No se añadió un enlace especial para forzar A en el caso de no-medición.

## Resultados ejecutados

| Desarrollo conocido | Pares | Coinciden con gold | Discrepan | Ambiguos sin resolver |
| --- | ---: | ---: | ---: | ---: |
| Mínimos v3 | 64 | 64 | 0 | 0 |
| Proposiciones condicionales | 36 | 34 | 1 | 1 |
| Total | 100 | 98 | 1 | 1 |

El ambiguo permanece dentro del denominador y bloquea el pase. Cero colisiones de firma pregunta/hechos con etiquetas distintas en este conjunto; no prueba que la gramática sea completa para otros textos.

- `pcv1_08_01/claim_00`: exclusividad de proveedor inmediato permite A como eslabón intermedio; exclusión de todo recorrido por red permite B. Gold histórico B conservado; el resolver informa A/B sin adjudicar la lectura.
- `pcv1_control_17/claim_01`: «No se midió la duración del reproductor Lino hasta apagarse». El resolver devuelve none; gold histórico A conservado. La representación contiene solo ausencia del procedimiento, sin inventar desconocimiento de la autonomía.

Una revisión semántica posterior al diagnóstico, sin gold ni resultados entregados al crítico, recomienda none bajo la definición estricta de A y señala tensión con la introducción más amplia de v3. No realizar una prueba puede coexistir con un máximo conocido por especificación. Esta objeción se registra aparte: **no convierte el resultado publicado en 99/100 ni cambia el gold**. Una próxima versión deberá declarar si el estado de una medición del atributo cuenta como aporte de evidencia, separado de disponibilidad del valor; requerirá casos límite y revisión antes de ejecutarse. [Revisión específica](../../outputs/iromem/evaluation/typed-facts-sufficiency-v1-2026-10-20/measurement-status-review.json).

Los 100 resultados se conservaron exactamente en tres variantes: orden de hechos/preguntas/alternativas, renombrado de entidades/atributos/valores/configuraciones/IDs y ambas transformaciones. Son 300 comparaciones adicionales, no casos nuevos ni pruebas de extracción. El fallo y la ambigüedad también se conservan. Se reprodujeron informe bruto y puntuación byte por byte; se rechazaron sobrescritura, cambio de etiquetas frente a gold fuente y hechos distintos para el mismo texto.

[Contrato](typed-facts-v1-contract.md), [entrada revisada](../../outputs/iromem/evaluation/typed-facts-sufficiency-v1-2026-10-20/input-reviewed.json), [roles brutos](../../outputs/iromem/evaluation/typed-facts-sufficiency-v1-2026-10-20/report.json), [puntuación](../../outputs/iromem/evaluation/typed-facts-sufficiency-v1-2026-10-20/score.json), [transformaciones](../../outputs/iromem/evaluation/typed-facts-sufficiency-v1-2026-10-20/metamorphic-audit.json), [verificación](../../outputs/iromem/evaluation/typed-facts-sufficiency-v1-2026-10-20/verification.json).

## Coste, límites y siguiente puerta

315 pruebas pasan con configuración predeterminada y con HTTP/2 optativo; compilación y formato pasan sin avisos. Dieciséis pruebas nuevas de comportamiento cubren relación, casos ajenos, universal/miembro, partes, planes, contraste, negación, tipos/estados y alcances. Se conservaron fuentes ejecutadas y fuentes finales verificadas: tras la primera ejecución se reforzó exclusivamente la comprobación de gold del puntuador, sin cambiar el cálculo ni los resultados. No hay app, SQL, servicio, dependencia, cambio de proveedor o política nuevos.

Llamadas remotas nuevas: 0. Coste de proveedor nuevo: USD 0. Acumulado conservado: 5.881 operaciones de proveedor, 23 campañas completas, USD 0,582095598 conocidos más cinco operaciones de importe desconocido. Excluye programación/anotación/revisión de Codex. Los 100 pares y sus transformaciones son operaciones locales, no se suman a ese contador remoto. No se midió coste/p50/p95 de un recorrido automático; no se puede afirmar ahorro ni mejora de latencia. Los resultados anteriores de modelos no son directamente comparables con hechos manuales perfectos.

1. Precisar prospectivamente el alcance de A para estado de evidencia/procedimiento frente a estado del dato; mantener identidad, premisa y atributo separados. Resolver el tratamiento de lecturas ambiguas sin escoger la que aprueba.
2. Congelar ejemplos límite nuevos con revisión sin resultados, conservando los dos desacuerdos históricos y sin reetiquetarlos.
3. Solo después probar una representación y resolver revisados. Sigue pendiente la puerta local conjunta; este pase parcial no permite ampliar ni construir un extractor sobre una semántica no fijada.
4. Extracción automática, recorrido real, recursos ≤1,5×, campañas completas y estabilidad siguen pendientes antes de V6 y revisión ciega nueva.
5. Publicar métricas y coste de cada etapa; adoptar únicamente una mejora demostrada. Goal activo, sin promoción ni cambios de README o gráficas del producto.
