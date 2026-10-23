# Agregación asimétrica: desarrollo aprobado, cohorte nueva fallida

Fecha: 2026-10-19. `roles_primary_veto_v1`, Jev explícito, dos selectores en una petición y top16. Se acepta el primer ID ante acuerdo o abstención del segundo; un segundo ID distinto veta. No es verificación independiente. [Protocolo previo](roles-primary-veto-v1-protocol-2026-10-19.md). **No se promueve; goal activo.**

## Métricas

Cada celda muestra base → candidata. Son IDs permitidos por la rúbrica, no respuestas finales ni verdad poblacional.

| Medida | v4 conocida | v2 conocida | v5 nueva al llamar |
| --- | --- | --- | --- |
| Apoyo útil |16→16/16|16→16/16|16→16/16|
| Refutación útil |13→16/17|16→16/16|14→15/16|
| Contexto parcial útil |14→16/16|13→16/16|15→14/16|
| Ausencias correctas |13→15/15|16→16/16|15→16/16|
| IDs no permitidos |2→0|0→0|1→1|
| Pérdidas bloqueantes |0|0|3|
| Nuevas evidencias útiles |5|3|3, una tras fallo técnico base|
| Clasificaciones degradadas |0→0|0→0|1→0|
| p50 ms |380,177→378,591|370,982→361,611|399,860→366,274|
| p95 ms |708,724→516,705|705,359→512,839|716,187→416,274|
| Coste conocido USD |0,004501500→0,005441310|0,003987700→0,005069484|0,003428400→0,004924626|
| Calidad |Pasa|Pasa|Falla|
| Recursos |Pasa|Pasa|Falla: recibo ausente y base degradada|

384 operaciones/peticiones terminaron. Coste conocido de estas tres campañas: **USD0,027353020 más un importe desconocido** de una llamada base con HTTP503, sin recibo; no se supone coste cero. No incluye Codex. V5 tiene64 clasificaciones completas de candidata y63 completas de base; las128 operaciones tienen evento terminal. El p95 incluye los64 tiempos por política y no una integración automática.

Los ratios de recursos en desarrollo pasaron el límite1,5: coste1,209×/1,271×; p500,996×/0,975×; p950,729×/0,727×. En v5, el ratio de coste completo es desconocido; no usar el cociente de subtotales conocidos como coste relativo completo. Se conserva el informe inicial con ese campo y el corregido, verificando que es la única diferencia. La candidata no tuvo fallos técnicos y el fallo de base no explica sus tres regresiones de calidad.

## Qué falló en casos nuevos

- `cev5q_066_06`: firma pendiente de pago y decisión de tesorería pendiente son dos memorias permitidas y compatibles. Cada voto eligió una. La regla de veto vació la semilla y perdió evidencia de base.
- `cev5q_067_06`: humedad/aplastamiento pendiente y sellado hermético son partes complementarias permitidas. Dos IDs distintos no eran contradicción; se perdió otro parcial.
- `cev5q_071_04`: ambos votos eligieron presentar carné para reservar Olmo, pero eso no niega que haga falta avalista. La memoria que elimina explícitamente ese requisito estaba disponible y la base la eligió. Es evidencia insuficiente para la premisa, no un error de referente ni falsedad final demostrada.

Las tres regresiones bastan para bloquear, sin compensarlas con ganancias. El crítico posterior contrastó fuente y gold y defendió conservarlos. Esa revisión posterior no se presenta como ciega. La revisión ciega original entre agentes de la misma familia y las tres conciliaciones de gold anteriores a llamadas están preservadas y ligadas por hash.

Hay además una limitación del gate automático de completitud: su escala de relación —apoyo/refutación frente a parcial— no mide cada parte solicitada. En `072_05`, ambos IDs están permitidos como parcial, pero la base informa dos esquinas tratadas y dos pendientes mientras la candidata solo describe la superior izquierda. No se modifica el score congelado retrospectivamente; se registra que igualdad de tier no prueba igual cobertura de datos. Un banco futuro deberá anotar esa cobertura por ID antes de llamar, con revisión separada.

## Trazas y verificaciones

V4:48 acuerdos,15 abstenciones dobles,1 conflicto. V2:47 acuerdos,16 abstenciones dobles y1 propuesta aceptada sin segundo voto —la refutación del acceso al farallón—. V5:46 acuerdos,16 abstenciones dobles,2 conflictos y ninguna aceptación sin segundo voto. El acuerdo equivocado de Olmo muestra que el problema nuevo no deriva de aceptar [ID,none]. Los dos conflictos v5 contienen ambos IDs permitidos.

[Desarrollo v4](../../outputs/iromem/evaluation/roles-primary-veto-v1-development-2026-10-19/cohort-v4/report.json), [desarrollo v2](../../outputs/iromem/evaluation/roles-primary-veto-v1-development-2026-10-19/cohort-v2/report.json), [cohorte nueva](../../outputs/iromem/evaluation/roles-primary-veto-v1-fresh-2026-10-19/cohort-v5/report.json), [diagnóstico por caso](../../outputs/iromem/evaluation/roles-primary-veto-v1-fresh-2026-10-19/failure-analysis.json). Se auditaron diarios, búsquedas, semillas y hashes; no fuentes externas ni respuestas finales.65pruebas workspace pasaron antes de campañas y29del evaluador tras corregir el ratio desconocido. Las fuentes de código previas se conservaron en snapshots del experimento. La primera búsqueda local rechazó un sobre con campos auxiliares; se corrigió solo el adaptador de consultas, sin cambiar corpus/gold/política y antes de llamadas.

## Decisión y siguiente mecanismo

Conservar el fallo y baseline; v5 pasa a desarrollo observado y no puede volver a presentarse como nueva. El contrato se aclara: diferencia de ID no prueba incompatibilidad y requisito relacionado no prueba exclusividad. La siguiente hipótesis es evaluar cada recuerdo respecto al dato solicitado o una corrección explícita, admitiendo varias evidencias útiles sin competición artificial. Debe fijar selección/composición entre positivos, cobertura de partes, formato, coste y latencia antes de otro contraste. Una nueva versión necesita otro banco nuevo con revisión ciega; no hay aprobación ni garantía general en estas cifras.
