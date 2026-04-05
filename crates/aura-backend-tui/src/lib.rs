//! # Aura Terminal UI Backend
//!
//! Generates a self-contained Rust program that renders Aura apps in the terminal.
//! Uses raw ANSI escape codes — no external TUI framework dependency.
//!
//! ## HIR → Terminal Mapping
//! - Column → vertical newline-separated blocks
//! - Row → horizontal space-separated inline
//! - Text → stdout print with ANSI styling
//! - Heading → bold + large text
//! - Button → [Label] with highlight
//! - Divider → ─── line
//! - Design tokens → ANSI color codes

mod codegen;

pub use codegen::{TuiOutput, compile_to_tui};
