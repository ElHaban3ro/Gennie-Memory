# Tarea pendiente: corrección operativa y un documento conocido cerrado

El conjunto mixto conocido pasa de **107/116 a 108/116 documentos completos y fieles para ambos críticos**: quedan ocho. Se corrigió una fuga de valores de respuesta parcial y se cerró el documento de páginas pendientes sin inventarle tarea. **765 pruebas pasan**, cero fallos, 24 ignoradas; matriz integrada 9/9 y 216 resultados históricos idénticos. El objetivo de cinco pasos sigue activo en el paso 3. No hay nueva medición de extracción autónoma, recuperación, producto o V6.

## Problema reproducido y cambio

La frase «cero páginas pendientes» no identifica digitalizar ni revisar. En una matriz tipada sintética nueva, el comparador ya marcaba incompleta la respuesta cuando solo una parte fijaba tarea, pero conservaba cero en `partial.answer_terms` y seleccionaba el claim. Fallaron cuatro de nueve controles válidos. No bastaba añadir una deuda de configuración.

El feature optativo `pending_task_scope` del perfil contextual V1 protege todos los canales de respuesta, incluida procedencia por dependencias. Permite continuar solamente a cantidades planas `pending_pages`, con un claim, petición answer/amount o truth/status y tarea exactamente coincidente: ausencia/ausencia o el mismo símbolo de acción registrado. Ausencia no equivale a cualquier tarea. Sigue exigiendo todos los guards previos. Tarea global, duplicada o desconocida, composición, branch/binder, modos no admitidos y cuerpos compuestos quedan bloqueados. El witness afectado se convierte en candidato pendiente sin términos, proposiciones de respuesta ni bindings; permanecen los witnesses y conflictos independientes. El original sigue disponible para auditoría.

La [crítica del contrato y del código](documentary-v3-pending-task-guard-critique-2026-10-22.md) señaló superficies de escape. Se corrigió un retorno que omitía analizar conflictos cuando ningún otro canal cambiaba; el nuevo control incluye un conflicto aislado con payload y otro independiente. El crítico no identificó un productor actual que explotara ese caso: fue cierre preventivo del contrato, separado de la fuga parcial reproducida. También se sustituyeron copias de payloads conservados por movimientos de ownership. El turno de revisión terminó con un error de capacidad después de comunicar y guardar sus conclusiones; no se presenta como una segunda ejecución de pruebas.

El cambio pertenece exclusivamente al evaluador Rust. No hay nuevo DTO, dependencia, esquema SQL, servicio, política de producto o equivalencia lingüística. La [definición del contrato](documentary-v3-pending-task-scope-contract-2026-10-22.md) distingue protección mecánica de fidelidad textual.

## Corrección conocida y revisión

En `d-79d8fc5f9965fe9d130446dd`, «Del expediente Atlas no quedan páginas pendientes», se conserva exactamente la cantidad cero, el sujeto, el fundamento descriptivo y el resto del raw. Se retira la anotación que trataba una tarea no especificada por la fuente como una pérdida del extractor, y se declara el perfil protector. Un parche inverso comprueba que las únicas diferencias son cobertura, feature de vocabulario y su hash. No se añade tarea ni se afirma acción realizada, cierre de expediente o total de páginas.

Dos críticos nuevos, en contextos separados de la misma familia, aceptaron las cuatro dimensiones cada uno: **8/8 faithful**. Recibieron un único paquete congelado de texto, candidato, vocabulario y guía semántica, sin contraparte, gold, diagnósticos, validación, código ni el otro dictamen. El receptor verificó hashes, citas UTF-8 y punteros. No es revisión humana ni independencia de familia; tampoco mide extracción autónoma.

Dos revisores anteriores también emitieron juicios favorables, pero ambos declararon exposición accidental al párrafo diagnóstico al localizar el contrato. Sus entregas se conservan y se excluyen de la puerta ciega. Se sustituyeron por los dos contextos nuevos con un paquete que ya no incluía ese párrafo. La primera guía congelada admitía prospectivamente constraint; el código final es más conservador y no lo admite. Esa reducción operativa no modifica el significado de esta fuente ni sus dictámenes.

## Verificación y métricas

