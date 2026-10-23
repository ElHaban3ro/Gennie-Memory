# Banco completo Codex entry_via V1: resultado y diagnóstico

La ampliación del piloto de ocho textos a las 116 entradas conocidas **no pasa la puerta completa**. Se implementó y verificó el adaptador del banco; la extracción sigue perdiendo restricciones o encontrando límites de representación. No se modifica la política vigente ni se declara una mejora de recuperación. Goal activo, paso 3.

## Métricas completas

| Medida | Fuentes | Preguntas | Total |
| --- | ---: | ---: | ---: |
| Entradas previstas y entregadas | 83 | 33 | 116 |
| DTO completos hasta normalización | 72 | 30 | 102/116 (87,93%) |
| Cuatro dimensiones faithful para ambos críticos originales | 66 | 27 | 93/116 (80,17%) |

Las 116 representaciones corresponden a 100 pares del banco de desarrollo conocido. No son 116 consultas de recuperación ni una cohorte nueva. Cada una conserva raw, normalizado o error; no se descartan las 14 que no normalizan. Los diez lotes tienen entregas vinculadas y cobertura completa. Hay ocho errores de traducción grounded y seis de normalización tipada; cinco de estos últimos son declaraciones explícitas de unsupported.

| Familia de críticos | Faithful | Defective | Uncertain | Unavailable | Juicios previstos |
| --- | ---: | ---: | ---: | ---: | ---: |
| A | 421 | 23 | 4 | 16 | 464 |
| B | 424 | 13 | 7 | 20 | 464 |

Se completan 20 revisiones, cuatro dimensiones por entrada para cada familia: 928 juicios en total. Estos porcentajes describen aceptación por críticos del mismo sistema de modelos; **no son exactitud contrastada con verdad independiente**, precisión ni recall. La puerta congelada exigía 116 DTO y 464 juicios faithful de cada familia. Falla sin sustituirla por otra condición.

El piloto anterior pasó ocho de ocho textos y 32/32 juicios para cada crítico. El banco mayor demuestra que ese pase no se generaliza a todo el desarrollo. No es una comparación de rendimiento entre dos políticas de recuperación: cambian el tamaño y la composición del conjunto evaluado.

## Qué falla y qué aprendimos

Los límites comprobados por el normalizador incluyen propietarios anónimos con cardinalidad, referencias anónimas sin identidad operativa, tipos por miembro de un conjunto cerrado y una premisa de medición sin método especificado. En este último caso, la extensión de medición genérica se acepta en otras rutas, pero la premisa de pregunta sigue pasando por el parser tipado previo; no se elimina la premisa ni se inventa un método para recuperar la fila. La suficiencia de una credencial tampoco equivale a una lista exhaustiva de requisitos: la abstención conserva esa limitación, pero no proporciona una extracción completa.

Las revisiones señalan además restricciones omitidas: clase tutor solicitada y unidad de respuesta, propiedad inferida desde ubicación, clase de relación añadida por atributos como agua/programa y existencia de un objeto convertida en conteo general. Las referencias a otro predicado no deben tratarse automáticamente como contexto inútil. El [inventario](../../outputs/iromem/evaluation/codex-entry-bank-v1-2026-10-21/failure-inventory.json) separa errores mecánicos, límites del contrato, diagnósticos semánticos y sus incertidumbres; no introduce nuevas etiquetas.

Los 29 desacuerdos de etiqueta reciben arbitraje retrospectivo, todos y solo ellos, repartidos determinísticamente con A/B alternado y sin gold, referencias manuales o métricas. El diagnóstico termina en 14 defective, siete faithful y ocho unavailable. No reemplaza juicios originales ni recalcula el pase de 93 entradas. También cuestiona hallazgos de los críticos: «el reproductor Lino» sí es substring literal de «del reproductor Lino»; una colección íntegra puede ser una entidad singular; preservar un raw defectuoso no significa que el normalizador haya creado ese defecto. Una abstención honesta no es una afirmación factual falsa, pero tampoco una extracción disponible y completa. La crítica automatizada también requiere evaluación.

