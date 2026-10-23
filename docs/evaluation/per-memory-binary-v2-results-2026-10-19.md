# Utilidad binaria compacta v2: piloto y campaña conocida

2026-10-19. **No pasa calidad ni recursos; no se amplía a v4/v2 ni a V6.** El piloto favorable de diez consultas no se sostuvo en el banco completo. Baseline permanece predeterminada y el goal sigue activo.

## Variante y piloto

Una petición Jev clasifica ocho recuerdos con instrucciones compartidas estrictas, memoria como dato y utilidad binaria. Se conservan todos los positivos. Hipótesis, gold y puertas se fijaron antes de llamar; no se modificó la versión anterior. El piloto conocido pasó diez consultas sin ID ajeno ni pérdida de ID permitido de base: diez HTTP y USD0,000462588,0,8877× la base histórica del subconjunto. Eso permitió ampliar, no aprobar.

## Comparación pareada completa v5

64preguntas conocidas,128operaciones con orden alternado Decisions/baseline frente a Jev/binaria-v2. Corpus, preguntas, etiquetas y candidatos idénticos; políticas explícitas por caso. Todas las operaciones y recibos completos. La auditoría ordinaria valida1.940candidatos y87memorias de semilla, revisiones, fuentes declaradas, ámbitos y eventos. No certifica verdad factual.

| Relación de pregunta | Base | Binaria v2 | Total |
| --- | ---: | ---: | ---: |
| Apoyo: semilla útil |16 |8 |16 |
| Refutación: semilla útil |15 |16 |16 |
| Parcial: semilla útil |15 |14 |16 |
| Sin evidencia: semilla vacía |15 |16 |16 |
| Semillas con algún ID ajeno |1 |0 |64 |

Hay nueve consultas que pierden evidencia útil de base; ninguna degradación técnica. Una recuperación nueva de refutación y dos aportaciones adicionales no compensan esas pérdidas. Las nueve tenían el recuerdo permitido en la primera posición, dentro del prefijo8: el recorte no explica sus pérdidas. No exigir responder todas las partes ni confundir requisito con exclusividad; el texto más estricto no aseguró la distinción. La consulta de la sonda pasó el piloto y luego se abstuvo; no hay garantía de repetibilidad a partir de una sola observación.

| Recurso | Base | Binaria v2 | Cociente |
| --- | ---: | ---: | ---: |
| USD informados |0.003480200 |0.002980572 |0.8564 |
| Tokens de entrada del proveedor |34.802 |70.966 |2.0391 |
| Mediana de operación, ms |375.12 |352.54 |0.9398 |
| p95 de operación, ms |604.41 |916.47 |1.5163 |

Coste total de ambas políticas USD0,006460772. P95 supera el límite1,5×: no redondearlo a un pase. Menor coste pese a más tokens procede de proveedores/modelos distintos; no mide tokens enviados a Codex ni coste de programación. Tiempos del runner de recall, no recorrido completo hasta respuesta de usuario. No se midió verdad de respuesta generada ni todas las partes disponibles en esta cohorte; V6 incorpora otra anotación prospectiva.

## Revisión y siguiente diagnóstico

Revisión estática separada del código: parser binario, límite8, semilla completa, un solo recibo y puertas conservados. Se corrigió el preflight para exigir huellas de todas las fuentes y rechazar una política de tipo inválido; compatible con el piloto verificado anterior.87pruebas workspace y fmt check pasan. No hay cambio UI, servidor, SQL, secretos ni dependencia.

Las instrucciones de cada pregunta en esta variante contienen solo la memoria; la tarea está en state/criterios comunes. La siguiente hipótesis comprobable es que hacer explícita la acción de clasificar esa memoria respecto de la consulta en cada pregunta recupera evidencia directa. Todavía no está demostrada: comparar versiones congeladas con el resto del payload constante, incluir pérdidas y negativos difíciles, medir coste completo y detener ante regresión. No utilizar V6 para ajustarla ni llamar mejora al mero ahorro o a un piloto seleccionado.

[Protocolo anterior a llamadas](per-memory-binary-v2-protocol-2026-10-19.md), [piloto](../../outputs/iromem/evaluation/per-memory-binary-v2-pilot-2026-10-19/report.json), [campaña pareada](../../outputs/iromem/evaluation/per-memory-binary-v2-development-2026-10-19/paired-v5/report.json), [auditoría](../../outputs/iromem/evaluation/per-memory-binary-v2-development-2026-10-19/paired-v5/audit.json). Se preserva también el [piloto verificado fallido](per-memory-verified-v1-results-2026-10-19.md).
