# zvvnmod-utn57

[English](README.md)

独立的 ZVVNMOD ↔ UTN #57 Rust 库。

当前第一步包含：

- 根据用户名称表进行可重复的代码生成；
- 生成 ZVVNMOD code 的语义化 Rust 常量；
- 生成 `merged ZVVNMOD code → component ZVVNMOD code sequence` 分解 Map；
- 从输入 stream 删除旧 FVS1/FVS2/FVS3/MVS/FVS4；
- 生成 30 条用户确认的 `Ir_fina` 替换规则；
- 生成包含97个 positioned UTN #57 written units与147条非空 reviewed rows（100条main + 47条particle）的 typed relation；
- 对一个 ZVVNMOD shape run 执行 longest-match replacement，得到 positioned UTN #57 written units；
- 编排完整文本：只转换正式 ZVVNMOD shape codes，把 Nirugu/MVS 作为结构输入处理，输入 ZWJ 与其余字符全部原样保留；
- 通过纯 Rust 的 [`mongol-norm`](https://crates.io/crates/mongol-norm) crate 把 positioned UTN #57 written units 规范化为 canonical Unicode，直接链接进库中。

数据流为：

```text
包含 ZVVNMOD shape 的完整文本
→ 分类正式 ZVVNMOD runs 与 passthrough spans
→ 每个 ZVVNMOD run 直接转成 `Utn57PositionedWrittenUnit`
→ mongol-norm 在进程内分别规范化每个 run
→ 按原边界交错补回 passthrough spans
```

passthrough 文本不进入 normalizer；本库保存其原 code point 与 source boundary，并在
normalization 后补回。全程在进程内完成：没有子进程、没有解释器，运行时不访问文件系统和网络，
因此 Rust 能跑的地方本库都能跑，包括 `wasm32-unknown-unknown`。反向转换和未 reviewed
的结构推断仍不在当前范围内。

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
│   ├── normalize.rs
│   ├── conversion.rs
│   ├── preprocess.rs
│   ├── text.rs
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

正式 inventory 只包含来自字体的显式 ZVVNMOD shapes。旧 FVS1/FVS2/FVS3/MVS/FVS4
值不是 ZVVNMOD codes，因此不生成对应 Rust 常量。`discard_legacy_controls()`
在 `Ir_fina` replacement 前从输入 stream 删除 U+E140 至 U+E144；后续 mapping
阶段再根据 ZVVNMOD written-unit patterns 重建所需的 UTN #57 MVS units。

## 删除旧 controls

旧 control 值在转换第一阶段被删除：

```text
[A_INIT, U+E140, A_MEDI, U+E144]
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

- `data/utn57-written-units.csv` 与网站同名 catalogue 逐字节相同，定义97个 positioned UTN #57 written units，其中包括control `MVS`（`U+180E`）；锁定SHA-256为`2b924e3baeaab7582793585b5911a672037b05b5b65daa2771521839c3e088f6`。
- `data/zvvnmod-utn57-map.csv` 与网站下载artifact逐字节相同。文件先包含canonical metadata comment，随后是`id,sources,targets,note` CSV header与147条双边非空reviewed sequence relations：100条main mappings和全部47条particle mappings。锁定SHA-256为`cc58b012ea2e3a1709d723d115ad9eed00de13d32bba166991a1447c889a358c`；独立锁定的reviewed baseline为`sha256:83a60c3e1ac9df98a14c1a6d979f7c5c8733f1e70d52b81f41de1dd321ea5016`。

生成器验证canonical metadata、exact CSV schemas、row widths、quote transitions、单空格ordered sequences、稳定row ID语法与reviewed ambiguity set。`python3 scripts/check_website_contract.py --website-root ../satsrag-site-mapping-editor`读取网站已合并的Git blobs，证明byte identity，并把原样复制的bytes交给实际generator。

relation中的 ZVVNMOD source identifiers通过 `data/zvvnmod-unicode-names.csv` 解析；mapping CSV不再重复保存 source或target catalogues。

`convert_zvvnmod_run()` 依次执行：

1. 删除旧 U+E140–U+E144；
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

Nirugu 输出为 `Utn57WrittenUnit::Nirugu` + `Utn57Position::Control`，不会虚构 positional Nirugu。

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
    Utn57KVariant, Utn57Position, Utn57WrittenUnit, Utn57PositionedWrittenUnit, B_I_INIT, K_INIT,
};

assert_eq!(
    convert_zvvnmod_run(&[B_I_INIT]).unwrap(),
    vec![
        Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::B, Utn57Position::Init),
        Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::I, Utn57Position::Medi),
    ],
);

let options = Utn57ConversionOptions { k_variant: Utn57KVariant::K2 };
assert_eq!(convert_zvvnmod_run_with_options(&[K_INIT], options).unwrap()[0].written_unit, Utn57WrittenUnit::K2);
```

完整文本转换由 Rust 调用方和 CLI 共用同一个 API：

```rust
use zvvnmod_utn57::convert_zvvnmod_to_utn57;

let output = convert_zvvnmod_to_utn57(zvvnmod_text)?;
```

`convert_zvvnmod_to_utn57` 是稳定的公共边界。它通过下面说明的 `mongol-norm` crate 完成
normalization，不需要任何安装步骤。

完整文本分类器只转换139个正式 ZVVNMOD shape codes，其中包含 ZVVNMOD 自己的 Nirugu 编码。
标准 Unicode `U+180A` Nirugu、`U+180E` MVS、`U+202F` NNBSP 和输入 `U+200D` ZWJ
都没有 ZVVNMOD shaping 语义：本库原样保留它们，并以它们分隔相邻 shape runs。
`U+202F` 的 suffix 专用语义明确暂缓；本次不新增 `U+202F` mapping，也不把它加入正式 inventory。
normalizer 在编码 positioned written units 时可能自行输出 ZWJ；
该后端输出同样原样保留，不会拿输入 ZWJ 去替换或去重。正式 shape inventory 之外
的所有字符——包括 Unicode 标点、数字、空白、普通混合文本、emoji 和非 ZVVNMOD PUA——
都按原 code point 和原顺序保留。MenkShape `U+E23F..=U+E242` 等 source-specific aliases
由上游 source converter 处理，本库不解释其语义。旧 ZVVNMOD `U+E140..=U+E144`
FVS1-FVS4/MVS 控制码明确排除。

## `mongol-norm` normalization 后端

Normalization 使用纯 Rust 的 [`mongol-norm`](https://crates.io/crates/mongol-norm) crate，
作为普通 Cargo dependency 链接。它自身零依赖，不需要 Python，也没有要安装的数据文件：

```toml
[dependencies]
zvvnmod-utn57 = "0.1.0-alpha.4"
```

```bash
cargo install zvvnmod-utn57 --version 0.1.0-alpha.4
zvvnmod-to-utn57 '<zvvnmod-text>'
```

`src/normalize.rs` 把每个 reviewed `Utn57PositionedWrittenUnit` 映射到后端的
`PositionedWrittenUnit`，然后调用：

```rust
Shaper::new(Locale::Mng).normalize_positioned_written_units(&records)
```

unit 和 position 的映射写成穷尽 `match`，而不是走 `contract_name` 查表：
`src/generated/utn57_mapping.rs` 是生成代码，所以生成器新增 written unit 时会让本 crate
**编译失败**，而不是只在运行时报错。`MNG` shaper 由 `OnceLock` 每进程构建一次，所有 run 共用。

后端的错误变体本身就是有用信息，因此不做包装。`Utn57TextConversionError::Normalize`
原样携带 `mongol_norm::Error`，并且本 crate 重导出了 `mongol_norm`，调用方无需单独声明
依赖即可 match `UnsupportedPositionedUnit`、`ChainPositionMismatch` 等具体变体：

```rust
use zvvnmod_utn57::{convert_zvvnmod_to_utn57, mongol_norm, Utn57TextConversionError};

match convert_zvvnmod_to_utn57(text) {
    Ok(output) => println!("{output}"),
    Err(Utn57TextConversionError::Normalize(mongol_norm::Error::ChainPositionMismatch)) => { /* … */ }
    Err(error) => eprintln!("{error}"),
}
```

Cargo 语义中每个 `0.0.x` 版本都与下一个不兼容，所以 `mongol-norm = "0.0.4"` 本身就是精确锁定，
与它替换掉的 hash-locked wheel 意图一致。

## 验证

```bash
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo check --target wasm32-unknown-unknown --lib
cargo package
```

这里的 Python 只是 `scripts/` 下的构建期生成器套件；库本身运行时不含 Python。
没有任何测试带 `#[ignore]`，也没有任何测试需要先跑安装步骤。

## 许可证

本项目使用 [Apache License 2.0](LICENSE)，与上游 `meco` 和 `meco-rust` 保持一致。
