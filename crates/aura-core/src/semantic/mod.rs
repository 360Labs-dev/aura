//! # Aura Semantic Analysis
//!
//! The semantic analyzer performs:
//! 1. Name resolution — map identifiers to declarations
//! 2. Type inference and checking — infer types, verify compatibility
//! 3. Security type enforcement — E0200-E0299 (secret, sanitized, etc.)
//! 4. State mutation validation — E0300-E0399 (only in action blocks)
//! 5. Design token validation — E0400-E0499
//! 6. Error poisoning — suppress cascade errors from a single root cause
//!
//! Input: AST (from parser)
//! Output: Typed symbol table + errors

mod scope;

pub use scope::{AnalysisResult, SemanticAnalyzer};
