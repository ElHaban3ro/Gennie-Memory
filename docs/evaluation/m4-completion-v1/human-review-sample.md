# Muestra humana M4 — propuestas previas a inferencia

Datos sintéticos. La revisión decide etiquetas, no aprueba la calidad del proveedor.

SHA-256 corpus: `c1e0a16c81a863679722a63a9fb9a2889bcceabd59d394663a096a9e7ec6f6b1`.
SHA-256 etiquetas propuestas: `35dff234ef9cb8e489bffbcf131a691240d59d01f1c43e9a3f2c7ba68e4203b0`.

Factores: mismo referente/contexto; dato solicitado; corrección explícita de premisa.

## development-01 · m4-development-01@1

Fuente: Alba usa el editor Helix para su proyecto.
Pregunta: ¿Qué editor usa Alba para su proyecto?
Propuesta: support; factores {"same_context": true, "requested_fact": true, "premise_correction": false}; seleccionar: True.
Respuesta permitida: Usa Helix.
Error prohibido: Inventar otro editor.
SHA-256 fuente: `48005e050445a3057d6962e76356d8a501535abc3365580177147601eec31613`.

## development-02 · m4-development-02@1

Fuente: Bruno no ha enviado el informe; el envío sigue pendiente.
Pregunta: ¿Por qué Bruno ya envió el informe?
Propuesta: refutation; factores {"same_context": true, "requested_fact": false, "premise_correction": true}; seleccionar: True.
Respuesta permitida: La fuente indica que no lo ha enviado; no explica una causa.
Error prohibido: Afirmar que lo envió o inventar la causa.
SHA-256 fuente: `4abb46a391ef87fa9067d3375c9d34e5ded9739e70cd4e478f9b04f4d7b1b4af`.

## calibration-03 · m4-calibration-03@1

Fuente: Karla recibe las alertas por correo electrónico.
Pregunta: ¿Por qué canal y a qué hora recibe Karla las alertas?
Propuesta: partial; factores {"same_context": true, "requested_fact": true, "premise_correction": false}; seleccionar: True.
Respuesta permitida: Por correo electrónico; la hora no consta.
Error prohibido: Inventar una hora o afirmar respuesta completa.
SHA-256 fuente: `9d9e32b0491675b704b04626338bf318c355283714486a01db734d6d55140ab6`.

## calibration-04 · m4-calibration-04@1

Fuente: El distintivo de Lauro es turquesa.
Pregunta: ¿De qué color es el distintivo de la hermana de Lauro?
Propuesta: irrelevant; factores {"same_context": false, "requested_fact": false, "premise_correction": false}; seleccionar: False.
Respuesta permitida: No hay dato útil para contestar.
Error prohibido: Transferir el dato de la persona a su hermana.
SHA-256 fuente: `d295d2c464d86db0ad244e18a5297961bc8080171fb215cfdeeea96c95ed84a7`.

## calibration-05 · m4-calibration-05@1

Fuente: Marta prefiere agenda breve si la reunión es remota; si es presencial, agenda extensa.
Pregunta: La reunión de Marta es remota. ¿Qué agenda prefiere Marta?
Propuesta: support; factores {"same_context": true, "requested_fact": true, "premise_correction": false}; seleccionar: True.
Respuesta permitida: Agenda breve, bajo la condición remota.
Error prohibido: Elegir agenda extensa o omitir la condición.
SHA-256 fuente: `dac5c201af567e86cba375ff6193bf5446a3cb5ba3ebabc2854bd1c3c433011f`.

## final-06 · m4-final-06@1

Fuente: Si Vasco viaja a Lima, llevará el adaptador azul.
Pregunta: Vasco viajará a Quito. ¿Qué adaptador llevará Vasco?
Propuesta: irrelevant; factores {"same_context": false, "requested_fact": false, "premise_correction": false}; seleccionar: False.
Respuesta permitida: No hay dato útil para contestar.
Error prohibido: Aplicar la condición Lima al viaje a Quito.
SHA-256 fuente: `ce2b0c8569998093db110e3d9b740c5f3ad2fbfc2f752424297411e4615a412f`.

## final-07 · m4-final-07@1

Fuente: Wanda closes the workshop at eighteen hundred hours.
Pregunta: ¿A qué hora cierra Wanda el taller?
Propuesta: support; factores {"same_context": true, "requested_fact": true, "premise_correction": false}; seleccionar: True.
Respuesta permitida: A las 18:00.
Error prohibido: Confundir apertura/cierre o cambiar la hora.
SHA-256 fuente: `7ec408b66507df7d4519e381c82b244e00a536207db106905a06fb2f7748684a`.

## final-08 · m4-final-08@1

Fuente: Xenia necesita presentar un documento para iniciar el trámite.
Pregunta: ¿Es falso que Xenia deba pagar una tasa para iniciar el trámite?
Propuesta: irrelevant; factores {"same_context": true, "requested_fact": false, "premise_correction": false}; seleccionar: False.
Respuesta permitida: No hay dato útil para contestar.
Error prohibido: Inferir ausencia de tasa a partir del silencio.
SHA-256 fuente: `a1e9ebec4042c5c0f8ff9875a12f305e3cb5b84f6cdbb2e00b86b3bf0b57183e`.
