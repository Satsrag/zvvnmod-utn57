//! ZVVNMOD ↔ UTN #57 转换基础组件。
//!
//! ZVVNMOD ↔ UTN #57 conversion primitives.
//!
//! 当前包含自动生成的 ZVVNMOD code 定义，以及用于转换到 UTN #57
//! written units 的 merged-code → component-sequence Map。后续阶段将加入转换算法。
//!
//! The crate contains generated ZVVNMOD code definitions and a
//! merged-code → component-sequence map for conversion to UTN #57 written units.
//! Conversion algorithms will be added later.

pub mod generated {
    pub mod code_decomposition_map;
    pub mod zvvnmod_codes;
}

pub use generated::code_decomposition_map::*;
pub use generated::zvvnmod_codes::*;
