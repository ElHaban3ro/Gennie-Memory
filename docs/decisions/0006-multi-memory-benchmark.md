# 0006 — Varias memorias por entidad en el banco v4

Fecha: 2026-10-15. Estado: implementado como escenario experimental; evaluación final pendiente.

v3 daba al clasificador una sola memoria activa por entidad y ámbito, de modo que filtrar la entidad resolvía casi toda la elección positiva. Para medir discriminación entre recuerdos de un mismo proyecto se creó v4 sin modificar v1–v3: tres preferencias activas de tareas distintas por proyecto, una cuarta tarea sin recuerdo para negativos y 40.000 preguntas positivas repartidas entre las tres. Los 1.640 IDs de memoria son hashes opacos con un prefijo idéntico, sin marca visible de «principal» o «auxiliar».

El validador comprueba identidad, ámbito, estado, hecho esperado, tarea, particiones y hashes. En 50 casos de desarrollo, FTS5 puso 23/30 positivos primero, el híbrido con encoder Laya 19/30 y Laya `choice` solo seleccionó correctamente 1/30 tras FTS5 y 2/30 tras híbrido. Ambos métodos recuperaron el positivo en top 20, y el clasificador lo tuvo entre cuatro candidatos en los 30 casos. La abstención local, no la ausencia del candidato, dominó el fallo de semillas. Jev y OpenAI Decisions acertaron cuatro casos cada uno en un sondeo seleccionado por hash; esa muestra no sustenta una comparación general.

Se mantiene el filtro de entidad porque corresponde a contexto estructurado que un host puede aportar, pero el host de v4 sigue siendo sintético. La campaña amplia, revisión humana de etiquetas, condiciones, fuentes y un perfil de embedding independiente siguen pendientes. Ver [contrato](../evaluation/contract-v1.md), [pilotos](../evaluation/pilot-2026-10-15.md) y [roadmap](../../ROADMAP.md).
