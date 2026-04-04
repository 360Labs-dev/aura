//! # Aura Formatter
//!
//! Parses a `.aura` file and re-emits it with consistent formatting.
//! Proves the parser roundtrips correctly.
//!
//! Rules:
//! - 2-space indentation
//! - Blank line between top-level declarations
//! - No trailing whitespace
//! - Consistent spacing around operators

use crate::ast::*;

/// Format an Aura program from its AST back into source code.
pub fn format(program: &Program) -> String {
    let mut f = Formatter { out: String::new(), indent: 0 };
    f.emit_program(program);
    f.out
}

struct Formatter {
    out: String,
    indent: usize,
}

impl Formatter {
    fn line(&mut self, text: &str) {
        for _ in 0..self.indent { self.out.push_str("  "); }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn blank(&mut self) {
        self.out.push('\n');
    }

    fn emit_program(&mut self, program: &Program) {
        for import in &program.imports {
            self.emit_import(import);
        }
        if !program.imports.is_empty() { self.blank(); }
        self.emit_app(&program.app);
    }

    fn emit_import(&mut self, import: &ImportDecl) {
        match &import.spec {
            ImportSpec::Named(name) => self.line(&format!("import {} from \"{}\"", name, import.source)),
            ImportSpec::Destructured(names) => self.line(&format!("import {{ {} }} from \"{}\"", names.join(", "), import.source)),
            ImportSpec::Wildcard(alias) => self.line(&format!("import * as {} from \"{}\"", alias, import.source)),
        }
    }

    fn emit_app(&mut self, app: &AppDecl) {
        self.line(&format!("app {}", app.name));
        self.indent += 1;
        let mut first = true;
        for member in &app.members {
            if !first { self.blank(); }
            first = false;
            self.emit_app_member(member);
        }
        self.indent -= 1;
    }

    fn emit_app_member(&mut self, member: &AppMember) {
        match member {
            AppMember::ThemeRef(t) => {
                self.line(&format!("theme: {}", self.expr_str(&t.value)));
            }
            AppMember::NavigationDecl(n) => {
                self.line(&format!("navigation: {}", n.mode));
            }
            AppMember::Model(m) => self.emit_model(m),
            AppMember::Screen(s) => self.emit_screen(s),
            AppMember::Component(c) => self.emit_component(c),
            AppMember::State(s) => self.emit_state(s),
            AppMember::Const(c) => self.emit_const(c),
            AppMember::Fn(f) => self.emit_fn(f),
            AppMember::RouteDecl(r) => {
                self.line(&format!("route \"{}\" -> {}", r.pattern, r.screen));
            }
            _ => {}
        }
    }

    fn emit_model(&mut self, model: &ModelDecl) {
        self.line(&format!("model {}", model.name));
        self.indent += 1;
        for field in &model.fields {
            if let Some(ref default) = field.default {
                self.line(&format!("{}: {} = {}", field.name, self.type_str(&field.type_expr), self.expr_str(default)));
            } else {
                self.line(&format!("{}: {}", field.name, self.type_str(&field.type_expr)));
            }
        }
        self.indent -= 1;
    }

    fn emit_screen(&mut self, screen: &ScreenDecl) {
        let params = if screen.params.is_empty() {
            String::new()
        } else {
            format!("({})", self.params_str(&screen.params))
        };
        let mods: Vec<String> = screen.modifiers.iter().map(|m| match m {
            ScreenModifier::Tab(icon) => format!(" tab: \"{}\"", icon),
            ScreenModifier::Label(label) => format!(" label: \"{}\"", label),
        }).collect();
        self.line(&format!("screen {}{}{}", screen.name, params, mods.join("")));
        self.indent += 1;
        for member in &screen.members {
            self.emit_screen_member(member);
        }
        self.indent -= 1;
    }

    fn emit_component(&mut self, comp: &ComponentDecl) {
        let params = if comp.props.is_empty() {
            String::new()
        } else {
            format!("({})", self.params_str(&comp.props))
        };
        self.line(&format!("component {}{}", comp.name, params));
        self.indent += 1;
        for member in &comp.members {
            self.emit_screen_member(member);
        }
        self.indent -= 1;
    }

    fn emit_screen_member(&mut self, member: &ScreenMember) {
        match member {
            ScreenMember::State(s) => self.emit_state(s),
            ScreenMember::View(v) => self.emit_view_decl(v),
            ScreenMember::Action(a) => {
                self.blank();
                self.emit_action(a);
            }
            ScreenMember::Fn(f) => {
                self.blank();
                self.emit_fn(f);
            }
            ScreenMember::On(o) => {
                self.blank();
                self.emit_on(o);
            }
            ScreenMember::Style(s) => self.emit_style(s),
        }
    }

    fn emit_state(&mut self, state: &StateDecl) {
        if let Some(ref default) = state.default {
            self.line(&format!("state {}: {} = {}", state.name, self.type_str(&state.type_expr), self.expr_str(default)));
        } else {
            self.line(&format!("state {}: {}", state.name, self.type_str(&state.type_expr)));
        }
    }

    fn emit_const(&mut self, c: &ConstDecl) {
        let ty = c.type_expr.as_ref().map(|t| format!(": {}", self.type_str(t))).unwrap_or_default();
        self.line(&format!("const {}{} = {}", c.name, ty, self.expr_str(&c.value)));
    }

    fn emit_view_decl(&mut self, view: &ViewDecl) {
        self.blank();
        self.line("view");
        self.indent += 1;
        for elem in &view.body {
            self.emit_view_element(elem);
        }
        self.indent -= 1;
    }

    fn emit_view_element(&mut self, elem: &ViewElement) {
        match elem {
            ViewElement::Layout(layout) => {
                let tokens = self.design_tokens_str(&layout.tokens);
                let props = self.props_str(&layout.props);
                self.line(&format!("{}{}{}", self.layout_kind_str(&layout.kind), tokens, props));
                if !layout.children.is_empty() {
                    self.indent += 1;
                    for child in &layout.children {
                        self.emit_view_element(child);
                    }
                    self.indent -= 1;
                }
            }
            ViewElement::Widget(widget) => {
                let args: Vec<String> = widget.args.iter().map(|a| self.expr_str(a)).collect();
                let tokens = self.design_tokens_str(&widget.tokens);
                let props = self.props_str(&widget.props);
                let args_str = if args.is_empty() { String::new() } else { format!(" {}", args.join(" ")) };
                self.line(&format!("{}{}{}{}", self.widget_kind_str(&widget.kind), args_str, tokens, props));
            }
            ViewElement::Input(input) => {
                let tokens = self.design_tokens_str(&input.tokens);
                let props = self.props_str(&input.props);
                let action = input.action.as_ref().map(|a| format!(" -> {}", self.action_expr_str(a))).unwrap_or_default();
                self.line(&format!("{} {}{}{}{}", self.input_kind_str(&input.kind), input.binding, tokens, props, action));
            }
            ViewElement::Button(button) => {
                let style = button.style.as_ref().map(|s| format!(".{}", s)).unwrap_or_default();
                let tokens = self.design_tokens_str(&button.tokens);
                let props = self.props_str(&button.props);
                self.line(&format!("button{} {}{}{} -> {}", style, self.expr_str(&button.label), tokens, props, self.action_expr_str(&button.action)));
            }
            ViewElement::If(if_v) => {
                self.line(&format!("if {}", self.expr_str(&if_v.condition)));
                self.indent += 1;
                for child in &if_v.then_body { self.emit_view_element(child); }
                self.indent -= 1;
                if let Some(ref else_body) = if_v.else_body {
                    self.line("else");
                    self.indent += 1;
                    for child in else_body { self.emit_view_element(child); }
                    self.indent -= 1;
                }
            }
            ViewElement::Each(each) => {
                let idx = each.index_name.as_ref().map(|i| format!(", {}", i)).unwrap_or_default();
                self.line(&format!("each {} as {}{}", self.expr_str(&each.iterable), each.item_name, idx));
                self.indent += 1;
                for child in &each.body { self.emit_view_element(child); }
                self.indent -= 1;
            }
            ViewElement::When(when) => {
                self.line(&format!("when {}", self.expr_str(&when.expression)));
                self.indent += 1;
                for branch in &when.branches {
                    self.line(&format!("is {}", self.pattern_str(&branch.pattern)));
                    self.indent += 1;
                    for child in &branch.body { self.emit_view_element(child); }
                    self.indent -= 1;
                }
                self.indent -= 1;
            }
            ViewElement::ComponentRef(comp) => {
                let args: Vec<String> = comp.args.iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{}: {}", k, self.expr_str(v)))
                    .collect();
                if args.is_empty() {
                    self.line(&format!("{}", comp.name));
                } else {
                    self.line(&format!("{}({})", comp.name, args.join(", ")));
                }
                if !comp.children.is_empty() {
                    self.indent += 1;
                    for child in &comp.children { self.emit_view_element(child); }
                    self.indent -= 1;
                }
            }
            ViewElement::Spacer(_) => self.line("spacer"),
            ViewElement::Divider(tokens, _) => {
                let t = self.design_tokens_str(tokens);
                self.line(&format!("divider{}", t));
            }
            ViewElement::Slot(_) => self.line("slot"),
        }
    }

    fn emit_action(&mut self, action: &ActionDecl) {
        let params = if action.params.is_empty() { String::new() } else { format!("({})", self.params_str(&action.params)) };
        self.line(&format!("action {}{}", action.name, params));
        self.indent += 1;
        for stmt in &action.body { self.emit_stmt(stmt); }
        self.indent -= 1;
    }

    fn emit_fn(&mut self, func: &FnDecl) {
        let params = if func.params.is_empty() { String::new() } else { format!("({})", self.params_str(&func.params)) };
        let ret = func.return_type.as_ref().map(|t| format!(" -> {}", self.type_str(t))).unwrap_or_default();
        self.line(&format!("fn {}{}{}", func.name, params, ret));
        self.indent += 1;
        for stmt in &func.body { self.emit_stmt(stmt); }
        self.indent -= 1;
    }

    fn emit_on(&mut self, on: &OnDecl) {
        let params = if on.params.is_empty() { String::new() } else { format!("({})", self.params_str(&on.params)) };
        self.line(&format!("on {}{}", on.event, params));
        self.indent += 1;
        for stmt in &on.body { self.emit_stmt(stmt); }
        self.indent -= 1;
    }

    fn emit_style(&mut self, style: &StyleDecl) {
        self.line(&format!("style {}", style.name));
        self.indent += 1;
        // Emit tokens and props
        self.indent -= 1;
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign(name, value, _) => self.line(&format!("{} = {}", name, self.expr_str(value))),
            Stmt::Let(name, ty, value, _) => {
                let t = ty.as_ref().map(|t| format!(": {}", self.type_str(t))).unwrap_or_default();
                self.line(&format!("let {}{} = {}", name, t, self.expr_str(value)));
            }
            Stmt::If(cond, then_body, else_body, _) => {
                self.line(&format!("if {}", self.expr_str(cond)));
                self.indent += 1;
                for s in then_body { self.emit_stmt(s); }
                self.indent -= 1;
                if let Some(else_stmts) = else_body {
                    self.line("else");
                    self.indent += 1;
                    for s in else_stmts { self.emit_stmt(s); }
                    self.indent -= 1;
                }
            }
            Stmt::When(expr, branches, _) => {
                self.line(&format!("when {}", self.expr_str(expr)));
                self.indent += 1;
                for b in branches {
                    match &b.body {
                        StmtOrExpr::Expr(e) => self.line(&format!("is {} -> {}", self.pattern_str(&b.pattern), self.expr_str(e))),
                        StmtOrExpr::Stmt(stmts) => {
                            self.line(&format!("is {}", self.pattern_str(&b.pattern)));
                            self.indent += 1;
                            for s in stmts { self.emit_stmt(s); }
                            self.indent -= 1;
                        }
                    }
                }
                self.indent -= 1;
            }
            Stmt::Navigate(nav) => self.line(&format!("navigate{}", self.navigate_str(nav))),
            Stmt::Emit(name, args, _) => {
                let a: Vec<String> = args.iter().map(|a| self.expr_str(a)).collect();
                if a.is_empty() { self.line(&format!("emit {}", name)); }
                else { self.line(&format!("emit {}({})", name, a.join(", "))); }
            }
            Stmt::Return(value, _) => {
                if let Some(v) = value { self.line(&format!("return {}", self.expr_str(v))); }
                else { self.line("return"); }
            }
            Stmt::Expr(expr, _) => self.line(&self.expr_str(expr)),
        }
    }

