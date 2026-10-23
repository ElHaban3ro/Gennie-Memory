# Replay no lineal de una corrección fija V1

Intervención completada una vez sobre copia temporal del checkpoint V5 matched, coeficiente 1. El archivo padre permanece intacto. **Solo entrenamiento: no nueva validación de recuperación, comparación de producto o adopción.** El objetivo original sigue activo y las reservas permanecen cerradas.

[Protocolo previo](utility-gradient-replay-v1-protocol-2026-10-22.md). Evidencia en `outputs/iromem/evaluation/utility-gradient-replay-v1-2026-10-22/`. Plan SHA-256 `8dc5c186aa00efcf5aeffd5421c32b47c6b34cb837ed7c2c507eed1b3c1f8fe5`.

## Ajuste real dentro de muestra

| Medida de los mismos 176 train | Padre medido antes | Candidato tras intervención |
| --- | ---: | ---: |
| Decisiones correctas /176 | 155 | 174 |
| Útiles preservados /72 | 70 | 72 |
| Ajenos seleccionados /104 | 19 | 2 |

Ganancias: 19; errores nuevos: 0, incluidos 0 útiles perdidos y 0 nuevos ajenos. Todos los IDs y resultados se conservan. No elegir otro coeficiente o casos después de observarlos.

La referencia son los márgenes de entrenamiento medidos en el diagnóstico anterior, con mismo checkpoint, datos y runtime verificados; no hubo nueva corrida completa contemporánea del padre. Los 176 pares se usaron para construir delta: estos resultados miden ajuste, no generalización. La última regresión conocida del padre continúa en 59/64, 13/17 útiles y un ajeno; el candidato no se evaluó en esos casos.

## Modelo real frente a aproximación lineal

Compatibilidad por fila: 4/176; L2 relativo de error 1.34331; compatibilidad conjunta False. Tolerancias previas por fila max(0,1; 0,1·abs(cambio predicho)) y L2 relativo≤0,1, sin reajustarlas.

Error absoluto máximo 6.10156, mediano 1.32813. De los 149 márgenes fuertes protegidos linealmente, 146 cambian más de0,1 en valor absoluto; cambio firmado mínimo -6.10156 y máximo 0.9375. De los 27 déficits, 5 alcanzan margen firmado≥1,9 y 5 ≥2.

La predicción usa delta realmente aplicada después de redondeo FP32, no únicamente la propuesta FP64. Rango y factibilidad de igualdades no garantizan esta intervención finita en un modelo no lineal. Incompatibilidad del surrogate y mejora/empeoramiento del ajuste son resultados distintos; tampoco un ajuste favorable autoriza una reserva.

## Integridad, coste y límites

176 forward, cero backward y cero actualizaciones de optimizador; una intervención de parámetros fija previa a los forward. Duración 20.048 s. Hash de padre intacto; candidato y base sin cambios durante la inferencia. Auditoría independiente 9194/9194, sin volver a ejecutar el modelo.

API nueva USD0, ninguna llamada a proveedores; coste económico local y Codex desconocido. Acumulado externo sin cambios:6155 operaciones,23 campañas emparejadas completas,USD4,566379428 conocidos más24 operaciones de importe desconocido,excluyendo Codex.

Mismos 213 hashes de fuente Rust/Cargo y binario de evaluación: se heredan879 pruebas aprobadas/24 ignoradas por defecto,881/75 con HTTP/2 y formato previo, sin repetir Cargo para scripts aislados. No servicios/puertos, dependencias o pesos base nuevos, DB, secretos, env persistente, procesos ajenos, rama, commit o push.

No se cargaron desarrollo, conocidos, closed o V6 en este replay. Siguen exigidos mecanismo que preserve evidencia/rechace ajenos, validación local completa, después casos nuevos con revisión ciega y comparación vigente, recibos/coste/p50/p95≤1,5×, estabilidad y V6 final. No se cambia ninguna puerta por el resultado de esta sonda.

La auditoría final incluye salida 0 observada y fuentes revalidadas: 9194/9194, además de 307 comprobaciones previas. El primer dictamen offline previo al archivo terminal se preservó antes de añadir esas comprobaciones; no hubo más inferencias. La revisión causal admite evaluar el único candidato exacto sobre los 88 casos locales observados con protocolo separado, por la mejora real sin nuevos errores. No valida la aproximación lineal ni generalización fresca.
