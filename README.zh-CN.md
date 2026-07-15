# zvvnmod-utn57

[English](README.md)

独立的 ZVVNMOD ↔ UTN #57 Rust 库。

当前第一步包含：

- 根据用户名称表进行可重复的代码生成；
- 生成 ZVVNMOD code 的语义化 Rust 常量；
- 将多个 written shape 合并为 `ZvvnmodShape`；
- 生成 `CODE_TO_SHAPE`；
- 生成 `Shape → 全部 ZVVNMOD aliases` Map；
- 生成 FVS1/FVS2/FVS3/MVS 控制常量。

完整双向转换算法尚未加入。

## 目录

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

## 命名规则

CSV 名称由 `written-unit + position` 对组成。

```text
i    → INIT
m    → MEDI
f    → FINA
isol → ISOL
```

单 shape 示例：

```text
A i    → A_INIT
A m    → A_MEDI
Ir f   → IR_FINA
```

多 shape 会合并 unit 名。

```text
B i I f → B_I_ISOL
B i I m → B_I_MEDI
B m I m → B_I_MEDI
B m I f → B_I_FINA
```

多 shape 的整体位置规则：

1. 第一项为 `i` 且末项为 `f` 时，整体为 `ISOL`；
2. 其他情况使用末项位置。

同一个合并 shape 对应多个 code 时，全部保留。

```text
B_I_MEDI → [B_I_MEDI, B_I_MEDI_ALT_1]
```

CSV 中最先出现的 code 是 canonical，后续 code 使用 `_ALT_n`，不会静默覆盖。

四个 control-table 名称及生成的 Rust 常量为：

```text
U+E140 → Fvs1 → FVS1
U+E141 → Fvs2 → FVS2
U+E142 → Fvs3 → FVS3
U+E143 → Mvs  → MVS
```

它们是 code 常量，不进入 `ZvvnmodShape` Map。

## 生成

分别生成 code 定义和 Map：

```bash
python3 scripts/generate_zvvnmod_codes.py
python3 scripts/generate_shape_map.py
```

也可以一次生成两者：

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

## 验证

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test
```

## 许可证

本项目使用 [Apache License 2.0](LICENSE)，与上游 `meco` 和 `meco-rust` 保持一致。
