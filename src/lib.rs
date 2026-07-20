//! ZVVNMOD ↔ UTN #57 转换基础组件。
//!
//! ZVVNMOD ↔ UTN #57 conversion primitives.
//!
//! 当前包含显式字体 shape 的自动生成 ZVVNMOD code 定义、legacy control 删除、
//! `Ir_fina` replacement、merged-code decomposition，以及由 reviewed mapping JSON
//! 生成的 typed UTN #57 written-unit replacement。
//!
//! The crate contains generated ZVVNMOD code definitions for explicit font shapes,
//! legacy-control removal, `Ir_fina` replacement, merged-code decomposition, and
//! typed UTN #57 written-unit replacement generated from the reviewed mapping JSON.

pub mod generated {
    pub mod code_decomposition_map;
    pub mod ir_fina;
    pub mod utn57_mapping;
    pub mod zvvnmod_codes;
}
pub mod conversion;
pub mod preprocess;

pub use conversion::*;
pub use generated::code_decomposition_map::*;
pub use generated::ir_fina::*;
pub use generated::utn57_mapping::*;
pub use generated::zvvnmod_codes::*;
pub use preprocess::*;
