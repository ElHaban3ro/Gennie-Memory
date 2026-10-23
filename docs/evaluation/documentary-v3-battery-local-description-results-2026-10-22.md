# Reserva y descripción nominal local: corrección conocida

Estado: progreso del paso3, objetivo completo activo. Una corrección manual de fuente supera forma y doble crítica: banco mixto **112→113/116 completas y fieles para ambos críticos**. Quedan Lino, credencial y Cobre. No se ejecutó otra extracción autónoma ni se midió recuperación frente a la base.

## Cambio demostrado

Fuente: «Se reservó una batería nueva para Cobre en Nacar.» La reserva conserva una sola correlación entre batería, acción y beneficiario. «Nueva» se representa como restricción tipada local del argumento batería dentro de `described_ref`, usando el predicado existente `property` y un atributo genérico añadido al vocabulario, `nominal_qualitative_descriptor`. La proposición no se publica como hecho global de batería ni de una clase. No se infieren carga, uso, instalación, fabricación o entrega.

El feature léxico ya implementado conserva «en Nacar» como condición positiva de ese Claim completo, con complemento íntegro y contexto original fijo. No escoge lugar, modo o una adjunción interior. La clase léxica opaca «batería nueva» se descartó como solución composicional; la clase se mantiene «batería» y el adjetivo está separado en la restricción del mismo argumento.

La candidata del autor conservaba coverage unresolved para «en Nacar»: validó partial. El coordinador produjo otra versión que cambia únicamente esa fila a represented, porque el carrier tipado ya conserva la construcción. Esto no acredita su fidelidad; la versión nueva completa recibió las revisiones separadas posteriores. La candidata del autor, su historia parcial y el original conocido permanecen intactos.

## Evidencia y alcance

Dos contextos nuevos recibieron solo texto, raw, vocabulario y semántica declarada, sin preguntas, gold, resultados ni crítica ajena. Ambos emitieron faithful en text_fidelity, completeness, attribution y operative_preservation: **8/8 juicios**. La auditoría verificó ocho citas UTF-8 con offsets y46punteros. SHA del paquete: `fea22a33a585ecfa272e9fd9aca9e8a03c6dc90429dba22d1f5bb360d01ad7b0`.

Es una corrección manual con revisión por la misma familia Codex en contextos separados; no se presenta como extracción ciega, validación humana o independencia de familia. La exposición del autor a interfaces e instrucciones históricas figura en su recibo. La versión definitiva de coverage es un ajuste del coordinador con contexto previo.

La validación de la candidata definitiva devolvió complete sin issues. Dos ejecuciones reales del selector existente comprobaron:

1. Localizador del beneficiario Cobre: selected, documento y Claim íntegros, con described_ref, restricción «nueva» y condición «en Nacar» conservados; sin términos/proposiciones de respuesta y sin respuesta completa.
2. Localizador del beneficiario Nacar: no_match. Compartir documento no sustituye el beneficiario.

El derivado seleccionado es exactamente la versión revisada por ambos críticos. El runner construyó el manifiesto a partir de esos dictámenes, no de un accepted aportado por el autor. La base técnica se obtuvo retirando únicamente el carrier y su habilitación para verificar la inversión: es una base semánticamente incompleta, **no una fuente aceptada ni evidencia desnuda utilizable**. Solo el derivado tiene fidelidad revisada. Esa procedencia y la conversión de dictámenes al registro de revisión figuran en `consumption-controls/provenance.json`; los hashes no certifican verdad.

| Banco conocido completo | Antes | Ahora |
| --- | ---: | ---: |
| Estructuralmente completas | 113/116 | 114/116 |
| Fieles para ambos críticos | 112/116 | 113/116 |
| Completas y fieles para ambos | 112/116 | 113/116 |
| Pendientes de la puerta conjunta | 4 | 3 |

Las otras115filas conservan sus inputs, validaciones y dictámenes por hash, con el mismo binario evaluador. No se repitieron Cargo o las384reproducciones: las826pruebas,24ignoradas y384reproducciones del punto118 siguen siendo evidencia anterior vinculada al mismo binario y fuentes del evaluador, no una ejecución nueva. El binario sigue SHA256 `37fefa56d07099d5b372bf7e95b95df0dbadac89421905b96a9448472587b9e5`. No hubo cambio Rust, dependencia, configuración de producto, DB de usuario, servicio, rama o entrega Git.

La extracción autónoma original permanece99/116; la utilidad histórica sigue31/39 y cero ajenos/60. Los dos controles locales no se suman a esa utilidad ni prueban recuperación entre documentos. No se midió latencia, RSS o ahorro de tokens de producción en esta tanda.

## Qué cambia la siguiente acción

La investigación primaria sobre posesivos y el mismo crítico B redujeron la objeción de κ libre: κ(U,c0) fijo puede conservar la dependencia sin resolver el antecedente. Su dictamen integral sigue uncertain; no se sustituye por un muestreo favorable ni suma un pase. [Nota de evidencia lingüística y límites](documentary-v3-possessive-linguistic-evidence-2026-10-22.md). La RAE describe el fenómeno general, no adjudica la oración del banco. [Rasgos distintivos de los posesivos](https://www.rae.es/gram%C3%A1tica-b%C3%A1sica/demostrativos-y-posesivos/los-posesivos/rasgos-distintivos).

Para las dos cláusulas de medición con «hasta», quedó un [diseño tipado](documentary-v3-until-measurement-design-2026-10-22.md) y su [crítica reparada](documentary-v3-until-measurement-design-critique-2026-10-22.md). La composición no elimina el operador para publicar negación global, no afirma apagado ni medición posterior y mantiene adjunción/sujeto abiertos. La voz `nonagentive_se` evita decidir impersonal frente a pasiva refleja. El Body nuevo está justificado por inseparabilidad de alcance, pero **no está implementado ni verificado**. La regla positiva propuesta usa el V2 existente para contexto parcial de duración con propietario explícito y Same acreditado, sin inventar cifra ni nueva CLI. Este contrato es la siguiente implementación; no se cuenta como cierre de Lino o Cobre.

## Coste y continuidad

Esta tanda usó cinco contextos nativos nuevos y tres followups; coste Codex no cuantificado. Hubo cuatro ejecuciones CLI breves, cero procesos Cargo y cero de formato. Se consultaron fuentes lingüísticas primarias mediante web; **cero llamadas a proveedores de modelos y USD0 de API de modelos**. El acumulado externo histórico permanece6129operaciones/23campañas, USD4,565416508 conocidos más20 costes desconocidos, sin Codex.

El objetivo conserva sus cinco pasos y todas sus puertas: cerrar representaciones fieles, guía/vocabulario compartidos y extracción116 completa con doble revisión; preservar utilidad y rechazar ajenos; después comparar base, recursos/estabilidad y casos nuevos ciegos antes de adopción. V6 no se abrió. El permiso no bloquea ninguna de estas tareas.

Artefactos: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/battery-reserve-source-correction-v1/` y `battery-reserve-source-correction-v2/`, con versiones, recibos, paquetes, críticas, auditoría, validaciones, controles y banco113/116por fila.