La [auditoría independiente de a028](manual-reference-source-audit-v1-results-2026-10-21.md) cuestiona la relación water_supply de una referencia manual y deja incertidumbre sobre el ID del portador. Es un diagnóstico retrospectivo de un caso, no una tasa de error del banco. El 100/100 manual histórico se conserva como concordancia con anotaciones, sin convertirlo en prueba autónoma de fidelidad. Referencias, gold y métricas anteriores permanecen intactos.

## Implementación, verificación y límites

El módulo `automatic_extraction/codex/bank.rs` prepara/reconstruye todos los paquetes, verifica el pase previo desde sus entregas y críticas reales y conserva las 116 filas. Solo añade dos comandos de evaluación; no cambia servicios, dependencias, SQL o runtime de producto. Una envoltura no vinculable degrada todo el lote; el error de contenido afecta su propia fila. El raw no se repara ni se escogen respuestas por gold.

Pasan 520 pruebas offline de workspace tanto por defecto como con HTTP/2 y 13 comprobaciones CLI adversarias. Cuatro bancos manuales, diez informes de extracción, seis informes semánticos, tres auditores locales, ambos pilotos Codex y su informe semántico se reproducen sin diferencias. La revisión estática inicial identificó que el pase del piloto podía depender de contadores/hashes incompletos; se corrigió antes de inferencia y la revisión posterior no dejó bloqueadores. Permanece el límite de que los valores scope/normalized_contract del paquete de revisión del piloto se exigen presentes pero no se reconstruyen; sus bytes reales están congelados y preservados.

Se archivan diez fuentes y hashes del ejecutable antes de extraer, sin cambios posteriores de ese código/binario. Es un archivo parcial: los hashes y las declaraciones de exposición no prueban criptográficamente el contexto efectivo del modelo. Los agentes reciben paquetes aislados y no usan el normalizador antes de entregar. La revisión ve texto/raw/normalizado, no gold ni frames manuales. No hay evaluación humana o externa.

Se conserva el fallo inicial de compilación y su arreglo. La conciliación aceptaba inicialmente solo claves path/quote, mientras tres revisores recibieron una instrucción sin nombres exactos y devolvieron pointer/source_quote. Se registra la diferencia y se admiten ambas formas con las mismas exigencias de cita/puntero, sin editar un juicio. El replay V1 usa su resultado corregido histórico normalization-fixed.json; el resultado inicial defectuoso también permanece archivado.

## Coste y siguiente puerta

**Cero llamadas OpenRouter nuevas, USD 0 de coste externo nuevo observado.** Coste y tokens de las tareas Codex desconocidos; no se presentan como gratuitos. Acumulado sin cambios: 6129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos más 20 operaciones con importe desconocido, excluyendo Codex. Estas tareas no se cuentan como peticiones HTTP de proveedor.

No se midieron recall, precisión, latencia end-to-end, recursos ni estabilidad. V6 sigue reservada: solo se verificó el hash de su manifiesto, sin abrir el corpus ni enviar llamadas.

La siguiente iteración debe versionar la representación para preservar restricciones y evidencia útil parcial —suficiencia, referencias anónimas/cardinalidad, unidades y premisas sin método— y separar con claridad los defectos del raw de los introducidos por la proyección o por la revisión. Antes de otra extracción completa, exige contraejemplos prospectivos y revisión del contrato; no memorizar estos textos ni cambiar sus etiquetas para aprobar. Después siguen la conservación de utilidad, los 126 pares conocidos, comparación real con baseline y recursos/estabilidad. Los casos nuevos y la decisión de adopción siguen pendientes.

[Reporte estructurado](../../outputs/iromem/evaluation/codex-entry-bank-v1-2026-10-21/report.json), [revisión completa](../../outputs/iromem/evaluation/codex-entry-bank-v1-2026-10-21/review-reconciliation.json), [arbitraje diagnóstico](../../outputs/iromem/evaluation/codex-entry-bank-v1-2026-10-21/arbitration-reconciliation.json), [verificación](../../outputs/iromem/evaluation/codex-entry-bank-v1-2026-10-21/verification.json), [protocolo congelado](codex-entry-bank-v1-protocol-2026-10-21.md).
