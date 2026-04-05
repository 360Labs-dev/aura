//! # Aura WinUI Backend
//!
//! Generates WinUI 3 / C# + XAML from Aura HIR.
//!
//! ## HIR → WinUI Mapping
//! - HIRColumn → StackPanel (Vertical)
//! - HIRRow → StackPanel (Horizontal)
//! - HIRText → TextBlock
//! - HIRHeading → TextBlock with large style
//! - HIRButton → Button
//! - HIRTextField → TextBox
//! - HIRCheckbox → CheckBox
//! - HIRToggle → ToggleSwitch
//! - HIRSlider → Slider
//! - HIRDivider → Border (thin line)
//! - Design tokens → WinUI resource dictionaries

mod codegen;

pub use codegen::{compile_to_winui, WinUiOutput};
