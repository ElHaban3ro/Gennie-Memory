# Comparación remota en peticiones completas sin coincidencia FTS5

Estado: **ejecutada el 2026-10-15**, con autorización del usuario. Se usaron las [400 peticiones sintéticas completas](../v5-natural-gap/README.md) ya congeladas: 200 positivas y 200 negativas; 200 casos en desarrollo y 200 en calibración, con 50 entidades distintas por partición. Las 40 frases en inglés se reutilizan entre entidades y se redactaron para eliminar coincidencias FTS5. La búsqueda híbrida había incluido el recuerdo esperado entre dos candidatos en los 200 positivos. Ambos proveedores recibieron los **mismos candidatos y preguntas por caso**. No se modificó la instrucción entre particiones ni se abrió final.

Los [planes de desarrollo](development-plan.json) y [calibración](calibration-plan.json) fijaron 400 llamadas cada uno, US$0,20 y 1.200 segundos activos por partición. Se completaron las 800 tareas remotas: un HTTP 504 de OpenAI en desarrollo quedó como fallo terminal, sin reintento, y no hubo llamadas inciertas. El coste informado o reservado de ambas particiones sumó **US$0,01824** (Jev US$0,00813; OpenAI US$0,01011), por debajo de los topes. Se recompusieron las 200 semillas por modelo y partición contra la misma base y los vectores congelados.

| Método | Desarrollo: positivos / 100 | Desarrollo: negativos / 100 | Calibración: positivos / 100 | Calibración: negativos / 100 | Total exacto / 400 |
| --- | ---: | ---: | ---: | ---: | ---: |
| FTS5 top‑1 local | 0 | 100 | 0 | 100 | 200 |
| Híbrida top‑1 local | 52 | 0 | 52 | 0 | 104 |
| Híbrida Laya choice local | 2 | 94 | 0 | 94 | 190 |
| Híbrida Laya binario condicionado local | 50 | 0 | 49 | 0 | 99 |
| **Híbrida + Jev 1.13** | **73** | **100** | **74** | **100** | **347** |
| Híbrida + OpenAI GPT‑6 Luna Decisions | 40 | 99 | 29 | 100 | 268 |

Jev conservó hecho, condición y fuente declarada en sus 147 positivos acertados; OpenAI también en sus 69 aciertos. No hubo IDs prohibidos. El único 504 afectó un negativo de modalidad sin decidir y se contabilizó como error. En desarrollo, la tabla emparejada Jev/OpenAI da 138 aciertos de ambos, 35 solo de Jev, 1 solo de OpenAI y 26 errores de ambos; en calibración, 128, 46, 1 y 25. La p50 de **decisión remota en llamadas completas** fue 470,99/473,71 ms para Jev y 481,85/485,32 ms para OpenAI (desarrollo/calibración). Estas latencias excluyen búsqueda, composición y fallos; no representan el tiempo integrado de un host.

Por segmento positivo, Jev acertó 38/50 y 40/50 peticiones remotas, y 35/50 y 34/50 presenciales (desarrollo/calibración). OpenAI acertó 21/50 y 19/50 remotas, y 19/50 y 10/50 presenciales. Ambos rechazaron casi todos los negativos. Esto muestra valor de la búsqueda semántica y una política de Jev más equilibrada en estas plantillas, pero **147/200 positivos** aún dejan demasiadas omisiones para concluir viabilidad general. El 100 % positivo del [corpus principal](../v5-campaign-remote/README.md) no se traslada intacto a paráfrasis sin coincidencia textual. Tampoco son preguntas humanas ni etiquetas revisadas por una persona.

## Evidencia

- [Resumen de desarrollo](development-summary.json) y [calibración](calibration-summary.json): huellas del plan, métricas por segmento y comparación emparejada con FTS5, top‑1 y dos variantes Laya. Los planes internos tienen SHA-256 2a6d33876d1ef1405d2fc9163e05b32fd753e1649fa16b9af1c00d83a089180a y 833e329166f6843d1547dfc09f58e87e8ee5abab5a4abca3e9ca1ada22752e48, respectivamente.
- Decisiones completas: [desarrollo](development-decisions.json) y [calibración](calibration-decisions.json). Contienen IDs, probabilidades, estados, modelo servido, latencia y coste informado, sin clave.
- Semillas por caso: [desarrollo Jev](development-jev-seed-results.jsonl), [desarrollo OpenAI](development-openai-seed-results.jsonl), [calibración Jev](calibration-jev-seed-results.jsonl) y [calibración OpenAI](calibration-openai-seed-results.jsonl). Los cuatro archivos de evaluación agregada del mismo directorio verifican fidelidad y errores.
- Los diarios reanudables permanecen ignorados por Git en .local/pilots/v5-natural-gap-remote-development-journal.jsonl y .local/pilots/v5-natural-gap-remote-calibration-journal.jsonl; SHA-256 7da788e9b66e2d1909051154b219eb5ef67ff628d904642ad97c01465e812769 y ef4747ba685c7cd36c21e781017d09cf5be5b2bc4e3156b6d36074821ef77277.
- El resumidor histórico verificó todos los pares caso/modelo, decisiones, candidatos, semillas y etiquetas antes de escribir cada resumen. Su script fue retirado.

Las etiquetas esperadas se usaron solo para evaluación local; la petición remota incluyó pregunta, ámbito y síntesis candidatas, además de «none». La fuente es sintética y declarada, no verificada externamente. Desarrollo y calibración tienen entidades disjuntas pero las mismas familias de frases; estos resultados no estiman generalización a tráfico real. No se eligió umbral o política de producción ni se ejecutó la partición final.
