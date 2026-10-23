# Referencias e identificadores compactos sin cambiar criterios v3

Protocolo previo a llamadasV3. ScopedV1 pasó calidadV5completa y falló coste1,5539×. ScopedV2 redujo coste6,36%en piloto y añadió un ID no permitido (tapa ante pregunta por acometida), por lo que se detuvo. Un contraste pequeño no establece causalidad determinista del cambio de texto; no se rescataV2.

ObjetivoV3: reducir coste preservando calidad deV1y su texto literal de preguntas/reglas/criterios. `per_memory_scoped_v3` deriva deV1: alias técnicos memory_00→m0, pregunta memory_00_fact→m0_fact y memoria anidada state.memories.memory_00→state.m0. Se conservan consulta, ámbito, resúmenes, orden, modeloJev1.13,16choices/top8, opciones originales completas y recibo único. Una prueba revierte los alias/estructura y exige igualdad exacta del payload original. El parser remapea solo nombres de respuestas; no inventa elecciones/probabilidades/identidad y conserva ID/revisión de candidatos. La traza scoped_checks y revalidación son las deV1. Menos estructura no garantiza menos tokens ni invariancia semántica del modelo: se miden ambos.

Piloto conocido20operaciones alternadas, controlScopedV1 y candidataScopedV3sobre las mismas diez consultas/goldV5. Primer par2para formato; si completo18restantes en mismo diario sin repetir. Congelar fuentes/corpus/manifest/casos/protocolo/binarios antes de llamar. CLI sin puerto, DB temporal propia compartida con campañas scoped previas,60s previstos, estimaciónUSD0,002. Recibos reales, sin reintentos inciertos.

Puerta de coste: cero pérdida de ID útil deV1, ID no permitido o degradación; costeV3estrictamente menor y coste/p50/p95<=1,5×, recibos completos. No exige una ganancia adicional de calidad en este piloto de optimización. El objetivo global no cambia.

Si pasa, campañaV5completa128operaciones contra Decisions/baseline con puerta global intacta: cero ID no permitido, pérdida de evidencia útil/completitud/partes disponibles de la base o degradación nueva, alguna mejora, recibos completos y coste/p50/p95<=1,5×. Comprobar además que no se pierda ningún ID útil anotado deScopedV1en los mismos64casos, usando reportes/casos/auditorías vinculados por hashes. No comparar tiempos no emparejados entre versiones como prueba causal.

Si pasa todo, continuarV4/V2conocidas y solo después V6congelada/revisada aparte. Un fallo detiene la expansión de esa versión. No retocar gold, repetir hasta obtener pase, relajar1,5×ni ajustar sobreV6. No hay dependencias, secretos, SQL, puertos o defaults nuevos. Codex programación/revisión no incluido en costes del clasificador.
