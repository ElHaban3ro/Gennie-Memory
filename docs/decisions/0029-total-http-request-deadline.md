# 0029 — Plazo total por petición HTTP

Estado: implementado y probado localmente; comportamiento del proveedor pendiente de nueva ejecución prospectiva.

El contraste Qwen9B OFF/ON inició dos peticiones. La primera devolvió recibo; la segunda seguía activa después de 102 segundos pese al timeout de cliente de 20 segundos. Se detuvieron únicamente los dos procesos identificados de esa prueba. No se capturaron chunks remotos: no atribuimos al proveedor el patrón reproducido localmente.

Con reqwest blocking 0.12.28, configurar solo ClientBuilder.timeout permite renovar el plazo en cada lectura del cuerpo. Una reproducción HTTP/1 local con JSON válido entregado lentamente pasó de 229 ms con timeout de 100 ms a error dentro del plazo al fijar RequestBuilder.timeout. La implementación de la dependencia confirma la diferencia entre espera de lectura blocking y plazo de petición async.

Se mantiene el plazo existente de 20 segundos y se fija también en cada RequestBuilder del clasificador, testigo y transporte de investigación. No se introduce reintento ni otro motor. El presupuesto pertenece a cada HTTP; una operación con varias etapas puede durar más. Se conservan límite de bytes, errores, uso parcial y degradación sin selección fiable. El endpoint parametrizado privado permite probar el transporte real con servidores efímeros locales.

Tres regresiones cubren cuerpo lento HTTP/1, cabeceras más cuerpo en un presupuesto y cuerpo lento HTTP/2. Los servidores son propiedad de cada prueba, sus threads terminan y no usan claves reales. No se reinició el dashboard ni se volvió a llamar a OpenRouter después de la interrupción. Las mediciones previas conservan el código ejecutado y no reciben retroactivamente este arreglo.

Alternativas: ampliar el timeout ocultaría el incumplimiento; sustituir el cliente añade un mecanismo innecesario ante la reproducción mínima. Revisar esta decisión si las pruebas reales revelan otro escape del plazo o si cambia el transporte.

Dependencias: bootstrap declara acceso dev a Tokio 1.53.1 ya fijado en el lock, con feature time; no cambia versiones ni añade paquete o dependencia de producción. Se comprobaron [documentación exacta](https://docs.rs/tokio/1.53.1/tokio/) y [índice RustSec](https://rustsec.org/packages/tokio.html). El índice incluye RUSTSEC-2026-0384 sobre Interval y versión 1.54.0; el detalle no estuvo accesible, por lo que esto no es una auditoría exhaustiva ni una afirmación de ausencia de vulnerabilidades. El mock nuevo no usa Interval. La revisión no identificó evidencia de compromiso, sin acreditarse una auditoría de cadena de suministro.
