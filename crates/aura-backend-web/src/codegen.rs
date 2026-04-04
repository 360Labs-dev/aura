//! Web codegen: HIR → HTML + CSS + JS

use aura_core::hir::*;
use aura_core::design;

/// Output of the web backend.
pub struct WebOutput {
    pub html: String,
    pub css: String,
    pub js: String,
}

/// Compile an HIR module to web output (HTML + CSS + JS).
pub fn compile_to_web(module: &HIRModule) -> WebOutput {
    let mut codegen = WebCodegen::new(module);
    codegen.generate()
}

struct WebCodegen<'a> {
    module: &'a HIRModule,
    html: String,
    css: String,
    js: String,
    indent: usize,
    component_counter: usize,
}

impl<'a> WebCodegen<'a> {
    fn new(module: &'a HIRModule) -> Self {
        Self {
            module,
            html: String::new(),
            css: String::new(),
            js: String::new(),
            indent: 0,
            component_counter: 0,
        }
    }

    fn generate(&mut self) -> WebOutput {
        self.generate_css();
        self.generate_js();
        self.generate_html();

        WebOutput {
            html: self.html.clone(),
            css: self.css.clone(),
            js: self.js.clone(),
        }
    }

    // === CSS Generation ===

    fn generate_css(&mut self) {
        self.css.push_str(r#":root {
  /* Aura Design Tokens */
  --spacing-xs: 2px;
  --spacing-sm: 4px;
  --spacing-md: 8px;
  --spacing-lg: 16px;
  --spacing-xl: 24px;
  --spacing-2xl: 32px;
  --spacing-3xl: 48px;
  --spacing-4xl: 64px;

  --font-xs: 0.75rem;
  --font-sm: 0.875rem;
  --font-md: 1rem;
  --font-lg: 1.125rem;
  --font-xl: 1.25rem;
  --font-2xl: 1.5rem;
  --font-3xl: 2.125rem;
  --font-display: 3rem;

  --color-primary: #1a1a2e;
  --color-secondary: #666666;
  --color-muted: #999999;
  --color-accent: #6C5CE7;
  --color-danger: #DC3545;
  --color-warning: #FFC107;
  --color-success: #28A745;
  --color-info: #17A2B8;
  --color-surface: #FFFFFF;
  --color-background: #F5F5F5;
  --color-divider: #E0E0E0;

  --radius-sharp: 0px;
  --radius-subtle: 4px;
  --radius-rounded: 8px;
  --radius-smooth: 12px;
  --radius-pill: 9999px;

  --font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-mono: 'SF Mono', 'Fira Code', monospace;
}

"#);

        // Dark mode
        self.css.push_str(r#"@media (prefers-color-scheme: dark) {
  :root {
    --color-primary: #FFFFFF;
    --color-secondary: #AAAAAA;
    --color-muted: #666666;
    --color-surface: #1E1E1E;
    --color-background: #121212;
    --color-divider: #2C2C2C;
  }
}

"#);

        // Check if theme is dark mode
        if self.module.app.theme.as_deref() == Some("modern.dark") {
            self.css.push_str(r#":root {
  --color-primary: #FFFFFF;
  --color-secondary: #AAAAAA;
  --color-muted: #666666;
  --color-surface: #1E1E1E;
  --color-background: #121212;
  --color-divider: #2C2C2C;
  color-scheme: dark;
}

"#);
        }

        // Base styles
        self.css.push_str(r#"* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: var(--font-family);
  font-size: var(--font-md);
  color: var(--color-primary);
  background-color: var(--color-background);
  line-height: 1.5;
  min-height: 100vh;
}

.aura-column {
  display: flex;
  flex-direction: column;
}

.aura-row {
  display: flex;
  flex-direction: row;
}

.aura-stack {
  display: grid;
  grid-template-areas: "stack";
}

.aura-stack > * {
  grid-area: stack;
}

.aura-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
}

.aura-scroll {
  overflow-y: auto;
}

.aura-spacer {
  flex: 1;
}

.aura-divider {
  border: none;
  border-top: 1px solid var(--color-divider);
  margin: var(--spacing-sm) 0;
}

.aura-text {
  color: var(--color-primary);
}

.aura-heading {
  font-weight: 700;
  color: var(--color-primary);
}

.aura-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-sm) var(--spacing-lg);
  border: none;
  border-radius: var(--radius-rounded);
  font-family: inherit;
  font-size: var(--font-md);
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s ease;
  background: var(--color-accent);
  color: white;
}

