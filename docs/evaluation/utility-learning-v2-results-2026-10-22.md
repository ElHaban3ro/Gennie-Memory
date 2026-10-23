# Utilidad V2: recupera positivos frente a V1, todavía pierde evidencia

La protección de positivos mejora la adaptación anterior, pero **V2 falla la puerta de calidad y no se adopta**. En los 64 pares conocidos logra 59 aciertos, conserva 14 de 17 útiles y selecciona dos ajenos. Frente a la base todavía pierde dos útiles adicionales; el porcentaje agregado no compensa esas pérdidas. Las reservas de 24 pares y V6 siguen sin abrir.

| Conjunto y medida | Base sin entrenar | V1, CE | V2, CE + protección |
| --- | ---: | ---: | ---: |
| Entrenamiento: aciertos /48 | 33 | 44 | 38 |
| Entrenamiento: útiles conservados /24 | 24 | 22 | 24 |
| Entrenamiento: ajenos seleccionados | 15 | 2 | 10 |
| Desarrollo observado: aciertos /24 | 13 | 17 | 14 |
| Desarrollo observado: útiles conservados /12 | 12 | 10 | 11 |
| Desarrollo observado: ajenos seleccionados | 11 | 5 | 9 |
| Conocidos: aciertos /64 | 53 | 54 | 59 |
| Conocidos: útiles conservados /17 | 16 | 8 | 14 |
| Conocidos: ajenos seleccionados | 10 | 1 | 2 |

La regla exigía 24/24 en desarrollo y 64/64 en conocidos sin pérdidas ni ajenos, con alguna ganancia. No se sustituye por un promedio o por elegir el mejor checkpoint. V2 se compara con la base y V1, conservando cada error individual. Desarrollo ya fue observado en V1 y es regresión de desarrollo; ninguno de esos 24 o de los 64 conocidos se entrenó.

El desglose conocido recupera B4/4 y C3/3, pero conserva A7/10. De los tres útiles omitidos, uno ya fallaba en la base y dos son pérdidas nuevas: estado explícito de formato no documentado y cantidad ilegible en una fuente. Persisten dos selecciones ajenas: opcionalidad de un campo que no acredita su valor y atributo de un referente diferente. Estos ejemplos motivan cobertura nueva; sus textos o paráfrasis no se incorporan como entrenamiento.

## Contraste ejecutado

La única diferencia matemática frente a V1 es `CE + y·max(0, 2−m)²`, con `m=logit_yes−logit_no`, FP32, coeficiente1 y corte de inferencia0,5. No se eligió ese margen mediante barrido. Base, adaptadores iniciales, datos, seis órdenes, semilla, optimizador y presupuesto coincidieron con V1; hashes y controles lo comprueban. Se arranca de los pesos originales, sin continuar los adaptadores aprendidos V1.

Las diez comprobaciones independientes de fórmula y derivadas pasaron antes de inferir. En los negativos el término extra es cero. Durante las 144 exposiciones de positivos se activó en 64 pasos, distribuidos 4/16/10/12/7/15 por época; nunca en negativos. La penalización se opone a la caída posterior al margen2, no corrige el desequilibrio inicial mientras los positivos están saturados ni garantiza una restricción dura. Recuperar útiles frente a V1 viene acompañado de más ajenos: tampoco es mejora uniforme entre ambas adaptaciones.

Se completaron seis épocas, 288 forward/backward y 72 actualizaciones, con base congelada y 32 tensores LoRA FP32. Se verificaron gradientes positivos, B final no cero, finitud y hashes base conservados. La pérdida media CE por época fue 1,187/0,527/0,557/0,488/0,467/0,349; la pérdida total incluye además la penalización y no se compara directamente con CE de V1. Entrenamiento no es validación.

Entrenamiento: 76,460s. Corrida completa: 132,849s. Son tiempos de ejecución en esta Mac, no latencia de recall, comparación controlada de recursos o velocidad de producto. Pico RSS1,581GB y pico MLX1,690GB pueden solaparse y no se suman. No se midió batch20 de los pesos aprendidos tras fallar calidad; los 518ms anteriores corresponden al modelo sin entrenar.

