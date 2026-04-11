//! # Aura Terminal UI Backend
//!
//! Runtime TUI backend that renders Aura apps directly in the terminal using
//! ratatui. No codegen step — the HIR is interpreted and rendered live, with
//! full state, input, and action support.
//!
//! ## HIR → Terminal Mapping
//! - Column → vertical Layout chunks
//! - Row → horizontal Layout chunks
//! - Text → Paragraph widget with styled spans
//! - Heading → Paragraph with bold modifier
//! - Button → Paragraph with bordered Block (clickable via focus + Enter)
//! - TextField → Paragraph with border; typed into when focused
//! - Checkbox/Toggle → interactive state binding
//! - Divider → horizontal rule block
//! - Spacer → Fill constraint
//! - Design tokens → ratatui Style (Color, Modifier, padding)
//!
//! ## Controls
//! - `Tab` / `Shift-Tab` — move focus between interactive elements
//! - `Enter` — activate focused button
//! - `Space` — toggle focused checkbox/toggle
//! - Type to edit focused text field; `Backspace` to delete
//! - `q` or `Esc` — quit

mod eval;
mod render;
mod runtime;
mod value;

pub use runtime::{TuiError, run_tui};
pub use value::Value;
