# Corrección de cuatro representaciones conocidas — 2026-10-22

Se corrigieron y revisaron cuatro errores de autor de la tanda anterior. Las cuatro entradas pasan la validación estructural y reciben los cuatro juicios faithful de dos críticos nuevos: 32/32 dimensiones fieles. El conjunto conocido corregido tiene **106/116 completas, 104/116 aceptadas por ambos críticos y 103/116 que cumplen ambas condiciones**. Quedan 13 entradas; la puerta completa sigue fallando.

No es una nueva extracción autónoma: las cuatro correcciones son retrospectivas y las otras 112 entradas conservan exactamente sus representaciones y dictámenes previos. Se revalidaron las 116 con el mismo binario; los 112 informes sin cambios coinciden íntegros con sus informes anteriores. No cambian la puntuación original de 99/116 ni las mediciones de utilidad previas de 31/39 y cero ajenos/60.

| Medición en los mismos 116 textos | Original congelado | Conjunto corregido conocido |
| --- | ---: | ---: |
| Completas | 102 | 106 |
| Aceptadas por ambos críticos | 100 | 104 |
| Completas y aceptadas por ambos | 99 | 103 |
| Parciales | 10 | 10 |
| Inválidas | 4 | 0 |
| Puerta completa | No | No |

## Cambios y revisión

Se congeló el protocolo de corrección antes de modificar los raws. Una única corrección por entrada, sin resultados de selección ni etiquetas:

- Salvia: `lights.binding_scope_id=null`, conservando su scope, asociaciones y el binder que cuenta seis parejas. Un referente dentro de un binder no se convierte por ello en su variable.
- Risco: completar el rol mode del target de restricción en ambas lecturas con su propio miembro descrito ya presente. Conservadas resolución local, modalidad y negación independiente.
- Olmo: completar supplier en el target con el miembro K existente. Sin convertir suministro en procedencia aguas arriba.
- Malva: completar material en el target con el miembro descrito existente. Conservadas procedencia local, exclusividad y negación independiente.

La aplicación inversa de cada cambio recupera exactamente el JSON original. No se alteraron textos, vocabulario, namespace, ámbitos ni campos ajenos a los cambios enumerados. Los originales y dictámenes anteriores permanecen intactos.

Cuatro contextos nuevos revisaron solo sus paquetes fuente o pregunta, sin originales anteriores, explicación de la corrección, críticas previas, contrapartes, etiquetas ni resultados. Se verificaron hashes, IDs, orden, claves duplicadas, cuatro dimensiones, paths y citas. Son críticos de la misma familia Codex, no validación humana o independencia de familia.

## Límites, coste y siguiente trabajo

No cambió Rust ni el binario. La última suite verificada del mismo código conserva 730 pruebas aprobadas, cero fallidas y 24 ignoradas; no se repitió Cargo. Verificación actual: cuatro validaciones de correcciones, revalidación íntegra de 116 y cuatro revisiones independientes. Esto no mide utilidad ni latencia de producción.

**Cero llamadas externas y USD 0 externos nuevos**; coste/tokens Codex de los cuatro contextos desconocidos. No hubo servicios nuevos, cambios de base de datos, credenciales, dependencias o producto. Sin adopción ni apertura de V6.

El siguiente trabajo aborda los 13 problemas restantes, incluidos los portadores ausentes de finalidad normativa y magnitud condicionada por ruta. Las ambigüedades y ausencias reales no se resolverán inventando contexto. Cualquier distinción prospectiva entre incertidumbre fiel y pérdida semántica requiere contrato y crítica propios; no se recalculará la tanda fallida ni se relajarán sus puertas para aprobarla. La utilidad solo se ejecutará tras superar las puertas locales correspondientes.

## Evidencia

`outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-author-corrections-v1/` contiene protocol.json, validation-summary.json, las cuatro representaciones, sus validaciones, los cuatro paquetes críticos y dictámenes, report.json y known116-bundle/{protocol,validation-report}.json. Los scripts de derivación, revalidación y auditoría quedan guardados junto a los artefactos.
