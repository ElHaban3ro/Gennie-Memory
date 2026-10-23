# Contraste de interacción pregunta–documento V1

La candidata queda rechazada: no cumple la conjunción de mejora y preservación. No se adopta ni se barre otra resta, escala, capa o corte.

| Variante | Aciertos /432 | Útiles /200 | Ajenos /232 |
| --- | --- | --- | --- |
| Referencia histórica fija | 374 (86.57 %) | 171 | 29 |
| Control raw reproducido | 336 (77.78 %) | 172 | 68 |
| Interacción 2×2 | 355 (82.18 %) | 169 | 46 |

Frente a raw: 16 pérdidas útiles nuevas, 11 ajenos nuevos y 5 ganancias C. Frente al padre: 17 pérdidas útiles, 28 ajenos nuevos y 9 ganancias C. Todos los IDs y métricas por rol/componente permanecen en los artefactos. Un total de útiles igual o mayor no equivale a preservar los mismos recuerdos.

## Procedimiento y límites

Una sola intervención sobre representación: I=h(q,d)−h(q,vacío)−h(vacío,d)+h(vacío,vacío), en CPU FP64 desde estados FP32 del mismo encoder FP16 base congelado. Se mantienen los 688 contenidos, 816 ocurrencias conditional V2, etiquetas, grupos y receta binaria151. La normalización se ajusta por representación usando exclusivamente background560 fuera del componente. Esto mide el paquete representación/geometría de regularización, no un efecto causal semántico puro. Los campos vacíos pueden estar fuera de distribución; no se les asignó gold ni se usaron como ejemplos TRAIN.

El control reproduce V2: decisiones idénticas y diferencia máxima de probabilidades 0.0. OOF excluye los componentes completos; son 432 objetivos TRAIN ya observados, no validación fresca. No se abrió la puerta original88 ni V6.

859 capturas nuevas deduplicadas (312 preguntas solas, 546 documentos solos y un vacío), más 688 features raw reutilizados por ID y hash. Los datos no se ampliaron para mejorar etiquetas. No hubo pesos nuevos, LoRA, red o cambios productivos.

## Verificación y recursos

La revisión crítica previa permitió una prueba única con límites explícitos. El primer plan y su preflight de 500 checks se preservan como rechazados: el gate global original del runner no comprobaba toda la conjunción del protocolo. Se corrigió antes de cualquier carga del modelo, conservando el runner original; el preflight final pasó 508 checks, incluidos todos los paths requeridos y la expresión exacta del gate. No se cambió la receta para rescatar un resultado.

Auditoría CPU separada aprobada: alineación por ID, interacción, normalización, objetivos, gradientes, scores, métricas, límites, pesos y fuentes congeladas. Sin refit ni encoder en la auditoría. Sus límites se conservan en audit-independent.json. Ejecución real exit0 y PID ausente, según terminal-observation.json.

Contadores: 859 forwards, 8 fits CPU y 64 pasos; cero AD/updates del transformer. Tiempo total 89.717387 s (captura 88.717001, lectores 0.336419); RSS pico 3587948544 bytes. Son recursos instrumentales, no latencia del producto ni puerta original de recursos. Cero llamadas API, USD0 externos adicionales; coste Codex/local sin cuantificar.

Los 218 archivos fuente Rust conservan sus hashes. Las suites 928default/930HTTP2 se heredan de150 y no se repiten. cargo fmt --all --check se ejecuta en el cierre. Objetivo completo activo, dos de cinco etapas cerradas.

## Decisión

La candidata queda rechazada: no cumple la conjunción de mejora y preservación. No se adopta ni se barre otra resta, escala, capa o corte.
No convertir una representación o su fit en prueba de mejora automática. Las puertas originales, reservas, recibos y presupuesto de recursos permanecen vigentes. Antes de otra ejecución se exige una hipótesis distinta ligada al fallo y un criterio de descarte previo; sin barridos retrospectivos de esta representación.
