# Vínculo semántico V3: política local y distinciones explícitas

Fecha: 2026-10-22. Paso3activo. Una sola candidata, `semantic-link-span-v3`, después del falloV2. No se rescatan sus respuestas o cambian gold:V2queda detenida tras12HTTP/11juicios estructuralmente aceptados; su revisión fue3fieles,6no sustentados y3ambiguos. Acertar esas once selecciones no validó la interpretación.

## Cambios y límites

El modelo devuelve siete campos: review_id, query_focus_ids, document_evidence_ids, referent_link, focus_link, contribution, rationale. decision se elimina del esquema remoto. La política local valida enums y deriva exactamente la misma regla previa: si algún vínculo es unresolved o el aporte indeterminate, decisión indeterminate; en otro caso, different/other/none produce exclude; el resto include. Solo include selecciona. Todo útil indeterminado sigue siendo omisión. Artefacto conserva raw_card de siete campos, decisión compuesta y decision_origin=local_policy; nunca se atribuye esa decisión al modelo. V1/V2 siguen verificando sus propias decisiones remotas conforme a la misma regla.

Tres aclaraciones semánticas generales, sin casos/IDs/ejemplos del banco:1)identidad del referente es independiente de que el texto aporte otra magnitud o no aporte la pedida;2)record_absence requiere una afirmación explícita sobre ausencia/estado del dato pertinente, no silencio uotroatributo;3)evidencia debe conservar cláusulas mínimas completas con negación, atribución, condiciones y cantidades compuestas. Son hipótesis de mejora de seguimiento de contrato, no fidelidad garantizada.

Se mantienen mapas de tokens canónicos, fuentes completas, offsetsUTF8 y composición de referencias deV2. IDs exactos, fuente correcta, no vacíos, únicos y orden numérico; slices de tokens contiguos conservan separadores y grupos separados no se concatenan artificialmente. No hay recopia, casefold, fuzzyrepair, corrección retrospectiva o decisión por gold. Referencias válidas no prueban pertinencia/fidelidad. La redundancia eliminada no corrige por sí sola categorías falsas, citas incompletas o contexto equivocado.

## Comparación y gates

Mismos432TRAIN en el mismo orden, inputsgold y decisiones del padre idénticos byte a byte aV2, sin nuevas etiquetas, revisión manual al modelo, demos, scores o IDsoriginales en mensajes. Padre fijo del diagnóstico145, no control contemporáneo de recursos/producto. Piloto fijo primeras24background; cero pérdidas útiles nuevas o ajenos nuevos y sin fallo técnico/recibo antes de408restantes. No repetir prefijo. Filtro432:≥1ganancia útilC y cero daños nuevos. Toda falta de juicio permanece no evaluada.

Puerta original24/24+64/64, ganancia histórica, preservación de seis conjuntos, casos nuevos ciegos, producto, recibos/recursos≤1,5× y estabilidad intactos. Reservas originales/V6cerradas. La concordancia binaria, integridad y revisión de fidelidad se reportan separadas; no afirmar calidad general a partir de un prefijo o solo por derivar una decisión coherente. Una salida técnicamente inválida sigue siendo error, no indeterminate.

## Ejecución

Mismo google/gemini-2.5-flash-lite y proveedor estándar google-ai-studio, temperatura0, reasoningfalse, max1024, sin fallback/retry. Alias remoto sin revisión inmutable de pesos. Tarifas actuales/saldo se ligan al plan; recalcular payloads siete campos, estimación y cotas antes de red. Techo≤USD0,50, máximo432HTTP,900stotal y20sporrequest. Coste conocido+cota siguiente y reserva temporal antes de cada petición; detener ante presupuesto/plazo/error/recibo incompleto/refsinválidas/piloto degradado. No asumir tokenización exacta o garantía de facturación a partir de cotaUTF8bytes+1024framing estimado+1024output.

Mismo harnesscfg(test) porperfil/directory, guard fuentes/binario exacto antesmarcador/credenciales, archivoexclusivo y diario durable; guardar recibo/body antesdecode. V1/V2conservadas ysin reejecución. Preparador offline genera432payloads+span-index; pruebas de wire7, decisión local/origen, invalid enums/refs, silencio≠ausencia como límite de inferencia y regresiónV1/V2. Todo paso técnico no demuestra verdad de las categorías remotas.

CLI propio sin puerto, estimado2–15min o detención temprana; conservarhandle/PID y observar terminal sinreinicios por timeout ni detenerprocesosajenos. Ningún cambio producto/default,DB,secretos,deps,servicios,dashboard,ramas/commits/push o READMEgráficas. Coste Codex/local desconocido. Preflight independiente antes de inferencia, auditoría y revisión de fuente después. Artefactos: outputs/iromem/evaluation/semantic-link-span-v3-2026-10-22/.
