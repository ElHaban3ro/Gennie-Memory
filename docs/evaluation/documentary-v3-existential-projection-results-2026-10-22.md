# Conclusión existencial común sin identidad global

La candidata optativa resuelve el control obligatorio que fallaba: ambas lecturas de «Senda tiene una bandeja de papel grande» conservan ahora una conclusión común estructurada sobre el ítem, sin afirmar identidad global de la bandeja ni una propiedad común de tamaño. **Pasan 7/7 controles CLI, 790 pruebas de workspace y 351 replays exactos.** El objetivo sigue activo en el paso 3; los seis pendientes de fidelidad permanecen y no hay adopción de producto.

## Qué cambió y qué demuestra

El feature `existential_conclusion_projection` añade un canal de conclusión al reporte V2 del evaluador. La variable existe solo dentro de esa conclusión y solicitud. El cuerpo conserva átomo, roles, polaridad, juicio, fundamento, descripciones, enlaces y dependencias admitidas. Cada cruce entre lecturas aporta su prueba y mapa de origen; fuente, pregunta y matriz originales permanecen íntegros dentro del reporte. El consumidor reconstruye las pruebas antes de publicar completitud.

La conclusión común significa que cada lectura acredita algún ítem con esa descripción. No implica que las lecturas identifiquen el mismo objeto ni permite unir variables de conclusiones distintas. El papel sigue siendo un uso léxico del complemento nominal; no se introduce una entidad física independiente. La metadata de «grande» permanece atribuida por lectura y no se transforma en una propiedad común. Una dependencia efectiva hacia esa metadata impide dejarla como residual.

La fuente, pregunta y expectativa del control no cambiaron. Una comprobación de inversión recupera exactamente el input anterior al retirar únicamente el permiso nuevo y restituir los hashes del vocabulario. Con el perfil anterior el control sigue siendo condicional, como exige la compatibilidad. Con la candidata explícita recibe `completion_kind=existential_conclusion`, un cuerpo tipado consumible y un mapa por cada celda. Las funciones de identidad, `reference_key`, `canonical_inner` y `Same` conservan su significado.

El [diseño cerrado](documentary-v3-existential-projection-design-2026-10-22.md), su [contracrítica](documentary-v3-existential-projection-countercritique-2026-10-22.md) y el [addendum de implementación](documentary-v3-existential-projection-implementation-addendum-2026-10-22.md) distinguen el contenido existencial de la prueba local, el cierre dirigido de dependencias y los límites del subset. El nuevo módulo pertenece exclusivamente a `tools/evaluation`; no añade crates, dependencias, SQL, servicios o rutas productivas.

## Controles, fallos y compatibilidad

| Control CLI | Resultado final |
| --- | --- |
| Control nominal original, permiso nuevo y misma expectativa | Conclusión existencial completa con procedencia íntegra |
| Renombrado de IDs técnicos | Mismo contenido normalizado, mapas locales propios |
| Enlace restrictivo adicional en una sola lectura | Sin conclusión común; no borra la restricción |
| Metadata convertida en dependencia efectiva | Sin rescate que ignore la dependencia |
| Envolvente abierta | Sigue inelegible; no elimina lecturas |
| Una lectura sin apoyo | Sin conclusión común |
| Mismo nombre literal en otra ocurrencia documental | Sin fusión de anclas históricas diferentes |

Los seis controles iniciales quedaron declarados antes del código. El séptimo deriva de un hallazgo estático y se registra como diagnóstico conocido, no como caso nuevo reservado. Salvo la envolvente deliberadamente abierta, fuente y pregunta validan completas en todos los controles. Un verificador separado del generador comprobó los campos cerrados, cuerpo nominal, único binder, ausencia de «grande» como contenido común, punteros, índices y cobertura de todos los cruces de los dos positivos.

La revisión estática detectó tres problemas materiales y preserva sus snapshots: claves de anclas externas omitidas en equivalencia; punteros de procedencia que podían señalar otra arista; y límites locales que no marcaban agotamiento global. Se corrigieron conservando las claves históricas por lado y nodo, resolviendo extremos/etiquetas completos y centralizando todos los límites para invalidar la operación entera. No se afirma haber ejecutado una respuesta falsa con esos borradores. El contraejemplo de homónimos conserva el rechazo del binario anterior y del final.

