//! Jetpack Compose codegen: HIR → Kotlin source code

use aura_core::design;
use aura_core::hir::*;

pub struct ComposeOutput {
    pub kotlin: String,
    pub filename: String,
}

pub fn compile_to_compose(module: &HIRModule) -> ComposeOutput {
    let mut cg = ComposeCodegen::new(module);
    cg.generate();
    ComposeOutput {
        kotlin: cg.out,
        filename: format!("{}Activity.kt", module.app.name),
    }
}

struct ComposeCodegen<'a> {
    module: &'a HIRModule,
    out: String,
    indent: usize,
}

impl<'a> ComposeCodegen<'a> {
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
        self.line(&format!(
            "package com.aura.{}",
            self.module.app.name.to_lowercase()
        ));
        self.blank();
        self.line("import android.os.Bundle");
        self.line("import androidx.activity.ComponentActivity");
        self.line("import androidx.activity.compose.setContent");
        self.line("import androidx.compose.foundation.layout.*");
        self.line("import androidx.compose.foundation.lazy.*");
        self.line("import androidx.compose.foundation.rememberScrollState");
        self.line("import androidx.compose.foundation.verticalScroll");
        self.line("import androidx.compose.material3.*");
        self.line("import androidx.compose.runtime.*");
        self.line("import androidx.compose.ui.*");
        self.line("import androidx.compose.ui.text.font.FontWeight");
        self.line("import androidx.compose.ui.unit.dp");
        self.line("import androidx.compose.ui.unit.sp");
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

