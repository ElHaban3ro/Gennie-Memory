# 0013 — Protocolo previo a la evaluación final del MVP

Fecha: 2026-10-16. Estado: protocolo aplicado a la cohorte sintética; la revisión humana independiente y la decisión final de producto siguen pendientes. Continúa la política de [0010](0010-keyword-search-default.md) y [0012](0012-keyword-prefix-expansion.md), sin cambiar la ruta del producto. [Resultados](../evaluation/mvp-final-synthetic-2026-10-16.md).

## Contexto

Las cohortes 014–017 midieron 441/460 semillas positivas correctas con Decisions y 80/80 negativas vacías, pero reutilizan plantillas del mismo fixture. El banco solo llega al sujeto 020 en `project`, `conversation` y `agent`; no permite reservar cuatro sujetos completos posteriores a 017. Falta una evaluación final sin ajuste posterior y una muestra independiente de preguntas naturales revisadas.

## Decisión

Reservar 018–020 como cohorte sintética final de doce ámbitos, 345 positivos y 60 negativos. Fijar de antemano la ruta FTS5 por palabras clave con máximo veinte candidatos, Decisions como principal y Jev como comparación independiente, sin cascada. Usar 95 % observado de semillas positivas correctas (al menos 328/345) como meta exploratoria y exigir informar cualquier falso recuerdo en los 60 negativos. Separar cobertura, selección, fidelidad de semilla, degradación, latencia y coste, con resultados por tipo de ámbito e intervalos de incertidumbre. No ajustar la política al ver esta cohorte.

La [especificación operativa](../evaluation/mvp-final-protocol-2026-10-16.md) congela hashes, auditoría previa, secuencia y tamaño de campaña. La [muestra de 48 etiquetas](../evaluation/mvp-final-audit-sample-2026-10-16.md) está preparada, pero aún no tiene revisión humana. No llamar «humanas» a preguntas o etiquetas derivadas de plantillas o redactadas por un modelo.

## Alternativas y coste

Reutilizar 014–017 abarataría llamadas, pero ya se observaron sus resultados y no sería una prueba final independiente. Añadir solo sujetos `user` posteriores a 020 aumentaría cantidad a costa de perder equilibrio entre los cuatro tipos. Crear ahora otro corpus permitiría mayor diversidad, pero requiere diseño y revisión de etiquetas; se tratará como prueba separada. La cohorte elegida implica hasta 810 llamadas remotas en dos proveedores y una muestra pequeña de sujetos por tipo, con limitación estadística explícita.

## Condición de revisión

Revisar el protocolo si la auditoría previa encuentra etiquetas ambiguas o incorrectas, antes de ejecutar la campaña. Si ya se abrió algún resultado, conservarlo como diagnóstico y reservar una cohorte nueva para cualquier prueba final posterior. El resultado sintético, aunque supere la meta, no aprueba por sí solo integración Codex ni calidad en preguntas humanas.
