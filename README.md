# zvvnmod-utn57

[简体中文](README.zh-CN.md)

A standalone Rust library for ZVVNMOD ↔ UTN #57 conversion.

The current first milestone includes:

- reproducible code generation from the user-supplied name table;
- semantic Rust constants for ZVVNMOD codes;
- a `merged ZVVNMOD code → component ZVVNMOD code sequence` decomposition map;
- FVS1/FVS2/FVS3/MVS control constants;
- 30 user-confirmed `Ir_fina` replacement rules.

The complete bidirectional conversion algorithm has not been added yet. `Ir_fina` replacement is the first conversion stage implemented in the crate.

## Layout

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README.zh-CN.md
├── data/
│   ├── ir-fina-replacements.csv
│   └── zvvnmod-unicode-names.csv
├── scripts/
│   ├── generate_ir_fina.py
│   ├── generate_zvvnmod.py
│   ├── generate_zvvnmod_codes.py
│   └── generate_code_decomposition_map.py
├── src/
│   ├── lib.rs
│   └── generated/
│       ├── code_decomposition_map.rs
│       ├── ir_fina.rs
│       └── zvvnmod_codes.rs
└── tests/
    ├── generated.rs
    └── test_generator.py
```

## Naming rules

CSV names consist of `written-unit + position` pairs.

```text
i    → INIT
m    → MEDI
f    → FINA
isol → ISOL
```

Single-shape examples:

```text
A i    → A_INIT
A m    → A_MEDI
Ir f   → IR_FINA
```

For a multi-part ZVVNMOD code, the unit names are merged.

```text
B i I f → B_I_ISOL
B i I m → B_I_INIT
B m I m → B_I_MEDI
B m I f → B_I_FINA
```

Overall position rules for a multi-part shape:

1. `i ... f` → `ISOL`
2. `i ... m` → `INIT`
3. `m ... m` → `MEDI`
4. `m ... f` → `FINA`
5. `f ... f` → `FINA`

`ZvvnmodCode` is the only generated Rust identity; there is no separate `ZvvnmodShape` object. A multi-part code such as `B_I_INIT` already identifies its glyph shape.

The generated decomposition Map uses a merged ZVVNMOD code as its key and its component code sequence as its value:

```text
B_I_INIT   → [B_INIT, I_MEDI]
B_I_MEDI   → [B_MEDI, I_MEDI]
G_O_I_INIT → [G_INIT, O_MEDI, I_MEDI]
```

This Map expands a merged ZVVNMOD code before conversion to UTN #57 written units. Its component-oriented output stays close to the UTN #57 representation. `Ir_fina` helper replacement must run before decomposition because it consumes the helper and changes the preceding merged code. If a required component code is absent from the CSV, no decomposition is invented.

The four control-table names and their generated Rust constants are:

```text
U+E140 → Fvs1 → FVS1
U+E141 → Fvs2 → FVS2
U+E142 → Fvs3 → FVS3
U+E143 → Mvs  → MVS
```

They are code constants and are not inserted into the decomposition map.

## `Ir_fina` replacement

`Ir_fina` is a ZVVNMOD helper with no standalone UTN written-unit counterpart. It indicates that the preceding ZVVNMOD form must be replaced with a specific final form. The replacement therefore runs before code decomposition and later written-form or UTN conversion: it consumes `IR_FINA` together with the preceding code instead of emitting `IR_FINA` as an independent unit.

Examples:

```text
O_MEDI + IR_FINA   → UE_FINA
T_MEDI + IR_FINA   → T_FINA
B_I_INIT + IR_FINA → B_I_ISOL
B_O_MEDI + IR_FINA → B_UE_FINA
```

The 30 authoritative rules are stored in `data/ir-fina-replacements.csv` with readable generated names such as `O_MEDI`, `IR_FINA`, and `UE_FINA`. The generator resolves and validates every name against the model derived from `data/zvvnmod-unicode-names.csv`; raw hexadecimal code references are not used in the replacement table.

`replace_ir_fina()` scans a complete code stream from left to right. A supported `preceding + IR_FINA` helper sequence is replaced with its specific final-form code. An unmatched `IR_FINA` returns `IrFinaReplacementError` instead of being silently retained or dropped.

## Generation

Generate code definitions, the decomposition map, and replacements separately:

```bash
python3 scripts/generate_zvvnmod_codes.py
python3 scripts/generate_code_decomposition_map.py
python3 scripts/generate_ir_fina.py
```

All outputs can also be generated at once:

```bash
python3 scripts/generate_zvvnmod.py
```

## Rust API

```rust
use zvvnmod_utn57::{
    replace_ir_fina, zvvnmod_code_decomposition_map, B_INIT, B_I_INIT,
    I_MEDI, IR_FINA, O_MEDI, UE_FINA,
};

let replaced = replace_ir_fina(&[O_MEDI, IR_FINA]).unwrap();
assert_eq!(replaced, vec![UE_FINA]);

let map = zvvnmod_code_decomposition_map();
assert_eq!(
    map.get(&B_I_INIT),
    Some(&[B_INIT, I_MEDI].as_slice()),
);
```

## Validation

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE), matching the `meco` and `meco-rust` upstream projects.
