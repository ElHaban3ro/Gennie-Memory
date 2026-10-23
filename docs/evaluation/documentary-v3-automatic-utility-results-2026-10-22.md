# Utilidad automática: primer contraste completo

La extracción revisada del banco conocido alcanza 116/116 entradas completas y fieles según ambos críticos, después de correcciones de desarrollo. Eso no equivale a rendimiento de recuperación. La primera ejecución automática de los 100 pares recupera **16/39 útiles**, frente a **36/39** con las representaciones manuales. Ambos brazos seleccionan **0/60 ajenos**. La puerta de utilidad falla y el candidato no se adopta.

| Medida | Extracción automática revisada | Control con representación manual |
| --- | ---: | ---: |
| Recuerdos útiles conservados | 16/39 (41,0%) | 36/39 (92,3%) |
| Selecciones ajenas | 0/60 | 0/60 |
| Concordancia con referencia histórica | 76/100 | 97/100 |
| Respuestas completas emitidas | 0 | 3 |
| Tiempo CLI p50 | 12.765 ms | 12.387 ms |
| Tiempo CLI p95 | 21.970 ms | 14.418 ms |

Hay una referencia ambigua informada por separado; la concordancia de 100 no debe confundirse con exactitud semántica no ambigua. Los tiempos miden procesos CLI sobre representaciones ya disponibles: excluyen extracción, revisión y red y no acreditan recursos o latencia de producción. El control manual no es la configuración clasificadora vigente.

La nueva extracción V7 inicial obtuvo 108/116 aceptadas, frente a 112/116 en la tanda anterior: conservó 104, recuperó cuatro y perdió ocho. Dos rondas acotadas de corrección semántica alcanzaron 115; una normalización de dimensión conservó esa cifra. El soporte optativo de términos descritos y una corrección estructural acotada de la última entrega permitieron 116/116. Se conservaron todos los intentos y dictámenes originales. Las 115 entradas idénticas retienen sus revisiones con comprobación exacta de texto/raw/validación/contrato; la entrada modificada recibe dos críticos nuevos. Es refinamiento conocido con feedback, no una nueva extracción independiente perfecta.

El contraste utiliza solo las representaciones automáticas como entrada. La unión se hace por tipo y SHA256 del texto. La única unión de vocabularios permite el perfil optativo de términos descritos; el otro lado se revalida y debe producir un informe idéntico salvo el SHA de vocabulario. Se verifica la procedencia de los 232 dictámenes. La selección en sobres requiere evidencia en todos los cruces de lecturas; la utilidad condicional queda separada.

Diagnóstico en curso: representaciones individualmente fieles usan formas distintas sin una normalización interoperable; también hay bloqueos de identidad contextual y calificadores. No se reescriben raws ni referencias para hacerlos pasar. La siguiente corrección debe resolver una causa general y conservar los 60 negativos y los informes históricos.

Pasan 688 pruebas del workspace (24 omitidas), formato, compilación y 226 informes históricos idénticos. Cero llamadas nuevas a proveedores externos y USD 0 en ese ledger; coste Codex desconocido. No se abrió la cohorte reservada V6. El objetivo continúa en el paso 3; baseline, recursos, estabilidad y validación nueva siguen pendientes.

Evidencia reproducible: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/automatic-utility-v2/report.json`, manifiesto de entradas, 200 ejecuciones y reportes CLI. Fidelidad: `dimension-typed-profile-v1/merged-report.json` y procedencias completas en el mismo árbol.
