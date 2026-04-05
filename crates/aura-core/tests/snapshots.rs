//! Baseline/snapshot tests for codegen.
//!
//! Like TypeScript's baseline tests: compile .aura files through each backend,
//! snapshot the output. Any codegen change shows up as a diff.
//! Uses the `insta` crate for snapshot management.

use aura_core::hir::build_hir;
use aura_core::parser::parse;

fn compile_to_hir(source: &str) -> aura_core::hir::HIRModule {
    let result = parse(source);
    assert!(
        result.program.is_some(),
        "Parse failed: {:?}",
        result.errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
    build_hir(result.program.as_ref().unwrap())
}

// === Web Backend Snapshots ===

#[test]
fn snapshot_web_hello_world() {
    let hir = compile_to_hir("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
    let output = aura_backend_web::compile_to_web(&hir);
    insta::assert_snapshot!("web_hello_js", output.js);
}

#[test]
fn snapshot_web_counter() {
    let source = r#"app Counter
  screen Main
    state count: int = 0
    view
      column gap.lg padding.xl align.center
        heading "Counter" size.2xl
        text count size.display .bold
        row gap.md
          button "-" .danger -> decrement()
          button "+" .accent -> increment()
    action increment
      count = count + 1
    action decrement
      count = count - 1"#;
    let hir = compile_to_hir(source);
    let output = aura_backend_web::compile_to_web(&hir);
    insta::assert_snapshot!("web_counter_js", output.js);
}

#[test]
fn snapshot_web_todo() {
    let source = r#"app Tasks
  model Task
    title: text
    done: bool = false
  screen Main
    state tasks: list[Task] = []
    state input: text = ""
    view
      column gap.md padding.lg
        heading "Tasks"
        row gap.sm
          textfield input placeholder: "New task..."
          button "Add" .accent -> addTask(input)
        each tasks as task
          row gap.md align.center
            checkbox task.done
            text task.title
    action addTask(title: text)
      tasks = tasks.append(Task(title: title))"#;
    let hir = compile_to_hir(source);
    let output = aura_backend_web::compile_to_web(&hir);
    insta::assert_snapshot!("web_todo_js", output.js);
}

// === SwiftUI Backend Snapshots ===

#[test]
fn snapshot_swift_hello() {
    let hir = compile_to_hir("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
    let output = aura_backend_swift::compile_to_swift(&hir);
    insta::assert_snapshot!("swift_hello", output.swift);
}

#[test]
fn snapshot_swift_counter() {
    let source = r#"app Counter
  screen Main
    state count: int = 0
    view
      column gap.lg padding.xl
        text count
        button "+" .accent -> increment()
    action increment
      count = count + 1"#;
    let hir = compile_to_hir(source);
    let output = aura_backend_swift::compile_to_swift(&hir);
    insta::assert_snapshot!("swift_counter", output.swift);
}

// === Compose Backend Snapshots ===

#[test]
fn snapshot_compose_hello() {
    let hir = compile_to_hir("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
    let output = aura_backend_compose::compile_to_compose(&hir);
    insta::assert_snapshot!("compose_hello", output.kotlin);
}

#[test]
fn snapshot_compose_counter() {
    let source = r#"app Counter
  screen Main
    state count: int = 0
    view
      column gap.lg padding.xl
        text count
        button "+" .accent -> increment()
    action increment
      count = count + 1"#;
    let hir = compile_to_hir(source);
    let output = aura_backend_compose::compile_to_compose(&hir);
    insta::assert_snapshot!("compose_counter", output.kotlin);
}

// === Cross-Backend Consistency ===

#[test]
fn snapshot_all_backends_model() {
    let source = r#"app Test
  model User
    name: text
    email: email
    active: bool = true
  screen Main
    view
      text "hi""#;
    let hir = compile_to_hir(source);

    let web = aura_backend_web::compile_to_web(&hir);
    let swift = aura_backend_swift::compile_to_swift(&hir);
    let compose = aura_backend_compose::compile_to_compose(&hir);

    // All backends should generate the User model
    assert!(web.js.contains("User"), "Web should generate User model");
    assert!(swift.swift.contains("struct User"), "Swift should generate User struct");
    assert!(compose.kotlin.contains("data class User"), "Compose should generate User data class");

    insta::assert_snapshot!("web_model_js", web.js);
    insta::assert_snapshot!("swift_model", swift.swift);
    insta::assert_snapshot!("compose_model", compose.kotlin);
}
