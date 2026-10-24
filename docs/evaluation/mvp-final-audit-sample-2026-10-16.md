# Muestra ciega para auditar etiquetas de la cohorte 018–020

Estado: **pendiente de revisión humana**. Selección anterior a cualquier resultado: tres positivos y un negativo por ámbito, ordenados dentro de cada clase por SHA-256 de `case_id`. La cohorte y el método de selección están fijados en [el protocolo](mvp-final-protocol-2026-10-16.md). Revisar la pregunta y el recuerdo sin consultar la salida de Gennie Memory; anotar ambigüedad, vigencia, ámbito y si la condición/fuente declarada son fieles. Una fuente `synthetic://` es solo un dato ficticio del fixture.

El asistente comprobó los 48 pares de pregunta y resumen y verificó por código que los 36 positivos referencian una memoria del mismo ámbito con hecho, condición y fuente idénticos al fixture; los doce negativos no tienen ID esperado. No detectó ambigüedad evidente en esta muestra. Esta es una revisión del asistente, **no una revisión humana independiente**: las casillas permanecen pendientes para esa revisión. Si se corrige una etiqueta, conservar este documento y registrar la decisión antes de ejecutar la campaña.

### synthetic_agent_018 · gmv1_feaab21969517d55d92a

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué herramienta ofrece el entorno de agent ficticio 018?
- **Esperado:** El entorno ficticio de agent ficticio 018 ofrece la herramienta comparador.
- **ID esperado:** gmv1_4521a536899d24927f8a
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/018/tool_available"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_018 · gmv1_ae97ea6cd9d3579151f4

- [ ] **Etiqueta:** positiva; pregunta: ¿Cuál es la siguiente comprobación de agent ficticio 018?
- **Esperado:** La siguiente comprobación registrada para agent ficticio 018 es contar los casos.
- **ID esperado:** gmv1_b50c0a013d978b113514
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/018/next_check"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_018 · gmv1_824095ce610bca938f72

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué idioma de salida registra el perfil de agent ficticio 018?
- **Esperado:** El perfil ficticio de agent ficticio 018 registra portugués como idioma de salida.
- **ID esperado:** gmv1_b9b78ec2634f8cf61864
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/018/output_language"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_018 · gmv1_00f0c1d4e8fe83addd2b

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es el código secreto de agent ficticio 018?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_019 · gmv1_836d278d7fa61b19197a

- [ ] **Etiqueta:** positiva; pregunta: ¿Cómo se llamó el conjunto de evaluación de agent ficticio 019?
- **Esperado:** El conjunto ficticio de evaluación de agent ficticio 019 se llamó lote alfa.
- **ID esperado:** gmv1_ba8ad93cf7dc4e6e218a
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/019/evaluation_set"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_019 · gmv1_d3a916a220898edaf708

- [ ] **Etiqueta:** positiva; pregunta: ¿Cuál es la siguiente comprobación de agent ficticio 019?
- **Esperado:** La siguiente comprobación registrada para agent ficticio 019 es medir latencia.
- **ID esperado:** gmv1_80870452015739928013
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/019/next_check"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_019 · gmv1_784ab3c592685406545a

- [ ] **Etiqueta:** positiva; pregunta: ¿Cuál es el punto de control de agent ficticio 019?
- **Esperado:** El último punto de control ficticio de agent ficticio 019 es fase tres.
- **ID esperado:** gmv1_590cd4b554791aa3f0ee
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/019/checkpoint"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_019 · gmv1_a9dd21801832dee4888c

- [ ] **Etiqueta:** negativa; pregunta: ¿Qué contraseña usa agent ficticio 019?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_020 · gmv1_510c2fbc46286828c5aa

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué artefacto produjo agent ficticio 020?
- **Esperado:** El artefacto ficticio producido por agent ficticio 020 se llama tabla-b.csv.
- **ID esperado:** gmv1_f4efcad0f5c285d6a112
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/020/artifact"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_020 · gmv1_ea9b916e4c3020d4b8c4

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué solución provisional registró agent ficticio 020?
- **Esperado:** La solución provisional registrada por agent ficticio 020 fue reducir el lote.
- **ID esperado:** gmv1_2ba9b8f7e1871564cc2a
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/020/workaround"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_020 · gmv1_ef86ec434d06cc0a46a9

