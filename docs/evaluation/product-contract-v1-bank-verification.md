# Preparación y adjudicación sintética del banco de producto v1

Este registro describe la preparación anterior a inferencia. La [campaña posterior y cierre](product-contract-v1-results-2026-10-23.md) completó las tres etapas; sus resultados no se sustituyen por el preflight local descrito aquí.

Fecha: 2026-10-23 (America/Bogota). Continuación local de la consolidación del contrato, sin campaña remota.

## Resultado

Se preparó un [banco nuevo](../../tests/fixtures/product-contract-v1/README.md) de 18 consultas y 36 recuerdos. Cada partición tiene seis consultas, tres evidencias útiles y tres controles de comportamiento. La política congelada es el perfil optativo Decisions/per_memory_v1; el predeterminado baseline no cambia.

El validador liga opcionalmente corpus/política al manifiesto y rechaza evidencia inexistente, de otro ámbito/entidad, obsoleta, inactiva, fuera de vigencia, duplicada o incompatible con el resultado esperado. Los grupos deben abarcar también distractores. Sigue sin duplicar toda la validación runtime ni probar relevancia semántica.

Dos subagentes autorizados revisaron solo el mismo paquete sin etiquetas ni particiones y su rúbrica, con salidas distintas. Ambos coincidieron con las 18 etiquetas, pero señalaron ambigüedades. Se conservó la primera versión íntegra en `history/draft-1`; se precisó la solicitud HIN-24 y la precedencia de relaciones provisionales/condicionadas. Ambos revisaron nuevamente y aceptaron la versión corregida sin bloqueos de anotación. Permanecen observaciones sobre límites de las afirmaciones y resultados de búsqueda no ejecutados.

`audit-product-reviews` comprueba identidad de paquete/rúbrica por hash, correspondencia exacta con solicitudes y corpus, dos identidades distintas, cobertura íntegra, IDs/revisiones y coherencia de selección. Su salida conserva observaciones y desacuerdos; acuerdo de etiquetas no es una promoción automática. [Auditoría vigente](../../tests/fixtures/product-contract-v1/review-audit.json).

## Verificación

- Seis pruebas focales de product_suite pasan: separación/hashes, ligadura de banco, política/resultados, evidencia gold, grupos de distractores y revisión con identidades separadas. El test de revisión verifica también rechazo de una rúbrica cambiada tras revisar.
- `cargo test --offline -p gennie-memory-eval`: **707 pasan, cero fallos y cero ignoradas**. Se mantuvieron dos advertencias preexistentes por código no usado. Tras añadir la última aserción de rúbrica obsoleta se repitió la prueba focal de revisión.
- Compilación offline del evaluador, `cargo fmt --all --check` y `git diff --check`.
- CLI real con base temporal y credencial vacía: carga de 36 recuerdos y seis solicitudes de desarrollo válidas. Las tres evidencias anotadas están entre los candidatos. La aclaración no clasifica; el control de búsqueda vacía no llama al modelo; cuatro casos con candidatos terminan con `credential_missing`, sin semilla fiable. [Preflight vigente](../../tests/fixtures/product-contract-v1/development-preflight.json).
- Se eliminó la base temporal propia. No se abrieron puertos ni se detuvieron servicios. Cero llamadas API, USD 0 API; coste de revisión Codex desconocido.

Las pruebas ampliadas afectan únicamente a herramientas/fixtures de evaluación. No hubo cambios runtime, SQL, dependencias, transporte ni UI en esta continuación; la verificación de workspace/HTTP2 del cambio anterior permanece separada en su [registro](product-contract-v1-verification.md).

## Pendientes antes de aprobar el producto

No hay selección real medida de este banco, ni calibración, ni resultado final. Las dos particiones reservadas tienen anotaciones revisadas, pero no consultas runtime ejecutadas. Las hipótesis `no_candidates` de esas particiones siguen por comprobar. La revisión es sintética entre agentes, no humana ni prueba de independencia de familia de modelos.

El protocolo propone como máximo 12 llamadas, 300 segundos y USD 0,10 para las tres particiones; estos límites no autorizan gasto por sí mismos ni sustituyen el control de un runner. Antes de iniciar desarrollo remoto: comprobar disponibilidad/precio del modelo y preparar ejecución con registro durable, control de presupuesto y sin reintentos inciertos. Avanzar a calibración/final solo después de superar la etapa precedente. Un cambio de política posterior a observar resultados reservados exige renovar la reserva afectada. No se modifican las métricas ni gráficas históricas del README con esta evidencia local.