Los primeros comandos fallaron por sintaxis; después dos tests focales fallaron por consumir fuera de la operación activa y por asumir un contador de presupuesto vacío tras el preflight. Se repararon las condiciones de prueba sin relajar el consumidor. Otra prueba pretendía omitir dos roles del patrón y el validador la rechazó: se sustituyó por una unidad explícita del normalizador con documentos válidos, sin afirmar cobertura integrada de consultas multislots. Los logs iniciales y los fallos permanecen separados.

La suite final verifica **790 aprobadas, cero fallos y 24 ignoradas**, con formato correcto. Incluye seis unidades nuevas: conclusión/procedencia y origen alterado, ausencia de feature/perfil inválido, renombrado, correlación de incidencias y uso léxico no físico, agotamiento compartido y límite local que invalida toda la operación.

Los **351 replays** son: 216 informes históricos sin perfil nuevo, 116 validaciones del conjunto mixto actual, 18 controles tipados anteriores y el control original sin el permiso nuevo. Todos son idénticos por bytes. Los 116 raws y la aceptación de fidelidad permanecen en **110/116**, con **111/116 completos estructuralmente**. La calidad de extracción autónoma original sigue en **99/116**; la utilidad histórica sigue en **31/39**, cero ajenos/60. Ninguna de esas cifras se actualiza con pruebas manuales de otro alcance.

## Recursos y alcance pendiente

El feature se activa antes de los recorridos nuevos. IDs, copias, claves de ordenamiento, comparaciones y materialización consumen el presupuesto compartido; los límites invalidan la operación, sin conservar un subconjunto favorable. Las claves de orden se calculan una vez con cargo previo. Los errores globales conservan el sobre técnico existente `documentary_request_context_limit_v1`, entrada íntegra y ninguna respuesta completa; no cuentan como negativos semánticos.

Esta primera implementación no admite qualifiers no vacíos, context_bindings, premisas o segundo claim efectivo, scopes no documentales, selectores de contexto no vacíos ni términos sin regla tipada. Los marca `not_supported`, conserva las lecturas y no los cuenta como aciertos. No certifica extracción autónoma, exhaustividad lingüística, utilidad general ni integración instalada.

En los siete procesos CLI finales se observaron **23,3–479,5 ms**, incluido arranque y escritura; son observaciones locales heterogéneas, sin comparación estadística ni medición de latencia de producto. El reporte del control pasa de **62.949 a 123.354 bytes** por el canal y la conservación explícita de originales/pruebas. Es volumen local de evaluación, no tokens remotos ni tráfico enviado a Codex. Los valores por caso se guardan en `resource-observations.json`.

La candidata de credencial preparada en paralelo sigue **parcial y semánticamente no aceptada**: conserva el posesivo sin resolver, pero no está acreditado que `owned_by` equivalga a un posesivo documental. No se usa para aumentar calidad ni se modifica su dictamen. El autor registró exposición accidental a snippets históricos y no se presenta como crítico ciego. Quedan esa entrada, QFaro, Lino y las tres fuentes con «en Nacar»; después siguen las puertas de extracción coherente completa con doble revisión, utilidad sin pérdidas/ajenos, comparación contra producto, recursos, estabilidad y V6 reservada.

## Coste y estado del objetivo

**Cero llamadas externas y USD 0 nuevos de proveedor.** Cuatro contextos Codex nuevos y cinco turnos adicionales en contextos de diseño/crítica; coste y tokens Codex desconocidos. El ledger externo conserva 6129 operaciones, 23 campañas, USD 4,565416508 conocidos y veinte importes desconocidos, excluyendo Codex.

Se ejecutaron 368 operaciones CLI locales, siete comandos Cargo de prueba/build y siete de formato, seriales. El turno anterior fue progreso y este también: cambia el mecanismo y cierra un fallo obligatorio conservando los contraejemplos. El objetivo original de cinco pasos sigue activo y no está logrado. Sin permisos pendientes, servicios, credenciales, dependencias, commit, push, modificación de predeterminados o apertura de V6. Los artefactos y fallos se conservan en `existential-conclusion-projection-v1/` y `credential-term-correction-v1/`, con instantánea 115.
