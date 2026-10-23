# Consenso con orden inverso: pase v4, fallo v2

Fecha: 2026-10-19. Desarrollo conocido. Política `roles_consensus_mirrored_v1`, Jev explícito, dos preguntas en una petición; segundo orden inverso de los mismos top16, normalización de letras a ID antes del acuerdo. No promovida. [Protocolo previo](roles-consensus-mirrored-v1-protocol-2026-10-19.md).

| Medida | v4 base → candidata | v2 base → candidata |
| --- | --- | --- |
| Apoyo útil |16→16/16|16→16/16|
| Refutación útil |13→16/17|16→15/16|
| Contexto parcial útil |14→16/16|13→16/16|
| Ausencias correctas |13→15/15|16→16/16|
| ID no permitidos |2→0|0→0|
| Pérdidas por caso |0|1|
| Ganancias útiles |5|3|
| Coste USD |0,004501500→0,005441310|0,003987700→0,005069484|
| p50 ms |382,230→357,892|368,747→369,107|
| p95 ms |820,494→467,997|601,405→532,508|
| Calidad |Pasa|Falla|
| Recursos |Pasa|Pasa|

256 operaciones/peticiones completas, recibos globales una vez. Coste conjunto USD0,018999994, sin Codex. Ambas cohortes fueron conocidas al ajustar esta variante. La comparación de momentos distintos con batch previo no prueba causalidad del orden inverso ni estabilidad de la latencia.

V2 bloquea por `cev2q_049_03`: pregunta por la entrada actual al farallón; el recuerdo dice que los senderos costeros del tramo sur cerraron el acceso por erosión. Primer voto elige esa refutación; segundo se abstiene. El acuerdo estricto produce semilla vacía y pierde evidencia que baseline conserva. Tres ganancias parciales no compensan esta pérdida.

V4 tuvo dos desacuerdos: una pregunta de contraseña recibe [none, recuerdo de ubicación del álbum], correctamente vacía; el caso de moho recibe dos IDs distintos y queda vacío, conservando la abstención de base. Una categoría casi perfecta no equivale a verdad general o revisión humana. Agentes de misma familia revisaron el banco por separado; no se generaron respuestas finales de Codex.

[Informe v4](../../outputs/iromem/evaluation/roles-consensus-mirrored-v1-development-2026-10-19/cohort-v4/report.json), [informe v2](../../outputs/iromem/evaluation/roles-consensus-mirrored-v1-development-2026-10-19/cohort-v2/report.json). Se detuvo antes de v5; sus etiquetas no se usaron para ajustar. Se conserva esta variante y todos sus fallos. La hipótesis posterior de agregación asimétrica se declara en otro protocolo y exige medición nueva completa, sin convertir una proyección retrospectiva en validación.
