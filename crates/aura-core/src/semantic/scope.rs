//! Scope-based semantic analysis for Aura.
//!
//! Walks the AST, resolves names, checks types, enforces security rules,
//! validates state mutation, and implements error poisoning.

use std::collections::HashMap;

use crate::ast::*;
use crate::errors::{AuraError, ErrorCode, Fix, FixAction, Severity};
use crate::lexer::Span;
use crate::types::*;

/// Result of semantic analysis.
pub struct AnalysisResult {
    /// Errors and warnings found during analysis.
    pub errors: Vec<AuraError>,
    /// Resolved types for all declarations.
    pub symbols: SymbolTable,
}

/// A symbol in the symbol table.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub resolved_type: AuraType,
    pub span: Span,
    /// If true, downstream errors referencing this symbol are suppressed.
    pub poisoned: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// A model type definition.
    Model,
    /// A field in a model.
    Field,
    /// A screen.
    Screen,
    /// A component.
    Component,
    /// A state variable.
    State,
    /// A local variable (let binding or function param).
    Local,
    /// A constant.
    Constant,
    /// A function.
    Function,
    /// An action.
    Action,
    /// A loop variable (from `each ... as item`).
    LoopVar,
    /// A function/action parameter.
    Parameter,
}

/// Hierarchical symbol table with scoping.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            scopes: vec![Scope {
                name: "global".to_string(),
                symbols: HashMap::new(),
                parent: None,
            }],
        }
    }

    fn push_scope(&mut self, name: &str, parent: usize) -> usize {
        let id = self.scopes.len();
        self.scopes.push(Scope {
            name: name.to_string(),
            symbols: HashMap::new(),
            parent: Some(parent),
        });
        id
    }

    fn define(&mut self, scope: usize, symbol: Symbol) {
        self.scopes[scope]
            .symbols
            .insert(symbol.name.clone(), symbol);
    }

    pub fn lookup(&self, scope: usize, name: &str) -> Option<&Symbol> {
        let s = &self.scopes[scope];
        if let Some(sym) = s.symbols.get(name) {
            return Some(sym);
        }
        if let Some(parent) = s.parent {
            return self.lookup(parent, name);
        }
        None
    }

    /// Get all symbol names visible in a scope (for "did you mean?" suggestions).
    fn visible_names(&self, scope: usize) -> Vec<String> {
        let mut names = Vec::new();
        let mut current = Some(scope);
        while let Some(id) = current {
            let s = &self.scopes[id];
            names.extend(s.symbols.keys().cloned());
            current = s.parent;
        }
        names
    }
}

/// The context for what kind of block we're currently inside.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockContext {
    /// Top-level app body.
    App,
    /// Inside a screen.
    Screen,
    /// Inside a component.
    Component,
    /// Inside a view block (no mutations allowed).
    View,
    /// Inside an action block (mutations allowed).
    Action,
    /// Inside a pure function (no mutations).
    Function,
}

