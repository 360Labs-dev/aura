//! # Aura Agent API
//!
//! Structured AST mutation protocol for AI coding agents.
//! Agents operate on the AST directly — no text editing.
//!
//! ## Operations
//! - `ast.get` — read the full AST
//! - `ast.query` — query specific nodes
//! - `ast.insert` — insert a new node
//! - `ast.modify` — modify an existing node
//! - `ast.delete` — delete a node
//! - `ast.batch` — atomic batch mutations
//!
//! ## Concurrency
//! - Optimistic concurrency with version numbers
//! - Rate limiting (100 mutations/sec/agent default)
//! - All mutations validated against type system before applying
//!
//! Phase 4 implementation.
