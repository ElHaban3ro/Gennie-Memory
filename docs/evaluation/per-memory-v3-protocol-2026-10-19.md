# Preguntas autosuficientes por recuerdo v3

2026-10-19. `per_memory_v3` usa Jev y hasta ocho preguntas, cada una con consulta original y un único recuerdo literal. Mantiene cuatro relaciones, composición de todos los positivos, validación completa y recibo global único. Busca corregir mezclas de sujetos/atributos observadas en v2 sin filtros de IDs ni vocabulario de dominios del banco. V1/v2 quedan congeladas; predeterminado baseline.

Primero: diagnóstico conocido de ocho consultas v5: cinco errores v2, dos evidencias complementarias y una ausencia. Ocho operaciones, CLI sin puerto, DB temporal propia, duración estimada hasta30s. No prueba generalización ni estima ratios de recursos del banco. Si no conserva todas las selecciones permitidas de base o añade cualquier ID no permitido, detener antes de ampliar, conservar resultado y diagnosticar.

Solo si pasa: comparación pareada64preguntas v5 conocida,128operaciones. Puertas sin cambios: todos los IDs seleccionados permitidos, ningún ID permitido de base perdido, ninguna degradación nueva, mejora útil o evidencias adicionales; operaciones/recibos completos y coste/p50/p95 <=1,5veces base. Todos los positivos valen, ninguna truncación oculta. No repetir llamadas inciertas ni convertir bancos observados en validación nueva. Banco v6 espera conciliación de revisión ciega y desarrollo aprobado antes de llamadas.