- [ ] **Etiqueta:** positiva; pregunta: ¿Con qué contrasta afirmaciones el flujo de agent ficticio 020?
- **Esperado:** El flujo documentado de agent ficticio 020 contrasta afirmaciones con registros.
- **ID esperado:** gmv1_957a2c71e814c16ef76e
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "tool", "ref": "synthetic://general-memory-v1/agent/020/evidence"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_agent_020 · gmv1_2b8a640258f87abd2035

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es el código secreto de agent ficticio 020?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_018 · gmv1_4490d0c7a7555f0b1325

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué siguiente paso se acordó en conversation ficticio 018?
- **Esperado:** En conversation ficticio 018, se acordó hacer una llamada como siguiente paso.
- **ID esperado:** gmv1_b2fb0a06ab311ac3df7b
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/018/next_step"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_018 · gmv1_242250a6248a245761ec

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué indica la nota de traspaso de conversation ficticio 018?
- **Esperado:** La nota de traspaso de conversation ficticio 018 indica pedir un dato.
- **ID esperado:** gmv1_97d9081d04987a7740c4
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/018/handoff_note"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_018 · gmv1_4cdf762ec48f26acafb6

- [ ] **Etiqueta:** positiva; pregunta: ¿Cómo debe revisarse el resultado de conversation ficticio 018?
- **Esperado:** En conversation ficticio 018, el resultado debe revisarse con una segunda lectura.
- **ID esperado:** gmv1_7ede13fdb679b0ed10c2
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/018/review_rule"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_018 · gmv1_99e850664d4104ed41da

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es el código secreto de conversation ficticio 018?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_019 · gmv1_b32b0343c9809775d001

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué se priorizó en conversation ficticio 019?
- **Esperado:** En conversation ficticio 019, se priorizó corregir el error.
- **ID esperado:** gmv1_1cf99e4fc9f72dced9d3
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/019/priority"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_019 · gmv1_2e8eaa3218a3f202863c

- [ ] **Etiqueta:** positiva; pregunta: ¿Quién asumió la revisión en conversation ficticio 019?
- **Esperado:** En conversation ficticio 019, Berto asumió la siguiente revisión.
- **ID esperado:** gmv1_053bbcefd1a808504e47
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/019/agreed_owner"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_019 · gmv1_0027064e979413ccd03d

- [ ] **Etiqueta:** positiva; pregunta: ¿Cuál es la tarea actual de conversation ficticio 019?
- **Esperado:** En conversation ficticio 019, la tarea actual es corregir un ejemplo.
- **ID esperado:** gmv1_10a99e3987ff2a6ac376
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/019/current_task"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_019 · gmv1_46ff4dfba50c97695bb2

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es el color favorito de conversation ficticio 019?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_020 · gmv1_80f9e502998ad04783a4

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué se priorizó en conversation ficticio 020?
- **Esperado:** En conversation ficticio 020, se priorizó medir latencia.
- **ID esperado:** gmv1_3fabdbeec243901f4e91
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/020/priority"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_020 · gmv1_ecea481cb998f4b5aedd

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué fuente se señaló en conversation ficticio 020?
- **Esperado:** En conversation ficticio 020, la fuente señalada fue fila 8 de la tabla.
- **ID esperado:** gmv1_af50a2591d4b33ae6e62
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/020/source_pointer"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_020 · gmv1_9a6d294794f87a58a034

