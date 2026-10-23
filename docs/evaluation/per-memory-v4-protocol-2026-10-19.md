# Clasificación compacta por recuerdo v4

2026-10-19. Hipótesis nueva tras fallos v2/v3: Decisions puede distinguir mejor los pares que Jev, pero hace falta reducir instrucciones repetidas para contener coste. `per_memory_v4` mantiene prefijo ocho, cuatro relaciones, todos los positivos, un recibo global y reglas compartidas breves; cambia proveedor experimental a Decisions y payload compacto. No atribuir un resultado solo al proveedor: también cambia el texto.

No reducir a cuatro: el análisis local del diario v5 pierde dos IDs permitidos de base al recortar4,5o6. Ocho conserva los IDs observados; eso no demuestra cobertura de todo el corpus.

Diagnóstico inicial: diez preguntas conocidas v5 (cinco errores v2, dos complementarias, una refutación y dos cuyos IDs de base caen después del sexto puesto). Diez operaciones máximo, CLI sin puerto, DB temporal propia, duración prevista hasta30s. Nunca constituye validación independiente. Antes de ampliar debe conservar todos los IDs permitidos de base y no seleccionar ninguno ajeno al conjunto anotado; si falla, conservar resultado y detener ampliación.

Si pasa diagnóstico, campaña pareada64preguntas/128operaciones conocida. Puertas originales: ninguna selección ajena, ninguna evidencia permitida de base perdida ni degradación nueva; mejora útil o evidencia adicional; recibos y operaciones completos; coste,p50,p95<=1,5veces base. No cambiar puertas tras observar. Predeterminado Decisions/baseline intacto; v6 congelado con revisión separada espera desarrollo aprobado. Los tokens de proveedor y bytes de semilla no son tokens ni precio de Codex.
