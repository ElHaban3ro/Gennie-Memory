# Utilidad V2: contraste de pérdida con preservación de positivos

Fecha: 2026-10-22. Protocolo prospectivo, una corrida desde los pesos base originales; no continúa el adaptadorV1. V1falló:16→8/17útiles conocidos pese a reducir ajenos10→1. Los136scores bajaron. La derivada inicial de CE respecto al margen yes−no fue mucho mayor para negativos que para positivos saturados; es evidencia de salida, no prueba de alineamiento de gradientes de parámetros. Se contrasta únicamente una pérdida alternativa que se oponga a posteriores caídas de positivos.

## Único factor experimental

Sea m=logit_yes−logit_no en FP32 y y∈{0,1}. Fijar **L=CE([logit_no,logit_yes],y)+y·max(0,2−m)²**, coeficiente1 y margen2. Para irrelevantes, pérdida/derivada idénticas aV1. Para útiles con margen≥2, término extra cero; por debajo añade derivada−2(2−m). No modifica el corte de inferencia0,5. Es una penalización, no una garantía de conservación. Como los positivos iniciales están saturados, no se afirma corregir inmediatamente el desequilibrio inicial: debe oponerse a la caída una vez cruza el margen fijado. Puede aumentar ajenos y fallar; se conservan las puertas completas.

Antes de inferir, comprobar fórmula/derivadas con aritmética independiente y autodiffMLXpara y0/1ym−4,0,1,2,4; diferencias≤1e-5, finitud y negativo sin término extra. No buscar margen, coeficiente o semilla alternativos tras medir. Ni ponderación por roles/IDs/familias, ni rankingtop1, ni calibración de scores.

## Variables conservadas y controles

Todos los demás detalles de [V1](utility-learning-v1-protocol-2026-10-22.md) quedan iguales: mismos48trainf01–f06, base Qwen3-Reranker-0.6B/revisión e61197ed45024b0ed8a2d74b80b4d909f1255473 y assets/runtimeMLXexistentes; baseFP16congelada, LoRAFP32q/vúltimas8capas/rank8/scale8/dropout0. AdamWlr0,0001,betas0,9/0,999,eps1e-8,wd0,01,bias_correction=True,acumulación4yclip1. Semilla20261022/PCG64, exactamente las mismas seis permutaciones guardadas enV1;288forward/backwardy72actualizaciones. Guardar hash del estado inicial LoRA y exigir igualdad contraV1, además de órdenes/inputs/pesos/base/instrucción/umbral. Los96datos finales siguen congelados y conciliados; nada se reetiqueta.

El control sin entrenar48+24+64y el adaptadorB=0sobre64se vuelven a medir y contrastar contraV1 con cero cambios de decisión y Δmáx≤0,02. Los inputs/tokenids/nombres/dtypes, basehashantes/después, finitud, gradiente positivo y Bfinal no cero se verifican como antes. No compilar entrenamiento ni reutilizar K/Ventre pasos. No datos o gradientes de desarrollo, conocidos o reservas al entrenar. Solo query/document llegan al modelo; target binario train a la pérdida.

Mejora de instrumentación sin cambiar el cálculo: diario UTC RFC3339con milisegundos y ordinal secuencial, evento explícito de checkpoint final con SHA y datos/hiperparámetros congelados ANTESde la primera evaluación entrenada. Hashes de módulos de modelo/loader/tuner/optimizador y protocolo/código antes de inferir. Persistir el control de fórmula antes de scores. Gradiente adicional se calcula directamente sobre los dos logits de esa misma pasada, no un segundo forward ni teacher.

## Puertas y presupuesto

Único checkpoint: final de época6, congelado antes de evaluación. Evaluar train48, desarrollo24 y conocidos64; desarrollo ahora es observado desdeV1y se etiqueta como regresión de desarrollo, no validación nueva. Comparar contra base y contraV1, incluyendo todas las ganancias/pérdidas y roles. No elegir epochs anteriores, promedios o variantes de puntuación.

Puerta intacta previa aclosed:24/24desarrollo+64/64conocidos, cero pérdidas útiles, ajenos o degradaciones respecto al gold/control y al menos una corrección. Si falla, no abrir closed24niV6, no medir recursos para promover y conservar pesos/errores. Si pasa, closed24solo se abre una vez tras congelar todo y exige24/24sin pérdida; todavía faltan pools nuevos sin útiles/con múltiples útiles, comparación contemporánea contra ruta vigente, recursos≤1,5×, estabilidad yV6según objetivo original. Un pase sintético pequeño o del ranking no completa esas puertas.

Presupuesto: una corrida estimada2–5minutos, límite duro1800s;288pasadas traincon backward+336scores de evaluación, hasta24closedsolo si pasa. Sin nuevas instalaciones, dependencias/modelos o APIs. Cero coste API; electricidad/amortización/Codex desconocidos. Fallo técnico/NaN/Inf/OOM/hash/contexto/timeout termina y se preserva sin reintento en el mismo directorio. CLI temporal sin puerto, proceso/watchdog propios, sin tocar servicios ajenos, DB,secreto,env,Rust,Cargo,rama,commitopush.

Artefactos: `outputs/iromem/evaluation/utility-learning-v2-2026-10-22/`, nuevo directorio inmutable con runner/protocolo/preparación/control fórmula/órdenes antes de scores, journal, controles, pérdidas, checkpoint, comparación y auditoría. V1 permanece congelada como control. Las restricciones del protocolo de [datos](utility-learning-data-v1-protocol-2026-10-22.md) y el objetivo original prevalecen; no hay aprobación de adopción anticipada.
