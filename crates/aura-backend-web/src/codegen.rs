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
    /// Variables that are loop-scoped (not state-scoped).
    local_vars: Vec<String>,
}

impl<'a> WebCodegen<'a> {
    fn new(module: &'a HIRModule) -> Self {
        Self {
            module,
            html: String::new(),
            css: String::new(),
            js: String::new(),
            local_vars: Vec::new(),
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
        let is_dark = self.module.app.theme.as_deref() == Some("modern.dark");

        if is_dark {
            self.css.push_str(r#":root {
  --spacing-xs: 2px; --spacing-sm: 4px; --spacing-md: 8px; --spacing-lg: 16px;
  --spacing-xl: 24px; --spacing-2xl: 32px; --spacing-3xl: 48px; --spacing-4xl: 64px;
  --font-xs: 0.75rem; --font-sm: 0.875rem; --font-md: 1rem; --font-lg: 1.125rem;
  --font-xl: 1.25rem; --font-2xl: 1.5rem; --font-3xl: 2.125rem; --font-display: 3rem;
  --color-primary: #F0F0F8; --color-secondary: #A0A0B8; --color-muted: #606078;
  --color-accent: #7C6CF0; --color-accent-hover: #8B7EF5;
  --color-danger: #FF6B7A; --color-danger-hover: #FF8A95;
  --color-warning: #FFBE5C; --color-success: #4ADE80; --color-info: #60A5FA;
  --color-surface: #1A1A2E; --color-surface-hover: #222240;
  --color-background: #0F0F1A; --color-card: #16162A;
  --color-divider: rgba(255,255,255,0.08); --color-input-bg: #12121F;
  --color-input-border: rgba(255,255,255,0.1);
  --radius-sharp: 0px; --radius-subtle: 6px; --radius-rounded: 12px;
  --radius-smooth: 16px; --radius-pill: 9999px;
  --shadow-sm: 0 2px 8px rgba(0,0,0,0.3); --shadow-md: 0 4px 16px rgba(0,0,0,0.4);
  --shadow-lg: 0 8px 32px rgba(0,0,0,0.5);
  --font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
  color-scheme: dark;
}
"#);
        } else {
            self.css.push_str(r#":root {
  --spacing-xs: 2px; --spacing-sm: 4px; --spacing-md: 8px; --spacing-lg: 16px;
  --spacing-xl: 24px; --spacing-2xl: 32px; --spacing-3xl: 48px; --spacing-4xl: 64px;
  --font-xs: 0.75rem; --font-sm: 0.875rem; --font-md: 1rem; --font-lg: 1.125rem;
  --font-xl: 1.25rem; --font-2xl: 1.5rem; --font-3xl: 2.125rem; --font-display: 3rem;
  --color-primary: #1a1a2e; --color-secondary: #555570; --color-muted: #8888a0;
  --color-accent: #6C5CE7; --color-accent-hover: #5A4BD6;
  --color-danger: #DC3545; --color-danger-hover: #c82333;
  --color-warning: #F0AD4E; --color-success: #28A745; --color-info: #17A2B8;
  --color-surface: #FFFFFF; --color-surface-hover: #F8F8FC;
  --color-background: #F4F4FA; --color-card: #FFFFFF;
  --color-divider: rgba(0,0,0,0.06); --color-input-bg: #FFFFFF;
  --color-input-border: rgba(0,0,0,0.12);
  --radius-sharp: 0px; --radius-subtle: 6px; --radius-rounded: 12px;
  --radius-smooth: 16px; --radius-pill: 9999px;
  --shadow-sm: 0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04);
  --shadow-md: 0 4px 12px rgba(0,0,0,0.08); --shadow-lg: 0 8px 24px rgba(0,0,0,0.12);
  --font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
}
"#);
        }

        self.css.push_str(r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap');

*, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: var(--font-family);
  font-size: var(--font-md);
  color: var(--color-primary);
  background: var(--color-background);
  line-height: 1.6;
  min-height: 100vh;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

#app {
  max-width: 480px;
  margin: 0 auto;
  min-height: 100vh;
  background: var(--color-background);
  position: relative;
}

/* Layout */
.aura-column { display: flex; flex-direction: column; }
.aura-row { display: flex; flex-direction: row; align-items: center; }
.aura-stack { display: grid; grid-template-areas: "s"; }
.aura-stack > * { grid-area: s; }
.aura-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); }
.aura-scroll { overflow-y: auto; flex: 1; }
.aura-spacer { flex: 1; }

