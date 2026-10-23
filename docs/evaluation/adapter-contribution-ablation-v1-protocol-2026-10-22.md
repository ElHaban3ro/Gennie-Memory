# Ablación fija de contribución LoRA

Fecha:2026-10-22. Paso3delobjetivoíntegro activo. [Prefijo deunidadde evidencia](utility-unit-instruction-v1-results-2026-10-22.md) falló:6útilesrecuperados,18ajenos nuevos,ninguna pérdida útil;no se puntuaron128condicionales. No se rescata mediante otra instrucción/umbral. Se investiga si dos estados del mismo reranker aportan señales complementarias en TRAIN, antes de proponer un mecanismo de combinación.

## Factor único y estados

ModeloQwen3-Reranker0.6BconbaseFP16 ylosmismos32arraysLoRAFP32q/vúltimas8capas,r8scale8dropout0. PadreV5matched SHA7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74. Una cargaGPU,modo eval,todosparámetrosfrozen,sinAD/optimizador. Estados enorden `parent` → `adapter_off`;432TRAINcadauno,864forwards/scores exactos.

Ambos conservan el mismo grafo LoRALinear y operaciones. Según fuente runtime fijada:`y=linear(x); z=(dropout(x)@A)@B; return y+(scale*z).astype(x.dtype)`. El checkpoint `adapter_off` se deriva porCPU **antesdeinferencias**,copiando16matricesA delpadreexactas yponiendo16B a ceroconmismaforma/dtypeFP32;no esentrenamiento. Conintermedios yresultadosfinitos, anula la contribución residual. Entradas finitas por sí solas no evitan overflow antes de multiplicar porB0. No se promete identidad bitabit con una arquitectura original sin wrappers ni se confunde quitar residual con evaluar otro modelo independiente. Si aparecen no finitos,fallo técnico,no reconstruir resultados.

Cargar32arrayspadre,medir432; cargar32arraysadapter_off,medir432; restaurar32arrayspadre alfinal sin otrosforwards. Verificar en cada estado fingerprintadapter contraarchivo,baseigual,shape/dtype/finite,Aidénticas,Boffzero,yrestorefinalexacto. Archivo padre ypesosHFbase inmutables. Las mutaciones del estadoGPU son la intervención declarada, no updates de entrenamiento. Guardar archivooff/derivación/fingerprints; no afirmar pesosidénticos entre estados.

## Datos, ejecución y estimando

Mismos432paresTRAIN fase144(background176+pool256),sinprefix,instruccióncontroloriginal íntegra,tokensoriginales,sin truncamiento,corte≥0,5. No leer/puntuar conocidos88,sondas40,reservadooriginal24 oV6final. No crear ejemplos nuevos ni cambiar gold. Padre contemporáneo reproduce referencias realescontrol144 score≤0,02/mismadecisión. No usar cachécomo nueva inferencia; aquí se ejecutan ambos estados completos.

Instrumentar el mismo LoRALinear en ambos estados, conservando exactamente orden/operadores/casts: comprobar finitud real de x,linear(x),dropout(x),dropout(x)@A,producto posterior porB,residualcast ysalida. Son7componentes×16módulos=112comprobaciones escalares porforward, evaluadas junto conlos logits,sin sincronización por capa. Verificar16llamadas y16bits por componente;fallo técnico ante cualquiera no finito. Esta instrumentación añade overhead común yno representa la ruta productiva. Prueba pequeña adicional sobre16módulosconB0,2entradasFP16acotadas:salida numéricamente igual a linear(x),finitud ydtype. No es un forward del transformer ni una prueba universal para entradas arbitrarias;las comprobaciones reales de campaña cubren sus operandos. Guardar artefacto/procedencia de la prueba antes de puntuar off, concontador separado.

Congelar código/datos/tokens/instrucción/protocolo/pesos/runtime/revisión porhash antesdemodelo. Preflightnativo separado. Persistir tokensSHA/logitsno/yes/margen/score/tiempo porID,eventos antes/después,estadosantes/después,contador864/0back/0updates,1735eventos (ready1+2×(start1+864eventosinferencia+end1)+restore1+completed1). Deadline1200s,sinreintentos porfallo técnico incierto.

## Resultado diagnóstico y falsificador

Publicar métricas **individuales** ytabla2×2dedecisiones porrol/gold:ambosaceptan,solopadre,solooff,ninguno. Goldsolo sirve para describir errores, nunca decide enpredicción. Medir exclusivosC aceptadosporoffqueelpadreomitía yelcoste de none exclusivosoff; registrar también pérdidas delpadre. Si nohayningúnCexclusivo, nohayapoyo enesta muestra paraunrescatedeCmediante decisionesbinariasfijasdeoff; no refuta todocalibrador con scorescontinuos uotrosdatos. Si loshay, soloespotencialcomplementariedad,no selector útildemostrado.

No mezclar/umbralizar/entrenar enestafase. No usar unión congold como clasificador, ni publicar precisiónoracle/ensemble ficticia o atribuir generalización. Elfactor causallocal es anularcontribución LoRA enelgrafofijo; no identifica qué ejemplo/algoritmo generó los erroresni una causa semántica de completitud. Orden/tiempos puedenafectar latencia;no inferir ahorro/recursos deunseguimiento único.

Esto **no supera la puerta original**24/24+64/64,ceroerrores/degradaciones,≥1ganancia histórica,preservación6sets; no se reevalúa aquí yqueda no medida. Cualquier selectorposterior debe serunalgoritmo realdefinido sobreTRAIN,sin decidirporID/gold eninferencia yconpuertas intactas. Siguen casosnuevosrevisiónciega/productoFTS5Decisions/recursosrecibosp50p95≤1,5×/estabilidad/V6 antesdeadoptar. Producto/defaults/README intactos.

## Coste y procesos

UnCLI localpropio sinpuerto,estimado3–5min;864forwards0AD/updates,API0. CosteCodex/local económico desconocido. Noahorro ficticio porcompartir pesosbase:unafuturadosvistas requeriríamediciónreal. Preflight y auditoríaoffline independientes,sinmodelo. Mismos213hashesRust/binario heredados,noCargo nuevo por scripts. Sinservicios/deps/descargas/DB/secretos/ramas/commits;anunciar proceso/puerto,conservarhandle/PID/terminal,no detenerprocesosajenos o reiniciar por timeout deobservación.
