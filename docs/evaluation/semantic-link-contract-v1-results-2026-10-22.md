# Contrato de vínculo semántico: diagnóstico ciego

Fecha: 2026-10-22. Paso 3 activo. [Protocolo previo](semantic-link-contract-v1-protocol-2026-10-22.md). Se evaluó una ficha manual uniforme sobre textos TRAIN existentes. No se ejecutó un clasificador productivo ni se midió generalización.

## Resultado

- Fichas completas: 52/52. Concordancia literal de decisión con el gold histórico: **51/52**.
- Pares dirigidos C/none correctamente resueltos: **12/13**.
- Indeterminados: **1**. Puerta diagnóstica prospectiva: **no superada**.

Muestra dirigida: 13 parciales exclusivos de la ablación anterior, 13 testigos ajenos distintos y 26 controles (8 apoyo, 8 refutación, 10 ajenos). Selección y orden congelados antes de revisión; texto y gold originales intactos. Un contexto nativo nuevo recibió solo contrato y textos con IDs neutros, sin puntuaciones, etiquetas, mapa o resultados. No es una muestra poblacional ni revisión humana independiente de familia.

La ficha separa referente, foco solicitado y aporte del recuerdo, conserva citas literales y permite incertidumbre. La evidencia parcial no necesita resolver toda la pregunta; la corrección no tiene que confirmar su premisa y la ausencia de una casilla solo limita lo que documenta la fuente. Una cita válida por sí sola no acredita el vínculo semántico.

## Caso indeterminado conservado

Pregunta: «¿Cuántos ejemplares se imprimieron del cartel Risco?» Fuente: «Del cartel Risco Bajo se imprimieron 180 ejemplares.»

El revisor no afirma que Risco y Risco Bajo sean iguales ni que sean distintos: conserva la identidad sin resolver y no admite el recuerdo. El gold histórico none permanece intacto. No se elimina este caso ni se repite el revisor para mejorar el porcentaje. La puerta exigía cero indeterminados; no se cambia retrospectivamente.

## Fidelidad y límites

La revisión separada del autor considera fieles las52fichas, incluida la abstención de identidad. Ninguna interpretación no sustentada ni fidelidad pendiente. Este juicio procede de otro agente de la misma familia que conocía el diseño y la historia145; no es revisión humana ni ceguera completa al contexto histórico. En146no leyó gold, mapa privado o auditoría.

Preflight fuente/anonimización: 161/161 controles. Auditoría posterior: 744 controles estructurales, citas, IDs y hashes; todos pasan. La revisión de fidelidad contrastó las 52 fichas con pregunta y documento. El revisor de fidelidad es otro agente de la misma familia, con contexto previo de diseño y diagnóstico145; esa revisión no fue ciega a las fichas. No recibió el mapa privado ni el gold.

La concordancia de fichas manuales no demuestra que un modelo económico pueda extraerlas automáticamente. La puerta original 24/24 + 64/64, preservación de seis conjuntos, comparación productiva, recursos, recibos, estabilidad y V6 siguen pendientes. Producto, DB, secretos, dependencias, ramas y README/gráficas sin cambios.

## Coste y continuación

No hubo nuevas llamadas a proveedores: USD 0 de API externa. No se ejecutó el transformer local ni entrenamiento. Sí hubo revisión mediante agentes Codex; su coste y latencia económica no están cuantificados y no se equiparan a una ruta productiva gratuita. Acumulado externo intacto: 6.155 operaciones, USD 4,566379428 conocidos y 24 operaciones de coste desconocido.

La puerta manual V1 permanece no superada:51/52decisiones concordantes y una duda de identidad fiel. No se cambia gold ni se repite la anotación. La siguiente candidata automática será semantic-link-flash-lite-v1: Gemini2.5FlashLite con ficha JSON y citas literales, usando el transporte generativo existente y un adaptador experimental nuevo. Antes de red se comprobarán disponibilidad, tarifas, recibos y presupuesto completo. La interfaz prospectiva seleccionará solo include; exclude e indeterminate retendrán la evidencia sin seleccionarla, con deuda explícita. Toda abstención sobre un útil contará como omisión; no es un rescate del gate V1. Primero se ejecutará un piloto fijo de24TRAIN y solo sin fallos técnicos o nuevas pérdidas/ajenos se completarán los432pares, sin repetir las24llamadas. Siguen la puerta original88, seis conjuntos, recursos, estabilidad y casos nuevos antes de adoptar.

Artefactos: `outputs/iromem/evaluation/semantic-link-contract-v1-2026-10-22/`. Objetivo íntegro activo; las reservas siguen cerradas.
