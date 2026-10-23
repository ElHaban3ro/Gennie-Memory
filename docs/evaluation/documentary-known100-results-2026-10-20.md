# Suficiencia documental en 100 pares conocidos

UTC 2026-10-20. Avance del paso 3: representación y resolver locales sobre anotaciones manuales, sin extracción automática, nueva política runtime ni llamadas de proveedor. El objetivo de cinco pasos permanece activo. La cohorte combina los 64 controles mínimos y 36 juicios condicionales de proposiciones ya conocidos; no incluye todo el banco difícil de 126 pares ni es validación nueva.

## Hallazgos y correcciones

La revisión ciega de los 83 textos y 33 preguntas detectó dos carencias antes de ejecutar. «Se midió la autonomía» no identifica un método: V5 ahora conserva `measurement_method_scope=unspecified` en lugar de inventarlo. Una pregunta genérica puede recibir ese estado; una pregunta por una prueba concreta no. Esta extensión no activa el puente de ejecución a una magnitud si falta procedimiento. [Representación de identidad parcial](measurement-identity-v1-extension.md).

«Admite Borrador» tampoco implica «usa Borrador». La pregunta declara su fundamento solicitado independientemente del candidato: las siete preguntas que dicen «usa» reciben la lectura explícita `actual_assignment`. Se conservan el borrador, la objeción inicial de tres pares pendientes y la conciliación previa a ejecución. Bajo esa lectura, dos A y un C históricos de capacidad no satisfacen el pedido de uso; se registran como diferencias prospectivas de representación/contrato, sin modificar gold o porcentajes anteriores. C requiere asignaciones de ambos sujetos. [Alcance solicitado e incompatibilidad direccional](request-basis-v1-extension.md).

La crítica del código encontró además que B ignoraba el fundamento: no-uso podía refutar capacidad. Se corrigieron ambos sentidos: no-uso y capacidad pueden coexistir; uso observado puede refutar incapacidad bajo misma identidad/condiciones. Una incapacidad explícita puede excluir uso requerido, pero una mera norma puede incumplirse y no demuestra imposibilidad física. Las comparaciones de valores frente a conjuntos exhaustivos funcionan en ambas posiciones. Las pruebas incluyen negación, capacidad, asignación, observación, norma, condiciones distintas y método no especificado.

El texto de Olmo sigue admitiendo una lectura pragmática adicional de exclusión de red. No se fuerza exhaustividad ni se infiere ausencia de red desde lluvia. Los hechos literales ciertos de suministro inmediato se conservan; la pregunta por red mantiene A/B y revisión, sin publicar B cierta. La pregunta por depósito conserva su dato cierto aunque otra interpretación afecte a una propiedad distinta.

## Evaluación antes congelada y resultado

El autor y el crítico ciego concordaron en los 100 roles, selecciones, necesidades de revisión y cuentas de hechos originales que por sí solos contribuyen al campo solicitado. La calificación obligatoria es el mínimo que exija cualquiera de ambas anotaciones; advertencias conservadoras adicionales se cuentan aparte. Ese criterio se fijó antes de ejecutar, con etiquetas conciliadas y hashes. La revisión es del mismo modelo/familia, sin revisión humana. No prueba verdad externa ni grounding de las proposiciones en sus conversaciones históricas.

| Medición | Resultado | Límite |
| --- | ---: | --- |
| Comportamientos de relación/selección/revisión/cuenta | 100/100 | Entrada anotada manualmente; no calidad NLP |
| Pares seleccionados | 40 | Incluyen apoyo, refutación y aportes parciales; no respuestas completas |
| Pares sin selección | 60 | Bajo el alcance de pregunta declarado |
| Hechos pedidos invariantes | 31 | Todos con testigos comprobados; conservan calificadores y sujeto original |
| Relación con revisión | 1 | Olmo/red: A cierta como aporte, B condicional; no B demostrada |
| Calificaciones emitidas | 24 | Dos advertencias conservadoras adicionales respecto al mínimo anotado |
| Comparaciones de renombrado/orden | 300/300 estables | Incluye 50 identificadores renombrados, resultados negativos y ambiguo |
| Pruebas workspace | 357 con/sin HTTP2 | Trece pruebas nuevas respecto a la etapa anterior |
| API / coste nuevo de proveedor | 0 / USD 0 | No incluye Codex/desarrollo/revisión |

`passes_manual_representation=true`; **`passes_autonomous_quality=false`** porque todavía no hay evaluación de extracción ni recorrido automático. No se presenta 98→100 como mejora de rendimiento: V1, V4 y V5 documental previo se reproducen idénticos en bytes, con sus propios contratos y resultados. El 100/100 actual usa una representación prospectiva que hace explícitas diferencias antes ausentes. No hay una medición nueva de latencia integrada, tokens runtime, ahorro o estabilidad de proveedor.

El auditor comprueba cada hecho publicado contra la fuente cierta o todas las lecturas cerradas, preservando valores, componentes, fundamento y scopes. En una cadena de suministro, el hecho intermedio que identifica al sujeto cuenta como testigo directo; el siguiente tramo permanece en el registro para composición, sin duplicarlo como respuesta independiente ni inferir un endpoint sin prueba. La selección del texto completo no significa que el resolver haya calculado esa composición.

El scorer rechaza dos adulteraciones aun con hashes recalculados: retirar un mínimo de calificación sin conciliar la etiqueta final y cambiar la cuenta del revisor. Las salidas y puntuaciones nuevas se reproducen idénticas. Los snapshots 001–050 permanecen intactos; V6 no se abrió semánticamente ni recibió llamadas. No hubo servicios, puertos, credenciales, SQL o cambios de configuración predeterminada.

## Lo que sigue y contabilidad

La siguiente prueba debe medir **extracción desde texto**: hechos de cada fuente sin pregunta y solicitud de cada pregunta sin candidatos. Comparar fidelidad, relaciones finales y hechos conservados, manteniendo fuera del prompt etiquetas, IDs de benchmark y resultados. Un fallo de extracción se registra como tal, sin convertirlo en abstención fiable ni reemplazar la salida por anotaciones manuales. El pase manual permite preparar ese diagnóstico; no permite aún abrir V6, afirmar integración ni promover. La suficiencia de todo el desarrollo difícil y las puertas de calidad/recursos/estabilidad siguen pendientes.

Acumulado del objetivo: **5881 operaciones lógicas de proveedor, 23 campañas completas, USD 0,582095598 conocidos y cinco operaciones de coste desconocido**, excluyendo Codex. Esta etapa no añade gasto remoto. Baseline y dashboard permanecen sin cambios.

Artefactos: `tests/fixtures/documentary-known100-v1/` y `outputs/iromem/evaluation/documentary-known100-v1-2026-10-20/`. Ejecución: `audit-epistemic-binding-v5 <fuente> <salida-nueva>` y `score-documentary-known100 <banco> <salida> <score-nuevo>`.
