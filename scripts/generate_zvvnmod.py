#!/usr/bin/env python3
"""从已审核的 CSV 生成 Rust ZVVNMOD code 定义和 code decomposition Map。

Generate Rust ZVVNMOD code definitions and the code decomposition map from the reviewed CSV.
"""

from __future__ import annotations

import argparse
import csv
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
    """One authoritative CSV row resolved to its generated Rust constant."""

    codepoint: int
    const_name: str
    source_name: str
    source: str


@dataclass(frozen=True)
class CodeDecomposition:
    merged: CodeEntry
    components: tuple[CodeEntry, ...]


@dataclass(frozen=True)
class IrFinaReplacement:
    prefix: CodeEntry
    suffix: CodeEntry
    result: CodeEntry
    source: str


@dataclass
class Model:
    codes: list[CodeEntry]
    code_decompositions: list[CodeDecomposition]


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
    """构建 ZVVNMOD codes 和 decompositions。 / Build ZVVNMOD codes and decompositions."""

    parsed_rows: list[tuple[InputRow, ParsedCodeName]] = []
    for row in rows:
        parsed_rows.append((row, parse_code_name(row.name, row.codepoint, row.source)))

    codes: list[CodeEntry] = []
    codes_by_name: dict[str, CodeEntry] = {}
    code_by_value: dict[int, CodeEntry] = {}
    used_const_names: set[str] = set()
    for row, parsed in parsed_rows:
        const_name = parsed.rust_name
        if const_name in used_const_names:
            raise ValueError(f"duplicate generated code name {const_name}")
        used_const_names.add(const_name)
        entry = CodeEntry(row.codepoint, const_name, row.name, row.source)
        codes.append(entry)
        code_by_value[row.codepoint] = entry
        if not parsed.is_control:
            codes_by_name[const_name] = entry

    decompositions: list[CodeDecomposition] = []
    for row, parsed in parsed_rows:
        if parsed.is_control or len(parsed.component_names) <= 1:
            continue
        component_codes = [codes_by_name.get(name) for name in parsed.component_names]
        # CSV 中缺少 component code 时不补造 decomposition。
        # Do not invent a decomposition when a component code is absent from the CSV.
        if any(entry is None for entry in component_codes):
            continue
        components = tuple(entry for entry in component_codes if entry is not None)
        decompositions.append(
            CodeDecomposition(code_by_value[row.codepoint], components)
        )

    return Model(codes, decompositions)


def read_ir_fina_csv(path: Path, model: Model) -> list[IrFinaReplacement]:
    """读取并校验 Ir_fina 替换表。 / Read and validate the Ir_fina replacement table."""

    code_by_name = {entry.const_name: entry for entry in model.codes}
    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        required = {"prefix_name", "ir_fina_name", "result_name", "source"}
        if set(reader.fieldnames or ()) != required:
            raise ValueError(
                "expected Ir_fina CSV header "
                f"prefix_name,ir_fina_name,result_name,source; got {reader.fieldnames}"
            )

        rules: list[IrFinaReplacement] = []
        seen_pairs: set[tuple[str, str]] = set()
        for line_number, row in enumerate(reader, start=2):
            names = {
                column: row[column].strip()
                for column in ("prefix_name", "ir_fina_name", "result_name")
            }
            entries: dict[str, CodeEntry] = {}
            for column, name in names.items():
                try:
                    entries[column] = code_by_name[name]
                except KeyError as error:
                    raise ValueError(
                        f"line {line_number}: unknown generated {column} {name!r}"
                    ) from error

            pair = (names["prefix_name"], names["ir_fina_name"])
            if pair in seen_pairs:
                raise ValueError(
                    f"line {line_number}: duplicate Ir_fina replacement "
                    f"{pair[0]} + {pair[1]}"
                )
            seen_pairs.add(pair)

            suffix = entries["ir_fina_name"]
            if suffix.const_name != "IR_FINA":
                raise ValueError(
                    f"line {line_number}: expected IR_FINA suffix, got {suffix.const_name}"
                )
            source = row["source"].strip()
            if source != "user-confirmed":
                raise ValueError(f"line {line_number}: unsupported source {source!r}")
            rules.append(
                IrFinaReplacement(
                    entries["prefix_name"],
                    suffix,
                    entries["result_name"],
                    source,
                )
            )
    return rules


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


