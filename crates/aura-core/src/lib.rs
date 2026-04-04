//! # Aura Core
//!
//! The core compiler library for the Aura programming language.
//!
//! ## Pipeline
//! ```text
//! .aura source → Lexer → Tokens → Parser → AST → Semantic Analysis → HIR → LIR → Backend
//! ```

pub mod ast;
pub mod design;
pub mod diff;
pub mod errors;
pub mod explain;
pub mod hir;
pub mod lexer;
pub mod lir;
pub mod parser;
pub mod semantic;
pub mod types;

/// Re-export commonly used types.
pub use errors::{AuraError, ErrorCode, Fix, Severity};
pub use types::{AuraType, PrimitiveType, SecurityType};
