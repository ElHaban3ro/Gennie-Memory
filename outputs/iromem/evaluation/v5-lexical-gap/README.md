# Sondeo v5 equilibrado sin coincidencia textual

Este sondeo reúne **400 preguntas sintéticas**: 200 positivas y 200 negativas, repartidas por igual entre desarrollo y calibración. Cada partición usa 50 entidades distintas; no hay entidades compartidas entre ellas ni se abrió `final`. Las [etiquetas positivas](../../../../tests/benchmark/v5/lexical-gap-probe/manifest.json) y [negativas](../../../../tests/benchmark/v5/lexical-gap-probe/negative-manifest.json) se derivaron de casos v5 deterministas de la misma entidad. Las paráfrasis se redactaron para eliminar las coincidencias FTS5: diez frases cortas por modo positivo y diez frases en inglés por tipo negativo.

FTS5 devolvió **cero candidatos en los 400 casos**. La búsqueda híbrida con Laya MPS tuvo cobertura semántica completa y recuperó el recuerdo etiquetado entre sus dos candidatos en los 200 positivos. Los tres selectores híbridos recibieron **los mismos candidatos y vectores** por caso.

| Método | Positivos exactos / 200 | Negativos vacíos / 200 | Semillas exactas / 400 |
| --- | ---: | ---: | ---: |
| Filtros + FTS5, top‑1 | 0 | 200 | 200 |
| Híbrida, top‑1 | 113 | 0 | 113 |
| Híbrida, Laya `choice` | 0 | 191 | 191 |
| Híbrida, Laya `noul` condicionado (0,5) | 92 | 3 | 95 |

Por partición, top‑1 híbrido acertó 57/100 positivos en desarrollo y 56/100 en calibración; Laya condicionado 48/100 y 44/100. Laya `choice` se abstuvo en los 200 positivos. En negativos, `choice` dejó vacíos 95/100 y 96/100; condicionado 1/100 y 2/100. Top‑1 añadió una memoria en los 200 negativos. FTS5 acertó los negativos porque su búsqueda quedó vacía, pero perdió todos los positivos.

La exactitud total depende de la mezcla 50/50 elegida para este sondeo y oculta el intercambio entre recuerdos omitidos y falsos recuerdos. El índice semántico **sí aporta cobertura** cuando faltan términos compartidos, pero todavía no hay una selección fiable tras esa recuperación. En cada positivo acertado se conservaron hecho, condición y fuente declarada. No hubo fallos de decisión, semillas degradadas, filtraciones de IDs prohibidos ni llamadas remotas. Top‑1 usa Laya para codificar la pregunta en la ruta híbrida, aunque no lo llama como clasificador.

Las preguntas se diseñaron para provocar una ausencia FTS5; las positivas son frases breves, y las negativas expresan modalidad no decidida o una petición de no usar recuerdos. No representan una distribución de preguntas humanas, y las etiquetas derivadas no tienen revisión humana. Las latencias proceden de recorridos separados en orden fijo; [el resumen](summary.json) conserva los percentiles por método y grupo. La [ablación principal v5](../../../../docs/evaluation/retrieval-ablation-v5.md) documenta el contraste con 3.080 preguntas que sí conservaban señales léxicas.

Para corregir la forma telegráfica de los positivos, se hizo después un [sondeo de 400 peticiones completas](../v5-natural-gap/README.md) con las mismas entidades y etiquetas derivadas, manteniendo la ausencia de coincidencias FTS5.

## Evidencia

- [Resumen validado por caso](summary.json) y [huellas de las 16 ejecuciones](execution-provenance.json).
- Positivos: [FTS5 desarrollo](development-fts5-top1.json), [híbrida top‑1 desarrollo](development-hybrid-top1.json), [Laya `choice` desarrollo](development-hybrid-laya-choice.json), [Laya condicionado desarrollo](development-hybrid-laya-conditions.json); [FTS5 calibración](calibration-fts5-top1.json), [híbrida top‑1 calibración](calibration-hybrid-top1.json), [Laya `choice` calibración](calibration-hybrid-laya-choice.json), [Laya condicionado calibración](calibration-hybrid-laya-conditions.json).
- Negativos: [FTS5 desarrollo](negatives-development-fts5-top1.json), [híbrida top‑1 desarrollo](negatives-development-hybrid-top1.json), [Laya `choice` desarrollo](negatives-development-hybrid-laya-choice.json), [Laya condicionado desarrollo](negatives-development-hybrid-laya-conditions.json); [FTS5 calibración](negatives-calibration-fts5-top1.json), [híbrida top‑1 calibración](negatives-calibration-hybrid-top1.json), [Laya `choice` calibración](negatives-calibration-hybrid-laya-choice.json), [Laya condicionado calibración](negatives-calibration-hybrid-laya-conditions.json).
- Los generadores históricos de este sondeo fueron retirados; los casos permanecen congelados. La base y el binario coincidieron con los de la campaña principal de calibración; el modelo Laya local se cargó en modo lectura.
