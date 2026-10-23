# Descriptores completos en roles de término

El perfil optativo `described_reference_term_roles_v1` requiere `local_reference_descriptions_v1` y admite descriptores completos en argumentos declarados `term` y miembros de restricciones cuyo rol sea `term`. No amplía símbolos, cantidades ni otros portadores; conserva restricciones, dependencias y deuda de identidad. Sin perfil se mantienen los resultados anteriores.

Pasan 688 pruebas del workspace, con 24 omitidas, seis específicas del perfil, formato y compilación. Los 226 informes históricos se reproducen idénticos. Se conservaron dos errores iniciales de preparación: un target incompleto y una expectativa que confundía selección parcial con respuesta completa. Un proveedor distinto conserva, cuando corresponde, un parcial sin apoyo ni respuesta completa.

La entrada real reveló además un error estructural independiente: el autor omitió el argumento `value` obligatorio en el átomo objetivo de una restricción. Se conserva su rechazo; el perfil no convierte ese átomo en un patrón ni relaja sus campos obligatorios. La corrección de esa entrega y su doble revisión se registran por separado en la campaña.

Evidencia: `outputs/iromem/evaluation/documentary-v3-described-term-roles-2026-10-22/verification.json`, reproducción histórica y logs completos en el mismo directorio. Binario SHA256 `bb0750dda92a6235f7ee6962b1305a6f3685df02162eb91ce0565c53a35f1f12`. Cero nuevas llamadas externas y USD 0 en ese ledger; coste Codex desconocido. Es una capacidad de evaluación, sin adopción en producto ni demostración de calidad autónoma.
