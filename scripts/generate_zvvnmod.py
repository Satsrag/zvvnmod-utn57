#!/usr/bin/env python3
"""从已审核的 CSV 生成 Rust ZVVNMOD code 定义和 sequence replacement Map。

Generate Rust ZVVNMOD code definitions and the sequence replacement map from the reviewed CSV.
"""

from __future__ import annotations

import argparse
import csv
from collections import OrderedDict
from dataclasses import dataclass
from itertools import product
from pathlib import Path
from typing import Iterable

POSITION_WORDS = {
    "i": ("INIT", "Init"),
    "m": ("MEDI", "Medi"),
    "f": ("FINA", "Fina"),
    "isol": ("ISOL", "Isol"),
}

@dataclass(frozen=True)
class ParsedCodeName:
    rust_name: str
    units: tuple[str, ...]
    position: str | None
    is_control: bool = False
    component_names: tuple[str, ...] = ()


@dataclass(frozen=True)
class InputRow:
    codepoint: int
    name: str
    source: str


@dataclass(frozen=True)
class CodeEntry:
    codepoint: int
    const_name: str
    semantic_name: str | None
    source_name: str
    source: str


@dataclass(frozen=True)
class SequenceReplacement:
    sequence: tuple[CodeEntry, ...]
    result: CodeEntry


@dataclass
class Model:
    codes: list[CodeEntry]
    sequence_replacements: list[SequenceReplacement]


def _unit_identifier(unit: str) -> str:
    """将 written-unit 名称转换为 Rust 标识符。 / Convert a written-unit name to a Rust identifier."""

    identifier = "".join(ch if ch.isalnum() else "_" for ch in unit).upper()
    identifier = "_".join(part for part in identifier.split("_") if part)
    if not identifier or identifier[0].isdigit():
        raise ValueError(f"invalid written-unit ID: {unit!r}")
    return identifier


def parse_code_name(
    name: str, codepoint: int | None = None, source: str = "font"
) -> ParsedCodeName:
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
        return ParsedCodeName(_unit_identifier(name), (), None, True)
    if source != "font":
        raise ValueError(f"unsupported source {source!r}")
    if not name:
        raise ValueError(f"missing name for U+{codepoint:04X}" if codepoint is not None else "missing name")
    if name == "Nirugu":
        return ParsedCodeName("NIRUGU", ("Nirugu",), None)

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
        edge_positions = (short_positions[0], short_positions[-1])
        edge_variants = {
            ("i", "f"): ("ISOL", "Isol"),
            ("i", "m"): ("INIT", "Init"),
            ("m", "m"): ("MEDI", "Medi"),
            ("m", "f"): ("FINA", "Fina"),
            ("f", "f"): ("FINA", "Fina"),
        }
        if (
            edge_positions not in edge_variants
            or any(position != "m" for position in short_positions[1:-1])
        ):
            raise ValueError(f"invalid multi-shape positions in {name!r}")
        position_suffix, position_variant = edge_variants[edge_positions]
    else:
        position_suffix, position_variant = POSITION_WORDS[short_positions[-1]]

    rust_units = "_".join(_unit_identifier(unit) for unit in units)
    component_names = tuple(
        f"{_unit_identifier(unit)}_{POSITION_WORDS[short_position][0]}"
        for unit, short_position in zip(units, short_positions)
    )
    return ParsedCodeName(
        rust_name=f"{rust_units}_{position_suffix}",
        units=tuple(units),
        position=position_variant,
        component_names=component_names,
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
    """构建 ZVVNMOD codes 和 sequence replacements。 / Build ZVVNMOD codes and sequence replacements."""

    parsed_rows: list[tuple[InputRow, ParsedCodeName]] = []
    name_to_rows: OrderedDict[str, list[InputRow]] = OrderedDict()
    for row in rows:
        parsed = parse_code_name(row.name, row.codepoint, row.source)
        parsed_rows.append((row, parsed))
        if not parsed.is_control:
            name_to_rows.setdefault(parsed.rust_name, []).append(row)

    alias_index: dict[int, int] = {}
    for named_rows in name_to_rows.values():
        for index, row in enumerate(named_rows):
            alias_index[row.codepoint] = index

    codes: list[CodeEntry] = []
    codes_by_name: OrderedDict[str, list[CodeEntry]] = OrderedDict()
    code_by_value: dict[int, CodeEntry] = {}
    for row, parsed in parsed_rows:
        if parsed.is_control:
            const_name = parsed.rust_name
            semantic_name = None
        else:
            index = alias_index[row.codepoint]
            const_name = parsed.rust_name if index == 0 else f"{parsed.rust_name}_ALT_{index}"
            semantic_name = parsed.rust_name
        entry = CodeEntry(row.codepoint, const_name, semantic_name, row.name, row.source)
        codes.append(entry)
        code_by_value[row.codepoint] = entry
        if semantic_name is not None:
            codes_by_name.setdefault(semantic_name, []).append(entry)

    replacements: list[SequenceReplacement] = []
    result_by_sequence: dict[tuple[int, ...], CodeEntry] = {}
    for row, parsed in parsed_rows:
        if parsed.is_control or len(parsed.component_names) <= 1:
            continue
        component_codes = [codes_by_name.get(name) for name in parsed.component_names]
        # CSV 中缺少 component code 时不补造 replacement。
        # Do not invent a replacement when a component code is absent from the CSV.
        if any(entries is None for entries in component_codes):
            continue
        available_codes = [entries for entries in component_codes if entries is not None]
        result = code_by_value[row.codepoint]
        for sequence in product(*available_codes):
            key = tuple(entry.codepoint for entry in sequence)
            existing = result_by_sequence.get(key)
            if existing is not None and existing != result:
                rendered = " + ".join(f"U+{value:04X}" for value in key)
                raise ValueError(
                    f"ambiguous sequence replacement {rendered}: "
                    f"{existing.const_name} vs {result.const_name}"
                )
            if existing is None:
                replacements.append(SequenceReplacement(sequence, result))
                result_by_sequence[key] = result

    return Model(codes, replacements)


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
    ]

    for entry in model.codes:
        comment = entry.source_name or entry.const_name
        lines.append(f"/// Code U+{entry.codepoint:04X}: {comment} ({entry.source}).")
        lines.append(f"pub const {entry.const_name}: ZvvnmodCode = ZvvnmodCode(0x{entry.codepoint:04X});")
    lines.append("")
    return "\n".join(lines)


