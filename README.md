# zvvnmod-utn57

[简体中文](README.zh-CN.md)

A standalone Rust library for ZVVNMOD ↔ UTN #57 conversion.

The current first milestone includes:

- reproducible code generation from the user-supplied name table;
- semantic Rust constants for ZVVNMOD codes;
- a `merged ZVVNMOD code → component ZVVNMOD code sequence` decomposition map;
- legacy FVS1/FVS2/FVS3/MVS removal from input streams;
- 30 user-confirmed `Ir_fina` replacement rules;
- a typed, generated relation containing 97 UTN #57 targets and 147 non-empty reviewed mapping rows (100 main + 47 particle);
- executable longest-match replacement from one ZVVNMOD written-form run to UTN #57 written units;
- a command bridge that invokes the published `mongol-norm==0.0.4` Python package for Unicode serialization.

Forward mapping replacement is implemented through the reviewed written-unit stage, including reviewed particle sequence corrections and reviewed MVS targets. The Rust mapping core does not reimplement Unicode serialization. The `zvvnmod-to-unicode` command starts Python once per conversion and delegates positioned-unit serialization to `mongol-norm`; reverse conversion and unreviewed structural inference remain out of scope.

## Layout

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README.zh-CN.md
├── requirements-mongol-norm.txt
├── data/
│   ├── ir-fina-replacements.csv
│   ├── utn57-written-units.csv
│   ├── zvvnmod-unicode-names.csv
│   └── zvvnmod-utn57-map.csv
├── scripts/
│   ├── check_website_contract.py
│   ├── install_mongol_norm.sh
│   ├── mongol_norm_positioned.py
│   ├── generate_ir_fina.py
│   ├── generate_utn57_mapping.py
│   ├── generate_zvvnmod.py
│   ├── generate_zvvnmod_codes.py
│   ├── generate_code_decomposition_map.py
│   └── strict_csv.py
├── src/
│   ├── lib.rs
│   ├── command_bridge.rs
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

The merged website contract at `Satsrag/satsrag.github.io@0b50ba5b9f5c0ee66040ce6e8f343230b8832513` is consumed directly as CSV:

- `data/utn57-written-units.csv` is byte-for-byte identical to the website target catalogue. It defines 97 typed UTN #57 targets, including `MVS` (`U+180E`) as a control; its locked SHA-256 is `2b924e3baeaab7582793585b5911a672037b05b5b65daa2771521839c3e088f6`.
- `data/zvvnmod-utn57-map.csv` is byte-for-byte identical to the website download artifact. It contains a canonical metadata comment followed by the `id,sources,targets,note` CSV header and 147 non-empty reviewed sequence relations: 100 main mappings plus all 47 particle mappings. Its locked SHA-256 is `cc58b012ea2e3a1709d723d115ad9eed00de13d32bba166991a1447c889a358c`; the independently locked reviewed baseline is `sha256:83a60c3e1ac9df98a14c1a6d979f7c5c8733f1e70d52b81f41de1dd321ea5016`.

The generator validates canonical metadata, exact CSV schemas, row widths, quote transitions, ordered single-space sequences, stable row-ID syntax, and the reviewed ambiguity set. `python3 scripts/check_website_contract.py --website-root ../satsrag-site-mapping-editor` reads the merged website Git blobs, proves byte identity, and runs those copied bytes through the actual generator.

ZVVNMOD source identifiers in the relation are resolved against `data/zvvnmod-unicode-names.csv`; source and target catalogues are not duplicated in the mapping CSV.

The generated relation preserves all reviewed non-empty rows. `convert_zvvnmod_run()`
applies these stages in order:

1. discard legacy U+E140–U+E143 values;
2. replace `Ir_fina` helpers;
3. decompose general merged codes while retaining reviewed chachlag forms;
4. apply longest-match, preserving every equal-longest candidate;
5. select a candidate with the position and registered semantic resolvers.

For equal-longest positional candidates, the unique target whose overall position
matches the source sequence's intrinsic position is the fallback. A unique target
matching the normalized run's actual matched-span position overrides it; if no
such target exists, the fallback remains selected. Target sequence position uses
the left edge of the first and right edge of the last position-bearing unit, so
controls such as `MVS` and Nirugu do not bear an edge. K/K2 remains a separate
caller-selected semantic ambiguity. Any other non-unique family returns typed
`Utn57ConversionError::UnresolvedAmbiguity` with sorted stable candidate row IDs;
CSV order never selects a semantic result.

The reviewed relation contains both `AA_FINA → Aa:isol` and
`AA_FINA → Aa:fina`. Consequently, `AA_FINA` alone selects `Aa:isol`, while a
matched final span selects `Aa:fina`. The longer
`A_MEDI AA_FINA → Aa:fina` relation collapses that sequence before either
singleton candidate can match. The ten reviewed chachlag relations still win by
longest match and emit their final/isolated onset followed by `MVS + Aa:isol`;
no additional chachlag relation is inferred. ZVVNMOD uses the same shapes for UTN
K and K2: default conversion emits K, while callers with nominal or other context
can explicitly select K2 with `Utn57ConversionOptions`.

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

## External `mongol-norm` command

The current Unicode endpoint uses an external Python command; it does not embed CPython or link
`libpython`. Install the exact reviewed package into the repository-local import directory:

```bash
scripts/install_mongol_norm.sh
```

Then invoke the command with one ZVVNMOD PUA string:

```bash
cargo run --bin zvvnmod-to-unicode -- '<zvvnmod-text>'
```

The Rust command maps ZVVNMOD to typed positioned units, sends protocol-versioned JSON to the
bundled Python bridge over stdin, and reads canonical Unicode from stdout. The bridge calls only
the public API:

```python
MongolianShaper("MNG").normalize_positioned_written_units(records)
```

The installer uses `pip --target`, needs neither root nor `python3-venv`, pins the 0.0.4 wheel by
SHA-256, stages and validates the installation before replacing the destination, and verifies the
singleton `O:init` result. The command starts Python in isolated mode, ignores the current directory
and inherited `PYTHONPATH`, and requires both package metadata and runtime `__version__` to equal
0.0.4. Set `ZVVNMOD_MONGOL_NORM_PATH` when using a non-default install directory or a binary built
outside this checkout. The installer and runtime both select a custom Python executable with
`ZVVNMOD_MONGOL_NORM_PYTHON`; the installer also retains `PYTHON` as a lower-priority fallback.

This subprocess bridge is deliberately simple and starts Python once per conversion command. A
conversion, including stdin/stdout/stderr collection, has a 30-second deadline, with stdout and
stderr capture limited to 1 MiB each. On Unix, the standard-library command integration places the
bridge in a dedicated process group; a small documented POSIX `kill` FFI call terminates the whole
group on timeout or pipeline error, including descendants whose direct parent already exited. No
Rust runtime dependency is added. The bridge is suitable for the current CLI integration; a
persistent worker can be added later if profiling shows process startup is a bottleneck.

## Validation

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test
scripts/install_mongol_norm.sh
cargo test --test command_bridge --test command_cli -- --ignored
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE), matching the `meco` and `meco-rust` upstream projects.
