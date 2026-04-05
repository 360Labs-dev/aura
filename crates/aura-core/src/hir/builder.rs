//! HIR Builder — transforms AST into HIR.
//!
//! Takes a parsed + analyzed AST and produces an HIRModule
//! that codegen backends consume.

use crate::ast;
use crate::design::{self, ResolvedDesign, ResolvedSpacing, ResolvedTypography, ResolvedColor, ResolvedShape};
use crate::hir::nodes::*;
use crate::types::AuraType;

/// Default theme spacing base in pixels.
const DEFAULT_SPACING_BASE: f64 = 8.0;

/// Build HIR from a parsed program.
pub fn build_hir(program: &ast::Program) -> HIRModule {
    let mut builder = HIRBuilder::new();
    builder.build(program)
}

struct HIRBuilder {
    spacing_base: f64,
}

impl HIRBuilder {
    fn new() -> Self {
        Self {
            spacing_base: DEFAULT_SPACING_BASE,
        }
    }

    fn build(&mut self, program: &ast::Program) -> HIRModule {
        let mut app_name = String::new();
        let mut theme_name = None;
        let mut nav_mode = NavigationMode::Stack;
        let mut routes = Vec::new();
        let mut models = Vec::new();
        let mut screens = Vec::new();
        let mut components = Vec::new();
        let mut themes = Vec::new();
        let mut app_state = Vec::new();
        let mut app_fns = Vec::new();

        app_name = program.app.name.clone();

        for member in &program.app.members {
            match member {
                ast::AppMember::ThemeRef(t) => {
                    theme_name = Some(self.expr_to_string(&t.value));
                }
                ast::AppMember::NavigationDecl(n) => {
                    nav_mode = match n.mode.as_str() {
                        "tabs" => NavigationMode::Tabs,
                        _ => NavigationMode::Stack,
                    };
                }
                ast::AppMember::RouteDecl(r) => {
                    routes.push(HIRRoute {
                        pattern: r.pattern.clone(),
                        screen: r.screen.clone(),
                    });
                }
                ast::AppMember::Model(m) => {
                    models.push(self.build_model(m));
                }
                ast::AppMember::Screen(s) => {
                    screens.push(self.build_screen(s));
                }
                ast::AppMember::Component(c) => {
                    components.push(self.build_component(c));
                }
                ast::AppMember::State(s) => {
                    app_state.push(self.build_state(s));
                }
                ast::AppMember::Fn(f) => {
                    app_fns.push(self.build_function(f));
                }
                _ => {}
            }
        }

        HIRModule {
            app: HIRApp {
                name: app_name,
                theme: theme_name,
                navigation: nav_mode,
                routes,
            },
            models,
            screens,
            components,
            themes,
        }
    }

    // === Model ===

    fn build_model(&self, model: &ast::ModelDecl) -> HIRModel {
        HIRModel {
            name: model.name.clone(),
            fields: model
                .fields
                .iter()
                .map(|f| HIRField {
                    name: f.name.clone(),
                    field_type: self.resolve_ast_type(&f.type_expr),
                    default: f.default.as_ref().map(|e| self.build_expr(e)),
                })
                .collect(),
        }
    }

    // === Screen ===

    fn build_screen(&mut self, screen: &ast::ScreenDecl) -> HIRScreen {
        let mut state = Vec::new();
        let mut view = HIRView::Group(Vec::new());
        let mut actions = Vec::new();
        let mut functions = Vec::new();
        let tab = self.build_screen_tab(&screen.modifiers);

        let params: Vec<HIRParam> = screen
            .params
            .iter()
            .map(|p| self.build_param(p))
            .collect();

        for member in &screen.members {
            match member {
                ast::ScreenMember::State(s) => state.push(self.build_state(s)),
                ast::ScreenMember::View(v) => view = self.build_view(v),
                ast::ScreenMember::Action(a) => actions.push(self.build_action(a)),
                ast::ScreenMember::Fn(f) => functions.push(self.build_function(f)),
                _ => {}
            }
        }

        HIRScreen {
            name: screen.name.clone(),
            params,
            state,
            view,
            actions,
            functions,
            tab,
        }
    }

    fn build_screen_tab(&self, modifiers: &[ast::ScreenModifier]) -> Option<HIRTab> {
        let icon = modifiers.iter().find_map(|m| {
            if let ast::ScreenModifier::Tab(i) = m {
                Some(i.clone())
            } else {
                None
            }
        });
        let label = modifiers.iter().find_map(|m| {
            if let ast::ScreenModifier::Label(l) = m {
                Some(l.clone())
            } else {
                None
            }
        });
        icon.map(|i| HIRTab {
            icon: i,
            label: label.unwrap_or_default(),
        })
    }

