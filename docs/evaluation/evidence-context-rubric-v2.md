# Evidencia solicitada y contexto relacionado v2

Fecha2026-10-19. Rúbrica para bancos futuros. Surge de una revisión retrospectiva deV5: no modifica los bancos congeladosV1–V6 ni sus métricas y no es una nueva mejora medida. El contrato de recuperación mantiene apoyo, refutación y parciales útiles; no sacrifica evidencia parcial para aumentar precisión.

Por cada par pregunta/fragmento, registrar referente y condiciones, partes solicitadas, cita literal, afirmación permitida, rol y razón. Primero comprobar compatibilidad de identidad/ámbito. Después aplicar estas reglas por separado:

- `supports_answer`: contiene los datos solicitados y cubre todas las partes pedidas.
- `refutes_premise`: niega explícitamente o implica un estado incompatible con una premisa corregible. Silencio, preparación, ubicación y requisitos asociados no bastan. «Solo exige carné» puede refutar necesidad de avalista; «presenta carné» por sí solo no establece exclusividad.
- `pending_or_partial`: aporta un dato o estado explícito de alguna parte solicitada, o solo una de varias partes. «Falta el ensayo húmedo» informa su estado; «muestra preparada» no establece que no exista un ensayo previo.
- `related_context`: aporta información del tema, pero no un dato/estado solicitado ni corrección válida. Se conserva como contexto anotado aparte; no se cuenta como respuesta, refutación o parte requerida ni se transforma en falsedad. Su selección se informa aparte en futuros bancos.
- `unrelated`: no satisface las relaciones útiles y tampoco aporta contexto relevante. Una consulta sin ninguna evidencia útil puede tener contexto relacionado.

El rol pertenece al par completo. Un ID que responde únicamente el sellado entre tres aspectos pedidos sigue siendo parcial aunque su dato de sellado esté completo. Un conjunto de IDs parciales puede cubrir toda la pregunta: anotar cobertura por parte, sin inferirlo del mejor rol. No exigir todos los IDs redundantes; distinguir evidencia requerida para cobertura de alternativas equivalentes y contexto opcional.

No inferir «no realizado» solo porque hay una cita futura, ni aprobación por recepción de documentos, grado por preparación de muestra o ausencia de avalista por otro requisito. Si una frase contextual contiene también un dato solicitado, clasificar la contribución útil citando ese fragmento; no descartar el recuerdo por contener contexto adicional. Identidad literal tampoco basta para validar el significado.

Una futura revisión ciega recibe fuente y pregunta, no gold/resultados/cuotas. Se registran desacuerdos y se concilian antes de llamadas. Guardar fuente, preguntas, etiquetas y revisión con hashes, versión y relación entre IDs. No aplicar esta rúbrica retrospectivamente y presentar la diferencia como rendimiento mejorado. Si un banco observado debe corregirse, generar otra versión de desarrollo, conservar ambas e informar resultados por separado; la validación necesita otros casos.

La [conciliación retrospectiva](../../outputs/iromem/evaluation/score-source-audit-adjudication-2026-10-19/adjudication.json) registra diez diferencias de contexto y dos de rol. Es juicio de agentes de la misma familia y no revisión humana. No cambia el contrato runtime ni envía etiquetas al proveedor.
