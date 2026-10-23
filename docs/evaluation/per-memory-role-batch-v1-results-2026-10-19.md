# Verificación por rol en lote V1: resultados

La variante **no cumple calidad ni coste**. Se conserva `baseline` y no se ejecuta V6. El objetivo sigue activo; agrupar HTTP no fue una optimización válida de este mecanismo.

## Implementación comprobada

`per_memory_role_batch_v1` conserva propuesta Jev V2, instrucciones completas de verificación de datos V1 y correcciones V4. Todos los positivos tienen un juicio propio. Un solo positivo usa la petición aislada anterior idéntica; varios comparten consulta/reglas y llevan cada resumen únicamente en sus preguntas. El formato limita lo que contiene cada pregunta; no garantiza aislamiento interno del modelo.

La traza distingue `batch.usage`, contado una vez, y `batch.verdicts`, sin recibos por recuerdo. Servicio y auditor JSON independiente rechazan mezcla con checks individuales, falta de respuestas, IDs/revisiones ajenos, composición incorrecta y consumo duplicado. Ante fallo conserva recibos conocidos y degrada toda la operación. No cambia SQL, secretos, dependencias, puertos o configuración predeterminada.

135 pruebas offline pasan; una prueba remota manual se omite normalmente. Compilación y formato pasan. La prueba manual autorizada se ejecutó una sola vez y conservó previamente payloads, expectativas y huella del ejecutable. Estos resultados de software no demuestran exactitud semántica.

## Diagnóstico de aislamiento

Cuatro escenas de desarrollo, con dato correcto/ajeno y corrección correcta/ajena, presentadas solas, con un recuerdo ajeno y con texto adversario: **12/12 juicios objetivo esperados, cero cambios bajo distractor**. Coste completo USD 0,000595600, doce HTTP. El informe Rust independiente vincula solicitudes, preflight y diario, y comprueba el roster completo. No es holdout ni garantía de aislamiento. Su pase permitió el siguiente diagnóstico fijado previamente.

## Casos difíciles conocidos

| Grupo | Pérdidas frente a evidencia útil previa | IDs ajenos | Coste USD | Coste/base histórica | HTTP | CLI p50/p95 |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| 13 V5 | 2 IDs | 0 | 0,001400320 | 1,916× | 24 | 877 / 1190 ms |
| 5 V4 | 1 ID | 0 | 0,000729844 | 2,087× | 10 | 731 / 1020 ms |

En V5 se pierden `cev5_068_06`, corrección, y `cev5_069_12`, contexto parcial. Ambos estaban en la propuesta y el lote los rechazó. La segunda selección también omite `cev5_069_11`, permitido aunque no fuera exigido por la preservación histórica. Hay 12 verdaderos positivos y cinco falsos negativos dentro de los 91 candidatos clasificados; no equivale a completitud de todas las memorias del corpus. En V4 se pierde el parcial `cev4_062_27`, igualmente propuesto y rechazado. Se rechazan los dos IDs de contexto ajeno, pero no se compensa pérdida útil con esa mejora.

Los grupos usan etiquetas originales y fuentes congeladas. Las comparaciones de coste son contra subconjuntos históricos, no campañas emparejadas contemporáneas. Las latencias corresponden a estas ejecuciones CLI, no a una integración Codex; cinco/trece casos no establecen estabilidad.

## Diagnóstico retrospectivo

El código de propuesta V2 tiene la misma huella que en su ejecución previa. De las trece propuestas V5, diez tienen exactamente los mismos tipos y dieciocho casos conservan sus consultas/corpus originales; tres propuestas V5 y dos V4 varían. No se atribuye esa variación a una modificación del proponente. Se conserva una conciliación por caso en `outputs/iromem/evaluation/per-memory-role-batch-v1-pilot-2026-10-19/retrospective-comparison.json`.

Las dos pérdidas principales V5 ocurren con la misma propuesta que antes. La comprobación cambió de redacción V2 aislada a V1 completa representada en lote; por ello este contraste no aísla el efecto de agrupar de los distintos textos. Demuestra que el mecanismo ensayado falla, no cuál de esos dos cambios causa cada fallo. V1 aislada anterior es otra referencia histórica, no una repetición contemporánea controlada.

La propuesta V5 consume 14.960 tokens y USD 0,000628320, igual total de entrada/coste que V2 previa. Los once lotes de verificación añaden 7.720 tokens y USD 0,000772000. En V4 son 5.982 tokens de propuesta/USD 0,000251244 y 4.786 tokens de verificación/USD 0,000478600. Reducir HTTP de 38 a 24 en V5 no produjo ahorro total frente a V2: USD 0,001396120 → 0,001400320. No se infiere eficiencia de la cantidad de peticiones.

## Coste y siguiente paso

Este avance añade **30 operaciones, 46 HTTP y USD 0,002725764** con recibos completos. Acumulado del objetivo: **3341 operaciones, 21 campañas emparejadas completas, USD 0,238710414 conocidos y tres operaciones históricas de coste desconocido**. No incluye programación/revisión con Codex. Los doce controles de aislamiento y dieciocho casos conocidos no incrementan las campañas emparejadas completas.

Se descarta la expansión de este lote. El siguiente experimento propuesto es un contraste **solo de proveedor de comprobación**, con el texto completo V1 aislado y propuestas históricas congeladas: medir si Jev puede verificar esos positivos con precisión suficiente y menor coste. Antes de implementarlo en producto debe superar un diagnóstico que preserve todos los resultados negativos y positivos; la menor tarifa observada en otros recibos no demuestra ni ahorro equivalente ni precisión. No está implementado ni medido en este avance.

Siguen pendientes las puertas completas de desarrollo, consistencia, comparación nueva V6 con revisión independiente y decisión de incorporación. No se cambian etiquetas ni puertas para declarar pase.
