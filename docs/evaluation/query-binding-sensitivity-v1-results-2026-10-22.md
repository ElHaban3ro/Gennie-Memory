# Sensibilidad a la pregunta: señal conservada, ampliación perjudicial

Paso 3 activo. Diagnóstico de fuentes conocidas, sin entrenamiento. Con el mismo encoder y los mismos vectores, el lector original responde mejor a cambios de pregunta que los dos lectores ampliados. No se incorpora ningún mecanismo ni se abre reserva. [Protocolo](query-binding-sensitivity-v1-protocol-2026-10-22.md).

| Medición | Control152 | Ampliado152 | Estadísticas fijas153 | Yes/no base |
| --- | ---: | ---: | ---: | ---: |
| Aciertos /18 | 16 | 8 | 7 | 10 |
| Útiles conservados /10 | 10 | 5 | 4 | 10 |
| Ajenos seleccionados /8 | 2 | 5 | 5 | 8 |
| Parejas completas /6 | 4 | 0 | 0 | 0 |
| Ternas completas /2 | 2 | 0 | 0 | 0 |
| Parejas con útil por encima de ajeno /6 | 6 | 6 | 6 | 5 |
| Premisa conservada: ambos B seleccionados /2 | 2 | 0 | 0 | 2 |
| Retirada de premisa: B seleccionado y none rechazado /2 | 2 | 0 | 0 | 0 |

## Interpretación limitada

Los tres lectores aprendidos ordenan bien útil frente a ajeno en las seis parejas. El control acierta además ambas ternas: conserva la corrección al cambiar el atributo solicitado y deja de seleccionarla cuando se retira la atribución falsa. Con los mismos features, el ampliado pierde esa conducta. Esto contradice atribuir todos los fallos del diagnóstico a una ausencia total de señal del encoder; no demuestra suficiencia general de ese encoder o del control, que siguen fallando los432 originales.

En la segunda terna, el ampliado puntúa la corrección útil con 0.503507 y la consulta sin premisa falsa, ajena, con 0.673604. Un único corte monótono no puede seleccionar la primera y rechazar la segunda. Es una consecuencia de esos scores fijos, no un barrido de umbrales o propuesta de calibración retrospectiva.

La ampliación y su interacción con el lector merecen la siguiente intervención; estos18casos no identifican por sí mismos una causa única, ni prueban que separar roles o añadir datos vaya a mejorar. La base yes/no conserva10/10útiles pero selecciona8/8ajenos: tampoco basta usar directamente su score.

## Banco, revisión y separación

V1 de20filas se conserva sin ejecutar. Antes de inferencias se retiran dos contrastes que cambiaban identidad, fuera del alcance previsto, y se separan atributo/premisa en dos ternas. V2: ocho fuentes y18filas, ocho originales íntegros y diez preguntas nuevas. Seis parejas cambian relación pedida; dos ternas distinguen B→B→none. Los contrastes instancia/plantilla y perfiles se informan con su límite: no hay un único factor lingüístico común a todos.

Revisión fuente separada sin etiquetas:18/18de acuerdo binario y17/18de rol inicialmente. La revisión canónica de una ausencia explícita confirmó A, conservando el dictamen C inicial: el valor no conservado es un estado explícito del atributo, según rúbrica anterior. Conciliación completa antes del encoder; ningún gold original o borrador se modificó tras resultados. Misma familia Codex con historia del proyecto; no validación humana o independencia de familia.

Cada documento usa los tres lectores que excluyeron su componente original completo y homólogos. Ocho features cacheados y diez nuevos; las ocho predicciones de referencia se reproducen con diferencia máxima2,776e-16, dentro del límite1e-12. El resultado sigue siendo diagnóstico sobre fuentes observadas, no generalización fresca, nuevoTRAIN o aprobación del paso4.

## Verificación y coste

Preflight independiente396/396 antes de inferencia y275archivos congelados intactos. Se comprueban310pesos contra BF16persistido convertido a FP16, fingerprints, tokens completos y proyección del mismo hidden. Auditor CPU independiente reconstruye scores, normalizaciones, exclusiones, parejas y ternas sin encoder o refit.

La primera auditoría falló por un error de su script: exigía counters en forward_started, evento que no los declara. El informe fallido permanece intacto; la corrección se limita al auditor y la segunda auditoría pasa. La versión de script fallida fue reconstruida invirtiendo esa única corrección, no capturada antes de modificarla; esa limitación está declarada en audit-correction.json. Fuentes congeladas y resultados del experimento no cambiaron.

Diez forwards nuevos, cero fits/backwards/actualizaciones del transformer. Duración instrumental del runner: 7.883024s; picoRSS: 3783491584bytes; picoMLX: 1591623904bytes. Cero API/USD0 nuevos; costeCodex,energía y amortización desconocidos. No es latencia productiva. CLI terminal con exit0observado,sinpuerto/servicio.

Las218fuentes Rust/producto no cambiaron. Se heredan928pruebas default y930HTTP2del150; no se repiten para este diagnóstico de investigación. Formato actual verificado por separado. Producto,base del usuario,dependencias y secretos conservados. Acumulado externo permanece6319operaciones,USD4,582739128 conocidos y25sinimporte.

## Estado del objetivo

No se cambia el corte ni se incorpora el lector anterior por16/18curados. Sus343/432previos siguen fallando calidad frente al padre374/432; los ampliados328/432y326/432también. Todas las puertas originales permanecen:24/24desarrollo,64/64conocidos,seis conjuntos,recursos≤1,5×base,recibos,estabilidad yV6ciega. Objetivo activo,2/5etapas cerradas. La siguiente intervención se elegirá con crítica de diseño sobre la interferencia entre modos de utilidad, antes de otro ajuste.

Se examinó además una propuesta de conservar las direcciones del control y ajustar únicamente el intercepto. Se descartó **sin entrenarla**: en dos componentes, un ajeno que el padre rechazaba ya puntúa por encima de un útil que el padre conservaba. Por ejemplo,0,35494 frente a0,02133enelcomponenteB. Ningún desplazamiento escalar puede rechazar el primero y retener el segundo. La prueba de orden, con IDs y restricciones originales, queda en offset-preservation-impossibility.json; no es búsqueda de umbrales.

La siguiente hipótesis elegida es exploratoria: tres lecturas A/B/C supervisadas por separado y selección por OR, con los mismos datos/features/folds/receta. Permite una unión de fronteras lineales; cambia objetivos, capacidad y desequilibrio de clases, por lo que cualquier ganancia no demostraría por sí sola que la mezcla de roles era la causa. Los roles serán objetivos TRAIN, nunca entradas runtime. Se preparan controles de implementación; todavía no se entrenó ninguna cabeza real ni se midió esta candidata. Su coste previsto es quince ajustes CPU, cero forwards de encoder y cero API, sin barrido de regularización o corte.
