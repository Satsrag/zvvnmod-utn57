# zvvnmod-utn57

[English](README.md)

独立的 ZVVNMOD ↔ UTN #57 Rust 库。

当前第一步包含：

- 根据用户名称表进行可重复的代码生成；
- 生成 ZVVNMOD code 的语义化 Rust 常量；
- 生成 `merged ZVVNMOD code → component ZVVNMOD code sequence` 分解 Map；
- 从输入 stream 删除旧 FVS1/FVS2/FVS3/MVS；
- 生成 30 条用户确认的 `Ir_fina` 替换规则；
- 生成包含97个 UTN #57 targets与147条非空 reviewed rows（100条main + 47条particle）的 typed relation；
- 对一个 ZVVNMOD written-form run 执行 longest-match replacement。

已实现正向 reviewed written-unit mapping replacement，包括 reviewed particle sequence corrections与reviewed MVS targets。本次不猜测relation中未编码的通用ZWJ reconstruction、particle boundary、target serialization或反向转换结构规则。

## 目录

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
│   ├── check_website_contract.py
│   ├── generate_ir_fina.py
│   ├── generate_utn57_mapping.py
│   ├── generate_zvvnmod.py
│   ├── generate_zvvnmod_codes.py
│   ├── generate_code_decomposition_map.py
│   └── strict_csv.py
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

直接消费合并网站合同 `Satsrag/satsrag.github.io@0b50ba5b9f5c0ee66040ce6e8f343230b8832513` 中的CSV：

- `data/utn57-written-units.csv` 与网站target catalogue逐字节相同，定义97个typed UTN #57 targets，其中包括control `MVS`（`U+180E`）；锁定SHA-256为`2b924e3baeaab7582793585b5911a672037b05b5b65daa2771521839c3e088f6`。
- `data/zvvnmod-utn57-map.csv` 与网站下载artifact逐字节相同。文件先包含canonical metadata comment，随后是`id,sources,targets,note` CSV header与147条双边非空reviewed sequence relations：100条main mappings和全部47条particle mappings。锁定SHA-256为`cc58b012ea2e3a1709d723d115ad9eed00de13d32bba166991a1447c889a358c`；独立锁定的reviewed baseline为`sha256:83a60c3e1ac9df98a14c1a6d979f7c5c8733f1e70d52b81f41de1dd321ea5016`。

生成器验证canonical metadata、exact CSV schemas、row widths、quote transitions、单空格ordered sequences、稳定row ID语法与reviewed ambiguity set。`python3 scripts/check_website_contract.py --website-root ../satsrag-site-mapping-editor`读取网站已合并的Git blobs，证明byte identity，并把原样复制的bytes交给实际generator。

relation中的 ZVVNMOD source identifiers通过 `data/zvvnmod-unicode-names.csv` 解析；mapping CSV不再重复保存 source或target catalogues。

`convert_zvvnmod_run()` 依次执行：

1. 删除旧 U+E140–U+E143；
2. 执行 `Ir_fina` replacement；
3. 分解普通 merged codes，同时保留 reviewed chachlag forms；
4. 执行longest-match并保留全部equal-longest candidates；
5. 通过位置resolver和已注册的语义resolver选择candidate。

对于equal-longest位置candidates，target整体位置与source sequence理论位置一致的唯一
candidate是fallback。若normalized run中的实际matched span位置存在另一唯一candidate，
则覆盖fallback；若不存在，则继续使用fallback。Target sequence的整体位置由第一个
position-bearing unit的左边界和最后一个position-bearing unit的右边界共同决定；`MVS`、
Nirugu等control不承担边界。K/K2仍是独立的调用方语义选择。任何其他无法唯一选择的
family都返回typed `Utn57ConversionError::UnresolvedAmbiguity`，其中包含排序后的稳定
candidate row IDs；CSV顺序不会决定语义结果。

reviewed relation同时包含`AA_FINA → Aa:isol`和`AA_FINA → Aa:fina`。因此独占整个run的
`AA_FINA`选择`Aa:isol`，位于实际word尾部的matched span选择`Aa:fina`。更长的
`A_MEDI AA_FINA → Aa:fina`会先折叠该sequence，不让任一singleton candidate提前匹配。
十条reviewed chachlag relations仍通过longest match优先，输出各自的final/isolated onset，
随后输出`MVS + Aa:isol`；不推断额外的chachlag relation。ZVVNMOD的K/K2共用shape：
默认输出K；调用方拥有nominal/context信息时，可通过`Utn57ConversionOptions`显式选择K2。

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
