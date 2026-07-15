//! ZVVNMOD ↔ UTN #57 转换基础组件。
//!
//! ZVVNMOD ↔ UTN #57 conversion primitives.
//!
//! 当前第一阶段包含自动生成的 ZVVNMOD 编码名称，以及合并后的
//! shape-to-code 别名映射。后续阶段将加入转换算法。
//!
//! The first milestone contains generated ZVVNMOD code names and merged
//! shape-to-code aliases. Conversion algorithms will be added in later steps.

pub mod generated {
    pub mod shape_map;
    pub mod zvvnmod_codes;
}

pub use generated::shape_map::*;
pub use generated::zvvnmod_codes::*;
