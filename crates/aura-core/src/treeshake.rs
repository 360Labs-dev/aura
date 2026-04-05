//! # Dead Code Elimination (Tree Shaking)
//!
//! Removes unused declarations from the HIR before codegen.
//! Like TypeScript/Webpack tree shaking — only emit code that's reachable.
//!
//! ## Algorithm
//! 1. Start from the first screen (entry point)
//! 2. Walk its view tree, collecting referenced components and models
//! 3. Walk actions and functions, collecting referenced types
//! 4. Transitively include all dependencies
//! 5. Remove unreferenced models, components, functions

use std::collections::HashSet;

use crate::hir::*;

/// Remove unused declarations from an HIR module.
/// Returns the number of declarations removed.
pub fn tree_shake(module: &mut HIRModule) -> usize {
    // Collect all referenced names starting from screens
    let mut referenced: HashSet<String> = HashSet::new();

    // All screens are entry points
    for screen in &module.screens {
        collect_screen_refs(screen, &mut referenced);
    }

    // Count removals
    let before = module.models.len() + module.components.len();

    // Remove unreferenced models
    module.models.retain(|m| {
        let keep = referenced.contains(&m.name);
        if !keep {
            eprintln!("  [tree-shake] Removing unused model: {}", m.name);
        }
        keep
    });

    // Remove unreferenced components
    module.components.retain(|c| {
        let keep = referenced.contains(&c.name);
        if !keep {
            eprintln!("  [tree-shake] Removing unused component: {}", c.name);
        }
        keep
    });

    let after = module.models.len() + module.components.len();
    before - after
}

/// Collect all type/component names referenced by a screen.
fn collect_screen_refs(screen: &HIRScreen, refs: &mut HashSet<String>) {
    // State types
    for state in &screen.state {
        collect_type_refs(&state.state_type, refs);
    }

    // Params
    for param in &screen.params {
        collect_type_refs(&param.param_type, refs);
    }

    // View tree
    collect_view_refs(&screen.view, refs);

    // Actions
    for action in &screen.actions {
        for param in &action.params {
            collect_type_refs(&param.param_type, refs);
        }
        for stmt in &action.body {
            collect_stmt_refs(stmt, refs);
        }
    }

    // Functions
    for func in &screen.functions {
        collect_type_refs(&func.return_type, refs);
        for param in &func.params {
            collect_type_refs(&param.param_type, refs);
        }
    }
}

/// Collect type names from a type.
fn collect_type_refs(ty: &crate::types::AuraType, refs: &mut HashSet<String>) {
    match ty {
        crate::types::AuraType::Named(name) => {
            refs.insert(name.clone());
        }
        crate::types::AuraType::List(inner)
        | crate::types::AuraType::Set(inner)
        | crate::types::AuraType::Optional(inner) => {
            collect_type_refs(inner, refs);
        }
        crate::types::AuraType::Map(k, v) => {
            collect_type_refs(k, refs);
            collect_type_refs(v, refs);
        }
        crate::types::AuraType::Function(ft) => {
            for p in &ft.params {
                collect_type_refs(p, refs);
            }
            collect_type_refs(&ft.return_type, refs);
        }
        crate::types::AuraType::Union(types) => {
            for t in types {
                collect_type_refs(t, refs);
            }
        }
        _ => {}
    }
}

/// Collect referenced names from a view tree.
fn collect_view_refs(view: &HIRView, refs: &mut HashSet<String>) {
    match view {
        HIRView::Column(l) | HIRView::Row(l) | HIRView::Stack(l) | HIRView::Wrap(l) => {
            for child in &l.children {
                collect_view_refs(child, refs);
            }
        }
        HIRView::Grid(g) => {
            for child in &g.children {
                collect_view_refs(child, refs);
            }
        }
        HIRView::Scroll(s) => {
            for child in &s.children {
                collect_view_refs(child, refs);
            }
        }
        HIRView::ComponentRef(comp) => {
            refs.insert(comp.name.clone());
            for child in &comp.children {
                collect_view_refs(child, refs);
            }
        }
        HIRView::Conditional(cond) => {
            collect_view_refs(&cond.then_view, refs);
            if let Some(ref else_view) = cond.else_view {
                collect_view_refs(else_view, refs);
            }
        }
        HIRView::Each(each) => {
            collect_view_refs(&each.body, refs);
        }
        HIRView::Switch(sw) => {
            for case in &sw.cases {
                collect_view_refs(&case.view, refs);
            }
        }
        HIRView::Group(children) => {
            for child in children {
                collect_view_refs(child, refs);
            }
        }
        _ => {}
    }
}

/// Collect referenced names from statements.
fn collect_stmt_refs(stmt: &HIRStmt, refs: &mut HashSet<String>) {
    match stmt {
        HIRStmt::Assign(_, expr) | HIRStmt::Expr(expr) => {
            collect_expr_refs(expr, refs);
        }
        HIRStmt::Let(_, ty, expr) => {
            collect_type_refs(ty, refs);
            collect_expr_refs(expr, refs);
        }
        HIRStmt::If(cond, then_body, else_body) => {
            collect_expr_refs(cond, refs);
            for s in then_body {
                collect_stmt_refs(s, refs);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    collect_stmt_refs(s, refs);
                }
            }
        }
        _ => {}
    }
}

/// Collect referenced names from expressions.
fn collect_expr_refs(expr: &HIRExpr, refs: &mut HashSet<String>) {
    match expr {
        HIRExpr::Constructor(name, args, _) => {
            refs.insert(name.clone());
            for (_, v) in args {
                collect_expr_refs(v, refs);
            }
        }
        HIRExpr::Call(func, args, _) => {
            collect_expr_refs(func, refs);
            for a in args {
                collect_expr_refs(a, refs);
            }
        }
        HIRExpr::NamedCall(func, args, _) => {
            collect_expr_refs(func, refs);
            for (_, v) in args {
                collect_expr_refs(v, refs);
            }
        }
        HIRExpr::MemberAccess(obj, _, _) => {
            collect_expr_refs(obj, refs);
        }
        HIRExpr::BinOp(l, _, r, _) => {
            collect_expr_refs(l, refs);
            collect_expr_refs(r, refs);
        }
        HIRExpr::Lambda(_, body, _) => {
            collect_expr_refs(body, refs);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_shake_unused_model() {
        let source = r#"app Test
  model Used
    name: text
  model Unused
    data: int
  screen Main
    state items: list[Used] = []
    view
      text "hi""#;
        let result = crate::parser::parse(source);
        let mut hir = crate::hir::build_hir(result.program.as_ref().unwrap());

        assert_eq!(hir.models.len(), 2);
        let removed = tree_shake(&mut hir);
        assert_eq!(removed, 1, "Should remove 1 unused model");
        assert_eq!(hir.models.len(), 1);
        assert_eq!(hir.models[0].name, "Used");
    }

    #[test]
    fn test_tree_shake_keeps_used_component() {
        let source = r#"app Test
  component Card(title: text)
    view
      text title
  component Unused(x: int)
    view
      text "unused"
  screen Main
    view
      Card(title: "hi")"#;
        let result = crate::parser::parse(source);
        let mut hir = crate::hir::build_hir(result.program.as_ref().unwrap());

        assert_eq!(hir.components.len(), 2);
        let removed = tree_shake(&mut hir);
        assert_eq!(removed, 1);
        assert_eq!(hir.components[0].name, "Card");
    }
}
