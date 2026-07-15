//! ZVVNMOD ↔ UTN #57 conversion primitives.
//!
//! The first milestone contains generated ZVVNMOD code names and merged
//! shape-to-code aliases. Conversion algorithms will be added in later steps.

pub mod generated {
    pub mod shape_map;
    pub mod zvvnmod_codes;
}

pub use generated::shape_map::*;
pub use generated::zvvnmod_codes::*;
