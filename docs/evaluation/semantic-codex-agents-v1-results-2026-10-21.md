# Evaluación con subagentes Codex: acuerdo conocido tras arbitraje

Paso 3 del objetivo activo. **Dos agentes evaluaron los 46 inputs sin referencias ni resultados previos. Cada uno conserva los 35 controles fieles y bloquea los 11 no aceptables, aunque cada uno discrepa en una dimensión. Un árbitro posterior resuelve los dos desacuerdos y la salida derivada concuerda en 184/184 dimensiones con las referencias congeladas. No hay prueba de extracción, recuperación o latencia de producción.** [Reporte Rust](../../outputs/iromem/evaluation/semantic-codex-agents-v1-2026-10-21/report.json).

## Método y exposición

Se usa un paquete canónico común con contrato, rúbrica, checklist F1 e inputs raw/normalized íntegros. No se incluyen etiquetas de referencia, resultados anteriores, identidad de proveedores o scores. Cada agente parte sin historial del chat y recibe solo su paquete. El autor evalúa y el crítico independiente intenta detectar defectos sin inventarlos; ninguno ve las respuestas del otro al generar su salida. Ambos validan localmente forma/citas/rutas antes de entregarla: es trabajo iterativo de agentes, no una inferencia HTTP única.

La presentación difiere de los pilotos HTTP: árboles canónicos completos, contrato común una sola vez y paths canónicos. El archivo preparado mide 211.771 bytes; no se convierte ese tamaño en tokens, ahorro monetario o tráfico real de Codex. No es una ablación exclusiva de modelo o transporte.

La regla original acepta solo si ambos agentes aceptan las cuatro dimensiones y registra desacuerdos sin escoger respuestas según gold. Después de ver dos desacuerdos se define una etapa arbitral separada: se incluyen todas y solo las dimensiones con labels distintos, con orden A/B alternado y sin referencia/score. El árbitro ve propuestas, input y contrato. El reporte reconstruye esa selección para impedir inclusión o exclusión por conveniencia. No se alteran respuestas individuales ni etiquetas para mejorar cifras.

## Métricas del banco conocido

| Medida | Autor | Crítico | Derivado tras árbitro |
| --- | --- | --- | --- |
| Inputs válidos | 46/46 | 46/46 | 46/46 |
| Dimensiones concordantes | 183/184 | 183/184 | 184/184 |
| Aceptaciones indebidas | 0 | 0 | 0 |
| Rechazos de controles fieles | 0 | 0 | 0 |
| Controles fieles conservados | 35/35 | 35/35 | 35/35 |
| No aceptables bloqueados | 11/11 | 11/11 | 11/11 |

Las once construcciones no aceptables incluyen nueve defectuosas y dos inciertas. Las referencias no son verdad externa verificada. El autor aprueba indebidamente normalized_preservation en s019; el crítico la rechaza indebidamente en s013. Los dos errores permanecen en sus archivos y métricas. El árbitro distingue preservación fiel de una equivalencia de clase falsa en s013 frente a pérdida operativa de «del norte» en s019. [Paquete de desacuerdos](../../outputs/iromem/evaluation/semantic-codex-agents-v1-2026-10-21/adjudication-packet.json) y [resoluciones](../../outputs/iromem/evaluation/semantic-codex-agents-v1-2026-10-21/agent-arbiter.json).

La puerta original por evaluador sigue fallida —un error cada uno—. El 184/184 corresponde a una composición posterior conocida, no repara la primera fase ni prueba generalización. Se añade un crítico de razones que ve predicciones, pero no referencias, score o identidad: revisa los 184 juicios derivados y 80 del piloto HTTP anterior. No encuentra objeciones en los 46 inputs derivados y localiza dos problemas de fidelidad/atribución en el HTTP s019. Esta es revisión con referencia oculta, no predicción ciega ni validación humana. [Dictamen](../../outputs/iromem/evaluation/semantic-codex-agents-v1-2026-10-21/rationale-critical-review.json), [conciliación](../../outputs/iromem/evaluation/semantic-codex-agents-v1-2026-10-21/rationale-review-reconciliation.json). El reporte cuantitativo se mantiene separado de este dictamen y conserva su indicador de revisión no incorporada.

## Verificación y trazabilidad

`report-semantic-agents` verifica los 46 inputs contra el paquete canónico congelado, el contrato/checklist público y la conciliación de referencia anterior. Usa checked V1 sin cambios para juicios, citas literales, todos sus offsets UTF8 y paths; rechaza duplicados, omisiones, IDs ajenos, exposición declarada incompatible y resoluciones ajenas. Cierra los campos del paquete arbitral. No es un transporte ni cambia la política del producto.

El revisor estático detectó falta de cierre del paquete arbitral y de vínculo entrada/salida por agente. Se cerró el paquete y cada agente creó un sidecar con hashes de su input y salida, sin reescribir juicios. **Estos vínculos se registraron después de las salidas: no prueban criptográficamente el contexto previo ni ausencia efectiva de exposición.** El reporte declara pre_inference_output_binding_proven=false; no se oculta esta limitación. Futuros encargos deberán fijar ese vínculo en el protocolo inicial. La revisión estática posterior acepta el reporte con ese límite; no es evidencia humana de calidad.

Pasan 510 pruebas con y sin HTTP2. Los CLI negativos rechazan salida alterada, cobertura incompleta/duplicada, path inventado y contenido extra de referencia aunque se recalculen sus hashes. Cuatro bancos manuales, diez informes de extracción y nueve auditores/reportes anteriores se reproducen idénticos, incluido el piloto F1 interrumpido. El reporte actual también se reproduce en bytes. [Verificación](../../outputs/iromem/evaluation/semantic-codex-agents-v1-2026-10-21/verification.json).

## Coste, política vigente y siguiente puerta

Este flujo hizo **cero nuevas peticiones a OpenRouter**. No hay recibos de dinero/tokens de los subagentes ni medición de latencia del producto; no se los declara gratuitos ni se comparan sus tiempos con recall. La política actual autoriza API para Decisions, Jev y otros modelos de clasificación y comparativas; para LLM potentes/de frontera se usan subagentes Codex. El usuario informó que amplió el límite de la clave, sin indicar importe; no se verificó aquí su saldo porque no hubo nuevas llamadas. No se modificaron credenciales. El runner histórico de estos pilotos LLM por HTTP se detiene antes de cargar la clave; los runners de clasificación permanecen disponibles.

La ronda HTTP F1 anterior queda cerrada por separado: 24 operaciones intentadas, 20 recibos conocidos/USD 1,045548, cuatro HTTP403 sin recibo y 22 no enviadas. No se reanuda ni se repite con Opus. Acumulado proveedor: **6.129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos más 20 operaciones históricas de coste desconocido**. El contador de clave reportado por OpenRouter no es el libro de este chat. Codex queda fuera de esa cuenta por falta de recibos comparables.

El próximo paso es extraer representaciones desde texto sin frames/gold mediante subagentes Codex y comprobarlas con el normalizador real y un crítico separado. Después deben superarse tubería completa, conocidos100/126, comparación con clasificación vigente, recursos/estabilidad y cohorte nueva ciega. El acuerdo en construcciones manuales conocidas no sustituye esas puertas. Baseline, SQL, dashboard, servicios, README/gráficas y V6 se conservan. Goal activo; sin promoción.