        // Activity
        self.line(&format!(
            "class {}Activity : ComponentActivity() {{",
            self.module.app.name
        ));
        self.indent += 1;
        self.line("override fun onCreate(savedInstanceState: Bundle?) {");
        self.indent += 1;
        self.line("super.onCreate(savedInstanceState)");
        self.line("setContent {");
        self.indent += 1;
        self.line("MaterialTheme {");
        self.indent += 1;
        if let Some(screen) = self.module.screens.first() {
            self.line(&format!("{}Screen()", screen.name));
        }
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
    }

    fn emit_model(&mut self, model: &HIRModel) {
        let fields: Vec<String> = model
            .fields
            .iter()
            .map(|f| {
                let kt = self.type_to_kotlin(&f.field_type);
                if let Some(ref default) = f.default {
                    format!("val {}: {} = {}", f.name, kt, self.expr_to_kotlin(default))
                } else {
                    format!("val {}: {}", f.name, kt)
                }
            })
            .collect();
        self.line(&format!("data class {}(", model.name));
        self.indent += 1;
        for (i, field) in fields.iter().enumerate() {
            let comma = if i < fields.len() - 1 { "," } else { "" };
            self.line(&format!("{}{}", field, comma));
        }
        self.indent -= 1;
        self.line(")");
    }

    fn emit_screen(&mut self, screen: &HIRScreen) {
        self.line("@Composable");
        let params: Vec<String> = screen
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_to_kotlin(&p.param_type)))
            .collect();
        self.line(&format!(
            "fun {}Screen({}) {{",
            screen.name,
            params.join(", ")
        ));
        self.indent += 1;

        // State
        for state in &screen.state {
            let kt = self.type_to_kotlin(&state.state_type);
            let default = state
                .initial
                .as_ref()
                .map(|e| self.expr_to_kotlin(e))
                .unwrap_or_else(|| self.default_value(&state.state_type));
            self.line(&format!(
                "var {} by remember {{ mutableStateOf<{}>({}){} }}",
                state.name, kt, default, ""
            ));
        }
        if !screen.state.is_empty() {
            self.blank();
        }

        // Helper functions (inline)
        for func in &screen.functions {
            self.emit_function(func);
            self.blank();
        }

        // View
        self.emit_view(&screen.view);

        self.indent -= 1;
        self.line("}");

        // Actions as standalone functions if needed (simplified: inline above)
    }

    fn emit_component(&mut self, comp: &HIRComponent) {
        self.line("@Composable");
        let props: Vec<String> = comp
            .props
            .iter()
            .map(|p| {
                let kt = self.type_to_kotlin(&p.param_type);
                if let Some(ref default) = p.default {
                    format!("{}: {} = {}", p.name, kt, self.expr_to_kotlin(default))
                } else {
                    format!("{}: {}", p.name, kt)
                }
            })
            .collect();
        self.line(&format!("fun {}({}) {{", comp.name, props.join(", ")));
        self.indent += 1;

        for state in &comp.state {
            let kt = self.type_to_kotlin(&state.state_type);
            let default = state
                .initial
                .as_ref()
                .map(|e| self.expr_to_kotlin(e))
                .unwrap_or_else(|| self.default_value(&state.state_type));
            self.line(&format!(
                "var {} by remember {{ mutableStateOf<{}>({}) }}",
                state.name, kt, default
            ));
        }

        self.emit_view(&comp.view);

        self.indent -= 1;
        self.line("}");
    }

    fn emit_view(&mut self, view: &HIRView) {
        match view {
            HIRView::Column(layout) => {
                let mods = self.layout_modifier(&layout.design);
                let spacing = self.spacing_dp(&layout.design);
                self.line(&format!(
                    "Column({}verticalArrangement = Arrangement.spacedBy({}.dp)) {{",
                    mods, spacing
                ));
                self.indent += 1;
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
            }
            HIRView::Row(layout) => {
                let mods = self.layout_modifier(&layout.design);
                let spacing = self.spacing_dp(&layout.design);
                self.line(&format!(
                    "Row({}horizontalArrangement = Arrangement.spacedBy({}.dp)) {{",
                    mods, spacing
                ));
                self.indent += 1;
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
            }
            HIRView::Stack(layout) => {
                self.line("Box {");
                self.indent += 1;
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
            }
            HIRView::Grid(grid) => {
                self.line("LazyVerticalGrid(columns = GridCells.Adaptive(160.dp)) {");
                self.indent += 1;
                self.line("items(/* collection */) { item ->");
                self.indent += 1;
                for child in &grid.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
                self.indent -= 1;
                self.line("}");
            }
            HIRView::Scroll(scroll) => {
                self.line("Column(modifier = Modifier.verticalScroll(rememberScrollState())) {");
                self.indent += 1;
                for child in &scroll.children {
                    self.emit_view(child);
                }
                self.indent -= 1;
                self.line("}");
            }
            HIRView::Text(text) => {
                let content = self.expr_to_kotlin(&text.content);
                let mods = self.text_modifiers(&text.design);
                self.line(&format!("Text(text = {}{})", content, mods));
            }
            HIRView::Heading(heading) => {
                let content = self.expr_to_kotlin(&heading.content);
                let size = match heading.level {
                    1 => "28",
                    2 => "24",
                    3 => "20",
                    _ => "18",
                };
                self.line(&format!(
                    "Text(text = {}, fontSize = {}.sp, fontWeight = FontWeight.Bold)",
                    content, size
                ));
            }
            HIRView::Image(image) => {
                self.line("// AsyncImage placeholder");
                self.line("Box(modifier = Modifier.fillMaxWidth()) { /* Image */ }");
            }
            HIRView::Icon(icon) => {
                let name = self.expr_to_kotlin(&icon.name);
                self.line(&format!(
                    "Icon(imageVector = Icons.Default.Star, contentDescription = {})",
                    name
                ));
            }
            HIRView::Badge(badge) => {
                let content = self.expr_to_kotlin(&badge.content);
                self.line(&format!("Badge {{ Text({}) }}", content));
            }
            HIRView::Progress(_) => {
                self.line("CircularProgressIndicator()");
            }
            HIRView::Button(button) => {
                let label = self.expr_to_kotlin(&button.label);
                let action = self.action_expr_to_kotlin(&button.action);
                match button.style {
                    ButtonStyle::Icon => {
                        self.line(&format!("IconButton(onClick = {{ {} }}) {{", action));
                        self.indent += 1;
                        self.line(&format!(
                            "Icon(imageVector = Icons.Default.Star, contentDescription = {})",
                            label
                        ));
                        self.indent -= 1;
                        self.line("}");
                    }
                    ButtonStyle::Outline => {
                        self.line(&format!("OutlinedButton(onClick = {{ {} }}) {{", action));
                        self.indent += 1;
                        self.line(&format!("Text({})", label));
                        self.indent -= 1;
                        self.line("}");
                    }
                    _ => {
                        self.line(&format!("Button(onClick = {{ {} }}) {{", action));
                        self.indent += 1;
                        self.line(&format!("Text({})", label));
                        self.indent -= 1;
                        self.line("}");
                    }
                }
            }
            HIRView::TextField(field) => {
                let ph = field.placeholder.as_deref().unwrap_or("");
                self.line(&format!(
                    "OutlinedTextField(value = {}, onValueChange = {{ {} = it }}, placeholder = {{ Text(\"{}\") }})",
                    field.binding, field.binding, ph
                ));
            }
            HIRView::Checkbox(cb) => {
                self.line(&format!(
                    "Checkbox(checked = {}, onCheckedChange = {{ {} = it }})",
                    cb.binding, cb.binding
                ));
            }
            HIRView::Toggle(toggle) => {
                let label = toggle.label.as_deref().unwrap_or("");
                self.line(&format!(
                    "Row(verticalAlignment = Alignment.CenterVertically) {{ Text(\"{}\"); Switch(checked = {}, onCheckedChange = {{ {} = it }}) }}",
                    label, toggle.binding, toggle.binding
                ));
            }
            HIRView::Slider(slider) => {
                self.line(&format!(
                    "Slider(value = {}.toFloat(), onValueChange = {{ {} = it.toInt() }}, valueRange = {}f..{}f)",
                    slider.binding, slider.binding, slider.min, slider.max
                ));
            }
            HIRView::Spacer => {
                self.line("Spacer(modifier = Modifier.weight(1f))");
            }
            HIRView::Divider(_) => {
                self.line("Divider()");
            }
            HIRView::Conditional(cond) => {
                let condition = self.expr_to_kotlin(&cond.condition);
                self.line(&format!("if ({}) {{", condition));
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
                let iterable = self.expr_to_kotlin(&each.iterable);
                self.line(&format!("{}.forEach {{ {} ->", iterable, each.item_name));
                self.indent += 1;
                self.emit_view(&each.body);
                self.indent -= 1;
                self.line("}");
            }
            HIRView::ComponentRef(comp_ref) => {
                let args: Vec<String> = comp_ref
                    .args
                    .iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{} = {}", k, self.expr_to_kotlin(v)))
                    .collect();
                if args.is_empty() {
                    self.line(&format!("{}()", comp_ref.name));
                } else {
                    self.line(&format!("{}({})", comp_ref.name, args.join(", ")));
                }
            }
            HIRView::Group(children) => {
                for child in children {
                    self.emit_view(child);
                }
            }
            _ => {
                self.line("// unsupported element");
            }
        }
    }

    fn emit_function(&mut self, func: &HIRFunction) {
        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_to_kotlin(&p.param_type)))
            .collect();
        let ret = self.type_to_kotlin(&func.return_type);
        self.line(&format!(
            "fun {}({}): {} {{",
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

    fn emit_stmt(&mut self, stmt: &HIRStmt) {
        match stmt {
            HIRStmt::Assign(name, value) => {
                self.line(&format!("{} = {}", name, self.expr_to_kotlin(value)));
            }
            HIRStmt::Let(name, _, value) => {
                self.line(&format!("val {} = {}", name, self.expr_to_kotlin(value)));
            }
            HIRStmt::If(cond, then_body, else_body) => {
                self.line(&format!("if ({}) {{", self.expr_to_kotlin(cond)));
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
                self.line(&format!("return {}", self.expr_to_kotlin(value)));
            }
            HIRStmt::Return(None) => self.line("return"),
            HIRStmt::Expr(expr) => self.line(&self.expr_to_kotlin(expr)),
            _ => self.line("// unsupported"),
        }
    }

    fn expr_to_kotlin(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => format!("\"{}\"", s),
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::FloatLit(f) => format!("{}f", f),
            HIRExpr::BoolLit(b) => b.to_string(),
            HIRExpr::Nil => "null".to_string(),
            HIRExpr::Var(name, _) => name.clone(),
            HIRExpr::MemberAccess(obj, member, _) => {
                format!("{}.{}", self.expr_to_kotlin(obj), member)
            }
            HIRExpr::Call(func, args, _) => {
                let f = self.expr_to_kotlin(func);
                let a: Vec<String> = args.iter().map(|a| self.expr_to_kotlin(a)).collect();
                format!("{}({})", f, a.join(", "))
            }
            HIRExpr::BinOp(left, op, right, _) => {
                let l = self.expr_to_kotlin(left);
                let r = self.expr_to_kotlin(right);
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
                    aura_core::ast::BinOp::And => "&&",
                    aura_core::ast::BinOp::Or => "||",
                    _ => "/* ? */",
                };
                format!("({} {} {})", l, op_str, r)
            }
            HIRExpr::UnaryOp(op, operand, _) => {
                let o = self.expr_to_kotlin(operand);
                match op {
                    aura_core::ast::UnaryOp::Not => format!("!{}", o),
                    aura_core::ast::UnaryOp::Neg => format!("-{}", o),
                }
            }
            HIRExpr::Constructor(name, args, _) => {
                let fields: Vec<String> = args
                    .iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{} = {}", k, self.expr_to_kotlin(v)))
                    .collect();
                format!("{}({})", name, fields.join(", "))
            }
            HIRExpr::Lambda(params, body, _) => {
                let p: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
                format!("{{ {} -> {} }}", p.join(", "), self.expr_to_kotlin(body))
            }
            _ => "null".to_string(),
        }
    }

    fn action_expr_to_kotlin(&self, action: &HIRActionExpr) -> String {
        match action {
            HIRActionExpr::Call(name, args) => {
                let a: Vec<String> = args.iter().map(|a| self.expr_to_kotlin(a)).collect();
                if a.is_empty() {
                    format!("{}()", name)
                } else {
                    format!("{}({})", name, a.join(", "))
                }
            }
            HIRActionExpr::Navigate(nav) => match nav {
                HIRNavigate::Back => "onBackPressed()".to_string(),
                _ => "/* navigate */".to_string(),
            },
            HIRActionExpr::Sequence(actions) => actions
                .iter()
                .map(|a| self.action_expr_to_kotlin(a))
                .collect::<Vec<_>>()
                .join("; "),
        }
    }

    fn type_to_kotlin(&self, ty: &aura_core::types::AuraType) -> String {
        use aura_core::types::*;
        match ty {
            AuraType::Primitive(p) => match p {
                PrimitiveType::Text => "String".to_string(),
                PrimitiveType::Int => "Int".to_string(),
                PrimitiveType::Float => "Double".to_string(),
                PrimitiveType::Bool => "Boolean".to_string(),
                PrimitiveType::Timestamp => "Long".to_string(),
                PrimitiveType::Duration => "Long".to_string(),
                PrimitiveType::Percent => "Double".to_string(),
            },
            AuraType::List(inner) => format!("List<{}>", self.type_to_kotlin(inner)),
            AuraType::Set(inner) => format!("Set<{}>", self.type_to_kotlin(inner)),
            AuraType::Map(k, v) => format!(
                "Map<{}, {}>",
                self.type_to_kotlin(k),
                self.type_to_kotlin(v)
            ),
            AuraType::Optional(inner) => format!("{}?", self.type_to_kotlin(inner)),
            AuraType::Named(name) => name.clone(),
            AuraType::Action(_) => "() -> Unit".to_string(),
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
            AuraType::List(_) => "emptyList()".to_string(),
            AuraType::Map(_, _) => "emptyMap()".to_string(),
            AuraType::Optional(_) => "null".to_string(),
            _ => "\"\"".to_string(),
        }
    }

    fn spacing_dp(&self, design: &design::ResolvedDesign) -> String {
        design
            .spacing
            .as_ref()
            .and_then(|s| s.gap)
            .map(|g| format!("{}", g))
            .unwrap_or_else(|| "8".to_string())
    }

    fn layout_modifier(&self, design: &design::ResolvedDesign) -> String {
        let mut mods = Vec::new();
        if let Some(ref spacing) = design.spacing {
            if let Some(p) = spacing.padding_top {
                if spacing.padding_top == spacing.padding_bottom
                    && spacing.padding_left == spacing.padding_right
                    && spacing.padding_top == spacing.padding_left
                {
                    mods.push(format!("modifier = Modifier.padding({}.dp), ", p));
                }
            }
        }
        mods.join("")
    }

    fn text_modifiers(&self, design: &design::ResolvedDesign) -> String {
        let mut parts = Vec::new();
        if let Some(ref typo) = design.typography {
            if let Some(size) = typo.size {
                parts.push(format!(", fontSize = {}.sp", (size * 16.0).round()));
            }
            if let Some(weight) = typo.weight {
                let w = match weight {
                    700 => "FontWeight.Bold",
                    500 => "FontWeight.Medium",
                    600 => "FontWeight.SemiBold",
                    _ => "FontWeight.Normal",
                };
                parts.push(format!(", fontWeight = {}", w));
            }
        }
        if let Some(ref color) = design.color {
            if let Some(ref fg) = color.foreground {
                let c = match fg.as_str() {
                    "accent" => "MaterialTheme.colorScheme.primary",
                    "secondary" => "MaterialTheme.colorScheme.secondary",
                    "muted" => "MaterialTheme.colorScheme.outline",
                    "danger" => "MaterialTheme.colorScheme.error",
                    _ => "MaterialTheme.colorScheme.onSurface",
                };
                parts.push(format!(", color = {}", c));
            }
        }
        parts.join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_source(source: &str) -> ComposeOutput {
        let result = aura_core::parser::parse(source);
        assert!(
            result.errors.is_empty(),
            "Parse errors: {:?}",
            result.errors
        );
        let hir = aura_core::hir::build_hir(result.program.as_ref().unwrap());
        compile_to_compose(&hir)
    }

    #[test]
    fn test_minimal_compose() {
        let output =
            compile_source("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
        assert!(output.kotlin.contains("package com.aura.hello"));
        assert!(output.kotlin.contains("@Composable"));
        assert!(output.kotlin.contains("fun MainScreen()"));
        assert!(output.kotlin.contains("Text(text = \"Hello, Aura!\")"));
        assert!(output.kotlin.contains("class HelloActivity"));
    }

    #[test]
    fn test_state_generates_remember() {
        let output = compile_source(
            "\
app Test
  screen Main
    state count: int = 0
    view
      text \"hi\"",
        );
        assert!(output.kotlin.contains("mutableStateOf"));
        assert!(output.kotlin.contains("count"));
    }

    #[test]
    fn test_model_generates_data_class() {
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
        assert!(output.kotlin.contains("data class Todo("));
        assert!(output.kotlin.contains("val title: String"));
        assert!(output.kotlin.contains("val done: Boolean = false"));
    }

    #[test]
    fn test_button_generates_compose_button() {
        let output = compile_source(
            "\
app Test
  screen Main
    view
      button \"Save\" .accent -> save()
    action save
      return",
        );
        assert!(output.kotlin.contains("Button(onClick"));
        assert!(output.kotlin.contains("save()"));
    }

    #[test]
    fn test_layout_generates_column_row() {
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
        assert!(output.kotlin.contains("Column("));
        assert!(output.kotlin.contains("Row("));
        assert!(output.kotlin.contains("Arrangement.spacedBy"));
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
        assert!(output.kotlin.contains("items.forEach { item ->"));
    }
}
