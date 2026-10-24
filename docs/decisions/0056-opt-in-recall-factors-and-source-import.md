# 0056 — Factores, calibración empírica y adquisición explícita de fuentes

Fecha: 2026-10-24. Estado: mecanismos optativos implementados y evaluados; calibradores medidos rechazados para adopción por empeorar Brier en reserva.

El usuario encargó completar los pendientes de recall y M4 mediante agentes y devolución del trabajo terminado. Se reutiliza `per_memory_factors_v1` en producto v1 para separar aplicabilidad, dato solicitado y corrección por candidato conservando preguntas/reglas y baseline. El perfil de producto incorpora condiciones, estado y fuente junto al resumen, sin modificar la presentación heredada de recall. Se conservan probabilidades individuales válidas y se añade un calibrador optativo de frecuencias suavizadas, ligado por SHA256 exacto al adaptador, proveedor/política/modelo y datos de calibración. La aplicación de puntuaciones es informativa y no selecciona recuerdos. No equivale a verdad factual ni garantiza rendimiento fuera de la cohorte.

Un conector HTTPS personal explícito importa texto UTF-8 cuyo hash esperado aporta el operador, con límites/timeout y sin credenciales/redirects/proxies. Registra usando la conexión y contrato de snapshots existentes, después de adquirir/verificar bytes fuera de transacción. La prueba de adquisición aparece en un recibo separado; no se reetiqueta el snapshot como autenticidad universal. No se añaden tablas o dependencias, ni se leen automáticamente URLs de recuerdos.

Alternativas descartadas: inventar scores con booleanos; promover el baseline sin evidencia comparativa; reactivar E5; calibrar con etiquetas de final; descargar URLs durante recall; presentar registro manual como autenticación remota. Una calibración de tasa base es válida como mecanismo acotado, pero una mejora discriminativa requiere muestra mayor y evidencia independiente.

Coste: tres juicios por cada top 8 en una petición; mayor payload que una elección singular, más texto/trazas por candidato; red optativa acotada para importar fuentes; CPU local pequeña para bins. Sin coste de proveedor adicional por aplicar el artefacto. La calidad/coste real se mide con recibos y no se presume por estas decisiones.

[Contrato](../architecture/recall-completion-v1.md), [verificación y alcance](../evaluation/recall-completion-v1/README.md). Se revisa al cambiar prompt/parser/modelo, ampliar servicios autenticados o proponer nuevos umbrales/defaults. La huella cambia incluso ante cambios del archivo de pruebas; un nuevo artefacto exige nueva ligadura explícita.

Resultado reservado:48 operaciones v2 sin degradación; selección Decisions 8/8 y Jev 7/8. Los seis contrastes Brier por factor/proveedor empeoraron al aplicar los calibradores de dos intervalos; se conservan experimentales para reproducción, sin umbral o baseline promovidos. La igualdad del score publicado con el artefacto demuestra aplicación, no éxito semántico. [Resultados](../evaluation/recall-completion-v1/README.md).
