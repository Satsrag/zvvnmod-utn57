# zvvnmod-utn57

[English](README.md)

独立的 ZVVNMOD ↔ UTN #57 Rust 库。

当前第一步包含：

- 根据用户名称表进行可重复的代码生成；
- 生成 ZVVNMOD code 的语义化 Rust 常量；
- 生成 `merged ZVVNMOD code → component ZVVNMOD code sequence` 分解 Map；
- 从输入 stream 删除旧 FVS1/FVS2/FVS3/MVS；
- 生成 30 条用户确认的 `Ir_fina` 替换规则；
- 生成包含 96 个 UTN #57 targets 与 91 条非空 reviewed rows 的 typed relation；
- 对一个 ZVVNMOD written-form run 执行 longest-match replacement。

已实现正向 reviewed written-unit mapping replacement。本次不猜测 main mapping 尚未编码的 canonical MVS/ZWJ reconstruction、particle boundary、target serialization 或反向转换规则。

## 目录

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README.zh-CN.md
├── data/
│   ├── ir-fina-replacements.csv
│   ├── zvvnmod-unicode-names.csv
│   └── zvvnmod-utn57-map.json
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

multi-part ZVVNMOD code 会合并 unit 名。

```text
B i I f → B_I_ISOL
B i I m → B_I_INIT
B m I m → B_I_MEDI
B m I f → B_I_FINA
```

多 shape 的整体位置规则：

1. `i ... f` → `ISOL`；
2. `i ... m` → `INIT`；
3. `m ... m` → `MEDI`；
4. `m ... f` → `FINA`；
5. `f ... f` → `FINA`。

`ZvvnmodCode` 是唯一生成的 Rust identity；不再定义独立的 `ZvvnmodShape` 对象。`B_I_INIT` 这样的 multi-part code 本身已经表示它的字形。

生成的分解 Map 以 merged ZVVNMOD code 为 key，以 component code sequence 为 value：

```text
B_I_INIT   → [B_INIT, I_MEDI]
B_I_MEDI   → [B_MEDI, I_MEDI]
G_O_I_INIT → [G_INIT, O_MEDI, I_MEDI]
```

该 Map 在转换到 UTN #57 written units 之前展开 merged ZVVNMOD code；component-oriented 输出更接近 UTN #57 表示。`Ir_fina` helper replacement 必须先执行，因为它会消耗 helper 并修改前一个 merged code。CSV 缺少必要 component code 时，不补造 decomposition。

正式 inventory 只包含来自字体的显式 ZVVNMOD shapes。旧 FVS1/FVS2/FVS3/MVS
值不是 ZVVNMOD codes，因此不生成对应 Rust 常量。`discard_legacy_controls()`
在 `Ir_fina` replacement 前从输入 stream 删除 U+E140 至 U+E143；后续 mapping
阶段再根据 ZVVNMOD writing-unit patterns 重建所需的 UTN #57 MVS units。

## 删除旧 controls

旧 control 值在转换第一阶段被删除：

```text
[A_INIT, U+E140, A_MEDI, U+E143]
→ [A_INIT, A_MEDI]
```

该操作保留其他所有 codes 及其顺序，并且必须在 `replace_ir_fina()` 前执行。

## `Ir_fina` 替换

`Ir_fina` 是 ZVVNMOD helper，没有可独立输出的 UTN written unit。它表示必须把前一个 ZVVNMOD form 替换为特定 final form。因此该替换必须发生在 code decomposition 以及后续 written-form 或 UTN 转换之前：`IR_FINA` 会与前一个 code 一起被消费，而不会作为独立 unit 输出。

示例：

```text
O_MEDI + IR_FINA   → UE_FINA
T_MEDI + IR_FINA   → T_FINA
B_I_INIT + IR_FINA → B_I_ISOL
B_O_MEDI + IR_FINA → B_UE_FINA
```

30 条权威规则保存在 `data/ir-fina-replacements.csv`，使用 `O_MEDI`、`IR_FINA`、`UE_FINA` 这类可读的生成名称。生成器根据 `data/zvvnmod-unicode-names.csv` 派生的 model 解析并验证每个名称；替换表不使用原始十六进制 code reference。

`replace_ir_fina()` 从左到右扫描完整 code stream。支持的 `preceding + IR_FINA` helper sequence 会替换为特定 final-form code；无法匹配的 `IR_FINA` 返回 `IrFinaReplacementError`，不会静默保留或删除。

## Reviewed mapping replacement

`data/zvvnmod-utn57-map.json` byte-for-byte 来自
`Satsrag/satsrag.github.io@966bd99943ab6dbd6846258491d0abd4caa689d9` 的
`mapping/data/zvvnmod-utn57-map.json`；锁定 SHA-256 为
`10dc660f941f28bb16671bce9c4ae9bf4df4f1b8a62f52ba6e51aada6ff612b2`。

`convert_zvvnmod_run()` 依次执行：

1. 删除旧 U+E140–U+E143；
2. 执行 `Ir_fina` replacement；
3. 分解普通 merged codes，同时保留 reviewed chachlag forms；
4. 对 reviewed relation 执行 longest-match。

输入表示一个 connected written-form run。因此，完整 run 只有 `AA_FINA` 时输出
`Aa:isol`；接在连接 form 后时输出 `Aa:fina`。ZVVNMOD 的 K/K2 共用 shape：
默认输出 K；调用方拥有 nominal/context 信息时，可通过 `Utn57ConversionOptions`
显式选择 K2。

Nirugu 输出为 `Utn57Unit::Nirugu` + `Utn57Position::Control`，不会虚构 positional Nirugu。

## 生成

分别生成 code 定义、decomposition Map 和 replacement：

```bash
python3 scripts/generate_zvvnmod_codes.py
python3 scripts/generate_code_decomposition_map.py
python3 scripts/generate_ir_fina.py
python3 scripts/generate_utn57_mapping.py
```

也可以一次生成全部输出：

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

## 验证

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test
```

## 许可证

本项目使用 [Apache License 2.0](LICENSE)，与上游 `meco` 和 `meco-rust` 保持一致。
