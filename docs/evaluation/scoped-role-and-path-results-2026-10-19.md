# Roles y referencias de campos: ninguna mejora aprobada

Goal activo, predeterminado conservado y V6 sin llamadas. Se completaron tres diagnósticos sobre las mismas18preguntas conocidas de V4/V5 y sus siete candidatos.108HTTP:18controles ScopedAnchorV1 y18candidatas por diagnóstico, orden alternado y un intento por operación. No son108casos nuevos. Se conservan todos los resultados anteriores y el gold original.

| Diagnóstico | Elección de rol | Criterios revisados | Rutas documentadas |
| --- | ---: | ---: | ---: |
| Regresiones iniciales sin resolver, de5 | 1 | 2 | 4 |
| IDs no permitidos | 3 | 4 | 1 |
| IDs útiles perdidos frente al control contemporáneo | 0 | 0 | 2 |
| IDs útiles históricos seleccionados antes perdidos | 0 | 0 | 2 |
| IDs útiles ganados frente al control contemporáneo | 5 | 6 | 0 |
| Coste candidata/control | 0,935× | 0,989× | 1,007× |
| Pasa calidad | No | No | No |
| Pasa recursos | Sí | Sí | Sí |

Las dos variantes de rol conservan los IDs útiles seleccionados antes, pero sus extras bloquean adopción. Eso no afirma recuperar todo el gold: los informes también guardan omisiones dentro del prefijo, incluso cuando no eran pérdida de una selección anterior. La puerta de recursos sigue coste≤1,03×control y p50/p95≤1,5×; no se relajó después de observar respuestas.

Una elección por recuerdo guarda A como aporte solicitado, B como refutación o none; el testigo literal conserva su papel de comprobación de nombre, no de certificación semántica. La categoría real se registra en `role_checks`, sin presentar dos juicios F/C independientes. Una normalización interna del auditor sirve solo para comprobar la composición.

Errores de rol: se acepta un borrado puntual como política de retención de M4; exigir carné como exclusión de avalista; y la descripción de un depósito como exclusión de agua de red. El nombre del lugar/caso aparece en todos, así que el testigo literal no los excluye. Precisar A/B conserva los IDs útiles históricos pero añade además «no se exige registrar domicilio» como negación del domicilio solicitado. La revisión de criterios falla pese al ahorro; no se amplía.

La [guía primaria de TypeSafe](https://docs.typesafe.ai/concepts/how-to-build-with-system-one) recomienda referencias a campos con rutas entre backticks desde los nombres dentro de state; el [ejemplo oficial de OpenRouter](https://openrouter.ai/blog/tutorials/how-to-use-jev/) sigue esa notación. El código anterior escribía state.user_prompt y state.memories.memory_NN como referencias textuales sin backticks. Se ensayó SOLO esa sustitución sobre ScopedAnchorV1, conservando sus dos preguntas F/C y el resto del payload. Cada ruta resuelve y su reversión recupera todo el payload original. Es una diferencia de sintaxis comprobada, no prueba de un error de protocolo o de la causa de los errores semánticos.

La medición de rutas no supera calidad: conserva cuatro regresiones iniciales, añade el archivo puntual deCendal y pierde los IDs útiles06024/06106 en las preguntas06003/06101. No se cambia la sintaxis del producto ni se invalidan retrospectivamente las campañas anteriores.

| HTTP del arnés y consumo | Rol | Criterios | Rutas |
| --- | ---: | ---: | ---: |
| Coste nuevo USD, control+candidata | 0,002831808 | 0,002911188 | 0,002937648 |
| Mediana control/candidata, ms | 301,14 / 298,95 | 304,95 / 297,21 | 318,67 / 305,96 |
| p95 control/candidata, ms | 474,45 / 406,33 | 573,18 / 426,29 | 427,29 / 349,04 |

Tanda: **USD0,008680644**,108HTTP, cero errores técnicos o costes nuevos desconocidos. Coste excluye Codex de programación/revisión. Son tiempos HTTP sobre candidatos congelados, no recall integrado, respuesta final ni garantía poblacional. Acumulado:5074operaciones/23campañas completas, USD0,536955336conocidos y tres importes históricos desconocidos. Estos diagnósticos no añaden campañas completas.

202pruebas workspace pasan; formato y build offline correctos. Los18controles coinciden entre los tres diagnósticos; en las18candidatas de criterios solo cambian A/B y en las18de rutas solo las referencias. Se verificó igualdad de JSON preparado y constructores congelados; no se capturó tráfico de red. Fuentes ejecutadas y hashes de binarios se conservan, sin afirmar archivo o reproducción bit a bit de todos los binarios. El auditor recompone desde raw sin confiar en el éxito del runner. No hay política runtime nueva, dependencias o cambios SQL, servidor persistente o base de campaña nueva.

Siguiente contraste preparado, aún sin llamadas: otras familias del [catálogo oficial de decisiones](https://openrouter.ai/api/v1/models?output_modalities=decisions) con RoleV5 idéntico, cambiando únicamente model. La presencia en catálogo y el precio publicado no prueban calidad, revisión de pesos inmutable, disponibilidad o factura. [Protocolo del contraste](scoped-role-family-contrast-protocol-2026-10-20.md). Mantener las puertas antes de implementación CLI, campañas completasV5/V4/V2, alias, estabilidad y V6 ciega. Ninguna variante de esta tanda se incorpora.

[Protocolo de rol](scoped-anchor-role-v5-protocol-2026-10-19.md), [criterios](scoped-anchor-role-criteria-v2-protocol-2026-10-19.md), [rutas](scoped-anchor-path-v1-protocol-2026-10-19.md). [Informe de rol](../../outputs/iromem/evaluation/scoped-anchor-role-v5-2026-10-19/report.json), [criterios](../../outputs/iromem/evaluation/scoped-anchor-role-criteria-v2-2026-10-19/report.json), [rutas](../../outputs/iromem/evaluation/scoped-anchor-path-v1-2026-10-19/report.json).