/* Divider */
.aura-divider {
  border: none;
  height: 1px;
  background: var(--color-divider);
  margin: 8px 0;
}

/* Typography */
.aura-text {
  color: var(--color-primary);
  font-size: var(--font-md);
  line-height: 1.5;
}

.aura-heading {
  font-weight: 700;
  color: var(--color-primary);
  letter-spacing: -0.02em;
  line-height: 1.2;
}

h1.aura-heading { font-size: var(--font-2xl); }
h2.aura-heading { font-size: var(--font-xl); }
h3.aura-heading { font-size: var(--font-lg); }

/* Buttons */
.aura-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 20px;
  border: none;
  border-radius: var(--radius-rounded);
  font-family: inherit;
  font-size: var(--font-sm);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  background: var(--color-accent);
  color: white;
  white-space: nowrap;
  user-select: none;
  -webkit-tap-highlight-color: transparent;
}
.aura-button:hover { background: var(--color-accent-hover); transform: translateY(-1px); box-shadow: var(--shadow-md); }
.aura-button:active { transform: translateY(0); }

.aura-button.icon {
  padding: 8px;
  width: 36px; height: 36px;
  border-radius: var(--radius-rounded);
  background: var(--color-accent);
  color: white;
  font-size: 16px;
}
.aura-button.icon:hover { background: var(--color-accent-hover); }

.aura-button.outline {
  background: transparent;
  border: 1.5px solid var(--color-accent);
  color: var(--color-accent);
}
.aura-button.outline:hover { background: var(--color-accent); color: white; }

.aura-button.ghost {
  background: transparent;
  color: var(--color-secondary);
  padding: 8px 12px;
}
.aura-button.ghost:hover { background: var(--color-surface-hover); color: var(--color-primary); }

/* Inputs */
.aura-input {
  padding: 10px 14px;
  border: 1.5px solid var(--color-input-border);
  border-radius: var(--radius-rounded);
  font-family: inherit;
  font-size: var(--font-md);
  color: var(--color-primary);
  background: var(--color-input-bg);
  outline: none;
  transition: all 0.2s ease;
  width: 100%;
}
.aura-input:focus {
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px rgba(108, 92, 231, 0.15);
}
.aura-input::placeholder { color: var(--color-muted); }

.aura-checkbox {
  width: 20px; height: 20px;
  accent-color: var(--color-accent);
  cursor: pointer;
  border-radius: 4px;
}

.aura-toggle {
  display: flex; align-items: center; gap: 10px;
  font-size: var(--font-md); cursor: pointer; user-select: none;
}
.aura-toggle input[type="checkbox"] {
  width: 44px; height: 24px; appearance: none; -webkit-appearance: none;
  background: var(--color-input-border); border-radius: 12px;
  position: relative; cursor: pointer; transition: background 0.2s;
}
.aura-toggle input[type="checkbox"]::after {
  content: ''; position: absolute; top: 2px; left: 2px;
  width: 20px; height: 20px; background: white;
  border-radius: 50%; transition: transform 0.2s;
}
.aura-toggle input[type="checkbox"]:checked { background: var(--color-accent); }
.aura-toggle input[type="checkbox"]:checked::after { transform: translateX(20px); }

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

/* Icons (emoji fallback) */
.aura-icon { font-size: 20px; line-height: 1; }

/* Semantic colors */
.color-accent { color: var(--color-accent) !important; }
.color-danger { color: var(--color-danger) !important; }
.color-warning { color: var(--color-warning) !important; }
.color-success { color: var(--color-success) !important; }
.color-info { color: var(--color-info) !important; }
.color-secondary { color: var(--color-secondary) !important; }
.color-muted { color: var(--color-muted) !important; }