La evaluación añadió 336 scores: 136 de base, 64 de adaptador cero y 136 finales. Son 192 puntuaciones sobre los mismos64conocidos,96sobre48trainy48sobre24desarrollo; no336ejemplos nuevos. Junto al entrenamiento se ejecutaron624forwardsdeltransformer,288backwardy72updates. Las diez comprobaciones escalares de autodiff no son inferencias de modelo o llamadas HTTP. Proceso terminal con código0, sin fallo técnico, reintentos o checkpoint alternativo.

## Auditoría, decisión y siguiente paso

Auditoría independiente:808/808comprobaciones de integridad y concordancia, `all_passed=true` con `quality_passed=false`. Recalculó las diez derivadas, las288pérdidas desde margen/target, órdenes/estado inicial contraV1, métricas/errores/comparación, y rehasheó32arrays finales desde safetensors. Reconstruyó1251eventos UTC/ordinal, incluidos datos/órdenes/HP/checkpoint ligados antes de evaluar los pesos finales. Los hashes base registrados no sustituyen una copia posterior independiente de todos sus tensores; la limitación queda explícita. La auditoría no genera nuevas inferencias ni valida fuentes externas.

V2 queda congelada como fallida. No se sigue ajustando el peso de esta penalización para acertar el banco observado. El siguiente contraste se prepara con más datos nuevos y revisión separada: estados documentales, opcionalidad frente a hecho, identidad de referente y configuración parcial, incluyendo pools sin útiles y con varios útiles. Autores nuevos reciben rúbrica abstracta y no los textos/resultados de los bancos previos. Una expansión no prueba por sí misma mejora ni permite otra corrida sin protocolo de mezcla/presupuesto/control.

El objetivo original permanece activo en paso3. Siguen pendientes calidad completa, pools nuevos revisados, comparación contemporánea con configuración vigente, recursos≤1,5×, estabilidad y reserva V6. No cambió el predeterminado, el producto o la base de datos.

Coste API nuevo: **USD0, cero inferencias remotas**. Sin instalaciones, dependencias o pesos base nuevos. Energía/amortización/Codex no cuantificados. Acumulado externo intacto:6155operaciones,23campañas emparejadas completas,USD4,566379428conocidos y24importes desconocidos, sin Codex. Las nuevas puntuaciones locales se cuentan aparte. Rust/Cargo no cambian: las suites879default/881HTTP2se heredan únicamente tras verificar213hashes y binario idénticos, sin presentarlas como ejecución nueva.

Evidencia: `outputs/iromem/evaluation/utility-learning-v2-2026-10-22/`. [Protocolo V2](utility-learning-v2-protocol-2026-10-22.md), [V1 fallida](utility-learning-v1-results-2026-10-22.md), [expansión de datos](utility-learning-expansion-v1-protocol-2026-10-22.md). Diario, inputs permitidos, código, preflight, controles, pérdidas, checkpoint, comparación, auditoría y diagnóstico quedan preservados.

Preparación siguiente terminada:128pares nuevos,32pools de cuatro documentos,48útiles/80irrelevantes; ocho pools sin útiles y dieciséis con varios. Los128juicios ciegos coinciden en utilidad binaria. Dos dudas previas a scores recibieron adjudicación separada con casos enmascarados: encabezado de unidad sin dato sigue ajeno y un correo que además refuta una premisa admite A/B, usando Aprimario. Se conserva la anotación original B del autor y se registra la convención de rol; no cambian textos o targets binarios. Autores recibieron motivos abstractos y resumen agregado, sin textos/errores individuales previos; crítico no recibió cuotas de autoría. Datos finales congelados porSHA, sin entrenamiento o puntuación. La revisión separada no detectó duplicados exactos normalizados ni solapamiento con construcciones reservadas; no es prueba de ausencia de toda paráfrasis o generalización.
