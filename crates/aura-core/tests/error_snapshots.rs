//! Error diagnostic snapshot tests.
//! Ensures error messages don't silently change.

fn errors_for(source: &str) -> String {
    let result = aura_core::parser::parse(source);
    let mut all_errors = result.errors;
    if let Some(ref program) = result.program {
        let analysis = aura_core::semantic::SemanticAnalyzer::new().analyze(program);
        all_errors.extend(analysis.errors);
    }
    all_errors
        .iter()
        .map(|e| {
            let fix = e
                .fix
                .as_ref()
                .map(|f| format!(" fix: '{}' ({:.0}%)", f.replacement, f.confidence * 100.0))
                .unwrap_or_default();
            let help = e
                .help
                .as_ref()
                .map(|h| format!(" help: {}", h))
                .unwrap_or_default();
            format!("[{}] {}{}{}", e.code, e.message, fix, help)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn snap_error_typo_variable() {
    insta::assert_snapshot!(
        "error_typo_variable",
        errors_for(
            "\
app T
  screen M
    state todos: list[text] = []
    view
      text \"hi\"
    action test
      todoos = []"
        )
    );
}

#[test]
fn snap_error_duplicate_field() {
    insta::assert_snapshot!(
        "error_duplicate_field",
        errors_for(
            "\
app T
  model Bad
    name: text
    name: int"
        )
    );
}

#[test]
fn snap_error_secret_comparison() {
    insta::assert_snapshot!(
        "error_secret_comparison",
        errors_for(
            "\
app T
  fn check(a: secret, b: secret) -> bool
    a == b"
        )
    );
}

#[test]
fn snap_error_state_mutation_in_fn() {
    insta::assert_snapshot!(
        "error_state_mutation_fn",
        errors_for(
            "\
app T
  screen M
    state x: int = 0
    view
      text \"hi\"
    fn bad
      x = 1"
        )
    );
}

#[test]
fn snap_error_invalid_design_token() {
    insta::assert_snapshot!(
        "error_invalid_token",
        errors_for(
            "\
app T
  screen M
    view
      column .xxxlarge
        text \"hi\""
        )
    );
}

#[test]
fn snap_error_parse_bad_indent() {
    insta::assert_snapshot!(
        "error_bad_indent",
        errors_for(
            "\
app T
   screen M
    view
      text \"hi\""
        )
    );
}

#[test]
fn snap_error_type_mismatch() {
    insta::assert_snapshot!(
        "error_type_mismatch",
        errors_for(
            "\
app T
  model Item
    count: int = \"not a number\""
        )
    );
}

#[test]
fn snap_error_empty_program() {
    insta::assert_snapshot!("error_empty", errors_for(""));
}

#[test]
fn snap_clean_program() {
    insta::assert_snapshot!(
        "error_clean",
        errors_for(
            "\
app T
  model Todo
    title: text
    done: bool = false
  screen M
    state items: list[Todo] = []
    view
      text \"hi\""
        )
    );
}

#[test]
fn snap_error_union_type() {
    insta::assert_snapshot!(
        "error_union",
        errors_for(
            "\
app T
  model Result
    value: text | int
  screen M
    view
      text \"hi\""
        )
    );
}
