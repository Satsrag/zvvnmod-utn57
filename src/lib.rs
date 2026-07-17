//! ZVVNMOD ↔ UTN #57 转换基础组件。
//!
//! ZVVNMOD ↔ UTN #57 conversion primitives.
//!
//! 当前包含自动生成的 ZVVNMOD code 定义、用于转换到 UTN #57
//! written units 的 merged-code → component-sequence Map，以及必须在该分解前执行的
//! `Ir_fina` helper 替换。
//!
//! The crate contains generated ZVVNMOD code definitions, a merged-code →
//! component-sequence map for conversion to UTN #57 written units, and `Ir_fina`
//! helper replacement that must run before decomposition.

pub mod generated {
    pub mod code_decomposition_map;
    pub mod ir_fina;
    pub mod zvvnmod_codes;
}

pub use generated::code_decomposition_map::*;
pub use generated::ir_fina::*;
pub use generated::zvvnmod_codes::*;
