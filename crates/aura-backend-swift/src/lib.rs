//! # Aura SwiftUI Backend
//!
//! Generates SwiftUI code from Aura HIR.
//!
//! ## HIR → SwiftUI Mapping
//! - HIRColumn → VStack
//! - HIRRow → HStack
//! - HIRStack → ZStack
//! - HIRText → Text
//! - HIRButton → Button
//! - HIRTextField → TextField
//! - HIRToggle → Toggle
//! - HIRCheckbox → Toggle (checkbox style)
//! - HIRList/Each → ForEach
//! - HIRNavigate → NavigationLink
//! - HIRState → @State

mod codegen;

pub use codegen::{SwiftOutput, compile_to_swift};
