# Calibración v5 de memorias semilla: muestra preparada

Estado: **muestra congelada y campaña local completada**; ver [resultados por caso](../../outputs/iromem/evaluation/v5-campaign-calibration/README.md). La [campaña de desarrollo](../../outputs/iromem/evaluation/v5-campaign-development/README.md) midió antes los mismos tres selectores. Esta partición contrasta su comportamiento en entidades distintas antes de abrir `final`; sus etiquetas no se usaron para elegir las variantes de desarrollo.

## Corpus

- Archivo: [`calibration-campaign-1480.jsonl`](../../tests/benchmark/v5/calibration-campaign-1480.jsonl), derivado del banco v5.
- SHA-256: `e93b7a64d0083603423f3fa21042475c7a6738054bd25d14b1b4ed746e3209a5`.
- 1.480 consultas sintéticas de `calibration`: una positiva por cada una de las 1.000 formas y 480 negativas, 80 por cada uno de seis segmentos. Los 1.480 IDs son únicos.
- 171 entidades; cero IDs y cero entidades compartidos con la campaña de desarrollo de 1.600 casos. Las etiquetas proceden de reglas deterministas del generador.
- Selección histórica congelada en `tests/benchmark/v5/calibration-campaign-1480.jsonl`; el selector Python original fue retirado. `iromem-eval validate-benchmark tests/benchmark/v5` comprueba el banco base versionado.

## Siguiente evaluación

Se compararon los tres selectores locales sobre la misma búsqueda híbrida y los mismos candidatos, con diarios reanudables y MPS. El presupuesto fue **tres recorridos de 1.480 casos, hasta 15 minutos activos en total y cero llamadas pagadas**. Ninguna invocación llegó al corte individual de cinco minutos. Se midieron exactitud de semilla por segmento, abstención, memoria equivocada, fidelidad de hecho/condición/fuente, latencias y fallos. La muestra aún no prueba generalización a preguntas humanas; tampoco se fijó una política de producción.

El runner local admite `--split calibration` con el archivo congelado como `--queries` y registra la partición en el manifiesto del diario y en el informe. El valor predeterminado sigue siendo `development`; `final` se rechaza. Hay una prueba de contrato que verifica la selección explícita de calibración y el rechazo de final. El [resumen](../../outputs/iromem/evaluation/v5-campaign-calibration/v5-calibration-summary.json) valida que los tres métodos vieron los mismos candidatos.

La regla de [pruebas](../rules/testing.md) exige definir presupuesto y duración y aprobar una campaña grande antes de iniciarla. La campaña local se ejecutó bajo la petición del usuario de probar un conjunto enorme con Laya en esta Mac; su alcance y límite se anunciaron antes de comenzar. El usuario autorizó después la comparación pagada de desarrollo; se completó y consta en outputs/iromem/evaluation/v5-campaign-remote/README.md. Además se evaluó un sondeo remoto de desarrollo/calibración con entidades separadas en outputs/iromem/evaluation/v5-natural-gap-remote/README.md.
