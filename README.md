# zvvnmod-utn57

[简体中文](README.zh-CN.md)

A standalone Rust library for ZVVNMOD ↔ UTN #57 conversion.

The current first milestone includes:

- reproducible code generation from the user-supplied name table;
- semantic Rust constants for ZVVNMOD codes;
- merged `ZvvnmodShape` values for multi-part written shapes;
- `CODE_TO_SHAPE`;
- a `Shape → all ZVVNMOD aliases` map;
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
│   └── generate_shape_map.py
├── src/
│   ├── lib.rs
│   └── generated/
│       ├── zvvnmod_codes.rs
│       └── shape_map.rs
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

For a multi-part shape, the unit names are merged.

```text
B i I f → B_I_ISOL
B i I m → B_I_MEDI
B m I m → B_I_MEDI
B m I f → B_I_FINA
```

Overall position rules for a multi-part shape:

1. If the first item is `i` and the final item is `f`, the overall position is `ISOL`.
2. Otherwise, the final item's position is used.

When several codes represent the same merged shape, all aliases are retained.

```text
B_I_MEDI → [B_I_MEDI, B_I_MEDI_ALT_1]
```

The first code in CSV order is canonical. Later codes use `_ALT_n` and never silently overwrite an existing code.

The four control-table names and their generated Rust constants are:

```text
U+E140 → Fvs1 → FVS1
U+E141 → Fvs2 → FVS2
U+E142 → Fvs3 → FVS3
U+E143 → Mvs  → MVS
```

They are code constants and are not inserted into the `ZvvnmodShape` map.

## Generation

Generate code definitions and the map separately:

```bash
python3 scripts/generate_zvvnmod_codes.py
python3 scripts/generate_shape_map.py
```

Both outputs can also be generated at once:

```bash
python3 scripts/generate_zvvnmod.py
```

## Rust API

```rust
use zvvnmod_utn57::{shape_to_zvvnmod_map, ZvvnmodShape};

let map = shape_to_zvvnmod_map();
let aliases = map[&ZvvnmodShape::B_I_MEDI];
```

`aliases[0]` is the canonical ZVVNMOD code.

## Validation

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE), matching the `meco` and `meco-rust` upstream projects.