/// The semantic analyzer.
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<AuraError>,
    /// Names that are poisoned — suppress downstream errors referencing them.
    poisoned_names: HashMap<String, Span>,
    /// Current block context (for mutation checking).
    context: BlockContext,
    /// Current scope index.
    current_scope: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            poisoned_names: HashMap::new(),
            context: BlockContext::App,
            current_scope: 0,
        }
    }

    /// Analyze a parsed program.
    pub fn analyze(mut self, program: &Program) -> AnalysisResult {
        self.analyze_program(program);
        AnalysisResult {
            errors: self.errors,
            symbols: self.symbols,
        }
    }

    // === Error helpers ===

    fn error(&mut self, code: ErrorCode, message: String, span: Span) {
        self.errors
            .push(AuraError::new(code, Severity::Error, message, span));
    }

    fn warning(&mut self, code: ErrorCode, message: String, span: Span) {
        self.errors
            .push(AuraError::new(code, Severity::Warning, message, span));
    }

    fn error_with_fix(&mut self, code: ErrorCode, message: String, span: Span, fix: Fix) {
        self.errors
            .push(AuraError::new(code, Severity::Error, message, span).with_fix(fix));
    }

    /// Check if a name is poisoned (suppresses downstream errors).
    fn is_poisoned(&self, name: &str) -> bool {
        self.poisoned_names.contains_key(name)
    }

    /// Mark a name as poisoned.
    fn poison(&mut self, name: &str, span: Span) {
        self.poisoned_names.insert(name.to_string(), span);
    }

    // === Program analysis ===

    fn analyze_program(&mut self, program: &Program) {
        // Pass 1: Register all type names (models, screens, components) so they can
        // reference each other regardless of declaration order.
        for member in &program.app.members {
            match member {
                AppMember::Model(m) => {
                    self.symbols.define(
                        self.current_scope,
                        Symbol {
                            name: m.name.clone(),
                            kind: SymbolKind::Model,
                            resolved_type: AuraType::Named(m.name.clone()),
                            span: m.span,
                            poisoned: false,
                        },
                    );
                }
                AppMember::Screen(s) => {
                    self.symbols.define(
                        self.current_scope,
                        Symbol {
                            name: s.name.clone(),
                            kind: SymbolKind::Screen,
                            resolved_type: AuraType::Named(s.name.clone()),
                            span: s.span,
                            poisoned: false,
                        },
                    );
                }
                AppMember::Component(c) => {
                    self.symbols.define(
                        self.current_scope,
                        Symbol {
                            name: c.name.clone(),
                            kind: SymbolKind::Component,
                            resolved_type: AuraType::Named(c.name.clone()),
                            span: c.span,
                            poisoned: false,
                        },
                    );
                }
                AppMember::Const(c) => {
                    let ty = if let Some(ref te) = c.type_expr {
                        self.resolve_type_expr(te)
                    } else {
                        self.infer_expr_type(&c.value)
                    };
                    self.symbols.define(
                        self.current_scope,
                        Symbol {
                            name: c.name.clone(),
                            kind: SymbolKind::Constant,
                            resolved_type: ty,
                            span: c.span,
                            poisoned: false,
                        },
                    );
                }
                AppMember::State(s) => {
                    let ty = self.resolve_type_expr(&s.type_expr);
                    self.symbols.define(
                        self.current_scope,
                        Symbol {
                            name: s.name.clone(),
                            kind: SymbolKind::State,
                            resolved_type: ty,
                            span: s.span,
                            poisoned: false,
                        },
                    );
                }
                _ => {}
            }
        }

        // Pass 2: Analyze each member in detail.
        for member in &program.app.members {
            match member {
                AppMember::Model(m) => self.analyze_model(m),
                AppMember::Screen(s) => self.analyze_screen(s),
                AppMember::Component(c) => self.analyze_component(c),
                AppMember::Fn(f) => self.analyze_fn(f),
                _ => {}
            }
        }
    }

    // === Model analysis ===

    fn analyze_model(&mut self, model: &ModelDecl) {
        let scope = self.symbols.push_scope(&model.name, self.current_scope);

        let mut seen_fields: HashMap<String, Span> = HashMap::new();

        for field in &model.fields {
            // Check duplicate fields
            if let Some(prev_span) = seen_fields.get(&field.name) {
                self.error(
                    ErrorCode::E0105,
                    format!("Duplicate field '{}' in model '{}'", field.name, model.name),
                    field.span,
                );
                continue;
            }
            seen_fields.insert(field.name.clone(), field.span);

            let field_type = self.resolve_type_expr(&field.type_expr);

            // Check default value type matches field type
            if let Some(ref default) = field.default {
                let default_type = self.infer_expr_type(default);
                self.check_type_compat(&field_type, &default_type, field.span);
            }

            self.symbols.define(
                scope,
                Symbol {
                    name: field.name.clone(),
                    kind: SymbolKind::Field,
                    resolved_type: field_type,
                    span: field.span,
                    poisoned: false,
                },
            );
        }
    }

    // === Screen analysis ===

    fn analyze_screen(&mut self, screen: &ScreenDecl) {
        let scope = self.symbols.push_scope(&screen.name, self.current_scope);
        let prev_scope = self.current_scope;
        self.current_scope = scope;

        // Register parameters
        for param in &screen.params {
            let ty = self.resolve_type_expr(&param.type_expr);
            self.symbols.define(
                scope,
                Symbol {
                    name: param.name.clone(),
                    kind: SymbolKind::Parameter,
                    resolved_type: ty,
                    span: param.span,
                    poisoned: false,
                },
            );
        }

        // Analyze members
        for member in &screen.members {
            match member {
                ScreenMember::State(s) => self.analyze_state_decl(s),
                ScreenMember::View(v) => {
                    let prev = self.context;
                    self.context = BlockContext::View;
                    self.analyze_view(v);
                    self.context = prev;
                }
                ScreenMember::Action(a) => {
                    let prev = self.context;
                    self.context = BlockContext::Action;
                    self.analyze_action(a);
                    self.context = prev;
                }
                ScreenMember::Fn(f) => {
                    let prev = self.context;
                    self.context = BlockContext::Function;
                    self.analyze_fn(f);
                    self.context = prev;
                }
                _ => {}
            }
        }

        self.current_scope = prev_scope;
    }

    // === Component analysis ===

    fn analyze_component(&mut self, comp: &ComponentDecl) {
        let scope = self.symbols.push_scope(&comp.name, self.current_scope);
        let prev_scope = self.current_scope;
        self.current_scope = scope;

        // Register props
        for prop in &comp.props {
            let ty = self.resolve_type_expr(&prop.type_expr);
            self.symbols.define(
                scope,
                Symbol {
                    name: prop.name.clone(),
                    kind: SymbolKind::Parameter,
                    resolved_type: ty,
                    span: prop.span,
                    poisoned: false,
                },
            );
        }

        // Analyze members (same as screen)
        for member in &comp.members {
            match member {
                ScreenMember::State(s) => self.analyze_state_decl(s),
                ScreenMember::View(v) => {
                    let prev = self.context;
                    self.context = BlockContext::View;
                    self.analyze_view(v);
                    self.context = prev;
                }
                ScreenMember::Action(a) => {
                    let prev = self.context;
                    self.context = BlockContext::Action;
                    self.analyze_action(a);
                    self.context = prev;
                }
                ScreenMember::Fn(f) => {
                    let prev = self.context;
                    self.context = BlockContext::Function;
                    self.analyze_fn(f);
                    self.context = prev;
                }
                _ => {}
            }
        }

        self.current_scope = prev_scope;
    }

    // === State ===

    fn analyze_state_decl(&mut self, state: &StateDecl) {
        let ty = self.resolve_type_expr(&state.type_expr);

        if let Some(ref default) = state.default {
            let default_type = self.infer_expr_type(default);
            self.check_type_compat(&ty, &default_type, state.span);
        }

        self.symbols.define(
            self.current_scope,
            Symbol {
                name: state.name.clone(),
                kind: SymbolKind::State,
                resolved_type: ty,
                span: state.span,
                poisoned: false,
            },
        );
    }

    // === View analysis ===

    fn analyze_view(&mut self, view: &ViewDecl) {
        for elem in &view.body {
            self.analyze_view_element(elem);
        }
    }

    fn analyze_view_element(&mut self, elem: &ViewElement) {
        match elem {
            ViewElement::Layout(layout) => {
                self.validate_design_tokens(&layout.tokens);
                for child in &layout.children {
                    self.analyze_view_element(child);
                }
            }
            ViewElement::Widget(widget) => {
                self.validate_design_tokens(&widget.tokens);
                for arg in &widget.args {
                    let ty = self.infer_expr_type(arg);
                    // Security check: secret types in view text
                    self.check_no_secret_in_display(&ty, arg);
                }
            }
            ViewElement::Input(input) => {
                self.validate_design_tokens(&input.tokens);
                // Verify binding references a state variable
                let binding_name = input.binding.split('.').next().unwrap_or(&input.binding);
                if let Some(sym) = self.symbols.lookup(self.current_scope, binding_name) {
                    if sym.kind != SymbolKind::State && sym.kind != SymbolKind::Parameter {
                        self.warning(
                            ErrorCode::E0310,
                            format!(
                                "'{}' is not a state variable — input binding may not update",
                                binding_name
                            ),
                            input.span,
                        );
                    }
                }
            }
            ViewElement::Button(button) => {
                self.validate_design_tokens(&button.tokens);
                let ty = self.infer_expr_type(&button.label);
                self.check_no_secret_in_display(&ty, &button.label);
            }
            ViewElement::If(if_view) => {
                let cond_type = self.infer_expr_type(&if_view.condition);
                self.check_type_compat(
                    &AuraType::Primitive(PrimitiveType::Bool),
                    &cond_type,
                    if_view.span,
                );
                // TYPE NARROWING for view conditionals
                let narrow_scope = self
                    .symbols
                    .push_scope("view_if_narrow", self.current_scope);
                self.apply_narrowing(&if_view.condition, narrow_scope);
                let prev = self.current_scope;
                self.current_scope = narrow_scope;
                for child in &if_view.then_body {
                    self.analyze_view_element(child);
                }
                self.current_scope = prev;
                if let Some(ref else_body) = if_view.else_body {
                    for child in else_body {
                        self.analyze_view_element(child);
                    }
                }
            }
            ViewElement::Each(each) => {
                let iter_type = self.infer_expr_type(&each.iterable);
                // Extract item type from list
                let item_type = match &iter_type {
                    AuraType::List(inner) => *inner.clone(),
                    AuraType::Poison => AuraType::Poison,
                    _ => {
                        self.error(
                            ErrorCode::E0101,
                            format!("'each' requires a list, got {}", iter_type.display_name()),
                            each.span,
                        );
                        AuraType::Poison
                    }
                };

                // Register loop variable in a new scope
                let loop_scope = self
                    .symbols
                    .push_scope(&format!("each_{}", each.item_name), self.current_scope);
                self.symbols.define(
                    loop_scope,
                    Symbol {
                        name: each.item_name.clone(),
                        kind: SymbolKind::LoopVar,
                        resolved_type: item_type,
                        span: each.span,
                        poisoned: false,
                    },
                );

                let prev = self.current_scope;
                self.current_scope = loop_scope;
                for child in &each.body {
                    self.analyze_view_element(child);
                }
                self.current_scope = prev;
            }
            ViewElement::When(when) => {
                self.infer_expr_type(&when.expression);
                for branch in &when.branches {
                    for child in &branch.body {
                        self.analyze_view_element(child);
                    }
                }
            }
            ViewElement::ComponentRef(comp_ref) => {
                // Verify component exists
                if self
                    .symbols
                    .lookup(self.current_scope, &comp_ref.name)
                    .is_none()
                {
                    if !self.is_poisoned(&comp_ref.name) {
                        self.error(
                            ErrorCode::E0104,
                            format!("Unknown component '{}'", comp_ref.name),
                            comp_ref.span,
                        );
                    }
                }
                for child in &comp_ref.children {
                    self.analyze_view_element(child);
                }
            }
            ViewElement::Divider(tokens, _) => {
                self.validate_design_tokens(tokens);
            }
            ViewElement::Spacer(_) | ViewElement::Slot(_) => {}
        }
    }

    // === Action analysis ===

    fn analyze_action(&mut self, action: &ActionDecl) {
        let scope = self
            .symbols
            .push_scope(&format!("action_{}", action.name), self.current_scope);

        // Register as action in parent scope
        let param_types: Vec<AuraType> = action
            .params
            .iter()
            .map(|p| self.resolve_type_expr(&p.type_expr))
            .collect();

        self.symbols.define(
            self.current_scope,
            Symbol {
                name: action.name.clone(),
                kind: SymbolKind::Action,
                resolved_type: AuraType::Action(param_types),
                span: action.span,
                poisoned: false,
            },
        );

        // Register params in action scope
        for param in &action.params {
            let ty = self.resolve_type_expr(&param.type_expr);
            self.symbols.define(
                scope,
                Symbol {
                    name: param.name.clone(),
                    kind: SymbolKind::Parameter,
                    resolved_type: ty,
                    span: param.span,
                    poisoned: false,
                },
            );
        }

        let prev = self.current_scope;
        self.current_scope = scope;
        self.analyze_statements(&action.body);
        self.current_scope = prev;
    }

    // === Function analysis ===

    fn analyze_fn(&mut self, func: &FnDecl) {
        let scope = self
            .symbols
            .push_scope(&format!("fn_{}", func.name), self.current_scope);

        let param_types: Vec<AuraType> = func
            .params
            .iter()
            .map(|p| self.resolve_type_expr(&p.type_expr))
            .collect();
        let return_type = func
            .return_type
            .as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or(AuraType::Primitive(PrimitiveType::Text)); // default inferred later

        self.symbols.define(
            self.current_scope,
            Symbol {
                name: func.name.clone(),
                kind: SymbolKind::Function,
                resolved_type: AuraType::Function(FunctionType {
                    type_params: Vec::new(),
                    params: param_types,
                    return_type: Box::new(return_type),
                }),
                span: func.span,
                poisoned: false,
            },
        );

        for param in &func.params {
            let ty = self.resolve_type_expr(&param.type_expr);
            self.symbols.define(
                scope,
                Symbol {
                    name: param.name.clone(),
                    kind: SymbolKind::Parameter,
                    resolved_type: ty,
                    span: param.span,
                    poisoned: false,
                },
            );
        }

        let prev = self.current_scope;
        let prev_ctx = self.context;
        self.current_scope = scope;
        self.context = BlockContext::Function;
        self.analyze_statements(&func.body);
        self.context = prev_ctx;
        self.current_scope = prev;
    }

    // === Statement analysis ===

    fn analyze_statements(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.analyze_statement(stmt);
        }
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign(name, value, span) => {
                // Check mutation context
                if self.context == BlockContext::View {
                    self.error(
                        ErrorCode::E0301,
                        format!("State mutation in view block is not allowed. Move '{}' assignment to an action.", name),
                        *span,
                    );
                } else if self.context == BlockContext::Function {
                    // Check if it's a state variable being mutated
                    if let Some(sym) = self.symbols.lookup(self.current_scope, name) {
                        if sym.kind == SymbolKind::State {
                            self.error(
                                ErrorCode::E0302,
                                format!("Cannot mutate state '{}' in a pure function. Use an action instead.", name),
                                *span,
                            );
                        }
                    }
                }

                let value_type = self.infer_expr_type(value);

                // Verify target exists
                if let Some(sym) = self.symbols.lookup(self.current_scope, name) {
                    let expected = sym.resolved_type.clone();
                    self.check_type_compat(&expected, &value_type, *span);
                } else if !self.is_poisoned(name) {
                    self.undefined_variable_error(name, *span);
                }
            }
            Stmt::Let(name, type_expr, value, span) => {
                let declared_type = type_expr.as_ref().map(|t| self.resolve_type_expr(t));
                let value_type = self.infer_expr_type(value);

                let resolved = if let Some(ref dt) = declared_type {
                    self.check_type_compat(dt, &value_type, *span);
                    dt.clone()
                } else {
                    value_type
                };

                self.symbols.define(
                    self.current_scope,
                    Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Local,
                        resolved_type: resolved,
                        span: *span,
                        poisoned: false,
                    },
                );
            }
            Stmt::If(cond, then_body, else_body, span) => {
                let cond_type = self.infer_expr_type(cond);
                self.check_type_compat(
                    &AuraType::Primitive(PrimitiveType::Bool),
                    &cond_type,
                    *span,
                );

                // TYPE NARROWING: analyze condition for type refinements
                // Pattern: if x is some(val) → register val in then-branch scope
                // Pattern: if x != nil → narrow x to non-optional in then-branch
                let narrowing_scope = self.symbols.push_scope("if_narrow", self.current_scope);
                self.apply_narrowing(cond, narrowing_scope);

                let prev_scope = self.current_scope;
                self.current_scope = narrowing_scope;
                self.analyze_statements(then_body);
                self.current_scope = prev_scope;

                if let Some(else_stmts) = else_body {
                    self.analyze_statements(else_stmts);
                }
            }
            Stmt::When(expr, branches, _span) => {
                self.infer_expr_type(expr);
                for branch in branches {
                    match &branch.body {
                        StmtOrExpr::Stmt(stmts) => self.analyze_statements(stmts),
                        StmtOrExpr::Expr(e) => {
                            self.infer_expr_type(e);
                        }
                    }
                }
            }
            Stmt::Return(value, _span) => {
                if let Some(expr) = value {
                    self.infer_expr_type(expr);
                }
            }
            Stmt::Expr(expr, _span) => {
                self.infer_expr_type(expr);
            }
            Stmt::Navigate(_) | Stmt::Emit(_, _, _) => {}
        }
    }

    // === Type resolution ===

    fn resolve_type_expr(&mut self, type_expr: &TypeExpr) -> AuraType {
        match type_expr {
            TypeExpr::Named(name, span) => self.resolve_type_name(name, *span),
            TypeExpr::Collection(kind, args, _span) => {
                let resolved_args: Vec<_> =
                    args.iter().map(|a| self.resolve_type_expr(a)).collect();
                match kind.as_str() {
                    "list" => AuraType::List(Box::new(
                        resolved_args.into_iter().next().unwrap_or(AuraType::Poison),
                    )),
                    "set" => AuraType::Set(Box::new(
                        resolved_args.into_iter().next().unwrap_or(AuraType::Poison),
                    )),
                    "map" => {
                        let mut iter = resolved_args.into_iter();
                        let k = iter.next().unwrap_or(AuraType::Poison);
                        let v = iter.next().unwrap_or(AuraType::Poison);
                        AuraType::Map(Box::new(k), Box::new(v))
                    }
                    _ => AuraType::Poison,
                }
            }
            TypeExpr::Optional(inner, _span) => {
                AuraType::Optional(Box::new(self.resolve_type_expr(inner)))
            }
            TypeExpr::Enum(variants, _span) => {
                let resolved: Vec<_> = variants
                    .iter()
                    .map(|v| EnumVariant {
                        name: v.name.clone(),
                        fields: v
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), self.resolve_type_expr(&f.type_expr)))
                            .collect(),
                    })
                    .collect();
                AuraType::Enum(resolved)
            }
            TypeExpr::Function(params, ret, _span) => {
                let param_types: Vec<_> =
                    params.iter().map(|p| self.resolve_type_expr(p)).collect();
                let return_type = ret
                    .as_ref()
                    .map(|r| self.resolve_type_expr(r))
                    .unwrap_or(AuraType::Primitive(PrimitiveType::Text));
                AuraType::Function(FunctionType {
                    type_params: Vec::new(),
                    params: param_types,
                    return_type: Box::new(return_type),
                })
            }
            TypeExpr::Action(params, _span) => {
                let param_types: Vec<_> =
                    params.iter().map(|p| self.resolve_type_expr(p)).collect();
                AuraType::Action(param_types)
            }
        }
    }

    fn resolve_type_name(&self, name: &str, _span: Span) -> AuraType {
        match name {
            "text" => AuraType::Primitive(PrimitiveType::Text),
            "int" => AuraType::Primitive(PrimitiveType::Int),
            "float" => AuraType::Primitive(PrimitiveType::Float),
            "bool" => AuraType::Primitive(PrimitiveType::Bool),
            "timestamp" => AuraType::Primitive(PrimitiveType::Timestamp),
            "duration" => AuraType::Primitive(PrimitiveType::Duration),
            "percent" => AuraType::Primitive(PrimitiveType::Percent),
            "secret" => AuraType::Security(SecurityType::Secret),
            "sanitized" => AuraType::Security(SecurityType::Sanitized),
            "email" => AuraType::Security(SecurityType::Email),
            "url" => AuraType::Security(SecurityType::Url),
            "token" => AuraType::Security(SecurityType::Token),
            _ => AuraType::Named(name.to_string()),
        }
    }

    // === Type inference ===

    fn infer_expr_type(&mut self, expr: &Expr) -> AuraType {
        match expr {
            Expr::IntLit(_, _) => AuraType::Primitive(PrimitiveType::Int),
            Expr::FloatLit(_, _) => AuraType::Primitive(PrimitiveType::Float),
            Expr::StringLit(_, _) => AuraType::Primitive(PrimitiveType::Text),
            Expr::PercentLit(_, _) => AuraType::Primitive(PrimitiveType::Percent),
            Expr::BoolLit(_, _) => AuraType::Primitive(PrimitiveType::Bool),
            Expr::Nil(_) => AuraType::Optional(Box::new(AuraType::Poison)),
            Expr::Var(name, span) => {
                if let Some(sym) = self.symbols.lookup(self.current_scope, name) {
                    if sym.poisoned {
                        return AuraType::Poison;
                    }
                    sym.resolved_type.clone()
                } else if self.is_poisoned(name) {
                    AuraType::Poison
                } else {
                    self.undefined_variable_error(name, *span);
                    AuraType::Poison
                }
            }
            Expr::MemberAccess(obj, member, _span) => {
                let obj_type = self.infer_expr_type(obj);
                if obj_type.is_poison() {
                    return AuraType::Poison;
                }
                // Look up member on the resolved type
                match &obj_type {
                    AuraType::Named(type_name) => {
                        // Search model fields in the symbol table
                        if let Some(model_scope) =
                            self.symbols.scopes.iter().find(|s| s.name == *type_name)
                        {
                            if let Some(field) = model_scope.symbols.get(member.as_str()) {
                                return field.resolved_type.clone();
                            }
                        }
                        // Common built-in members
                        match member.as_str() {
                            "count" => AuraType::Primitive(PrimitiveType::Int),
                            "isEmpty" => AuraType::Primitive(PrimitiveType::Bool),
                            "toText" | "toText()" => AuraType::Primitive(PrimitiveType::Text),
                            _ => AuraType::Poison, // Unknown field
                        }
                    }
                    AuraType::List(inner) => match member.as_str() {
                        "count" | "count()" => AuraType::Primitive(PrimitiveType::Int),
                        "isEmpty" => AuraType::Primitive(PrimitiveType::Bool),
                        "first" | "last" => AuraType::Optional(inner.clone()),
                        _ => AuraType::Poison,
                    },
                    AuraType::Primitive(PrimitiveType::Text) => match member.as_str() {
                        "count" | "count()" => AuraType::Primitive(PrimitiveType::Int),
                        "isEmpty" => AuraType::Primitive(PrimitiveType::Bool),
                        "trim" | "uppercase" | "lowercase" => {
                            AuraType::Primitive(PrimitiveType::Text)
                        }
                        _ => AuraType::Poison,
                    },
                    AuraType::Primitive(PrimitiveType::Int)
                    | AuraType::Primitive(PrimitiveType::Float) => match member.as_str() {
                        "toFloat" => AuraType::Primitive(PrimitiveType::Float),
                        "toInt" => AuraType::Primitive(PrimitiveType::Int),
                        "toText" => AuraType::Primitive(PrimitiveType::Text),
                        "abs" => obj_type.clone(),
                        _ => AuraType::Poison,
                    },
                    AuraType::Optional(inner) => {
                        // Accessing members on optional — pass through to inner type
                        AuraType::Poison // Should use nil-check first
                    }
                    _ => AuraType::Poison,
                }
            }
            Expr::Call(func, args, _span) => {
                // BIDIRECTIONAL INFERENCE: for method calls on collections,
                // infer lambda parameter types from the collection's element type.
                // e.g., todos.where(t => not t.done) → t inferred as Todo
                if let Expr::MemberAccess(obj, method, _) = func.as_ref() {
                    let obj_type = self.infer_expr_type(obj);
                    match (&obj_type, method.as_str()) {
                        (
                            AuraType::List(elem_type),
                            "where" | "filter" | "find" | "map" | "forEach",
                        ) => {
                            // If the first arg is a lambda, register its param with elem_type
                            if let Some(Expr::Lambda(params, body, _)) = args.first() {
                                if let Some(param) = params.first() {
                                    let lambda_scope =
                                        self.symbols.push_scope("lambda", self.current_scope);
                                    self.symbols.define(
                                        lambda_scope,
                                        Symbol {
                                            name: param.name.clone(),
                                            kind: SymbolKind::Parameter,
                                            resolved_type: *elem_type.clone(),
                                            span: param.span,
                                            poisoned: false,
                                        },
                                    );
                                    let prev = self.current_scope;
                                    self.current_scope = lambda_scope;
                                    let body_type = self.infer_expr_type(body);
                                    self.current_scope = prev;

                                    // Return type depends on method
                                    return match method.as_str() {
                                        "where" | "filter" => obj_type, // list[T] → list[T]
                                        "find" => AuraType::Optional(elem_type.clone()),
                                        "map" => AuraType::List(Box::new(body_type)),
                                        _ => AuraType::Poison,
                                    };
                                }
                            }
                            // Non-lambda args — return type based on method
                            return match method.as_str() {
                                "where" | "filter" => obj_type,
                                "find" => AuraType::Optional(elem_type.clone()),
                                "count" => AuraType::Primitive(PrimitiveType::Int),
                                _ => AuraType::Poison,
                            };
                        }
                        _ => {}
                    }
                }

                let func_type = self.infer_expr_type(func);
                match func_type {
                    AuraType::Function(ft) => *ft.return_type,
                    AuraType::Poison => AuraType::Poison,
                    _ => AuraType::Poison,
                }
            }
            Expr::NamedCall(func, _args, _span) => {
                let func_type = self.infer_expr_type(func);
                match func_type {
                    AuraType::Function(ft) => *ft.return_type,
                    // Constructor call: Todo(title: "x") -> Todo
                    AuraType::Named(name) => AuraType::Named(name),
                    AuraType::Poison => AuraType::Poison,
                    _ => AuraType::Poison,
                }
            }
            Expr::Constructor(name, _args, _span) => AuraType::Named(name.clone()),
            Expr::BinOp(left, op, right, span) => {
                let lt = self.infer_expr_type(left);
                let rt = self.infer_expr_type(right);
                if lt.is_poison() || rt.is_poison() {
                    return AuraType::Poison;
                }
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // Numeric ops return the operand type
                        lt
                    }
                    BinOp::Eq
                    | BinOp::NotEq
                    | BinOp::Lt
                    | BinOp::Gt
                    | BinOp::LtEq
                    | BinOp::GtEq
                    | BinOp::And
                    | BinOp::Or => {
                        // Comparison/logical ops: check for secret == comparison
                        if matches!(op, BinOp::Eq | BinOp::NotEq) {
                            if lt.is_no_log() {
                                self.error(
                                    ErrorCode::E0203,
                                    format!(
                                        "Cannot compare {} with ==. Use .verify() instead.",
                                        lt.display_name()
                                    ),
                                    *span,
                                );
                            }
                        }
                        AuraType::Primitive(PrimitiveType::Bool)
                    }
                    BinOp::Range => AuraType::Poison, // TODO: range type
                }
            }
            Expr::UnaryOp(op, operand, _span) => {
                let t = self.infer_expr_type(operand);
                match op {
                    UnaryOp::Not => AuraType::Primitive(PrimitiveType::Bool),
                    UnaryOp::Neg => t,
                }
            }
            Expr::Lambda(_, body, _) => self.infer_expr_type(body),
            Expr::Pipe(left, _right, _span) => {
                self.infer_expr_type(left);
                // Pipe result is the right side's return type — simplified
                AuraType::Poison
            }
            Expr::Conditional(_, then_expr, _else_expr, _span) => self.infer_expr_type(then_expr),
            Expr::NilCoalesce(left, right, _span) => {
                let lt = self.infer_expr_type(left);
                let _rt = self.infer_expr_type(right);
                // Unwrap optional
                match lt {
                    AuraType::Optional(inner) => *inner,
                    _ => lt,
                }
            }
            Expr::Index(obj, _idx, _span) => {
                let obj_type = self.infer_expr_type(obj);
                match obj_type {
                    AuraType::List(inner) => *inner,
                    AuraType::Map(_, v) => AuraType::Optional(v),
                    _ => AuraType::Poison,
                }
            }
            Expr::DesignToken(_, _) => AuraType::Poison, // Design tokens are not expressions with types
        }
    }

    // === Security type checks ===

    /// E0202: Secret value in string interpolation / display context.
    fn check_no_secret_in_display(&mut self, ty: &AuraType, expr: &Expr) {
        if ty.is_no_log() {
            let span = self.expr_span(expr);
            self.error(
                ErrorCode::E0202,
                format!(
                    "{} value cannot be displayed. Secret and token types are not renderable.",
                    ty.display_name()
                ),
                span,
            );
        }
    }

    // === Type compatibility ===

    fn check_type_compat(&mut self, expected: &AuraType, actual: &AuraType, span: Span) {
        if self.types_compatible(expected, actual) {
            return;
        }

        self.error(
            ErrorCode::E0101,
            format!(
                "Type mismatch: expected {}, found {}",
                expected.display_name(),
                actual.display_name()
            ),
            span,
        );
    }

    // === Design token validation ===

    fn validate_design_tokens(&mut self, tokens: &[DesignToken]) {
        for token in tokens {
            let first = token.segments.first().map(|s| s.as_str()).unwrap_or("");
            if !is_valid_design_token(first) && !is_compound_design_token(&token.segments) {
                self.error_with_fix(
                    ErrorCode::E0400,
                    format!("Unknown design token '.{}'", token.segments.join(".")),
                    token.span,
                    Fix {
                        action: FixAction::Replace,
                        span: token.span,
                        replacement: suggest_design_token(first),
                        confidence: 0.5,
                    },
                );
            }
        }
    }

    // === Error helpers ===

    /// Check if two types are compatible (structural typing + union support).
    ///
    /// Like TypeScript: structural compatibility, not nominal.
    /// - Poison is compatible with anything (error recovery)
    /// - Same type is always compatible
    /// - Named types with same name are compatible
    /// - Collections check inner type recursively
    /// - Optional[T] accepts T and nil
    /// - Union[A, B] accepts A or B individually
    /// - Named types are lenient (defer to runtime) during early development
    fn types_compatible(&self, expected: &AuraType, actual: &AuraType) -> bool {
        // Poison is compatible with anything
        if expected.is_poison() || actual.is_poison() {
            return true;
        }

        // Same type
        if expected == actual {
            return true;
        }

        match (expected, actual) {
            // Named types with same name
            (AuraType::Named(a), AuraType::Named(b)) if a == b => true,

            // Collections — check inner types
            (AuraType::List(a), AuraType::List(b)) => self.types_compatible(a, b),
            (AuraType::Set(a), AuraType::Set(b)) => self.types_compatible(a, b),
            (AuraType::Map(ak, av), AuraType::Map(bk, bv)) => {
                self.types_compatible(ak, bk) && self.types_compatible(av, bv)
            }

            // Optional — T is assignable to Optional[T], nil is assignable to any Optional
            (AuraType::Optional(inner), other) => {
                self.types_compatible(inner, other) || other.is_poison()
            }
            (_, AuraType::Optional(inner)) if inner.is_poison() => true, // nil literal

            // UNION TYPES: A | B accepts A or B
            (AuraType::Union(variants), actual) => {
                variants.iter().any(|v| self.types_compatible(v, actual))
            }
            (expected, AuraType::Union(variants)) => {
                variants.iter().any(|v| self.types_compatible(expected, v))
            }

            // STRUCTURAL TYPING: Named types are lenient
            // (full structural check would require model field lookup across scopes)
            (AuraType::Named(_), _) | (_, AuraType::Named(_)) => true,

            // Function type compatibility
            (AuraType::Function(a), AuraType::Function(b)) => {
                a.params.len() == b.params.len()
                    && a.params
                        .iter()
                        .zip(&b.params)
                        .all(|(ap, bp)| self.types_compatible(ap, bp))
                    && self.types_compatible(&a.return_type, &b.return_type)
            }

            // Action compatibility
            (AuraType::Action(a), AuraType::Action(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b)
                        .all(|(ap, bp)| self.types_compatible(ap, bp))
            }

            // Enum compatibility (same variants)
            (AuraType::Enum(a), AuraType::Enum(b)) => {
                a.iter().all(|av| b.iter().any(|bv| bv.name == av.name))
            }

            _ => false,
        }
    }

    /// Apply type narrowing based on a condition expression.
    ///
    /// TypeScript-style flow analysis: if the condition tests a type,
    /// register the narrowed type in the given scope.
    fn apply_narrowing(&mut self, cond: &Expr, scope: usize) {
        match cond {
            // Pattern: x is some(val) → register val with unwrapped type
            Expr::BinOp(left, BinOp::Eq, right, _) | Expr::BinOp(left, BinOp::NotEq, right, _) => {
                // x != nil → narrow x to non-optional
                if matches!(right.as_ref(), Expr::Nil(_)) {
                    if let Expr::Var(name, span) = left.as_ref() {
                        if let Some(sym) = self.symbols.lookup(self.current_scope, name) {
                            if let AuraType::Optional(inner) = &sym.resolved_type {
                                // Narrow: optional[T] → T in the then-branch
                                self.symbols.define(
                                    scope,
                                    Symbol {
                                        name: name.clone(),
                                        kind: sym.kind.clone(),
                                        resolved_type: *inner.clone(),
                                        span: *span,
                                        poisoned: false,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            // Pattern: not x.isEmpty → x has items (informational, not type-level)
            Expr::UnaryOp(UnaryOp::Not, inner, _) => {
                // Recurse into the negated expression for future narrowing patterns
                let _ = inner;
            }
            _ => {}
        }
    }

    fn undefined_variable_error(&mut self, name: &str, span: Span) {
        let visible = self.symbols.visible_names(self.current_scope);
        let visible_refs: Vec<&str> = visible.iter().map(|s| s.as_str()).collect();

        let (message, fix) = if let Some((suggestion, confidence)) =
            crate::errors::suggest_similar(name, &visible_refs, 3)
        {
            (
                format!(
                    "Unknown variable '{}'. Did you mean '{}'?",
                    name, suggestion
                ),
                Some(Fix {
                    action: FixAction::Replace,
                    span,
                    replacement: suggestion.to_string(),
                    confidence,
                }),
            )
        } else {
            (format!("Unknown variable '{}'", name), None)
        };

        let mut err = AuraError::new(ErrorCode::E0103, Severity::Error, message, span);
        if let Some(f) = fix {
            err = err.with_fix(f);
        }
        self.errors.push(err);

        // Poison this name to suppress downstream errors
        self.poison(name, span);
    }

    fn expr_span(&self, expr: &Expr) -> Span {
        match expr {
            Expr::IntLit(_, s)
            | Expr::FloatLit(_, s)
            | Expr::StringLit(_, s)
            | Expr::PercentLit(_, s)
            | Expr::BoolLit(_, s)
            | Expr::Nil(s)
            | Expr::Var(_, s) => *s,
            Expr::MemberAccess(_, _, s)
            | Expr::Call(_, _, s)
            | Expr::NamedCall(_, _, s)
            | Expr::Index(_, _, s)
            | Expr::BinOp(_, _, _, s)
            | Expr::UnaryOp(_, _, s)
            | Expr::Lambda(_, _, s)
            | Expr::Constructor(_, _, s)
            | Expr::Pipe(_, _, s)
            | Expr::Conditional(_, _, _, s)
            | Expr::NilCoalesce(_, _, s)
            | Expr::DesignToken(_, s) => *s,
        }
    }
}

// === Design token validation helpers ===

fn is_valid_design_token(name: &str) -> bool {
    matches!(
        name,
        // Spacing
        "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl"
        // Typography
        | "display" | "bold" | "medium" | "semibold" | "heavy" | "light" | "thin"
        | "regular" | "black" | "italic" | "mono" | "underline" | "strike"
        | "center" | "leading" | "trailing" | "uppercase" | "lowercase" | "capitalize"
        // Color
        | "primary" | "secondary" | "muted" | "accent" | "danger" | "warning"
        | "success" | "info" | "surface" | "background"
        // Shape
        | "sharp" | "subtle" | "rounded" | "smooth" | "pill" | "circle"
        // Motion
        | "ease" | "spring" | "bounce" | "instant" | "fast" | "normal" | "slow"
        // Sizing
        | "fill" | "fit"
        // State
        | "indeterminate" | "disabled" | "loading" | "selected"
        // Other
        | "translucent"
    )
}

fn is_compound_design_token(segments: &[String]) -> bool {
    if segments.len() < 2 {
        return false;
    }
    let first = segments[0].as_str();
    matches!(
        first,
        "gap"
            | "padding"
            | "margin"
            | "size"
            | "width"
            | "height"
            | "radius"
            | "shadow"
            | "elevation"
            | "opacity"
            | "align"
            | "justify"
            | "max"
            | "min"
            | "onPrimary"
            | "onAccent"
            | "onDanger"
            | "onSurface"
            | "horizontal"
            | "vertical"
            | "top"
            | "bottom"
            | "left"
            | "right"
            | "start"
            | "end"
    )
}

fn suggest_design_token(name: &str) -> String {
    // Simple Levenshtein-based suggestion
    let candidates = [
        "xs",
        "sm",
        "md",
        "lg",
        "xl",
        "bold",
        "accent",
        "primary",
        "secondary",
        "muted",
        "danger",
        "rounded",
        "surface",
        "center",
    ];
    let refs: Vec<&str> = candidates.to_vec();
    if let Some((suggestion, _)) = crate::errors::suggest_similar(name, &refs, 3) {
        format!(".{}", suggestion)
    } else {
        format!(".{}", name)
    }
}

/// Run semantic analysis on parsed source code.
pub fn analyze(source: &str) -> AnalysisResult {
    let parse_result = crate::parser::parse(source);

    if let Some(ref program) = parse_result.program {
        let mut result = SemanticAnalyzer::new().analyze(program);
        // Prepend any parse errors
        let mut all_errors = parse_result.errors;
        all_errors.append(&mut result.errors);
        result.errors = all_errors;
        result
    } else {
        AnalysisResult {
            errors: parse_result.errors,
            symbols: SymbolTable::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze_and_check(source: &str) -> Vec<(ErrorCode, String)> {
        let result = analyze(source);
        result
            .errors
            .iter()
            .map(|e| (e.code, e.message.clone()))
            .collect()
    }

    fn has_error(errors: &[(ErrorCode, String)], code: ErrorCode) -> bool {
        errors.iter().any(|(c, _)| *c == code)
    }

    fn error_count(errors: &[(ErrorCode, String)]) -> usize {
        errors.len()
    }

    #[test]
    fn test_clean_program() {
        let errors = analyze_and_check(
            "\
app Test
  model Todo
    title: text
    done: bool = false
  screen Main
    state todos: list[Todo] = []
    view
      text \"Hello\"",
        );
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_duplicate_field() {
        let errors = analyze_and_check(
            "\
app Test
  model Bad
    name: text
    name: int",
        );
        assert!(has_error(&errors, ErrorCode::E0105));
    }

    #[test]
    fn test_state_mutation_in_view() {
        let errors = analyze_and_check(
            "\
app Test
  screen Main
    state x: int = 0
    view
      text \"hi\"
    action doStuff
      x = x + 1",
        );
        // The action mutation is fine, no E0301
        assert!(!has_error(&errors, ErrorCode::E0301));
    }

    #[test]
    fn test_secret_type_in_equality() {
        // Test at the action level where params are in scope
        let errors = analyze_and_check(
            "\
app Test
  screen Main
    view
      text \"hi\"
    action check(a: secret, b: secret)
      let result = a == b",
        );
        assert!(
            has_error(&errors, ErrorCode::E0203),
            "Expected E0203 for secret == comparison. Errors: {:?}",
            errors
        );
    }

    #[test]
    fn test_unknown_variable_with_suggestion() {
        let result = analyze(
            "\
app Test
  screen Main
    state todos: list[text] = []
    view
      text \"hi\"
    action test
      todoos = []",
        );
        let err = result.errors.iter().find(|e| e.code == ErrorCode::E0103);
        assert!(err.is_some(), "Expected E0103 error");
        let err = err.unwrap();
        assert!(
            err.message.contains("todoos"),
            "Error should mention the typo"
        );
        // Should have a fix suggestion
        assert!(err.fix.is_some(), "Should suggest a fix");
        let fix = err.fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "todos");
        assert!(fix.confidence > 0.7);
    }

    #[test]
    fn test_error_poisoning() {
        let result = analyze(
            "\
app Test
  screen Main
    state items: list[text] = []
    view
      text \"hi\"
    action test
      let x = unknownVar
      let y = x + 1
      let z = y + 2",
        );
        // Only ONE error for unknownVar, not cascading errors for x and y
        let e103_count = result
            .errors
            .iter()
            .filter(|e| e.code == ErrorCode::E0103)
            .count();
        assert_eq!(
            e103_count, 1,
            "Poisoning should suppress cascade errors. Got {} E0103 errors",
            e103_count
        );
    }

    #[test]
    fn test_design_token_validation() {
        let errors = analyze_and_check(
            "\
app Test
  screen Main
    view
      column .md .accent
        text \"hi\" .bold",
        );
        // All valid tokens — no errors
        assert!(
            !has_error(&errors, ErrorCode::E0400),
            "Errors: {:?}",
            errors
        );
    }

    #[test]
    fn test_invalid_design_token() {
        let errors = analyze_and_check(
            "\
app Test
  screen Main
    view
      column .xxxlarge
        text \"hi\"",
        );
        assert!(has_error(&errors, ErrorCode::E0400));
    }

    #[test]
    fn test_each_registers_loop_var() {
        let errors = analyze_and_check(
            "\
app Test
  screen Main
    state items: list[text] = []
    view
      each items as item
        text item",
        );
        // item should be resolvable — no E0103
        assert!(
            !has_error(&errors, ErrorCode::E0103),
            "Errors: {:?}",
            errors
        );
    }

    #[test]
    fn test_fn_and_action_registered() {
        let errors = analyze_and_check(
            "\
app Test
  screen Main
    state x: int = 0
    view
      button \"Go\" .accent -> increment()
    action increment
      x = x + 1
    fn double(n: int) -> int
      n * 2",
        );
        // Should be clean
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_component_props_in_scope() {
        let errors = analyze_and_check(
            "\
app Test
  component Card(title: text, count: int)
    view
      text title
  screen Main
    view
      Card(title: \"hello\", count: 5)",
        );
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }
}
