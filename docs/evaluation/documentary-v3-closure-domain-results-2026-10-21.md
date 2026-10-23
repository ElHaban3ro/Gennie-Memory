# Dominio explícito de restricciones cerradas V3

Se implementó en el evaluador un dominio opcional `answer_domain` para distinguir «la única cámara que posee» de «lo único que posee». Una restricción de cámaras no permite excluir cartuchos ni cerrar una consulta sobre todos los objetos. La pertenencia exige clase léxica o tipo semántico declarado en la pregunta; ausencia o diferencia de clase no demuestra incompatibilidad. No se incorporó al producto.

## Contrato y comportamiento

`all` declara alcance irrestricto; `lexical` conserva una cita de clase; `semantic` conserva un tipo registrado y sus citas. El perfil optativo `restriction_domain_profile.version = explicit_answer_domains_v1` exige dominio explícito en cada `value_restriction`. Su ausencia o `null` deja ese claim pendiente y conserva hechos independientes. Solo los datos históricos sin perfil ni campo conservan el cierre irrestricto anterior. Esto no acredita fidelidad de sus extracciones.

Antes de excluir un valor o completar un conjunto se exige pertenencia al dominio o coincidencia con el filtro de clase solicitado. Las cautelas de admisión también exigen pertenencia de la configuración deseada; una etiqueta textual aislada no prueba una clase. Los testigos conservan el dominio, y las restricciones transportadas como contexto conservan el cuerpo completo. Una restricción con dominio desconocido no se publica como contexto. Los dominios de clase de esta versión requieren roles declarados `ref`; otros tipos quedan pendientes. No se infiere disjunción entre clases, jerarquías ni equivalencia léxica/semántica.

## Evidencia local

| Comprobación | Resultado |
| --- | --- |
| Controles nuevos, escritos manualmente antes de ejecutar | 13/13 |
| Variantes de cita, esquema, tipo, perfil y rol no soportado | 5/5 |
| Pruebas workspace | 613 pasan, 24 omitidas |
| Pruebas V3 incluidas en workspace | 91 motor + 2 adaptador |
| Pares manuales históricos con salida idéntica | 100/100 |
| Validaciones congeladas con salida idéntica | 116/116 |
| Controles anteriores con salida idéntica | 71/71 |
| Formato y compilación del evaluador | pasan |

Los controles cubren exclusión dentro del dominio, clase no acreditada, consulta sin clase, conjuntos acotados/no acotados, restricción negativa, dominio semántico, alcance irrestricto explícito y compatibilidad histórica. Dominio ausente y `null` preservan por separado un segundo hecho sobre masa. Son pruebas del operador sobre representaciones manuales, no validación de un extractor.

Se preservó un intento fallido de comparación: el control `qualified` difería del informe de una fase más antigua, pero coincidía con el binario inmediatamente anterior. Los nueve controles de esa familia se compararon con ese binario; los otros 62 con sus informes originales. No se reescribieron resultados previos. El informe JSON identifica cada referencia de comparación.

## Calidad y coste

Las salidas manuales conservan 36/39 referencias útiles no ambiguas seleccionadas, tres respuestas completas y cero selecciones en las 60 referencias heredadas `none`. La referencia ambigua sigue separada. Estos números no son precisión automática ni una medición nueva de mejora. Siguen pendientes las tres omisiones conocidas y la fidelidad completa.

La última extracción congelada conserva 112/116 entradas estructuralmente completas y 65/116 fieles para ambos críticos. No se ejecutó extracción nueva, revisión nueva, utilidad sobre raws defectuosos, comparación de producción ni V6. El requisito previo de 14/39 pares útiles con ambos lados aceptados tampoco es recall ni un límite superior.

Cero peticiones API y USD 0 nuevos de proveedor. El coste y los tokens de Codex son desconocidos. El acumulado histórico sigue en 6129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos y 20 operaciones con coste desconocido. Los 13 procesos CLI nuevos tardaron aproximadamente 13,19 ms de mediana y 15,35 ms de p95; una ejecución por control, incluyendo arranque y disco. No es latencia del extractor, del proveedor ni de producción, ni una comparación de rendimiento.

El objetivo continúa en el paso 3. La siguiente etapa debe ligar una guía prospectiva al nuevo perfil y resolver citas, uso neutral, genitivos débiles y finalidad operativa; después, extracción y revisión del banco completo antes de medir utilidad automática. No se promueve el mecanismo ni se consume la cohorte ciega.

Artefactos: `outputs/iromem/evaluation/documentary-v3-closure-domain-2026-10-21/`. Los raws, críticas, etiquetas y cohortes previas se conservan.
