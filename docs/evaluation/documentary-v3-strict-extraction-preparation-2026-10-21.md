# Preparación de la extracción V3 con dominios explícitos

Se preparó una nueva tanda completa de desarrollo conocido, sin ejecutarla: 83fuentes y33preguntas, exactamente los116textos de la extracción congelada anterior. El objetivo es medir si la guía revisada y el contrato actual permiten conservar las restricciones antes de volver a medir utilidad. El banco no se reduce a las entradas previamente aceptadas ni a los tres fallos manuales restantes.

## Protocolo prospectivo

Diez paquetes independientes: siete lotes fuente de12/12/12/12/12/12/11y tres lotes pregunta de11. Se barajó cada clase con semilla fija ligada al manifiesto86del objetivo; los IDs opacos y el roster privado se separan. Cada paquete contiene solamente texto/ID/hash, vocabulario, notación, guía general y código del validador sin pruebas. No contiene raws anteriores, etiquetas, pares, críticas ni métricas. No se suministra la narrativa histórica del contrato.

La guíaV3 incorpora la guíaV2y el dominio obligatorio de cada cierre, evidencia local de cada cláusula, distinción entre uso y perfil, clases explícitas de configuraciones y conservación de finalidad/procedencia. No incluye ejemplos del corpus ni respuestas. Se liga por hash al vocabulario estricto actual. Cambian guía, vocabulario/contrato y orden de textos; una comparación posterior no aislará causalmente un único cambio.

Se exige contexto nuevo para cada extracción y cada crítica. Una entrega por lote, sin validación ni corrección tras feedback. El padre conserva bytes originales, fallos y ausencias y valida con Rust. Dos críticos separados por lote reciben texto/raw/validación, sin el resultado del otro. La revisión compara fidelidad textual, completitud, atribución y preservación operativa; cada juicio necesita comprobación afirmativa y cada nofaithful una cita y un pointer existente. Son contextos de una misma familia, no validación humana ni independencia de familia.

La puerta permanece sin rebajar:116entradas completas y464dimensiones faithful por crítico. Un fallo conserva toda la tanda y bloquea utilidad automática. Después se necesitan conservación global de evidencia útil, pares difíciles, baseline, recursos/estabilidad y la cohorte ciegaV6. Ningún pase de preparación, esquema o hash sustituye esas puertas.

## Auditoría de preparación

Se verificaron10paquetes,116textos y la igualdad de inventario por tipo/hash contra el banco anterior. Se rechazaron seis controles negativos: omisión de texto, ID duplicado, etiqueta inyectada, hash incorrecto del vocabulario, texto cambiado y guía ligada a un vocabulario antiguo. Un control manual ajeno a los116textos validó completo en Rust y confirmó la serialización canónica del vocabulario.

Se preservaron dos fallos de preparación antes de cualquier inferencia: hash calculado en orden de inserción en vez de claves ordenadas y extensión de una lista de notación como caracteres. Ambos fueron corregidos y auditados; las versiones fallidas permanecen en `preparation-attempt-1` y `preparation-attempt-2`. No se reparó ni reevaluó una salida de modelo.

La preparación no ejecutó extracciones, críticas ni clasificaciones de los116textos. No se reactivaron agentes, llamó a proveedores, abrióV6, modificó producto ni repitió la suite Rust. Código/binario idénticos a la fase anterior con613pruebas/24omitidas. Sigue vigente el resultado automático112/116estructuralmente completas y65/116aceptadas por ambos críticos; las selecciones manuales36/39siguen históricas, no un resultado nuevo.

## Ejecución pendiente y coste

Se solicitó aclarar autorización para abrir nuevos subagentes después de la instrucción del usuario de cerrar los anteriores. La tanda está preparada, no despachada: diez tareas de extracción y veinte críticas, estimación30–60min. Ninguna espera de conversación se registra como proceso vivo o ejecución en marcha.

API nuevaUSD0y cero peticiones. Tokens/coste Codex desconocidos. Acumulado de proveedor sin cambios:6129operaciones lógicas,23campañas emparejadas completas,USD4,565416508conocidos y20operaciones con coste desconocido. No hay nueva métrica de calidad ni latencia de extracción/producción.

El objetivo permanece activo en el paso3. La siguiente acción dependiente es ejecutar los paquetes congelados en contextos nuevos y conservar íntegros sus resultados, antes de construir las críticas. Artefactos en `outputs/iromem/evaluation/documentary-v3-strict-extraction-2026-10-21/`.
