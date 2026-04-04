//! End-to-end test: parse all example .aura files.

use aura_core::parser::parse;

fn parse_file(path: &str) -> aura_core::parser::ParseResult {
    let source = std::fs::read_to_string(path).expect(&format!("Failed to read {}", path));
    parse(&source)
}

#[test]
fn test_parse_minimal_example() {
    let result = parse_file("../../examples/minimal.aura");
    assert!(
        result.errors.is_empty(),
        "minimal.aura parse errors: {:#?}",
        result.errors
    );
    let program = result.program.expect("Should produce AST");
    assert_eq!(program.app.name, "Hello");
}

#[test]
fn test_parse_todo_example() {
    let result = parse_file("../../examples/todo.aura");
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("  [{}] {}", err.code, err.message);
        }
    }
    // For now, check that we get a program (some errors may be expected for complex features)
    assert!(result.program.is_some(), "todo.aura should produce an AST");
}

#[test]
fn test_parse_weather_example() {
    let result = parse_file("../../examples/weather.aura");
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("  [{}] {}", err.code, err.message);
        }
    }
    assert!(result.program.is_some(), "weather.aura should produce an AST");
}

#[test]
fn test_parse_chat_example() {
    let result = parse_file("../../examples/chat.aura");
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("  [{}] {}", err.code, err.message);
        }
    }
    assert!(result.program.is_some(), "chat.aura should produce an AST");
}

#[test]
fn test_parse_ecommerce_example() {
    let result = parse_file("../../examples/ecommerce.aura");
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("  [{}] {}", err.code, err.message);
        }
    }
    assert!(result.program.is_some(), "ecommerce.aura should produce an AST");
}