| Evidencia | Resultado |
| --- | ---: |
| Matriz integrada nueva: tarea ausente/digitalizar/revisar | 9/9 |
| Coincidencias que conservan la respuesta | 3/3 |
| Combinaciones no coincidentes sin cifras ni selección | 6/6 |
| Pruebas workspace actuales | 765 pasan, 0 fallan, 24 ignoradas |
| Informes históricos sin perfil, replay exacto | 216/216 |
| Documentos mixtos revalidados con binario actual | 116/116 ejecutados |
| Documentos sin corrección, informes idénticos al anterior | 115/115 |
| Mixto conocido estructuralmente completo | 109→110/116 |
| Mixto conocido aceptado por ambos críticos | 108→108/116 |
| Mixto conocido completo y aceptado por ambos | 107→108/116 |

El documento ya tenía revisión favorable histórica pero seguía parcial; por eso el total aceptado por ambos no aumenta. El conjunto combina versiones de vocabulario y correcciones conocidas: no es campaña nueva coherente ni extracción independiente. La extracción autónoma original **99/116** permanece inalterada. La utilidad anterior **31/39**, cero ajenos/60, sigue siendo histórica; no se suman estos controles a sus cifras ni se renuevan gráficas del README con otra métrica.

Los primeros nueve controles del probe eran inválidos por `Request.unit` mal formado. Se conservan; una reparación acotada convirtió ese campo al objeto correcto antes de interpretar la seguridad. La matriz válida anterior fallida, ejecuciones intermedias y replays están preservados, separados de los finales. La aceptación final utiliza exclusivamente `execution-final-v5`, `legacy-regression-final-v3`, `mixed116-final-v2`, `workspace-final-v2.log` y el binario SHA-256 `89a1a37c99abe11060c6417e92229391196d31d61064cd3155e33524b780be76`. `cargo fmt --all --check` pasa. Las pruebas focales usan Engine sintético; la matriz CLI aporta la validación completa de ambos documentos.

## Intento rechazado y pendientes

El piloto de cuatro lecturas cerradas para la pregunta del tutor de Faro pasó validación estructural. Ambos críticos rechazaron el cierre: plausibilidad no acredita exhaustividad ni control del infinitivo; finalidad y necesidad pueden coexistir. Se verificaron sus hashes, citas y pointers. No se aceptó como mejora ni se cambiaron sus dictámenes.

Las auditorías de minimalidad de [ambigüedad](documentary-v3-remaining-ambiguity-minimality-2026-10-22.md), [referencia/tarea](documentary-v3-reference-task-minimality-2026-10-22.md) y [contexto Nacar](documentary-v3-nacar-context-minimality-2026-10-22.md) son propuestas documentales. Este turno implementa solo el guard de tarea; no declara implementados los demás operadores propuestos.

Quedan ocho casos: pregunta de tutor/finalidad; sentido de matrículas; alcance de hasta apagarse en Lino; suficiencia de credencial y posesivo; tres fuentes con en Nacar; adjunción de grande a bandeja/papel. La lista exacta está en `known116-report.json`. Después se mantiene la extracción completa con guía/vocabulario compartidos y doble revisión, utilidad sin pérdidas, comparación de producto, recursos/estabilidad y V6 reservada. No se abre una puerta por aprobar forma, ni se cambia el requisito 116.

No se ha medido latencia de producto ni costo de extracción escalable. Los controles cubren el carrier acotado; no prueban soporte general para reglas, restricciones o todo alcance lingüístico. La ausencia de una tarea en un raw no demuestra que el texto no la expresara: sigue siendo responsabilidad de revisión de fidelidad, no de este guard.

## Coste y trazabilidad

**Cero llamadas externas nuevas; USD 0 nuevos de proveedor.** Once contextos nativos nuevos y un turno adicional de revisión de código; un intento de spawn sin contexto creado y un error de capacidad al terminar la crítica se registran aparte. Coste/tokens Codex desconocidos. El ledger externo conserva 6129 operaciones, 23 campañas, USD 4,565416508 conocidos y veinte importes desconocidos, excluyendo Codex.

Se ejecutaron 926 operaciones CLI locales durante la iteración: una validación del piloto rechazado; 45 controles de matriz (nueve inválidos, nueve baseline y tres tandas corregidas); tres replays de 216; dos revalidaciones de 116. Se preservan los intermedios sin sumarlos como casos nuevos. Hubo seis comandos Cargo de build/test y cinco de formato, todos seriales. No se arrancaron servicios ni se publicaron cambios.

Artefactos bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/atlas-task-scope-probe-v1/` y `faro-finite-readings-pilot-v1/`, con scripts, verificación e integridad de revisión. La instantánea 112 conserva los estados anteriores y este avance. El objetivo continúa activo; no está terminado.
