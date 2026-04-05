//! WinUI codegen: HIR → C# + XAML

use aura_core::hir::*;
use aura_core::design;

pub struct WinUiOutput {
    pub xaml: String,
    pub cs: String,
    pub xaml_filename: String,
    pub cs_filename: String,
}

pub fn compile_to_winui(module: &HIRModule) -> WinUiOutput {
    let mut cg = WinUiCodegen::new(module);
    cg.generate();
    WinUiOutput {
        xaml: cg.xaml,
        cs: cg.cs,
        xaml_filename: "MainPage.xaml".to_string(),
        cs_filename: "MainPage.xaml.cs".to_string(),
    }
}

struct WinUiCodegen<'a> {
    module: &'a HIRModule,
    xaml: String,
    cs: String,
    indent: usize,
}

impl<'a> WinUiCodegen<'a> {
    fn new(module: &'a HIRModule) -> Self {
        Self { module, xaml: String::new(), cs: String::new(), indent: 0 }
    }

    fn xline(&mut self, text: &str) {
        for _ in 0..self.indent { self.xaml.push_str("    "); }
        self.xaml.push_str(text);
        self.xaml.push('\n');
    }

    fn cline(&mut self, text: &str) {
        for _ in 0..self.indent { self.cs.push_str("    "); }
        self.cs.push_str(text);
        self.cs.push('\n');
    }

    fn generate(&mut self) {
        self.generate_xaml();
        self.generate_cs();
    }

