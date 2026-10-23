# P3: selección B/C con apoyo A idéntico en ambos brazos

Fecha:2026-10-22. Experimento NUEVO del paso3,adaptativo respecto a datos TRAIN deP1/P2 yprospectivo respecto al entrenamiento. [P1](utility-hard-mining-v1-protocol-2026-10-22.md) y[P2](utility-hard-mining-v2-protocol-2026-10-22.md) quedan fallidos por soporteA0con exigenciaA≥4/C≥4; **ninguno se entrenó ni pasa ahora**. No modificar esos archivos,etiquetas o flags. Sus datos TRAIN ya observados no son validación fresca.

La crítica de diseño independiente acepta una nueva pregunta B/C con controles A compartidos,con límites. P2contiene64útilesA ya aceptados,18/32útilesB y19/32C;27omisiones útiles están en B/C. A tiene además9ajenos seleccionados y los anchors uniformes solo cubren uno: no asumir que Aestá resuelto o protegido.

## Tratamiento y elegibilidad P3

Pregunta:manteniendo EXACTAMENTE los mismos cuatro grupos A en ambos brazos,¿seleccionar cuatro grupos B ycuatroC con errores útiles aporta calidad frente a cuatroB/cuatroCuniformes?

Usar únicamente poolP2de256 con gold/revisiones congelados antes de sus scores y el padreV5matched exacto7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74. Reutilizar sus scores verificables para selección TRAIN; no nueva inferencia de minería ni afirmar que la política P3 precedió esos scores. Fijar este protocolo,código,grupos,HPyreglas antes de cualquier actualización P3.

- A: los4uniformes ya fijados enP2 son anchors IDÉNTICOS en ambos brazos. No los llamar hardA ni afirmar cobertura de los9ajenos.
- `uniform`: conservar IDs uniformesP2B/C,sin sorteo nuevo o rerolls.
- `hard`: porB/C,primeros4del orden P2 errores útiles descendente,errores totales descendente,pérdidaV2descendente,IDascendente. Grupos completos,no filas sueltas.

ElegibilidadP3:NUEVOmínimo4grupos con error útil enB y4enC,actualmente12y12; equivalencia exacta de anchors Aen ambos brazos;cuotas yfuentes válidas. Esto no satisface la elegibilidad A/CdeP1/P2. Declararlo en selección/plan yreportar P1/P2false. La intervención se limita a B/C,pero su aceptación exige calidad/retención también enA ynone; no reducir el objetivo global a B/C.

Suplemento48=8A+8B+8C+24none. Fondo176idéntico;224presentaciones,96útiles y128none. Mismos roles/targets en cada posición,fondo176en las mismas posiciones. Índices MT19937Python20261022congelados y orden de grupos porIDantesdepermutar;sin score en orden. Reportar solapamiento B/C yA compartidos.

## Presupuesto y ejecución constantes

Dos brazos uniform→hard,desde el MISMO padre,pesos yAdamWfreshidénticos. Unaépoca,224forward/backwardy56updatesporbrazo. LossV2CE+target*max(0,2−margin)^2;lr1e-4,betas0,9/0,999,eps1e-8,wd0,01,biascorrectiontrue,media4gradientes antes de clipglobal1. No escalas/épocas/semillas/checkpoints alternativos. BaseQwen3-Reranker-0.6BFP16,32LoRAFP32q/vúltimas8,r8scale8dropout0,327680parámetros ymisma instrucción/tokenización/runtime/corte0,5.

Antes/final porbrazo560pares=fondo176+pool256+desarrollo24+conocidos64+sondas8/32;2240scores_eval+448trainforward/backward=2688forwardy112updates. Poolincluye seleccionados/no seleccionados,está observado yparteentrenado,no validación nueva. Cada estado inicial debe reproducir referencias del padre(cachedP2yfuentesanteriores):score≤0,02ymismasdecisiones. No usar la repetición para recalcular selección. Completar ambos brazos aunque falle calidad,salvo fallo técnico;no reiniciar intentos inciertos.

Preflight independiente antes de inferencias de entrenamiento;guard deeligibilidadP3ySHAexactos de selección/plan antes de journal. Persistir IDs/targets/indices/tokenhash/backward,pérdidas/componentes/normas/updates;finitud,fingerprints base/LoRA ycheckpoint final ligadoantesdeevaluación. Fuente de evaluación/HPfijada antesdeactualizar. Auditor independiente de toda la cadena,con límites de tensores no persistidos.

## Reglas de calidad y decisión anteriores al entrenamiento

1. **Puerta original intacta**:24/24desarrollo y64/64conocidos,cero pérdidas útiles/ajenos/degradaciones y≥1ganancia contra reranker sin adaptar histórico. Comparar también padre contemporáneo yuniform/hard;no confundir referencias históricas con el productoFTS5/Decisions.
2. **Preservación suplementaria**: en cada uno de los seis conjuntos medidos,ninguna evidencia útil aceptada por el padre puede perderse yningún none previamente rechazado puede añadirse. Informar daños porID/rol/inicial/final aunque los agregados mejoren. Este guard añade protección,no reemplaza ni rebaja la puerta original. No afirmar que preservar al padre corrige sus errores anteriores.
3. Un brazo solo habilita la siguiente validación si pasa1y2yhayintegridad completa. Si ninguno pasa,rechazar ambos. Si solo uno pasa,congelarlo para la siguiente etapa. Si ambos pasan,preferir uniform porreglaprevia;no haypruebafresca de que el coste de minería adicional compense. Una mejora relativa al otrobrazo,no alpadre/noalabase,no es pase.
4. Ningún pase local autoriza adopción. Siguen caso nuevo con revisión ciega tras paso3,comparación de ruta productiva completa,recursos/recibos/p50/p95≤1,5×,estabilidad yreserva finalV6. Reservas original24 yV6cerradas ahora. No entrenar88observados,sondas8/32 nireservas;no fabricar paráfrasis de sus fallos o cambiar gold.

Resultado exploratorio adaptativo sobre TRAIN,no causa pura de dificultad ni garantía poblacional. Las diferencias de composición/lexicodominios son parte del paquete de selección. Los anchors A no se presentan como protección demostrada.

## Coste y procesos

CLIpropios sinpuerto/API:estimado4–5minmodelos,secuenciales,watchdog1200sporbrazo. Registrar handle/PID/terminal real;inspeccionar mismo proceso si se pierde observación,sin reinicio por timeout ni detener procesos ajenos. Sin descargas/dependencias/servicios/DB/secretos/ramas/commits/producto. APIUSD0,Codex/local económico desconocidos;no afirmar gratuidad. Tiempos de TRAIN/forward no miden coste o latencia de ruta completa.