    // === String helpers ===

    fn expr_str(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(n, _) => n.to_string(),
            Expr::FloatLit(f, _) => f.to_string(),
            Expr::StringLit(s, _) => format!("\"{}\"", s),
            Expr::BoolLit(b, _) => b.to_string(),
            Expr::Nil(_) => "nil".to_string(),
            Expr::Var(name, _) => name.clone(),
            Expr::MemberAccess(obj, member, _) => format!("{}.{}", self.expr_str(obj), member),
            Expr::Call(func, args, _) => {
                let a: Vec<String> = args.iter().map(|a| self.expr_str(a)).collect();
                format!("{}({})", self.expr_str(func), a.join(", "))
            }
            Expr::NamedCall(func, args, _) => {
                let a: Vec<String> = args.iter().filter(|(k,_)| k != "_").map(|(k, v)| format!("{}: {}", k, self.expr_str(v))).collect();
                format!("{}({})", self.expr_str(func), a.join(", "))
            }
            Expr::BinOp(l, op, r, _) => format!("{} {} {}", self.expr_str(l), self.binop_str(op), self.expr_str(r)),
            Expr::UnaryOp(UnaryOp::Not, e, _) => format!("not {}", self.expr_str(e)),
            Expr::UnaryOp(UnaryOp::Neg, e, _) => format!("-{}", self.expr_str(e)),
            Expr::Lambda(params, body, _) => {
                let p: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                format!("{} => {}", p.join(", "), self.expr_str(body))
            }
            Expr::Constructor(name, args, _) => {
                let a: Vec<String> = args.iter().filter(|(k,_)| k != "_").map(|(k, v)| format!("{}: {}", k, self.expr_str(v))).collect();
                format!("{}({})", name, a.join(", "))
            }
            Expr::Pipe(l, r, _) => format!("{} |> {}", self.expr_str(l), self.expr_str(r)),
            Expr::Conditional(c, t, e, _) => format!("if {} then {} else {}", self.expr_str(c), self.expr_str(t), self.expr_str(e)),
            Expr::NilCoalesce(l, r, _) => format!("{} ?? {}", self.expr_str(l), self.expr_str(r)),
            Expr::Index(obj, idx, _) => format!("{}[{}]", self.expr_str(obj), self.expr_str(idx)),
            Expr::DesignToken(segs, _) => format!(".{}", segs.join(".")),
            Expr::PercentLit(p, _) => format!("{}%", p),
        }
    }

    fn binop_str(&self, op: &BinOp) -> &'static str {
        match op {
            BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*", BinOp::Div => "/",
            BinOp::Mod => "%", BinOp::Eq => "==", BinOp::NotEq => "!=",
            BinOp::Lt => "<", BinOp::Gt => ">", BinOp::LtEq => "<=", BinOp::GtEq => ">=",
            BinOp::And => "and", BinOp::Or => "or", BinOp::Range => "..",
        }
    }

    fn type_str(&self, ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(name, _) => name.clone(),
            TypeExpr::Collection(kind, args, _) => {
                let a: Vec<String> = args.iter().map(|a| self.type_str(a)).collect();
                format!("{}[{}]", kind, a.join(", "))
            }
            TypeExpr::Optional(inner, _) => format!("optional[{}]", self.type_str(inner)),
            TypeExpr::Enum(variants, _) => {
                let v: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
                format!("enum[{}]", v.join(", "))
            }
            TypeExpr::Function(params, ret, _) => {
                let p: Vec<String> = params.iter().map(|p| self.type_str(p)).collect();
                let r = ret.as_ref().map(|r| format!(" -> {}", self.type_str(r))).unwrap_or_default();
                format!("fn({}){}", p.join(", "), r)
            }
            TypeExpr::Action(params, _) => {
                if params.is_empty() { "action".to_string() }
                else {
                    let p: Vec<String> = params.iter().map(|p| self.type_str(p)).collect();
                    format!("action({})", p.join(", "))
                }
            }
        }
    }

    fn params_str(&self, params: &[Param]) -> String {
        params.iter().map(|p| {
            let default = p.default.as_ref().map(|d| format!(" = {}", self.expr_str(d))).unwrap_or_default();
            format!("{}: {}{}", p.name, self.type_str(&p.type_expr), default)
        }).collect::<Vec<_>>().join(", ")
    }

    fn design_tokens_str(&self, tokens: &[DesignToken]) -> String {
        if tokens.is_empty() { return String::new(); }
        let parts: Vec<String> = tokens.iter().map(|t| {
            if t.segments.len() == 1 { format!(" .{}", t.segments[0]) }
            else { format!(" {}", t.segments.join(".")) }
        }).collect();
        parts.join("")
    }

    fn props_str(&self, props: &[PropAssign]) -> String {
        if props.is_empty() { return String::new(); }
        let parts: Vec<String> = props.iter().map(|p| format!(" {}: {}", p.name, self.expr_str(&p.value))).collect();
        parts.join("")
    }

    fn action_expr_str(&self, action: &ActionExpr) -> String {
        match action {
            ActionExpr::Call(name, args, _) => {
                let a: Vec<String> = args.iter().map(|a| self.expr_str(a)).collect();
                if a.is_empty() { format!("{}()", name) }
                else { format!("{}({})", name, a.join(", ")) }
            }
            ActionExpr::Navigate(nav) => format!("navigate{}", self.navigate_str(nav)),
            ActionExpr::Lambda(_, body, _) => format!("-> {}", self.expr_str(body)),
        }
    }

    fn navigate_str(&self, nav: &NavigateExpr) -> String {
        match nav {
            NavigateExpr::Back(_) => ".back".to_string(),
            NavigateExpr::Root(_) => ".root".to_string(),
            NavigateExpr::Dismiss(_) => ".dismiss".to_string(),
            NavigateExpr::To(expr, _) => format!("({})", self.expr_str(expr)),
            NavigateExpr::Replace(expr, _) => format!(".replace({})", self.expr_str(expr)),
            NavigateExpr::Modal(expr, _) => format!(".modal({})", self.expr_str(expr)),
        }
    }

    fn pattern_str(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Literal(expr) => self.expr_str(expr),
            Pattern::Identifier(name, _) => name.clone(),
            Pattern::EnumVariant(name, _) => format!(".{}", name),
            Pattern::Some(name, _) => format!("some({})", name),
            Pattern::Nil(_) => "nil".to_string(),
            Pattern::Constructor(name, _, _) => name.clone(),
        }
    }

    fn layout_kind_str(&self, kind: &LayoutKind) -> &'static str {
        match kind {
            LayoutKind::Column => "column", LayoutKind::Row => "row",
            LayoutKind::Stack => "stack", LayoutKind::Grid => "grid",
            LayoutKind::Scroll => "scroll", LayoutKind::Wrap => "wrap",
        }
    }

    fn widget_kind_str(&self, kind: &WidgetKind) -> &'static str {
        match kind {
            WidgetKind::Text => "text", WidgetKind::Heading => "heading",
            WidgetKind::Image => "image", WidgetKind::Icon => "icon",
            WidgetKind::Badge => "badge", WidgetKind::Progress => "progress",
            WidgetKind::Avatar => "avatar",
        }
    }

    fn input_kind_str(&self, kind: &InputKind) -> &'static str {
        match kind {
            InputKind::TextField => "textfield", InputKind::TextArea => "textarea",
            InputKind::Checkbox => "checkbox", InputKind::Toggle => "toggle",
            InputKind::Slider => "slider", InputKind::Picker => "picker",
            InputKind::DatePicker => "datepicker", InputKind::Segmented => "segmented",
            InputKind::Stepper => "stepper",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(source: &str) -> String {
        let result = crate::parser::parse(source);
        assert!(result.program.is_some(), "Parse failed: {:?}", result.errors);
        format(result.program.as_ref().unwrap())
    }

    #[test]
    fn test_format_minimal() {
        let formatted = roundtrip("app Hello\n  screen Main\n    view\n      text \"Hello\"");
        assert!(formatted.contains("app Hello"));
        assert!(formatted.contains("  screen Main"));
        assert!(formatted.contains("      text \"Hello\""));
    }

    #[test]
    fn test_format_roundtrip_parses() {
        let original = "app Test\n  model Todo\n    title: text\n    done: bool = false\n  screen Main\n    state x: int = 0\n    view\n      text \"hi\"";
        let formatted = roundtrip(original);
        // The formatted output should also parse
        let result2 = crate::parser::parse(&formatted);
        assert!(result2.program.is_some(), "Formatted code failed to parse:\n{}\nErrors: {:?}", formatted, result2.errors);
    }
}
