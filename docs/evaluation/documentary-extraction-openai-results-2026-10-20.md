# Extracción documental: pilotos de contrato y formato con OpenAI

**Ninguna variante pasa calidad; el objetivo continúa en el paso 3.** Se implementó y ejecutó un contraste de modelo, se corrigió una incompatibilidad de parámetros y se probaron restricciones de esquema/guía y entrega JSON. Mejorar la forma del DTO no demostró fidelidad. El 100/100 manual conocido sigue siendo manual; no hay mejora automática de recuperación demostrada ni adopción.

## Diseño y resultados

Mismo piloto posicional de cuatro fuentes y cuatro preguntas, repartido por el banco conocido, sin selección por etiquetas o resultados. Fuente y pregunta se interpretan por separado; se envían únicamente texto y ontología de dominio, más esquema donde corresponda. No se envían frames manuales, gold, resultados ni catálogo de entidades. Cada versión tiene protocolo, preparación, código ejecutado, diario y recibos propios; máximo cuatro concurrentes, sin reintentos. V6 reservada no se abrió ni recibió llamadas.

| Variante | Cambio explícito | HTTP terminados / previstos | DTO completos: fuentes / preguntas | Fieles según ambos críticos / 8 | Puerta |
| --- | --- | ---: | ---: | ---: | --- |
| OpenAI V3 | Payload nativo anterior, cambiando modelo/ruta a GPT‑4.1 Mini/OpenAI | 4/8; cuatro 404 | 0 / 0 | Sin contenido de modelo | Falla técnica |
| OpenAI V4 | Retirar únicamente `reasoning.enabled=false`, no admitido por la ruta | 8/8; todos 200 | 0 / 0 | 2 | Falla |
| Guiada V5 nativa | Mismo modelo/roster; esquema más restrictivo y aclaraciones generales del contrato | 8/8; todos 200 | 4 / 1 | 2 | Falla |
| Guiada JSON J1 | Mismo esquema/guía V5, moviendo el esquema al mensaje con JSON object | 8/8; todos 200 | 2 / 1 | 1 | Falla |

Los cuatro 404 de V3 identifican `Filter by Parameters`. El catálogo no declara `reasoning`; V4, idéntica salvo retirar ese campo desactivado, llega al modelo. Se añadió comprobación local de parámetros presentes contra el catálogo antes de enviar, con pruebas negativas. No se reintentó ni sobrescribió V3. Los 24 éxitos posteriores sirvieron el alias `openai/gpt-4.1-mini`, proveedor `OpenAI`, por HTTP/2; el alias no verifica por sí solo una revisión fechada.

Los tres pilotos con contenido producen ocho JSON que pasan su esquema de forma, pero varios fallan el contrato adicional de dominio. V5 restringe campos de medición a `measurement`, exige dato o conjunto exhaustivo —no ambos— y al menos una lectura. Se conservan parser y degradación; no se arreglan respuestas con gold. El salto **0→5/8 DTO completos** mide cumplimiento de contrato, no una mejora de calidad o recuperación. V5 cambia guía y esquema juntos: no aísla cuál causa una diferencia.

## Fidelidad: la puerta que impide una falsa mejora

Dos críticos revisaron cada salida de los tres pilotos con contenido, sin etiquetas, frames manuales, roles esperados ni resultados de ablaciones. V4 y V5 tuvieron contextos nuevos; los revisores de J1 conocían V5 del mismo piloto y lo declararon. Es revisión del mismo sistema de modelos, no humana ni validación de una cohorte nueva.

| Variante | Primer crítico: fiel / defecto / sin frame | Segundo crítico: fiel / defecto / sin frame | Coincidencias y discrepancias |
| --- | --- | --- | --- |
| V4 | 3 / 5 / 0 | 2 / 6 / 0 | 2 fieles, 5 defectos coincidentes, 1 discrepancia |
| V5 | 2 / 5 / 1 | 3 / 4 / 1 | 2 fieles, 4 defectos y 1 frame ausente coincidentes, 1 discrepancia |
| J1 | 1 / 7 / 0 | 1 / 7 / 0 | 1 fiel y 7 defectos coincidentes |

Se conservaron ambas críticas y las discrepancias sin recalificar salidas. Un contenido juzgado fiel puede seguir siendo inválido por contrato; un DTO válido puede contener un hecho falso. V5 devolvió `status=ok` con un frame nulo: es degradación, no abstención fiable. No hay ocho extracciones fieles en ningún intento, por lo que no se amplía a 116 ni se abre V6.

Ejemplo determinante: «El formulario del pedido O7 de Darsa **no exige** teléfono de contacto». V5 devuelve `requires`, `positive=true` y conjunto exhaustivo `contact_phone`: invierte la negación pese a pasar el contrato estructural. J1 conserva `positive=false`, pero usa `datum.text=no`: pierde el objeto teléfono y representa una negación de `requires(no)`. Ninguna versión pasa esa fuente. También persisten preguntas numéricas convertidas en medición/máximo y alcance de medición inventado. La cita literal no corrige estos cambios de significado.

Las cuatro ablaciones conservan el denominador de 100 pares, aunque el piloto cubra pocos:

| Variante | Control manual: coincidencias / 100 | Auto fuente: coincidencias / degradados | Auto pregunta: coincidencias / degradados | Auto ambos: coincidencias / degradados |
| --- | ---: | --- | --- | --- |
| V3 | 100 | 0 / 100 | 0 / 100 | 0 / 100 |
| V4 | 100 | 0 / 100 | 0 / 100 | 0 / 100 |
| V5 | 100 | 3 / 96 | 0 / 98 | 1 / 99 |
| J1 | 100 | 2 / 98 | 4 / 96 | 1 / 99 |

Son coincidencias de límites sobre un piloto incompleto, no precisión de un buscador operativo. Los controles manuales reproducen 100/100; la coincidencia automática en un `none` no demuestra fidelidad de la fuente. Los DTO ausentes o inválidos permanecen degradados; cero selecciones ajenas computadas con esa cobertura no es un pase. Los informes conservan errores de rol, hechos, límites y pérdidas por separado.

También hay un límite de interfaz: la pregunta de Atlas de V5 es fiel según ambos críticos, pero usa `expediente Atlas` donde el frame manual usa `Atlas`. La coincidencia literal de claves puede perder evidencia aunque el referente se conserve. Ese fallo de alineación no se llama falsedad semántica ni se corrige con aliases obtenidos del gold; una futura representación debe separar nombre, tipo y propietario y normalizarlos mediante reglas independientes del banco.

## Coste y tiempos observados

| Variante | USD conocidos | Importes desconocidos | Duración runner | Fuente p50 / p95 ms | Pregunta p50 / p95 ms |
| --- | ---: | ---: | ---: | --- | --- |
| V3 | Sin recibos | 4 | 0,667 s | 601,225 / 644,155 | 603,242 / 618,773 |
| V4 | 0,006513200 | 0 | 4,995 s | 1975,484 / 2345,319 | 2140,859 / 2617,956 |
| V5 | 0,009380400 | 0 | 6,462 s | 2277,532 / 4147,182 | 1626,686 / 2221,043 |
| J1 | 0,011660400 | 0 | 5,469 s | 1664,606 / 2242,192 | 1624,965 / 3173,888 |

Tanda completa: **28 operaciones / 28 HTTP, USD 0,027554000 conocidos y cuatro importes sin confirmar**. El total exacto proviene de sumar recibos una vez; no se infiere gratuidad de los 404. Fuentes: USD 0,014460000; preguntas: USD 0,013094000. Los éxitos informan 110.101 tokens de entrada y 4.072 de salida del proveedor, no tokens enviados a Codex. Hay caché de entrada informada por el proveedor; se usa su coste real, no el nominal sin caché ni una garantía de ahorro futuro.

Las latencias son de peticiones y duración del runner, incluyen errores terminados, y no miden recall ni recorrido de producto. Son intentos únicos sin prueba de estabilidad. Los costes/tiempos del piloto no pasan el gate de recursos contra baseline ni demuestran amortización de indexación.

Acumulado del objetivo: **5969 operaciones / 23 campañas emparejadas completas, USD 0,630117308 conocidos más 16 importes desconocidos**, sin Codex. Los pilotos no añaden campañas completas.

## Verificación y siguiente mecanismo

Pasan **382 pruebas** en configuración predeterminada y HTTP/2, sin avisos, además de formato/diff. Se reproducen byte a byte raw/score de V1, V4, V5 y known100 manuales y los seis informes de extracción, incluidos los fallidos. Se verifican banco, preparación, código ejecutado, diarios, recibos y snapshots anteriores. No hay servicios, puertos, nuevas credenciales, dependencias o SQL; baseline, dashboard y gold quedan conservados. El harness registra fallos y puede terminar `ok`: la puerta experimental sigue fallida.

El próximo mecanismo debe simplificar la representación generada, en vez de añadir más campos opcionales o relajar el validador: primero cláusulas atómicas con cita y tipo de hecho, después estructuras específicas por tipo —requisito con objeto/polaridad, cantidad, procedimiento, asignación, flujo o estado documental— y traducción determinista al contrato V5. Debe admitir varias cláusulas/tipos por fuente para conservar parciales y refutaciones; una clasificación exclusiva de todo el recuerdo podría perderlas. Preguntas seguirán independientes de fuentes. Esa hipótesis está **pendiente de implementar y medir**; no se presenta como solución lograda. Mantendrá puerta completa de fidelidad antes de recursos, banco difícil y casos nuevos.

Protocolos: [V3](documentary-extraction-openai-v3-protocol-2026-10-20.md), [V4](documentary-extraction-openai-v4-protocol-2026-10-20.md), [V5](documentary-extraction-openai-guided-v5-protocol-2026-10-20.md), [J1](documentary-extraction-openai-guided-json-v1-protocol-2026-10-20.md). Artefactos: [V4](../../outputs/iromem/evaluation/documentary-extraction-openai-v4-2026-10-20/report.json), [V5](../../outputs/iromem/evaluation/documentary-extraction-openai-guided-v5-2026-10-20/report.json), [J1](../../outputs/iromem/evaluation/documentary-extraction-openai-guided-json-v1-2026-10-20/report.json), [verificación](../../outputs/iromem/evaluation/documentary-extraction-openai-guided-json-v1-2026-10-20/verification.json), [contabilidad del objetivo](../../outputs/iromem/evaluation/quality-goal-2026-10-19/index.json).
