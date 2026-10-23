# Propuesta por contexto y rol: resultados v5

Estado2026-10-19: **la candidata falla calidad y coste; no se amplía**. El diagnóstico fijo18conocidas se ejecutó una vez completo, según protocolo previo, sin tocar gold oV6. Default y READMEconservados. El goal sigue activo y la autorización general de pruebas está resuelta.

La propuesta ahora guarda dos juicios reales: `context_for_fact` y `evidence_kind`(dato/parte, corrección, ninguno). Un dato exige contexto; una corrección no recibe ese veto global y pasa al verificador tipadoV4. La traza `role_proposal` sustituye a scoped_proposal solo en esta versión. Contexto y tipo se validan por ID/revisión/composición tanto en servicio como en un auditor JSONaparte; no se inventan dos booleanos independientes. [Protocolo previo](per-memory-correction-verified-v5-protocol-2026-10-19.md).

| Grupo conocido | Calidad | Coste / HTTP | Latencia CLI |
| --- | --- | --- | --- |
| 13V5 | Pierde06812, previamente permitido y seleccionado.10/13sets exactos; sin IDsajenos | USD0,001212640;19HTTP;1,6596×histórico | p50374,5ms;p951330,9ms |
| 5regresionesV4 | Conserva los5IDsútiles, pero añade2ajenos.3/5sets exactos | USD0,000423544;6HTTP;1,2112×histórico | p50441,0ms;p95830,3ms |

V5no supera preservación ni el límite1,5×coste. V4pasa coste, falla los sets exigidos;5/5IDsútiles no implica selección limpia. Son latencias de campañas conocidas con una política, no comparación emparejada ni integración Codex. No se promueve el porcentaje de sets o categorías como verdad final o completitud semántica.

La traza revela el origen del fallo: ambos IDsajenos(Sarga/Lindeyevento aislado/política)recibieron contexto=trueyrol de dato. La implementación aplicó el contexto correctamente; el juicio semántico del proponente fue erróneo. Añadir un campo no fue suficiente.06812, condición explícita para publicar la copia, tuvo contexto=truepero rol=unrelated; nunca llegó al verificador y se perdió. El gold permanece intacto, sin rescatarlo por IDo reinterpretar la ejecución.

Hubo progreso acotado en las correcciones: se aceptó la autoría correcta06806, se rechazó la propuesta del libreto ajeno06807y06607se aceptó con ordinaria=false/contraste=true. Es una observación real de esa vía, no estabilidad o superioridad general demostradas. La verificación no corrige datos falsamente aceptados ni propuestas omitidas.

Se corrigió además una omisión del evaluador: el perfil de verificaciónV4estaba admitido, pero su límite7no figuraba en la tabla para futuras campañas completas. Sus pilotos históricos validaban7explícito, por lo que no se alteran sus resultados. Una comprobación de perfiles impide que V4/V5admitidas se traten como política sin límite/auditoría por recuerdo.

122pruebas workspace pasan, formato y compilación correctos. Cubren contexto solo para datos, corrección con contexto=false, roles originales, omisiones/elecciones extrañas, ID/revisión, un recibo global y auditoría independiente. No demuestran semántica correcta del proveedor. Todas las18operaciones/25HTTPtienen recibos completos: **USD0,001636184nuevos**. Acumulado: **3275operaciones/21campañas emparejadas, USD0,232082070conocidos+3operaciones históricas de importe desconocido**, excluyendo Codex. Bases temporales propias eliminadas, sin servicios ni reinicios.

Siguiente hipótesis, sin implementar ni adoptar: propuesta de rol más pequeña y comprobación aislada de cada positivo con Decisions, con tarea específica para datos/partes y otra para correcciones. La propuesta deberá conservar condiciones explícitas del estado solicitado sin inferir cumplimiento ni usar un requisito distinto como respuesta. El verificador de datos deberá comprobar sujeto/objeto/contexto y evidencia parcial, sin exigir el valor completo desconocido. El tipo de propuesta real se conservará sin fabricar un contexto=true. Hay un antecedente de verificación genérica de todos los positivos que costó2,03×y dejó un error: no se presupone que este reparto de tareas sea correcto o más barato. Primero un diagnóstico conocido acotado, todos los recibos y las puertas intactas; nada de campañas completas oV6mientras falle.

[Decisión conjunta](../../outputs/iromem/evaluation/per-memory-correction-verified-v5-development-2026-10-19/decision.json), [V5con omisiones/trazas](../../outputs/iromem/evaluation/per-memory-correction-verified-v5-pilot-2026-10-19/metrics.json), [V4con ambos IDsajenos](../../outputs/iromem/evaluation/per-memory-correction-verified-v5-v4-targeted-2026-10-19/report.json).
