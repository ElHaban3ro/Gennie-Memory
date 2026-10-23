# Contexto de capacidad: recuperación medida — 2026-10-22

El mecanismo experimental recupera una evidencia útil del banco completo sin perder ninguna anterior: **23/39→24/39 (59,0 %→61,5 %)**, con **cero selecciones ajenas de 60**. Quedan 15 omisiones; una pregunta ambigua continúa separada. No supera la puerta completa 39/39 y no se adopta en producto.

| Medición | Antes | Después |
| --- | --- | --- |
| Evidencias útiles seleccionadas | 23/39 | 24/39 |
| Selecciones ajenas | 0/60 | 0/60 |
| Pérdidas de selecciones anteriores | — | 0 |
| Respuestas completas | 0 | 0 |
| Selecciones solo condicionales | 1 | 1 |

Se usaron las mismas 100 entradas automáticas congeladas, los mismos textos, vocabularios y etiquetas, y la misma regla de selección en **todas** las lecturas. No hubo nueva extracción para esta comparación. Los binarios anteriores/nuevos alternaron orden. La ganancia es `cv3q_15/cv3_15_00`: «Risco … no admite el modo Fino» ahora aporta esa advertencia a la pregunta por la resolución en Fino. Se conserva el fundamento `capability`, los términos de respuesta vacíos y `actual_use_inferred:false`; no se atribuye una resolución ni se infiere uso real.

## Cambio y fallos reproducidos

La ruta existente de contexto de admisión solo admitía fundamento `description`. El cambio permite también `capability` exclusivamente en contexto V2. Exige la declaración de relevancia `admits→quantity`, el perfil de asignación, una pregunta positiva de cantidad con fundamento `assignment`, sujeto y modo compatibles, y las condiciones, ámbitos, dependencias y premisas ya exigidos. No añade soporte implícito de `property/value`: Duna permanece pendiente, en vez de reutilizar una declaración cuyo target es otro predicado.

La primera prueba reprodujo el descarte con `capability`; tras la corrección pasan 72 combinaciones de fundamento, control y adaptador. La crítica independiente encontró un defecto real: seleccionaba «Fino de Orbe» para «Fino de Risco». Una prueba adicional reprodujo esa selección incorrecta. Se añadió comprobación bidireccional de los enlaces restrictivos de los referentes y sus alias; una segunda prueba descubrió el mismo problema anidado. La comprobación ahora recorre también los anclajes, con límite conservador de profundidad y rechazo de anáforas, bindings o dependencias pendientes. Las 16 combinaciones de tipo de enlace, correspondencia y anidamiento pasan. Los casos fallidos y las primeras implementaciones se preservan por logs/hash; no se cambió el contraejemplo para hacerlo pasar.

La comprobación nueva de referentes se aplica a la ampliación de `capability`. La ruta histórica de `description` conserva su comportamiento; no se afirma que todo contraejemplo posible de esa ruta esté resuelto. No se ha demostrado seguridad semántica universal por pasar estos controles.

## Verificación y límites

- `cargo fmt --all --check`, compilación offline y **723 pruebas aprobadas**, cero fallos y 24 ignoradas.
- **226/226 informes históricos idénticos** con el binario final.
- Crítica independiente: **28/28 resultados con documentos válidos pasan**. Otros dos resultados corresponden a un fixture cíclico inválido, separado y conservado; no son dos pases semánticos ni un fallo reproducido del helper. El rechazo del ciclo lo demuestra el validador; el límite del helper se revisó estáticamente.
- El contraejemplo original de propietario distinto queda rechazado. Pasan referentes iguales y anidados compatibles; se rechazan vínculos distintos, unilaterales y aportados por alias. Todos los positivos nuevos permanecen parciales y conservan su fuente.

Los tiempos CLI observados fueron p50 46,724→45,422 ms y p95 95,673→91,248 ms. Hubo suite y replays locales concurrentes: son tiempos descriptivos, no una medición aislada ni una mejora acreditada de recursos. Excluyen extracción, crítica, red y producto. No se compara este tiempo con otra tanda bajo distinta carga.

**Coste externo nuevo: USD 0 y cero llamadas.** Coste y tokens facturables de Codex desconocidos; los subagentes no se presentan como gratuitos. Acumulado externo conservado: 6.129 operaciones, 23 campañas emparejadas, USD 4,565416508 conocidos más 20 operaciones con coste desconocido, sin Codex.

## Diagnóstico y contrato pendiente

La auditoría estática de las 16 omisiones anteriores quedó registrada en nueve familias: corrección entre predicados, coordenadas de asignación, alcance de calificadores, contexto de agua, anclaje documental, admisión, normalización de retención, selección de lecturas y guard de proveedor. La corrección actual resuelve una de esas omisiones; el resto no se marca reparado por compartir familia. Sálamo exige normalización/reextracción independiente de pasado, estado crudo y dominio universal; no procede eliminar sus guards. Olmo del piloto canónico y Olmo del banco completo tienen frames distintos.

Dos relaciones generales nuevas para material ligado a una acción y su procedencia local pasaron un preflight manual con dos críticos separados: cuatro representaciones, cuatro dimensiones por crítico, sin contrapartes ni etiquetas. Esa prueba solo permite congelar una guía prospectiva; no acredita extracción automática o recuperación. El piloto de extracción correspondiente terminó en un único intento semántico: cuatro textos completos y cuatro aceptados por ambos críticos nuevos, sin reparación técnica. El autor recibió únicamente textos, guía, vocabulario y notación; los críticos no vieron prototipos, contrapartes, etiquetas ni resultados anteriores. Son casos de desarrollo conocidos y revisión de la misma familia Codex, no revisión humana ni reserva. Se recupera la representación de Malva y se conservan tres controles; no se actualiza la fidelidad global de 116 ni el recall de 100 por este resultado. Evidencia separada en `material-canonical-extraction-pilot-v1/report.json`.

La propuesta temporal sigue pendiente: no basta un literal `occurred_at` para «no…hasta». Su crítica detectó límites de ventana contextual, identidad de episodios repetidos y posición temporal de eventos durativos. No se implementó un operador incompleto ni se declaró cerrado un inventario incierto para mejorar la puntuación.

Los pasos 1–3 avanzan con contrato, banco y mecanismo, pero la puerta local sigue fallando. Los pasos 4–5 conservan sus requisitos: comparación contemporánea con producto, recursos completos, estabilidad y reserva con revisión ciega antes de adoptar. La reserva `conversation-evidence-v6` no se abrió; defaults, README, dashboard, datos y credenciales no cambiaron.

## Evidencia reproducible

Bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`:

- `admission-capability-context-v1/known100-comparison/report.json`: comparación completa, resultados y hashes.
- `admission-capability-context-v1/critical-review/`: contraejemplo original, controles congelados y seguimiento.
- `admission-capability-context-v1/root-verification/`: fuentes finales, logs y verificación.
- `remaining16-cause-audit-v1/audit.json`: causas estáticas y referencias precisas.
- `material-contract-v6-independent-review-v1/report.json`: expresividad manual y revisión doble.
- `canonical-contract-v1/draft-v6-material/`: guía y vocabulario prospectivos congelados.
- `canonical-contract-v6-temporal-critical-v1/`: crítica de la propuesta temporal todavía no implementada.

Binario final SHA-256: `8216087caeb227bcc5dc428ea790f1d8f2d57540b29b9220283a9c32949ea657`.
