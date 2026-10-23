# Diversidad de TRAIN con lectura fija: contraste emparejado

Paso 3 del objetivo de calidad; prospectivo antes de capturar las representaciones nuevas. No cambia producto, contrato, etiquetas originales o criterios de adopción. [Antecedente](frozen-utility-readout-v1-results-2026-10-22.md): las cinco cabezas convergieron y ajustaron sus TRAIN pero fallaron al excluir familias; la hipótesis comprobable es si añadir construcciones nuevas mejora transferencia, manteniendo el resto fijo.

## Datos y separación

Se preservan los 432 pares observados, su orden, textos, etiquetas, features1024 FP32 y referencia histórica. Se añaden 128 pares TRAIN en 16 familias, redactadas antes de resultados: 16 apoyo A,24 corrección B,24 aporte parcial C y64 ajenos. Revisión source-only separada sin etiquetas:128/128 concordancias binarias y de rol; citas literales comprobadas. Revisor Codex de la misma familia y con historia del proyecto; no equivale a revisión humana ni independencia de familia. Borrador, paquete ciego, revisión y conciliación se preservan sin correcciones tras puntuaciones.

El mapa extendido agrupa560filas en15componentes por procedencia, duplicados y construcciones homologadas. Cinco contienen los432targets originales; diez contienen solo80nuevos. Los48nuevos relacionados con familias originales se excluyen cuando su componente es test. La agrupación conserva las particiones originales312/72/32/8/8. Compartir rol no basta para unir familias: puentes activos y similitudes no demostradas se registran por separado. No prueba independencia semántica exhaustiva.

Cada fold evalúa únicamente las filas originales de su componente y compara dos ajustes contemporáneos:

- Control: originales fuera del componente.
- Ampliado: originales y nuevos fuera del componente completo.

Ambos requieren las dos clases en TRAIN y excluyen todo el componente de normalización y ajuste. Cada original recibe una predicción por brazo. Los nuevos128no son test fresco; sus puntuaciones base solo documentan captura. El control debe reproducir decisiones y probabilidades151 con diferencia absoluta≤1e-12, porque features, receta, orden y particiones originales no cambian.

## Una ejecución y una receta

Mismo Qwen3-Reranker0.6B original FP16 sin LoRA del151,310tensores congelados, pesos persistidos y fingerprint de parámetros ligados por SHA. Runtime aislado existente; tokenizer local, texto íntegro y mismos prefijos e instrucción. Se reutilizan432features verificadas y se ejecutan solo128forwards nuevos; cero backwards o actualizaciones del transformer. El helper de captura procede del151y vuelve a comprobar casts BF16→FP16, proyección de los mismos hidden hacia no/yes y fingerprints antes/después.

Se importa sin modificar readout.py151: media/desviación de cada TRAIN, escala1si desviación<1e-8, sesgo añadido, CE binaria media+0,01/2·||w||² incluyendo sesgo, FP64 CPU. Newton/Woodbury desde cero, máximo50actualizaciones por fit, gradiente infinito≤1e-7, Armijo1e-4 y como máximo30reducciones. Corte sigmoid≥0,5. Cinco folds×dos brazos=10fits, máximo500pasos CPU. No PCA, balanceo, búsqueda de λ, umbral, checkpoints o candidatos alternativos; no cabeza final desplegable tras fallo.

Marcador exclusivo y diario sincronizado antes/después de operaciones. Hashes de inputs, módulos, runtime, assets y fuentes se fijan en plan.json antes de inferencias; controles sintéticos no usan features reales. Preflight separado previo; auditoría posterior reconstruye exclusiones, normalización, estacionariedad y predicciones desde artefactos sin volver a ajustar ni ejecutar encoder.

## Puertas, límites y coste

El contraste de datos debe informar por brazo aciertos, útiles conservados, ajenos seleccionados, ganancias C y todas las pérdidas útiles y ajenos nuevos frente al padre histórico. El pase preliminar exige al menos una ganancia C y cero pérdidas útiles nuevas o ajenos nuevos; mejorar el promedio o el control fallido no basta. No reemplaza las puertas originales:24/24desarrollo,64/64conocidos, conservación de seis conjuntos, recursos coste/p50/p95≤1,5×base, recibos completos, estabilidad y V6ciega. Solo si procede se evalúan esas etapas separadas. Reservas cerradas y producto vigente conservado.

CLI local sin puerto, una sola ejecución estimada2–5minutos y plazo máximo900s; no llamadas remotas ni coste API nuevo. Tiempo instrumental de captura/CPU, RSS y MLX se reportan separados: no representan latencia del recorrido de producto. Energía, amortización y coste Codex desconocidos. Los432vectores cacheados no se vuelven a contar como inferencias nuevas. Ante fallo, conservar resultados y diagnosticar la causa antes de otra intervención.