    // === Component ===

    fn build_component(&mut self, comp: &ast::ComponentDecl) -> HIRComponent {
        let props: Vec<HIRParam> = comp.props.iter().map(|p| self.build_param(p)).collect();
        let mut state = Vec::new();
        let mut view = HIRView::Group(Vec::new());
        let mut actions = Vec::new();
        let mut functions = Vec::new();

        for member in &comp.members {
            match member {
                ast::ScreenMember::State(s) => state.push(self.build_state(s)),
                ast::ScreenMember::View(v) => view = self.build_view(v),
                ast::ScreenMember::Action(a) => actions.push(self.build_action(a)),
                ast::ScreenMember::Fn(f) => functions.push(self.build_function(f)),
                _ => {}
            }
        }

        HIRComponent {
            name: comp.name.clone(),
            props,
            state,
            view,
            actions,
            functions,
        }
    }

    // === View ===

    fn build_view(&mut self, view: &ast::ViewDecl) -> HIRView {
        if view.body.len() == 1 {
            self.build_view_element(&view.body[0])
        } else {
            HIRView::Group(
                view.body
                    .iter()
                    .map(|e| self.build_view_element(e))
                    .collect(),
            )
        }
    }

    fn build_view_element(&mut self, elem: &ast::ViewElement) -> HIRView {
        match elem {
            ast::ViewElement::Layout(layout) => self.build_layout(layout),
            ast::ViewElement::Widget(widget) => self.build_widget(widget),
            ast::ViewElement::Input(input) => self.build_input(input),
            ast::ViewElement::Button(button) => self.build_button(button),
            ast::ViewElement::If(if_view) => self.build_if(if_view),
            ast::ViewElement::Each(each) => self.build_each(each),
            ast::ViewElement::When(when) => self.build_when(when),
            ast::ViewElement::ComponentRef(comp) => self.build_comp_ref(comp),
            ast::ViewElement::Spacer(_) => HIRView::Spacer,
            ast::ViewElement::Divider(tokens, _) => {
                HIRView::Divider(self.resolve_design(tokens, &[]))
            }
            ast::ViewElement::Slot(_) => HIRView::Slot,
        }
    }

    fn build_layout(&mut self, layout: &ast::LayoutElement) -> HIRView {
        let design = self.resolve_design(&layout.tokens, &layout.props);
        let children: Vec<HIRView> = layout
            .children
            .iter()
            .map(|c| self.build_view_element(c))
            .collect();
        let hir_layout = HIRLayout { design, children };

        match layout.kind {
            ast::LayoutKind::Column => HIRView::Column(hir_layout),
            ast::LayoutKind::Row => HIRView::Row(hir_layout),
            ast::LayoutKind::Stack => HIRView::Stack(hir_layout),
            ast::LayoutKind::Grid => HIRView::Grid(HIRGridLayout {
                columns: None,
                min_width: None,
                design: hir_layout.design,
                children: hir_layout.children,
            }),
            ast::LayoutKind::Scroll => HIRView::Scroll(HIRScrollLayout {
                direction: ScrollDirection::Vertical,
                design: hir_layout.design,
                children: hir_layout.children,
            }),
            ast::LayoutKind::Wrap => HIRView::Wrap(hir_layout),
        }
    }

    fn build_widget(&self, widget: &ast::WidgetElement) -> HIRView {
        let design = self.resolve_design(&widget.tokens, &widget.props);
        let content = widget
            .args
            .first()
            .map(|e| self.build_expr(e))
            .unwrap_or(HIRExpr::StringLit(String::new()));

        match widget.kind {
            ast::WidgetKind::Text => HIRView::Text(HIRText { content, design }),
            ast::WidgetKind::Heading => HIRView::Heading(HIRHeading {
                content,
                level: 1,
                design,
            }),
            ast::WidgetKind::Image => HIRView::Image(HIRImage {
                source: content,
                design,
            }),
            ast::WidgetKind::Icon => HIRView::Icon(HIRIcon {
                name: content,
                design,
            }),
            ast::WidgetKind::Badge => HIRView::Badge(HIRBadge { content, design }),
            ast::WidgetKind::Progress => HIRView::Progress(HIRProgress {
                value: content,
                design,
            }),
            ast::WidgetKind::Avatar => HIRView::Avatar(HIRAvatar {
                source: content,
                design,
            }),
        }
    }

