//! Reusable SVG components for the measured, synthetic README figures.

use serde_json::Value;
use std::fs;
use std::path::Path;

const INK: &str = "#25313b";
const MUTED: &str = "#617078";
const GRID: &str = "#d8dfdf";
const TEAL: &str = "#18817e";
const GOLD: &str = "#b77a27";
const BLUE: &str = "#617db1";

struct Svg(String);

impl Svg {
    fn new(width: u32, height: u32, title: &str) -> Self {
        let mut svg = Self(format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"{}\">\n<rect width=\"100%\" height=\"100%\" fill=\"#fffefa\"/>\n",
            escape(title)
        ));
        svg.text(58, 75, title, 34, INK, "start", "Georgia,serif");
        svg
    }
    fn text(
        &mut self,
        x: i32,
        y: i32,
        value: &str,
        size: u32,
        color: &str,
        anchor: &str,
        font: &str,
    ) {
        self.0.push_str(&format!("<text x=\"{x}\" y=\"{y}\" fill=\"{color}\" text-anchor=\"{anchor}\" font-family=\"{font}\" font-size=\"{size}\">{}</text>\n",escape(value)));
    }
    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: &str, width: u32) {
        self.0.push_str(&format!("<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{color}\" stroke-width=\"{width}\"/>\n"));
    }
    fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: &str) {
        self.0.push_str(&format!(
            "<rect x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" fill=\"{color}\"/>\n"
        ));
    }
    fn save(mut self, path: &Path) -> Result<(), String> {
        self.0.push_str("</svg>\n");
        fs::write(path, self.0).map_err(|error| error.to_string())
    }
}

fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn load(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
        .map_err(|error| format!("{}: {error}", path.display()))
}
fn f(value: &Value, pointer: &str) -> Result<f64, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("falta {pointer}"))
}
fn n(value: &Value, pointer: &str) -> Result<u64, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("falta {pointer}"))
}
fn median(mut values: Vec<f64>) -> Result<f64, String> {
    if values.is_empty() {
        return Err("informe Codex sin filas".into());
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    Ok(if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    })
}
fn codex_p50(value: &Value) -> Result<f64, String> {
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or("faltan filas Codex")?;
    median(
        rows.iter()
            .map(|row| f(row, "/final_answer_ms"))
            .collect::<Result<Vec<_>, _>>()?,
    )
}

struct Panel<'a> {
    x: i32,
    title: &'a str,
    subtitle: &'a str,
    unit: &'a str,
    maximum: f64,
    bars: [(&'a str, f64, &'a str); 2],
    decimals: usize,
}

fn panel(svg: &mut Svg, p: &Panel<'_>) {
    let top = 215;
    let base = 497;
    let left = p.x + 52;
    let right = p.x + 445;
    svg.text(p.x, 144, p.title, 22, INK, "start", "Georgia,serif");
    svg.text(p.x, 174, p.subtitle, 16, MUTED, "start", "Arial,sans-serif");
    svg.text(left, 199, p.unit, 14, MUTED, "start", "Arial,sans-serif");
    for part in [0.0, 0.5, 1.0] {
        let y = base - ((base - top) as f64 * part).round() as i32;
        svg.line(left, y, right, y, GRID, 1);
        svg.text(
            left - 10,
            y + 5,
            &format!(
                "{:.*}",
                if p.decimals > 0 { 1 } else { 0 },
                p.maximum * part
            ),
            14,
            MUTED,
            "end",
            "Arial,sans-serif",
        );
    }
    svg.line(left, top, left, base, INK, 1);
    for (index, (label, value, color)) in p.bars.iter().enumerate() {
        let center = p.x + 165 + (index as i32) * 175;
        let height = ((base - top) as f64 * (*value / p.maximum)).round() as i32;
        let height = height.clamp(0, base - top);
        svg.rect(center - 48, base - height, 96, height, color);
        svg.text(
            center,
            base - height - 13,
            &format!("{:.*}", p.decimals, value),
            18,
            INK,
            "middle",
            "Arial,sans-serif",
        );
        svg.text(
            center,
            base + 30,
            label,
            17,
            INK,
            "middle",
            "Arial,sans-serif",
        );
    }
}

