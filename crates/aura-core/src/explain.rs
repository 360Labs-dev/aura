//! # Aura Explain
//!
//! Converts an Aura program (via HIR) into a human-readable English description.
//! This makes Aura accessible to non-programmers reviewing AI-generated code.
//!
//! Usage: `aura explain src/main.aura`

use crate::hir::*;

/// Produce a human-readable explanation of an HIR module.
pub fn explain(module: &HIRModule) -> String {
    let mut out = String::new();
    let mut ex = Explainer {
        out: &mut out,
        indent: 0,
    };
    ex.explain_module(module);
    out
}

struct Explainer<'a> {
    out: &'a mut String,
    indent: usize,
}

impl<'a> Explainer<'a> {
    fn line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn blank(&mut self) {
        self.out.push('\n');
    }

    fn explain_module(&mut self, module: &HIRModule) {
        self.line(&format!("App: {}", module.app.name));
        if let Some(ref theme) = module.app.theme {
            self.line(&format!("Theme: {}", theme));
        }
        self.line(&format!("Navigation: {:?}", module.app.navigation));
        if !module.app.routes.is_empty() {
            self.line("Routes:");
            self.indent += 1;
            for route in &module.app.routes {
                self.line(&format!("{} → {}", route.pattern, route.screen));
            }
            self.indent -= 1;
        }
        self.blank();

        // Models
        if !module.models.is_empty() {
            self.line("Data models:");
            self.indent += 1;
            for model in &module.models {
                self.explain_model(model);
            }
            self.indent -= 1;
            self.blank();
        }

        // Screens
        for screen in &module.screens {
            self.explain_screen(screen);
            self.blank();
        }

        // Components
        if !module.components.is_empty() {
            self.line("Reusable components:");
            self.indent += 1;
            for comp in &module.components {
                self.explain_component(comp);
            }
            self.indent -= 1;
        }
    }

    fn explain_model(&mut self, model: &HIRModel) {
        let field_names: Vec<String> = model
            .fields
            .iter()
            .map(|f| {
                let ty = f.field_type.display_name();
                if let Some(ref default) = f.default {
                    format!(
                        "{} ({}, default: {})",
                        f.name,
                        ty,
                        self.expr_summary(default)
                    )
                } else {
                    format!("{} ({})", f.name, ty)
                }
            })
            .collect();
        self.line(&format!("{}: {}", model.name, field_names.join(", ")));
    }