.bg-surface {
  background: var(--color-surface) !important;
  border-radius: var(--radius-rounded);
  box-shadow: var(--shadow-sm);
}
.bg-accent { background: var(--color-accent) !important; color: white !important; }
.bg-danger { background: var(--color-danger) !important; color: white !important; }

/* Typography */
.text-bold { font-weight: 700 !important; }
.text-medium { font-weight: 600 !important; }
.text-italic { font-style: italic; }
.text-mono { font-family: var(--font-mono); }
.text-center { text-align: center; }
.text-strike { text-decoration: line-through; opacity: 0.5; }
.text-underline { text-decoration: underline; }
.text-uppercase { text-transform: uppercase; letter-spacing: 0.05em; font-size: 0.85em; }

/* Alignment */
.align-center { align-items: center; }
.align-start { align-items: flex-start; }
.align-end { align-items: flex-end; }
.justify-center { justify-content: center; }
.justify-between { justify-content: space-between; }
.justify-end { justify-content: flex-end; }

/* Shape */
.rounded { border-radius: var(--radius-rounded); }
.smooth { border-radius: var(--radius-smooth); }
.pill { border-radius: var(--radius-pill) !important; }
.circle { border-radius: 50%; }

/* Slider */
input[type="range"] {
  -webkit-appearance: none; appearance: none;
  width: 100%; height: 6px; border-radius: 3px;
  background: var(--color-input-border); outline: none;
}
input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none; appearance: none;
  width: 20px; height: 20px; border-radius: 50%;
  background: var(--color-accent); cursor: pointer;
  box-shadow: 0 2px 6px rgba(0,0,0,0.2);
}

/* Animations */
@keyframes fadeIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
#app > * { animation: fadeIn 0.3s ease-out; }

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
    <noscript>This Aura app requires JavaScript.</noscript>
  </div>
  <script src="app.js"></script>
