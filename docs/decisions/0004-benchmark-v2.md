# 0004 — Corpus v2 con respuestas variables y mil formulaciones

Fecha: 2026-10-15. Estado: implementado como infraestructura experimental; calidad lingüística y etiquetas pendientes de revisión humana.

El banco v1 repetía el mismo valor de preferencia para cada tipo de tarea en todas las entidades. Eso permitía inferir la respuesta a partir del nombre de la tarea sin recuperar la memoria. Se conserva v1 como historial de los pilotos y se añade v2 sin alterar sus hashes ni resultados.

v2 asigna uno de seis valores ficticios por tipo de tarea a cada entidad mediante una regla determinista. Usa los mismos 400 proyectos y ámbitos simulados, con 40.000 positivos y 1.600 negativos. Enumera 1.000 formas superficiales: 20 marcos de petición, 10 prefacios y 5 cierres. Cada proyecto recibe 100 formas que cubren los 20 marcos; las particiones se mantienen por entidad. Los negativos nuevos preguntan por una tarea distinta del mismo proyecto o piden explícitamente no consultar recuerdos. Un párrafo distractor crea casos de contexto más largo sin incluir la respuesta.

Esta variedad comprueba más situaciones que v1, pero las 1.000 combinaciones no son 1.000 intenciones humanas independientes. Los valores se generan por regla, no por juicio humano; la revisión manual de una muestra y escenarios de condiciones, fuentes inciertas y sensibilidad siguen pendientes. No se usará la partición final para ajustar instrucciones, umbrales ni el generador. Ver [contrato](../evaluation/contract-v1.md), [pilotos](../evaluation/pilot-2026-10-15.md) y [roadmap](../../ROADMAP.md).
