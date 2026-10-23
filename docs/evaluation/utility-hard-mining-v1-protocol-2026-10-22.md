# TRAIN difícil frente a uniforme: protocolo único

Fecha:2026-10-22. Objetivo completo activo en paso3. Los32positivos f17 de la última ampliación incluían16útiles ya aceptados por el padre; una sonda controlada mostró32/32tanto padre como diverse en nominales/directos derivados de TRAIN. Agregar material fácil no probó resolver pérdidas útiles. Esta intervención selecciona grupos de TRAIN por dificultad del padre frente a selección uniforme del mismo pool, sin tocar el gold según scores.

## Pool y revisión antes de modelos

Pool NUEVO de192pares:16grupos A,16B y16C, cuatro documentos por pregunta. Cada grupo tiene2útiles del rol declarado y2none;96útiles/96ajenos. A: valor solicitado y ausencia/ilegibilidad explícita de ese campo, frente a otro campo u objeto. B: corrección explícita de atribución con controles de otro objeto. C: partes de pregunta compuesta o componentes fácticos reales del mismo total/resultado, frente a atributos/objetos ajenos. No inferir hecho desde permiso, capacidad o plantilla; la simple mención de entidad no hace útil un documento.

Autores separados con archivos propios no ven observados,sondas,reservas o resultados de modelo. Dos revisores leen preguntas/documentos y rúbrica con IDs neutrales, sin etiquetas/roles del autor ni predicciones. Exigir192/192etiquetas yroles concordantes o reconciliación explícita antes de modelos; preservar originales/ambigüedades. No forzar utilidad para obtener ejemplos difíciles. Si una revisión cambia casos, volver a revisar y congelar la versión válida antes de puntuar. Misma familia nativa,no validación humana/independencia de modelos.

Ninguno de los88observados,sondas8/32 o reservas entra en autoría/minería/TRAIN. Verificar coincidencias textuales exactas con ellos y TRAIN antiguo; no afirmar ausencia de paráfrasis semánticas cuando se comparten relaciones deliberadamente. Pool completo es material de selección TRAIN,no benchmark fresco; los no seleccionados tampoco son validación independiente por haberse observado sus scores.

## Minería prospectiva y control

Puntuar los192pares completos una sola vez con padre V5matched SHA7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74,modelo/instrucción/tokenizador/runtime FP16base yLoRAFP32 exactos. Guardar logits,margin=yes−no,score y decisión>=0,5. Cero backward/update. No versión V6posterior ni nueva temperatura/escala/umbral. Preflight técnico independiente antes del journal; congelar plan,datos/gold/revisiones,fuentes,checkpoint,tokens,política y HP futuros antes del primer score.

Cuatro grupos completos por estrato A/B/C,48presentaciones por brazo. `hard`: ordenar dentro de cada rol por número de errores útiles descendente,luego errores binarios totales descendente,luego suma de pérdida V2 descendente,desempate IDlexicográfico ascendente. Pérdida V2 por par:CE(score,target)+target*max(0,2−margin)^2; calcular CE estable desde logits/margen. No seleccionar filas sueltas ni eliminar útiles de un grupo. Elegir primeros4porrol,entrenarlos ordenados por pool_id,con el orden original positivo1/positivo2/none1/none2.

`uniform`: muestrear4grupos por rol sin reemplazo con Pythonrandom.Random(20261022),sobre IDs elegibles ordenados,consumiendo el generador secuencialmente en A,B,C. Fijar IDs antes de scores. No eliminar solapamiento con hard ni remuestrear si no conviene; registrar solapamiento. Misma cuota de roles/targets y grupos/presentaciones. Diferencias de subtipos/longitud/dominio son parte del paquete de selección,no un efecto puro de dificultad escalar.

**Precondición para entrenar**: al menos4grupos con algún error útil en A y al menos4en C; los4hard de esos estratos deben cumplirlo. B mantiene cuota4aunque no haya errores útiles,informando esa disponibilidad. Si falta soporte,no entrenar ni rebajar cuotas tras scores: conservar insuficiencia del pool y revisar la autoría/hipótesis por separado. Esta precondición impide llamar mejora de cobertura útil a seleccionar solo negativos difíciles. Mostrar errores positivos disponibles y seleccionados por estrato,además del criterio completo.

## Entrenamiento ya fijado, condicionado a soporte

Ambos brazos parten del MISMO padre exacto,con optimizador nuevoidéntico. Fondo176TRAIN sin cambio, más48seleccionados:224presentaciones,96útiles y128none; suplemento8A+8B+8C+24none. Mismas etiquetas/roles por posición entre brazos,fondo176en mismas posiciones. Permutación MT19937Pythonrandom.Random(20261022).shuffle(range224) idéntica,índices persistidos; no score en orden. Orden de ejecución uniform→hard.

Una época por brazo,224forward/backward,56updates;448backward y112updates en total. Mismos HP V2:CE+target*hinge(2−margin)^2,AdamWfresh lr1e-4,betas0,9/0,999,eps1e-8,wd0,01,biascorrectiontrue,media de4gradientes antes declipglobal1. LoRAq/v últimas8,r8scale8dropout0,32arraysFP32/327680parámetros,baseFP16inmutable,semilla20261022. No barrer épocas/penalizaciones ni elegir checkpoints intermedios. Fijar fuente de entrenamiento antes de minería; materializar/validar selección y arrays después de scoring sin cambiar algoritmo/HP,y otro preflight antes de entrenar.

Antes/final por brazo496pares: fondo176,pool192,desarrollo24,conocidos64,sondaatribución8,sondaclef/direct32. Son1984scores_eval+448trainforward/backward=2432forward de campaña, más192de minería si se ejecuta todo. Los48del suplemento y el pool no prueban generalización; conservar resultados de seleccionados/no seleccionados como TRAIN observado. Scores iniciales del mismo padre deben concordar con referencias congeladas de cada fuente dentro de0,02y decisiones iguales; no recalcular selección desde las reevaluaciones.

## Puerta y límites sin cambio

Puerta original24/24desarrollo+64/64conocidos,cero pérdidas útiles/ajenos/degradaciones y≥1ganancia frente al reranker sin adaptar histórico. Comparar también padre contemporáneo y uniformvs hard. Completar ambos brazos aunque falle calidad; un fallo técnico queda registrado y no se reinicia en el mismo directorio. Persistir cada ID/target/index/tokenhash/backward/normas/update,checkpoint antes de final y fingerprints de pesos/base. Auditor offline de datos/minería/selección/entrenamiento,con límites de tensores no persistidos.

No adopción ni reserva desde minería/fit. Paso4 de casos nuevos con revisión ciega y comparación productiva sucede solo tras superar puerta local del paso3,según objetivo del usuario; no presentar este pool como ese paso4. Recursos completos≤1,5×,recibos,estabilidad ycohorte finalV6siguen pendientes. No reducir objetivo aB:preservación A/Cobligatoria.

CLI locales sinpuerto/API. Estimación:1minmodelo de minería y4–5min de entrenamiento/evaluación tras preparación;watchdog600sminería,1200sporbrazo. Anunciar/identificar handle/PID,registrar salida real,inspeccionar mismo proceso ante pérdida de observación,sin reintento por timeout ni parar ajenos. No descargas/dependencias/servicios/DB/secretos/ramas/commits/cambiosdelproducto. NuevaAPIUSD0,coste económico local/Codex desconocido. Tiempos de minería/TRAIN no acreditan latencia/coste de ruta completa del producto.
