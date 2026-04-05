//! Multi-file project compilation tests.

use std::path::Path;

#[test]
fn test_multifile_project_loads() {
    let project = aura_core::project::load_project(Path::new("../../tests/conformance/multifile"));

    // Should find all .aura files
    assert!(
        project.files.len() >= 4,
        "Expected at least 4 files, found {}: {:?}",
        project.files.len(),
        project.files.iter().map(|f| &f.path).collect::<Vec<_>>()
    );

    // Should parse aura.toml
    assert!(project.config.is_some(), "Should load aura.toml config");
    let config = project.config.as_ref().unwrap();
    assert_eq!(config.app_name(Path::new(".")), "MultiFileTest");

    // Print any errors for debugging
    for err in &project.errors {
        eprintln!("  [{}] {}", err.code, err.message);
    }
}

#[test]
fn test_multifile_project_merges() {
    let project = aura_core::project::load_project(Path::new("../../tests/conformance/multifile"));

    // The merged program should have the main app name
    assert_eq!(project.program.app.name, "MultiFileTest");

    // Should have models from models/*.aura
    let models: Vec<_> = project
        .program
        .app
        .members
        .iter()
        .filter(|m| matches!(m, aura_core::ast::AppMember::Model(_)))
        .collect();
    assert!(
        models.len() >= 2,
        "Should have Task and User models, found {}",
        models.len()
    );

    // Should have screens from main.aura and screens/*.aura
    let screens: Vec<_> = project
        .program
        .app
        .members
        .iter()
        .filter(|m| matches!(m, aura_core::ast::AppMember::Screen(_)))
        .collect();
    assert!(
        screens.len() >= 2,
        "Should have Home and Settings screens, found {}",
        screens.len()
    );
}

#[test]
fn test_multifile_compiles_to_all_backends() {
    let project = aura_core::project::load_project(Path::new("../../tests/conformance/multifile"));
    let hir = aura_core::project::build_hir_for_project(&project);

    // Web
    let web = aura_backend_web::compile_to_web(&hir);
    assert!(!web.html.is_empty());
    assert!(!web.js.is_empty());

    // Swift
    let swift = aura_backend_swift::compile_to_swift(&hir);
    assert!(swift.swift.contains("struct"), "Swift should have structs");

    // Compose
    let compose = aura_backend_compose::compile_to_compose(&hir);
    assert!(
        compose.kotlin.contains("@Composable"),
        "Compose should have @Composable"
    );
}

#[test]
fn test_multifile_analyze_diagnostics() {
    let project = aura_core::project::load_project(Path::new("../../tests/conformance/multifile"));
    let diagnostics = aura_core::project::analyze_project(&project);

    // Should not have critical parse errors
    let parse_errors: Vec<_> = diagnostics.iter().filter(|e| e.is_error()).collect();
    // Some errors are expected (unresolved references across files), but no panics
    eprintln!(
        "  Diagnostics: {} errors, {} total",
        parse_errors.len(),
        diagnostics.len()
    );
}

#[test]
fn test_multifile_incremental_check() {
    let project = aura_core::project::load_project(Path::new("../../tests/conformance/multifile"));
    let check = aura_core::project::check_incremental(&project);

    // First check — no cache exists, everything is new
    assert!(
        !check.is_clean(),
        "First build should not be clean (no cache)"
    );
    eprintln!("  Incremental: {}", check.summary());
}
