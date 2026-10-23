# Entrenamiento con familias nuevas: contraste de expansión V1

Fecha: 2026-10-23. Paso 3 del objetivo íntegro. **Estado: abortado antes de cargar el modelo por solapamiento completo con `background`.** Véase [auditoría de preflight](utility-expansion-training-v1-preflight-2026-10-23.md). No es validación independiente ni cambia el producto, etiquetas históricas o reservas.

## Hipótesis

La expansión conciliada de cuatro familias nuevas puede enseñar distinciones útiles que el TRAIN previo no cubría suficientemente. El contraste mide específicamente el valor de esas fuentes nuevas frente a repetir ejemplos previos con el mismo coste de entrenamiento. No asume que más datos mejoren calidad y no reemplaza las pruebas de slot ni demuestra que la cobertura semántica sea completa.

## Entradas y brazos

La prueba de separación es una condición de admisión, no una métrica posterior. Esta versión falla y no se ejecuta: las 128 filas propuestas aparecen en `background` con los mismos IDs y pares query-document; por tanto no se pueden usar en entrenamiento para medir rendimiento en esos conjuntos. El archivo final se conserva como evidencia histórica, pero queda excluido de cualquier entrenamiento asociado a V8. No se filtran filas para rescatar el plan.

- `replay_control`: núcleo 224 más 128 repeticiones deterministas, muestreadas del núcleo sin reemplazo en la segunda exposición (48 útiles y 80 irrelevantes para igualar la distribución binaria de la expansión). Los IDs de instancia son nuevos, pero query/document se conservan byte por byte.
- `expanded`: el mismo núcleo 224 más las 128 filas nuevas f13–f16.

Cada brazo tiene 352 exposiciones, 144 útiles y 208 irrelevantes. El control conserva el número de exposiciones, proporción de clases, pasos y optimización; la intervención es sustituir repeticiones por textos de familias nuevas. La selección de filas del control y el orden común se fijan con semilla `20261023`, sin mirar el resultado del modelo.

## Ejecución y análisis congelados

Una época, 352 forwards de entrenamiento, 352 backwards, acumulación 4 y 88 pasos AdamW por brazo. Se conserva la receta fija V8 de LoRA: rango 8 en q/v de las últimas ocho capas, FP16 base/FP32 adaptador, AdamW `lr=1e-4`, `betas=[0.9,0.999]`, `eps=1e-8`, `weight_decay=0.01`, corrección de sesgo y recorte global 1. Se usa la loss V2 CE más margen positivo 2 para útiles; no hay teacher, penalización de preservación, búsqueda de hiperparámetros, umbral, checkpoint ni reintento. La salida binaria usa el corte fijo 0,5. El modelo recibe solo query y document; label, role, familia, IDs de instancia y rationale quedan fuera del payload/tokenización.

Primero se puntúa padre, control y candidato únicamente en background176 + pool256 (432 pares ya observados). No se carga la otra partición evaluativa hasta comprobar la puerta primaria. El candidato solo avanza si, frente al control y al padre, gana al menos un útil C del pool, no pierde ningún útil antes seleccionado y no añade ningún `none`; cualquier salida no finita o fallo de integridad detiene la calificación y conserva el artefacto.

Si supera esa puerta, se puntúan las 24 consultas de desarrollo, 64 conocidas y 40 sondas ya observadas, sin alterar sus etiquetas. Se mantienen las puertas originales: desarrollo 24/24 y conocidos 64/64, cero pérdidas/ajenos/degradaciones frente al padre en los seis conjuntos, y al menos una ganancia frente al reranker local histórico no adaptado. Las sondas son preservación secundaria, no reserva. Si cualquier puerta falla, se rechaza el candidato y no se abre reserva original24 ni V6. Aunque pase, esto solo permite preparar la etapa 4: faltan casos nuevos con revisión ciega, ruta de producto completa, recursos, recibos, latencia y estabilidad.

## Instrumentación, recursos y límites

Se preservan fuentes, hashes, orden, IDs originales/instancia, tokens, logits, score, margen, errores por ID/rol, latencia por forward, tiempo de entrenamiento, pico de memoria, huellas antes/después de pesos y artefactos de ambos brazos. Control y expansión comparten exactamente el primer núcleo y los 128 slots adicionales del calendario; se registran las posiciones y texto/token hashes para permitir auditoría. Sin llamadas API, base de datos, secreto, servicio, descarga, cambio en runtime/default, rama, commit o push.

Coste API previsto USD 0. El coste local y Codex no se declaran cero. La estimación de MLX es 8–12 minutos para ambos brazos, más preparación y auditoría; no equivale a latencia de producto ni a presupuesto end-to-end. Si la infraestructura de runtime fijada no está disponible o difiere, no instalar ni cambiar versiones: detener preflight y preservar el fallo técnico.
