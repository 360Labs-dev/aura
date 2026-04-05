//! # Aura Web Backend
//!
//! Generates HTML/CSS/JavaScript from Aura HIR.
//!
//! ## Output Structure
//! - `index.html` — App shell with component rendering
//! - `styles.css` — Design tokens as CSS custom properties + component styles
//! - `app.js` — Reactive state management + event handlers

mod codegen;

pub use codegen::{WebOutput, compile_to_web};
