//! SwiftUI codegen: HIR → Swift source code

use aura_core::design;
use aura_core::hir::*;

pub struct SwiftOutput {
    pub swift: String,
    pub filename: String,
}

pub fn compile_to_swift(module: &HIRModule) -> SwiftOutput {
    let mut cg = SwiftCodegen::new(module);
    cg.generate();
    SwiftOutput {
        swift: cg.out,
        filename: format!("{}App.swift", module.app.name),
    }
}

struct SwiftCodegen<'a> {
    module: &'a HIRModule,
    out: String,
    indent: usize,
}

impl<'a> SwiftCodegen<'a> {
    fn new(module: &'a HIRModule) -> Self {
        Self {
            module,
            out: String::new(),
            indent: 0,
        }
    }

    fn line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn blank(&mut self) {
        self.out.push('\n');
    }

    fn generate(&mut self) {
        self.line("import SwiftUI");
        self.blank();

        // Models
        for model in &self.module.models {
            self.emit_model(model);
            self.blank();
        }

        // Screens
        for screen in &self.module.screens {
            self.emit_screen(screen);
            self.blank();
        }

        // Components
        for comp in &self.module.components {
            self.emit_component(comp);
            self.blank();
        }

        // App entry point
        self.line("@main");
        self.line(&format!("struct {}App: App {{", self.module.app.name));
        self.indent += 1;
        self.line("var body: some Scene {");
        self.indent += 1;
        self.line("WindowGroup {");
        self.indent += 1;
        if let Some(screen) = self.module.screens.first() {
            if self.module.app.navigation == NavigationMode::Tabs {
                self.line("TabView {");
                self.indent += 1;
                for s in &self.module.screens {
                    self.line(&format!("{}View()", s.name));
                    if let Some(ref tab) = s.tab {
                        self.indent += 1;
                        self.line(&format!(
                            ".tabItem {{ Label(\"{}\", systemImage: \"{}\") }}",
                            tab.label, tab.icon
                        ));
                        self.indent -= 1;
                    }
                }
                self.indent -= 1;
                self.line("}");
            } else {
                self.line("NavigationStack {");
                self.indent += 1;
                self.line(&format!("{}View()", screen.name));
                self.indent -= 1;
                self.line("}");
            }
        }
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
    }

    // === Models ===

    fn emit_model(&mut self, model: &HIRModel) {
        self.line(&format!("struct {}: Identifiable, Hashable {{", model.name));
        self.indent += 1;
        self.line("let id = UUID()");
        for field in &model.fields {
            let swift_type = self.type_to_swift(&field.field_type);
            if let Some(ref default) = field.default {
                self.line(&format!(
                    "var {}: {} = {}",
                    field.name,
                    swift_type,
                    self.expr_to_swift(default)
                ));
            } else {
                self.line(&format!("var {}: {}", field.name, swift_type));
            }
        }
        self.indent -= 1;
        self.line("}");
    }

    // === Screens ===

    fn emit_screen(&mut self, screen: &HIRScreen) {
        self.line(&format!("struct {}View: View {{", screen.name));
        self.indent += 1;

        // Params as properties
        for param in &screen.params {
            self.line(&format!(
                "let {}: {}",
                param.name,
                self.type_to_swift(&param.param_type)
            ));
        }

        // State
        for state in &screen.state {
            let swift_type = self.type_to_swift(&state.state_type);
            let default = state
                .initial
                .as_ref()
                .map(|e| self.expr_to_swift(e))
                .unwrap_or_else(|| self.default_value(&state.state_type));
            self.line(&format!(
                "@State private var {}: {} = {}",
                state.name, swift_type, default
            ));
        }

        self.blank();

        // Body
        self.line("var body: some View {");
        self.indent += 1;
        self.emit_view(&screen.view);
        self.indent -= 1;
        self.line("}");

        // Functions
        for func in &screen.functions {
            self.blank();
            self.emit_function(func);
        }

        // Actions as methods
        for action in &screen.actions {
            self.blank();
            self.emit_action(action);
        }

        self.indent -= 1;
        self.line("}");
    }

