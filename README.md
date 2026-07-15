# zvvnmod-utn57

独立的 ZVVNMOD ↔ UTN #57 Rust 库。

A standalone Rust library for ZVVNMOD ↔ UTN #57 conversion.

当前第一步包含：

- 根据用户名称表进行可重复的代码生成；
- 生成 ZVVNMOD code 的语义化 Rust 常量；
- 将多个 written shape 合并为 `ZvvnmodShape`；
- 生成 `CODE_TO_SHAPE`；
- 生成 `Shape → 全部 ZVVNMOD aliases` Map；
- 生成 FVS1/FVS2/FVS3/MVS 控制常量。

The current first milestone includes:

- reproducible code generation from the user-supplied name table;
- semantic Rust constants for ZVVNMOD codes;
- merged `ZvvnmodShape` values for multi-part written shapes;
- `CODE_TO_SHAPE`;
- a `Shape → all ZVVNMOD aliases` map;
- FVS1/FVS2/FVS3/MVS control constants.

完整双向转换算法尚未加入。

The complete bidirectional conversion algorithm has not been added yet.

## 目录 / Layout

```text
.
├── Cargo.toml
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

## 命名规则 / Naming rules

CSV 名称由 `written-unit + position` 对组成。

CSV names consist of `written-unit + position` pairs.

```text
i    → INIT
m    → MEDI
f    → FINA
isol → ISOL
```

单 shape 示例 / Single-shape examples:

```text
A i    → A_INIT
A m    → A_MEDI
Ir f   → IR_FINA
```

多 shape 会合并 unit 名。

For a multi-part shape, the unit names are merged.

```text
B i I f → B_I_ISOL
B i I m → B_I_MEDI
B m I m → B_I_MEDI
B m I f → B_I_FINA
```

多 shape 的整体位置规则：

1. 第一项为 `i` 且末项为 `f` 时，整体为 `ISOL`；
2. 其他情况使用末项位置。

Overall position rules for a multi-part shape:

1. If the first item is `i` and the final item is `f`, the overall position is `ISOL`.
2. Otherwise, the final item's position is used.

同一个合并 shape 对应多个 code 时，全部保留。

When several codes represent the same merged shape, all aliases are retained.

```text
B_I_MEDI → [B_I_MEDI, B_I_MEDI_ALT_1]
```

CSV 中最先出现的 code 是 canonical，后续 code 使用 `_ALT_n`，不会静默覆盖。

The first code in CSV order is canonical. Later codes use `_ALT_n` and never silently overwrite an existing code.

四个 control-table 名称及生成的 Rust 常量为：

The four control-table names and their generated Rust constants are:

```text
U+E140 → Fvs1 → FVS1
U+E141 → Fvs2 → FVS2
U+E142 → Fvs3 → FVS3
U+E143 → Mvs  → MVS
```

它们是 code 常量，不进入 `ZvvnmodShape` Map。

They are code constants and are not inserted into the `ZvvnmodShape` map.

## 生成 / Generation

分别生成 code 定义和 Map：

Generate code definitions and the map separately:

```bash
python3 scripts/generate_zvvnmod_codes.py
python3 scripts/generate_shape_map.py
```

也可以一次生成两者：

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

`aliases[0]` 是 canonical ZVVNMOD code。

`aliases[0]` is the canonical ZVVNMOD code.

## 验证 / Validation

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test
```
