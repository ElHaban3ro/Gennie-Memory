# Catálogo de referencias V1: suficiencia y conservación antes de red

UTC 2026-10-21. Paso 3 del goal activo. El diagnóstico anterior fue progreso: localizó residuales y refutó su uso como certificación semántica. Esta investigación comprueba la hipótesis de dividir referencia y hechos; no añade extractor, prompt o política de producto.

## Hipótesis y límites

Un catálogo local guarda referencias completas; la eventual etapa de cláusulas usaría IDs locales en subject/reference en vez de copiar todos sus campos. El prototipo puro materializa cada ID mediante copia exacta, rechazando IDs desconocidos, duplicados, referencias inline editadas o markers fuera de slots. No cambia campos de referencia, no repara desde texto o gold y no infiere correferencia. IDs iguales en fuente y pregunta son namespaces distintos. Referencias no usadas quedan visibles: no obligan a inventar hechos sobre cada nombre de un propietario. Equivocarse de ID permitido o predicado sigue siendo posible; conservar un objeto no lo vuelve verdadero.

`reference_catalog.rs` contiene solo el materializador local. `automatic_extraction/reference_matrix.rs` ejerce el traductor grounded V1, parser y proyección V5 existentes sobre construcciones manuales. No modifica esos mecanismos. `audit-reference-catalog tests/fixtures/reference-catalog-controls-v1.json <salida-nueva.json>` conserva los ocho controles LA01–08, aunque falle un lado, y escribe solo una salida nueva. Cada lado tiene texto, catálogo y cláusulas/pregunta autor-manuales. No representa salida de un modelo ni calidad de extracción.

## Matriz completa

Los ocho controles cubren: propietario distribuido entre frases; adjetivo como premisa corregible; propietarios en otro orden superficial; universal plural frente a miembro/clase singular; nombre homónimo con propietario distinto; componente aditivo sin total inventado; intermediario depósito frente a origen de red; La Paz frente a Paz sin borrar artículos. Las condiciones necesarias se declaran con el fixture antes del informe, separadas de resultados. Un revisor nuevo sin historial recibe textos y construcciones, sin condiciones ni resultados, para juzgar su interpretación semántica. Un crítico estático separado juzga código/expresividad, con exposición histórica declarada. No son críticos humanos ni independencia de familia.

Se distinguen tres cosas: representabilidad fiel, conservación exacta de objetos y compatibilidad útil de identidad entre fuente/pregunta. Un fallo de cualquiera impide el pase local completo; no se rebaja la puerta a la mera copia de objetos. No usar menciones de texto completo artificialmente como nombre/sintagma, ni cortar sensor desde sensores para fabricar una clase común. Un enlace externo a una mención puede necesitar una nueva versión explícita de evidencia, no un relajamiento silencioso del traductor histórico.

## Puertas, recursos y conservación histórica

Si la matriz no pasa íntegra, no se prepara ni ejecuta la primera etapa del modelo, y menos una tubería de dos llamadas. Registrar el defecto de representación/compatibilidad y corregir el contrato mediante otra versión revisada antes de repetir. Un eventual adaptador de evidencia distribuida debe conservar frase propia, cita del vínculo, portador, propietario, condiciones, tiempo y distinción fijo/supuesto; una clase compartida requiere una hipótesis distinta de normalización semántica, nunca un alias automático por cadena.

Incluso si la matriz pasara, solo habilitaría probar la capacidad de la etapa de referencias sobre todo el roster conocido; no seleccionar sus salidas buenas para alimentar la segunda. La tubería completa tendría que contar ambas etapas y respetar las puertas originales de calidad, p50/p95 y coste, sin amortización supuesta. Modelos, llamadas y coste no se inventan: este diagnóstico usa cero llamadas y USD 0 nuevos, sin medir latencia de recall/end-to-end.

Conservar construcciones iniciales y fallos técnicos, revisiones, informe, hash de entradas/fuentes y binario ejecutado. Reproducir cuatro bancos manuales y diez informes históricos. No tocar gold, defaults, README/gráficas, dependencias, SQL, servicios, dashboard o V6. Goal activo hasta calidad completa, casos difíciles, recursos/estabilidad, cohorte nueva ciega y adopción verificada.
