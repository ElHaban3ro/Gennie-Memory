# Qwen 9B: mínimo completo y fallo de calidad

Fecha: 2026-10-22. El segundo intento completó las 16 preguntas y los 64 juicios, con recibos válidos y cero tokens de razonamiento. Resultado: **49/64 roles correctos, 11 evidencias útiles perdidas y 3 selecciones ajenas**. Se descarta esta versión y no se amplía contra baseline ni se incorpora al producto. El objetivo de cinco pasos sigue activo, sin mejora de recuperación demostrada.

## Qué se implementó y comprobó

Familia `qwen9b` solo en evaluación: Qwen3.5-9B, DeepInfra exclusivamente, sin fallback, razonamiento desactivado, temperatura 0 y salida JSON cerrada con roles A/B/C/none. Conserva exactamente mensajes, fuentes, orden y esquema de Generative Contribution V2; solo cambia modelo/proveedor. La hipótesis era de capacidad del clasificador, no un mecanismo semántico nuevo ni una ablación pura de pesos.

El primer intento terminó en la primera petición con HTTP 429, cero clasificaciones y sin recibo. Se preserva como interrupción técnica. El saldo consultado no estaba agotado. Tres sondas neutras posteriores respondieron 200: Python simple, Rust HTTP/2 simple y Rust HTTP/2 con las mismas instrucciones/esquema y cuatro fuentes neutras. Estas sondas prueban disponibilidad en esos momentos, no la causa retrospectiva del 429. No permiten atribuirlo a HTTP/2, saldo o proveedor concreto.

Se congeló un protocolo explícito para un segundo intento separado y final en esta continuación. Las 16 peticiones y entradas son byte por byte idénticas al primer intento; no había respuestas previas de Qwen entre las que elegir. No hubo reintentos dentro de la tanda. Se añadió captura acotada de errores HTTP solo para observabilidad del intento separado, sin cambiar payload o selección. Una revisión detectó que un fallo de escritura podía impedir conservar el recibo: se corrigió antes de red, con escritura observacional sin panic, sincronización y prueba de no sobrescritura/fallo de directorio. El diario conserva el fallo/consumo aunque el sidecar no pueda guardarse.

## Resultado sobre el mínimo conocido

| Métrica | Segundo intento |
| --- | ---: |
| Respuestas válidas | 16/16 |
| Roles correctos | 49/64 |
| Escenarios con todos los roles correctos | 2/16 |
| Evidencias útiles conservadas | 6/17 |
| Evidencias útiles perdidas | 11 |
| Selecciones ajenas | 3 |
| Errores de rol | 15 |
| Coste API de esta tanda, recibos completos | USD 0,000823150 |
| Tokens de entrada/salida | 7921/207 |
| Tokens de razonamiento | 0 |
| HTTP p50/p95 | 1878,95/2631,83 ms |
| Pase de calidad | No |

Los tiempos son desde la petición HTTP hasta su lectura/validación, no recuperación completa ni entrega a Codex. No hay comparación contemporánea de recursos con baseline, estabilidad o resultado de producto. La tarifa declarada por el catálogo no certifica cuantización/pesos servidos inmutables; se observaron el alias solicitado y DeepInfra en los 16 recibos.

La confusión completa es A→A:4, A→C:1, A→none:5, B→B:1, B→none:3, C→none:3, none→A:2, none→B:1 y none→none:44. El único A→C conserva la selección, pero atribuye un rol incorrecto. Los cinco A→none, tres B→none y tres C→none explican las once pérdidas. No se atribuyen a transporte, formato inválido o truncamiento: las 16 respuestas finalizaron correctamente.

Gemini V2 histórico obtuvo 50/64 roles, conservó 14/17 útiles y añadió 10 ajenos. Qwen reduce esos ajenos a 3, pero pierde once útiles: **no es mejora aprobable**, aunque una dimensión aislada suba. Es comparación descriptiva de modelos/proveedores sobre desarrollo conocido, no contraste contemporáneo del producto. Gemma tuvo solo 8 juicios válidos antes del 429; no se compara su porcentaje parcial con los 64 de Qwen.

## Coste completo de esta continuación

Se iniciaron **20 inferencias**: primer intento Qwen 1, segundo intento 16 y 3 sondas neutras. Coste nuevo conocido: **USD 0,000876600**, más **una petición de importe desconocido** por el primer 429. Las sondas costaron USD 0,000053450; no se declaran gratuitas ni como benchmark. El presupuesto adicional del segundo intento fue USD 0,01, estimación conservadora USD 0,006819. La estimación no sustituye recibos ni cubre un importe que el proveedor no informó.

Acumulado externo: **6152 operaciones, USD 4,566326478 conocidos y 22 operaciones de importe desconocido**. Siguen siendo 23 campañas emparejadas completas; esta tanda no añade una. El historial anterior 6132/USD 4,565449878/+21 se conserva íntegro. Coste/tokens Codex de desarrollo y críticas no cuantificados; no es coste total de trabajo.

## Decisión, verificación y continuación

El cambio de modelo sin modificar el contrato no resuelve el filtro. Se preservan los 15 errores y se requiere revisar sus causas antes de otra inversión; no añadir ejemplos del banco ni repetir la misma salida para escoger una corrida favorable. La [revisión de fallos](qwen9b-classifier-failure-review-2026-10-22.md) separa problemas observables de capacidad/idioma/seguimiento de reglas, causas internas todavía no aisladas. No se modifica gold para rescatar resultados.

La candidata documental permanece congelada como fallida con 64/116 completas y fieles, all116 intacto y propuesta nominal diferida. Las puertas generales del objetivo no cambian: calidad sin ajenos, pérdidas útiles/completitud o degradación nueva, al menos una ganancia; recibos y recursos completos ≤1,5× base, estabilidad y después cohorte ciega. V6 no se abrió. No cambia default, dashboard, DB, dependencias, credenciales, servicios, README/gráficas de rendimiento, rama o entrega Git. No hay permisos pendientes.

Verificación final: 871 pruebas default y 872 con HTTP/2, cero fallos, 24/73 omitidas respectivamente; formato correcto. Quince informes de contribución —incluyendo ambos intentos Qwen y el Gemma anterior— se reconstruyen byte por byte idénticos. Las 425 reproducciones documentales son evidencia histórica, no se repitieron en esta fase. Pruebas/replays no son métricas de calidad.

Artefactos en `outputs/iromem/evaluation/contribution-qwen9b-minimal-{v1,attempt2}-2026-10-22` y `openrouter-classifier-429-diagnosis-2026-10-22`: protocolos, requests, entradas, hashes/código/binario, diarios, recibos, recursos HTTP, presupuestos y compatibilidad. [Protocolo original](qwen9b-classifier-v1-protocol-2026-10-22.md), [protocolo explícito del segundo intento](qwen9b-classifier-recovery-protocol-2026-10-22.md).