    fn build_input(&self, input: &ast::InputElement) -> HIRView {
        let design = self.resolve_design(&input.tokens, &input.props);
        let placeholder = input
            .props
            .iter()
            .find(|p| p.name == "placeholder")
            .and_then(|p| self.expr_to_string_opt(&p.value));

        match input.kind {
            ast::InputKind::TextField => HIRView::TextField(HIRTextField {
                binding: input.binding.clone(),
                placeholder,
                action: input.action.as_ref().map(|a| Box::new(self.build_action_expr(a))),
                design,
            }),
            ast::InputKind::TextArea => HIRView::TextArea(HIRTextArea {
                binding: input.binding.clone(),
                placeholder,
                design,
            }),
            ast::InputKind::Checkbox => HIRView::Checkbox(HIRCheckbox {
                binding: input.binding.clone(),
                design,
            }),
            ast::InputKind::Toggle => HIRView::Toggle(HIRToggle {
                binding: input.binding.clone(),
                label: input.props.iter().find(|p| p.name == "label").and_then(|p| self.expr_to_string_opt(&p.value)),
                design,
            }),
            ast::InputKind::Slider => HIRView::Slider(HIRSlider {
                binding: input.binding.clone(),
                min: self.prop_float(&input.props, "min", 0.0),
                max: self.prop_float(&input.props, "max", 100.0),
                step: self.prop_float(&input.props, "step", 1.0),
                design,
            }),
            ast::InputKind::Picker => HIRView::Picker(HIRPicker {
                binding: input.binding.clone(),
                options: input.props.iter().find(|p| p.name == "options")
                    .map(|p| self.build_expr(&p.value))
                    .unwrap_or(HIRExpr::StringLit(String::new())),
                design,
            }),
            ast::InputKind::DatePicker => HIRView::DatePicker(HIRDatePicker {
                binding: input.binding.clone(),
                label: input.props.iter().find(|p| p.name == "label").and_then(|p| self.expr_to_string_opt(&p.value)),
                design,
            }),
            ast::InputKind::Segmented => HIRView::Segmented(HIRSegmented {
                binding: input.binding.clone(),
                options: input.props.iter().find(|p| p.name == "options")
                    .map(|p| self.build_expr(&p.value))
                    .unwrap_or(HIRExpr::StringLit(String::new())),
                design,
            }),
            ast::InputKind::Stepper => HIRView::Slider(HIRSlider {
                binding: input.binding.clone(),
                min: self.prop_float(&input.props, "min", 0.0),
                max: self.prop_float(&input.props, "max", 100.0),
                step: self.prop_float(&input.props, "step", 1.0),
                design,
            }),
        }
    }

    fn build_button(&self, button: &ast::ButtonElement) -> HIRView {
        let design = self.resolve_design(&button.tokens, &button.props);
        let style = match button.style.as_deref() {
            Some("icon") => ButtonStyle::Icon,
            Some("outline") => ButtonStyle::Outline,
            Some("ghost") => ButtonStyle::Ghost,
            Some("link") => ButtonStyle::Link,
            _ => ButtonStyle::Default,
        };

        HIRView::Button(HIRButton {
            label: self.build_expr(&button.label),
            style,
            action: self.build_action_expr(&button.action),
            design,
        })
    }

    fn build_if(&mut self, if_view: &ast::IfView) -> HIRView {
        HIRView::Conditional(HIRConditional {
            condition: self.build_expr(&if_view.condition),
            then_view: Box::new(if if_view.then_body.len() == 1 {
                self.build_view_element(&if_view.then_body[0])
            } else {
                HIRView::Group(if_view.then_body.iter().map(|e| self.build_view_element(e)).collect())
            }),
            else_view: if_view.else_body.as_ref().map(|body| {
                Box::new(if body.len() == 1 {
                    self.build_view_element(&body[0])
                } else {
                    HIRView::Group(body.iter().map(|e| self.build_view_element(e)).collect())
                })
            }),
        })
    }

    fn build_each(&mut self, each: &ast::EachView) -> HIRView {
        let body = if each.body.len() == 1 {
            self.build_view_element(&each.body[0])
        } else {
            HIRView::Group(each.body.iter().map(|e| self.build_view_element(e)).collect())
        };

        HIRView::Each(HIREach {
            iterable: self.build_expr(&each.iterable),
            item_name: each.item_name.clone(),
            index_name: each.index_name.clone(),
            body: Box::new(body),
        })
    }

    fn build_when(&mut self, when: &ast::WhenView) -> HIRView {
        HIRView::Switch(HIRSwitch {
            expression: self.build_expr(&when.expression),
            cases: when
                .branches
                .iter()
                .map(|b| HIRSwitchCase {
                    pattern: self.build_pattern(&b.pattern),
                    view: if b.body.len() == 1 {
                        self.build_view_element(&b.body[0])
                    } else {
                        HIRView::Group(b.body.iter().map(|e| self.build_view_element(e)).collect())
                    },
                })
                .collect(),
        })
    }

