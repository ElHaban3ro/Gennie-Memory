mod benchmark;
mod campaign;
mod charts;
mod codex_benchmark;
mod cohort;
mod engram_benchmark;
mod retrieval;
mod search_benchmark;

use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("iromem-eval: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("uso: iromem-eval <comando> ...; consulte docs/evaluation/contract-v1.md")?;
    match command.as_str() {
        "retrieval" => {
            let results = args.next().ok_or("falta archivo JSONL de resultados")?;
            let queries = args.next().ok_or("falta archivo JSONL de consultas")?;
            let output = args.next();
            if args.next().is_some() {
                return Err("demasiados argumentos para retrieval".into());
            }
            retrieval::run(
                Path::new(&results),
                Path::new(&queries),
                output.as_deref().map(Path::new),
            )
        }
        "validate-benchmark" => {
            let directory = args.next().ok_or("falta directorio del banco")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para validate-benchmark".into());
            }
            benchmark::validate(Path::new(&directory))
        }
        "validate-general-memory" => {
            let directory = args.next().ok_or("falta directorio general-memory-v1")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            benchmark::validate_general_memory(Path::new(&directory))
        }
        "validate-candidate-matrix" => {
            let directory = args.next().ok_or("falta directorio candidate-matrix-v1")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            benchmark::validate_candidate_matrix(Path::new(&directory))
        }
        "verify-hashes" => {
            let directory = args.next().ok_or("falta directorio de paquete")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            benchmark::verify_hashes(Path::new(&directory))
        }
        "select-cohort" => {
            let queries = args.next().ok_or("faltan consultas del fixture")?;
            let memories = args.next().ok_or("faltan recuerdos del fixture")?;
            let first = args
                .next()
                .ok_or("falta primer sujeto")?
                .parse::<u32>()
                .map_err(|_| "primer sujeto inválido")?;
            let last = args
                .next()
                .ok_or("falta último sujeto")?
                .parse::<u32>()
                .map_err(|_| "último sujeto inválido")?;
            let output = args.next().ok_or("falta archivo de salida nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            cohort::run(
                Path::new(&queries),
                Path::new(&memories),
                first,
                last,
                Path::new(&output),
            )
        }
        "search-benchmark" => {
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let binary = args.next().ok_or("falta binario iromem")?;
            let database = args.next().ok_or("falta base SQLite")?;
            let output = args
                .next()
                .ok_or("falta archivo JSONL nuevo de resultados")?;
            let split = args.next().ok_or("falta partición")?;
            let mode = args.next().ok_or("falta modo keywords o natural")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para search-benchmark".into());
            }
            search_benchmark::run(
                Path::new(&cases),
                Path::new(&binary),
                Path::new(&database),
                Path::new(&output),
                &split,
                &mode,
            )
        }
        "campaign" => {
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let binary = args.next().ok_or("falta binario iromem")?;
            let database = args.next().ok_or("falta base SQLite")?;
            let journal = args.next().ok_or("falta diario JSONL")?;
            let provider = args.next().ok_or("falta proveedor decisions o jev")?;
            let split = args.next().ok_or("falta partición")?;
            let max_calls = args
                .next()
                .ok_or("falta máximo técnico de llamadas")?
                .parse::<usize>()
                .map_err(|_| "máximo de llamadas inválido")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para campaign".into());
            }
            campaign::run(
                Path::new(&cases),
                Path::new(&binary),
                Path::new(&database),
                Path::new(&journal),
                &provider,
                &split,
                max_calls,
            )
        }
        "campaign-report" => {
            let journal = args.next().ok_or("falta diario JSONL")?;
            let output = args.next();
            if args.next().is_some() {
                return Err("demasiados argumentos para campaign-report".into());
            }
            campaign::report(Path::new(&journal), output.as_deref().map(Path::new))
        }
        "codex-prompt" => {
            let journal = args.next().ok_or("falta diario de recall")?;
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let selected = args.next().ok_or("falta JSON de IDs seleccionados")?;
            let codex = args.next().ok_or("falta binario Codex")?;
            let output = args.next().ok_or("falta directorio de salida nuevo")?;
            let mode = args.next().ok_or("falta modo seed o control")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para codex-prompt".into());
            }
            codex_benchmark::run(
                Path::new(&journal),
                Path::new(&cases),
                Path::new(&selected),
                Path::new(&codex),
                Path::new(&output),
                &mode,
            )
        }
        "engram-benchmark" => {
            let cases = args.next().ok_or("falta archivo JSONL de casos")?;
            let memories = args.next().ok_or("falta fixture de recuerdos")?;
            let binary = args.next().ok_or("falta binario Engram")?;
            let data_dir = args.next().ok_or("falta directorio de datos nuevo")?;
            let output = args.next().ok_or("falta JSONL de salida nuevo")?;
            let split = args.next().ok_or("falta partición")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para engram-benchmark".into());
            }
            engram_benchmark::run(
                Path::new(&cases),
                Path::new(&memories),
                Path::new(&binary),
                Path::new(&data_dir),
                Path::new(&output),
                &split,
            )
        }
        "codex-engram" => {
            let cases = args.next().ok_or("falta JSONL de casos")?;
            let selected = args.next().ok_or("falta JSON de IDs seleccionados")?;
            let codex = args.next().ok_or("falta binario Codex")?;
            let engram = args.next().ok_or("falta binario Engram")?;
            let data_dir = args.next().ok_or("falta base Engram aislada")?;
            let output = args.next().ok_or("falta directorio de salida nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para codex-engram".into());
            }
            codex_benchmark::run_engram(
                Path::new(&cases),
                Path::new(&selected),
                Path::new(&codex),
                Path::new(&engram),
                Path::new(&data_dir),
                Path::new(&output),
            )
        }
        "render-charts" => {
            let mut inputs = Vec::new();
            for _ in 0..6 {
                inputs.push(args.next().ok_or("faltan informes para las gráficas")?);
            }
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos para render-charts".into());
            }
            charts::run(
                inputs.iter().map(Path::new).collect::<Vec<_>>().as_slice(),
                Path::new(&output),
            )
        }
        "render-keyword-charts" => {
            let mut inputs = Vec::new();
            for _ in 0..6 {
                inputs.push(args.next().ok_or("faltan informes")?);
            }
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_keyword_improvement(
                inputs.iter().map(Path::new).collect::<Vec<_>>().as_slice(),
                Path::new(&output),
            )
        }
        "render-current-charts" => {
            let search = args.next().ok_or("falta informe de búsqueda")?;
            let campaign = args.next().ok_or("falta informe de campaña")?;
            let engram = args.next().ok_or("falta informe Engram")?;
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_current_performance(
                Path::new(&search),
                Path::new(&campaign),
                Path::new(&engram),
                Path::new(&output),
            )
        }
        "render-codex-current" => {
            let seed = args.next().ok_or("falta resumen Codex con iROMEM")?;
            let engram = args.next().ok_or("falta resumen Codex con Engram")?;
            let paired = args.next().ok_or("falta informe emparejado")?;
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_current_codex(
                Path::new(&seed),
                Path::new(&engram),
                Path::new(&paired),
                Path::new(&output),
            )
        }
        "render-full-comparison" => {
            let aggregate = args.next().ok_or("falta agregado de la campaña")?;
            let output = args.next().ok_or("falta directorio de imágenes")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            charts::render_full_comparison(Path::new(&aggregate), Path::new(&output))
        }
        "paired-flow-report" => {
            let journal = args.next().ok_or("falta diario de recall")?;
            let seed = args.next().ok_or("falta resumen Codex con semilla")?;
            let engram = args.next().ok_or("falta resumen Codex con Engram")?;
            let control = args.next().ok_or("falta resumen Codex de control")?;
            let output = args.next().ok_or("falta JSON de salida nuevo")?;
            if args.next().is_some() {
                return Err("demasiados argumentos".into());
            }
            codex_benchmark::paired_report(
                Path::new(&journal),
                Path::new(&seed),
                Path::new(&engram),
                Path::new(&control),
                Path::new(&output),
            )
        }
        _ => Err("comando desconocido; consulte docs/evaluation/contract-v1.md".into()),
    }
}