    fn generate_xaml(&mut self) {
        self.xline("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
        self.xline(&format!(
            "<Page x:Class=\"{}.MainPage\"",
            self.module.app.name
        ));
        self.indent += 1;
        self.xline("xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\"");
        self.xline("xmlns:x=\"http://schemas.microsoft.com/winfx/2006/xaml\">");
        self.indent -= 1;
        self.xline("");

        self.indent += 1;
        if let Some(screen) = self.module.screens.first() {
            self.emit_view_xaml(&screen.view);
        }
        self.indent -= 1;

        self.xline("</Page>");
    }

    fn emit_view_xaml(&mut self, view: &HIRView) {
        match view {
            HIRView::Column(layout) => {
                let spacing = self.spacing_val(&layout.design);
                let padding = self.padding_val(&layout.design);
                self.xline(&format!("<StackPanel Orientation=\"Vertical\" Spacing=\"{}\" Padding=\"{}\">", spacing, padding));
                self.indent += 1;
                for child in &layout.children { self.emit_view_xaml(child); }
                self.indent -= 1;
                self.xline("</StackPanel>");
            }
            HIRView::Row(layout) => {
                let spacing = self.spacing_val(&layout.design);
                self.xline(&format!("<StackPanel Orientation=\"Horizontal\" Spacing=\"{}\">", spacing));
                self.indent += 1;
                for child in &layout.children { self.emit_view_xaml(child); }
                self.indent -= 1;
                self.xline("</StackPanel>");
            }
            HIRView::Stack(layout) => {
                self.xline("<Grid>");
                self.indent += 1;
                for child in &layout.children { self.emit_view_xaml(child); }
                self.indent -= 1;
                self.xline("</Grid>");
            }
            HIRView::Grid(grid) => {
                self.xline("<GridView>");
                self.indent += 1;
                for child in &grid.children { self.emit_view_xaml(child); }
                self.indent -= 1;
                self.xline("</GridView>");
            }
            HIRView::Scroll(scroll) => {
                self.xline("<ScrollViewer>");
                self.indent += 1;
                self.xline("<StackPanel>");
                self.indent += 1;
                for child in &scroll.children { self.emit_view_xaml(child); }
                self.indent -= 1;
                self.xline("</StackPanel>");
                self.indent -= 1;
                self.xline("</ScrollViewer>");
            }
            HIRView::Text(text) => {
                let content = self.expr_str(&text.content);
                let style = self.text_style_xaml(&text.design);
                self.xline(&format!("<TextBlock Text=\"{}\"{}/>", content, style));
            }
            HIRView::Heading(heading) => {
                let content = self.expr_str(&heading.content);
                let size = match heading.level {
                    1 => "28", 2 => "24", 3 => "20", _ => "18",
                };
                self.xline(&format!(
                    "<TextBlock Text=\"{}\" FontSize=\"{}\" FontWeight=\"Bold\"/>",
                    content, size
                ));
            }
            HIRView::Button(button) => {
                let label = self.expr_str(&button.label);
                let style = self.button_style(&button.design);
                match button.style {
                    ButtonStyle::Icon => {
                        self.xline(&format!(
                            "<Button{} ToolTipService.ToolTip=\"{}\">",
                            style, label
                        ));
                        self.indent += 1;
                        self.xline(&format!("<SymbolIcon Symbol=\"{}\" />", label));
                        self.indent -= 1;
                        self.xline("</Button>");
                    }
                    _ => {
                        self.xline(&format!("<Button Content=\"{}\"{}/>", label, style));
                    }
                }
            }
            HIRView::TextField(field) => {
                let ph = field.placeholder.as_deref().unwrap_or("");
                self.xline(&format!(
                    "<TextBox PlaceholderText=\"{}\" x:Name=\"{}\" />",
                    ph, field.binding
                ));
            }
            HIRView::Checkbox(cb) => {
                self.xline(&format!(
                    "<CheckBox x:Name=\"{}\" />",
                    cb.binding.replace('.', "_")
                ));
            }
            HIRView::Toggle(toggle) => {
                let header = toggle.label.as_deref().unwrap_or("");
                self.xline(&format!(
                    "<ToggleSwitch Header=\"{}\" x:Name=\"{}\" />",
                    header, toggle.binding
                ));
            }
            HIRView::Slider(slider) => {
                self.xline(&format!(
                    "<Slider Minimum=\"{}\" Maximum=\"{}\" StepFrequency=\"{}\" x:Name=\"{}\" />",
                    slider.min, slider.max, slider.step, slider.binding
                ));
            }
            HIRView::Divider(_) => {
                self.xline("<Border BorderThickness=\"0,0,0,1\" BorderBrush=\"{ThemeResource SystemBaseMediumLowColor}\" Margin=\"0,8\" />");
            }
            HIRView::Spacer => {
                self.xline("<Border Height=\"1\" HorizontalAlignment=\"Stretch\" />");
            }
            HIRView::Progress(_) => {
                self.xline("<ProgressBar IsIndeterminate=\"True\" />");
            }
            HIRView::Badge(badge) => {
                let content = self.expr_str(&badge.content);
                self.xline(&format!(
                    "<Border CornerRadius=\"12\" Background=\"{{ThemeResource SystemAccentColor}}\" Padding=\"6,2\"><TextBlock Text=\"{}\" Foreground=\"White\" FontSize=\"12\"/></Border>",
                    content
                ));
            }
            HIRView::Conditional(cond) => {
                self.xline("<!-- Conditional -->");
                self.emit_view_xaml(&cond.then_view);
            }
            HIRView::Each(each) => {
                self.xline(&format!("<!-- Each {} -->", each.item_name));
                self.xline("<ItemsRepeater>");
                self.indent += 1;
                self.xline("<ItemsRepeater.ItemTemplate>");
                self.indent += 1;
                self.xline("<DataTemplate>");
                self.indent += 1;
                self.emit_view_xaml(&each.body);
                self.indent -= 1;
                self.xline("</DataTemplate>");
                self.indent -= 1;
                self.xline("</ItemsRepeater.ItemTemplate>");
                self.indent -= 1;
                self.xline("</ItemsRepeater>");
            }
            HIRView::ComponentRef(comp_ref) => {
                if let Some(comp) = self.module.components.iter().find(|c| c.name == comp_ref.name) {
                    self.emit_view_xaml(&comp.view);
                } else {
                    self.xline(&format!("<!-- Component: {} -->", comp_ref.name));
                }
            }
            HIRView::Group(children) => {
                for child in children { self.emit_view_xaml(child); }
            }
            _ => {
                self.xline("<!-- Unsupported element -->");
            }
        }
    }

    fn generate_cs(&mut self) {
        self.cline(&format!("namespace {};", self.module.app.name));
        self.cline("");
        self.cline("using Microsoft.UI.Xaml;");
        self.cline("using Microsoft.UI.Xaml.Controls;");
        self.cline("using System.Collections.ObjectModel;");
        self.cline("using System.ComponentModel;");
        self.cline("");

        // Models
        for model in &self.module.models {
            self.cline(&format!("public class {}", model.name));
            self.cline("{");
            self.indent += 1;
            for field in &model.fields {
                let cs_type = self.type_to_cs(&field.field_type);
                let default = field.default.as_ref()
                    .map(|d| format!(" = {}", self.expr_to_cs(d)))
                    .unwrap_or_default();
                self.cline(&format!("public {} {} {{ get; set; }}{};", cs_type, capitalize(&field.name), default));
            }
            self.indent -= 1;
            self.cline("}");
            self.cline("");
        }

        // Page class
        self.cline("public sealed partial class MainPage : Page, INotifyPropertyChanged");
        self.cline("{");
        self.indent += 1;
        self.cline("public event PropertyChangedEventHandler PropertyChanged;");
        self.cline("");

        // State as properties
        if let Some(screen) = self.module.screens.first() {
            for state in &screen.state {
                let cs_type = self.type_to_cs(&state.state_type);
                let default = state.initial.as_ref()
                    .map(|d| self.expr_to_cs(d))
                    .unwrap_or_else(|| self.default_cs(&state.state_type));
                self.cline(&format!("private {} _{} = {};", cs_type, state.name, default));
                self.cline(&format!("public {} {}", cs_type, capitalize(&state.name)));
                self.cline("{");
                self.indent += 1;
                self.cline(&format!("get => _{};", state.name));
                self.cline(&format!("set {{ _{} = value; PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof({}))); }}", state.name, capitalize(&state.name)));
                self.indent -= 1;
                self.cline("}");
                self.cline("");
            }

            // Actions
            for action in &screen.actions {
                let params: Vec<String> = action.params.iter()
                    .map(|p| format!("{} {}", self.type_to_cs(&p.param_type), p.name))
                    .collect();
                self.cline(&format!("public void {}({})", capitalize(&action.name), params.join(", ")));
                self.cline("{");
                self.indent += 1;
                for stmt in &action.body {
                    self.emit_stmt_cs(stmt);
                }
                self.indent -= 1;
                self.cline("}");
                self.cline("");
            }
        }

        self.cline("public MainPage()");
        self.cline("{");
        self.indent += 1;
        self.cline("this.InitializeComponent();");
        self.indent -= 1;
        self.cline("}");

        self.indent -= 1;
        self.cline("}");
    }

    fn emit_stmt_cs(&mut self, stmt: &HIRStmt) {
        match stmt {
            HIRStmt::Assign(name, value) => {
                self.cline(&format!("{} = {};", capitalize(name), self.expr_to_cs(value)));
            }
            HIRStmt::Let(name, _, value) => {
                self.cline(&format!("var {} = {};", name, self.expr_to_cs(value)));
            }
            HIRStmt::Return(Some(value)) => {
                self.cline(&format!("return {};", self.expr_to_cs(value)));
            }
            HIRStmt::Return(None) => self.cline("return;"),
            _ => self.cline("// unsupported statement"),
        }
    }

    fn expr_str(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => s.clone(),
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::Var(name, _) => format!("{{{}}}", name),
            _ => "...".to_string(),
        }
    }