    fn build_comp_ref(&mut self, comp: &ast::ComponentRef) -> HIRView {
        HIRView::ComponentRef(HIRComponentRef {
            name: comp.name.clone(),
            args: comp
                .args
                .iter()
                .map(|(k, v)| (k.clone(), self.build_expr(v)))
                .collect(),
            children: comp
                .children
                .iter()
                .map(|c| self.build_view_element(c))
                .collect(),
        })
    }

    // === Actions & Functions ===

    fn build_action(&self, action: &ast::ActionDecl) -> HIRAction {
        HIRAction {
            name: action.name.clone(),
            params: action.params.iter().map(|p| self.build_param(p)).collect(),
            body: action.body.iter().map(|s| self.build_stmt(s)).collect(),
        }
    }

    fn build_function(&self, func: &ast::FnDecl) -> HIRFunction {
        HIRFunction {
            name: func.name.clone(),
            params: func.params.iter().map(|p| self.build_param(p)).collect(),
            return_type: func
                .return_type
                .as_ref()
                .map(|t| self.resolve_ast_type(t))
                .unwrap_or(AuraType::Poison),
            body: func.body.iter().map(|s| self.build_stmt(s)).collect(),
        }
    }

    fn build_state(&self, state: &ast::StateDecl) -> HIRState {
        HIRState {
            name: state.name.clone(),
            state_type: self.resolve_ast_type(&state.type_expr),
            initial: state.default.as_ref().map(|e| self.build_expr(e)),
        }
    }

    fn build_param(&self, param: &ast::Param) -> HIRParam {
        HIRParam {
            name: param.name.clone(),
            param_type: self.resolve_ast_type(&param.type_expr),
            default: param.default.as_ref().map(|e| self.build_expr(e)),
        }
    }

    // === Statements ===

    fn build_stmt(&self, stmt: &ast::Stmt) -> HIRStmt {
        match stmt {
            ast::Stmt::Assign(name, value, _) => {
                HIRStmt::Assign(name.clone(), self.build_expr(value))
            }
            ast::Stmt::Let(name, type_expr, value, _) => {
                let ty = type_expr
                    .as_ref()
                    .map(|t| self.resolve_ast_type(t))
                    .unwrap_or(AuraType::Poison);
                HIRStmt::Let(name.clone(), ty, self.build_expr(value))
            }
            ast::Stmt::If(cond, then_body, else_body, _) => HIRStmt::If(
                self.build_expr(cond),
                then_body.iter().map(|s| self.build_stmt(s)).collect(),
                else_body.as_ref().map(|b| b.iter().map(|s| self.build_stmt(s)).collect()),
            ),
            ast::Stmt::When(expr, branches, _) => HIRStmt::When(
                self.build_expr(expr),
                branches
                    .iter()
                    .map(|b| {
                        let body = match &b.body {
                            ast::StmtOrExpr::Stmt(stmts) => {
                                HIRStmtOrExpr::Stmts(stmts.iter().map(|s| self.build_stmt(s)).collect())
                            }
                            ast::StmtOrExpr::Expr(e) => HIRStmtOrExpr::Expr(self.build_expr(e)),
                        };
                        (self.build_pattern(&b.pattern), body)
                    })
                    .collect(),
            ),
            ast::Stmt::Navigate(nav) => HIRStmt::Navigate(self.build_navigate(nav)),
            ast::Stmt::Emit(name, args, _) => {
                HIRStmt::Emit(name.clone(), args.iter().map(|a| self.build_expr(a)).collect())
            }
            ast::Stmt::Return(value, _) => {
                HIRStmt::Return(value.as_ref().map(|e| self.build_expr(e)))
            }
            ast::Stmt::Expr(expr, _) => HIRStmt::Expr(self.build_expr(expr)),
        }
    }

    // === Expressions ===

