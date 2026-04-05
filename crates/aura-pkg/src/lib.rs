//! # Aura Package Manager
//!
//! Manages Aura packages: install, publish, resolve dependencies.
//!
//! ## Package Format
//! Each package is a directory with `aura.toml` and `src/` containing `.aura` files.
//! Packages are stored locally in `.aura-packages/` relative to project root.
//!
//! ## Commands
//! - `aura pkg install @scope/name` — install from registry or local path
//! - `aura pkg publish` — pack and publish the current project
//! - `aura pkg list` — show installed packages
//! - `aura pkg remove @scope/name` — remove a package

mod registry;
mod resolver;

pub use registry::{Package, PackageManifest};
pub use resolver::{InstallPlan, resolve_dependencies};
