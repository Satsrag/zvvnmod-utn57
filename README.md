# zvvnmod-utn57

[简体中文](README.zh-CN.md)

A standalone Rust library for ZVVNMOD ↔ UTN #57 conversion.

The current first milestone includes:

- reproducible code generation from the user-supplied name table;
- semantic Rust constants for ZVVNMOD codes;
- a `merged ZVVNMOD code → component ZVVNMOD code sequence` decomposition map;
- FVS1/FVS2/FVS3/MVS control constants.

The complete bidirectional conversion algorithm has not been added yet.

## Layout

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README.zh-CN.md
├── data/
│   └── zvvnmod-unicode-names.csv
├── scripts/
│   ├── generate_zvvnmod.py
│   ├── generate_zvvnmod_codes.py
│   └── generate_code_decomposition_map.py
├── src/
│   ├── lib.rs
│   └── generated/
│       ├── zvvnmod_codes.rs
│       └── code_decomposition_map.rs
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

## Generation

Generate code definitions and the map separately:

```bash
python3 scripts/generate_zvvnmod_codes.py
python3 scripts/generate_code_decomposition_map.py
```

Both outputs can also be generated at once:

```bash
python3 scripts/generate_zvvnmod.py
```

## Rust API

```rust
use zvvnmod_utn57::{
    zvvnmod_code_decomposition_map, B_INIT, B_I_INIT, I_MEDI,
};

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
