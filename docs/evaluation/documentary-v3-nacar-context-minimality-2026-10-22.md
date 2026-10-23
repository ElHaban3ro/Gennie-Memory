# Carrier mínimo para «en Nacar»

Fecha: 2026-10-22. Estado: análisis documental de diseño, sin implementación, extracción nueva, validación ejecutada ni recalificación.

## Dictamen

Sí: **«en» puede conservarse como adjunto relacional de categoría semántica no especificada**, con propietario, referente ancla y evidencia obligatorios. Es suficiente para conservar esa relación preposicional estrecha cuando su adjunción está determinada. No basta almacenar el token, reconocer el nombre o añadir una key al registro: el consumidor debe conservar la restricción y rechazar cualquier derivación que requiera decidir lugar, modo o ajuste. Si el propietario es ambiguo, el carrier no lo resuelve; necesita lecturas de adjunción justificadas, o un inventario abierto que siga bloqueando completitud.

La opción de menor cambio de forma es una key nueva de `Configuration`, provisionalmente `prepositional_en_context`, con valor exclusivamente `ref`. Su semántica puede ser neutral aunque el contenedor se llame configuration, siempre que la nueva definición niegue expresamente que denote un ajuste. Es una ampliación del contrato semántico, no un DTO nuevo ni un alias de `context`. Sin embargo, **no está demostrado que esa sola key pueda cubrir los tres documentos completos**: una hoja cuyo propietario sea el beneficiario nominal, o el intervalo en lugar del apagado, necesita un sitio de adjunción de ese tipo. No debe simularse poniendo el adjunto en el claim completo.

No se autoriza implementación aquí. La alternativa más amplia es una relación neutral de contexto nominal preposicional con propietarios tipados. Su coste adicional solo se justifica si los sitios existentes no permiten adjunción local sin cambiar significado. Ninguna opción concede `complete` por pertenencia al vocabulario.

## Alcance y base

