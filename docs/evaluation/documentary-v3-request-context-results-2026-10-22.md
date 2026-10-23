# Contexto del pedido y norma impersonal: implementación local

El perfil optativo `documentary_request_context_v1` está implementado y verificado en el evaluador, con **37/37 controles CLI**, **757 pruebas aprobadas**, cero fallos y 24 ignoradas. La regresión conserva **216/216 informes históricos idénticos byte a byte**. No se cambió el producto ni se cerró el objetivo de cinco pasos: continúa en el paso 3.

El [contrato previo](documentary-v3-request-context-contract-v1-2026-10-22.md) se congeló en tres versiones antes de implementar. La tercera crítica no dejó bloqueantes de diseño. Las negativas normativas contextuales quedaron explícitamente fuera de V1: negar una obligación y obligar a no actuar son proposiciones diferentes. Las versiones anteriores y sus cinco objeciones se conservan en `request-context-contract-v1/design-v{1,2,3}.md` y `critical-review-v{1,2,3}.*` dentro de los artefactos de esta tanda.

## Comportamiento implementado

El contexto nominal queda ligado a la magnitud del mismo claim o pedido: una cantidad durante una ruta no responde por otra ruta ni pierde su condición al pasar por un adaptador. El máximo conserva su calificador. Una observación de autonomía puede aportar contexto parcial a un máximo de la misma autonomía y ruta, con términos, proposiciones de respuesta y bindings vacíos; una duración operativa no se convierte en autonomía.

La finalidad o condición necesaria de una acción subordinada se conserva sin afirmar que ocurrió, sin inferir quién realiza un infinitivo y sin confundir necesidad con suficiencia. Un actor explícito puede responder si su clase, destinatario, destino, control, ámbito y condiciones están acreditados. Una norma impersonal compatible conserva un patrón abierto como contexto parcial; no fabrica un tutor individual ni existencia, universalidad o matrícula realizada.

Los carriers se validan con perfil cerrado, citas, referencias, ámbito y dependencias implícitas. Los consumidores históricos rechazan expresamente el perfil en comparación; solo la entrada contextual lo ejecuta. Se rechazan claves JSON duplicadas en la recepción V1 antes de construir el valor normalizado. El presupuesto compartido limita preflight, recorridos instrumentados, dependencias, componentes de identidad, candidatos y materialización: agotarlo invalida la operación completa, conserva el original o su referencia/hash y evita fallback o selección de las primeras celdas. Sus límites lógicos no son una medición de RAM física o latencia de producto.

Las lecturas alternativas conservan todas las celdas. Sus firmas incluyen los owners relevantes, contextos, mapas y control; no incorporan un claim ajeno solo por existir en el documento. Los IDs locales y el orden de conjuntos no crean diferencias semánticas. El agregado exige testigos completos iguales en todas las celdas; los parciales permanecen `conditional_reading`, sin respuesta completa.

## Casos nuevos y revisión separada

Se redactaron ocho documentos sintéticos nuevos, cuatro fuentes y cuatro preguntas, sin consultar corpus, diagnósticos históricos, gold ni V6. Un crítico separado de la misma familia, sin resultados ni etiquetas, emitió **32/32 juicios faithful** y verificó 105 spans. No es revisión humana ni independencia de familia, y los documentos fueron redactados expresamente: no miden extracción autónoma de un corpus nuevo.

La primera ejecución encontró tres dependencias mal formadas: un qualifier figuraba como consumer, variante no admitida por el contrato existente. Se conservaron las entregas e informes originales en `fresh-entries-v1` y `execution-v1`; V2 elimina exactamente esos tres edges, con parche inverso que restaura los originales. Una revisión posterior del mismo crítico aceptó las tres correcciones, doce juicios fieles, sin cambios semánticos. El resultado estructural **8/8 complete corresponde a V2**, no al primer intento.

| Par nuevo | Salida flat verificada | Duración CLI observada |
| --- | --- | ---: |
| Autonomía máxima explícita durante un circuito | Apoyo completo con valor contextualizado | 13,708 ms |
| Autonomía observada frente a máximo pedido | Parcial; sin valor de respuesta | 13,638 ms |
| Tutor nombrado y finalidad con actor subordinado explícito | Apoyo completo con actor | 13,867 ms |
| Norma impersonal con control de rol explícito | Parcial; sin actor de respuesta | 13,789 ms |

