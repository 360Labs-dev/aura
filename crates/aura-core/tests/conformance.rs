//! Backend conformance tests.
//!
//! Each conformance .aura file is compiled through ALL backends.
//! Verifies that every backend produces output without panicking.

use std::fs;

fn conformance_files() -> Vec<String> {
    let mut files = Vec::new();
    let dir = "../../tests/conformance";
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map(|e| e == "aura").unwrap_or(false) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();
    files
}

fn compile_all_backends(source: &str, name: &str) {
    // Parse
    let result = aura_core::parser::parse(source);
    assert!(
        result.program.is_some(),
        "Failed to parse {}: {:?}",
        name,
        result.errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );

    let program = result.program.unwrap();
    let hir = aura_core::hir::build_hir(&program);

    // Web backend
    let web = aura_backend_web::compile_to_web(&hir);
    assert!(!web.html.is_empty(), "{}: web HTML is empty", name);
    assert!(!web.css.is_empty(), "{}: web CSS is empty", name);
    assert!(
        web.html.contains("<!DOCTYPE html>"),
        "{}: web HTML missing doctype",
        name
    );

    // SwiftUI backend
    let swift = aura_backend_swift::compile_to_swift(&hir);
    assert!(!swift.swift.is_empty(), "{}: Swift output is empty", name);
    assert!(
        swift.swift.contains("import SwiftUI"),
        "{}: Swift missing import",
        name
    );
    assert!(
        swift.swift.contains("struct"),
        "{}: Swift missing struct",
        name
    );

    // Compose backend
    let compose = aura_backend_compose::compile_to_compose(&hir);
    assert!(
        !compose.kotlin.is_empty(),
        "{}: Kotlin output is empty",
        name
    );
    assert!(
        compose.kotlin.contains("@Composable"),
        "{}: Kotlin missing @Composable",
        name
    );
    assert!(
        compose.kotlin.contains("MaterialTheme"),
        "{}: Kotlin missing MaterialTheme",
        name
    );
}

#[test]
fn test_conformance_all_files() {
    let files = conformance_files();
    assert!(!files.is_empty(), "No conformance files found");

    for file in &files {
        let source = fs::read_to_string(file).expect(&format!("Cannot read {}", file));
        let name = std::path::Path::new(file)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        eprintln!("  Conformance: {}", name);
        compile_all_backends(&source, &name);
    }

    eprintln!(
        "  All {} conformance files passed across 3 backends",
        files.len()
    );
}

#[test]
fn test_conformance_examples() {
    // Also run all example files through all backends
    let examples = ["../../examples/minimal.aura"];

    for file in &examples {
        let source = fs::read_to_string(file).expect(&format!("Cannot read {}", file));
        compile_all_backends(&source, file);
    }
}