def render_code_sequence_map_rust(model: Model, source_name: str) -> str:
    lines = [
        "// Generated by scripts/generate_code_sequence_map.py — DO NOT EDIT.",
        f"// Source: {source_name}",
        "",
        "use super::zvvnmod_codes::*;",
        "use std::collections::HashMap;",
        "",
        "/// Decomposed ZVVNMOD code sequences and their merged replacement code.",
        "pub static ZVVNMOD_SEQUENCE_REPLACEMENTS: &[(&[ZvvnmodCode], ZvvnmodCode)] = &[",
    ]
    for replacement in model.sequence_replacements:
        lines.append(
            f"    (&[{_render_code_list(list(replacement.sequence))}], "
            f"{replacement.result.const_name}),"
        )
    lines.extend(
        [
            "];",
            "",
            "/// Build decomposed ZVVNMOD code sequence → merged ZVVNMOD code.",
            "pub fn code_sequence_to_zvvnmod_map() -> HashMap<&'static [ZvvnmodCode], ZvvnmodCode> {",
            "    ZVVNMOD_SEQUENCE_REPLACEMENTS.iter().copied().collect()",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def generate_codes(input_path: Path, output_path: Path) -> Model:
    model = build_model(read_csv(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(render_codes_rust(model, input_path.name), encoding="utf-8")
    return model


def generate_code_sequence_map(input_path: Path, output_path: Path) -> Model:
    model = build_model(read_csv(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        render_code_sequence_map_rust(model, input_path.name), encoding="utf-8"
    )
    return model


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=root / "data" / "zvvnmod-unicode-names.csv")
    parser.add_argument("--codes-output", type=Path, default=root / "src" / "generated" / "zvvnmod_codes.rs")
    parser.add_argument(
        "--sequence-map-output",
        type=Path,
        default=root / "src" / "generated" / "code_sequence_map.rs",
    )
    args = parser.parse_args()
    model = generate_codes(args.input, args.codes_output)
    generate_code_sequence_map(args.input, args.sequence_map_output)
    print(
        f"generated {len(model.codes)} codes, "
        f"{len(model.sequence_replacements)} sequence replacements -> "
        f"{args.codes_output}, {args.sequence_map_output}"
    )


if __name__ == "__main__":
    main()
