# zvvnmod-utn57

[简体中文](README.zh-CN.md)

A standalone Rust library for ZVVNMOD ↔ UTN #57 conversion.

The current first milestone includes:

- reproducible code generation from the user-supplied name table;
- semantic Rust constants for ZVVNMOD codes;
- a `merged ZVVNMOD code → component ZVVNMOD code sequence` decomposition map;
- legacy FVS1/FVS2/FVS3/MVS removal from input streams;
- 30 user-confirmed `Ir_fina` replacement rules;
- a typed, generated relation containing 96 UTN #57 targets and 138 non-empty reviewed mapping rows (91 main + 47 particle);
- executable longest-match replacement from one ZVVNMOD written-form run to UTN #57 written units.

Forward mapping replacement is implemented through the reviewed written-unit stage, including reviewed particle sequence corrections. Canonical MVS/ZWJ reconstruction, particle-boundary handling (including leading particle MVS), target serialization, and reverse conversion are not implemented in this change; the library does not guess structural rules that are absent from the relation.

## Layout

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README.zh-CN.md
├── data/
│   ├── ir-fina-replacements.csv
│   ├── utn57-written-units.csv
│   ├── zvvnmod-unicode-names.csv
│   └── zvvnmod-utn57-map.csv
├── scripts/
│   ├── generate_ir_fina.py
│   ├── generate_utn57_mapping.py
│   ├── generate_zvvnmod.py
│   ├── generate_zvvnmod_codes.py
│   └── generate_code_decomposition_map.py
├── src/
│   ├── lib.rs
│   ├── conversion.rs
│   ├── preprocess.rs
│   └── generated/
│       ├── code_decomposition_map.rs
│       ├── ir_fina.rs
│       ├── utn57_mapping.rs
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

The formal inventory contains only explicit ZVVNMOD shapes from the font. Legacy
FVS1/FVS2/FVS3/MVS values are not ZVVNMOD codes and are therefore not emitted as
Rust constants. `discard_legacy_controls()` removes U+E140 through U+E143 from an
input stream before `Ir_fina` replacement. Later mapping stages will reconstruct
required UTN #57 MVS units from ZVVNMOD writing-unit patterns.

## Legacy control removal

Legacy control values are discarded as the first conversion stage:

```text
[A_INIT, U+E140, A_MEDI, U+E143]
→ [A_INIT, A_MEDI]
```

The operation preserves all other codes and their order. It must run before
`replace_ir_fina()`.

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

## Reviewed mapping replacement

The reviewed website snapshot at `Satsrag/satsrag.github.io@966bd99943ab6dbd6846258491d0abd4caa689d9` is normalized into two non-duplicating authorities:

- `data/utn57-written-units.csv` defines the 96 typed UTN #57 targets; its locked SHA-256 is `a7635637c245f25144ee5d938a76c4dc83063953100bf7d7f8c61353826dfc26`.
- `data/zvvnmod-utn57-map.csv` contains 138 non-empty reviewed sequence relation rows: 91 main mappings plus all 47 particle mappings from `zvvnmod-utn57-particles.json` in that snapshot. Ordered `sources` and `targets` are space-delimited IDs within strict CSV fields. The particle source artifact SHA-256 is `e1ea535e8e40bd61e7b8e1beb9ec782a38a40e1bf8dda3d265dd6bcffabee09b`; the locked runtime relation SHA-256 is `5816e1d56e8b3fa7da7f2114562da463c4d449528aac3f9b73aade3afa157da0`.

ZVVNMOD source identifiers in the relation are resolved against `data/zvvnmod-unicode-names.csv`; source and target catalogues are not duplicated in the mapping CSV.

The generated relation preserves all reviewed non-empty rows. `convert_zvvnmod_run()`
applies these stages in order:

1. discard legacy U+E140–U+E143 values;
2. replace `Ir_fina` helpers;
3. decompose general merged codes while retaining reviewed chachlag forms;
4. apply the reviewed relation with longest-match.

The input is one connected written-form run. `AA_FINA` is therefore `Aa:isol` when
it is the complete run and `Aa:fina` after a connected form. ZVVNMOD uses the same
shapes for UTN K and K2: default conversion emits K, while callers with nominal or
other context can explicitly select K2 with `Utn57ConversionOptions`.

Nirugu is emitted as `Utn57Unit::Nirugu` with `Utn57Position::Control`; no
positional Nirugu form is invented.

## Generation

Generate code definitions, the decomposition map, and replacements separately:

```bash
python3 scripts/generate_zvvnmod_codes.py
python3 scripts/generate_code_decomposition_map.py
python3 scripts/generate_ir_fina.py
python3 scripts/generate_utn57_mapping.py
```

All outputs can also be generated at once:

```bash
python3 scripts/generate_zvvnmod.py
```

## Rust API

```rust
use zvvnmod_utn57::{
    convert_zvvnmod_run, convert_zvvnmod_run_with_options, Utn57ConversionOptions,
    Utn57KVariant, Utn57Position, Utn57Unit, Utn57WrittenUnit, B_I_INIT, K_INIT,
};

assert_eq!(
    convert_zvvnmod_run(&[B_I_INIT]).unwrap(),
    vec![
        Utn57WrittenUnit::new(Utn57Unit::B, Utn57Position::Init),
        Utn57WrittenUnit::new(Utn57Unit::I, Utn57Position::Medi),
    ],
);

let options = Utn57ConversionOptions { k_variant: Utn57KVariant::K2 };
assert_eq!(convert_zvvnmod_run_with_options(&[K_INIT], options).unwrap()[0].unit, Utn57Unit::K2);
```

## Validation

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE), matching the `meco` and `meco-rust` upstream projects.