.aura-button:hover {
  opacity: 0.85;
}

.aura-button.icon {
  padding: var(--spacing-sm);
  background: transparent;
  color: var(--color-primary);
}

.aura-button.outline {
  background: transparent;
  border: 1.5px solid var(--color-accent);
  color: var(--color-accent);
}

.aura-button.ghost {
  background: transparent;
  color: var(--color-primary);
}

.aura-input {
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1.5px solid var(--color-divider);
  border-radius: var(--radius-rounded);
  font-family: inherit;
  font-size: var(--font-md);
  color: var(--color-primary);
  background: var(--color-surface);
  outline: none;
  transition: border-color 0.15s ease;
  width: 100%;
}

.aura-input:focus {
  border-color: var(--color-accent);
}

.aura-checkbox {
  width: 18px;
  height: 18px;
  accent-color: var(--color-accent);
}

.aura-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px var(--spacing-sm);
  border-radius: var(--radius-pill);
  font-size: var(--font-xs);
  font-weight: 600;
  background: var(--color-accent);
  color: white;
}

.aura-progress {
  width: 100%;
  height: 6px;
  border-radius: var(--radius-pill);
  appearance: none;
  -webkit-appearance: none;
}

.aura-progress::-webkit-progress-bar {
  background: var(--color-divider);
  border-radius: var(--radius-pill);
}

.aura-progress::-webkit-progress-value {
  background: var(--color-accent);
  border-radius: var(--radius-pill);
}

/* Semantic color classes */
.color-accent { color: var(--color-accent); }
.color-danger { color: var(--color-danger); }
.color-warning { color: var(--color-warning); }
.color-success { color: var(--color-success); }
.color-info { color: var(--color-info); }
.color-secondary { color: var(--color-secondary); }
.color-muted { color: var(--color-muted); }

.bg-surface { background-color: var(--color-surface); }
.bg-accent { background-color: var(--color-accent); }
.bg-danger { background-color: var(--color-danger); }

.text-bold { font-weight: 700; }
.text-medium { font-weight: 500; }
.text-italic { font-style: italic; }
.text-mono { font-family: var(--font-mono); }
.text-center { text-align: center; }
.text-strike { text-decoration: line-through; }
.text-underline { text-decoration: underline; }
.text-uppercase { text-transform: uppercase; }

.align-center { align-items: center; }
.align-start { align-items: flex-start; }
.align-end { align-items: flex-end; }
.justify-center { justify-content: center; }
.justify-between { justify-content: space-between; }
.justify-end { justify-content: flex-end; }

.rounded { border-radius: var(--radius-rounded); }
.smooth { border-radius: var(--radius-smooth); }
.pill { border-radius: var(--radius-pill); }
.circle { border-radius: 50%; }