    fn build_expr(&self, expr: &ast::Expr) -> HIRExpr {
        match expr {
            ast::Expr::IntLit(n, _) => HIRExpr::IntLit(*n),
            ast::Expr::FloatLit(f, _) => HIRExpr::FloatLit(*f),
            ast::Expr::StringLit(s, _) => HIRExpr::StringLit(s.clone()),
            ast::Expr::PercentLit(p, _) => HIRExpr::PercentLit(*p),
            ast::Expr::BoolLit(b, _) => HIRExpr::BoolLit(*b),
            ast::Expr::Nil(_) => HIRExpr::Nil,
            ast::Expr::Var(name, _) => HIRExpr::Var(name.clone(), AuraType::Poison),
            ast::Expr::MemberAccess(obj, member, _) => {
                HIRExpr::MemberAccess(Box::new(self.build_expr(obj)), member.clone(), AuraType::Poison)
            }
            ast::Expr::Call(func, args, _) => HIRExpr::Call(
                Box::new(self.build_expr(func)),
                args.iter().map(|a| self.build_expr(a)).collect(),
                AuraType::Poison,
            ),
            ast::Expr::NamedCall(func, args, _) => HIRExpr::NamedCall(
                Box::new(self.build_expr(func)),
                args.iter().map(|(k, v)| (k.clone(), self.build_expr(v))).collect(),
                AuraType::Poison,
            ),
            ast::Expr::Index(obj, idx, _) => HIRExpr::Index(
                Box::new(self.build_expr(obj)),
                Box::new(self.build_expr(idx)),
                AuraType::Poison,
            ),
            ast::Expr::BinOp(left, op, right, _) => HIRExpr::BinOp(
                Box::new(self.build_expr(left)),
                *op,
                Box::new(self.build_expr(right)),
                AuraType::Poison,
            ),
            ast::Expr::UnaryOp(op, operand, _) => {
                HIRExpr::UnaryOp(*op, Box::new(self.build_expr(operand)), AuraType::Poison)
            }
            ast::Expr::Lambda(params, body, _) => HIRExpr::Lambda(
                params.iter().map(|p| self.build_param(p)).collect(),
                Box::new(self.build_expr(body)),
                AuraType::Poison,
            ),
            ast::Expr::Constructor(name, args, _) => HIRExpr::Constructor(
                name.clone(),
                args.iter().map(|(k, v)| (k.clone(), self.build_expr(v))).collect(),
                AuraType::Named(name.clone()),
            ),
            ast::Expr::Pipe(left, right, _) => HIRExpr::Pipe(
                Box::new(self.build_expr(left)),
                Box::new(self.build_expr(right)),
                AuraType::Poison,
            ),
            ast::Expr::Conditional(cond, then_e, else_e, _) => HIRExpr::Conditional(
                Box::new(self.build_expr(cond)),
                Box::new(self.build_expr(then_e)),
                Box::new(self.build_expr(else_e)),
                AuraType::Poison,
            ),
            ast::Expr::NilCoalesce(left, right, _) => HIRExpr::NilCoalesce(
                Box::new(self.build_expr(left)),
                Box::new(self.build_expr(right)),
                AuraType::Poison,
            ),
            ast::Expr::DesignToken(segments, _) => {
                HIRExpr::StringLit(format!(".{}", segments.join(".")))
            }
        }
    }

    fn build_action_expr(&self, action: &ast::ActionExpr) -> HIRActionExpr {
        match action {
            ast::ActionExpr::Call(name, args, _) => {
                HIRActionExpr::Call(name.clone(), args.iter().map(|a| self.build_expr(a)).collect())
            }
            ast::ActionExpr::Navigate(nav) => HIRActionExpr::Navigate(self.build_navigate_from_ast(nav)),
            ast::ActionExpr::Lambda(_, body, _) => {
                // Lambda action — convert body to an action call
                HIRActionExpr::Call("_lambda".to_string(), vec![self.build_expr(body)])
            }
        }
    }

    fn build_navigate(&self, nav: &ast::NavigateExpr) -> HIRNavigate {
        match nav {
            ast::NavigateExpr::To(expr, _) => HIRNavigate::To(self.build_expr(expr)),
            ast::NavigateExpr::Back(_) => HIRNavigate::Back,
            ast::NavigateExpr::Root(_) => HIRNavigate::Root,
            ast::NavigateExpr::Replace(expr, _) => HIRNavigate::Replace(self.build_expr(expr)),
            ast::NavigateExpr::Modal(expr, _) => HIRNavigate::Modal(self.build_expr(expr)),
            ast::NavigateExpr::Dismiss(_) => HIRNavigate::Dismiss,
        }
    }

    fn build_navigate_from_ast(&self, nav: &ast::NavigateExpr) -> HIRNavigate {
        self.build_navigate(nav)
    }

    fn build_pattern(&self, pattern: &ast::Pattern) -> HIRPattern {
        match pattern {
            ast::Pattern::Literal(expr) => HIRPattern::Literal(self.build_expr(expr)),
            ast::Pattern::Identifier(name, _) => HIRPattern::EnumVariant(name.clone()),
            ast::Pattern::EnumVariant(name, _) => HIRPattern::EnumVariant(name.clone()),
            ast::Pattern::Some(name, _) => HIRPattern::Some(name.clone()),
            ast::Pattern::Nil(_) => HIRPattern::Nil,
            ast::Pattern::Constructor(name, _, _) => HIRPattern::EnumVariant(name.clone()),
        }
    }

    // === Design token resolution ===