Se leyeron `AGENTS.md`, `docs/rules/testing.md`, `docs/rules/backend.md`, los índices de memoria y decisiones del proyecto y [la auditoría de incertidumbre](documentary-v3-uncertainty-gate-audit-2026-10-22.md). La evidencia específica se limita a estos tres archivos bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-author-corrections-v1/known116-bundle/`:

- `d-38fb24e622a073a293ead090-input.json`.
- `d-b7123caa690427c7c04d4c74-input.json`.
- `d-5d8e6d45d1ee52311cdfb8b4-input.json`.

Se inspeccionaron su texto, representación y vocabulario propios y únicamente las tres entradas correspondientes de `consolidated-failure-diagnosis-v1/source-diagnosis.json`. Sus dictámenes son diagnósticos retrospectivos conservados, no crítica nueva independiente. No se inspeccionaron preguntas, contrapartes, gold, utility, V6, código ni otros raws. No se ejecutaron Cargo, binarios o APIs. El único archivo escrito es este informe.

La auditoría general fija la distinción central: honestidad sobre una pérdida no es preservación. El contrato histórico y la puerta `116 complete + faithful` siguen sin cambio; este documento tampoco demuestra que un perfil futuro pase esa puerta.

## Qué está explícito y qué se perdió

Los offsets siguientes son bytes UTF-8, intervalos semiabiertos, de las fuentes propias.

| Fuente y texto | Núcleo explícito | Complemento y estado observado |
| --- | --- | --- |
| `d-38fb24e622a073a293ead090`: «Cobre funcionó ochenta minutos en Nacar sin apagarse; no se midió su duración hasta apagarse.» | Operación, ochenta minutos y ausencia de apagado durante ella; una segunda cláusula de medición negada que debe conservarse separada. | `en Nacar` `[32,40)`, ancla `Nacar` `[35,40)`. `c1` tiene `run_duration` y `no_shutdown`; `nacar` existe, sin enlace al claim. La cobertura del complemento es `unresolved`. |
| `d-b7123caa690427c7c04d4c74`: «Se reservó una batería nueva para Cobre en Nacar.» | Reserva ocurrida, batería y beneficiario; no identifica actor. | `en Nacar` `[42,50)`, ancla `[45,50)`. `c1` contiene `action_beneficiary`, con `battery` y `cobre`, sin qualifiers; `nacar` carece de enlace. El diagnóstico conserva ambigüedad entre reserva y beneficiario, además de categoría abierta. |
| `d-5d8e6d45d1ee52311cdfb8b4`: «El reproductor Zinc se apagó tras cien minutos en Nacar.» | Apagado e intervalo de cien minutos; no afirma medición o autonomía máxima. | `en Nacar` `[48,56)`, ancla `[51,56)`. `shutdown` lleva `shutdown_duration`, sin complemento relacional. El diagnóstico deja por revisar adjunción al apagado o al intervalo. |

En las tres representaciones `nacar.semantic_type` es null y sus `links` están vacíos. Su existencia y dependencia del scope `root` preservan una mención, no la relación. Es una pérdida de extracción/representación concreta, incluso si la categoría de Nacar es genuinamente indeterminada en la fuente. La ausencia de arista no se vuelve fiel cambiando su motivo a `source_unspecified`.

Los diagnósticos citan definiciones distintas: `context` requiere ajuste/configuración expresamente nombrado y `generic_location` localización explícita. Ninguna puede elegirse por capitalización o por la preposición sola. El `configuration_request_profile` enumera keys permitidas; la enumeración no contiene una relación preposicional neutral ni demuestra que un valor satisfaga sus guards. `associated_with`, descrito para genitivo nominal de relación no especificada, tampoco autoriza extender sin crítica su alcance a estos adjuntos de ocurrencia.

## Dos opciones y su coste mínimo

### A. Key nueva sobre `Configuration`, sin DTO nuevo

Una versión prospectiva podría fijar exactamente una key `prepositional_en_context`, sin texto de significado libre. El valor sería un `ref` al referente nominal local y explícito. La key codifica únicamente el enlace preposicional «en» y su categoría no especificada. No contiene un campo `meaning`, ni un string con la cláusula, ni un enum abierto de relaciones posibles. La evidencia del qualifier debe incluir el complemento; el referente ancla debe tener su propia evidencia nominal. `resolution: known`, si se reutiliza, significaría enlace y ancla presentes, nunca categoría conocida o ambigüedad cerrada.

El owner se obtiene del sitio tipado de adjunción y se verifica contra una dependencia que no puede crear por sí sola esa adjunción. Para la hoja de operación de Cobre sería `c1.body.value.qualifier_ids → nuevo qualifier → ref nacar`; para la hoja de apagado de Zinc sería `shutdown.body.value.qualifier_ids → nuevo qualifier → ref nacar`. Se necesita además la dependencia del owner al qualifier y de este al ancla, con scope válido y evidencia propia. El nombre de la key no cambia otros calificadores ni transfiere el complemento a `c2`, la medición negada.

Su coste mínimo es una fila semántica versionada, guard de valor `ref`/ancla nominal, reglas de owner y evidencia, validación de dependencias, consumo conservador y crítica del nuevo sentido neutral. No requiere necesariamente cambiar la estructura serializada del qualifier. **No significa cero código futuro**: el verificador y consumidor deben reconocer y hacer efectiva esa semántica; este informe no inspeccionó su capacidad actual ni propone un parche.

El límite decisivo es el sitio de adjunción. En la reserva, poner el qualifier en la proposición conserva una hoja «reserva en Nacar», pero no una hoja «Cobre en Nacar». En Zinc, ponerlo en `shutdown` no equivale a ponerlo en `shutdown_duration`. No se acreditó con estos inputs un sitio existente que permita ambas adjunciones locales de forma válida. Si requiere ampliar el conjunto de owners o añadir adjunción a un qualifier, ese coste se cuenta aparte. No se inventa una propiedad de Cobre ni otro evento para obtener un owner cómodo.

### B. Relación neutral de contexto nominal preposicional

Un carrier separado podría fijar campos cerrados para `owner`, `anchor`, relación `en`, categoría `unspecified` y evidencia. Los únicos tipos de owner a estudiar aquí serían una proposición de ocurrencia, una ocurrencia nominal local en un argumento y un intervalo explícito ya representado. Un owner nominal debe identificar su uso local —por ejemplo, el argumento `beneficiary` de `c1`—; apuntar solo al ID global de Cobre permitiría propagar indebidamente el contexto a otras ocurrencias. El intervalo debe apuntar al intervalo concreto, sin crear otra operación.

Esta forma expresa mejor el ámbito común, pero cuesta un carrier o extensión de aristas, owners tipados, validación y reglas de consumo nuevas. No sirve llamarlo `nominal_context` si se interpreta como propiedad nominal permanente: lo nominal es el complemento anclado, no necesariamente el owner. Tampoco debe admitir cláusulas, predicados libres, owners arbitrarios o preposiciones no definidas.

**Elección mínima:** estudiar primero A para las hojas de owner de ocurrencia ya disponibles. Adoptar B solo si se exige cubrir las hojas de owner nominal/intervalo y no existe un sitio local equivalente. La neutralidad semántica no obliga por sí sola a un DTO nuevo; la necesidad de distinguir owners sí puede obligar a ampliar el contrato. Con el material autorizado no se demuestra equivalencia ni cobertura total de A; se deja esa frontera explícita.

## Categoría abierta y adjunción abierta no son lo mismo

`category_unspecified` conserva el significado común de un enlace «en» sin enumerar lugar/modo/ajuste como si fueran lecturas exhaustivas. No afirma que esas categorías sean las únicas posibles; tampoco afirma que todas sean aceptables. No se requieren tres hojas categoriales fabricadas para conservar ese mínimo. Una futura crítica debe establecer que el carrier preserva exactamente lo que la fuente deja abierto, y no es una etiqueta para saltarse una relación más específica expresada.

La ambigüedad de adjunción cambia el owner y sí requiere tratamiento separado:

- En Cobre, el diagnóstico restringe el complemento a la operación. Ese owner permite estudiar una hoja neutral; no permite afirmar que el documento entero esté completo, porque persiste la cuestión independiente de «hasta apagarse» en la medición negada.
- En la reserva, las hojas candidatas de reserva y beneficiario deben conservar cada una batería, beneficiario, reserva y complemento con su owner correcto. El descriptor «nueva» constituye otra dimensión; el diagnóstico propone una vía local existente, sin probar su aceptación. Añadir el adjunto no subsana esa pérdida.
- En Zinc, las hojas candidatas de apagado e intervalo deben conservar íntegros apagado e intervalo. No se declara de antemano que esas dos hojas agoten el texto. Una hoja que omita el contexto del intervalo o lo copie en el apagado no es completa por compartir el resto de la estructura.

Cada conjunto de lecturas solo puede cerrarse si la crítica demuestra que su inventario es exhaustivo y cada hoja está completa respecto de su interpretación. Si no, sigue `open`; no se usa la intersección de las hojas enumeradas para conceder completitud. No se crean dos hojas automáticamente porque un diagnóstico mencione dos candidatos. Una sola hoja neutral completa en categoría tampoco cierra una adjunción no resuelta.

## Efecto operativo exigido y prohibición de fallback

El carrier debe ser una restricción efectiva sobre el owner local y un requisito de cualquier derivación que consuma ese contexto. No se concede compatibilidad a partir de que dos referencias se llamen Nacar: se conservan namespace, scope, referente local, predicado/rol del owner y evidencias. No se habilitan inversa, transitividad, equivalencia con lugar o modo, ni proyección de contexto a otros hechos del mismo sujeto.

Requisitos prospectivos, sin ejecución en este informe:

1. **Retirar solo el enlace:** manteniendo cita y `nacar`, debe perderse preservación de ese complemento; cobertura o extracción no pueden declararse completas. El consumidor no lo reconstruye leyendo la cita ni lo ignora para favorecer una derivación.
2. **Cambiar solo el ancla:** una entrada de control coherente en texto/evidencia, con otro ancla, debe producir una restricción distinta y bloquear consumo que requiera el contexto anterior. Un cambio de ID que conserve el mismo grafo debe ser invariante; un cambio de referente no.
3. **Cambiar owner:** mover el adjunto de reserva a beneficiario, o de apagado a intervalo, debe cambiar la firma y las dependencias consumidas. Si no cambia nada, el carrier es decorativo.
4. **Exigir categoría:** el enlace neutral no satisface un requisito espacial, de modo o ajuste. No se convierte a `generic_location`, `context`, `mode`, `profile` o `program`; ni el fallo permite recurrir al nombre, al texto completo o a una vía sin qualifier.
5. **Eliminar evidencia, ancla o sitio:** debe rechazarse estructura incompleta; no fabricar un owner ni tratar null como comodín. Quitar una hoja o mantener una deuda abierta bloquea cierre favorable.
6. **Preservar las demás dimensiones:** no transformar los ochenta minutos sin apagado en tiempo hasta apagado; no convertir reserva en uso de batería; no convertir cien minutos hasta apagado en una medición o máximo. La nueva arista no subsana automáticamente ninguna otra dimensión.

La preservación del enlace tiene así efecto verificable aunque no resuelva categoría: retiene alcance y restringe consumo. Si el motor solo conoce igualdad de nombres y acepta el mismo resultado al remover el carrier, no hay preservación operativa demostrada. Aun pasando esos controles, hace falta crítica semántica del texto; un mecanismo puede propagar con rigor una adjunción equivocada.

## Qué permanece sin soporte

Hoy no está incorporado ninguno de los dos contratos neutrales. No se declara nuevo estado completo para estas fuentes. Permanecen sin demostración la exhaustividad de las adjunciones candidatas, el sitio válido para owner nominal e intervalo bajo la opción A, la crítica de «nueva» y del alcance de «hasta apagarse», y toda validación de consumo. La neutralidad no completa identidad de Nacar, categoría ontológica, ubicación física, configuración, modo, actor de reserva, realización de uso, medición ni autonomía máxima.

El resultado de esta auditoría es un delta acotado: **enlace preposicional neutral + ancla explícita + owner local exacto + evidencia y dependencias obligatorias + consumo sin fallback**. Una key cerrada puede ser el transporte más barato de ese delta donde ya haya owner compatible. Ningún flag de pertenencia o incertidumbre reemplaza el enlace, decide el owner o convierte el documento en `complete`.
