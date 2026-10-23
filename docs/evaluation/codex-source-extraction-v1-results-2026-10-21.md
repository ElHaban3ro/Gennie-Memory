# Extracción desde texto con subagentes Codex V1

**El piloto logra siete extracciones válidas y fieles según ambos críticos de ocho previstas. No pasa la puerta completa y el objetivo sigue en el paso 3.** La ausencia restante revela una relación que el contrato no distingue suficientemente: entrar por una tubería no acredita por sí solo proveedor inmediato ni origen aguas arriba. No se fuerza una interpretación para conseguir ocho aciertos.

## Qué se ejecutó

[Protocolo](codex-source-extraction-v1-protocol-2026-10-21.md) fijado antes de las entregas, mismos ocho textos, ontología, instrucciones y esquemas del piloto grounded-reference conocido. Un agente nuevo recibe cuatro fuentes; otro, cuatro preguntas. No reciben gold, frames manuales, respuestas anteriores, el otro lado emparejado ni feedback del normalizador. Cada entrega incluye hash del paquete; se verifica igualdad con los datos públicos históricos. Comparten contexto dentro de cada etapa: no equivale a ocho inferencias HTTP aisladas. Los hashes y declaraciones no prueban criptográficamente el contexto efectivo del modelo.

Se implementó `normalize-codex-extraction` en el evaluador Rust. Aplica el esquema grounded con referencias locales, traductor y parser V5 existentes, conserva las ocho filas y registra errores sin reparar raw. Dos críticos nuevos revisan las ocho extracciones y normalizados con referencias/manuales/resultados ocultos. Cuatro dimensiones por entrada: fidelidad, completitud, atribución y preservación operativa. Un árbitro posterior diagnostica el único desacuerdo; no relaja la puerta fijada de ambos críticos completamente faithful.

## Métricas y límite

| Medida | Resultado |
| --- | ---: |
| Entregas previstas / recibidas | 8 / 8 |
| Fuentes con DTO completo | 3 / 4 |
| Preguntas con DTO completo | 4 / 4 |
| DTO completos totales | 7 / 8 |
| Entradas faithful en las cuatro dimensiones para ambos críticos | 7 / 8 |
| Primer crítico: faithful / defective / unavailable | 28 / 1 / 3 juicios |
| Segundo crítico: faithful / unavailable | 28 / 4 juicios |
| Desacuerdos de dimensión | 1 / 32 |
| Puerta del piloto / objetivo completo | Falla / pendiente |

Las siete entradas completas no tuvieron objeciones de ambos críticos. Esto es revisión del mismo sistema de modelos sobre desarrollo conocido, no verdad externa, validación humana, recall ni generalización. No hay selecciones de IDs medidas ni comparación nueva contra baseline. No se ejecutan conocidos100/126 o V6 después del fallo.

`op_054` conserva `unsupported`: «La red del huerto Verbena entra por la tubería T2». El esquema obliga a elegir immediate/upstream para flow; el extractor evita afirmar una relación más fuerte. Ambos críticos consideran ausente la extracción; difieren entre defective y unavailable para completitud. El árbitro decide defective por pérdida de cobertura y considera razonable la abstención. Propone comprobar prospectivamente una relación de trayecto/conexión, sin origen ni exclusividad implícitos. No se añaden aliases ni se cambian gold o salidas.

El piloto HTTP grounded-reference histórico tuvo cuatro DTO completos y una entrada faithful para ambos críticos con su rúbrica anterior. Ahora hay siete y siete. Son datos diagnósticos del mismo banco; cambia la ejecución mediante tareas Codex, el contexto compartido y la revisión por dimensiones. No es una ablación exclusiva de modelo ni una mejora porcentual comparable de recuperación.

## Fallos de integración y revisión conservados

Mi adaptador inicial llamó al helper de schema nativo, que no resuelve `$ref`, trata opcionales como obligatorios y rechaza números. Produjo cero DTO válidos. Un revisor estático confirmó el fallo. Se conserva ese resultado y su diagnóstico; el código inicial completo no quedó archivado. Lo corregí con validación local del vocabulario del esquema grounded, manteniendo intactos el helper histórico, traductor y parser. No se modificaron entregas de los agentes. La repetición produce siete DTO válidos.

El árbitro inicial usó dos quotes tomadas del raw en vez del texto fuente. Se conservó su salida y una segunda entrega corrigió solo esas dos citas. Verificación mecánica confirma el mismo label, razón, paths y diagnóstico; no mejora el juicio ni aprueba el piloto. Los dos dictámenes originales de críticos pasan cobertura, paths y literalidad sin reparación.

Pasan **514 pruebas offline** con y sin HTTP2, incluidas integración positiva de fuente/pregunta, polaridad negativa y nombre inventado rechazado. Diez comprobaciones CLI cubren replay idéntico, no sobrescritura, hash incorrecto, omisiones, duplicados, ID ajeno, exposición a gold, campo extra, esquema alterado y degradación por referencia no literal. Cuatro bancos manuales, diez reportes de extracción y nueve auditorías/reportes previos se reproducen idénticos. El snapshot065 y anteriores permanecen intactos; V6 solo se comprueba por hash de manifiesto.

## Coste y continuación

**Cero llamadas OpenRouter nuevas.** Coste y tokens de las tareas Codex desconocidos; no se declaran gratuitas. Sin medición de latencia de recall o producto. El acumulado proveedor permanece en **6129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos más 20 operaciones de coste desconocido**, excluyendo Codex.

Siguiente acción: especificar y someter a contraejemplos la diferencia entre trayecto de entrada, proveedor inmediato y origen aguas arriba, sin borrar los límites anteriores. Solo si preserva restricciones y utilidad se prepara otra versión de extracción con protocolo previo. Siguen pendientes extracción completa, conocidos100/126, comparación baseline, recursos/estabilidad y cohorte nueva ciega. APIs de clasificación siguen autorizadas; modelos frontera mediante Codex. Sin cambio de producto, configuración predeterminada, SQL, dashboard, dependencias, credenciales, servicios, README o gráficas de recuperación.

[Normalización](../../outputs/iromem/evaluation/codex-source-extraction-v1-2026-10-21/normalization-fixed.json), [revisión conciliada](../../outputs/iromem/evaluation/codex-source-extraction-v1-2026-10-21/review-reconciliation.json) y [verificación](../../outputs/iromem/evaluation/codex-source-extraction-v1-2026-10-21/verification.json).
