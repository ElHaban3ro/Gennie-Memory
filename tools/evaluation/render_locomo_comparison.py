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
TEAL = "#18817e"
GOLD = "#b77a27"
BLUE = "#617db1"
PURPLE = "#8a6f9b"
RED = "#b75948"
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


def grouped_bar_chart(path: Path, title: str, subtitle: str,
                      metrics: list[tuple[str, str, str]], max_value: float,
                      decimals: int = 0, suffix: str = "%",
                      show_legend: bool = True) -> None:
    lines = start(title, subtitle)
    left, right, top, bottom = 265, 1115, 175, 540
    for tick in range(0, 5):
        value = max_value * tick / 4
        x = round(left + (right - left) * tick / 4)
        lines.append(f'<line x1="{x}" y1="{top}" x2="{x}" y2="{bottom}" stroke="{GRID}"/>')
        lines.append(text(x, bottom + 30, f"{value:.{decimals}f}{suffix}", 14, MUTED, "middle"))
    row_height = min(68, max(36, (bottom - top) // max(1, len(metrics))))
    for index, (label, series, raw) in enumerate(metrics):
        y = top + index * row_height + 13
        value = float(raw)
        width = max(1, round((right - left) * value / max_value))
        lines.append(text(left - 14, y + 17, label, 16, INK, "end"))
        color = BLUE if "p95" in series or "20" in series else TEAL
        lines.append(f'<rect x="{left}" y="{y}" width="{width}" height="22" fill="{color}"/>')
        lines.append(text(min(left + width + 9, right - 3), y + 17, f"{value:.{decimals}f}{suffix}", 15, INK))
    legend_y = bottom + 76
    series_names = list(dict.fromkeys(series for _, series, _ in metrics))
    if show_legend and len(series_names) > 1:
        x = 58
        for series in series_names:
            color = BLUE if "p95" in series or "20" in series else TEAL
            lines.append(f'<rect x="{x}" y="{legend_y - 14}" width="14" height="14" fill="{color}"/>')
            lines.append(text(x + 21, legend_y, series, 14, MUTED))
            x += max(140, len(series) * 9 + 42)
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
            (label, "top 1", str(overall["top_1_accuracy"] * 100)),
            (label, "top 20", str(overall["recall_at_20"] * 100)),
        ])
    grouped_bar_chart(
        args.output / "locomo-retrieval-accuracy.svg",
        "Recuperación de evidencia en LoCoMo",
        "1.978 preguntas con evidencia · top 1 y top 20 · una corrida por motor",
        accuracy, 100, 1,
    )

    latency = []
    for label, report in reports.items():
        timings = report["overall"]["latency_ms"]
        latency.extend([
            (label, "p50", str(timings["p50"])),
            (label, "p95", str(timings["p95"])),
        ])
    grouped_bar_chart(
        args.output / "locomo-search-latency.svg",
        "Latencia local de búsqueda",
        "Misma cohorte de 1.982 consultas · milisegundos · una corrida por motor",
        latency, 125, 1, " ms",
    )

    all_evidence = []
    for label, report in reports.items():
        overall = report["overall"]
        all_evidence.extend([
            (label, "toda la evidencia @5", str(overall["all_evidence_recall_at_5"] * 100)),
            (label, "toda la evidencia @20", str(overall["all_evidence_recall_at_20"] * 100)),
        ])
    grouped_bar_chart(
        args.output / "locomo-all-evidence.svg",
        "Cobertura de todos los recuerdos anotados",
        "Porcentaje de preguntas positivas cuyo conjunto completo aparece en top 5 y top 20",
        all_evidence, 100, 1,
    )

    positives = decisions["positive"]
    negatives = decisions["negative"]
    route = [
        ("Candidato esperado en top 20", "Genny", str(decisions["candidate_recall"] / positives * 100)),
        ("Semilla con evidencia útil", "Decisions", str(decisions["positive_correct"] / positives * 100)),
        ("Toda la evidencia en la semilla", "Decisions", str(decisions["positive_all_evidence_selected"] / positives * 100)),
        ("Semilla vacía en negativos", "Decisions", str(decisions["negative_empty"] / negatives * 100)),
    ]
    grouped_bar_chart(
        args.output / "locomo-decisions-seed.svg",
        "Qué llega hasta la semilla",
        f"Decisions · {positives} positivas, {negatives} negativas · US${decisions['reported_cost_usd']:.4f} informados",
        route, 100, 1, show_legend=False,
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
