# Indeterminación normativa: representación, resolución y revisión

El paso 3 avanza con una corrección conocida aceptada: **110→111/116** representaciones completas y fieles para ambos revisores. Quedan cinco. Es una mezcla de representaciones manuales/corregidas conocidas; la extracción autónoma original sigue en **99/116**, y la utilidad histórica **31/39** no se ha vuelto a medir. No hay adopción de producto ni apertura de V6.

## Cambio implementado

El feature optativo `normative_question_indeterminacy`, noveno del perfil contextual V1, requiere `normative_action_relation`. Solo admite la forma normativa exacta declarada. La finalidad y la posición del agente subordinado ya estaban representadas; no se añade otro operador semántico. Se distingue la estructura completa de la resolución de ese agente y del ámbito normativo no explicitado.

El informe añade `representation_disposition` y `resolution_disposition`. Conserva todos los issues, con sus IDs y paths originales. `complete` representacional no demuestra fidelidad, identidad ni respuesta. La posición tipada `ambiguous_control` no se convierte en `extraction_unresolved`, `unspecified`, referencia ni owner_slot. El destino de la acción no se usa como autoridad normativa. Scope null conserva ausencia de dato y no funciona como wildcard.

El corte operacional se deriva del contenido al principio de `Engine::request`, antes de posesivo, límite relativo y legacy. Mantiene todos los canales de respuesta vacíos y conserva request/documento íntegros y deudas tipadas. La agregación entre lecturas y la conclusión existencial no eliminan ese corte. Requests y hechos independientes mantienen su comportamiento.

El constructor exige cobertura no vacía/toda representada, evidencia positiva no vacía, forma y dependencias válidas, y atribución exacta de cada issue permitido. Otros errores, extracción fallida, relación no representada, ramas, guards o formas fuera de alcance no reciben completitud favorable. El presupuesto acumulado cobra recorridos/copias y corta globalmente al agotarse; sin feature no recorre esta ruta.

## Verificación ejecutada

- **815 pruebas de workspace aprobadas**, cero fallos y 24 ignoradas. Hay quince pruebas nuevas del módulo. `cargo fmt --all --check` correcto.
- **8/8 controles CLI**: cuatro pares con documentos completos y bloqueo semántico verificado; dos controles de entradas parciales; dos rechazos técnicos de entradas inválidas. Estos últimos no se cuentan como negativos semánticos válidos.
- La proyección existencial conserva vacía la conclusión y las respuestas mientras el request esté pendiente. Los controles inspeccionan cada celda de la matriz, request/documento íntegros y ausencia de bindings/respuestas.
- **373/373 informes anteriores idénticos por bytes**, incluidos los 116 del conjunto vigente previo a la corrección, los controles nominales anteriores y las propuestas parciales de credencial/Lino/Faro sin el feature nuevo.

Los controles CLI calientes duraron **9,7–14,8 ms por proceso**, incluido el arranque. No son una medición de latencia de producto ni una comparación de recuperación.

La primera focal tuvo diez aciertos y tres fallos de preparación: endpoint requires=request no admitido por el grafo, y dos reservas de presupuesto que ignoraban el precobro del setup. Se conservaron los fallos y se corrigieron los casos. No se amplió el grafo para hacer pasar un test: request→request se verifica únicamente como rechazo técnico. La revisión estática encontró y cerró cuatro defectos de DTO/presupuesto/evidencia/procedencia; sus dictámenes y límites permanecen en el informe estático.

El primer runner CLI tuvo 7/8 porque esperaba partial donde la evidencia vacía hacía inválido el input. El criterio requerido seguía siendo no conceder completitud. Se corrigió su categoría a rechazo técnico; el segundo runner obtuvo 8/8 con los mismos ocho inputs y salidas idénticas. No se modificó el motor para ese cambio de lector.

## Corrección de Faro y doble revisión

Se conserva la pregunta exacta y la candidata anterior parcial. La nueva versión añade el opt-in y declara representado el verbo ya presente en su constructor; conserva agente ambiguo y curso como destino, con ámbito normativo null. Cambiar la cobertura por sí solo no habría pasado la validación ni la revisión.

Dos contextos nuevos recibieron únicamente pregunta, raw, vocabulario y guía semántica, sin fuente contraparte, código, pruebas, etiquetas, resultados o revisión ajena. Ambos emitieron **4/4 faithful**, ocho dimensiones en total. La auditoría verificó hashes del paquete, citas UTF-8 y pointers. Son revisiones de la misma familia, no humanas ni validación independiente. El código fuente de diseño y sus críticas no sustituyen esa revisión.

La nueva fila valida completa como representación, **con los dos issues originales conservados y resolución blocked**. El conjunto mixto queda en **112 estructuralmente completas, 111 fieles para ambos y 111 completas/fieles de 116**. La puerta global sigue fallida; no se reemplazó ninguna otra fila ni se cambiaron dictámenes históricos.

## Otros pendientes investigados

El diseño de «en Nacar» V1 fue rechazado por perder la restricción positiva al tratarla como metadata. V2 mejora composición y ownership, pero su crítica mantiene problemas de ancla de congelación, delta de vocabulario, host base y contención. No se implementó esa envoltura.

Tres autorías parciales corrigen por separado el host base: Cobre funciona efectivamente durante ochenta minutos, no una asignación; Zinc se apaga, sin convertir cien minutos en operación continua; la reserva conserva acción/beneficiario. El evaluador mantiene las tres parciales y las deudas sin/en/nueva/hasta/tras explícitas. Esas propuestas no tienen aceptación doble ni se incorporan al conjunto aceptado. Preservar bytes del raw histórico no obliga a conservar una interpretación semántica errónea.

Siguen pendientes Lino, la credencial y los tres textos de Nacar. Antes de utilidad, comparación con producto, recursos, estabilidad y casos nuevos ciegos debe cerrarse la fidelidad de los 116 con un vocabulario/guía coherentes. No se declara mejora de recuperación a partir de esta corrección.

## Coste y trazabilidad

**Cero llamadas externas nuevas y USD 0 de API incremental**. El consumo de agentes Codex no está cuantificado. La contabilidad externa histórica permanece en 6129 operaciones, USD 4,565416508 conocidos y veinte operaciones de coste desconocido.

Artefactos: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/normative-indeterminacy-contract-v1/`, junto con las autorías y críticas identificadas. Se conservaron inputs previos, fallos y revisiones. Sin nuevos servicios, dependencias, credenciales, cambios de DB, rama, commit/push ni configuración predeterminada.