    // === Components ===

    fn emit_component(&mut self, comp: &HIRComponent) {
        self.line(&format!("struct {}View: View {{", comp.name));
        self.indent += 1;

        for prop in &comp.props {
            let swift_type = self.type_to_swift(&prop.param_type);
            if let Some(ref default) = prop.default {
                self.line(&format!(
                    "var {}: {} = {}",
                    prop.name,
                    swift_type,
                    self.expr_to_swift(default)
                ));
            } else {
                self.line(&format!("let {}: {}", prop.name, swift_type));
            }
        }

        for state in &comp.state {
            let swift_type = self.type_to_swift(&state.state_type);
            let default = state
                .initial
                .as_ref()
                .map(|e| self.expr_to_swift(e))
                .unwrap_or_else(|| self.default_value(&state.state_type));
            self.line(&format!(
                "@State private var {}: {} = {}",
                state.name, swift_type, default
            ));
        }

        self.blank();
        self.line("var body: some View {");
        self.indent += 1;
        self.emit_view(&comp.view);
        self.indent -= 1;
        self.line("}");

        for func in &comp.functions {
            self.blank();
            self.emit_function(func);
        }

        for action in &comp.actions {
            self.blank();
            self.emit_action(action);
        }

        self.indent -= 1;
        self.line("}");
    }

    // === View emission ===

