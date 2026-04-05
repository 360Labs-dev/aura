//! # Aura Jetpack Compose Backend
//!
//! Generates Kotlin/Jetpack Compose code from Aura HIR.
//!
//! ## HIR → Compose Mapping
//! - HIRColumn → Column
//! - HIRRow → Row
//! - HIRStack → Box
//! - HIRText → Text
//! - HIRButton → Button / IconButton
//! - HIRTextField → OutlinedTextField
//! - HIRToggle → Switch
//! - HIRList/Each → LazyColumn / items
//! - HIRState → remember { mutableStateOf() }

mod codegen;

pub use codegen::{ComposeOutput, compile_to_compose};
