//! ZVVNMOD ↔ UTN #57 转换基础组件。
//!
//! ZVVNMOD ↔ UTN #57 conversion primitives.
//!
//! 当前包含自动生成的 ZVVNMOD code 定义，以及用于纠正分解输出的
//! code-sequence → merged-code Map。后续阶段将加入转换算法。
//!
//! The crate contains generated ZVVNMOD code definitions and a
//! code-sequence → merged-code map for correcting decomposed output.
//! Conversion algorithms will be added later.

pub mod generated {
    pub mod code_sequence_map;
    pub mod zvvnmod_codes;
}

pub use generated::code_sequence_map::*;
pub use generated::zvvnmod_codes::*;