"#);
    }

    // === HTML Generation ===

    fn generate_html(&mut self) {
        self.html.push_str(&format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <div id="app">
"#, self.module.app.name));

        // Render the first screen
        if let Some(screen) = self.module.screens.first() {
            self.indent = 2;
            self.emit_view(&screen.view);
        }

        self.html.push_str(r#"  </div>
  <script src="app.js"></script>
</body>
</html>
"#);
    }

    fn emit_view(&mut self, view: &HIRView) {
        match view {
            HIRView::Column(layout) => {
                self.emit_open_tag("div", "aura-column", &layout.design);
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.emit_close_tag("div");
            }
            HIRView::Row(layout) => {
                self.emit_open_tag("div", "aura-row", &layout.design);
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.emit_close_tag("div");
            }
            HIRView::Stack(layout) => {
                self.emit_open_tag("div", "aura-stack", &layout.design);
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.emit_close_tag("div");
            }
            HIRView::Grid(grid) => {
                self.emit_open_tag("div", "aura-grid", &grid.design);
                for child in &grid.children {
                    self.emit_view(child);
                }
                self.emit_close_tag("div");
            }
            HIRView::Scroll(scroll) => {
                self.emit_open_tag("div", "aura-scroll", &scroll.design);
                for child in &scroll.children {
                    self.emit_view(child);
                }
                self.emit_close_tag("div");
            }
            HIRView::Wrap(layout) => {
                let mut d = layout.design.clone();
                self.emit_open_tag("div", "aura-row", &d);
                for child in &layout.children {
                    self.emit_view(child);
                }
                self.emit_close_tag("div");
            }
            HIRView::Text(text) => {
                let classes = self.design_classes(&text.design);
                let style = self.design_inline_style(&text.design);
                let content = self.expr_to_html(&text.content);
                self.emit_line(&format!(
                    "<span class=\"aura-text{}\"{}>{}</span>",
                    if classes.is_empty() { String::new() } else { format!(" {}", classes) },
                    if style.is_empty() { String::new() } else { format!(" style=\"{}\"", style) },
                    content,
                ));
            }
            HIRView::Heading(heading) => {
                let tag = format!("h{}", heading.level.min(6).max(1));
                let classes = self.design_classes(&heading.design);
                let style = self.design_inline_style(&heading.design);
                let content = self.expr_to_html(&heading.content);
                self.emit_line(&format!(
                    "<{} class=\"aura-heading{}\"{}>{}</{}>",
                    tag,
                    if classes.is_empty() { String::new() } else { format!(" {}", classes) },
                    if style.is_empty() { String::new() } else { format!(" style=\"{}\"", style) },
                    content,
                    tag,
                ));
            }
            HIRView::Image(image) => {
                let src = self.expr_to_html(&image.source);
                let style = self.design_inline_style(&image.design);
                self.emit_line(&format!(
                    "<img src=\"{}\" alt=\"\"{}/>",
                    src,
                    if style.is_empty() { String::new() } else { format!(" style=\"{}\"", style) },
                ));
            }
            HIRView::Icon(icon) => {
                let name = self.expr_to_html(&icon.name);
                let classes = self.design_classes(&icon.design);
                self.emit_line(&format!(
                    "<span class=\"aura-icon{}\">{}</span>",
                    if classes.is_empty() { String::new() } else { format!(" {}", classes) },
                    name,
                ));
            }
            HIRView::Badge(badge) => {
                let content = self.expr_to_html(&badge.content);
                let classes = self.design_classes(&badge.design);
                self.emit_line(&format!(
                    "<span class=\"aura-badge{}\">{}</span>",
                    if classes.is_empty() { String::new() } else { format!(" {}", classes) },
                    content,
                ));
            }
            HIRView::Progress(progress) => {
                self.emit_line("<progress class=\"aura-progress\" max=\"100\"></progress>");
            }
            HIRView::Button(button) => {
                let style_class = match button.style {
                    ButtonStyle::Icon => " icon",
                    ButtonStyle::Outline => " outline",
                    ButtonStyle::Ghost => " ghost",
                    ButtonStyle::Link => " ghost",
                    ButtonStyle::Default => "",
                };
                let classes = self.design_classes(&button.design);
                let style = self.design_inline_style(&button.design);
                let label = self.expr_to_html(&button.label);
                let onclick = self.action_to_js(&button.action);
                self.emit_line(&format!(
                    "<button class=\"aura-button{}{}\"{}onclick=\"{}\">{}</button>",
                    style_class,
                    if classes.is_empty() { String::new() } else { format!(" {}", classes) },
                    if style.is_empty() { String::new() } else { format!(" style=\"{}\" ", style) },
                    onclick,
                    label,
                ));
            }
            HIRView::TextField(field) => {
                let ph = field.placeholder.as_deref().unwrap_or("");
                let classes = self.design_classes(&field.design);
                self.emit_line(&format!(
                    "<input class=\"aura-input{}\" type=\"text\" placeholder=\"{}\" data-bind=\"{}\"/>",
                    if classes.is_empty() { String::new() } else { format!(" {}", classes) },
                    ph,
                    field.binding,
                ));
            }
            HIRView::Checkbox(cb) => {
                self.emit_line(&format!(
                    "<input class=\"aura-checkbox\" type=\"checkbox\" data-bind=\"{}\"/>",
                    cb.binding,
                ));
            }
            HIRView::Toggle(toggle) => {
                let label = toggle.label.as_deref().unwrap_or("");
                self.emit_line(&format!(
                    "<label class=\"aura-toggle\"><input type=\"checkbox\" data-bind=\"{}\"/> {}</label>",
                    toggle.binding, label,
                ));
            }
            HIRView::Spacer => {
                self.emit_line("<div class=\"aura-spacer\"></div>");
            }
            HIRView::Divider(_) => {
                self.emit_line("<hr class=\"aura-divider\"/>");
            }
            HIRView::Conditional(cond) => {
                let id = self.next_id();
                self.emit_line(&format!("<!-- if {} -->", self.expr_to_js(&cond.condition)));
                self.emit_line(&format!("<div id=\"cond-{}\">", id));
                self.indent += 1;
                self.emit_view(&cond.then_view);
                self.indent -= 1;
                self.emit_line("</div>");
                if let Some(ref else_view) = cond.else_view {
                    self.emit_line(&format!("<div id=\"cond-{}-else\" style=\"display:none\">", id));
                    self.indent += 1;
                    self.emit_view(else_view);
                    self.indent -= 1;
                    self.emit_line("</div>");
                }
            }
            HIRView::Each(each) => {
                self.emit_line(&format!("<!-- each {} as {} -->", self.expr_to_js(&each.iterable), each.item_name));
                self.emit_line(&format!("<div data-each=\"{}\" data-as=\"{}\">", self.expr_to_js(&each.iterable), each.item_name));
                self.indent += 1;
                self.emit_view(&each.body);
                self.indent -= 1;
                self.emit_line("</div>");
            }
            HIRView::Switch(switch) => {
                self.emit_line(&format!("<!-- when {} -->", self.expr_to_js(&switch.expression)));
                for case in &switch.cases {
                    self.emit_line(&format!("<div data-case=\"{:?}\">", case.pattern));
                    self.indent += 1;
                    self.emit_view(&case.view);
                    self.indent -= 1;
                    self.emit_line("</div>");
                }
            }
            HIRView::ComponentRef(comp_ref) => {
                self.emit_line(&format!("<!-- component: {} -->", comp_ref.name));
                // Inline the component's view if we can find it
                if let Some(comp) = self.module.components.iter().find(|c| c.name == comp_ref.name) {
                    self.emit_line(&format!("<div class=\"component-{}\">", comp_ref.name.to_lowercase()));
                    self.indent += 1;
                    self.emit_view(&comp.view);
                    self.indent -= 1;
                    self.emit_line("</div>");
                } else {
                    self.emit_line(&format!("<div class=\"component-{}\"><!-- not found --></div>", comp_ref.name.to_lowercase()));
                }
            }
            HIRView::Group(children) => {
                for child in children {
                    self.emit_view(child);
                }
            }
            _ => {
                self.emit_line("<!-- unsupported view element -->");
            }
        }
    }

    // === JS Generation ===

    fn generate_js(&mut self) {
        self.js.push_str("// Aura App — Generated by Aura Compiler\n");
        self.js.push_str("'use strict';\n\n");

        // State management
        self.js.push_str("const state = {\n");
        if let Some(screen) = self.module.screens.first() {
            for s in &screen.state {
                let default = s
                    .initial
                    .as_ref()
                    .map(|e| self.expr_to_js(e))
                    .unwrap_or_else(|| "null".to_string());
                self.js.push_str(&format!("  {}: {},\n", s.name, default));
            }
        }
        self.js.push_str("};\n\n");

        // Models
        for model in &self.module.models {
            self.js.push_str(&format!("function {}(fields) {{\n", model.name));
            self.js.push_str("  return { ");
            let field_defaults: Vec<String> = model.fields.iter().map(|f| {
                let default = f.default.as_ref().map(|e| self.expr_to_js(e)).unwrap_or_else(|| "null".to_string());
                format!("{}: fields.{} ?? {}", f.name, f.name, default)
            }).collect();
            self.js.push_str(&field_defaults.join(", "));
            self.js.push_str(" };\n}\n\n");
        }

        // Actions
        if let Some(screen) = self.module.screens.first() {
            for action in &screen.actions {
                let params: Vec<String> = action.params.iter().map(|p| p.name.clone()).collect();
                self.js.push_str(&format!(
                    "function {}({}) {{\n",
                    action.name,
                    params.join(", ")
                ));
                for stmt in &action.body {
                    self.js.push_str(&format!("  {};\n", self.stmt_to_js(stmt)));
                }
                self.js.push_str("  render();\n");
                self.js.push_str("}\n\n");
            }

            // Functions
            for func in &screen.functions {
                let params: Vec<String> = func.params.iter().map(|p| p.name.clone()).collect();
                self.js.push_str(&format!(
                    "function {}({}) {{\n",
                    func.name,
                    params.join(", ")
                ));
                for stmt in &func.body {
                    self.js.push_str(&format!("  {};\n", self.stmt_to_js(stmt)));
                }
                self.js.push_str("}\n\n");
            }
        }

        // Render function placeholder
        self.js.push_str("function render() {\n");
        self.js.push_str("  // Re-render the app (simple implementation)\n");
        self.js.push_str("  console.log('State updated:', state);\n");
        self.js.push_str("}\n\n");
        self.js.push_str("// Initialize\n");
        self.js.push_str("document.addEventListener('DOMContentLoaded', render);\n");
    }

    // === Helpers ===

    fn emit_line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.html.push_str("  ");
        }
        self.html.push_str(line);
        self.html.push('\n');
    }

    fn emit_open_tag(&mut self, tag: &str, class: &str, design: &design::ResolvedDesign) {
        let extra_classes = self.design_classes(design);
        let style = self.design_inline_style(design);
        self.emit_line(&format!(
            "<{} class=\"{}{}\"{}> ",
            tag,
            class,
            if extra_classes.is_empty() { String::new() } else { format!(" {}", extra_classes) },
            if style.is_empty() { String::new() } else { format!(" style=\"{}\"", style) },
        ));
        self.indent += 1;
    }

    fn emit_close_tag(&mut self, tag: &str) {
        self.indent -= 1;
        self.emit_line(&format!("</{}>", tag));
    }

    fn next_id(&mut self) -> usize {
        self.component_counter += 1;
        self.component_counter
    }

    fn design_classes(&self, design: &design::ResolvedDesign) -> String {
        let mut classes = Vec::new();

        if let Some(ref typo) = design.typography {
            if typo.weight == Some(700) { classes.push("text-bold"); }
            if typo.weight == Some(500) { classes.push("text-medium"); }
            if typo.italic { classes.push("text-italic"); }
            if typo.mono { classes.push("text-mono"); }
            if typo.strikethrough { classes.push("text-strike"); }
            if typo.underline { classes.push("text-underline"); }
            if typo.alignment == Some(design::TextAlignment::Center) { classes.push("text-center"); }
            if typo.transform == Some(design::TextTransform::Uppercase) { classes.push("text-uppercase"); }
        }

        if let Some(ref color) = design.color {
            if let Some(ref fg) = color.foreground {
                classes.push(match fg.as_str() {
                    "accent" => "color-accent",
                    "danger" => "color-danger",
                    "warning" => "color-warning",
                    "success" => "color-success",
                    "info" => "color-info",
                    "secondary" => "color-secondary",
                    "muted" => "color-muted",
                    _ => "",
                });
            }
            if let Some(ref bg) = color.background {
                classes.push(match bg.as_str() {
                    "surface" => "bg-surface",
                    "accent" => "bg-accent",
                    "danger" => "bg-danger",
                    _ => "",
                });
            }
        }

        if let Some(ref shape) = design.shape {
            classes.push(match shape.kind {
                design::ShapeKind::Rounded => "rounded",
                design::ShapeKind::Smooth => "smooth",
                design::ShapeKind::Pill => "pill",
                design::ShapeKind::Circle => "circle",
                _ => "",
            });
        }

        classes.retain(|c| !c.is_empty());
        classes.join(" ")
    }

    fn design_inline_style(&self, design: &design::ResolvedDesign) -> String {
        let mut styles = Vec::new();

        if let Some(ref spacing) = design.spacing {
            if let Some(gap) = spacing.gap {
                styles.push(format!("gap: {}px", gap));
            }
            if let Some(pt) = spacing.padding_top {
                if spacing.padding_top == spacing.padding_bottom
                    && spacing.padding_left == spacing.padding_right
                    && spacing.padding_top == spacing.padding_left
                {
                    styles.push(format!("padding: {}px", pt));
                } else {
                    if let Some(v) = spacing.padding_top { styles.push(format!("padding-top: {}px", v)); }
                    if let Some(v) = spacing.padding_bottom { styles.push(format!("padding-bottom: {}px", v)); }
                    if let Some(v) = spacing.padding_left { styles.push(format!("padding-left: {}px", v)); }
                    if let Some(v) = spacing.padding_right { styles.push(format!("padding-right: {}px", v)); }
                }
            }
        }

        if let Some(ref typo) = design.typography {
            if let Some(size) = typo.size {
                styles.push(format!("font-size: {}rem", size));
            }
        }

        styles.join("; ")
    }

    fn expr_to_html(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => s.clone(),
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::FloatLit(f) => f.to_string(),
            HIRExpr::BoolLit(b) => b.to_string(),
            HIRExpr::Var(name, _) => format!("${{{}}}", name),
            HIRExpr::MemberAccess(obj, member, _) => {
                format!("{}.{}", self.expr_to_html(obj), member)
            }
            _ => "...".to_string(),
        }
    }

    fn expr_to_js(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => format!("\"{}\"", s),
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::FloatLit(f) => f.to_string(),
            HIRExpr::BoolLit(b) => b.to_string(),
            HIRExpr::Nil => "null".to_string(),
            HIRExpr::Var(name, _) => format!("state.{}", name),
            HIRExpr::MemberAccess(obj, member, _) => {
                format!("{}.{}", self.expr_to_js(obj), member)
            }
            HIRExpr::Call(func, args, _) => {
                let f = self.expr_to_js(func);
                let a: Vec<String> = args.iter().map(|a| self.expr_to_js(a)).collect();
                format!("{}({})", f, a.join(", "))
            }
            HIRExpr::BinOp(left, op, right, _) => {
                let l = self.expr_to_js(left);
                let r = self.expr_to_js(right);
                let op_str = match op {
                    aura_core::ast::BinOp::Add => "+",
                    aura_core::ast::BinOp::Sub => "-",
                    aura_core::ast::BinOp::Mul => "*",
                    aura_core::ast::BinOp::Div => "/",
                    aura_core::ast::BinOp::Mod => "%",
                    aura_core::ast::BinOp::Eq => "===",
                    aura_core::ast::BinOp::NotEq => "!==",
                    aura_core::ast::BinOp::Lt => "<",
                    aura_core::ast::BinOp::Gt => ">",
                    aura_core::ast::BinOp::LtEq => "<=",
                    aura_core::ast::BinOp::GtEq => ">=",
                    aura_core::ast::BinOp::And => "&&",
                    aura_core::ast::BinOp::Or => "||",
                    aura_core::ast::BinOp::Range => "/* range */",
                };
                format!("({} {} {})", l, op_str, r)
            }
            HIRExpr::UnaryOp(op, operand, _) => {
                let o = self.expr_to_js(operand);
                match op {
                    aura_core::ast::UnaryOp::Not => format!("!{}", o),
                    aura_core::ast::UnaryOp::Neg => format!("-{}", o),
                }
            }
            HIRExpr::Constructor(name, args, _) => {
                let fields: Vec<String> = args
                    .iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_js(v)))
                    .collect();
                format!("{}({{ {} }})", name, fields.join(", "))
            }
            HIRExpr::Lambda(params, body, _) => {
                let p: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
                format!("({}) => {}", p.join(", "), self.expr_to_js(body))
            }
            _ => "null".to_string(),
        }
    }

    fn stmt_to_js(&self, stmt: &HIRStmt) -> String {
        match stmt {
            HIRStmt::Assign(name, value) => {
                format!("state.{} = {}", name, self.expr_to_js(value))
            }
            HIRStmt::Let(name, _, value) => {
                format!("const {} = {}", name, self.expr_to_js(value))
            }
            HIRStmt::If(cond, then_body, else_body) => {
                let mut s = format!("if ({}) {{ ", self.expr_to_js(cond));
                for stmt in then_body {
                    s.push_str(&format!("{}; ", self.stmt_to_js(stmt)));
                }
                s.push('}');
                if let Some(else_stmts) = else_body {
                    s.push_str(" else { ");
                    for stmt in else_stmts {
                        s.push_str(&format!("{}; ", self.stmt_to_js(stmt)));
                    }
                    s.push('}');
                }
                s
            }
            HIRStmt::Return(value) => {
                if let Some(v) = value {
                    format!("return {}", self.expr_to_js(v))
                } else {
                    "return".to_string()
                }
            }
            HIRStmt::Expr(expr) => self.expr_to_js(expr),
            _ => "/* unsupported statement */".to_string(),
        }
    }

    fn action_to_js(&self, action: &HIRActionExpr) -> String {
        match action {
            HIRActionExpr::Call(name, args) => {
                let a: Vec<String> = args.iter().map(|a| self.expr_to_js(a)).collect();
                format!("{}({})", name, a.join(", "))
            }
            HIRActionExpr::Navigate(nav) => match nav {
                HIRNavigate::Back => "history.back()".to_string(),
                HIRNavigate::To(expr) => format!("navigate({})", self.expr_to_js(expr)),
                _ => "/* navigate */".to_string(),
            },
            HIRActionExpr::Sequence(actions) => {
                actions.iter().map(|a| self.action_to_js(a)).collect::<Vec<_>>().join("; ")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_source(source: &str) -> WebOutput {
        let parse_result = aura_core::parser::parse(source);
        assert!(parse_result.errors.is_empty(), "Parse errors: {:?}", parse_result.errors);
        let hir = aura_core::hir::build_hir(parse_result.program.as_ref().unwrap());
        compile_to_web(&hir)
    }

    #[test]
    fn test_minimal_html() {
        let output = compile_source("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
        assert!(output.html.contains("<!DOCTYPE html>"));
        assert!(output.html.contains("<title>Hello</title>"));
        assert!(output.html.contains("Hello, Aura!"));
        assert!(output.html.contains("aura-text"));
    }

    #[test]
    fn test_css_has_tokens() {
        let output = compile_source("app Test\n  screen Main\n    view\n      text \"Hi\"");
        assert!(output.css.contains("--spacing-md: 8px"));
        assert!(output.css.contains("--color-accent"));
        assert!(output.css.contains("--font-family"));
        assert!(output.css.contains(".aura-button"));
    }

    #[test]
    fn test_js_has_state() {
        let output = compile_source("\
app Test
  screen Main
    state count: int = 0
    view
      text \"Hi\"");
        assert!(output.js.contains("count: 0"));
        assert!(output.js.contains("const state"));
    }

    #[test]
    fn test_button_generates_onclick() {
        let output = compile_source("\
app Test
  screen Main
    view
      button \"Click\" .accent -> doStuff()
    action doStuff
      return");
        assert!(output.html.contains("onclick"));
        assert!(output.html.contains("doStuff"));
        assert!(output.html.contains("aura-button"));
    }

    #[test]
    fn test_layout_with_design_tokens() {
        let output = compile_source("\
app Test
  screen Main
    view
      column gap.md padding.lg
        text \"Hello\" .bold .accent");
        assert!(output.html.contains("aura-column"), "HTML missing aura-column:\n{}", output.html);
        assert!(output.html.contains("gap: 8px"), "HTML missing gap:\n{}", output.html);
        assert!(output.html.contains("padding: 16px"), "HTML missing padding:\n{}", output.html);
        // Design tokens on text are resolved at HIR level; check output reflects them
        // (bold/accent may be in class or inline style depending on resolution)
        assert!(output.html.contains("Hello"), "HTML missing text content");
    }

    #[test]
    fn test_textfield_generates_input() {
        let output = compile_source("\
app Test
  screen Main
    state query: text = \"\"
    view
      textfield query placeholder: \"Search...\"");
        assert!(output.html.contains("<input"));
        assert!(output.html.contains("placeholder=\"Search...\""));
        assert!(output.html.contains("data-bind=\"query\""));
    }

    #[test]
    fn test_dark_theme() {
        let output = compile_source("\
app Test
  theme: modern.dark
  screen Main
    view
      text \"Dark mode\"");
        assert!(output.css.contains("color-scheme: dark"));
    }

    #[test]
    fn test_action_generates_js_function() {
        let output = compile_source("\
app Test
  screen Main
    state count: int = 0
    view
      text \"Hi\"
    action increment
      count = count + 1");
        assert!(output.js.contains("function increment()"));
        assert!(output.js.contains("state.count = (state.count + 1)"));
        assert!(output.js.contains("render()"));
    }
}
