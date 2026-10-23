# Minería P2: ampliación prospectiva A2, reglas intactas

Fecha:2026-10-22. Este protocolo complementa [P1](utility-hard-mining-v1-protocol-2026-10-22.md), conservado íntegro. P1 terminó192scores,163correctos,69/96útiles,2ajenos;grupos con error útilA0/B12/C12. Su precondición A≥4/C≥4 falló y **no se entrenó**. La falta de dificultad en A no prueba dominio general de A ni causa estilística.

## Cambio acotado anterior a nuevos scores

Añadir64pares A2 independientes,16grupos nuevos2A+2none,con preguntas y notas narrativas sin vocabulario de fichas/campos/casillas. Mantener claridad del mismo atributo/referente,ausencia explícita y controlesdeotroatributo/objeto. Autor no ve scores ni observados,sondas o reservas; no se le exige producir fallos. Revisión ciega de64labels/roles antes de su primer score. Originales192y sus labels/revisiones/resultados no cambian. PoolP2=256pares:32gruposA,16B,16C;128útiles/128none. Los192previos ya tienen scores observados como TRAIN; no presentarP2como conjunto fresco.

Se repuntúan todos256 con el MISMO padre exacto. Los192previos deben conservar decisiones y diferencias de score≤0,02contraP1; si no,fallo técnico,sin reintento. Solo64son entradas nuevas del modelo. Misma política hard(errores útiles,errores totales,pérdidaV2,ID),cuota4gruposporA/B/C,**mínimo4gruposconerrorútilA y4C**. Si falla,no entrenar. No cambiar umbral,label,cuotas o soporte según resultados. B sin mínimo de error útil pero cuota4 igual.

Uniform: misma política y semillaPythonrandom20261022,muestreo4sinreemplazo desdeIDsordenados sucesivamenteA/B/C. Como A pasa de16a32grupos,susIDs y el consumo de RNG pueden cambiar también la selecciónB/C. Congelar los IDs nuevos ANTESdelprimer scoreP2 y no afirmar que son los mismosIDs deP1. No rerolls ni retirar solapamiento.

## Continuidad y presupuesto condicionados

Sin cambios de entrenamiento: padreV5matched7522394b...,fondo176+48seleccionados=224,8A+8B+8C+24noneadicionales;unaépoca/224backward/56updatesporbrazo,freshAdamW ylossV2 idénticos,orden uniform→hard,mismo orden224índices yroles/targets porposición,fuente congelada antes deP2. Guard de soporte=true/selectionSHA/miningplanSHAantesde journal. No entrenar material observado fuera del poolTRAINnireserva.

Evaluación antes/final cambia solo porque pool ahora256:560pares=fondo176+pool256+desarrollo24+conocidos64+sondas8/32.2240scores_eval+448trainforward/backward=2688forward/112updates si se ejecuta campaña;más256forward demineríaP2. Mining yfit siguen sin validar generalización. Puerta original24/24+64/64,cero pérdidas/ajenos/degradaciones y≥1ganancia intacta; comparación padre contemporáneo/uniform/hard,inmutabilidad fuente/weights,checkpoint antes final,revisiones/auditoría ylímites deP1se aplican.

CLIminería sinpuerto/API:estimado1min,watchdog600s;despuésentrenamientos4–5minestimados,1200scadauno,solo si soporte. Registrar handles/PID/terminal reales,sin reinicios por timeout. APIUSD0,local/Codexeconómico desconocidos; no servicios/descargas/dependencias/DB/secretos/ramas/commits/producto. Sin adopción o reservas por puntuación de pool; pasos4/5siguen tras puertas correspondientes. P1fallido no se cuenta como aprobado por esta ampliación.