- [ ] **Etiqueta:** positiva; pregunta: ¿A qué valor se corrigió el dato en conversation ficticio 020?
- **Esperado:** En conversation ficticio 020, se corrigió el dato a 18 unidades.
- **ID esperado:** gmv1_b5c7e67b8281d5d062e7
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/conversation/020/correction"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_conversation_020 · gmv1_37e56e70cb8535bad5d1

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es la dirección postal exacta de conversation ficticio 020?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_018 · gmv1_3ec9c684ac0ebb70334f

- [ ] **Etiqueta:** positiva; pregunta: ¿Cómo revierte cambios project ficticio 018?
- **Esperado:** project ficticio 018 revierte cambios con respaldo previo.
- **ID esperado:** gmv1_4520074f9daa00f865f4
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/018/rollback"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_018 · gmv1_15f66d353d7ac4e9a631

- [ ] **Etiqueta:** positiva; pregunta: ¿Dónde archiva resultados project ficticio 018?
- **Esperado:** project ficticio 018 archiva los resultados en almacén cifrado.
- **ID esperado:** gmv1_55ec43cfe4e3eef7fb61
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/018/archive"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_018 · gmv1_a9761b47e3809fb5a91b

- [ ] **Etiqueta:** positiva; pregunta: ¿Cómo valida entregas project ficticio 018?
- **Esperado:** project ficticio 018 valida entregas con revisión por pares.
- **ID esperado:** gmv1_e6e1cbb6f8b607c24ed0
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/018/test_strategy"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_018 · gmv1_3e0079e87e1142ecd0a7

- [ ] **Etiqueta:** negativa; pregunta: ¿Qué contraseña usa project ficticio 018?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_019 · gmv1_313c75b923445ae768e0

- [ ] **Etiqueta:** positiva; pregunta: ¿Cuál es el entregable principal de project ficticio 019?
- **Esperado:** El entregable principal de project ficticio 019 es un prototipo.
- **ID esperado:** gmv1_adfe4bed62a7f343316e
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/019/deliverable"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_019 · gmv1_a6e3a1165dd6aaf3a5b7

- [ ] **Etiqueta:** positiva; pregunta: ¿Dónde documenta decisiones project ficticio 019?
- **Esperado:** project ficticio 019 documenta decisiones en wiki interna.
- **ID esperado:** gmv1_730d0d2856b71dd43f82
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/019/documentation"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_019 · gmv1_545c99a6bd505d54c908

- [ ] **Etiqueta:** positiva; pregunta: ¿Cómo se hace el traspaso de project ficticio 019?
- **Esperado:** El traspaso de project ficticio 019 se hace mediante video breve.
- **ID esperado:** gmv1_d494b87409bd0d646362
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/019/handoff"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_019 · gmv1_e0c1d46b493305aaf305

- [ ] **Etiqueta:** negativa; pregunta: ¿Qué decisión tomó project ficticio 019 mañana?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_020 · gmv1_33d881574ef6e6bb1319

- [ ] **Etiqueta:** positiva; pregunta: ¿Dónde archiva resultados project ficticio 020?
- **Esperado:** project ficticio 020 archiva los resultados en repositorio.
- **ID esperado:** gmv1_42e142030166342bc930
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/020/archive"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_020 · gmv1_d20168b77c72d5662165