    fn emit_view(&mut self, view: &HIRView) {
        match view {
            HIRView::Column(layout) => {
                let spacing = self.spacing_value(&layout.design);
                self.line(&format!("VStack(spacing: {}) {{", spacing));
                self.indent += 1;
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
                self.emit_design_modifiers(&layout.design);
            }
            HIRView::Row(layout) => {
                let spacing = self.spacing_value(&layout.design);
                self.line(&format!("HStack(spacing: {}) {{", spacing));
                self.indent += 1;
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
                self.emit_design_modifiers(&layout.design);
            }
            HIRView::Stack(layout) => {
                self.line("ZStack {");
                self.indent += 1;
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
                self.emit_design_modifiers(&layout.design);
            }
            HIRView::Grid(grid) => {
                self.line("LazyVGrid(columns: [GridItem(.adaptive(minimum: 160))]) {");
                self.indent += 1;
                for child in &grid.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
                self.emit_design_modifiers(&grid.design);
            }
            HIRView::Scroll(scroll) => {
                let axis = match scroll.direction {
                    ScrollDirection::Horizontal => ".horizontal",
                    _ => ".vertical",
                };
                self.line(&format!("ScrollView({}) {{", axis));
                self.indent += 1;
                for child in &scroll.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
                self.emit_design_modifiers(&scroll.design);
            }
            HIRView::Text(text) => {
                let content = self.expr_to_swift(&text.content);
                self.line(&format!("Text({})", content));
                self.emit_text_modifiers(&text.design);
            }
            HIRView::Heading(heading) => {
                let content = self.expr_to_swift(&heading.content);
                self.line(&format!("Text({})", content));
                self.indent += 1;
                self.line(&format!(
                    ".font(.title{})",
                    if heading.level <= 1 {
                        ""
                    } else if heading.level == 2 {
                        "2"
                    } else {
                        "3"
                    }
                ));
                self.line(".fontWeight(.bold)");
                self.indent -= 1;
                self.emit_text_modifiers(&heading.design);
            }
            HIRView::Image(image) => {
                let src = self.expr_to_swift(&image.source);
                self.line(&format!(
                    "AsyncImage(url: URL(string: {})) {{ image in",
                    src
                ));
                self.indent += 1;
                self.line("image.resizable().aspectRatio(contentMode: .fill)");
                self.indent -= 1;
                self.line("} placeholder: {");
                self.indent += 1;
                self.line("ProgressView()");
                self.indent -= 1;
                self.line("}");
                self.emit_design_modifiers(&image.design);
            }
            HIRView::Icon(icon) => {
                let name = self.expr_to_swift(&icon.name);
                self.line(&format!("Image(systemName: {})", name));
                self.emit_design_modifiers(&icon.design);
            }
            HIRView::Badge(badge) => {
                let content = self.expr_to_swift(&badge.content);
                self.line(&format!("Text({})", content));
                self.indent += 1;
                self.line(".font(.caption2).fontWeight(.semibold)");
                self.line(".padding(.horizontal, 6).padding(.vertical, 2)");
                self.line(".background(Color.accentColor).foregroundColor(.white)");
                self.line(".clipShape(Capsule())");
                self.indent -= 1;
            }
            HIRView::Progress(_) => {
                self.line("ProgressView()");
            }
            HIRView::Avatar(avatar) => {
                let src = self.expr_to_swift(&avatar.source);
                self.line(&format!(
                    "AsyncImage(url: URL(string: {})) {{ image in",
                    src
                ));
                self.indent += 1;
                self.line("image.resizable().aspectRatio(contentMode: .fill)");
                self.indent -= 1;
                self.line("} placeholder: {");
                self.indent += 1;
                self.line("ProgressView()");
                self.indent -= 1;
                self.line("}");
                self.indent += 1;
                self.line(".frame(width: 48, height: 48)");
                self.line(".clipShape(Circle())");
                self.indent -= 1;
                self.emit_design_modifiers(&avatar.design);
            }
            HIRView::Button(button) => {
                let label = self.expr_to_swift(&button.label);
                let action = self.action_expr_to_swift(&button.action);
                match button.style {
                    ButtonStyle::Icon => {
                        self.line(&format!("Button(action: {{ {} }}) {{", action));
                        self.indent += 1;
                        self.line(&format!("Image(systemName: {})", label));
                        self.indent -= 1;
                        self.line("}");
                    }
                    _ => {
                        self.line(&format!("Button({}, action: {{ {} }})", label, action));
                    }
                }
                self.emit_button_modifiers(&button.design, &button.style);
            }
            HIRView::TextField(field) => {
                let placeholder = field.placeholder.as_deref().unwrap_or("");
                self.line(&format!(
                    "TextField(\"{}\", text: ${})",
                    placeholder, field.binding
                ));
                self.emit_design_modifiers(&field.design);
            }
            HIRView::TextArea(area) => {
                self.line(&format!("TextEditor(text: ${})", area.binding));
                self.indent += 1;
                self.line(".frame(minHeight: 120)");
                self.indent -= 1;
                self.emit_design_modifiers(&area.design);
            }
            HIRView::Checkbox(cb) => {
                self.line(&format!("Toggle(isOn: ${}) {{", cb.binding));
                self.indent += 1;
                self.line("EmptyView()");
                self.indent -= 1;
                self.line("}");
                self.line(".toggleStyle(.checkbox)");
            }
            HIRView::Toggle(toggle) => {
                let label = toggle.label.as_deref().unwrap_or("");
                self.line(&format!("Toggle(\"{}\", isOn: ${})", label, toggle.binding));
            }
            HIRView::Slider(slider) => {
                self.line(&format!(
                    "Slider(value: ${}, in: {}...{}, step: {})",
                    slider.binding, slider.min, slider.max, slider.step
                ));
            }
            HIRView::Picker(picker) => {
                self.line(&format!("Picker(\"\", selection: ${}) {{", picker.binding));
                self.indent += 1;
                self.line(&format!(
                    "ForEach({}, id: \\.self) {{ option in",
                    self.expr_to_swift(&picker.options)
                ));
                self.indent += 1;
                self.line("Text(String(describing: option)).tag(option)");
                self.indent -= 1;
                self.line("}");
                self.indent -= 1;
                self.line("}");
                self.emit_design_modifiers(&picker.design);
            }
            HIRView::DatePicker(date_picker) => {
                let label = date_picker.label.as_deref().unwrap_or("");
                self.line(&format!(
                    "DatePicker(\"{}\", selection: ${}, displayedComponents: [.date])",
                    label, date_picker.binding
                ));
                self.emit_design_modifiers(&date_picker.design);
            }
            HIRView::Segmented(seg) => {
                self.line(&format!("Picker(\"\", selection: ${}) {{", seg.binding));
                self.indent += 1;
                self.line(&format!(
                    "ForEach({}, id: \\.self) {{ option in",
                    self.expr_to_swift(&seg.options)
                ));
                self.indent += 1;
                self.line("Text(String(describing: option)).tag(option)");
                self.indent -= 1;
                self.line("}");
                self.indent -= 1;
                self.line("}");
                self.line(".pickerStyle(.segmented)");
                self.emit_design_modifiers(&seg.design);
            }
            HIRView::Spacer => {
                self.line("Spacer()");
            }
            HIRView::Divider(_) => {
                self.line("Divider()");
            }
            HIRView::Conditional(cond) => {
                let condition = self.expr_to_swift(&cond.condition);
                self.line(&format!("if {} {{", condition));
                self.indent += 1;
                self.emit_view(&cond.then_view);
                self.indent -= 1;
                if let Some(ref else_view) = cond.else_view {
                    self.line("} else {");
                    self.indent += 1;
                    self.emit_view(else_view);
                    self.indent -= 1;
                }
                self.line("}");
            }
            HIRView::Each(each) => {
                let iterable = self.expr_to_swift(&each.iterable);
                self.line(&format!(
                    "ForEach({}, id: \\.self) {{ {} in",
                    iterable, each.item_name
                ));
                self.indent += 1;
                self.emit_view(&each.body);
                self.indent -= 1;
                self.line("}");
            }
            HIRView::Switch(switch) => {
                let expr = self.expr_to_swift(&switch.expression);
                self.line(&format!("switch {} {{", expr));
                for case in &switch.cases {
                    self.line(&format!("case {:?}:", case.pattern));
                    self.indent += 1;
                    self.emit_view(&case.view);
                    self.indent -= 1;
                }
                self.line("}");
            }
            HIRView::ComponentRef(comp_ref) => {
                let args: Vec<String> = comp_ref
                    .args
                    .iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_swift(v)))
                    .collect();
                if args.is_empty() {
                    self.line(&format!("{}View()", comp_ref.name));
                } else {
                    self.line(&format!("{}View({})", comp_ref.name, args.join(", ")));
                }
            }
            HIRView::Group(children) => {
                if children.len() == 1 {
                    self.emit_view(&children[0]);
                } else {
                    self.line("Group {");
                    self.indent += 1;
                    for child in children {
                        self.emit_view(child);
                    }
                    self.indent -= 1;
                    self.line("}");
                }
            }
            HIRView::Slot => {
                self.line("// Slot (child content)");
            }
            _ => {
                self.line("EmptyView() // unsupported");
            }
        }
    }

    // === Functions & Actions ===

    fn emit_function(&mut self, func: &HIRFunction) {
        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_to_swift(&p.param_type)))
            .collect();
        let ret = self.type_to_swift(&func.return_type);
        self.line(&format!(
            "func {}({}) -> {} {{",
            func.name,
            params.join(", "),
            ret
        ));
        self.indent += 1;
        for stmt in &func.body {
            self.emit_stmt(stmt);
        }
        self.indent -= 1;
        self.line("}");
    }

    fn emit_action(&mut self, action: &HIRAction) {
        let params: Vec<String> = action
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_to_swift(&p.param_type)))
            .collect();
        self.line(&format!("func {}({}) {{", action.name, params.join(", ")));
        self.indent += 1;
        for stmt in &action.body {
            self.emit_stmt(stmt);
        }
        self.indent -= 1;
        self.line("}");
    }

    fn emit_stmt(&mut self, stmt: &HIRStmt) {
        match stmt {
            HIRStmt::Assign(name, value) => {
                self.line(&format!("{} = {}", name, self.expr_to_swift(value)));
            }
            HIRStmt::Let(name, _, value) => {
                self.line(&format!("let {} = {}", name, self.expr_to_swift(value)));
            }
            HIRStmt::If(cond, then_body, else_body) => {
                self.line(&format!("if {} {{", self.expr_to_swift(cond)));
                self.indent += 1;
                for s in then_body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                if let Some(else_stmts) = else_body {
                    self.line("} else {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                }
                self.line("}");
            }
            HIRStmt::Return(Some(value)) => {
                self.line(&format!("return {}", self.expr_to_swift(value)));
            }
            HIRStmt::Return(None) => {
                self.line("return");
            }
            HIRStmt::Expr(expr) => {
                self.line(&self.expr_to_swift(expr));
            }
            _ => {
                self.line("// unsupported statement");
            }
        }
    }

    // === Design modifiers ===

    fn emit_design_modifiers(&mut self, design: &design::ResolvedDesign) {
        self.indent += 1;
        if let Some(ref spacing) = design.spacing {
            if spacing.padding_top.is_some() {
                let p = spacing.padding_top.unwrap_or(0.0);
                if spacing.padding_top == spacing.padding_bottom
                    && spacing.padding_left == spacing.padding_right
                    && spacing.padding_top == spacing.padding_left
                {
                    self.line(&format!(".padding({})", p));
                } else {
                    if let Some(v) = spacing.padding_top {
                        self.line(&format!(".padding(.top, {})", v));
                    }
                    if let Some(v) = spacing.padding_bottom {
                        self.line(&format!(".padding(.bottom, {})", v));
                    }
                    if let Some(v) = spacing.padding_left {
                        self.line(&format!(".padding(.leading, {})", v));
                    }
                    if let Some(v) = spacing.padding_right {
                        self.line(&format!(".padding(.trailing, {})", v));
                    }
                }
            }
        }
        if let Some(ref color) = design.color {
            if let Some(ref fg) = color.foreground {
                self.line(&format!(".foregroundColor(.{})", self.semantic_color(fg)));
            }
            if let Some(ref bg) = color.background {
                self.line(&format!(".background(Color(.{}))", self.semantic_color(bg)));
            }
        }
        if let Some(ref shape) = design.shape {
            match shape.kind {
                design::ShapeKind::Circle => self.line(".clipShape(Circle())"),
                design::ShapeKind::Pill => self.line(&format!(".clipShape(Capsule())")),
                _ if shape.radius > 0.0 => self.line(&format!(".cornerRadius({})", shape.radius)),
                _ => {}
            }
        }
        self.indent -= 1;
    }

    fn emit_text_modifiers(&mut self, design: &design::ResolvedDesign) {
        self.indent += 1;
        if let Some(ref typo) = design.typography {
            if let Some(size) = typo.size {
                let swift_size = (size * 16.0).round(); // Convert rem to pt
                self.line(&format!(".font(.system(size: {}))", swift_size));
            }
            if let Some(weight) = typo.weight {
                let w = match weight {
                    100 => ".ultraLight",
                    300 => ".light",
                    400 => ".regular",
                    500 => ".medium",
                    600 => ".semibold",
                    700 => ".bold",
                    800 => ".heavy",
                    900 => ".black",
                    _ => ".regular",
                };
                self.line(&format!(".fontWeight({})", w));
            }
            if typo.italic {
                self.line(".italic()");
            }
            if typo.strikethrough {
                self.line(".strikethrough()");
            }
            if typo.underline {
                self.line(".underline()");
            }
            if typo.mono {
                self.line(".monospaced()");
            }
            if let Some(align) = typo.alignment {
                let a = match align {
                    design::TextAlignment::Center => ".center",
                    design::TextAlignment::Trailing => ".trailing",
                    design::TextAlignment::Leading => ".leading",
                };
                self.line(&format!(".multilineTextAlignment({})", a));
            }
        }
        if let Some(ref color) = design.color {
            if let Some(ref fg) = color.foreground {
                self.line(&format!(".foregroundColor(.{})", self.semantic_color(fg)));
            }
        }
        self.indent -= 1;
    }

    fn emit_button_modifiers(&mut self, design: &design::ResolvedDesign, style: &ButtonStyle) {
        self.indent += 1;
        match style {
            ButtonStyle::Default => {
                self.line(".buttonStyle(.borderedProminent)");
            }
            ButtonStyle::Outline => {
                self.line(".buttonStyle(.bordered)");
            }
            _ => {}
        }
        if let Some(ref color) = design.color {
            if let Some(ref fg) = color.foreground {
                self.line(&format!(".tint(.{})", self.semantic_color(fg)));
            }
        }
        if let Some(ref shape) = design.shape {
            if shape.kind == design::ShapeKind::Pill {
                self.line(".clipShape(Capsule())");
            }
        }
        self.indent -= 1;
    }

    // === Expression emission ===

    fn expr_to_swift(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => {
                if s.contains('{') && s.contains('}') {
                    let interpolated = s.replace('{', "\\(").replace('}', ")");
                    format!("\"{}\"", interpolated)
                } else {
                    format!("\"{}\"", s)
                }
            }
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::FloatLit(f) => f.to_string(),
            HIRExpr::PercentLit(p) => p.to_string(),
            HIRExpr::BoolLit(b) => b.to_string(),
            HIRExpr::Nil => "nil".to_string(),
            HIRExpr::Var(name, _) => name.clone(),
            HIRExpr::MemberAccess(obj, member, _) => {
                format!("{}.{}", self.expr_to_swift(obj), member)
            }
            HIRExpr::Call(func, args, _) => {
                let f = self.expr_to_swift(func);
                let a: Vec<String> = args.iter().map(|a| self.expr_to_swift(a)).collect();
                format!("{}({})", f, a.join(", "))
            }
            HIRExpr::NamedCall(func, args, _) => {
                let f = self.expr_to_swift(func);
                let a: Vec<String> = args
                    .iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_swift(v)))
                    .collect();
                format!("{}({})", f, a.join(", "))
            }
            HIRExpr::BinOp(left, op, right, _) => {
                let l = self.expr_to_swift(left);
                let r = self.expr_to_swift(right);
                let op_str = match op {
                    aura_core::ast::BinOp::Add => "+",
                    aura_core::ast::BinOp::Sub => "-",
                    aura_core::ast::BinOp::Mul => "*",
                    aura_core::ast::BinOp::Div => "/",
                    aura_core::ast::BinOp::Mod => "%",
                    aura_core::ast::BinOp::Eq => "==",
                    aura_core::ast::BinOp::NotEq => "!=",
                    aura_core::ast::BinOp::Lt => "<",
                    aura_core::ast::BinOp::Gt => ">",
                    aura_core::ast::BinOp::LtEq => "<=",
                    aura_core::ast::BinOp::GtEq => ">=",
                    aura_core::ast::BinOp::And => "&&",
                    aura_core::ast::BinOp::Or => "||",
                    _ => "/* ? */",
                };
                format!("({} {} {})", l, op_str, r)
            }
            HIRExpr::UnaryOp(op, operand, _) => {
                let o = self.expr_to_swift(operand);
                match op {
                    aura_core::ast::UnaryOp::Not => format!("!{}", o),
                    aura_core::ast::UnaryOp::Neg => format!("-{}", o),
                }
            }
            HIRExpr::Constructor(name, args, _) => {
                if name == "list" {
                    let items: Vec<String> = args
                        .iter()
                        .map(|(_, value)| self.expr_to_swift(value))
                        .collect();
                    return format!("[{}]", items.join(", "));
                }
                let fields: Vec<String> = args
                    .iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_swift(v)))
                    .collect();
                format!("{}({})", name, fields.join(", "))
            }
            HIRExpr::Lambda(params, body, _) => {
                let p: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
                format!("{{ {} in {} }}", p.join(", "), self.expr_to_swift(body))
            }
            HIRExpr::Conditional(condition, then_expr, else_expr, _) => format!(
                "({} ? {} : {})",
                self.expr_to_swift(condition),
                self.expr_to_swift(then_expr),
                self.expr_to_swift(else_expr)
            ),
            HIRExpr::NilCoalesce(left, right, _) => format!(
                "({} ?? {})",
                self.expr_to_swift(left),
                self.expr_to_swift(right)
            ),
            _ => "/* expr */".to_string(),
        }
    }

    fn action_expr_to_swift(&self, action: &HIRActionExpr) -> String {
        match action {
            HIRActionExpr::Call(name, args) => {
                let a: Vec<String> = args.iter().map(|a| self.expr_to_swift(a)).collect();
                if a.is_empty() {
                    format!("{}()", name)
                } else {
                    format!("{}({})", name, a.join(", "))
                }
            }
            HIRActionExpr::Navigate(nav) => match nav {
                HIRNavigate::Back => "dismiss()".to_string(),
                _ => "/* navigate */".to_string(),
            },
            HIRActionExpr::Sequence(actions) => actions
                .iter()
                .map(|a| self.action_expr_to_swift(a))
                .collect::<Vec<_>>()
                .join("; "),
        }
    }

    // === Type mapping ===

    fn type_to_swift(&self, ty: &aura_core::types::AuraType) -> String {
        use aura_core::types::*;
        match ty {
            AuraType::Primitive(p) => match p {
                PrimitiveType::Text => "String".to_string(),
                PrimitiveType::Int => "Int".to_string(),
                PrimitiveType::Float => "Double".to_string(),
                PrimitiveType::Bool => "Bool".to_string(),
                PrimitiveType::Timestamp => "Date".to_string(),
                PrimitiveType::Duration => "TimeInterval".to_string(),
                PrimitiveType::Percent => "Double".to_string(),
            },
            AuraType::Security(s) => match s {
                SecurityType::Secret => "String".to_string(),
                SecurityType::Sanitized => "String".to_string(),
                SecurityType::Email => "String".to_string(),
                SecurityType::Url => "URL".to_string(),
                SecurityType::Token => "String".to_string(),
            },
            AuraType::List(inner) => format!("[{}]", self.type_to_swift(inner)),
            AuraType::Set(inner) => format!("Set<{}>", self.type_to_swift(inner)),
            AuraType::Map(k, v) => {
                format!("[{}: {}]", self.type_to_swift(k), self.type_to_swift(v))
            }
            AuraType::Optional(inner) => format!("{}?", self.type_to_swift(inner)),
            AuraType::Named(name) => name.clone(),
            AuraType::Action(_) => "(() -> Void)".to_string(),
            AuraType::Enum(variants) => {
                // Inline enum — use String for now
                "String".to_string()
            }
            _ => "Any".to_string(),
        }
    }

    fn default_value(&self, ty: &aura_core::types::AuraType) -> String {
        use aura_core::types::*;
        match ty {
            AuraType::Primitive(PrimitiveType::Text) => "\"\"".to_string(),
            AuraType::Primitive(PrimitiveType::Int) => "0".to_string(),
            AuraType::Primitive(PrimitiveType::Float) => "0.0".to_string(),
            AuraType::Primitive(PrimitiveType::Bool) => "false".to_string(),
            AuraType::List(_) => "[]".to_string(),
            AuraType::Map(_, _) => "[:]".to_string(),
            AuraType::Optional(_) => "nil".to_string(),
            _ => "\"\"".to_string(),
        }
    }

    fn spacing_value(&self, design: &design::ResolvedDesign) -> String {
        design
            .spacing
            .as_ref()
            .and_then(|s| s.gap)
            .map(|g| format!("{}", g))
            .unwrap_or_else(|| "8".to_string())
    }

    fn semantic_color(&self, name: &str) -> String {
        match name {
            "accent" => "accentColor".to_string(),
            "primary" => "primary".to_string(),
            "secondary" => "secondary".to_string(),
            "muted" => "gray".to_string(),
            "danger" => "red".to_string(),
            "warning" => "orange".to_string(),
            "success" => "green".to_string(),
            "info" => "blue".to_string(),
            "surface" => "systemBackground".to_string(),
            "background" => "systemGroupedBackground".to_string(),
            _ => name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_source(source: &str) -> SwiftOutput {
        let result = aura_core::parser::parse(source);
        assert!(
            result.errors.is_empty(),
            "Parse errors: {:?}",
            result.errors
        );
        let hir = aura_core::hir::build_hir(result.program.as_ref().unwrap());
        compile_to_swift(&hir)
    }

    #[test]
    fn test_minimal_swift() {
        let output =
            compile_source("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
        assert!(output.swift.contains("import SwiftUI"));
        assert!(output.swift.contains("struct MainView: View"));
        assert!(output.swift.contains("Text(\"Hello, Aura!\")"));
        assert!(output.swift.contains("struct HelloApp: App"));
    }

    #[test]
    fn test_state_generates_at_state() {
        let output = compile_source(
            "\
app Test
  screen Main
    state count: int = 0
    view
      text \"hi\"",
        );
        assert!(output.swift.contains("@State private var count: Int = 0"));
    }

    #[test]
    fn test_model_generates_struct() {
        let output = compile_source(
            "\
app Test
  model Todo
    title: text
    done: bool = false
  screen Main
    view
      text \"hi\"",
        );
        assert!(output.swift.contains("struct Todo: Identifiable, Hashable"));
        assert!(output.swift.contains("var title: String"));
        assert!(output.swift.contains("var done: Bool = false"));
    }

    #[test]
    fn test_button_generates_swift_button() {
        let output = compile_source(
            "\
app Test
  screen Main
    view
      button \"Save\" .accent -> save()
    action save
      return",
        );
        assert!(output.swift.contains("Button(\"Save\""));
        assert!(output.swift.contains("save()"));
    }

    #[test]
    fn test_layout_generates_stacks() {
        let output = compile_source(
            "\
app Test
  screen Main
    view
      column gap.md padding.lg
        row gap.sm
          text \"A\"
          text \"B\"",
        );
        assert!(output.swift.contains("VStack(spacing: 8)"));
        assert!(output.swift.contains("HStack(spacing: 4)"));
        assert!(output.swift.contains(".padding(16)"));
    }

    #[test]
    fn test_navigation_tabs() {
        let output = compile_source(
            "\
app Test
  navigation: tabs
  screen Home tab: \"house\" label: \"Home\"
    view
      text \"Home\"
  screen Settings tab: \"gear\" label: \"Settings\"
    view
      text \"Settings\"",
        );
        assert!(output.swift.contains("TabView"));
        assert!(output.swift.contains(".tabItem"));
        assert!(output.swift.contains("house"));
    }

    #[test]
    fn test_each_generates_foreach() {
        let output = compile_source(
            "\
app Test
  screen Main
    state items: list[text] = []
    view
      each items as item
        text item",
        );
        assert!(output.swift.contains("ForEach(items"));
    }

    #[test]
    fn test_segmented_generates_segmented_picker() {
        let output = compile_source(
            "\
app Test
  screen Main
    state filter: text = \"All\"
    view
      segmented filter options: [\"All\", \"Active\", \"Done\"]",
        );
        assert!(output.swift.contains("Picker(\"\", selection: $filter)"));
        assert!(output.swift.contains("ForEach([\"All\", \"Active\", \"Done\"], id: \\.self)"));
        assert!(output.swift.contains(".pickerStyle(.segmented)"));
    }

    #[test]
    fn test_avatar_generates_async_image() {
        let output = compile_source(
            "\
app Test
  screen Main
    view
      avatar \"https://example.com/avatar.png\" .circle",
        );
        assert!(output
            .swift
            .contains("AsyncImage(url: URL(string: \"https://example.com/avatar.png\"))"));
        assert!(output.swift.contains(".clipShape(Circle())"));
    }
}
