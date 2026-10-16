# 0012 — Expansión por prefijo en la búsqueda rápida

Fecha: 2026-10-15. Estado: aceptada para la ruta rápida sintética; revisable con datos menos repetitivos. Continúa [0010](0010-keyword-search-default.md) sin cambiar la elección de FTS5 como predeterminado.

## Contexto

La ruta rápida original recuperó 452/460 positivos finales. Entre los ocho ausentes había formas como `invitación`/`invitaciones` y `documenta`/`documentación`; la lista de candidatos no estaba vacía, de modo que un respaldo activado solo ante lista vacía no servía. La selección de Decisions también fallaba cuando el recuerdo correcto sí estaba disponible.

## Decisión

Mantener el término exacto y añadir su prefijo FTS5 al quitar dos caracteres finales para términos de al menos siete caracteres. La expansión ocurre solo en la consulta `fts5_keywords`; no añade columnas, no reindexa memorias y no relaja filtros estructurados. Precisar la instrucción de Decisions para que distinga el atributo preguntado de la coincidencia de nombre y admita paráfrasis claras. Jev sigue seleccionable, sin cascada.

Los ocho fallos ya publicados de 002–005 orientaron la idea de expansión. Se eligió la combinación con 115 positivos y 20 negativos de desarrollo, antes de repetir esa partición conocida. Se rechazó por ahora un diccionario específico de sinónimos para `supuesto abandonado`: el único ejemplo de desarrollo y los tres finales no justifican una regla general. La cohorte de sujetos 010–013 quedó separada para una comprobación adicional sin ajustar el cambio.

## Evidencia y coste

En 002–005 reutilizado, candidatos 457/460 frente a 452/460; semillas correctas 442/460 frente a 424/460; negativos vacíos 80/80 en ambos. La búsqueda p50 subió de 5,72 a 6,45 ms; consulta a semilla p50 de 376 a 382 ms y p95 de 558 a 631 ms. El prompt de Decisions informó 25 tokens de entrada adicionales de mediana. En sujetos nuevos 010–013, candidatos 456/460, semillas 439/460 (95,4 %) y negativos vacíos 80/80; consulta a semilla p50 386 ms y p95 958 ms. El [informe](../evaluation/keyword-prefix-v1.md) enlaza los diarios y separa las corridas.

## Condición de revisión

Revisar con preguntas humanas y ámbitos más variados: un prefijo puede ampliar demasiado términos y desplazar recuerdos relevantes. También revisar si los quince fallos de selección con candidato disponible y los tres vacíos léxicos requieren cambios distintos. No proyectar el tiempo del piloto Codex anterior a esta ruta hasta medirla allí.