- [ ] **Etiqueta:** positiva; pregunta: ¿Quién es el contacto de soporte de project ficticio 020?
- **Esperado:** El contacto de soporte de project ficticio 020 es Ada.
- **ID esperado:** gmv1_7800f1f893814da384ed
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/020/support_contact"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_020 · gmv1_d9ef476212817dcd0d11

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué exige project ficticio 020 antes de publicar?
- **Esperado:** project ficticio 020 requiere aprobación de diseño antes de publicar.
- **ID esperado:** gmv1_a5d9eec3de5fef5ebb72
- **Condición declarada:** "Antes de publicar"
- **Fuente declarada:** {"kind": "document", "ref": "synthetic://general-memory-v1/project/020/release_gate"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_project_020 · gmv1_54e1d70c3a44e76ff3e8

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es el código secreto de project ficticio 020?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_018 · gmv1_453ed3782d822f998307

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué comida prefiere user ficticio 018?
- **Esperado:** user ficticio 018 prefiere comidas sin picante.
- **ID esperado:** gmv1_9885115ff69d04de05b6
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/018/diet"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_018 · gmv1_d28a524b4bc05227e17a

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué asiento prefiere user ficticio 018 al viajar?
- **Esperado:** user ficticio 018 prefiere asiento de pasillo al viajar.
- **ID esperado:** gmv1_1f2b2cd7894a9071f64e
- **Condición declarada:** "Al viajar"
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/018/travel_seat"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_018 · gmv1_53a45d5b59a10e9f9b03

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué bebida suele pedir user ficticio 018 en reuniones?
- **Esperado:** user ficticio 018 suele pedir agua en reuniones.
- **ID esperado:** gmv1_9b64a4e9eafdfcbb3afc
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/018/drink"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_018 · gmv1_346d5f430a04cf83777e

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es la dirección postal exacta de user ficticio 018?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_019 · gmv1_f9a696dcda563d8a81e3

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué ajuste de accesibilidad necesita user ficticio 019 en documentos?
- **Esperado:** user ficticio 019 necesita documentos con títulos navegables.
- **ID esperado:** gmv1_3a5ba0919f2b28defd8e
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/019/accessibility"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_019 · gmv1_b6ebe2b4ec8a7fd7a615

- [ ] **Etiqueta:** positiva; pregunta: ¿En qué ciudad ficticia vive user ficticio 019?
- **Esperado:** user ficticio 019 vive en la ciudad ficticia de Puerto Alba.
- **ID esperado:** gmv1_f64248d36f79f3e518f1
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/019/home_city"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_019 · gmv1_3a9a18bf3cf5dfad5d5c

- [ ] **Etiqueta:** positiva; pregunta: ¿Dónde guarda copias user ficticio 019?
- **Esperado:** user ficticio 019 guarda copias de sus documentos en servidor doméstico.
- **ID esperado:** gmv1_c1246c019e7f93e2eb26
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/019/backup_rule"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_019 · gmv1_ef471eecbc59251cb49f

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es el color favorito de user ficticio 019?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_020 · gmv1_30b500148e3b93d27050

- [ ] **Etiqueta:** positiva; pregunta: ¿Cuándo reserva user ficticio 020 tiempo de concentración?
- **Esperado:** user ficticio 020 reserva los martes para trabajo concentrado.
- **ID esperado:** gmv1_35b49dcba0927c3fa687
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/020/focus_window"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_020 · gmv1_cae73d18b83219a10dda

- [ ] **Etiqueta:** positiva; pregunta: ¿En qué ciudad ficticia vive user ficticio 020?
- **Esperado:** user ficticio 020 vive en la ciudad ficticia de Valle Nube.
- **ID esperado:** gmv1_610510efcdc2b4e14bc4
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/020/home_city"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_020 · gmv1_59af6e6dc8abea47dd5e

- [ ] **Etiqueta:** positiva; pregunta: ¿Qué tono prefiere user ficticio 020 en las respuestas?
- **Esperado:** user ficticio 020 prefiere respuestas de tono didáctico.
- **ID esperado:** gmv1_39e39a106533a418a787
- **Condición declarada:** null
- **Fuente declarada:** {"kind": "user", "ref": "synthetic://general-memory-v1/user/020/writing_tone"}
- **Revisión:** pendiente; observaciones: __________

### synthetic_user_020 · gmv1_2ec5b6829410b624e70f

- [ ] **Etiqueta:** negativa; pregunta: ¿Cuál es el código secreto de user ficticio 020?
- **Esperado:** sin recuerdo aplicable; semilla vacía
- **ID esperado:** ninguno
- **Revisión:** pendiente; observaciones: __________
