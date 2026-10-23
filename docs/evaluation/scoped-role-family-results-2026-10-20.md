# Contraste de familias: fallos conservados y próxima hipótesis

Goal activo; ninguna familia ni variante se incorpora al producto. Se contrastó RoleV5 de una elección por recuerdo cambiando solo model: mismas18preguntas conocidas V4/V5, siete candidatos, criterios/testigo y controles Jev contemporáneos. De108HTTP previstos se ejecutaron74. ScaleDown devolvió HTTP400en su primera llamada y se detuvo esa familia;34operaciones no se ejecutaron. Microsoft y Mercury completaron18pares cada uno. No se reintentó ninguna llamada.

| Familia candidata | Operaciones ejecutadas, con controles | IDs no permitidos | Pérdida de IDs útiles anteriores | Coste candidata/control | Mediana/p95 candidata contra control |
| --- | ---: | ---: | ---: | ---: | ---: |
| ScaleDown Classify-1 | 2/36 | No medido | No medido | Desconocido | No comparable |
| Microsoft-Decision-1 | 36/36 | 4 | 0 | 0,679× | 1,533× / 2,373× |
| Mercury Decide | 36/36 | 6 | 0 | 0,416× | 1,597× / 2,191× |

Las familias completas reducen coste y conservan los IDs útiles seleccionados antes, pero calidad y latencia fallan las puertas del contraste. No se llama a eso un pase parcial que habilite adopción. Se guardan también todas las omisiones de gold dentro del prefijo; preservar selecciones anteriores no demuestra recuperar todas las partes. El catálogo y sus precios no sustituyen respuestas/revisiones/recibos reales.

ScaleDown no obtuvo ninguna decisión candidata válida. El HTTP400no aporta recibo de consumo y su causa detallada no quedó registrada: no se afirma que fuera gratis, un error concreto de formato o una indisponibilidad del modelo. El primer informe contaba las decisiones no ejecutadas como pérdidas semánticas; se conserva intacto y el informev2las marca no disponibles, con comparaciones/deltas null. No hay14pérdidas semánticas medidas de ese modelo. La corrección del auditor tiene regresión local, no añade llamadas y no cambia datos, importes ni resultados de las dos familias completas.

[Informe vigente v2](../../outputs/iromem/evaluation/scoped-role-family-contrast-2026-10-20/report-v2.json), [informe original conservado](../../outputs/iromem/evaluation/scoped-role-family-contrast-2026-10-20/report.json), [nota de reparación](../../outputs/iromem/evaluation/scoped-role-family-contrast-2026-10-20/report-notes.json).

## Consumo y tiempos realmente medidos

| Corriente completa | Coste USD | MedianaHTTP ms | p95HTTP ms |
| --- | ---: | ---: | ---: |
| Jev control de Microsoft | 0,001368276 | 305,28 | 337,35 |
| Microsoft candidata | 0,000928872 | 467,97 | 800,52 |
| Jev control de Mercury | 0,001368276 | 296,97 | 323,15 |
| Mercury candidata | 0,000569860 | 474,11 | 708,01 |

Además se pagó/registró el control inicial de ScaleDown: USD0,000078330, más un importe no confirmado de su llamada fallida. Tanda total: **74HTTP y USD0,004313614 conocidos más una operación de coste desconocido**. Cero resultados inciertos; tener respuestaHTTP400conocida no confirma su importe.34operaciones sin ejecutar no se cuentan como llamadas o cargos desconocidos. Excluye Codex de programación/revisión. Son tiemposHTTPdel arnés, no recall integrado ni respuesta final; no se suman percentiles para inventar latencia de otra arquitectura.

Acumulado del objetivo:5148operaciones/23campañas completas, USD0,541268950conocidos y cuatro importes históricos/no confirmados. Los contrastes de18preguntas no añaden campañas completas al contador. Las réplicas de controles se contabilizan todas, aunque no se usen en una recomposición posterior.

## Análisis posterior sin llamadas

Se revalidaron los recibos y posteriors de los modelos completos y un control Jev completo elegido por orden fijo, no por su resultado. Se estudiaron vetos por probabilidad de la categoría elegida y margen sobre la alternativa mayor, tanto globalmente como porrol. Ninguna regla examinada separa todos los errores sin pérdida con el margen requerido0,05. En Microsoft/Bhay una diferencia de0,0076en probabilidad, insuficiente para ese margen; las otras categorías también se solapan. Una respuesta incorrecta puede tener más confianza que una útil. No se elige un umbral por pregunta ni se presenta este ajuste sobre datos conocidos como validación nueva. [Diagnóstico escalar](../../outputs/iromem/evaluation/scoped-role-family-contrast-2026-10-20/calibration-posthoc.json).

Una composición genérica posterior aporta otra hipótesis: conservar aportes A de Mercury; sus propuestas B solo sobreviven si Jev y Microsoft también aceptan utilidad. La selección lee roles/IDs, sin consultar gold; después se compara con el gold original. Sobre estos resultados ya recibidos conserva18juicios permitidos seleccionados, sin ID ajeno ni pérdida de IDs útiles históricos. Esto NO prueba el mecanismo selectivo: todos los jueces originales vieron los siete candidatos, y sus respuestas pueden cambiar al ver únicamente las propuestas. Tampoco prueba recuperar todo el gold: las omisiones existentes siguen registradas.

Las tres corrientes completas utilizadas en esa recomposición ya costaron USD0,002867008, incluidos una sola vez en la contabilidad global. El análisis añade0HTTP/USD0nuevos. No se confunde ese subtotal de corrientes completas con el coste de un router selectivo que todavía no existe ni se infiere su latencia. Los fallos de calidad/recursos originales permanecen y no se recalifican. [Hipótesis de cascada, retrospectiva](../../outputs/iromem/evaluation/scoped-role-family-contrast-2026-10-20/cascade-posthoc.json).

## Siguiente puerta

Preparado un piloto prospectivo de críticos selectivos, todavía sin implementación o llamadas. Mercury propone, Jev revisa solo B y Microsoft solo supervivientes; no ven juicios/probabilidades anteriores ni gold. Se medirá contra Decisions/base con la puerta global original≤1,5×de coste/p50/p95, calidad mejorada y cero regresiones; no cambia el límite ni el resultado del contraste directo entre familias. Antes de adoptar requiere CLI real, campañas completasV5/V4/V2, alias, estabilidad y V6 nueva revisada a ciegas. [Protocolo del piloto siguiente](sparse-role-critic-pilot-v1-protocol-2026-10-20.md).

205pruebas workspace pasan, formato/build offline correctos. Pruebas de parser/composición/métricas no prueban semántica. Fuentes y payloads congelados, categorías reales, revisiones servidas esperadas y recibos raw se conservan; el informev2distingue errores/no ejecución de resultados semánticos. No hay proveedor o política runtime nuevos, cambiosSQL, base de campaña o servidor persistente iniciado. Default, README, gold, reservas y dashboard permanecen; V6sin llamadas. [Protocolo ejecutado](scoped-role-family-contrast-protocol-2026-10-20.md), [ejecución](scoped-role-family-contrast-execution-v1.md).
