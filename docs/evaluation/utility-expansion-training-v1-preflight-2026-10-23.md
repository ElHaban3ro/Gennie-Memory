# Preflight de expansión de entrenamiento V1

Fecha: 2026-10-23. Paso 3 del objetivo de calidad. **Resultado: no ejecutar.** La prueba se detuvo antes de cargar el modelo y antes de cualquier puntuación o entrenamiento.

Se comparó el banco de expansión congelado de 128 filas con el TRAIN base de 224 filas y con las seis particiones ya observadas de V8: `background` (176), `pool` (256), `development` (24), `known` (64), `attribution_probe` (8) y `cleft_probe` (32). La comparación de texto aplica Unicode NFKC, `casefold` y colapso de espacios; compara hashes de query, document y del par query-document. No abrió V6 ni la reserva original24.

El banco se solapa por completo con `background` y el núcleo: **128/128 IDs, pares query-document y documentos coinciden en ambos**. También hay 32 queries compartidas dentro del banco de expansión, consistentes con documentos múltiples por pool. No hubo colisiones de ID, query, documento o par con las otras cinco particiones observadas. Este resultado demuestra contaminación train/evaluación, no una mejora ni un resultado del modelo. Ninguna fila se excluye para rescatar el experimento.

El revisor crítico encontró además que el borrador carecía de una puerta explícita de deduplicación previa al entrenamiento, no fijaba igualdad verificable de inicialización aleatoria y orden efectivo entre brazos, dejaba ambiguo el criterio de ganancia de C y no enumeraba los seis conjuntos de regresión. Aunque estos puntos pueden precisarse, el solapamiento completo basta para rechazar esta versión; no se ejecutó nada del modelo.

| Medida | Resultado |
| --- | ---: |
| Filas de expansión | 128 |
| Colisión con TRAIN base, por ID / par / documento | 128/128 cada una |
| Colisión con `background`, por ID / par / documento | 128/128 cada una |
| Colisiones de par con las otras 5 particiones | 0 |
| Filas pasadas a modelo / forwards / updates | 0 / 0 / 0 |
| API / coste API | 0 / USD 0 |
| Coste local | Ninguno de modelo; el coste Codex no está cuantificado |

Los bancos `query-conditional-training-v1/v2` y `utility-readout-diversity-v1` no son sustitutos automáticos: ya tienen experimentos locales previos con sus propias puertas de calidad fallidas o propuestas expresamente descartadas. No se reabren como candidatos en esta corrida. La próxima prueba requiere datos TRAIN realmente separados y revisión ciega; el objetivo sigue activo y ningún candidato se promueve.
