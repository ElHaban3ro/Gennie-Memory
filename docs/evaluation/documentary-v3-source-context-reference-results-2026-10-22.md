# Referencia al contexto fuente: implementación y límite semántico

Se implementó y verificó el transporte local que conserva una ocurrencia fuente sin reinterpretar su posesivo desde la pregunta. El banco conocido corregido sigue en **115/116 completas y fieles**: la credencial no recibe aceptación integral. El objetivo mantiene sus cinco pasos, con el paso3 activo. No hay permisos de usuario pendientes.

## Cambio comprobado

La inspección de la ruta anterior mostró que el testigo copiaba Claim, documentos y enlaces, pero el JSON aislado no imponía identidad de ocurrencia ni disciplina al lector siguiente. `source-context-reference-v1 bind|consume INPUT MANIFEST OUTPUT` añade esa frontera solo al evaluador Rust. No cambia DTO, vocabulario, matcher, validador, política de producto o dependencias.

El manifiesto externo liga un origen a la entrada fuente y una entrega lógica a su input completo. Dos ocurrencias con texto idéntico mantienen IDs distintos; repetir la misma entrega conserva su ID sin probar frescura. Bind obtiene un testigo posesivo parcial real tras validar fuente y pregunta completas, contrasta documento, Claim y arrays redundantes, y conserva las deudas y respuestas/bindings vacíos. Consume reclasifica la entrega registrada y exige igualdad de toda la referencia.

El vínculo refiere al contexto original como dependencia opaca. No reconstruye c0, no elige κ, no identifica al beneficiario con el posesivo ni afirma que falte un antecedente determinado en origen. Un lector que descarte el envoltorio queda fuera de esta garantía. La asignación de ocurrencias por el runner es una precondición explícita; el mecanismo no acredita verdad, autoría, autenticidad o ejecución histórica.

JSON estricto, lecturas acumuladas de16MiB, preinspección conjunta y un único presupuesto cubren hashes, reservas, reclasificación y comparación. Los rechazos no publican un archivo exitoso; una salida existente permanece intacta. [Contrato](documentary-v3-source-context-reference-contract-2026-10-22.md), [crítica previa](documentary-v3-source-context-reference-contract-critique-2026-10-22.md) y [revisión estática](documentary-v3-source-context-reference-code-review-2026-10-22.md).

## Verificación actual

| Comprobación | Resultado |
| --- | ---: |
| Pruebas focales del adaptador | 7/7 |
| Workspace offline | 849 aprobadas, 0 fallos, 24 ignoradas |
| Controles CLI propios del coordinador | 32/32:15 bind y17 consume |
| Replays históricos | 425/425 idénticos byte a byte |
| Casos iniciales de la ruta sin wrapper | 2/2 con DTO completos y contexto parcial íntegro |
| Banco mixto vigente | 115/116 completas y fieles, sin cambios |

Los controles incluyen misma ocurrencia con propietarios solicitados distintos, dos orígenes de texto idéntico, origen intercambiado, fuente/Claim/U/arrays alterados, respuestas y bindings inventados, deuda retirada, manifiesto cambiado, envoltorio desprendido, duplicados JSON, lectura conjunta excedida y salida existente. Los negativos estructurales o de procedencia no se llaman negativos semánticos correctos.

Los425replays incluyen los403anteriores, las dos nuevas fuentes de composición con «hasta»,18controles anteriores y los dos casos iniciales de transporte. Revalidan los116 del banco actual. La candidata de credencial reutiliza su validación completa de la reproducción370, ligada a los mismos bytes. Los hashes del motor y validador anteriores permanecen idénticos: solo cambian las declaraciones de módulo/CLI y se añaden el adaptador y sus pruebas.

La primera compilación falló en tres usos de `LowerHex` sobre el tipo digest de la dependencia instalada. Se sustituyeron por conversión hexadecimal explícita con precobro, sin dependencia nueva ni relajación de pruebas; el fallo se conserva. El contador real es siete pruebas focales, corrigiendo el recuento anticipado de ocho del implementador.

Binario final: `897b6e228c4d4d3d5f970b7e687745d800df0d8748b9a5a481b0659c32ded7e4`. Evidencia: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/source-context-reference-v1/verification.json`.

## Crítica fuente y causa pendiente

Se preparó una representación aumentada de la credencial: mismo rawV2, texto y vocabulario, con vínculo explícito a la ocurrencia original y semántica de consumo por referencia fuente. El cambio de alcance se declara; no se vuelve a votar sobre una candidata idéntica ni se reescriben dictámenes anteriores. Dos contextos nuevos recibieron únicamente fuente, raw, vocabulario y vínculo/semántica de origen; sin preguntas, resultados, código, gold o revisión ajena.

Ambos consideran fiel la preservación operativa del posesivo. B acepta las cuatro dimensiones; A mantiene incertidumbre en fidelidad textual, completitud y atribución: «obligación de tutor para Ina» puede describir una obligación correspondiente al tutor, y no queda justificada su conversión en requisito de clase para Ina. Resultado:5juicios faithful y3uncertain; **no aceptación integral, ningún acierto nuevo**. Se verificaron27citas por offsets y54punteros. Es revisión de la misma familia Codex, sin validación humana o independencia de familia.

Paquete SHA-256 `b6a6e9c43268445bf591f696e6ae4fc2ac676815f65d464ae99ad055273e0a99`; originales y auditoría en `credential-source-origin-correction-v3/`. Guardar fuente y origen no vuelve fiel una normalización semántica más fuerte. El raw anterior parcial del banco permanece vigente, aunque la candidata aumentada sea estructuralmente completa.

El [diseño fuente posterior](documentary-v3-obligation-nominal-source-design-2026-10-22.md) identifica una vía: conservar el nominal obligación→tutor con `nominal_complement_de` y el evento de eliminación, sin deducir ausencia actual de un requisito. Todavía debe especificar y demostrar la adscripción/correlación de «para Ina» y «En Faro». Dos contextos exteriores no prueban por sí solos una restricción nominal interna; no se implementa esa simplificación sin resolver su semántica. Es trabajo técnico pendiente, sin una nueva aprobación humana requerida.

## Coste y alcance de las mediciones

Esta tanda ejecutó459operaciones CLI, cuatro procesos Cargo —uno fallido— y dos de formato, cinco contextos Codex nuevos y cuatro reanudaciones. **Cero llamadas a proveedores; USD0 API nuevo.** Codex sigue sin importe cuantificado. El histórico externo no cambia:6.129operaciones, USD4,565416508 conocidos y20importes desconocidos; excluye Codex.

Los tres bind positivos tardaron30,99–346,86ms y los tres consume positivos30,56–31,91ms, incluyendo proceso y archivos locales; el primer bind es la observación más lenta. Las referencias ocupan88.527–88.541bytes locales. Son controles de desarrollo con n=3 por operación, sin comparación emparejada ni medición de RAM o integración. No son tokens remotos, latencia de producto o mejora de rendimiento. La extracción autónoma original99/116 y utilidad histórica31/39 no cambian; no se actualizan gráficas de rendimiento con estas cifras.

El siguiente trabajo es corregir la representación nominal pendiente y validarla; después siguen la extracción coherente de los116 con guía/vocabulario compartidos y doble crítica, utilidad/base, recursos, estabilidad y casos nuevos ciegos. V6 continúa sin abrir y no hay adopción de producto.
