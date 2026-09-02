//! ZVVNMOD ↔ UTN #57 conversion primitives.
//!
//! The crate contains generated ZVVNMOD code definitions for explicit font shapes,
//! legacy-control removal, `Ir_fina` replacement, merged-code decomposition, and
//! typed UTN #57 written-unit replacement generated directly from the reviewed runtime CSV,
//! and canonical normalization driven by the pure-Rust `mongol-norm` crate.
//!
//! Normalization errors are `mongol-norm`'s own [`mongol_norm::Error`]. The crate is re-exported
//! so a caller can match its variants without declaring the dependency separately.
//!
//! ZVVNMOD ↔ UTN #57 转换基础组件。
//!
//! 当前包含显式字体 shape 的自动生成 ZVVNMOD code 定义、legacy control 删除、
//! `Ir_fina` replacement、merged-code decomposition，以及由 reviewed runtime CSV
//! 生成的 typed UTN #57 written-unit replacement，以及由纯 Rust `mongol-norm`
//! crate 驱动的 canonical 归一化。

pub mod generated {
    pub mod code_decomposition_map;
    pub mod ir_fina;
    pub mod utn57_mapping;
    pub mod zvvnmod_codes;
}
pub mod api;
pub mod conversion;
pub mod normalize;
pub mod preprocess;
pub mod text;

pub use api::*;
pub use conversion::*;
pub use generated::code_decomposition_map::*;
pub use generated::ir_fina::*;
pub use generated::utn57_mapping::*;
pub use generated::zvvnmod_codes::*;
pub use normalize::*;
pub use preprocess::*;
pub use text::*;

/// The normalization backend, re-exported so callers can match [`mongol_norm::Error`]
/// and build shapers of their own without adding the dependency themselves.
pub use mongol_norm;
