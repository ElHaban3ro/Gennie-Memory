# Referencias con evidencia literal V1: sin pase de calidad

UTC 2026-10-21. Se implementó y ejecutó el nuevo contrato sin modificar las versiones anteriores. **4/8 DTO válidos; seis defectos de contenido acordados. No mejora la extracción ni habilita ampliar.** Goal activo en paso 3; baseline y V6 conservados. [Protocolo](grounded-reference-v1-protocol-2026-10-21.md), [verificación](../../outputs/iromem/evaluation/grounded-reference-v1-2026-10-21/verification.json).

## Implementación y prueba previa

El modelo declara mención, clase literal o ausencia explícita, nombre y propietarios por separado. La identidad se codifica de forma reversible conservadora; el tipo ontológico para rutas se conserva en un campo diferente, con cita propia. El módulo puro adapta esta versión al traductor por cláusulas y al parser V5 sin cambios. Una referencia inválida degrada toda la operación; no se reconstruyen nombres o clases faltantes desde su cita, la pregunta ni gold. [Decisión 0024](../decisions/0024-grounded-reference-evaluation.md).

Las comprobaciones cubren clase ajena a la mención, nombre/propietario/padre/cardinalidad, tipo ausente, clase anónima, Unicode/case/plural/clases largas sin truncado, tipos semánticos separados, rutas intermedias y varias cláusulas/lecturas/premisas. En una prueba de integración nueva, mi expectativa inicial C era incorrecta: el contrato V1/V5 incluye el dato parcial/intermedio en A con calificación, y reserva C para contraste de configuraciones. Se retienen log y fuente fallidos, la justificación y la prueba corregida, que exige únicamente el hecho de depósito inmediato, nunca una red inventada. No cambian resolver, banco ni etiquetas. [Adjudicación de la prueba](../../outputs/iromem/evaluation/grounded-reference-v1-2026-10-21/unit-expectation-adjudication.json).

436 pruebas pasan con y sin HTTP/2; 21 ejecuciones unitarias nuevas entre ambas inclusiones del contrato y el adaptador. Formato y diff-check pasan. La crítica inicial y la final se conservan; sin bloqueantes estáticos identificados antes de congelar. Nada de esto prueba fidelidad del modelo.

## Piloto completo conservado

Ocho posiciones idénticas al piloto full anterior, cuatro fuentes y cuatro preguntas, aisladas. Modelo OpenAI GPT-4.1, proveedor y ajustes iguales; las filas solo difieren en mensajes/esquema de referencias. No hay ejemplos, gold, frames ni catálogo de entidades en las peticiones. [Delta de payloads](../../outputs/iromem/evaluation/grounded-reference-v1-2026-10-21/payload-contrast.json).

| Indicador | Cláusulas full anterior | Referencias V1 |
| --- | ---: | ---: |
| HTTP 200 terminados | 8/8 | 8/8 |
| DTO completos válidos | 7/8 | **4/8** |
| Fuentes válidas | 3/4 | 2/4 |
| Preguntas válidas | 4/4 | 2/4 |
| Semánticamente fiel según ambos críticos | 2/8 | **2/8** |
| Fiel según ambos incluyendo evidencia exigida | 2/8 | **1/8** |
| Coste por recibos USD | 0,055206 | **0,059038** |

Son contratos distintos y comparación histórica de desarrollo; no A/B contemporánea, igualdad de dificultad ni estimación de precisión poblacional. Los cuatro DTO rechazados conservan su denominador; no se reevalúan tras relajar parser o rellenar campos.

## Significado y discrepancia de revisión

Dos revisores nuevos, sin historial, código, bancos, roles, gold ni V6, juzgaron todos los ocho textos/raw y cuatro normalizados disponibles. Son agentes de la misma familia, no humanos ni independencia poblacional. 16 juicios conservados.

El primero informa dos contenidos fieles y seis defectuosos. El adversarial informa uno fiel y siete defectuosos, pero separa explícitamente fidelidad semántica de conformidad/atribución de evidencia: el segundo caso semánticamente fiel tiene una cita del propietario demasiado amplia. El principal conserva esa diferencia: **dos acuerdos semánticos, un acuerdo de revisión completa, seis defectos de contenido acordados y una discrepancia de criterio de evidencia**. No se cambia el veredicto de ninguno ni se etiqueta la discrepancia como pérdida semántica confirmada. [Revisiones y resumen](../../outputs/iromem/evaluation/grounded-reference-v1-2026-10-21/blind-review-summary.json).

Los cuatro normalizados disponibles conservan sus campos raw, incluidos defectos heredados; cuatro no están disponibles. Fidelidad de traducción no significa fidelidad al texto original.

Hallazgos:

- El formulario conserva su clase y exención de teléfono, pero la cadena estructurada pierde el pedido O7 intermedio, aunque la mención lo contiene. Pasa el DTO sin preservar toda la identidad.
- La pregunta de O7 omite nombre y clase del pedido. La fuente de Cima conserva la frecuencia pero omite el nombre. La pregunta de Atlas omite nombre y clase. Las tres se rechazan como referencia no identificable.
- La red conserva una entidad T2 como valor, pero elimina el huerto Verbena de sus propietarios; se rechaza como referente anónimo sin propietario. No se amplía ignorando el vínculo omitido.
- La fuente Atlas conserva el nombre y cero pendientes, pero declara ausente la clase expediente que sí aparece en la mención; pasa estructura con pérdida de clase.
- Arce conserva total de músicos confirmados y su referente.
- La medición de faros conserva la pregunta sobre ejecución de autonomía; los revisores difieren sobre si la cita completa «los faros de Bruma» puede ser evidencia propia del propietario Bruma. Ambos consideran conservada su semántica, sin método o ejecución inventados.

Las menciones completas permiten detectar estas omisiones, pero no las evitan. No se repararon las descomposiciones deficientes tomando palabras de la cita ni se anunciaron como referencia correcta por contener todos los términos en algún campo.

## Cobertura, coste y tiempos

Control manual: 100/100 comportamientos previstos y 40 selecciones, sin certificar extracción automática. Los brazos mantienen denominador 100; no son 100 extracciones ejecutadas:

| Brazo | Pares calculados | Degradados | Coincidencias de comportamiento |
| --- | ---: | ---: | ---: |
| Manual | 100 | 0 | 100 |
| Fuente automática | 2 | 98 | 1 |
| Pregunta automática | 6 | 94 | 4 |
| Ambos automáticos | 0 | 100 | 0 |

Los brazos mixtos pueden perder coincidencia por namespaces manual/automático además de defectos del modelo. En ambos automáticos no hay pares calculables con las posiciones que pasaron: no se transforma su falta de cobertura en un none fiable, precisión 0% poblacional ni recuperación libre de errores.

Ocho HTTP/2 200 servidos como alias `openai/gpt-4.1`, proveedor OpenAI fijo; revisión fechada no verificada por separado. Cero reintentos y recibos nuevos desconocidos. Estimación anunciada USD 0,2055167, cota conservadora USD 0,705894. **Coste real USD 0,059038**, incluidos fallos; sin agentes/Codex. 44.455 tokens de entrada, 2314 de salida y 32.256 de entrada en caché según recibos: son tokens del proveedor experimental, no de Codex.

| Peticiones | p50 ms | p95 ms |
| --- | ---: | ---: |
| Fuentes, 4 intentos | 2482,786 | 3676,761 |
| Preguntas, 4 intentos | 1684,296 | 2055,681 |

Runner: 6222,635 ms con cuatro concurrentes; Cargo: 7,06 s. No miden recall, hook ni respuesta final. No hay pase de recursos, estabilidad o integración.

Acumulado conciliado: **5993 operaciones, 23 campañas emparejadas completas, USD 0,753482508 conocidos y 16 operaciones de coste desconocido**, sin Codex. Este diagnóstico añade ocho operaciones, no una campaña completa.

## Estado y siguiente acción

Cuatro bancos manuales y ocho informes anteriores, más el nuevo, se reproducen byte por byte. Snapshots001–055 intactos antes del cierre; estado nuevo en056. V6 conserva hash del manifest sin inspección semántica ni llamadas. No se cambian default, README/gráficas vigentes, secretos, dependencias, SQL, puertos, servicios o dashboard.

Después de completar pruebas y replays, el directorio local `target/` estaba ausente al cerrar procedencia. La causa no se estableció; el agente no lo eliminó. Se conserva el hash del runner capturado antes de las peticiones, pero el hash del binario CLI no está disponible al cierre. Las fuentes archivadas siguen coincidiendo con las actuales; no se repitieron inferencias ni se duplicó contabilidad. [Incidencia de cierre](../../outputs/iromem/evaluation/grounded-reference-v1-2026-10-21/closure-interruption.json).

La descomposición obligatoria de menciones en nombre/clase/propietarios no ha confirmado la mejora esperada. El próximo diseño debe reducir esa carga sin perder información referencial: evaluar un ancla literal completa que mantenga cabeza, propietarios y restricciones, separada de tipos ontológicos, cuantificación y enlaces. No basta borrar esos campos del mismo raw para aprobarlo; exige contrato nuevo, crítica de expresividad, controles de homónimos y misma puerta completa. No normalizar artículos, case, plurales o sinónimos para fabricar coincidencias, ni asumir identidad por cadenas iguales. La hipótesis solo está propuesta; no hay código ni calidad probada para ella.

La [crítica del ancla literal](../../outputs/iromem/evaluation/grounded-reference-v1-2026-10-21/literal-anchor-design-critic.json) no avala aún igual expresividad: identidad distribuida entre cláusulas, premisas corregibles dentro del sintagma, propietarios reordenados, universales/miembros y componentes pueden perderse. Antes de implementarla debe resolverse esa conservación o registrarse la cobertura que falta; no sustituir el objetivo por un subconjunto de referencias contiguas fáciles. Tampoco eliminar artículos automáticamente: La Paz/Paz y case de IDs son contraejemplos.

Siguen pendientes los objetivos de extracción y alineación completas, calidad conocida/difícil, recursos/estabilidad, cohorte nueva ciega y adopción. Goal activo, sin ampliación ni promoción.
