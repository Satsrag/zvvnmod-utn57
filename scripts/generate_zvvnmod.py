#!/usr/bin/env python3
"""从已审核的 CSV 生成 Rust ZVVNMOD 编码及 shape 定义。

Generate Rust ZVVNMOD code and shape definitions from the reviewed CSV.
"""

from __future__ import annotations

import argparse
import csv
from collections import OrderedDict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

POSITION_WORDS = {
    "i": ("INIT", "Init"),
    "m": ("MEDI", "Medi"),
    "f": ("FINA", "Fina"),
    "isol": ("ISOL", "Isol"),
}

@dataclass(frozen=True)
class ParsedShape:
    rust_name: str
    units: tuple[str, ...]
    position: str | None
    is_control: bool = False


@dataclass(frozen=True)
class InputRow:
    codepoint: int
    name: str
    source: str


@dataclass(frozen=True)
class CodeEntry:
    codepoint: int
    const_name: str
    shape_name: str | None
    source_name: str
    source: str


@dataclass
class Model:
    codes: list[CodeEntry]
    shapes: list[str]
    shape_to_codes: OrderedDict[str, list[CodeEntry]]


def _unit_identifier(unit: str) -> str:
    """将 written-unit 名称转换为 Rust 标识符。 / Convert a written-unit name to a Rust identifier."""

    identifier = "".join(ch if ch.isalnum() else "_" for ch in unit).upper()
    identifier = "_".join(part for part in identifier.split("_") if part)
    if not identifier or identifier[0].isdigit():
        raise ValueError(f"invalid written-unit ID: {unit!r}")
    return identifier


def parse_shape_name(
    name: str, codepoint: int | None = None, source: str = "font"
) -> ParsedShape:
    """解析一行 shape/control 名称。 / Parse one shape or control name.

    示例 / Example: ``B i I m`` → ``B_I_INIT``.
    """

    name = name.strip()
    # control-table 的名称由 CSV 决定，不在生成器中重复硬编码。
    # Control names come from the CSV and are not duplicated in the generator.
    if source == "control-table":
        if not name:
            location = f" for U+{codepoint:04X}" if codepoint is not None else ""
            raise ValueError(f"missing control name{location}")
        return ParsedShape(_unit_identifier(name), (), None, True)
    if source != "font":
        raise ValueError(f"unsupported source {source!r}")
    if not name:
        raise ValueError(f"missing name for U+{codepoint:04X}" if codepoint is not None else "missing name")
    if name == "Nirugu":
        return ParsedShape("NIRUGU", ("Nirugu",), None)

    parts = name.split()
    if len(parts) % 2:
        raise ValueError(f"name must contain unit/position pairs: {name!r}")

    units: list[str] = []
    short_positions: list[str] = []
    for index in range(0, len(parts), 2):
        unit, position = parts[index], parts[index + 1]
        if position not in POSITION_WORDS:
            raise ValueError(f"unknown position {position!r} in {name!r}")
        units.append(unit)
        short_positions.append(position)

    if len(units) > 1:
        # 多 shape 的首尾位置分别表示前后连接状态。
        # For a multi-part shape, the first and last positions encode its two joins.
        if (
            short_positions[0] not in {"i", "m"}
            or short_positions[-1] not in {"m", "f"}
            or any(position != "m" for position in short_positions[1:-1])
        ):
            raise ValueError(f"invalid multi-shape positions in {name!r}")
        position_suffix, position_variant = {
            ("i", "f"): ("ISOL", "Isol"),
            ("i", "m"): ("INIT", "Init"),
            ("m", "m"): ("MEDI", "Medi"),
            ("m", "f"): ("FINA", "Fina"),
        }[(short_positions[0], short_positions[-1])]
    else:
        position_suffix, position_variant = POSITION_WORDS[short_positions[-1]]

    rust_units = "_".join(_unit_identifier(unit) for unit in units)
    return ParsedShape(
        rust_name=f"{rust_units}_{position_suffix}",
        units=tuple(units),
        position=position_variant,
    )


def read_csv(path: Path) -> list[InputRow]:
    """读取并校验有效 CSV。 / Read and validate valid CSV."""

    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        required = {"unicode", "name", "source"}
        if set(reader.fieldnames or ()) != required:
            raise ValueError(f"expected CSV header unicode,name,source; got {reader.fieldnames}")
        rows = []
        seen = set()
        for line_number, row in enumerate(reader, start=2):
            raw = row["unicode"].strip().lower().removeprefix("u+")
            try:
                codepoint = int(raw, 16)
            except ValueError as error:
                raise ValueError(f"line {line_number}: invalid codepoint {raw!r}") from error
            if codepoint in seen:
                raise ValueError(f"line {line_number}: duplicate U+{codepoint:04X}")
            seen.add(codepoint)
            rows.append(InputRow(codepoint, row["name"].strip(), row["source"].strip()))
    return rows