Son cuatro observaciones locales de proceso CLI, incluidos arranque y serialización. No establecen mejora de latencia, p95 de producción, coste de inferencia ni tiempo de una integración Codex.

Los **37 controles** incluyen las ocho validaciones; los cuatro pares en las cuatro combinaciones flat/envelope; contextos retirados o cambiados; rechazo de ambas rutas históricas; claves duplicadas; seis contraejemplos críticos y un control adicional del orden filtro/conflictos. Las variantes derivadas son controles manuales del mecanismo, no otra extracción ciega.

## Fallos encontrados y corregidos

La crítica de código encontró cuatro defectos: bypass de clase `reference_pattern`, cierre de aliases y candidatos históricos sin débito suficiente, pérdida de respuesta independiente por un claim contextual ajeno y firmas sensibles al orden de qualifiers. Sus reproducciones pasan con el binario final. También se adelantó el filtro a la detección de conflictos: un candidato sin clase acreditada permanece pendiente, sin crear un conflicto espurio ni inventar desigualdad ontológica.

Se conservan los fallos de preparación y compilación: dos fixtures normativos sin dependencias, un import incorrecto al extraer el comprobador del grafo y un fixture de filtro que apuntaba a un referente nombrado. Se corrigieron las causas; no se relajó el validador para aprobarlos. Las pruebas de estrés cubren 192 referencias Same y numerosos owners históricos bajo el perfil nuevo. El crítico cerró sus cuatro hallazgos sobre la ejecución intermedia de 36 controles; root verificó después los 37 controles, los últimos helpers y la suite completa en el binario final.

## Alcance y estado del objetivo

La nueva ruta sigue siendo conservadora ante ámbitos cuantificados o de rama, premisas no descargadas, control/fuerza sin resolver, combinaciones múltiples de contexto y normas negativas no admitidas. Mantiene la evidencia como candidata pendiente. No promete seleccionar contexto parcial invariante a nivel envelope ni resolver cualquier formulación natural.

La última extracción autónoma consolidada conserva **99/116 completas y fieles**. El conjunto conocido con cuatro correcciones anteriores conserva **103/116**, con trece pendientes; esta ampliación no los vuelve a etiquetar ni los reextrae. Los 216 replays incluyen las 116 validaciones conocidas y cien informes previos de comparación; su igualdad es regresión, no nueva medición de utilidad. La anterior utilidad conocida 31/39 y cero ajenos/60 permanece histórica, sin atribuirla a estas cuatro preguntas nuevas.

Los pasos 1 y 2 del objetivo siguen cerrados; el 3 conserva una puerta de calidad pendiente. Los pasos 4 y 5 requieren superarla antes de contraste de producto, recursos, estabilidad y evaluación reservada. V6 no se abrió y `baseline` sigue vigente. El siguiente trabajo consiste en cerrar las carencias representacionales todavía diagnosticadas —descriptores locales, suficiencia referencial, estado pendiente y alcance temporal/ambigüedad— con operadores cerrados y controles; después, congelar la guía y ejecutar extracción completa con doble revisión. No se repetirá el corpus esperando un resultado favorable ni se llamará resuelta a información que la fuente no determina.

## Coste y artefactos

**Cero llamadas externas nuevas y USD 0 nuevos de proveedor.** Se usaron siete contextos nativos de subagentes y once turnos adicionales sobre contextos existentes. Su coste y tokens no están disponibles; no se consideran gratuitos. El ledger externo anterior sigue en 6129 operaciones, 23 campañas, USD 4,565416508 conocidos y veinte operaciones de coste desconocido, excluyendo Codex.

Artefactos: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/request-context-contract-v1/`. `execution-v3/execution-log.json` liga 37 entradas/informes por hash; `legacy-regression-v1/report.json` registra los 216 replays. El binario final es `9eed0999a3d210f09f5cc46587cd6d6d9a9c28e3fea61c7616866f20bc59581d`; `verification/verification.json` liga fuentes, pruebas y recursos. No se iniciaron servicios, hicieron llamadas de pago, modificaron credenciales ni publicaron cambios.
