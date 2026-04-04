//! # Aura Semantic Diff
//!
//! Compares two Aura programs at the HIR level and produces a
//! human-readable summary of what changed.
//!
//! Usage: `aura diff v1.aura v2.aura`

use crate::hir::*;

/// A semantic change between two versions.
#[derive(Debug)]
pub enum Change {
    Added(String),
    Removed(String),
    Changed(String),
    Unchanged(String),
}

/// Diff two HIR modules.
pub fn diff(old: &HIRModule, new: &HIRModule) -> Vec<Change> {
    let mut changes = Vec::new();

    // App-level changes
    if old.app.name != new.app.name {
        changes.push(Change::Changed(format!(
            "App renamed: {} → {}",
            old.app.name, new.app.name
        )));
    }
    if old.app.theme != new.app.theme {
        changes.push(Change::Changed(format!(
            "Theme changed: {:?} → {:?}",
            old.app.theme, new.app.theme
        )));
    }
    if old.app.navigation != new.app.navigation {
        changes.push(Change::Changed(format!(
            "Navigation changed: {:?} → {:?}",
            old.app.navigation, new.app.navigation
        )));
    }

    // Model changes
    diff_named_items(
        &old.models,
        &new.models,
        |m| &m.name,
        |old_m, new_m| diff_model(old_m, new_m),
        "Model",
        &mut changes,
    );

    // Screen changes
    diff_named_items(
        &old.screens,
        &new.screens,
        |s| &s.name,
        |old_s, new_s| diff_screen(old_s, new_s),
        "Screen",
        &mut changes,
    );

    // Component changes
    diff_named_items(
        &old.components,
        &new.components,
        |c| &c.name,
        |old_c, new_c| diff_component(old_c, new_c),
        "Component",
        &mut changes,
    );

    if changes.is_empty() {
        changes.push(Change::Unchanged("No semantic changes detected.".to_string()));
    }

    changes
}

/// Format a diff as human-readable text.
pub fn format_diff(changes: &[Change]) -> String {
    let mut out = String::new();
    for change in changes {
        match change {
            Change::Added(msg) => out.push_str(&format!("  + {}\n", msg)),
            Change::Removed(msg) => out.push_str(&format!("  - {}\n", msg)),
            Change::Changed(msg) => out.push_str(&format!("  ~ {}\n", msg)),
            Change::Unchanged(msg) => out.push_str(&format!("  = {}\n", msg)),
        }
    }
    out
}

fn diff_named_items<T, F, D>(
    old_items: &[T],
    new_items: &[T],
    name_fn: F,
    diff_fn: D,
    kind: &str,
    changes: &mut Vec<Change>,
) where
    F: Fn(&T) -> &str,
    D: Fn(&T, &T) -> Vec<Change>,
{
    let old_names: Vec<&str> = old_items.iter().map(|i| name_fn(i)).collect();
    let new_names: Vec<&str> = new_items.iter().map(|i| name_fn(i)).collect();

    // Added
    for new_item in new_items {
        let name = name_fn(new_item);
        if !old_names.contains(&name) {
            changes.push(Change::Added(format!("{}: {}", kind, name)));
        }
    }

    // Removed
    for old_item in old_items {
        let name = name_fn(old_item);
        if !new_names.contains(&name) {
            changes.push(Change::Removed(format!("{}: {}", kind, name)));
        }
    }

    // Changed
    for old_item in old_items {
        let name = name_fn(old_item);
        if let Some(new_item) = new_items.iter().find(|i| name_fn(i) == name) {
            let item_changes = diff_fn(old_item, new_item);
            if !item_changes.is_empty() {
                changes.push(Change::Changed(format!("{}: {} modified", kind, name)));
                changes.extend(item_changes);
            }
        }
    }
}

fn diff_model(old: &HIRModel, new: &HIRModel) -> Vec<Change> {
    let mut changes = Vec::new();
    let old_fields: Vec<&str> = old.fields.iter().map(|f| f.name.as_str()).collect();
    let new_fields: Vec<&str> = new.fields.iter().map(|f| f.name.as_str()).collect();

    for f in &new.fields {
        if !old_fields.contains(&f.name.as_str()) {
            changes.push(Change::Added(format!(
                "  field '{}' ({})",
                f.name,
                f.field_type.display_name()
            )));
        }
    }
    for f in &old.fields {
        if !new_fields.contains(&f.name.as_str()) {
            changes.push(Change::Removed(format!("  field '{}'", f.name)));
        }
    }
    // Type changes for existing fields
    for old_f in &old.fields {
        if let Some(new_f) = new.fields.iter().find(|f| f.name == old_f.name) {
            if old_f.field_type != new_f.field_type {
                changes.push(Change::Changed(format!(
                    "  field '{}' type: {} → {}",
                    old_f.name,
                    old_f.field_type.display_name(),
                    new_f.field_type.display_name()
                )));
            }
        }
    }
    changes
}