</body>
</html>
"#, self.module.app.name));
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

    // === JS Generation (Reactive) ===

    fn generate_js(&mut self) {
        self.js.push_str("// Aura App — Generated by Aura Compiler\n");
        self.js.push_str("'use strict';\n\n");

        // Reactive state with Proxy
        self.js.push_str("const _state = {\n");
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

        self.js.push_str("const state = new Proxy(_state, {\n");
        self.js.push_str("  set(target, prop, value) {\n");
        self.js.push_str("    target[prop] = value;\n");
        self.js.push_str("    _scheduleRender();\n");
        self.js.push_str("    return true;\n");
        self.js.push_str("  }\n");
        self.js.push_str("});\n\n");

        self.js.push_str("let _renderPending = false;\n");
        self.js.push_str("function _scheduleRender() {\n");
        self.js.push_str("  if (!_renderPending) {\n");
        self.js.push_str("    _renderPending = true;\n");
        self.js.push_str("    requestAnimationFrame(() => { _renderPending = false; render(); });\n");
        self.js.push_str("  }\n");
        self.js.push_str("}\n\n");

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

        // Reactive render function — rebuilds DOM from state
        self.js.push_str("function render() {\n");
        self.js.push_str("  const app = document.getElementById('app');\n");
        self.js.push_str("  if (!app) return;\n");
        self.js.push_str("  app.innerHTML = renderView();\n");
        self.js.push_str("  _bindEvents();\n");
        self.js.push_str("}\n\n");

        // Render the view tree as HTML string
        self.js.push_str("function renderView() {\n");
        self.js.push_str("  return `\n");
        if let Some(screen) = self.module.screens.first() {
            let template = self.view_to_js_template(&screen.view, 2);
            self.js.push_str(&template);
        }
        self.js.push_str("`;\n");
        self.js.push_str("}\n\n");

        // Icon name to emoji mapping
        self.js.push_str("const _icons = {");
        self.js.push_str("'arrow.up':'\u{2B06}\u{FE0F}','arrow.up.circle.fill':'\u{2B06}\u{FE0F}',");
        self.js.push_str("'trash':'\u{1F5D1}','trash.fill':'\u{1F5D1}',");
        self.js.push_str("'plus':'+','plus.circle':'+','minus':'\u{2212}',");
        self.js.push_str("'star':'\u{2B50}','star.fill':'\u{2B50}',");
        self.js.push_str("'heart':'\u{2764}\u{FE0F}','heart.fill':'\u{2764}\u{FE0F}',");
        self.js.push_str("'checkmark':'\u{2705}','checkmark.circle':'\u{2705}',");
        self.js.push_str("'xmark':'\u{274C}','magnifyingglass':'\u{1F50D}',");
        self.js.push_str("'gear':'\u{2699}\u{FE0F}','person':'\u{1F464}',");
        self.js.push_str("'house':'\u{1F3E0}','bell':'\u{1F514}',");
        self.js.push_str("'camera':'\u{1F4F7}','photo':'\u{1F5BC}\u{FE0F}',");
        self.js.push_str("'sun.max':'\u{2600}\u{FE0F}','cloud':'\u{2601}\u{FE0F}',");
        self.js.push_str("'cloud.rain':'\u{1F327}\u{FE0F}','cloud.snow':'\u{1F328}\u{FE0F}',");
        self.js.push_str("'drop':'\u{1F4A7}','wind':'\u{1F4A8}',");
        self.js.push_str("'lock.circle':'\u{1F512}','lock':'\u{1F512}',");
        self.js.push_str("'phone':'\u{1F4DE}','video':'\u{1F4F9}','mic':'\u{1F3A4}',");
        self.js.push_str("'cart':'\u{1F6D2}','inbox':'\u{1F4E5}',");
        self.js.push_str("'square.and.pencil':'\u{270F}\u{FE0F}',");
        self.js.push_str("'bubble.left.and.bubble.right':'\u{1F4AC}',");
        self.js.push_str("};\n");
        self.js.push_str("function _icon(n) { return _icons[n] || n; }\n\n");

        // Event binding after render
        self.js.push_str("function _bindEvents() {\n");
        self.js.push_str("  // Bind input elements to state\n");
        self.js.push_str("  document.querySelectorAll('[data-bind]').forEach(el => {\n");
        self.js.push_str("    const key = el.dataset.bind;\n");
        self.js.push_str("    if (el.type === 'checkbox') {\n");
        self.js.push_str("      el.checked = !!state[key];\n");
        self.js.push_str("      el.onchange = () => { state[key] = el.checked; };\n");
        self.js.push_str("    } else {\n");
        self.js.push_str("      el.value = state[key] || '';\n");
        self.js.push_str("      el.oninput = () => { state[key] = el.value; };\n");
        self.js.push_str("    }\n");
        self.js.push_str("  });\n");
        self.js.push_str("}\n\n");

        self.js.push_str("// Initialize on DOM ready\n");
        self.js.push_str("document.addEventListener('DOMContentLoaded', render);\n");
    }

    /// Generate a JS template literal for a view tree.
    fn view_to_js_template(&mut self, view: &HIRView, depth: usize) -> String {
        let pad = "    ".repeat(depth);
        match view {
            HIRView::Column(layout) => {
                let style = self.design_inline_style(&layout.design);
                let cls = self.design_classes(&layout.design);
                let children: Vec<String> = layout.children.iter().map(|c| self.view_to_js_template(c, depth + 1)).collect();
                format!("{}<div class=\"aura-column{}\" style=\"{}\">\n{}{}</div>\n", pad, if cls.is_empty() { String::new() } else { format!(" {}", cls) }, style, children.join(""), pad)
            }
            HIRView::Row(layout) => {
                let style = self.design_inline_style(&layout.design);
                let cls = self.design_classes(&layout.design);
                let children: Vec<String> = layout.children.iter().map(|c| self.view_to_js_template(c, depth + 1)).collect();
                format!("{}<div class=\"aura-row{}\" style=\"{}\">\n{}{}</div>\n", pad, if cls.is_empty() { String::new() } else { format!(" {}", cls) }, style, children.join(""), pad)
            }
            HIRView::Scroll(scroll) => {
                let children: Vec<String> = scroll.children.iter().map(|c| self.view_to_js_template(c, depth + 1)).collect();
                format!("{}<div class=\"aura-scroll\">\n{}{}</div>\n", pad, children.join(""), pad)
            }
            HIRView::Grid(grid) => {
                let children: Vec<String> = grid.children.iter().map(|c| self.view_to_js_template(c, depth + 1)).collect();
                format!("{}<div class=\"aura-grid\">\n{}{}</div>\n", pad, children.join(""), pad)
            }
            HIRView::Text(text) => {
                let content = self.expr_to_js_template(&text.content);
                let cls = self.design_classes(&text.design);
                let style = self.design_inline_style(&text.design);
                format!("{}<span class=\"aura-text{}\"{}>{}</span>\n", pad, if cls.is_empty() { String::new() } else { format!(" {}", cls) }, if style.is_empty() { String::new() } else { format!(" style=\"{}\"", style) }, content)
            }
            HIRView::Heading(heading) => {
                let content = self.expr_to_js_template(&heading.content);
                let cls = self.design_classes(&heading.design);
                let style = self.design_inline_style(&heading.design);
                let tag = format!("h{}", heading.level.min(6).max(1));
                format!("{}<{} class=\"aura-heading{}\"{}>{}</{}>\n", pad, tag, if cls.is_empty() { String::new() } else { format!(" {}", cls) }, if style.is_empty() { String::new() } else { format!(" style=\"{}\"", style) }, content, tag)
            }
            HIRView::Button(button) => {
                let raw_label = self.expr_to_js_template(&button.label);
                let onclick = self.action_to_js(&button.action);
                let style_class = match button.style {
                    ButtonStyle::Icon => " icon",
                    ButtonStyle::Outline => " outline",
                    ButtonStyle::Ghost => " ghost",
                    _ => "",
                };
                let cls = self.design_classes(&button.design);
                // For icon buttons, wrap label in _icon() for emoji conversion
                let label = if button.style == ButtonStyle::Icon {
                    format!("${{_icon('{}')}}", raw_label)
                } else {
                    raw_label
                };
                format!("{}<button class=\"aura-button{}{}\" onclick=\"{}\">{}</button>\n", pad, style_class, if cls.is_empty() { String::new() } else { format!(" {}", cls) }, onclick, label)
            }
            HIRView::TextField(field) => {
                let ph = field.placeholder.as_deref().unwrap_or("");
                let cls = self.design_classes(&field.design);
                format!("{}<input class=\"aura-input{}\" type=\"text\" placeholder=\"{}\" data-bind=\"{}\" value=\"${{state.{} || ''}}\"/>\n", pad, if cls.is_empty() { String::new() } else { format!(" {}", cls) }, ph, field.binding, field.binding)
            }
            HIRView::Checkbox(cb) => {
                format!("{}<input class=\"aura-checkbox\" type=\"checkbox\" data-bind=\"{}\" ${{state.{} ? 'checked' : ''}}/>\n", pad, cb.binding, cb.binding)
            }
            HIRView::Toggle(toggle) => {
                let label = toggle.label.as_deref().unwrap_or("");
                format!("{}<label class=\"aura-toggle\"><input type=\"checkbox\" data-bind=\"{}\"/> {}</label>\n", pad, toggle.binding, label)
            }
            HIRView::Spacer => format!("{}<div class=\"aura-spacer\"></div>\n", pad),
            HIRView::Divider(_) => format!("{}<hr class=\"aura-divider\"/>\n", pad),
            HIRView::Progress(_) => format!("{}<progress class=\"aura-progress\" max=\"100\"></progress>\n", pad),
            HIRView::Icon(icon) => {
                let name = self.expr_to_js_template(&icon.name);
                let cls = self.design_classes(&icon.design);
                format!("{}<span class=\"aura-icon{}\">${{_icon('{}')}}</span>\n", pad, if cls.is_empty() { String::new() } else { format!(" {}", cls) }, name)
            }
            HIRView::Badge(badge) => {
                let content = self.expr_to_js_template(&badge.content);
                format!("{}<span class=\"aura-badge\">{}</span>\n", pad, content)
            }
            HIRView::Conditional(cond) => {
                let condition = self.expr_to_js(&cond.condition);
                let then_html = self.view_to_js_template(&cond.then_view, depth);
                let else_html = cond.else_view.as_ref().map(|v| self.view_to_js_template(v, depth)).unwrap_or_default();
                format!("${{({}) ? `{}` : `{}`}}", condition, then_html.trim(), else_html.trim())
            }
            HIRView::Each(each) => {
                let iterable = self.expr_to_js(&each.iterable);
                // Register loop variable so expr_to_js doesn't prefix with state.
                self.local_vars.push(each.item_name.clone());
                let item_template = self.view_to_js_template(&each.body, depth + 1);
                self.local_vars.pop();
                format!("${{{}.map({} => `{}`).join('')}}", iterable, each.item_name, item_template.trim().replace('`', "\\`"))
            }
            HIRView::ComponentRef(comp_ref) => {
                if let Some(comp) = self.module.components.iter().find(|c| c.name == comp_ref.name) {
                    self.view_to_js_template(&comp.view, depth)
                } else {
                    format!("{}<!-- {} -->\n", pad, comp_ref.name)
                }
            }
            HIRView::Group(children) => {
                children.iter().map(|c| self.view_to_js_template(c, depth)).collect::<Vec<_>>().join("")
            }
            _ => format!("{}<!-- unsupported -->\n", pad),
        }
    }

    /// Convert HIR expression to JS template literal expression (uses ${} syntax).
    fn expr_to_js_template(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => {
                // Check for interpolation markers
                if s.contains('{') && s.contains('}') {
                    // Convert {name} to ${state.name}
                    let mut result = s.clone();
                    while let Some(start) = result.find('{') {
                        if let Some(end) = result[start..].find('}') {
                            let var_name = &result[start + 1..start + end];
                            let replacement = format!("${{state.{}}}", var_name);
                            result = format!("{}{}{}", &result[..start], replacement, &result[start + end + 1..]);
                        } else {
                            break;
                        }
                    }
                    result
                } else {
                    s.clone()
                }
            }
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::Var(name, _) => {
                if self.local_vars.contains(name) {
                    format!("${{{}}}", name)
                } else {
                    format!("${{state.{}}}", name)
                }
            }
            HIRExpr::MemberAccess(obj, member, _) => {
                format!("${{{}.{}}}", self.expr_to_js(obj), member)
            }
            _ => self.expr_to_js(expr),
        }
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
            HIRExpr::Var(name, _) => {
                if self.local_vars.contains(name) {
                    name.clone()
                } else {
                    format!("state.{}", name)
                }
            }
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
                // Handle built-in collection constructors
                if name == "list" {
                    let items: Vec<String> = args.iter().map(|(_, v)| self.expr_to_js(v)).collect();
                    return format!("[{}]", items.join(", "));
                }
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

    fn icon_to_emoji(&self, name: &str) -> String {
        match name {
            "arrow.up" | "arrow.up.circle.fill" => "\u{2B06}\u{FE0F}".to_string(),
            "arrow.down" => "\u{2B07}\u{FE0F}".to_string(),
            "chevron.left" | "chevron.right" => "\u{203A}".to_string(),
            "plus" | "plus.circle" => "+".to_string(),
            "minus" | "minus.circle" => "\u{2212}".to_string(),
            "trash" | "trash.fill" => "\u{1F5D1}".to_string(),
            "star" | "star.fill" => "\u{2B50}".to_string(),
            "heart" | "heart.fill" => "\u{2764}\u{FE0F}".to_string(),
            "checkmark" | "checkmark.circle" | "checkmark.circle.fill" => "\u{2705}".to_string(),
            "xmark" | "xmark.circle" => "\u{274C}".to_string(),
            "magnifyingglass" => "\u{1F50D}".to_string(),
            "gear" | "gearshape" => "\u{2699}\u{FE0F}".to_string(),
            "person" | "person.circle" => "\u{1F464}".to_string(),
            "house" | "house.fill" => "\u{1F3E0}".to_string(),
            "bell" | "bell.fill" => "\u{1F514}".to_string(),
            "camera" | "camera.fill" => "\u{1F4F7}".to_string(),
            "photo" | "photo.fill" => "\u{1F5BC}\u{FE0F}".to_string(),
            "sun.max" | "sun.max.fill" => "\u{2600}\u{FE0F}".to_string(),
            "cloud" | "cloud.fill" => "\u{2601}\u{FE0F}".to_string(),
            "cloud.rain" => "\u{1F327}\u{FE0F}".to_string(),
            "cloud.snow" => "\u{1F328}\u{FE0F}".to_string(),
            "cloud.bolt" => "\u{26C8}\u{FE0F}".to_string(),
            "drop" | "drop.fill" => "\u{1F4A7}".to_string(),
            "wind" => "\u{1F4A8}".to_string(),
            "lock.circle" | "lock" | "lock.fill" => "\u{1F512}".to_string(),
            "phone" | "phone.fill" => "\u{1F4DE}".to_string(),
            "video" | "video.fill" => "\u{1F4F9}".to_string(),
            "mic" | "mic.fill" => "\u{1F3A4}".to_string(),
            "square.and.pencil" => "\u{270F}\u{FE0F}".to_string(),
            "bubble.left.and.bubble.right" => "\u{1F4AC}".to_string(),
            "cart" | "cart.fill" => "\u{1F6D2}".to_string(),
            "inbox" => "\u{1F4E5}".to_string(),
            _ => name.to_string(),
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
        assert!(output.html.contains("<!DOCTYPE html>"), "Missing doctype");
        assert!(output.html.contains("<title>Hello</title>"), "Missing title");
        assert!(output.html.contains("id=\"app\""), "Missing app div");
        // Content is now rendered by JS
        assert!(output.js.contains("Hello, Aura!"), "JS missing content");
        assert!(output.js.contains("aura-text"), "JS missing class");
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
    fn test_js_has_reactive_state() {
        let output = compile_source("\
app Test
  screen Main
    state count: int = 0
    view
      text \"Hi\"");
        assert!(output.js.contains("count: 0"), "Missing state init");
        assert!(output.js.contains("Proxy"), "Missing reactive Proxy");
        assert!(output.js.contains("_scheduleRender"), "Missing render scheduling");
        assert!(output.js.contains("renderView()"), "Missing renderView call");
    }

    #[test]
    fn test_button_onclick_in_template() {
        let output = compile_source("\
app Test
  screen Main
    view
      button \"Click\" .accent -> doStuff()
    action doStuff
      return");
        assert!(output.js.contains("onclick"), "JS missing onclick");
        assert!(output.js.contains("doStuff"), "JS missing action name");
        assert!(output.js.contains("aura-button"), "JS missing button class");
    }

    #[test]
    fn test_layout_design_tokens_in_template() {
        let output = compile_source("\
app Test
  screen Main
    view
      column gap.md padding.lg
        text \"Hello\" .bold .accent");
        assert!(output.js.contains("aura-column"), "JS missing column class");
        assert!(output.js.contains("gap: 8px"), "JS missing gap");
        assert!(output.js.contains("padding: 16px"), "JS missing padding");
        assert!(output.js.contains("Hello"), "JS missing text content");
    }

    #[test]
    fn test_textfield_with_binding() {
        let output = compile_source("\
app Test
  screen Main
    state query: text = \"\"
    view
      textfield query placeholder: \"Search...\"");
        assert!(output.js.contains("data-bind=\"query\""), "JS missing binding");
        assert!(output.js.contains("placeholder=\"Search...\""), "JS missing placeholder");
        assert!(output.js.contains("_bindEvents"), "JS missing event binding");
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
        assert!(output.js.contains("function increment()"), "Missing action function");
        assert!(output.js.contains("state.count = (state.count + 1)"), "Missing assignment");
        // No explicit render() call needed — Proxy auto-triggers _scheduleRender
    }
}
