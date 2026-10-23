# Unidad de evidencia: contraste de instrucción

Fecha:2026-10-22. Paso3 del objetivo íntegro activo. [V8](utility-learning-v8-results-2026-10-22.md) no superó calidad: ambos conservaron191/191TRAIN protegidos, y el término nuevo no evitó daños externos. Diagnóstico posterior sobre80paresTRAIN: revisión ciega nueva80/80labels/roles,0ambigüedades; Ccompuesto10/16 ycomponente9/16,decisiones iguales enpadre/control/preserve. No demuestra causa de completitud ni generalización.

## Intervención única

El control es la instrucción exacta V8/V5parent. `utility` antepone exactamente:

> Evaluate the document as one piece of evidence for a later answer, not as a standalone complete answer. One actual requested part or one explicit component of a requested total can be useful even when the other parts are absent and the total cannot be computed.

Después va **íntegro y sin modificación** el control. Este ya menciona parcialidad: no se presenta como contrato semántico nuevo, requisito antes ausente o causa comprobada. Contraste del paquete de énfasis/prefijo, no de completitud pura: cambia texto/longitud/posición y puede afectar otras relaciones. No ejemplos, nombres de casos, números de respuestas, búsquedas de prompt, umbrales o reglas porID.

Mismo padreV5matched7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74/baseQwen3-Reranker0.6BFP16+LoRAFP32r8q/vúltimas8. Todo congelado y sin autograd/optimizador. Una cargaGPU,modo eval, mismas preguntas/documentos originales exactos,tokens sin truncamiento y corte≥0,5. Lo único distinto del input es `<Instruct>`. No integrar producto/defaults. Protocolo/datos/tokens/runtime/código/revisión/criterios porhash antes del modelo.

## Etapas y presupuesto previos

1. Puntuar `control` → `utility`, cada uno background176+pool256=432TRAIN observados. Total864forwards/scores,0backward,0updates. Control reproduce referencias históricas por score≤0,02ymisma decisión. Reportar logits reales/márgenes/score,tokens ytiempos porID.
2. Gate adicionalTRAIN: almenos1útilC quecontrolomitíaahoraaceptado, **cero** nuevas pérdidas útiles delcontrol y**cero** none nuevos seleccionados enambosconjuntos432. Son transiciones porID, no solo agregadoC. No preservar errores delpadre comoverdad. Si falla,rechazar variante yno puntuar desarrollo/conocidos/sondas; declararlos no reevaluados, no completar métricas inexistentes. No modificar instrucción/reintentar por calidad.
3. Solo si pasa2: cargar datos/tokenizados condicionales128=(desarrollo24+conocidos64+sondas8/32); puntuar control128→utility128. Máximo256forwards adicionales. Totalmáximo1120scores,0backward/updates. Los128son ya observados; esta barrera evita reevaluación innecesaria, no los convierte en reserva fresca. Preparador y auditor pueden verificar archivos/hashes condicionales, pero runner no parsea esos textos hasta gate2. Reservas realoriginal24yV6finalcerradas.
4. El gateTRAIN **añade**, no reemplaza, la puerta original24/24+64/64,cero pérdidas/ajenos/degradaciones y≥1ganancia contra reranker sinadaptar histórico; exige también preservación suplementaria en los seis conjuntos frente alcontrol contemporáneo. Solo si todo pasa, congelar utility para posterior validación; nunca adoptar ahora. Comparación histórica reranker no equivale a productoFTS5/Decisions. Si no se puntúan128, puerta original queda no medida, no nueva cifra fallida o exitosa.

La hipótesis es adaptativa a diagnósticoTRAIN ya observado; no se afirma prospectiva antes de esos datos. Una sola candidata se fija antes de los scores de este contraste. Tras puerta local siguen casos nuevos con revisión ciega y producto completo,recursos/recibos/p50/p95≤1,5×,estabilidad yV6final. Sin alterargold,fuente,clasificaciónoriginal,README ni gráficas productivas por un resultado descartado.

## Verificación y límites

Preflight nativo separado antes de inferencias. Verificar instruccionesexactas,tokens/control contra V8,revisión/datasets,modelo/runtime/padre32arrays ybase, nofuncionesAD/updates enrunner. Persistir before/after fingerprintstodaslaspesas ypadrefile,sin mutation. Auditar diarysecuencial1728+ready+gate+completed=1731eventos enrechazoTRAIN,o2240+3=2243eventos si seabrecondicional; cualquier fallo técnico se conserva separado. Hashes,tokenID,logits→margen→score,gateporID,reportes yterminal0 se verifican offline sin nuevosmodelos. Fingerprints fuente/binario213sin cambios: heredar pruebas Rust de mismas fuentes, no repetirCargo por scripts.

DatosTRAIN/conocidos ya observados,no validar generalización con ellos. Scores no calibrados; tokens corresponden al clasificador local, no tokens enviados aCodex. Tiempos incluyen render/hashing/journal/sync, no son latencia productiva ni ahorro end-to-end. Cero inferenciasAPI; costes Codex/local económicos desconocidos.

Un CLI propio sinpuerto,estimado3–5min,deadline1200s. Anunciar propósito/puerto antes de ejecución,conservar PID/handle y observar mismo proceso,sin reiniciar por timeout deobservación. No detener procesosajenos. Sin servicios,deps,descargas,DB,secretos,ramas,commit/push.