    fn resolve_design(&self, tokens: &[ast::DesignToken], props: &[ast::PropAssign]) -> ResolvedDesign {
        let mut design = ResolvedDesign::default();

        for token in tokens {
            let segments: Vec<&str> = token.segments.iter().map(|s| s.as_str()).collect();
            self.apply_design_token(&mut design, &segments);
        }

        // Apply prop-based overrides (e.g., padding: 20)
        for prop in props {
            match prop.name.as_str() {
                "gap" => {
                    if let Some(v) = self.expr_to_float(&prop.value) {
                        let spacing = design.spacing.get_or_insert_with(|| ResolvedSpacing {
                            gap: None, padding_top: None, padding_bottom: None,
                            padding_left: None, padding_right: None,
                            margin_top: None, margin_bottom: None,
                            margin_left: None, margin_right: None,
                        });
                        spacing.gap = Some(v);
                    }
                }
                _ => {}
            }
        }

        design
    }

    fn apply_design_token(&self, design: &mut ResolvedDesign, segments: &[&str]) {
        match segments {
            // Compound tokens: gap.md, padding.lg, etc.
            ["gap", size] => {
                if let Some(px) = design::resolve_spacing(size, self.spacing_base) {
                    let spacing = design.spacing.get_or_insert_with(Default::default);
                    spacing.gap = Some(px);
                }
            }
            ["padding", size] => {
                if let Some(px) = design::resolve_spacing(size, self.spacing_base) {
                    let spacing = design.spacing.get_or_insert_with(Default::default);
                    spacing.padding_top = Some(px);
                    spacing.padding_bottom = Some(px);
                    spacing.padding_left = Some(px);
                    spacing.padding_right = Some(px);
                }
            }
            ["padding", dir, size] => {
                if let Some(px) = design::resolve_spacing(size, self.spacing_base) {
                    let spacing = design.spacing.get_or_insert_with(Default::default);
                    match *dir {
                        "top" => spacing.padding_top = Some(px),
                        "bottom" => spacing.padding_bottom = Some(px),
                        "left" | "leading" | "start" => spacing.padding_left = Some(px),
                        "right" | "trailing" | "end" => spacing.padding_right = Some(px),
                        "horizontal" => {
                            spacing.padding_left = Some(px);
                            spacing.padding_right = Some(px);
                        }
                        "vertical" => {
                            spacing.padding_top = Some(px);
                            spacing.padding_bottom = Some(px);
                        }
                        _ => {}
                    }
                }
            }
            ["margin", size] => {
                if let Some(px) = design::resolve_spacing(size, self.spacing_base) {
                    let spacing = design.spacing.get_or_insert_with(Default::default);
                    spacing.margin_top = Some(px);
                    spacing.margin_bottom = Some(px);
                    spacing.margin_left = Some(px);
                    spacing.margin_right = Some(px);
                }
            }
            ["size", size_tok] => {
                // size.xl, size.2xl, size.display → typography size
                if let Some(size_val) = design::typography_size(size_tok) {
                    let typo = design.typography.get_or_insert_with(Default::default);
                    typo.size = Some(size_val);
                }
            }
            ["width", _size] | ["height", _size] => {
                // width/height tokens — handled at a higher level
            }
            ["align", direction] => {
                match *direction {
                    "center" => {
                        // Center both flex alignment AND text
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.alignment = Some(design::TextAlignment::Center);
                    }
                    "start" | "leading" => {}
                    "end" | "trailing" => {}
                    _ => {}
                }
            }
            ["justify", mode] => {
                // justify.center, justify.between etc — handled via CSS classes
                let _ = mode;
            }
            // Single tokens
            [single] => {
                // Spacing (when used directly on elements that support it)
                if design::spacing_multiplier(single).is_some() {
                    // Context-dependent: could be spacing or typography size
                    // Default to spacing for layouts, typography for text
                }

                // Typography weight
                if let Some(weight) = design::font_weight(single) {
                    let typo = design.typography.get_or_insert_with(Default::default);
                    typo.weight = Some(weight);
                }

                // Typography size (when prefixed with size.)
                if let Some(size) = design::typography_size(single) {
                    let typo = design.typography.get_or_insert_with(Default::default);
                    typo.size = Some(size);
                }

                // Typography style
                match *single {
                    "italic" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.italic = true;
                    }
                    "mono" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.mono = true;
                    }
                    "underline" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.underline = true;
                    }
                    "strike" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.strikethrough = true;
                    }
                    "center" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.alignment = Some(design::TextAlignment::Center);
                    }
                    "leading" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.alignment = Some(design::TextAlignment::Leading);
                    }
                    "trailing" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.alignment = Some(design::TextAlignment::Trailing);
                    }
                    "uppercase" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.transform = Some(design::TextTransform::Uppercase);
                    }
                    "lowercase" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.transform = Some(design::TextTransform::Lowercase);
                    }
                    "capitalize" => {
                        let typo = design.typography.get_or_insert_with(Default::default);
                        typo.transform = Some(design::TextTransform::Capitalize);
                    }
                    _ => {}
                }

                // Semantic colors
                match *single {
                    "primary" | "secondary" | "muted" | "accent" | "danger" | "warning"
                    | "success" | "info" => {
                        let color = design.color.get_or_insert_with(|| ResolvedColor {
                            foreground: None,
                            background: None,
                        });
                        color.foreground = Some(single.to_string());
                    }
                    "surface" | "background" => {
                        let color = design.color.get_or_insert_with(|| ResolvedColor {
                            foreground: None,
                            background: None,
                        });
                        color.background = Some(single.to_string());
                    }
                    _ => {}
                }

                // Shape
                if let Some(radius) = design::shape_radius(single) {
                    design.shape = Some(ResolvedShape {
                        radius,
                        kind: match *single {
                            "sharp" => design::ShapeKind::Sharp,
                            "subtle" => design::ShapeKind::Subtle,
                            "rounded" => design::ShapeKind::Rounded,
                            "smooth" => design::ShapeKind::Smooth,
                            "pill" => design::ShapeKind::Pill,
                            "circle" => design::ShapeKind::Circle,
                            _ => design::ShapeKind::Rounded,
                        },
                    });
                }
            }
            _ => {}
        }
    }

    // === Type resolution ===

    fn resolve_ast_type(&self, type_expr: &ast::TypeExpr) -> AuraType {
        match type_expr {
            ast::TypeExpr::Named(name, _) => match name.as_str() {
                "text" => AuraType::Primitive(crate::types::PrimitiveType::Text),
                "int" => AuraType::Primitive(crate::types::PrimitiveType::Int),
                "float" => AuraType::Primitive(crate::types::PrimitiveType::Float),
                "bool" => AuraType::Primitive(crate::types::PrimitiveType::Bool),
                "timestamp" => AuraType::Primitive(crate::types::PrimitiveType::Timestamp),
                "duration" => AuraType::Primitive(crate::types::PrimitiveType::Duration),
                "percent" => AuraType::Primitive(crate::types::PrimitiveType::Percent),
                "secret" => AuraType::Security(crate::types::SecurityType::Secret),
                "sanitized" => AuraType::Security(crate::types::SecurityType::Sanitized),
                "email" => AuraType::Security(crate::types::SecurityType::Email),
                "url" => AuraType::Security(crate::types::SecurityType::Url),
                "token" => AuraType::Security(crate::types::SecurityType::Token),
                _ => AuraType::Named(name.clone()),
            },
            ast::TypeExpr::Collection(kind, args, _) => {
                let resolved: Vec<_> = args.iter().map(|a| self.resolve_ast_type(a)).collect();
                match kind.as_str() {
                    "list" => AuraType::List(Box::new(resolved.into_iter().next().unwrap_or(AuraType::Poison))),
                    "set" => AuraType::Set(Box::new(resolved.into_iter().next().unwrap_or(AuraType::Poison))),
                    "map" => {
                        let mut iter = resolved.into_iter();
                        AuraType::Map(
                            Box::new(iter.next().unwrap_or(AuraType::Poison)),
                            Box::new(iter.next().unwrap_or(AuraType::Poison)),
                        )
                    }
                    _ => AuraType::Poison,
                }
            }
            ast::TypeExpr::Optional(inner, _) => {
                AuraType::Optional(Box::new(self.resolve_ast_type(inner)))
            }
            ast::TypeExpr::Enum(variants, _) => AuraType::Enum(
                variants.iter().map(|v| crate::types::EnumVariant {
                    name: v.name.clone(),
                    fields: v.fields.iter().map(|f| (f.name.clone(), self.resolve_ast_type(&f.type_expr))).collect(),
                }).collect(),
            ),
            ast::TypeExpr::Function(params, ret, _) => AuraType::Function(crate::types::FunctionType { type_params: Vec::new(),
                params: params.iter().map(|p| self.resolve_ast_type(p)).collect(),
                return_type: Box::new(ret.as_ref().map(|r| self.resolve_ast_type(r)).unwrap_or(AuraType::Poison)),
            }),
            ast::TypeExpr::Action(params, _) => {
                AuraType::Action(params.iter().map(|p| self.resolve_ast_type(p)).collect())
            }
        }
    }

    // === Helpers ===

    fn expr_to_string(&self, expr: &ast::Expr) -> String {
        match expr {
            ast::Expr::StringLit(s, _) => s.clone(),
            ast::Expr::Var(name, _) => name.clone(),
            ast::Expr::MemberAccess(obj, member, _) => {
                format!("{}.{}", self.expr_to_string(obj), member)
            }
            _ => format!("{:?}", expr),
        }
    }

    fn expr_to_string_opt(&self, expr: &ast::Expr) -> Option<String> {
        match expr {
            ast::Expr::StringLit(s, _) => Some(s.clone()),
            _ => None,
        }
    }

    fn expr_to_float(&self, expr: &ast::Expr) -> Option<f64> {
        match expr {
            ast::Expr::IntLit(n, _) => Some(*n as f64),
            ast::Expr::FloatLit(f, _) => Some(*f),
            _ => None,
        }
    }

    fn prop_float(&self, props: &[ast::PropAssign], name: &str, default: f64) -> f64 {
        props
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| self.expr_to_float(&p.value))
            .unwrap_or(default)
    }
}

