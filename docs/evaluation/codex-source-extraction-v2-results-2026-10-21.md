# Extracción Codex V2: piloto conocido completo

**Ocho de ocho extracciones completas y faithful en las cuatro dimensiones para ambos críticos independientes. El piloto pasa; el objetivo completo todavía no.** Se repitieron las cuatro fuentes y cuatro preguntas, sin reparar las salidas V1 ni retocar referencias. No es una medición de recall ni una comparación contra baseline.

## Cambio y prueba

Dos propuestas independientes, sin gold ni resultados anteriores, examinaron las variantes del esquema original y no justificaron una representación de entrada por vía. Este diagnóstico no prueba imposibilidad general. Se definió la propiedad explícita [entry_via](entry-via-v1-contract-2026-10-21.md): el sujeto pasa/entra por la vía, sin convertirla en proveedor inmediato u origen, reemplazar una red por su propietario ni inventar exclusividad. Un crítico conceptual encontró el riesgo de distribuir alternativas como hechos o borrar condiciones; se incorporó la regla y sus fronteras manuales antes de inferir.

[Protocolo V2](codex-source-extraction-v2-protocol-2026-10-21.md), contrato, paquetes y copias parciales de fuente ligados antes de las entregas. Solo se añade entry_via al vocabulario, su definición general a instrucciones y la expansión correspondiente de los esquemas. Textos y orden se conservan. Dos agentes nuevos reciben fuente y pregunta por separado, sin frames manuales, gold, respuestas anteriores ni feedback del normalizador. Contexto compartido dentro de cada etapa; no equivale a ocho inferencias aisladas o un transporte productivo. Los hashes/declaraciones no prueban criptográficamente el contexto efectivo.

## Métricas

| Medida | V1 anterior | V2 actual |
| --- | ---: | ---: |
| Entregas recibidas / previstas | 8/8 | 8/8 |
| DTO completos: fuentes | 3/4 | 4/4 |
| DTO completos: preguntas | 4/4 | 4/4 |
| Entradas faithful en todas las dimensiones para ambos críticos | 7/8 | 8/8 |
| Juicios faithful del primer crítico | 28/32 | 32/32 |
| Juicios faithful del segundo crítico | 28/32 | 32/32 |
| Desacuerdos de dimensión | 1 | 0 |
| Puerta del piloto | Falla | Pasa |

La comparación es de desarrollo conocido con tareas Codex y críticas nuevas; no aísla causalidad de la guía frente a variación de los modelos ni demuestra generalización. V1 conserva su abstención y métricas. La salida nueva representa entrada por tubería como entry_via del sujeto red con propietario huerto; no la publica como fuente del agua. Los otros siete contenidos también se regeneraron y revisaron completos. No se eligió la mejor respuesta entre versiones o agentes según gold.

[Normalización](../../outputs/iromem/evaluation/codex-source-extraction-v2-2026-10-21/normalization.json), [dos críticas conciliadas](../../outputs/iromem/evaluation/codex-source-extraction-v2-2026-10-21/review-reconciliation.json) y [reporte consolidado](../../outputs/iromem/evaluation/codex-source-extraction-v2-2026-10-21/report.json). El normalizador puro conserva su indicador de revisión no incorporada; el consolidado añade los dictámenes por separado. No hace falta arbitraje: cero desacuerdos. Es revisión de modelos del mismo sistema, sin validación humana o verdad externa.

## Integridad y límites

El revisor estático encontró tres problemas de V2: alias requirement.item_attribute para la propiedad reservada, metadatos de fuente sin comprobar y falta de derivación exacta desde V1. Se conserva su dictamen inicial. Se corrigieron con rechazo de ese alias, comparación exacta de paquetes derivados, vínculo al protocolo padre y validación de las cinco copias parciales previas. El rechazo de contenido inválido se registra por fila y conserva el denominador. La relectura estática no identifica más hallazgos accionables; no es revisión semántica ni ejecución de pruebas.

El reporte fue reproducido con el normalizador final después de esos ajustes, byte por byte idéntico, sin modificar ninguna entrega. El adaptador archivado antes de inferencia difiere del reporter final por estas verificaciones. Otras fuentes del normalizador final se archivaron después y se declara ese momento; no se afirma enlace previo completo. Binario identificado por hash, no archivado.

Pasan **516 pruebas offline** con y sin HTTP2, además de quince comprobaciones CLI: replay, no sobrescritura, preparación reproducible de paquetes, vínculos/cobertura/exposición, drift coherente de texto y vocabulario, padre/archivo previo alterado y dos usos incorrectos de entry_via degradados por fila. Las fronteras manuales conservan sujeto/propietario, polaridad, modalidad, condición y alternativa abierta; no permiten publicar un origen a partir de una entrada. Son pruebas de representación, no extracción ni pertinencia general.

Cuatro bancos manuales, diez reportes históricos de extracción, nueve auditores/reportes previos y ambos reportes Codex anteriores se reproducen idénticos. Snapshot066 y anteriores intactos; V6 solo comprobada por hash de manifiesto.

Se registra una preocupación retrospectiva del coordinador, que conoce el historial: a028 manual usa sujeto Verbena y water_supply/T2 donde el texto habla de una red del huerto y entrada por tubería. Es una atribución que requiere revisión separada antes de usar 100/100 manual como evidencia semántica. **No es una etiqueta nueva, fallo puntuado ni corrección del gold.** La representación y resultados antiguos permanecen intactos. La pertinencia parcial entre relaciones/referentes también debe validarse; falta de matching no se redefine como irrelevancia para mejorar cifras.

## Coste y siguiente puerta

**Cero llamadas OpenRouter nuevas. Coste y tokens Codex desconocidos.** Sin medición nueva de latencia de recall, ahorro o recursos productivos. Acumulado proveedor invariable: **6129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos más 20 operaciones de coste desconocido**, excluyendo Codex.

El siguiente banco conocido contiene 83 fuentes y 33 preguntas: 116 extracciones distintas para 100 pares. Se registra el plan de lotes, todavía sin llamadas ni adaptador de banco completo; antes de puntuar se revisará la fidelidad de referencias por separado. Después siguen extracción/recorrido completos, conservación de evidencia útil, conocidos126, comparación baseline, costes/latencia/estabilidad y cohorte nueva ciega. No se adopta la variante ni se cambia producto/default, SQL, dashboard, credenciales, dependencias, servicios, README o gráficas de recuperación. Goal activo en paso 3.
