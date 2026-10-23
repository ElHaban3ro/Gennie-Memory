# Retención consolidada: contraste completo — 2026-10-22

Informe histórico del snapshot 107. La extracción posterior de 116 textos y su puerta fallida están en el [estado actual](documentary-v3-consolidated-extraction-results-2026-10-22.md). La preparación descrita aquí no representa el despacho vigente.

La configuración evaluada alcanza **31/39 evidencias útiles (79,5 %)** frente a **28/39 (71,8 %)**, con **cero selecciones ajenas de 60** y ninguna pérdida. Se ejecutaron los 100 pares en tres brazos con el mismo binario: no se sumaron aritméticamente los aciertos del piloto.

| Medición | Base | Solo vocabulario | Representaciones consolidadas |
| --- | --- | --- | --- |
| Evidencias útiles | 28/39 | 28/39 | 31/39 |
| Selecciones ajenas | 0/60 | 0/60 | 0/60 |
| Respuestas completas | 0 | 0 | 0 |
| Selección solo condicional | 1 | 1 | 1 |
| Pérdidas anteriores | — | 0 | 0 |

Quedan ocho omisiones y un ambiguo separado. La puerta completa 39/39 sigue fallando. Es una integración de representaciones previamente revisadas sobre desarrollo conocido, **no una nueva extracción autónoma de las 116 entradas**, generalización o adopción de producto.

## Cambios y comparación

Se preservaron 96 pares de frames anteriores. En cuatro pares se incorporaron exactamente dos fuentes y dos preguntas del piloto de retención, con su namespace y ámbito por par intactos. Texto, orden, etiquetas y referencias no cambiaron. El único cambio común de vocabulario fue registrar `raw_file`; se conservaron el enlace y perfil de admisión de Duna, incluido el vocabulario especial de una fila. No se importaron los perfiles de material/configuración ajenos del vocabulario piloto.

La revisión independiente comprobó los 100 hashes, el delta por fila y la conservación de los cuatro documentos salvo su nuevo hash de vocabulario. El preflight dejó tres contextos parciales y un rechazo, sin respuestas completas. Después se ejecutó el banco completo, alternando los tres brazos por caso. Los 100 resultados de la base se reprodujeron idénticos a su informe anterior.

Las ganancias son `pcv1_01_01/claim_00`, `pcv1_01_01/claim_01` y `pcv1_01_02/claim_00`. Conservan pasado, operación terminal, estado crudo y dominio universal donde corresponde. La política anónima y la membresía no se convierten en identidad acreditada: permanecen las deudas y las proposiciones/ámbitos originales, con términos y proposiciones de respuesta y bindings vacíos. El caso individual frente a plazo común universal sigue rechazado. El ambiguo conserva su observación anterior sin entrar al denominador de útiles o ajenos.

## Preparación de la siguiente extracción

Hay diez paquetes separados con 83 fuentes y 33 preguntas, 116 IDs opacos, guía/vocabulario completos y roster privado del coordinador. V2 conserva todos los textos e IDs. Se corrigieron dos conflictos antes de autores: AR01 exigía uso efectivo globalmente y podía eliminar declaraciones válidas de capacidad; ahora solo restringe la premisa assignment del perfil correspondiente. Además se explicitó la precedencia del perfil de `described_ref` sobre el shorthand de la notación base en los roles admitidos.

La crítica focal verificó V1 intacta, las dos modificaciones de guía y sus diez copias de paquete. El preflight del código actual con el vocabulario completo obtuvo **100 pares válidos y 100 resultados exactamente iguales** a la consolidación vigente. Se conservó y reparó un error del harness al leer `selected_claim_ids` en un agregado de lecturas: se reutilizaron sus 29 informes existentes y solo se ejecutaron los 71 restantes. No cambió la regla de agregación, el clasificador o las etiquetas.

`root-dispatch-protocol.json` congela hashes, aislamiento fuente/pregunta, un intento semántico, reparación mecánica acotada y doble revisión en cuatro dimensiones (`text_fidelity`, `completeness`, `attribution`, `operative_preservation`). **No se han iniciado autores ni nuevas extracciones.** Preparación y compatibilidad no prueban fidelidad autónoma. El diagnóstico retrospectivo de omisiones está fuera de los paquetes y no se entrega a autores.

## Ocho omisiones actuales

Una auditoría desde los inputs e informes vigentes confirma seis problemas de representación/contrato: Faro, Ópalo, Cobre, Malva, Vado y Nube. Delta y Lino permiten investigar contexto documental y medición con resultado ausente. Son hipótesis verificadas estáticamente, sin ganancias nuevas atribuidas. No se resolverá Lino cambiando todas las lecturas por alguna; tampoco Cobre eliminando la ruta, Nube igualando frecuencia genérica y tasa de muestreo o Vado inventando una fecha.

## Verificación, tiempos y coste

En esta etapa **no cambió Rust ni se repitió Cargo**. El binario y las fuentes del evaluador coinciden por hash con la versión ya verificada mediante 729 pruebas, 24 ignoradas y 226 informes históricos idénticos. La evidencia ejecutada ahora comprende cuatro pares de preflight, 300 clasificaciones de la comparación y 100 del preflight del contrato completo: 404 llamadas CLI locales, sin proveedor.

| Tiempo CLI observado | Base | Solo vocabulario | Consolidado |
| --- | --- | --- | --- |
| p50 | 14,047 ms | 13,839 ms | 13,920 ms |
| p95 | 21,455 ms | 20,479 ms | 23,716 ms |

Son tiempos descriptivos de esta tanda local; no acreditan mejora de recursos ni latencia de producción y excluyen extracción, crítica, red y entrega a Codex. No se comparan con otra tanda bajo distinta carga.

**Cero llamadas externas nuevas; USD 0 externos nuevos.** Coste y tokens facturables Codex desconocidos. El acumulado externo permanece en 6.129 operaciones, 23 campañas, USD 4,565416508 conocidos y 20 operaciones de importe desconocido, sin Codex.

M5 y el objetivo siguen activos en paso 3. Full116 fiel, puerta local completa, comparación contemporánea con producto, recursos/estabilidad y casos nuevos con revisión ciega siguen pendientes. V6 continúa cerrada; default de producto, README, dashboard, datos y credenciales no cambian.

## Evidencia

Bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`:

- `retention-consolidation-v1/known100-comparison/{protocol,report}.json`: derivación y tres brazos completos.
- `retention-consolidation-v1/critical-review/frozen-inputs-review.json`: revisión independiente previa a la ejecución.
- `retention-consolidation-v1/root-verification/`: hashes y auditoría del payload.
- `consolidated-extraction-contract-v1/STATUS.md`: estado vigente de preparación.
- `consolidated-extraction-contract-v1/root-dispatch-protocol.json`: paquetes y puertas congeladas, sin dispatch.
- `consolidated-extraction-contract-v1/root-code-preflight-v2/`: compatibilidad actual y error de harness preservado.
- `remaining8-current-audit-v1/audit.json`: ocho omisiones y evidencia actual, reservado al coordinador.

Binario conservado SHA-256: `c622474a0f442643d6050ef632bbba02412bb097fddc9c6b2a20c975a4773034`.
