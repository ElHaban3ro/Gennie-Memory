# Definiciones compartidas: ahorro mínimo, fallo en casos difíciles

Fecha UTC: 2026-10-20. La candidata conserva el pase mínimo de 64 roles y reduce su coste, pero **falla calidad y coste en los 18 casos difíciles**. No se incorpora una política nueva ni cambia `baseline`. El objetivo sigue activo en el paso 3; V6 permanece sin llamadas.

## Banco y contrato

Se conciliaron las cuatro diferencias de rol de las dos revisiones ciegas, con crítica informada de la propuesta y de los 126 roles enumerados. Se preservaron objeciones y ambigüedades, incluidas equipo/persona, programación/confirmación y lectura temporal de un total aún pendiente. La última revisión conoce la propuesta; no se presenta como otra revisión ciega. Todos los revisores comparten familia de modelo y no hay adjudicación humana.

El nuevo banco **de desarrollo** `tests/fixtures/evidence-contribution-known18-v1` mantiene exactamente las 18 preguntas, 126 textos, orden, IDs y revisiones originales. Sus 17 contribuciones anotadas difieren de los 19 IDs históricos permitidos: excluye tres recuerdos que no acreditan el atributo pedido —comprobación genérica, cita futura y muestra preparada— e incluye el depósito que alimenta Laurel como fuente intermedia parcial. La [conciliación](contribution-known18-adjudication-2026-10-20.md) limita lo que puede afirmarse desde cada fuente. El banco se congeló antes de este contraste, conservando la propuesta anterior y las correcciones de anotación solicitadas por el crítico.

No se modificaron gold ni resultados V1–V6 y no se recalcularon sus porcentajes. El reporte nuevo conserva ambas concordancias. Corregir una etiqueta no cuenta como mejora del modelo. Un rol canónico es una convención de evaluación con límites visibles, no prueba de que las lecturas alternativas desaparezcan.

## Resultados medidos

La variante compartida traslada las definiciones largas a `state.role_definitions`, manteniendo reglas, analogías, fuentes, esquema y modelo. Su reversión recupera exactamente el payload V3. En las mismas 16 preguntas mínimas pasó **64/64 roles**, conservó **17/17 contribuciones** y no añadió ninguna selección ajena. Coste: **USD 0,001451520**, frente a **USD 0,001569792** anteriores: **7,53 % menos** y 34.560 frente a 37.376 tokens de entrada. Las tandas no fueron contemporáneas ni emparejadas; no se atribuye causalidad de latencia a esta diferencia.

Después se ejecutaron 18 pares, con orden alternado, un cliente reutilizado y HTTP/2 observado en las 36 respuestas. La base usa su constructor real de payload con hasta 20 candidatos; Jev recibe los siete anotados. Un ID del control fuera de esos siete habría quedado «no revisado» y bloqueado la calificación; no ocurrió.

| Métrica, banco difícil revisado | Decisions / base | Jev / compartida |
| --- | ---: | ---: |
| Contribuciones útiles conservadas | 11/17 | 16/17 |
| Selecciones ajenas entre pares revisados | 1 | 4 |
| Errores de rol canónico | No clasifica roles | 5/126 |
| Mediana de clasificación | 368,19 ms | 304,72 ms |
| p95 de clasificación | 1.156,00 ms | 525,13 ms |
| Coste informado | USD 0,001119900 | USD 0,002087568 |

La candidata añade cinco contribuciones útiles frente a la base y no pierde ninguna de las que esta seleccionó, pero eso no compensa cuatro selecciones ajenas y una omisión restante. La concordancia de rol es 121/126; 14/18 preguntas tienen todos sus roles canónicos correctos. Sigue habiendo dos selecciones ajenas incluso contra el gold histórico original, además de los desacuerdos de la nueva rúbrica.

Coste/mediana/p95 respecto de base: **1,864× / 0,828× / 0,454×**. La puerta de recursos exige los tres valores ≤1,5×; falla coste. Haría falta reducir aproximadamente **19,5 %** del coste de la candidata en esta tanda, además de corregir calidad, para cumplir ese presupuesto. Los tiempos incluyen petición, lectura, JSON y validación; no búsqueda, composición ni respuesta final de `recall`. Con 18 observaciones el p95 por rango más próximo es el máximo: se conserva la petición más lenta. Una corrida no demuestra estabilidad ni latencia poblacional.

## Errores y siguiente hipótesis

- Asignó A al archivo de prueba borrado a los 14 días, sin que la fuente lo vincule a los crudos de M4 o a su política de retención.
- Asignó B a una negación sobre Neblina al preguntar por Luciérnaga: transferencia de referente.
- Asignó A a una comprobación genérica y a una cita futura al preguntar por autonomía. Sus ambigüedades quedaron documentadas antes de red; el rol canónico none no acredita que nunca se hayan hecho pruebas.
- Omitió la cláusula «depósito L que alimenta Laurel», pese a que el contrato conserva esa fuente intermedia sin inferir exclusividad.

Son decisiones semánticas válidas en formato, no errores de parser. Las probabilidades de varios errores son altas; no justifican un umbral que certifique verdad. Las trazas no prueban la causa interna. La hipótesis siguiente es presentar cada texto en su propia pregunta, reducir la repetición de instrucciones y exigir asociación del atributo pedido en esa fuente. Retirar contexto global puede perder correcciones, como ocurrió en un piloto anterior: habrá que preservar el pase mínimo y los contrastes, no asumir que aislamiento aparente demuestra independencia. Congelar esa representación y su protocolo antes de otra red; no repetir esta tanda para elegir respuestas favorables.

## Coste, integridad y alcance

Tanda nueva: **52 operaciones / 52 HTTP, USD 0,004658988**, sin cargos desconocidos ni llamadas inciertas. Incluye el filtro compartido y ambos lados del contraste difícil. Acumulado del objetivo: **5.441 operaciones, 23 campañas emparejadas completas, USD 0,560691512 conocidos + 4 operaciones históricas de importe desconocido**. Excluye programación y revisiones de Codex. Los tokens informados son del proveedor remoto.

Hubo un fallo local de preparación al insertar una huella de reporte en un mapa: se corrigió y conservó la preparación parcial, antes de preflight o diario, con **cero llamadas/coste remoto**. No se volvió a ejecutar ninguna llamada incierta. Las fuentes originales, hashes de V6 y snapshots 001–035 permanecen intactos.

**237 pruebas pasan con y sin HTTP/2**, formato y compilación pasan. Se comprobaron reversión de payload, correspondencia de índices/ID/revisión, esquema/probabilidades, concordancia histórica visible y p95 sin retirar extremos. Los reportes de esta tanda se reconstruyen exactamente desde los diarios y código ejecutado. La comprobación estructural no prueba semántica.

Artefactos principales: `outputs/iromem/evaluation/contribution-role-shared-v4-2026-10-20`, `contribution-role-shared-cost-2026-10-20.json`, `contribution-shared-known18-paired-v1-2026-10-20` y `quality-goal-2026-10-19/snapshots/036`. No se actualizan las gráficas del README con este resultado de desarrollo fallido. Siguen pendientes mecanismo elegible, runtime explícito, campañas completas, estabilidad y casos nuevos revisados antes de adopción.
