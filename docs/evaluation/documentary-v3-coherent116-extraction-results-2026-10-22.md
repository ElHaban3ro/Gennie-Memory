# Extracción coherente de116: filtro fallido y diagnóstico de contrato

La autoría nueva de los116 textos con guía y vocabulario compartidos **no supera el filtro**:79representaciones completas y64completas y fieles para ambos críticos. No se adopta ni se avanza a utilidad, baseline o V6 con esta candidata. La corrección manual116/116 conserva su registro histórico, pero no demuestra que esas representaciones puedan producirse coherentemente desde cero. El objetivo sigue activo en el paso3, sin permisos pendientes.

## Resultado comprobado

Cuatro contextos nuevos extrajeron83fuentes en lotes28/28/27 y33preguntas por separado. No recibieron raws anteriores, gold, contraparte o resultados. Usaron el mismo vocabulario de12features y la misma guía semántica. Los ejemplos de forma son genéricos ajenos al corpus. Se suministraron suplementos comunes de admisión durante la autoría; las versiones se conservan y no cambiaron guía semántica o vocabulario. Cada initial se guardó antes del feedback estructural; reparaciones, rondas y finales permanecen.

| Medida | Fuentes83 | Preguntas33 | Total116 |
| --- | ---: | ---: | ---: |
| Primera pasada completa | 32 | 27 | 59 |
| Primera pasada parcial | 14 | 6 | 20 |
| Primera pasada inválida | 37 | 0 | 37 |
| Final completa, tras feedback estructural | 52 | 27 | 79 |
| Final parcial | 31 | 6 | 37 |
| Final inválida | 0 | 0 | 0 |
| Final fiel en las4dimensiones para ambos críticos | 46 | 18 | 64 |
| Final completa y fiel para ambos | 46 | 18 | **64(55,2%)** |

Las64aceptaciones coinciden con DTO completos. Quince DTO completos no obtienen aceptación integral de ambos; una validación correcta no prueba fidelidad. No se excluyen las37entradas parciales ni se convierten en negativos correctos. La primera pasada solo tiene medición estructural; no se le atribuye crítica semántica que no se ejecutó.

Dos contextos fuente y otros dos de preguntas revisaron paquetes separados sin resultados, código, gold, autoría explicativa u otra crítica. Se auditaron928citas UTF-8 y1.260punteros, igualdad entre dictámenes individuales y agregados, hashes, IDs y cuatro dimensiones por entrada. Es revisión de la misma familia Codex, sin validación humana ni independencia de familia.

Paquete fuente SHA-256 `70ce235b549b35e6100e390ac4a629969b1f1a037b596fdafc5eee8ceb31ffec`; pregunta `746ad9b247c7d0bcbe10f8db24e668695c5c252503a83631b474e3f63cef717b`. Artefactos y auditoría: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/coherent-known116-fresh-v1/`.

## Qué explica el fallo

Hay pérdidas concretas nuevas: complemento de un pedido omitido, relaciones nominales sustituidas por asociaciones, grupos/cardinalidad o dominio universal incompletos, tarea elíptica de pendientes y distinción de demora de borrado frente a retención. Algunas aparecen pese a que el DTO valida completo. Deben repararse con carriers existentes cuando su semántica los permita; no se arreglan cambiando coverage a represented.

También existe discrepancia entre contratos y criterios. Un asociado nominal antes aceptado en el banco manual se penaliza ahora con una forma equivalente. El carrier nominal actual no admite cualquier caso: su complemento descriptor es anónimo, no un target propio nombrado, y la construcción literal tiene restricciones sobre de/del/de la. Por ello no basta ordenar al extractor que use ese carrier en todos los genitivos. Las críticas sobre posesivos también exigen publicar antecedentes que los contratos específicos conservaban como dependencia rígida de contexto original. Candidato saliente y ligadura acreditada requieren distinguirse.

El [diagnóstico retrospectivo](documentary-v3-coherent116-contract-diagnosis-2026-10-22.md) contrasta cinco casos con evidencia. No borra pérdidas nuevas, no convierte origen/texto retenido en fidelidad y no altera dictámenes. Se está realizando una adjudicación conceptual uniforme antes de modificar la guía o ampliar un operador. Repetir una votación sobre los mismos raws para conseguir aprobación no es una corrección.

## Comparaciones y alcance

El99/116autónomo anterior y el116/116manual pertenecen al historial del mismo banco conocido, con contratos/perfiles y procedencias diferentes. Esta no es una comparación emparejada del producto ni demuestra una caída porcentual causal de Decisions/Jev. Sí prueba que esta configuración de autoría compartida no reproduce el cierre manual y no cumple su puerta de calidad. Se mantiene el histórico, se publica el fallo y se impide la promoción.

El motor Rust conserva el binario `8595587555348a115c2e4cd6c9828b8345fe762ca1850bec72783af9f96570d3`; las863pruebas,72controles y425replays de la [fase de implementación](documentary-v3-nominal-event-context-results-2026-10-22.md) pertenecen al mismo código. No se repitió Cargo durante esta extracción porque no hubo cambios de código. El perfil de producto y las gráficas de clasificación/ruta no reciben estas cifras como reemplazo: no se ejecutó esa comparación.

## Coste y siguiente acción

La fase nueva usó ocho contextos nativos de autoría/crítica. Los recibos conservan288comandos CLI con argumentos y un fallo adicional de invocación declarado; el coordinador añadió116validaciones:405invocaciones locales documentadas. No se miden con ellas latencia de extracción, tokens de Codex o producción. **USD0 API nuevo y cero llamadas a proveedores; coste Codex desconocido.** No hay importes de API desconocidos nuevos. Diagnóstico y adjudicación prospectiva se registran como trabajo posterior separado.

Siguiente acción: resolver la discrepancia de criterio bajo un contrato único, corregir las omisiones con operadores acreditados, completar materiales de forma antes de la próxima autoría y probar de nuevo con contextos nuevos. Si un carrier actual no conserva una construcción, demostrar la carencia y revisar la ampliación antes del código. Los116textos permanecen en el denominador. Solo un pase coherente habilita utilidad/base, recursos, estabilidad y después casos nuevos ciegos. V6 sigue reservado; no hay adopción.
