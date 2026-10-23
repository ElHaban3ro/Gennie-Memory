# Diagnóstico local del gradiente de margen V1

Diagnóstico completado sobre los 176 ejemplos de entrenamiento del checkpoint V5 matched congelado. **Cero actualizaciones y pesos íntegros antes/después**. No hay nueva calidad medida ni adopción; el último mecanismo sigue fallando con 59/64 conocidos, 13/17 útiles y un ajeno. Objetivo original activo, reservas cerradas.

[Protocolo previo](utility-gradient-v1-protocol-2026-10-22.md). Evidencia en `outputs/iromem/evaluation/utility-gradient-v1-2026-10-22/`; matriz FP32 y cuatro controles persistidos.

## Sensibilidad local y pérdida

| Estrato de entrenamiento | Cantidad | Jacobianos casi nulos | Derivada total cero | Norma mediana Jacobiano | Norma mediana ideal de pérdida |
| --- | ---: | ---: | ---: | ---: | ---: |
| irrelevant_wrong | 19 | 0 | 0 | 275.784 | 227.132 |
| irrelevant_correct | 85 | 0 | 0 | 153.369 | 0.148384 |
| useful_wrong | 2 | 0 | 0 | 226.905 | 1488.9 |
| useful_correct | 70 | 0 | 0 | 153.824 | 0.00271171 |

Pares útil/ajeno de la misma consulta: 104; cosenos de corrección ideal definidos 104, indefinidos/casi nulos 0. Direcciones opuestas 104, alineadas 0, ortogonales 0. Se conservan 56 consultas, incluidas 8 sin útiles.

Las direcciones y normas son evidencia local en esta parametrización y checkpoint. Una oposición no demuestra por sí sola causa del fallo, falta de capacidad global o el paso efectivo de AdamW; sus estados finales no están persistidos. Las derivadas y vectores ideales se distinguen del autograd ejecutado en precisión mixta.

## Controles numéricos

| Control fijado por estrato | Norma real | Norma reconstruida | Error relativo L2 | Coseno | Compatible |
| --- | ---: | ---: | ---: | ---: | --- |
| t_f01_01 | 0.00138941 | 0.0014605 | 0.785436 | 0.707803 | False |
| t_f01_02 | 0.000580961 | 0.00051938 | 1.13959 | 0.279965 | False |
| x_f13_q02_m02 | 899.232 | 899.376 | 0.00106359 | 0.999999 | True |
| x_f13_q01_m01 | 0.0105253 | 0.0125204 | 0.403687 | 0.946605 | False |

Compatibilidad conjunta: False. Las tolerancias previas son L2 relativo ≤0,05 y coseno ≥0,99 con normas >1e-12; casi nulos requieren máximo absoluto ≤1e-6. Cuatro controles no prueban igualdad universal. Un desacuerdo se conserva y limita los vectores reconstruidos, sin repetir o mover tolerancias.

## Integridad, coste y límites

176 forward/backward de margen y 4 del objetivo completo, cero pasos de optimizador. Duración 29.035 s. Todos los scores de entrenamiento reproducen sus decisiones, delta máximo 8.52353e-08; no son casos frescos ni una mejora de recuperación.

Auditoría independiente: 10137/10137 comprobaciones de integridad. Recalcula métricas desde vectores, columnas, filas, controles y eventos; no vuelve a ejecutar el modelo. Código/runtime tienen hashes de preflight y auditoría final; no se afirma vigilancia continua de archivos durante ejecución.

API nueva USD 0, cero llamadas a proveedores; coste económico local y Codex desconocido. Acumulado externo intacto: 6155 operaciones, 23 campañas emparejadas completas, USD 4,566379428 conocidos más 24 operaciones de importe desconocido, excluyendo Codex.

Mismos 213 hashes de fuente Rust/Cargo y binario: se heredan 879 pruebas aprobadas/24 ignoradas por defecto, 881/75 con HTTP/2 y formato previo, sin repetir Cargo para scripts aislados. Sin dependencias/pesos nuevos, servicios/puertos, DB, secretos, env, procesos ajenos, rama, commit o push.

No se cargaron ejemplos de desarrollo, conocidos, closed o V6 para este diagnóstico. Siguientes puertas intactas: mecanismo que preserve evidencia y rechace ajenos; después casos nuevos con revisión ciega y comparación vigente, recibos/coste/p50/p95 ≤1,5×, estabilidad y V6 final. El diagnóstico no autoriza abrir reservas o adoptar el checkpoint fallido.

## Lectura causal y estudio exploratorio

No se observa un camino local muerto: todos los 176 Jacobianos de margen son no nulos, incluidos los dos útiles mal seleccionados y los 19 ajenos aceptados. Las 104 comparaciones útiles/ajenos presentan correcciones ideales opuestas (coseno mediano −0,6085). Es una medida local de acoplamiento en este checkpoint, no una prueba de incapacidad global ni de que el optimizador haya seguido esos vectores. Tres controles con pérdida CE pequeña difieren materialmente de la reconstrucción ideal; el útil incorrecto controlado sí coincide dentro de las tolerancias fijadas. No se atribuye todo el fallo a precisión numérica.

Se añadió un cálculo **exploratorio posterior**, sin modelo ni actualización, con los vectores ya guardados: resolver igualdades linealizadas para alcanzar margen firmado 2 donde hay déficit y dejar cambio cero en los 149 márgenes ya fuertes. El Gram tiene rango numérico 176/176 con corte relativo 1e-8 y condición aproximada 117209. Existe una solución de mínima norma para esas igualdades: residuo máximo 4,79e-11. No se afirma mínima norma para el problema más amplio de desigualdades.

La comprobación algebraica independiente pasa 12/12 mediante una resolución directa distinta de la descomposición usada inicialmente. Delta L2 0,45486, 6,81% de la norma conjunta de adaptadores; el cambio relativo llega a 72,39% en un bloque. Por ello no se presupone régimen lineal. El archivo de delta es una propuesta matemática sin aplicar, no un checkpoint mejorado ni una nueva métrica de calidad. Tanto fuente como revisión se conservan en el directorio del diagnóstico.

Siguiente paso único recomendado y aún no ejecutado: congelar protocolo, delta exacta y coeficiente 1, aplicarla una sola vez en una copia temporal del checkpoint y reproducir los 176 ejemplos entrenados. Comparar cambios reales de margen contra los predichos, incluidos los 149 protegidos, con tolerancias previas. Cero backward/optimizador/búsqueda de escalas; sin desarrollo, conocidos o reservas. Un acuerdo solo apoyaría esa intervención local; un desacuerdo impediría inferir mejora desde el surrogate. La autorización general de pruebas ya está concedida; no hay un permiso pendiente.

Un TypeError del auditor por claves duplicadas al combinar diccionarios se conservó con su fuente (`auditor-failed-v1.*`) y se corrigió antes de la auditoría offline final. No cambió inferencias, pesos o datos. Los hashes finales de runner, freezer, análisis y protocolo se revalidaron; no equivale a vigilancia continua durante la ejecución.