fn diff_screen(old: &HIRScreen, new: &HIRScreen) -> Vec<Change> {
    let mut changes = Vec::new();

    // State changes
    let old_state: Vec<&str> = old.state.iter().map(|s| s.name.as_str()).collect();
    let new_state: Vec<&str> = new.state.iter().map(|s| s.name.as_str()).collect();
    for s in &new.state {
        if !old_state.contains(&s.name.as_str()) {
            changes.push(Change::Added(format!(
                "  state '{}' ({})",
                s.name,
                s.state_type.display_name()
            )));
        }
    }
    for s in &old.state {
        if !new_state.contains(&s.name.as_str()) {
            changes.push(Change::Removed(format!("  state '{}'", s.name)));
        }
    }

    // Action changes
    let old_actions: Vec<&str> = old.actions.iter().map(|a| a.name.as_str()).collect();
    let new_actions: Vec<&str> = new.actions.iter().map(|a| a.name.as_str()).collect();
    for a in &new.actions {
        if !old_actions.contains(&a.name.as_str()) {
            changes.push(Change::Added(format!("  action '{}'", a.name)));
        }
    }
    for a in &old.actions {
        if !new_actions.contains(&a.name.as_str()) {
            changes.push(Change::Removed(format!("  action '{}'", a.name)));
        }
    }

    // Function changes
    let old_fns: Vec<&str> = old.functions.iter().map(|f| f.name.as_str()).collect();
    let new_fns: Vec<&str> = new.functions.iter().map(|f| f.name.as_str()).collect();
    for f in &new.functions {
        if !old_fns.contains(&f.name.as_str()) {
            changes.push(Change::Added(format!("  function '{}'", f.name)));
        }
    }
    for f in &old.functions {
        if !new_fns.contains(&f.name.as_str()) {
            changes.push(Change::Removed(format!("  function '{}'", f.name)));
        }
    }

    changes
}

fn diff_component(old: &HIRComponent, new: &HIRComponent) -> Vec<Change> {
    let mut changes = Vec::new();

    let old_props: Vec<&str> = old.props.iter().map(|p| p.name.as_str()).collect();
    let new_props: Vec<&str> = new.props.iter().map(|p| p.name.as_str()).collect();
    for p in &new.props {
        if !old_props.contains(&p.name.as_str()) {
            changes.push(Change::Added(format!(
                "  prop '{}' ({})",
                p.name,
                p.param_type.display_name()
            )));
        }
    }
    for p in &old.props {
        if !new_props.contains(&p.name.as_str()) {
            changes.push(Change::Removed(format!("  prop '{}'", p.name)));
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(source: &str) -> HIRModule {
        let result = crate::parser::parse(source);
        assert!(result.errors.is_empty(), "Parse errors: {:?}", result.errors);
        crate::hir::build_hir(result.program.as_ref().unwrap())
    }

    #[test]
    fn test_no_changes() {
        let a = build("app Test\n  screen Main\n    view\n      text \"Hi\"");
        let b = build("app Test\n  screen Main\n    view\n      text \"Hi\"");
        let changes = diff(&a, &b);
        assert!(changes.iter().any(|c| matches!(c, Change::Unchanged(_))));
    }

    #[test]
    fn test_added_screen() {
        let a = build("app Test\n  screen Main\n    view\n      text \"Hi\"");
        let b = build("app Test\n  screen Main\n    view\n      text \"Hi\"\n  screen Settings\n    view\n      text \"Settings\"");
        let changes = diff(&a, &b);
        let text = format_diff(&changes);
        assert!(text.contains("+ Screen: Settings"));
    }

    #[test]
    fn test_removed_model() {
        let a = build("app Test\n  model Todo\n    title: text\n  screen Main\n    view\n      text \"Hi\"");
        let b = build("app Test\n  screen Main\n    view\n      text \"Hi\"");
        let changes = diff(&a, &b);
        let text = format_diff(&changes);
        assert!(text.contains("- Model: Todo"));
    }

    #[test]
    fn test_added_field() {
        let a = build("app Test\n  model Todo\n    title: text\n  screen Main\n    view\n      text \"Hi\"");
        let b = build("app Test\n  model Todo\n    title: text\n    done: bool\n  screen Main\n    view\n      text \"Hi\"");
        let changes = diff(&a, &b);
        let text = format_diff(&changes);
        assert!(text.contains("field 'done'"));
    }

    #[test]
    fn test_theme_change() {
        let a = build("app Test\n  theme: modern.light\n  screen Main\n    view\n      text \"Hi\"");
        let b = build("app Test\n  theme: modern.dark\n  screen Main\n    view\n      text \"Hi\"");
        let changes = diff(&a, &b);
        let text = format_diff(&changes);
        assert!(text.contains("Theme changed"));
    }
}