fn render_speed(
    local: &Value,
    engram: &Value,
    seed_codex: &Value,
    engram_codex: &Value,
    path: &Path,
) -> Result<(), String> {
    let fast = f(local, "/overall/latency_ms/p50")?;
    let engram_search = f(engram, "/overall/latency_ms/p50")?;
    let seeded_s = codex_p50(seed_codex)? / 1000.0;
    let engram_s = codex_p50(engram_codex)? / 1000.0;
    let mut svg = Svg::new(1200, 645, "iROMEM frente a Engram: búsqueda y respuesta");
    svg.text(
        58,
        109,
        "Dos etapas medidas por separado · medianas · datos sintéticos",
        17,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    panel(
        &mut svg,
        &Panel {
            x: 58,
            title: "1. Buscar candidatos",
            subtitle: "540 preguntas · 2.800 recuerdos",
            unit: "milisegundos",
            maximum: (fast.max(engram_search) * 1.25).ceil().max(1.0),
            bars: [("iROMEM", fast, TEAL), ("Engram", engram_search, GOLD)],
            decimals: 2,
        },
    );
    panel(
        &mut svg,
        &Panel {
            x: 610,
            title: "2. Responder con Codex",
            subtitle: "6 turnos aislados · prompt vs MCP",
            unit: "segundos",
            maximum: (seeded_s.max(engram_s) * 1.25).ceil().max(1.0),
            bars: [("iROMEM", seeded_s, TEAL), ("Engram", engram_s, GOLD)],
            decimals: 2,
        },
    );
    svg.text(58,586,"En el panel 2, Codex recibió la semilla de iROMEM en el prompt; Engram se consultó por MCP.",16,MUTED,"start","Arial,sans-serif");
    svg.text(58,612,"Escalas y muestras distintas (540 y 6); la integración automática de iROMEM sigue pendiente.",16,MUTED,"start","Arial,sans-serif");
    svg.save(path)
}

fn render_accuracy(
    local: &Value,
    engram: &Value,
    decisions: &Value,
    jev: &Value,
    path: &Path,
) -> Result<(), String> {
    let positives = n(decisions, "/positive")?;
    if positives != n(jev, "/positive")?
        || positives != n(local, "/overall/positive")?
        || positives != n(engram, "/overall/positive")?
    {
        return Err("cohortes positivas diferentes".into());
    }
    let local_hits = (f(local, "/overall/recall_at_20")? * positives as f64).round() as u64;
    let engram_hits = (f(engram, "/overall/recall_at_20")? * positives as f64).round() as u64;
    let decisions_hits = n(decisions, "/positive_correct")?;
    let jev_hits = n(jev, "/positive_correct")?;
    let mut svg = Svg::new(1550, 645, "¿Cuántas veces acertó cada etapa?");
    svg.text(
        58,
        109,
        "Cohorte final · 460 preguntas positivas · cuentas observadas",
        17,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    panel(
        &mut svg,
        &Panel {
            x: 58,
            title: "1. El recuerdo llega al top 20",
            subtitle: "Búsqueda local en 2.800 recuerdos",
            unit: "aciertos / 460",
            maximum: positives as f64,
            bars: [
                ("iROMEM", local_hits as f64, TEAL),
                ("Engram", engram_hits as f64, GOLD),
            ],
            decimals: 0,
        },
    );
    panel(
        &mut svg,
        &Panel {
            x: 803,
            title: "2. La semilla final es correcta",
            subtitle: "Decisión completa de iROMEM",
            unit: "aciertos / 460",
            maximum: positives as f64,
            bars: [
                ("Decisions", decisions_hits as f64, TEAL),
                ("Jev", jev_hits as f64, BLUE),
            ],
            decimals: 0,
        },
    );
    svg.text(58,586,"Engram no produce una semilla con el contrato de iROMEM; por eso solo aparece en el panel de candidatos.",16,MUTED,"start","Arial,sans-serif");
    svg.text(58,612,&format!("Controles negativos: Decisions {}/{} vacíos; Jev {}/{} vacíos. Ejes completos desde cero.",
        n(decisions,"/negative_empty")?,n(decisions,"/negative")?,n(jev,"/negative_empty")?,n(jev,"/negative")?),16,MUTED,"start","Arial,sans-serif");
    svg.save(path)
}

pub fn run(paths: &[&Path], output_dir: &Path) -> Result<(), String> {
    if paths.len() != 6 {
        return Err("se requieren seis informes".into());
    }
    let data = paths
        .iter()
        .map(|path| load(path))
        .collect::<Result<Vec<_>, _>>()?;
    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    render_speed(
        &data[0],
        &data[1],
        &data[4],
        &data[5],
        &output_dir.join("speed-comparison-rust.svg"),
    )?;
    render_accuracy(
        &data[0],
        &data[1],
        &data[2],
        &data[3],
        &output_dir.join("accuracy-comparison-rust.svg"),
    )?;
    println!("dos figuras SVG en {}", output_dir.display());
    Ok(())
}

struct ThreeBars<'a> {
    x: i32,
    width: i32,
    title: &'a str,
    subtitle: &'a str,
    unit: &'a str,
    maximum: f64,
    bars: [(&'a str, f64, &'a str); 3],
    decimals: usize,
}

fn three_bars(svg: &mut Svg, panel: &ThreeBars<'_>) {
    let top = 215;
    let base = 505;
    let left = panel.x + 55;
    let right = panel.x + panel.width - 15;
    svg.text(panel.x, 144, panel.title, 22, INK, "start", "Georgia,serif");
    svg.text(
        panel.x,
        174,
        panel.subtitle,
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    svg.text(
        left,
        199,
        panel.unit,
        14,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    for fraction in [0.0, 0.5, 1.0] {
        let y = base - ((base - top) as f64 * fraction).round() as i32;
        svg.line(left, y, right, y, GRID, 1);
        svg.text(
            left - 10,
            y + 5,
            &format!("{:.*}", panel.decimals, panel.maximum * fraction),
            14,
            MUTED,
            "end",
            "Arial,sans-serif",
        );
    }
    svg.line(left, top, left, base, INK, 1);
    let span = (right - left) as f64;
    for (index, (label, value, color)) in panel.bars.iter().enumerate() {
        let center = left + (span * ((index + 1) as f64 / 4.0)).round() as i32;
        let height = ((base - top) as f64 * (*value / panel.maximum)).round() as i32;
        svg.rect(
            center - 39,
            base - height.clamp(0, base - top),
            78,
            height.clamp(0, base - top),
            color,
        );
        svg.text(
            center,
            base - height - 13,
            &format!("{:.*}", panel.decimals, value),
            17,
            INK,
            "middle",
            "Arial,sans-serif",
        );
        svg.text(
            center,
            base + 31,
            label,
            16,
            INK,
            "middle",
            "Arial,sans-serif",
        );
    }
}

pub fn render_keyword_improvement(paths: &[&Path], output_dir: &Path) -> Result<(), String> {
    if paths.len() != 6 {
        return Err("se requieren seis informes de búsqueda y campaña".into());
    }
    let values = paths
        .iter()
        .map(|path| load(path))
        .collect::<Result<Vec<_>, _>>()?;
    let positives = n(&values[1], "/positive")?;
    for index in [0, 2, 4] {
        if n(&values[index], "/overall/positive")? != positives
            || n(&values[index + 1], "/positive")? != positives
            || n(&values[index + 1], "/candidate_recall")?
                != (f(&values[index], "/overall/recall_at_20")? * positives as f64).round() as u64
        {
            return Err("informes de cohortes o candidatos incoherentes".into());
        }
    }
    let hits = [0, 2, 4]
        .map(|index| n(&values[index + 1], "/candidate_recall"))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let seeds = [1, 3, 5]
        .map(|index| n(&values[index], "/positive_correct"))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let colors = [GOLD, TEAL, BLUE];
    let labels = ["Antes", "Ajuste", "Nuevos"];
    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    let mut quality = Svg::new(1550, 680, "Recuerdos encontrados y semillas correctas");
    quality.text(
        58,
        110,
        "460 preguntas positivas por cohorte · barras desde cero · datos sintéticos",
        17,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    for (x, title, subtitle, data) in [
        (
            58,
            "1. Llegó entre 20 candidatos",
            "FTS5 · 2.800 recuerdos",
            &hits,
        ),
        (800, "2. Semilla final correcta", "FTS5 + Decisions", &seeds),
    ] {
        three_bars(
            &mut quality,
            &ThreeBars {
                x,
                width: 640,
                title,
                subtitle,
                unit: "aciertos / 460",
                maximum: positives as f64,
                bars: std::array::from_fn(|index| {
                    (labels[index], data[index] as f64, colors[index])
                }),
                decimals: 0,
            },
        );
    }
    quality.text(58, 591, "Antes y ajuste: mismos sujetos 002–005. Nuevos: sujetos 010–013, no usados para elegir el cambio.",
        16, MUTED, "start", "Arial,sans-serif");
    quality.text(58, 622, &format!("Negativos sin semilla: {}/80 · {}/80 · {}/80. Las plantillas sintéticas se repiten entre cohortes.",
        n(&values[1], "/negative_empty")?, n(&values[3], "/negative_empty")?, n(&values[5], "/negative_empty")?),
        16, MUTED, "start", "Arial,sans-serif");
    quality.text(
        58,
        650,
        &format!(
            "Cambio en los mismos sujetos: +{} candidatos correctos y +{} semillas correctas.",
            hits[1].saturating_sub(hits[0]),
            seeds[1].saturating_sub(seeds[0])
        ),
        16,
        TEAL,
        "start",
        "Arial,sans-serif",
    );
    quality.save(&output_dir.join("keyword-accuracy-v1.svg"))?;

    let mut speed = Svg::new(1550, 680, "Latencia medida por etapa");
    speed.text(
        58,
        110,
        "Medianas y p95 observados · 540 consultas por cohorte · escalas independientes",
        17,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    for (x, title, subtitle, pointer, indices) in [
        (
            58,
            "1. Búsqueda p50",
            "Candidatos · sesión local",
            "/overall/latency_ms/p50",
            [0, 2, 4],
        ),
        (
            553,
            "2. Semilla p50",
            "Consulta completa · red incluida",
            "/elapsed_p50_ms",
            [1, 3, 5],
        ),
        (
            1048,
            "3. Semilla p95",
            "Cola de latencia · red incluida",
            "/elapsed_p95_ms",
            [1, 3, 5],
        ),
    ] {
        let data = indices
            .map(|index| f(&values[index], pointer))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let maximum = (data.iter().copied().fold(0.0, f64::max) * 1.15)
            .ceil()
            .max(1.0);
        three_bars(
            &mut speed,
            &ThreeBars {
                x,
                width: 445,
                title,
                subtitle,
                unit: "milisegundos",
                maximum,
                bars: std::array::from_fn(|index| (labels[index], data[index], colors[index])),
                decimals: if x == 58 { 2 } else { 0 },
            },
        );
    }
    speed.text(58, 591, "Cada cohorte tuvo una corrida distinta; el p95 de red no se atribuye solo a la búsqueda local.",
        16, MUTED, "start", "Arial,sans-serif");
    speed.text(
        58,
        622,
        "Codex no se midió con esta versión. La gráfica de Codex anterior se conserva aparte.",
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    speed.save(&output_dir.join("keyword-latency-v1.svg"))?;
    Ok(())
}

pub fn render_current_performance(
    search_path: &Path,
    campaign_path: &Path,
    engram_path: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    let search = load(search_path)?;
    let campaign = load(campaign_path)?;
    let engram = load(engram_path)?;
    let positives = n(&search, "/overall/positive")?;
    let negatives = n(&search, "/overall/negative")?;
    if positives != n(&campaign, "/positive")?
        || negatives != n(&campaign, "/negative")?
        || positives != n(&engram, "/overall/positive")?
        || negatives != n(&engram, "/overall/negative")?
        || (f(&search, "/overall/recall_at_20")? * positives as f64).round() as u64
            != n(&campaign, "/candidate_recall")?
    {
        return Err("informes de búsqueda, Engram y campaña incoherentes".into());
    }

    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    let mut quality = Svg::new(1200, 650, "Recuerdos recuperados en las mismas preguntas");
    quality.text(
        58,
        110,
        "460 preguntas positivas · 2.800 recuerdos sintéticos · barras desde cero",
        17,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    let top_20 = [
        (f(&search, "/overall/recall_at_20")? * positives as f64).round(),
        (f(&engram, "/overall/recall_at_20")? * positives as f64).round(),
    ];
    let top_1 = [
        (f(&search, "/overall/top_1_accuracy")? * positives as f64).round(),
        (f(&engram, "/overall/top_1_accuracy")? * positives as f64).round(),
    ];
    for data in [
        Panel {
            x: 58,
            title: "Entre 20 candidatos",
            subtitle: "Recuerdo esperado presente",
            unit: "aciertos / 460",
            maximum: positives as f64,
            bars: [("iROMEM", top_20[0], TEAL), ("Engram", top_20[1], GOLD)],
            decimals: 0,
        },
        Panel {
            x: 610,
            title: "Primer candidato",
            subtitle: "Recuerdo esperado en posición 1",
            unit: "aciertos / 460",
            maximum: positives as f64,
            bars: [("iROMEM", top_1[0], TEAL), ("Engram", top_1[1], GOLD)],
            decimals: 0,
        },
    ] {
        panel(&mut quality, &data);
    }
    quality.text(
        58,
        570,
        &format!(
            "iROMEM + Decisions produjo {}/{} semillas correctas y dejó {}/{} negativos sin semilla.",
            n(&campaign, "/positive_correct")?,
            positives,
            n(&campaign, "/negative_empty")?,
            negatives
        ),
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    quality.text(
        58,
        602,
        "Engram se midió como buscador de candidatos; no se le atribuye una semilla clasificada.",
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    quality.save(&output_dir.join("current-accuracy.svg"))?;

    let search_p50 = f(&search, "/overall/latency_ms/p50")?;
    let search_p95 = f(&search, "/overall/latency_ms/p95")?;
    let engram_p50 = f(&engram, "/overall/latency_ms/p50")?;
    let engram_p95 = f(&engram, "/overall/latency_ms/p95")?;
    let mut latency = Svg::new(1200, 635, "Tiempo de búsqueda de candidatos");
    latency.text(
        58,
        110,
        "540 preguntas idénticas para iROMEM y Engram · milisegundos",
        17,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    for data in [
        Panel {
            x: 58,
            title: "Búsqueda: mediana",
            subtitle: "Candidatos · proceso local",
            unit: "milisegundos",
            maximum: (search_p50.max(engram_p50) * 1.1).ceil(),
            bars: [("iROMEM", search_p50, TEAL), ("Engram", engram_p50, GOLD)],
            decimals: 2,
        },
        Panel {
            x: 610,
            title: "Búsqueda: p95",
            subtitle: "Cola observada",
            unit: "milisegundos",
            maximum: (search_p95.max(engram_p95) * 1.1).ceil(),
            bars: [("iROMEM", search_p95, TEAL), ("Engram", engram_p95, GOLD)],
            decimals: 2,
        },
    ] {
        panel(&mut latency, &data);
    }
    latency.text(
        58,
        585,
        "Cada búsqueda se midió en una corrida local separada; estas barras no incluyen clasificación.",
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    latency.save(&output_dir.join("current-latency.svg"))?;

    Ok(())
}

pub fn render_current_codex(
    seed_path: &Path,
    engram_path: &Path,
    paired_path: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    let seed = load(seed_path)?;
    let engram = load(engram_path)?;
    let paired = load(paired_path)?;
    let seed_ids = seed
        .get("case_ids")
        .and_then(Value::as_array)
        .ok_or("faltan casos Codex con iROMEM")?;
    let engram_ids = engram
        .get("case_ids")
        .and_then(Value::as_array)
        .ok_or("faltan casos Codex con Engram")?;
    if seed_ids.is_empty()
        || seed_ids != engram_ids
        || n(&paired, "/sample_count")? != seed_ids.len() as u64
    {
        return Err("las corridas de Codex no tienen los mismos casos".into());
    }
    let seed_ms = codex_p50(&seed)?;
    let engram_ms = codex_p50(&engram)?;
    if (seed_ms - f(&paired, "/median_codex_seed_ms")?).abs() > 0.001
        || (engram_ms - f(&paired, "/median_codex_engram_ms")?).abs() > 0.001
    {
        return Err("el informe emparejado no coincide con los turnos Codex".into());
    }
    let input_p50 = |report: &Value| -> Result<f64, String> {
        let rows = report
            .get("rows")
            .and_then(Value::as_array)
            .ok_or("faltan filas Codex")?;
        if rows.len() != seed_ids.len() {
            return Err("el número de turnos Codex no coincide".into());
        }
        median(
            rows.iter()
                .map(|row| n(row, "/usage/input_tokens").map(|tokens| tokens as f64))
                .collect::<Result<Vec<_>, _>>()?,
        )
    };
    let seed_tokens = input_p50(&seed)?;
    let engram_tokens = input_p50(&engram)?;

    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    let mut svg = Svg::new(1200, 650, "Codex: iROMEM frente a Engram");
    svg.text(
        58,
        110,
        &format!(
            "{} turnos sintéticos emparejados · misma versión de Codex · medianas",
            seed_ids.len()
        ),
        17,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    for data in [
        Panel {
            x: 58,
            title: "Tiempo hasta la respuesta",
            subtitle: "Codex con semilla o búsqueda MCP",
            unit: "segundos",
            maximum: ((seed_ms.max(engram_ms) / 1000.0) * 1.2).ceil(),
            bars: [
                ("iROMEM", seed_ms / 1000.0, TEAL),
                ("Engram", engram_ms / 1000.0, GOLD),
            ],
            decimals: 2,
        },
        Panel {
            x: 610,
            title: "Tokens de entrada informados",
            subtitle: "Incluyen contexto e instrucciones",
            unit: "tokens",
            maximum: ((seed_tokens.max(engram_tokens) * 1.2) / 1000.0).ceil() * 1000.0,
            bars: [
                ("iROMEM", seed_tokens.round(), TEAL),
                ("Engram", engram_tokens.round(), GOLD),
            ],
            decimals: 0,
        },
    ] {
        panel(&mut svg, &data);
    }
    svg.text(
        58,
        575,
        "iROMEM entregó la semilla en el prompt; Engram se consultó por MCP.",
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    svg.text(
        58,
        600,
        "Los tokens incluyen contexto y caché; no equivalen al coste marginal de la recuperación.",
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    svg.text(
        58,
        625,
        "La preparación de semilla se midió aparte; la integración automática sigue pendiente.",
        16,
        MUTED,
        "start",
        "Arial,sans-serif",
    );
    svg.save(&output_dir.join("current-codex-comparison.svg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xml_labels_are_escaped() {
        assert_eq!(escape("A & <B>"), "A &amp; &lt;B&gt;");
    }
}