    fn explain_screen(&mut self, screen: &HIRScreen) {
        self.line(&format!("Screen: {}", screen.name));
        self.indent += 1;

        if let Some(ref tab) = screen.tab {
            self.line(&format!("Tab: {} ({})", tab.label, tab.icon));
        }

        if !screen.params.is_empty() {
            let params: Vec<String> = screen
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.param_type.display_name()))
                .collect();
            self.line(&format!("Parameters: {}", params.join(", ")));
        }

        if !screen.state.is_empty() {
            self.line("State:");
            self.indent += 1;
            for s in &screen.state {
                let init = s
                    .initial
                    .as_ref()
                    .map(|e| format!(" = {}", self.expr_summary(e)))
                    .unwrap_or_default();
                self.line(&format!(
                    "{}: {}{}",
                    s.name,
                    s.state_type.display_name(),
                    init
                ));
            }
            self.indent -= 1;
        }

        self.line("Layout:");
        self.indent += 1;
        self.explain_view(&screen.view);
        self.indent -= 1;

        if !screen.actions.is_empty() {
            self.line("Actions:");
            self.indent += 1;
            for action in &screen.actions {
                self.explain_action(action);
            }
            self.indent -= 1;
        }

        if !screen.functions.is_empty() {
            self.line("Functions:");
            self.indent += 1;
            for func in &screen.functions {
                let params: Vec<String> = func
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.param_type.display_name()))
                    .collect();
                let ret = func.return_type.display_name();
                if params.is_empty() {
                    self.line(&format!("{} → {}", func.name, ret));
                } else {
                    self.line(&format!("{}({}) → {}", func.name, params.join(", "), ret));
                }
            }
            self.indent -= 1;
        }

        self.indent -= 1;
    }

    fn explain_component(&mut self, comp: &HIRComponent) {
        let props: Vec<String> = comp
            .props
            .iter()
            .map(|p| format!("{}: {}", p.name, p.param_type.display_name()))
            .collect();
        self.line(&format!("{} ({})", comp.name, props.join(", ")));
        self.indent += 1;
        self.explain_view(&comp.view);
        self.indent -= 1;
    }

    fn explain_action(&mut self, action: &HIRAction) {
        let params: Vec<String> = action.params.iter().map(|p| p.name.clone()).collect();
        if params.is_empty() {
            self.line(&format!("{}:", action.name));
        } else {
            self.line(&format!("{}({}):", action.name, params.join(", ")));
        }
        self.indent += 1;
        for stmt in &action.body {
            self.explain_stmt(stmt);
        }
        self.indent -= 1;
    }

    fn explain_view(&mut self, view: &HIRView) {
        match view {
            HIRView::Column(layout) => {
                self.line("Vertical layout:");
                self.explain_layout_children(&layout.children);
            }
            HIRView::Row(layout) => {
                self.line("Horizontal layout:");
                self.explain_layout_children(&layout.children);
            }
            HIRView::Stack(layout) => {
                self.line("Layered stack:");
                self.explain_layout_children(&layout.children);
            }
            HIRView::Grid(grid) => {
                self.line("Grid:");
                self.explain_layout_children(&grid.children);
            }
            HIRView::Scroll(scroll) => {
                self.line("Scrollable area:");
                self.explain_layout_children(&scroll.children);
            }
            HIRView::Text(text) => {
                self.line(&format!("Text: {}", self.expr_summary(&text.content)));
            }
            HIRView::Heading(heading) => {
                self.line(&format!("Heading: {}", self.expr_summary(&heading.content)));
            }
            HIRView::Image(image) => {
                self.line(&format!("Image: {}", self.expr_summary(&image.source)));
            }
            HIRView::Icon(icon) => {
                self.line(&format!("Icon: {}", self.expr_summary(&icon.name)));
            }
            HIRView::Badge(badge) => {
                self.line(&format!("Badge: {}", self.expr_summary(&badge.content)));
            }
            HIRView::Progress(_) => {
                self.line("Progress indicator");
            }
            HIRView::Button(button) => {
                let label = self.expr_summary(&button.label);
                let action = self.action_expr_summary(&button.action);
                self.line(&format!("Button \"{}\" → {}", label, action));
            }
            HIRView::TextField(field) => {
                let ph = field.placeholder.as_deref().unwrap_or("...");
                self.line(&format!("Text input ({}), bound to {}", ph, field.binding));
            }
            HIRView::Checkbox(cb) => {
                self.line(&format!("Checkbox, bound to {}", cb.binding));
            }
            HIRView::Toggle(toggle) => {
                let label = toggle.label.as_deref().unwrap_or("toggle");
                self.line(&format!(
                    "Toggle \"{}\", bound to {}",
                    label, toggle.binding
                ));
            }
            HIRView::Slider(slider) => {
                self.line(&format!(
                    "Slider ({}-{}), bound to {}",
                    slider.min, slider.max, slider.binding
                ));
            }
            HIRView::Picker(picker) => {
                self.line(&format!("Picker, bound to {}", picker.binding));
            }
            HIRView::Segmented(seg) => {
                self.line(&format!("Segmented control, bound to {}", seg.binding));
            }
            HIRView::Spacer => {
                self.line("Spacer (flexible space)");
            }
            HIRView::Divider(_) => {
                self.line("Divider line");
            }
            HIRView::Conditional(cond) => {
                self.line(&format!("If {}:", self.expr_summary(&cond.condition)));
                self.indent += 1;
                self.explain_view(&cond.then_view);
                self.indent -= 1;
                if let Some(ref else_view) = cond.else_view {
                    self.line("Otherwise:");
                    self.indent += 1;
                    self.explain_view(else_view);
                    self.indent -= 1;
                }
            }
            HIRView::Each(each) => {
                self.line(&format!(
                    "For each {} in {}:",
                    each.item_name,
                    self.expr_summary(&each.iterable)
                ));
                self.indent += 1;
                self.explain_view(&each.body);
                self.indent -= 1;
            }
            HIRView::Switch(switch) => {
                self.line(&format!("When {}:", self.expr_summary(&switch.expression)));
                self.indent += 1;
                for case in &switch.cases {
                    self.line(&format!("{:?}:", case.pattern));
                    self.indent += 1;
                    self.explain_view(&case.view);
                    self.indent -= 1;
                }
                self.indent -= 1;
            }
            HIRView::ComponentRef(comp_ref) => {
                if comp_ref.args.is_empty() {
                    self.line(&format!("Component: {}", comp_ref.name));
                } else {
                    let args: Vec<String> = comp_ref
                        .args
                        .iter()
                        .filter(|(k, _)| k != "_")
                        .map(|(k, v)| format!("{}: {}", k, self.expr_summary(v)))
                        .collect();
                    self.line(&format!(
                        "Component: {}({})",
                        comp_ref.name,
                        args.join(", ")
                    ));
                }
            }
            HIRView::Group(children) => {
                for child in children {
                    self.explain_view(child);
                }
            }
            HIRView::Slot => {
                self.line("(slot for child content)");
            }
            _ => {
                self.line("(other element)");
            }
        }
    }

    fn explain_layout_children(&mut self, children: &[HIRView]) {
        self.indent += 1;
        for child in children {
            self.explain_view(child);
        }
        self.indent -= 1;
    }

    fn explain_stmt(&mut self, stmt: &HIRStmt) {
        match stmt {
            HIRStmt::Assign(name, value) => {
                self.line(&format!("Set {} to {}", name, self.expr_summary(value)));
            }
            HIRStmt::Let(name, _, value) => {
                self.line(&format!("Define {} as {}", name, self.expr_summary(value)));
            }
            HIRStmt::If(cond, then_body, else_body) => {
                self.line(&format!("If {}:", self.expr_summary(cond)));
                self.indent += 1;
                for s in then_body {
                    self.explain_stmt(s);
                }
                self.indent -= 1;
                if let Some(else_stmts) = else_body {
                    self.line("Otherwise:");
                    self.indent += 1;
                    for s in else_stmts {
                        self.explain_stmt(s);
                    }
                    self.indent -= 1;
                }
            }
            HIRStmt::Return(Some(value)) => {
                self.line(&format!("Return {}", self.expr_summary(value)));
            }
            HIRStmt::Return(None) => {
                self.line("Return");
            }
            HIRStmt::Expr(expr) => {
                self.line(&self.expr_summary(expr));
            }
            _ => {
                self.line("(other action)");
            }
        }
    }

    fn expr_summary(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => format!("\"{}\"", s),
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::FloatLit(f) => f.to_string(),
            HIRExpr::BoolLit(b) => b.to_string(),
            HIRExpr::Nil => "nothing".to_string(),
            HIRExpr::Var(name, _) => name.clone(),
            HIRExpr::MemberAccess(obj, member, _) => {
                format!("{}.{}", self.expr_summary(obj), member)
            }
            HIRExpr::Call(func, args, _) => {
                let f = self.expr_summary(func);
                let a: Vec<String> = args.iter().map(|a| self.expr_summary(a)).collect();
                format!("{}({})", f, a.join(", "))
            }
            HIRExpr::BinOp(left, op, right, _) => {
                let l = self.expr_summary(left);
                let r = self.expr_summary(right);
                let op_str = match op {
                    crate::ast::BinOp::Add => "+",
                    crate::ast::BinOp::Sub => "-",
                    crate::ast::BinOp::Mul => "*",
                    crate::ast::BinOp::Div => "/",
                    crate::ast::BinOp::Eq => "equals",
                    crate::ast::BinOp::NotEq => "not equal to",
                    crate::ast::BinOp::Lt => "less than",
                    crate::ast::BinOp::Gt => "greater than",
                    crate::ast::BinOp::And => "and",
                    crate::ast::BinOp::Or => "or",
                    _ => "?",
                };
                format!("{} {} {}", l, op_str, r)
            }
            HIRExpr::UnaryOp(op, operand, _) => match op {
                crate::ast::UnaryOp::Not => format!("not {}", self.expr_summary(operand)),
                crate::ast::UnaryOp::Neg => format!("-{}", self.expr_summary(operand)),
            },
            HIRExpr::Constructor(name, args, _) => {
                let fields: Vec<String> = args
                    .iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{}: {}", k, self.expr_summary(v)))
                    .collect();
                if fields.is_empty() {
                    format!("new {}", name)
                } else {
                    format!("new {}({})", name, fields.join(", "))
                }
            }
            HIRExpr::Lambda(params, body, _) => {
                let p: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
                format!("({}) => {}", p.join(", "), self.expr_summary(body))
            }
            _ => "...".to_string(),
        }
    }

    fn action_expr_summary(&self, action: &HIRActionExpr) -> String {
        match action {
            HIRActionExpr::Call(name, args) => {
                if args.is_empty() {
                    format!("{}", name)
                } else {
                    let a: Vec<String> = args.iter().map(|a| self.expr_summary(a)).collect();
                    format!("{}({})", name, a.join(", "))
                }
            }
            HIRActionExpr::Navigate(nav) => match nav {
                HIRNavigate::Back => "go back".to_string(),
                HIRNavigate::Root => "go to home".to_string(),
                HIRNavigate::To(expr) => format!("navigate to {}", self.expr_summary(expr)),
                HIRNavigate::Modal(expr) => format!("show modal {}", self.expr_summary(expr)),
                HIRNavigate::Replace(expr) => format!("replace with {}", self.expr_summary(expr)),
                HIRNavigate::Dismiss => "dismiss".to_string(),
            },
            HIRActionExpr::Sequence(actions) => actions
                .iter()
                .map(|a| self.action_expr_summary(a))
                .collect::<Vec<_>>()
                .join(", then "),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn explain_source(source: &str) -> String {
        let result = crate::parser::parse(source);
        assert!(
            result.errors.is_empty(),
            "Parse errors: {:?}",
            result.errors
        );
        let hir = crate::hir::build_hir(result.program.as_ref().unwrap());
        explain(&hir)
    }

    #[test]
    fn test_explain_minimal() {
        let output =
            explain_source("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
        assert!(output.contains("App: Hello"));
        assert!(output.contains("Screen: Main"));
        assert!(output.contains("Hello, Aura!"));
    }

    #[test]
    fn test_explain_model() {
        let output = explain_source(
            "\
app Test
  model Todo
    title: text
    done: bool = false
  screen Main
    view
      text \"hi\"",
        );
        assert!(output.contains("Todo"));
        assert!(output.contains("title"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_explain_button() {
        let output = explain_source(
            "\
app Test
  screen Main
    view
      button \"Save\" .accent -> save()
    action save
      return",
        );
        assert!(output.contains("Button"));
        assert!(output.contains("Save"));
        assert!(output.contains("save"));
    }

    #[test]
    fn test_explain_each() {
        let output = explain_source(
            "\
app Test
  screen Main
    state items: list[text] = []
    view
      each items as item
        text item",
        );
        assert!(output.contains("For each item in items"));
    }
}
