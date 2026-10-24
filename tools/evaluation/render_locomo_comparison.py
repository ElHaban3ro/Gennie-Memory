#!/usr/bin/env python3
"""Render current LoCoMo retrieval and Decisions figures from Rust reports."""

from __future__ import annotations

import argparse
import html
import json
from pathlib import Path

PAPER = "#fffefa"
INK = "#25313b"
MUTED = "#617078"
GRID = "#d8dfdf"
TOOL_COLORS = {
    "Genny · palabras clave": "#18817e",
    "Genny · natural": "#5277b8",
    "Engram 3.4.0": "#b77a27",
    "Lodis 0.6.0": "#8a6f9b",
    "Basic Memory": "#b75948",
    "Genny": "#18817e",
    "Decisions": "#496b53",
}


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def esc(text: str) -> str:
    return html.escape(text, quote=True)


def text(x: int, y: int, value: str, size: int = 17, color: str = INK,
         anchor: str = "start", font: str = "Arial,sans-serif") -> str:
    return (f'<text x="{x}" y="{y}" fill="{color}" text-anchor="{anchor}" '
            f'font-family="{font}" font-size="{size}">{esc(value)}</text>')


def start(title: str, subtitle: str, width: int = 1200, height: int = 650) -> list[str]:
    return [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}" role="img" aria-label="{esc(title)}">',
        f'<rect width="100%" height="100%" fill="{PAPER}"/>',
        text(58, 72, title, 34, INK, font="Georgia,serif"),
        text(58, 108, subtitle, 17, MUTED),
    ]


def save(lines: list[str], path: Path) -> None:
    path.write_text("\n".join([*lines, "</svg>"]) + "\n", encoding="utf-8")


def percent(value: float) -> str:
    return f"{value * 100:.2f}%"


def vertical_grouped_bar_chart(path: Path, title: str, subtitle: str,
                               metrics: list[tuple[str, str, str, str]], max_value: float,
                               decimals: int = 0, suffix: str = "%") -> None:
    """Render vertical bars, with stable tool colors and lighter secondary series."""
    lines = start(title, subtitle, height=730)
    left, right, top, bottom = 100, 1140, 165, 500
    groups: dict[str, list[tuple[str, float, str]]] = {}
    for category, series, raw, tool in metrics:
        if tool not in TOOL_COLORS:
            raise ValueError(f"falta color identificativo para {tool}")
        groups.setdefault(category, []).append((series, float(raw), tool))

    for tick in range(5):
        value = max_value * tick / 4
        y = round(bottom - (bottom - top) * tick / 4)
        lines.append(f'<line x1="{left}" y1="{y}" x2="{right}" y2="{y}" stroke="{GRID}"/>')
        lines.append(text(left - 14, y + 5, f"{value:.{decimals}f}{suffix}", 14, MUTED, "end"))

    group_width = (right - left) / max(1, len(groups))
    for group_index, (category, entries) in enumerate(groups.items()):
        center = left + group_width * (group_index + 0.5)
        series_count = len(entries)
        bar_width = min(48, group_width * 0.26)
        gap = min(10, group_width * 0.06)
        total_width = series_count * bar_width + (series_count - 1) * gap
        first_x = center - total_width / 2
        for bar_index, (series, value, tool) in enumerate(entries):
            bar_height = (bottom - top) * value / max_value
            x = first_x + bar_index * (bar_width + gap)
            y = bottom - bar_height
            opacity = 0.52 if series_count > 1 and bar_index == 1 else 1
            lines.append(
                f'<rect x="{x:.1f}" y="{y:.1f}" width="{bar_width:.1f}" '
                f'height="{bar_height:.1f}" fill="{TOOL_COLORS[tool]}" '
                f'fill-opacity="{opacity}"/>'
            )
            label_y = max(top - 8, y - 9)
            lines.append(text(round(x + bar_width / 2), round(label_y),
                              f"{value:.{decimals}f}{suffix}", 13, INK, "middle"))

        label_lines = category.split("\n")
        for line_index, label in enumerate(label_lines):
            lines.append(text(round(center), bottom + 29 + line_index * 19,
                              label, 14, INK, "middle"))

    series_names = list(dict.fromkeys(series for _, series, _, _ in metrics))
    tools = list(dict.fromkeys(tool for _, _, _, tool in metrics))
    legend_y = 608
    x = 100
    if len(series_names) > 1:
        lines.append(text(x, legend_y, "Tono de barra:", 14, MUTED))
        x += 108
        for index, series in enumerate(series_names):
            opacity = 0.52 if index == 1 else 1
            lines.append(f'<rect x="{x}" y="{legend_y - 14}" width="14" height="14" '
                         f'fill="{INK}" fill-opacity="{opacity}"/>')
            lines.append(text(x + 21, legend_y, series, 14, MUTED))
            x += max(120, len(series) * 8 + 42)

    tool_legend_y = 660
    lines.append(text(100, tool_legend_y, "Herramienta:", 14, MUTED))
    x = 205
    for tool in tools:
        lines.append(f'<rect x="{x}" y="{tool_legend_y - 14}" width="14" height="14" '
                     f'fill="{TOOL_COLORS[tool]}"/>')
        lines.append(text(x + 21, tool_legend_y, tool, 14, MUTED))
        x += max(140, len(tool) * 8 + 48)
    save(lines, path)