    fn expr_to_cs(&self, expr: &HIRExpr) -> String {
        match expr {
            HIRExpr::StringLit(s) => format!("\"{}\"", s),
            HIRExpr::IntLit(n) => n.to_string(),
            HIRExpr::FloatLit(f) => format!("{}d", f),
            HIRExpr::BoolLit(b) => b.to_string(),
            HIRExpr::Nil => "null".to_string(),
            HIRExpr::Var(name, _) => capitalize(name),
            HIRExpr::BinOp(l, op, r, _) => {
                let op_str = match op {
                    aura_core::ast::BinOp::Add => "+",
                    aura_core::ast::BinOp::Sub => "-",
                    aura_core::ast::BinOp::Mul => "*",
                    aura_core::ast::BinOp::Div => "/",
                    aura_core::ast::BinOp::Eq => "==",
                    aura_core::ast::BinOp::NotEq => "!=",
                    aura_core::ast::BinOp::And => "&&",
                    aura_core::ast::BinOp::Or => "||",
                    _ => "/* ? */",
                };
                format!("({} {} {})", self.expr_to_cs(l), op_str, self.expr_to_cs(r))
            }
            HIRExpr::Constructor(name, args, _) => {
                let fields: Vec<String> = args.iter()
                    .filter(|(k, _)| k != "_")
                    .map(|(k, v)| format!("{} = {}", capitalize(k), self.expr_to_cs(v)))
                    .collect();
                format!("new {} {{ {} }}", name, fields.join(", "))
            }
            _ => "null".to_string(),
        }
    }