def build_model(rows: Iterable[InputRow]) -> Model:
    """构建编码、shape 和别名模型。 / Build the code, shape, and alias model."""

    parsed_rows: list[tuple[InputRow, ParsedShape]] = []
    shape_to_rows: OrderedDict[str, list[InputRow]] = OrderedDict()
    for row in rows:
        parsed = parse_shape_name(row.name, row.codepoint, row.source)
        parsed_rows.append((row, parsed))
        if not parsed.is_control:
            shape_to_rows.setdefault(parsed.rust_name, []).append(row)

    alias_index: dict[int, int] = {}
    for shape_rows in shape_to_rows.values():
        for index, row in enumerate(shape_rows):
            alias_index[row.codepoint] = index

    codes: list[CodeEntry] = []
    shape_to_codes: OrderedDict[str, list[CodeEntry]] = OrderedDict()
    for row, parsed in parsed_rows:
        if parsed.is_control:
            const_name = parsed.rust_name
            shape_name = None
        else:
            index = alias_index[row.codepoint]
            const_name = parsed.rust_name if index == 0 else f"{parsed.rust_name}_ALT_{index}"
            shape_name = parsed.rust_name
        entry = CodeEntry(row.codepoint, const_name, shape_name, row.name, row.source)
        codes.append(entry)
        if shape_name is not None:
            shape_to_codes.setdefault(shape_name, []).append(entry)

    return Model(codes, list(shape_to_codes), shape_to_codes)


def _render_code_list(entries: list[CodeEntry]) -> str:
    return ", ".join(entry.const_name for entry in entries)


def render_codes_rust(model: Model, source_name: str) -> str:
    lines = [
        "// Generated by scripts/generate_zvvnmod_codes.py — DO NOT EDIT.",
        f"// Source: {source_name}",
        "",
        "/// A ZVVNMOD code value.",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]",
        "pub struct ZvvnmodCode(pub u32);",
        "",
        "impl ZvvnmodCode {",
        "    /// Return the Unicode code point.",
        "    pub const fn codepoint(self) -> u32 {",
        "        self.0",
        "    }",
        "    /// Convert to a Rust `char`.",
        "    pub fn as_char(self) -> Option<char> {",
        "        char::from_u32(self.0)",
        "    }",
        "}",
        "",
        "/// A merged ZVVNMOD written shape.",
        "#[allow(non_camel_case_types)]",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]",
        "pub enum ZvvnmodShape {",
    ]
    lines.extend(f"    {shape}," for shape in model.shapes)
    lines.extend(["}", ""])

    for entry in model.codes:
        comment = entry.source_name or entry.const_name
        lines.append(f"/// Code U+{entry.codepoint:04X}: {comment} ({entry.source}).")
        lines.append(f"pub const {entry.const_name}: ZvvnmodCode = ZvvnmodCode(0x{entry.codepoint:04X});")
    lines.append("")
    return "\n".join(lines)


def render_shape_map_rust(model: Model, source_name: str) -> str:
    lines = [
        "// Generated by scripts/generate_shape_map.py — DO NOT EDIT.",
        f"// Source: {source_name}",
        "",
        "use super::zvvnmod_codes::*;",
        "use std::collections::HashMap;",
        "",
    ]
    for shape, entries in model.shape_to_codes.items():
        lines.append(
            f"static {shape}_CODES: &[ZvvnmodCode] = &[{_render_code_list(entries)}];"
        )
    lines.extend(["", "/// Every named glyph code and its merged written shape.", "pub static CODE_TO_SHAPE: &[(ZvvnmodCode, ZvvnmodShape)] = &["])
    for entry in model.codes:
        if entry.shape_name is not None:
            lines.append(f"    ({entry.const_name}, ZvvnmodShape::{entry.shape_name}),")
    lines.extend(["];", ""])
    lines.extend([
        "/// Build Shape → all ZVVNMOD aliases; the first code is canonical.",
        "pub fn shape_to_zvvnmod_map() -> HashMap<ZvvnmodShape, &'static [ZvvnmodCode]> {",
        "    HashMap::from([",
    ])
    for shape in model.shapes:
        lines.append(f"        (ZvvnmodShape::{shape}, {shape}_CODES),")
    lines.extend(["    ])", "}", ""])
    return "\n".join(lines)


def generate_codes(input_path: Path, output_path: Path) -> Model:
    model = build_model(read_csv(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(render_codes_rust(model, input_path.name), encoding="utf-8")
    return model


def generate_shape_map(input_path: Path, output_path: Path) -> Model:
    model = build_model(read_csv(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(render_shape_map_rust(model, input_path.name), encoding="utf-8")
    return model


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=root / "data" / "zvvnmod-unicode-names.csv")
    parser.add_argument("--codes-output", type=Path, default=root / "src" / "generated" / "zvvnmod_codes.rs")
    parser.add_argument("--map-output", type=Path, default=root / "src" / "generated" / "shape_map.rs")
    args = parser.parse_args()
    model = generate_codes(args.input, args.codes_output)
    generate_shape_map(args.input, args.map_output)
    print(
        f"generated {len(model.codes)} codes, {len(model.shapes)} merged shapes -> "
        f"{args.codes_output}, {args.map_output}"
    )


if __name__ == "__main__":
    main()