def render(args: argparse.Namespace) -> None:
    reports = {
        "Genny · palabras clave": load(args.keywords),
        "Genny · natural": load(args.natural),
        "Engram 3.4.0": load(args.engram),
        "Lodis 0.6.0": load(args.lodis),
        "Basic Memory": load(args.basic),
    }
    decisions = load(args.decisions)
    args.output.mkdir(parents=True, exist_ok=True)

    accuracy = []
    for label, report in reports.items():
        overall = report["overall"]
        accuracy.extend([
            (label.replace(" · ", "\n"), "top 1", str(overall["top_1_accuracy"] * 100), label),
            (label.replace(" · ", "\n"), "top 20", str(overall["recall_at_20"] * 100), label),
        ])
    vertical_grouped_bar_chart(
        args.output / "locomo-retrieval-accuracy.svg",
        "Recuperación de evidencia en LoCoMo",
        "1.978 preguntas con evidencia · top 1 y top 20 · una corrida por motor",
        accuracy, 100, 1,
    )

    latency = []
    for label, report in reports.items():
        timings = report["overall"]["latency_ms"]
        latency.extend([
            (label.replace(" · ", "\n"), "p50", str(timings["p50"]), label),
            (label.replace(" · ", "\n"), "p95", str(timings["p95"]), label),
        ])
    vertical_grouped_bar_chart(
        args.output / "locomo-search-latency.svg",
        "Latencia local de búsqueda",
        "Misma cohorte de 1.982 consultas · milisegundos · una corrida por motor",
        latency, 125, 1, " ms",
    )

    all_evidence = []
    for label, report in reports.items():
        overall = report["overall"]
        all_evidence.extend([
            (label.replace(" · ", "\n"), "@5", str(overall["all_evidence_recall_at_5"] * 100), label),
            (label.replace(" · ", "\n"), "@20", str(overall["all_evidence_recall_at_20"] * 100), label),
        ])
    vertical_grouped_bar_chart(
        args.output / "locomo-all-evidence.svg",
        "Cobertura de todos los recuerdos anotados",
        "Porcentaje de preguntas positivas cuyo conjunto completo aparece en top 5 y top 20",
        all_evidence, 100, 1,
    )

    positives = decisions["positive"]
    negatives = decisions["negative"]
    route = [
        ("Candidato esperado\nen top 20", "resultado", str(decisions["candidate_recall"] / positives * 100), "Genny"),
        ("Semilla con\nevidencia útil", "resultado", str(decisions["positive_correct"] / positives * 100), "Decisions"),
        ("Toda la evidencia\nen la semilla", "resultado", str(decisions["positive_all_evidence_selected"] / positives * 100), "Decisions"),
        ("Semilla vacía\nen negativos", "resultado", str(decisions["negative_empty"] / negatives * 100), "Decisions"),
    ]
    vertical_grouped_bar_chart(
        args.output / "locomo-decisions-seed.svg",
        "Qué llega hasta la semilla",
        f"Decisions · {positives} positivas, {negatives} negativas · US${decisions['reported_cost_usd']:.4f} informados",
        route, 100, 1,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--keywords", type=Path, required=True)
    parser.add_argument("--natural", type=Path, required=True)
    parser.add_argument("--engram", type=Path, required=True)
    parser.add_argument("--lodis", type=Path, required=True)
    parser.add_argument("--basic", type=Path, required=True)
    parser.add_argument("--decisions", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    render(args)
    print(f"figuras LoCoMo guardadas en {args.output}")


if __name__ == "__main__":
    main()