    fn type_to_cs(&self, ty: &aura_core::types::AuraType) -> String {
        use aura_core::types::*;
        match ty {
            AuraType::Primitive(p) => match p {
                PrimitiveType::Text => "string".to_string(),
                PrimitiveType::Int => "int".to_string(),
                PrimitiveType::Float => "double".to_string(),
                PrimitiveType::Bool => "bool".to_string(),
                PrimitiveType::Timestamp => "DateTimeOffset".to_string(),
                PrimitiveType::Duration => "TimeSpan".to_string(),
                PrimitiveType::Percent => "double".to_string(),
            },
            AuraType::List(inner) => format!("ObservableCollection<{}>", self.type_to_cs(inner)),
            AuraType::Optional(inner) => format!("{}?", self.type_to_cs(inner)),
            AuraType::Named(name) => name.clone(),
            AuraType::Action(_) => "Action".to_string(),
            _ => "object".to_string(),
        }
    }

    fn default_cs(&self, ty: &aura_core::types::AuraType) -> String {
        use aura_core::types::*;
        match ty {
            AuraType::Primitive(PrimitiveType::Text) => "\"\"".to_string(),
            AuraType::Primitive(PrimitiveType::Int) => "0".to_string(),
            AuraType::Primitive(PrimitiveType::Float) => "0.0".to_string(),
            AuraType::Primitive(PrimitiveType::Bool) => "false".to_string(),
            AuraType::List(inner) => format!("new ObservableCollection<{}>()", self.type_to_cs(inner)),
            AuraType::Optional(_) => "null".to_string(),
            _ => "null".to_string(),
        }
    }

    fn spacing_val(&self, design: &design::ResolvedDesign) -> String {
        design.spacing.as_ref().and_then(|s| s.gap).map(|g| format!("{}", g)).unwrap_or_else(|| "8".to_string())
    }

    fn padding_val(&self, design: &design::ResolvedDesign) -> String {
        design.spacing.as_ref().and_then(|s| s.padding_top).map(|p| format!("{}", p)).unwrap_or_else(|| "0".to_string())
    }

