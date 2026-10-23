# 0028 — Conclusión existencial local en evaluación

Estado: implementada y verificada como capacidad optativa del evaluador; no adoptada en producto.

El agregador rechazaba la respuesta común de dos lecturas completas porque el término anónimo no tenía identidad canónica global. Cambiar esa identidad, devolver texto libre o sustituir la pregunta habría falseado el requisito. Se añade un canal tipado de conclusión existencial V2: cada lectura acredita su testigo local y el consumidor compara cuerpos completos, anclas externas históricas y mapas de origen. No afirma un objeto único compartido ni une variables entre conclusiones.

La alternativa de comparar solo clases/nombres se descarta por pérdida de restricciones y homónimos. La de intersectar atributos arbitrariamente se descarta porque puede retirar dependencias o correlaciones. El cierre es dirigido hacia los requisitos del apoyo, con metadata independiente preservada en originales. No se introduce otro contrato entre apps ni se cambia el producto.

El coste incluye normalización, mapas, verificación y mayor reporte local. Todos consumen presupuesto acumulado y los límites invalidan toda la operación. El primer subset y sus exclusiones son explícitos; no se presentan como capacidad general ni como negativos correctos. Siete controles CLI y 790 pruebas pasan, con 351 replays idénticos; los seis pendientes de fidelidad y la puerta de adopción siguen abiertos. Revisar cuando haya extracción coherente completa y comparación de utilidad/recursos en casos nuevos. [Informe](../evaluation/documentary-v3-existential-projection-results-2026-10-22.md).