// Need Default for ResolvedSpacing
impl Default for ResolvedSpacing {
    fn default() -> Self {
        Self {
            gap: None,
            padding_top: None,
            padding_bottom: None,
            padding_left: None,
            padding_right: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
        }
    }
}

impl Default for ResolvedTypography {
    fn default() -> Self {
        Self {
            size: None,
            weight: None,
            italic: false,
            mono: false,
            underline: false,
            strikethrough: false,
            alignment: None,
            transform: None,
            font_family: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_from_source(source: &str) -> HIRModule {
        let parse_result = crate::parser::parse(source);
        assert!(parse_result.errors.is_empty(), "Parse errors: {:?}", parse_result.errors);
        build_hir(parse_result.program.as_ref().unwrap())
    }

    #[test]
    fn test_build_minimal() {
        let hir = build_from_source("app Hello\n  screen Main\n    view\n      text \"Hi\"");
        assert_eq!(hir.app.name, "Hello");
        assert_eq!(hir.screens.len(), 1);
        assert_eq!(hir.screens[0].name, "Main");
    }

    #[test]
    fn test_build_model() {
        let hir = build_from_source("\
app Test
  model Todo
    title: text
    done: bool = false");
        assert_eq!(hir.models.len(), 1);
        assert_eq!(hir.models[0].name, "Todo");
        assert_eq!(hir.models[0].fields.len(), 2);
    }

    #[test]
    fn test_build_design_tokens() {
        let hir = build_from_source("\
app Test
  screen Main
    view
      column gap.md padding.lg
        text \"Hello\" .bold .accent");
        let view = &hir.screens[0].view;
        // Should be a Column with resolved design
        match view {
            HIRView::Column(layout) => {
                let spacing = layout.design.spacing.as_ref().unwrap();
                assert_eq!(spacing.gap, Some(8.0)); // md = 1x * 8 = 8
                assert_eq!(spacing.padding_top, Some(16.0)); // lg = 2x * 8 = 16
            }
            _ => panic!("Expected Column, got {:?}", std::mem::discriminant(view)),
        }
    }

    #[test]
    fn test_build_component() {
        let hir = build_from_source("\
app Test
  component Card(title: text)
    view
      text title");
        assert_eq!(hir.components.len(), 1);
        assert_eq!(hir.components[0].name, "Card");
        assert_eq!(hir.components[0].props.len(), 1);
    }

    #[test]
    fn test_build_action() {
        let hir = build_from_source("\
app Test
  screen Main
    state x: int = 0
    view
      text \"hi\"
    action increment
      x = x + 1");
        assert_eq!(hir.screens[0].actions.len(), 1);
        assert_eq!(hir.screens[0].actions[0].name, "increment");
    }

    #[test]
    fn test_build_with_theme() {
        let hir = build_from_source("\
app Test
  theme: modern.dark
  screen Main
    view
      text \"hi\"");
        assert_eq!(hir.app.theme, Some("modern.dark".to_string()));
    }

    #[test]
    fn test_size_display_token() {
        let source = "app T\n  screen M\n    view\n      text \"Hello\" size.display .bold";
        let result = crate::parser::parse(source);
        assert!(result.errors.is_empty(), "Parse errors: {:?}", result.errors);
        let hir = build_hir(result.program.as_ref().unwrap());
        match &hir.screens[0].view {
            HIRView::Text(t) => {
                let size = t.design.typography.as_ref().and_then(|ty| ty.size);
                let weight = t.design.typography.as_ref().and_then(|ty| ty.weight);
                assert_eq!(size, Some(3.0), "size.display should resolve to 3.0rem");
                assert_eq!(weight, Some(700), "bold should resolve to 700");
            }
            other => panic!("Expected Text, got: {:?}", std::mem::discriminant(other)),
        }
    }
}