    fn text_style_xaml(&self, design: &design::ResolvedDesign) -> String {
        let mut attrs = Vec::new();
        if let Some(ref typo) = design.typography {
            if let Some(size) = typo.size {
                attrs.push(format!(" FontSize=\"{}\"", (size * 16.0).round()));
            }
            if typo.weight == Some(700) { attrs.push(" FontWeight=\"Bold\"".to_string()); }
            if typo.weight == Some(500) { attrs.push(" FontWeight=\"Medium\"".to_string()); }
            if typo.italic { attrs.push(" FontStyle=\"Italic\"".to_string()); }
        }
        if let Some(ref color) = design.color {
            if let Some(ref fg) = color.foreground {
                let brush = match fg.as_str() {
                    "accent" => "{ThemeResource SystemAccentColor}",
                    "secondary" | "muted" => "{ThemeResource SystemBaseMediumColor}",
                    "danger" => "#DC3545",
                    "success" => "#28A745",
                    _ => "",
                };
                if !brush.is_empty() {
                    attrs.push(format!(" Foreground=\"{}\"", brush));
                }
            }
        }
        attrs.join("")
    }

    fn button_style(&self, design: &design::ResolvedDesign) -> String {
        let mut attrs = Vec::new();
        if let Some(ref shape) = design.shape {
            if shape.kind == design::ShapeKind::Pill {
                attrs.push(" CornerRadius=\"20\"".to_string());
            }
        }
        attrs.join("")
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map(|c| c.to_uppercase().to_string()).unwrap_or_default() + chars.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_source(source: &str) -> WinUiOutput {
        let result = aura_core::parser::parse(source);
        assert!(result.errors.is_empty(), "Parse errors: {:?}", result.errors);
        let hir = aura_core::hir::build_hir(result.program.as_ref().unwrap());
        compile_to_winui(&hir)
    }

    #[test]
    fn test_winui_minimal() {
        let output = compile_source("app Hello\n  screen Main\n    view\n      text \"Hello, Aura!\"");
        assert!(output.xaml.contains("<Page"));
        assert!(output.xaml.contains("TextBlock"));
        assert!(output.xaml.contains("Hello, Aura!"));
        assert!(output.cs.contains("namespace Hello"));
        assert!(output.cs.contains("MainPage"));
    }

    #[test]
    fn test_winui_state() {
        let output = compile_source("\
app Test
  screen Main
    state count: int = 0
    view
      text \"hi\"");
        assert!(output.cs.contains("private int _count = 0"));
        assert!(output.cs.contains("public int Count"));
        assert!(output.cs.contains("PropertyChanged"));
    }

    #[test]
    fn test_winui_model() {
        let output = compile_source("\
app Test
  model Todo
    title: text
    done: bool = false
  screen Main
    view
      text \"hi\"");
        assert!(output.cs.contains("public class Todo"));
        assert!(output.cs.contains("public string Title"));
        assert!(output.cs.contains("public bool Done"));
    }

    #[test]
    fn test_winui_layout() {
        let output = compile_source("\
app Test
  screen Main
    view
      column gap.md padding.lg
        row gap.sm
          text \"A\"
          text \"B\"");
        assert!(output.xaml.contains("StackPanel Orientation=\"Vertical\""));
        assert!(output.xaml.contains("StackPanel Orientation=\"Horizontal\""));
        assert!(output.xaml.contains("Spacing=\"8\""));
    }

    #[test]
    fn test_winui_button() {
        let output = compile_source("\
app Test
  screen Main
    view
      button \"Save\" .accent -> save()
    action save
      return");
        assert!(output.xaml.contains("Button Content=\"Save\""));
        assert!(output.cs.contains("public void Save()"));
    }

    #[test]
    fn test_winui_inputs() {
        let output = compile_source("\
app Test
  screen Main
    state q: text = \"\"
    state dark: bool = false
    view
      textfield q placeholder: \"Search...\"
      toggle dark label: \"Dark Mode\"");
        assert!(output.xaml.contains("TextBox PlaceholderText=\"Search...\""));
        assert!(output.xaml.contains("ToggleSwitch Header=\"Dark Mode\""));
    }
}
