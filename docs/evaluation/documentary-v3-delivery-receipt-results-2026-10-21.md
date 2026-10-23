# Recepción de entregas documentales V3 sin reparación

Se añadió `audit-documentary-v3-delivery <packet> <expected-frozen-sha256> <delivery-path> <new-report>` al evaluador Rust. El hash esperado debe venir del protocolo congelado, no recalcularse para aceptar un paquete cambiado. El comando no llama a modelos, clasifica recuerdos ni modifica producto.

## Contrato

Primero verifica bytes del paquete contra el hash esperado y comprueba versión, tipo, vocabulario canónico, IDs únicos, textos y sus hashes. El sobre de entrega exige versión exacta, hash del paquete, filas completas en el mismo orden y campos cerrados. Un fallo del sobre degrada todo el lote; no se rescatan filas convenientes. Una entrega inexistente o JSON ilegible conserva todas las filas esperadas como `unavailable`, nunca una muestra vacía aprobada. Errores de acceso o límites de16MiB abortan el comando y no producen un informe exitoso; el orquestador debe conservar ese fallo, no omitir el lote.

El contenido recibido se conserva íntegro: texto UTF8 o bytes para contenido noUTF8, más SHA256. Un sobre bien vinculado conserva cada raw original y su validación Rust sin repararlo. Un raw inválido no elimina otra fila independiente válida. Tipo documental o namespace incompatibles con el paquete invalidan esa fila para la campaña, aunque su DTO aislado pudiera ser válido. El informe nunca convierte integridad o completitud estructural en fidelidad, utilidad o puerta de calidad superada.

La salida usa creación exclusiva y no sobrescribe un informe existente. La ruta de una entrega ausente solo registra ausencia: no inicia ni espera un proceso. El uso de una entrega nueva corregida requeriría una nueva versión experimental; este comando no valida la política de una sola entrega por sí solo ni demuestra aislamiento real de contextos. Los hashes no prueban que el modelo respetó sus instrucciones.

## Verificación ejecutada

| Evidencia | Resultado |
| --- | --- |
| Pruebas de workspace | 619pasan,24omitidas |
| Pruebas del adaptador V3 incluidas | 8:6nuevas y2previas |
| Pruebas del motor V3 incluidas | 91 |
| Diez paquetes reales con entrega deliberadamente ausente | 116/116filas conservadas como pendientes |
| Control CLI con entrega manual de operador | Completo, sin afirmar fidelidad |
| Repetir salida existente | Rechazado, informe intacto |
| Hash esperado incorrecto | Rechazado, sin salida |
|100pares manuales históricos|Salidas idénticas|
|116validaciones congeladas|Salidas idénticas|
|Compilación y formato|Pasan|

Las pruebas nuevas incluyen JSON ilegible, contenido binario, hash/sobre/versiones alterados, omisión, duplicado, orden cambiado, campos de etiqueta inyectados, raw inválido y tipo/namespace incorrectos. El control positivo utiliza un frame manual del banco de operadores; no es una salida de extractor. Se conserva un error inicial de compilación por formato hexadecimal de la dependencia SHA256y su corrección antes de las verificaciones.

Los paquetes de la siguiente extracción siguen intactos y no despachados. No se generaron extracciones, críticas ni una nueva métrica de recuperación. Permanecen los resultados anteriores:112/116extracciones estructuralmente completas,65/116aceptadas por ambos críticos;36/39referencias útiles manuales seleccionadas, tres respuestas completas y cero selecciones en60referencias heredadas none. La ambigua sigue separada. No se consumióV6ni se promovió el mecanismo.

## Coste y siguiente acción

API nuevaUSD0, cero peticiones y cero subagentes nuevos. Coste/tokens Codex desconocidos. Acumulado sin cambios:6129operaciones lógicas,23campañas emparejadas completas,USD4,565416508conocidos y20operaciones con coste desconocido. No hay nueva medición de latencia de extracción o producción.

El objetivo permanece activo en el paso3. La extracción completa preparada requiere aclarar la autorización para nuevos contextos de subagentes tras la orden de cerrar los anteriores. No se interpreta el silencio como autorización ni se registra una conversación pendiente como trabajo en ejecución. Después corresponderá recibir las entregas con este comprobador, preservar fallos y realizar las dos críticas separadas antes de utilidad automática.

Artefactos en `outputs/iromem/evaluation/documentary-v3-delivery-receipt-2026-10-21/`.
