//! # Aura Core
//!
//! The core compiler library for the Aura programming language.
//!
//! ## Pipeline
//! ```text
//! .aura source → Lexer → Tokens → Parser → AST → Semantic Analysis → HIR → LIR → Backend
//! ```

pub mod ast;
pub mod cache;
pub mod declarations;
pub mod design;
pub mod diff;
pub mod errors;
pub mod explain;
pub mod fmt;
pub mod hir;
pub mod lexer;
pub mod lir;
pub mod parser;
pub mod project;
pub mod semantic;
pub mod sketch;
pub mod sourcemap;
pub mod treeshake;
pub mod types;

/// Re-export commonly used types.
pub use errors::{AuraError, ErrorCode, Fix, Severity};
pub use types::{AuraType, PrimitiveType, SecurityType};