def render_code_decomposition_map_rust(model: Model, source_name: str) -> str:
    lines = [
        "// Generated by scripts/generate_code_decomposition_map.py — DO NOT EDIT.",
        f"// Source: {source_name}",
        "",
        "use super::zvvnmod_codes::*;",
        "use std::collections::HashMap;",
        "",
        "/// Merged ZVVNMOD codes and their component code sequences.",
        "pub static ZVVNMOD_CODE_DECOMPOSITIONS: &[(ZvvnmodCode, &[ZvvnmodCode])] = &[",
    ]
    for decomposition in model.code_decompositions:
        lines.append(
            f"    ({decomposition.merged.const_name}, "
            f"&[{_render_code_list(list(decomposition.components))}]),"
        )
    lines.extend(
        [
            "];",
            "",
            "/// Build merged ZVVNMOD code → component ZVVNMOD code sequence.",
            "pub fn zvvnmod_code_decomposition_map() -> HashMap<ZvvnmodCode, &'static [ZvvnmodCode]> {",
            "    ZVVNMOD_CODE_DECOMPOSITIONS.iter().copied().collect()",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def render_ir_fina_rust(
    rules: list[IrFinaReplacement], names_source: str, rules_source: str
) -> str:
    lines = [
        "// Generated by scripts/generate_ir_fina.py — DO NOT EDIT.",
        f"// Sources: {names_source}, {rules_source}",
        "",
        "use super::zvvnmod_codes::*;",
        "",
        "/// `Ir_fina` replacement rules as `(preceding code, result code)`.",
        "///",
        "/// `Ir_fina` is a ZVVNMOD helper with no standalone UTN counterpart. It changes",
        "/// the preceding form into a specific final form, so it must be consumed before",
        "/// later written-form or UTN conversion.",
        "pub static IR_FINA_REPLACEMENTS: &[(ZvvnmodCode, ZvvnmodCode)] = &[",
    ]
    for rule in rules:
        lines.append(
            f"    // {rule.prefix.const_name} + {rule.suffix.const_name} → "
            f"{rule.result.const_name} ({rule.source})"
        )
        lines.append(f"    ({rule.prefix.const_name}, {rule.result.const_name}),")
    lines.extend(
        [
            "];",
            "",
            "/// An `Ir_fina` helper without a supported preceding replacement.",
            "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
            "pub struct IrFinaReplacementError {",
            "    /// Index of `Ir_fina` in the input code stream.",
            "    pub index: usize,",
            "    /// Preceding code, or `None` when `Ir_fina` is the first code.",
            "    pub preceding: Option<ZvvnmodCode>,",
            "}",
            "",
            "impl std::fmt::Display for IrFinaReplacementError {",
            "    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {",
            "        match self.preceding {",
            "            Some(code) => write!(",
            "                formatter,",
            "                \"unmatched Ir_fina at index {} after U+{:04X}\",",
            "                self.index,",
            "                code.codepoint()",
            "            ),",
            "            None => write!(",
            "                formatter,",
            "                \"unmatched Ir_fina at index {} without a preceding code\",",
            "                self.index",
            "            ),",
            "        }",
            "    }",
            "}",
            "",
            "impl std::error::Error for IrFinaReplacementError {}",
            "",
            "/// Replace each `preceding + Ir_fina` helper sequence with its specific final form.",
            "///",
            "/// This replacement runs before decomposition, written-form, or UTN conversion",
            "/// because `Ir_fina` has no standalone UTN counterpart; it only modifies the",
            "/// preceding ZVVNMOD form.",
            "pub fn replace_ir_fina(input: &[ZvvnmodCode]) -> Result<Vec<ZvvnmodCode>, IrFinaReplacementError> {",
            "    let mut output = Vec::with_capacity(input.len());",
            "    for (index, &code) in input.iter().enumerate() {",
            "        if code != IR_FINA {",
            "            output.push(code);",
            "            continue;",
            "        }",
            "",
            "        let preceding = output.last().copied();",
            "        let Some(prefix) = preceding else {",
            "            return Err(IrFinaReplacementError { index, preceding });",
            "        };",
            "        let Some((_, result)) = IR_FINA_REPLACEMENTS",
            "            .iter()",
            "            .find(|(candidate, _)| *candidate == prefix)",
            "        else {",
            "            return Err(IrFinaReplacementError { index, preceding });",
            "        };",
            "        output.pop();",
            "        output.push(*result);",
            "    }",
            "    Ok(output)",
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


def generate_code_decomposition_map(input_path: Path, output_path: Path) -> Model:
    model = build_model(read_csv(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        render_code_decomposition_map_rust(model, input_path.name), encoding="utf-8"
    )
    return model


def generate_ir_fina(
    names_path: Path, rules_path: Path, output_path: Path
) -> list[IrFinaReplacement]:
    """生成 Ir_fina 替换 Rust API。 / Generate the Rust Ir_fina replacement API."""

    model = build_model(read_csv(names_path))
    rules = read_ir_fina_csv(rules_path, model)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        render_ir_fina_rust(rules, names_path.name, rules_path.name),
        encoding="utf-8",
    )
    return rules


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=root / "data" / "zvvnmod-unicode-names.csv")
    parser.add_argument("--codes-output", type=Path, default=root / "src" / "generated" / "zvvnmod_codes.rs")
    parser.add_argument(
        "--decomposition-map-output",
        type=Path,
        default=root / "src" / "generated" / "code_decomposition_map.rs",
    )
    parser.add_argument(
        "--ir-fina-input",
        type=Path,
        default=root / "data" / "ir-fina-replacements.csv",
    )
    parser.add_argument(
        "--ir-fina-output",
        type=Path,
        default=root / "src" / "generated" / "ir_fina.rs",
    )
    args = parser.parse_args()
    model = generate_codes(args.input, args.codes_output)
    generate_code_decomposition_map(args.input, args.decomposition_map_output)
    rules = generate_ir_fina(args.input, args.ir_fina_input, args.ir_fina_output)
    print(
        f"generated {len(model.codes)} codes, "
        f"{len(model.code_decompositions)} code decompositions, "
        f"{len(rules)} Ir_fina replacements -> "
        f"{args.codes_output}, {args.decomposition_map_output}, {args.ir_fina_output}"
    )


if __name__ == "__main__":
    main()
