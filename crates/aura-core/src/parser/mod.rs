//! # Aura Parser
//!
//! Parses a token stream into an AST. Uses a hand-written recursive descent
//! parser (not chumsky) for better error recovery and clearer error messages.
//!
//! ## Design Decisions
//! - Hand-written recursive descent instead of chumsky combinators.
//!   Reason: chumsky's error recovery is generic; we need Aura-specific
//!   recovery (skip to next declaration on error, produce partial AST).
//! - The parser consumes `Vec<Spanned<Token>>` and produces `ast::Program`.
//! - All errors collected — does not stop at first error.
//! - Indent/Dedent tokens (from lexer) define block structure.

use crate::ast::*;
use crate::lexer::{Span, Spanned, Token};

/// Parse result containing the AST and any errors.
pub struct ParseResult {
    pub program: Option<Program>,
    pub errors: Vec<crate::errors::AuraError>,
}

/// The parser state.
struct Parser {
    tokens: Vec<Spanned<Token>>,
    pos: usize,
    errors: Vec<crate::errors::AuraError>,
}

impl Parser {
    fn new(tokens: Vec<Spanned<Token>>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    // === Token navigation ===

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.value)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or(Span::new(0, 0))
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn advance(&mut self) -> Option<&Spanned<Token>> {
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn check(&self, token: &Token) -> bool {
        self.peek() == Some(token)
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: &Token) -> Result<Span, ()> {
        if self.check(token) {
            let span = self.peek_span();
            self.advance();
            Ok(span)
        } else {
            let span = self.peek_span();
            self.error(
                crate::errors::ErrorCode::E0700,
                format!("Expected {}, found {}", token, self.peek_desc()),
                span,
            );
            Err(())
        }
    }

    fn peek_desc(&self) -> String {
        match self.peek() {
            Some(t) => format!("{}", t),
            None => "end of file".to_string(),
        }
    }

    fn skip_newlines(&mut self) {
        while self.eat(&Token::Newline) {}
    }

    fn error(&mut self, code: crate::errors::ErrorCode, message: String, span: Span) {
        self.errors.push(crate::errors::AuraError::new(
            code,
            crate::errors::Severity::Error,
            message,
            span,
        ));
    }

    /// Skip tokens until we find one that could start a new declaration (error recovery).
    fn recover_to_declaration(&mut self) {
        loop {
            match self.peek() {
                None => return,
                Some(Token::Newline) => {
                    self.advance();
                    // After newline, if we see a declaration keyword at indent 0, stop
                    match self.peek() {
                        Some(
                            Token::App
                            | Token::Model
                            | Token::Screen
                            | Token::Component
                            | Token::Const
                            | Token::Fn
                            | Token::Import
                            | Token::Dedent,
                        ) => return,
                        _ => {}
                    }
                }
                Some(Token::Dedent) => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Skip to end of current indented block.
    fn skip_block(&mut self) {
        let mut depth = 0i32;
        loop {
            match self.peek() {
                None => return,
                Some(Token::Indent) => {
                    depth += 1;
                    self.advance();
                }
                Some(Token::Dedent) => {
                    if depth <= 0 {
                        return;
                    }
                    depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    // === Expect helpers ===

    fn expect_ident(&mut self) -> Result<(String, Span), ()> {
        let span = self.peek_span();
        match self.peek().cloned() {
            Some(Token::Ident(name)) => {
                self.advance();
                Ok((name, span))
            }
            // Accept keywords as identifiers in certain contexts
            // (e.g., field names, method names after dot, prop names)
            Some(tok) if self.token_as_ident(&tok).is_some() => {
                let name = self.token_as_ident(&tok).unwrap();
                self.advance();
                Ok((name, span))
            }
            _ => {
                self.error(
                    crate::errors::ErrorCode::E0702,
                    format!("Expected identifier, found {}", self.peek_desc()),
                    span,
                );
                Err(())
            }
        }
    }

    /// Try to interpret a keyword token as an identifier name.
    /// Many Aura keywords are also valid as field names, method names, etc.
    fn token_as_ident(&self, token: &Token) -> Option<String> {
        match token {
            Token::Icon => Some("icon".to_string()),
            Token::Image => Some("image".to_string()),
            Token::Text => Some("text".to_string()),
            Token::Badge => Some("badge".to_string()),
            Token::Avatar => Some("avatar".to_string()),
            Token::Progress => Some("progress".to_string()),
            Token::Button => Some("button".to_string()),
            Token::Back => Some("back".to_string()),
            Token::Style => Some("style".to_string()),
            Token::Action => Some("action".to_string()),
            Token::State => Some("state".to_string()),
            Token::View => Some("view".to_string()),
            Token::Model => Some("model".to_string()),
            Token::Platform => Some("platform".to_string()),
            Token::Route => Some("route".to_string()),
            Token::Palette => Some("palette".to_string()),
            Token::Variants => Some("variants".to_string()),
            Token::Email => Some("email".to_string()),
            Token::Url => Some("url".to_string()),
            Token::Secret => Some("secret".to_string()),
            Token::Duration => Some("duration".to_string()),
            Token::Toggle => Some("toggle".to_string()),
            Token::Slider => Some("slider".to_string()),
            Token::Picker => Some("picker".to_string()),
            Token::Grid => Some("grid".to_string()),
            Token::Scroll => Some("scroll".to_string()),
            Token::Column => Some("column".to_string()),
            Token::Row => Some("row".to_string()),
            // Numeric-prefixed tokens like 2xl handled separately
            Token::Integer(n) => {
                // Check if followed by an ident to form "2xl", "3xl" etc.
                None
            }
            _ => None,
        }
    }

    fn expect_type_ident(&mut self) -> Result<(String, Span), ()> {
        match self.peek().cloned() {
            Some(Token::TypeIdent(name)) => {
                let span = self.peek_span();
                self.advance();
                Ok((name, span))
            }
            _ => {
                let span = self.peek_span();
                self.error(
                    crate::errors::ErrorCode::E0702,
                    format!("Expected type name (capitalized), found {}", self.peek_desc()),
                    span,
                );
                Err(())
            }
        }
    }

    fn expect_string(&mut self) -> Result<(String, Span), ()> {
        match self.peek().cloned() {
            Some(Token::StringLit(s)) => {
                let span = self.peek_span();
                self.advance();
                Ok((s, span))
            }
            _ => {
                let span = self.peek_span();
                self.error(
                    crate::errors::ErrorCode::E0701,
                    format!("Expected string literal, found {}", self.peek_desc()),
                    span,
                );
                Err(())
            }
        }
    }

    fn expect_indent(&mut self) -> Result<(), ()> {
        self.skip_newlines();
        if self.eat(&Token::Indent) {
            Ok(())
        } else {
            let span = self.peek_span();
            self.error(
                crate::errors::ErrorCode::E0700,
                format!("Expected indented block, found {}", self.peek_desc()),
                span,
            );
            Err(())
        }
    }

    fn expect_dedent(&mut self) {
        self.skip_newlines();
        if !self.eat(&Token::Dedent) && !self.at_end() {
            // Not fatal — just consume remaining tokens in block
        }
    }

    // === Top-level parsing ===

    fn parse_program(&mut self) -> Option<Program> {
        self.skip_newlines();

        let mut imports = Vec::new();
        while self.check(&Token::Import) {
            if let Some(imp) = self.parse_import() {
                imports.push(imp);
            }
            self.skip_newlines();
        }

        let app = self.parse_app()?;

        Some(Program { imports, app })
    }

    // === Import ===

    fn parse_import(&mut self) -> Option<ImportDecl> {
        let start = self.peek_span();
        self.expect(&Token::Import).ok()?;

        let spec = if self.eat(&Token::LBrace) {
            // { Foo, Bar }
            let mut names = Vec::new();
            loop {
                if self.check(&Token::RBrace) {
                    break;
                }
                let (name, _) = self.expect_type_ident().ok()?;
                names.push(name);
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
            self.expect(&Token::RBrace).ok()?;
            ImportSpec::Destructured(names)
        } else if self.eat(&Token::Star) {
            // * as Alias
            self.expect(&Token::As).ok()?;
            let (name, _) = self.expect_type_ident().ok()?;
            ImportSpec::Wildcard(name)
        } else {
            // Single name
            let (name, _) = self.expect_type_ident().ok()?;
            ImportSpec::Named(name)
        };

        self.expect(&Token::From).ok()?;
        let (source, end_span) = self.expect_string().ok()?;
        self.skip_newlines();

        Some(ImportDecl {
            spec,
            source,
            span: start.merge(end_span),
        })
    }

    // === App ===

    fn parse_app(&mut self) -> Option<AppDecl> {
        let start = self.peek_span();
        self.expect(&Token::App).ok()?;
        let (name, name_span) = self.expect_type_ident().ok()?;

        self.expect_indent().ok()?;

        let mut members = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&Token::Dedent) || self.at_end() {
                break;
            }
            if let Some(member) = self.parse_app_member() {
                members.push(member);
            } else {
                self.recover_to_declaration();
            }
        }

        self.expect_dedent();

        Some(AppDecl {
            name,
            members,
            span: start.merge(name_span),
        })
    }

    fn parse_app_member(&mut self) -> Option<AppMember> {
        match self.peek()? {
            Token::Theme => {
                let start = self.peek_span();
                self.advance();
                self.expect(&Token::Colon).ok()?;
                let value = self.parse_expression()?;
                self.skip_newlines();
                Some(AppMember::ThemeRef(ThemeRef {
                    value,
                    span: start,
                }))
            }
            Token::Model => self.parse_model().map(AppMember::Model),
            Token::Screen => self.parse_screen().map(AppMember::Screen),
            Token::Component => self.parse_component().map(AppMember::Component),
            Token::State => self.parse_state_decl().map(AppMember::State),
            Token::Const => self.parse_const().map(AppMember::Const),
            Token::Fn => self.parse_fn().map(AppMember::Fn),
            Token::Ident(s) if s == "navigation" => {
                let start = self.peek_span();
                self.advance();
                self.expect(&Token::Colon).ok()?;
                let (mode, _) = self.expect_ident().ok()?;
                self.skip_newlines();
                Some(AppMember::NavigationDecl(NavigationDecl {
                    mode,
                    span: start,
                }))
            }
            Token::Platform => {
                // platform: all — skip for now
                self.advance();
                self.expect(&Token::Colon).ok()?;
                self.parse_expression()?;
                self.skip_newlines();
                None // Platform is metadata, not a real member yet
            }
            Token::Route => {
                let start = self.peek_span();
                self.advance();
                let (pattern, _) = self.expect_string().ok()?;
                self.expect(&Token::Arrow).ok()?;
                let (screen, _) = self.expect_type_ident().ok()?;
                self.skip_newlines();
                Some(AppMember::RouteDecl(RouteDecl {
                    pattern,
                    screen,
                    params: Vec::new(),
                    span: start,
                }))
            }
            _ => {
                let span = self.peek_span();
                self.error(
                    crate::errors::ErrorCode::E0700,
                    format!("Unexpected token in app body: {}", self.peek_desc()),
                    span,
                );
                None
            }
        }
    }

    // === Model ===

    fn parse_model(&mut self) -> Option<ModelDecl> {
        let start = self.peek_span();
        self.expect(&Token::Model).ok()?;
        let (name, _) = self.expect_type_ident().ok()?;

        self.expect_indent().ok()?;

        let mut fields = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&Token::Dedent) || self.at_end() {
                break;
            }
            if let Some(field) = self.parse_field() {
                fields.push(field);
            } else {
                self.recover_to_declaration();
            }
        }

        self.expect_dedent();

        Some(ModelDecl {
            name,
            fields,
            span: start,
        })
    }

    fn parse_field(&mut self) -> Option<FieldDecl> {
        let (name, start) = self.expect_ident().ok()?;
        self.expect(&Token::Colon).ok()?;
        let type_expr = self.parse_type_expr()?;

        let default = if self.eat(&Token::Eq) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.skip_newlines();

        Some(FieldDecl {
            name,
            type_expr,
            default,
            span: start,
        })
    }

    // === Screen ===

    fn parse_screen(&mut self) -> Option<ScreenDecl> {
        let start = self.peek_span();
        self.expect(&Token::Screen).ok()?;
        let (name, _) = self.expect_type_ident().ok()?;

        let params = if self.eat(&Token::LParen) {
            let params = self.parse_param_list()?;
            self.expect(&Token::RParen).ok()?;
            params
        } else {
            Vec::new()
        };

        // Screen modifiers: tab: "icon" label: "Label"
        let mut modifiers = Vec::new();
        while let Some(Token::Ident(s)) = self.peek().cloned() {
            match s.as_str() {
                "tab" => {
                    self.advance();
                    self.expect(&Token::Colon).ok()?;
                    if self.eat(&Token::Nil) {
                        // tab: nil means no tab
                    } else {
                        let (icon, _) = self.expect_string().ok()?;
                        modifiers.push(ScreenModifier::Tab(icon));
                    }
                }
                "label" => {
                    self.advance();
                    self.expect(&Token::Colon).ok()?;
                    let (label, _) = self.expect_string().ok()?;
                    modifiers.push(ScreenModifier::Label(label));
                }
                _ => break,
            }
        }

        self.expect_indent().ok()?;

        let mut members = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&Token::Dedent) || self.at_end() {
                break;
            }
            if let Some(member) = self.parse_screen_member() {
                members.push(member);
            } else {
                self.recover_to_declaration();
            }
        }

        self.expect_dedent();

        Some(ScreenDecl {
            name,
            params,
            modifiers,
            members,
            span: start,
        })
    }

    fn parse_screen_member(&mut self) -> Option<ScreenMember> {
        match self.peek()? {
            Token::State => self.parse_state_decl().map(ScreenMember::State),
            Token::View => self.parse_view().map(ScreenMember::View),
            Token::Action => self.parse_action().map(ScreenMember::Action),
            Token::Fn => self.parse_fn().map(ScreenMember::Fn),
            Token::On => self.parse_on().map(ScreenMember::On),
            Token::Style => self.parse_style().map(ScreenMember::Style),
            _ => {
                let span = self.peek_span();
                self.error(
                    crate::errors::ErrorCode::E0700,
                    format!("Unexpected token in screen body: {}", self.peek_desc()),
                    span,
                );
                None
            }
        }
    }

    // === Component ===

    fn parse_component(&mut self) -> Option<ComponentDecl> {
        let start = self.peek_span();
        self.expect(&Token::Component).ok()?;
        let (name, _) = self.expect_type_ident().ok()?;

        let props = if self.eat(&Token::LParen) {
            let params = self.parse_param_list()?;
            self.expect(&Token::RParen).ok()?;
            params
        } else {
            Vec::new()
        };

        self.expect_indent().ok()?;

        let mut members = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&Token::Dedent) || self.at_end() {
                break;
            }
            if let Some(member) = self.parse_screen_member() {
                members.push(member);
            } else {
                self.recover_to_declaration();
            }
        }

        self.expect_dedent();

        Some(ComponentDecl {
            name,
            props,
            members,
            span: start,
        })
    }

    // === State ===

    fn parse_state_decl(&mut self) -> Option<StateDecl> {
        let start = self.peek_span();
        self.expect(&Token::State).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        self.expect(&Token::Colon).ok()?;
        let type_expr = self.parse_type_expr()?;

        let default = if self.eat(&Token::Eq) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.skip_newlines();

        Some(StateDecl {
            name,
            type_expr,
            default,
            span: start,
        })
    }

    // === Const ===

    fn parse_const(&mut self) -> Option<ConstDecl> {
        let start = self.peek_span();
        self.expect(&Token::Const).ok()?;

        let name = match self.peek().cloned() {
            Some(Token::Ident(s)) => {
                self.advance();
                s
            }
            Some(Token::TypeIdent(s)) => {
                self.advance();
                s
            }
            _ => {
                let (s, _) = self.expect_ident().ok()?;
                s
            }
        };

        let type_expr = if self.eat(&Token::Colon) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        self.expect(&Token::Eq).ok()?;
        let value = self.parse_expression()?;
        self.skip_newlines();

        Some(ConstDecl {
            name,
            type_expr,
            value,
            span: start,
        })
    }

    // === View ===

    fn parse_view(&mut self) -> Option<ViewDecl> {
        let start = self.peek_span();
        self.expect(&Token::View).ok()?;
        self.expect_indent().ok()?;

        let mut body = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&Token::Dedent) || self.at_end() {
                break;
            }
            if let Some(elem) = self.parse_view_element() {
                body.push(elem);
            } else {
                // Skip to next line on error
                self.recover_to_declaration();
            }
        }

        self.expect_dedent();

        Some(ViewDecl { body, span: start })
    }

    fn parse_view_element(&mut self) -> Option<ViewElement> {
        match self.peek()? {
            // Layout
            Token::Column | Token::Row | Token::Stack | Token::Grid | Token::Scroll
            | Token::Wrap => self.parse_layout().map(ViewElement::Layout),

            // Widgets
            Token::Text | Token::Heading | Token::Image | Token::Icon | Token::Badge
            | Token::Progress | Token::Avatar => self.parse_widget().map(ViewElement::Widget),

            // Inputs
            Token::TextField | Token::TextArea | Token::Checkbox | Token::Toggle
            | Token::Slider | Token::Picker | Token::DatePicker | Token::Segmented
            | Token::Stepper => self.parse_input().map(ViewElement::Input),

            // Button
            Token::Button | Token::Fab => self.parse_button().map(ViewElement::Button),

            // Control flow
            Token::If => self.parse_if_view().map(ViewElement::If),
            Token::Each => self.parse_each_view().map(ViewElement::Each),
            Token::When => self.parse_when_view().map(ViewElement::When),

            // Spacer / Divider / Slot
            Token::Spacer => {
                let span = self.peek_span();
                self.advance();
                self.skip_newlines();
                Some(ViewElement::Spacer(span))
            }
            Token::Divider => {
                let span = self.peek_span();
                self.advance();
                let tokens = self.parse_design_tokens();
                self.skip_newlines();
                Some(ViewElement::Divider(tokens, span))
            }
            Token::Slot => {
                let span = self.peek_span();
                self.advance();
                self.skip_newlines();
                Some(ViewElement::Slot(span))
            }

            // Component reference (starts with TypeIdent)
            Token::TypeIdent(_) => self.parse_component_ref().map(ViewElement::ComponentRef),

            _ => {
                let span = self.peek_span();
                self.error(
                    crate::errors::ErrorCode::E0700,
                    format!("Unexpected token in view: {}", self.peek_desc()),
                    span,
                );
                None
            }
        }
    }

    // === Layout elements ===

    fn parse_layout(&mut self) -> Option<LayoutElement> {
        let start = self.peek_span();
        let kind = match self.peek()? {
            Token::Column => LayoutKind::Column,
            Token::Row => LayoutKind::Row,
            Token::Stack => LayoutKind::Stack,
            Token::Grid => LayoutKind::Grid,
            Token::Scroll => LayoutKind::Scroll,
            Token::Wrap => LayoutKind::Wrap,
            _ => return None,
        };
        self.advance();

        let (tokens, props) = self.parse_design_tokens_and_props();
        self.skip_newlines();

        let children = if self.check(&Token::Indent) {
            self.parse_view_children()
        } else {
            Vec::new()
        };

        Some(LayoutElement {
            kind,
            tokens,
            props,
            children,
            span: start,
        })
    }

    fn parse_view_children(&mut self) -> Vec<ViewElement> {
        let mut children = Vec::new();
        if self.eat(&Token::Indent) {
            loop {
                self.skip_newlines();
                if self.check(&Token::Dedent) || self.at_end() {
                    break;
                }
                if let Some(elem) = self.parse_view_element() {
                    children.push(elem);
                } else {
                    self.recover_to_declaration();
                }
            }
            self.expect_dedent();
        }
        children
    }

    // === Widget elements ===

    fn parse_widget(&mut self) -> Option<WidgetElement> {
        let start = self.peek_span();
        let kind = match self.peek()? {
            Token::Text => WidgetKind::Text,
            Token::Heading => WidgetKind::Heading,
            Token::Image => WidgetKind::Image,
            Token::Icon => WidgetKind::Icon,
            Token::Badge => WidgetKind::Badge,
            Token::Progress => WidgetKind::Progress,
            Token::Avatar => WidgetKind::Avatar,
            _ => return None,
        };
        self.advance();

        // Parse arguments (expressions before design tokens/props)
        let mut args = Vec::new();
        loop {
            match self.peek() {
                Some(Token::StringLit(_)) | Some(Token::Integer(_)) | Some(Token::FloatLit(_)) => {
                    if let Some(expr) = self.parse_expression() {
                        args.push(expr);
                    } else {
                        break;
                    }
                }
                Some(Token::Ident(_)) | Some(Token::TypeIdent(_)) => {
                    // Could be a prop assignment (name: value) or an expression
                    // Lookahead: if next is Colon, it's a prop — handle in props parsing
                    if self.is_prop_assign_ahead() {
                        break;
                    }
                    if let Some(expr) = self.parse_expression() {
                        args.push(expr);
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        let (tokens, props) = self.parse_design_tokens_and_props();
        self.skip_newlines();

        Some(WidgetElement {
            kind,
            args,
            tokens,
            props,
            span: start,
        })
    }

    // === Input elements ===

    fn parse_input(&mut self) -> Option<InputElement> {
        let start = self.peek_span();
        let kind = match self.peek()? {
            Token::TextField => InputKind::TextField,
            Token::TextArea => InputKind::TextArea,
            Token::Checkbox => InputKind::Checkbox,
            Token::Toggle => InputKind::Toggle,
            Token::Slider => InputKind::Slider,
            Token::Picker => InputKind::Picker,
            Token::DatePicker => InputKind::DatePicker,
            Token::Segmented => InputKind::Segmented,
            Token::Stepper => InputKind::Stepper,
            _ => return None,
        };
        self.advance();

        // Binding (identifier or member access)
        let binding = self.parse_binding_path()?;

        let (tokens, props) = self.parse_design_tokens_and_props();

        let action = if self.eat(&Token::Arrow) {
            Some(self.parse_action_expr()?)
        } else {
            None
        };

        self.skip_newlines();

        // Check for indented continuation (e.g., textfield with -> on next line)
        let action = if action.is_none() && self.check(&Token::Indent) {
            let saved_pos = self.pos;
            self.advance(); // eat indent
            self.skip_newlines();
            if self.eat(&Token::Arrow) {
                let a = self.parse_action_expr();
                self.skip_newlines();
                self.expect_dedent();
                a
            } else {
                self.pos = saved_pos;
                None
            }
        } else {
            action
        };

        Some(InputElement {
            kind,
            binding,
            tokens,
            props,
            action,
            span: start,
        })
    }

    fn parse_binding_path(&mut self) -> Option<String> {
        let (first, _) = self.expect_ident().ok()?;
        let mut path = first;
        while self.check(&Token::Dot) {
            // Lookahead: is next token after dot an identifier? (not a design token)
            if let Some(Token::Ident(_)) = self.tokens.get(self.pos + 1).map(|t| &t.value) {
                // Check it's not a design token context (heuristic: design tokens are
                // known token names like md, lg, accent, bold etc.)
                let next_name = if let Some(Token::Ident(s)) = self.tokens.get(self.pos + 1).map(|t| &t.value) {
                    s.clone()
                } else {
                    break;
                };
                if is_design_token_name(&next_name) {
                    break;
                }
                self.advance(); // eat dot
                self.advance(); // eat ident
                path = format!("{}.{}", path, next_name);
            } else {
                break;
            }
        }
        Some(path)
    }

    // === Button ===

    fn parse_button(&mut self) -> Option<ButtonElement> {
        let start = self.peek_span();
        self.advance(); // eat 'button' or 'fab'

        // Optional style: button.icon, button.outline, etc.
        let style = if self.eat(&Token::Dot) {
            let (s, _) = self.expect_ident().ok()?;
            Some(s)
        } else {
            None
        };

        // Label expression
        let label = self.parse_expression()?;

        let (tokens, props) = self.parse_design_tokens_and_props();

        self.expect(&Token::Arrow).ok()?;
        let action = self.parse_action_expr()?;

        self.skip_newlines();

        Some(ButtonElement {
            style,
            label,
            tokens,
            props,
            action,
            span: start,
        })
    }

    // === Control flow in views ===

    fn parse_if_view(&mut self) -> Option<IfView> {
        let start = self.peek_span();
        self.expect(&Token::If).ok()?;
        let condition = self.parse_expression()?;
        self.skip_newlines();

        let then_body = self.parse_view_children();

        let else_body = if self.eat(&Token::Else) {
            self.skip_newlines();
            Some(self.parse_view_children())
        } else {
            None
        };

        Some(IfView {
            condition,
            then_body,
            else_body,
            span: start,
        })
    }

    fn parse_each_view(&mut self) -> Option<EachView> {
        let start = self.peek_span();
        self.expect(&Token::Each).ok()?;
        let iterable = self.parse_expression()?;
        self.expect(&Token::As).ok()?;
        let (item_name, _) = self.expect_ident().ok()?;

        let index_name = if self.eat(&Token::Comma) {
            let (idx, _) = self.expect_ident().ok()?;
            Some(idx)
        } else {
            None
        };

        self.skip_newlines();
        let body = self.parse_view_children();

        Some(EachView {
            iterable,
            item_name,
            index_name,
            body,
            span: start,
        })
    }

    fn parse_when_view(&mut self) -> Option<WhenView> {
        let start = self.peek_span();
        self.expect(&Token::When).ok()?;
        let expression = self.parse_expression()?;
        self.skip_newlines();

        let mut branches = Vec::new();
        if self.eat(&Token::Indent) {
            loop {
                self.skip_newlines();
                if self.check(&Token::Dedent) || self.at_end() {
                    break;
                }
                if let Some(branch) = self.parse_when_view_branch() {
                    branches.push(branch);
                } else {
                    self.recover_to_declaration();
                }
            }
            self.expect_dedent();
        }

        Some(WhenView {
            expression,
            branches,
            span: start,
        })
    }

    fn parse_when_view_branch(&mut self) -> Option<WhenBranch> {
        let start = self.peek_span();
        self.expect(&Token::Is).ok()?;
        let pattern = self.parse_pattern()?;
        self.skip_newlines();
        let body = self.parse_view_children();

        Some(WhenBranch {
            pattern,
            body,
            span: start,
        })
    }

    // === Component ref in view ===

    fn parse_component_ref(&mut self) -> Option<ComponentRef> {
        let start = self.peek_span();
        let (name, _) = self.expect_type_ident().ok()?;

        let args = if self.eat(&Token::LParen) {
            let args = self.parse_named_args();
            self.expect(&Token::RParen).ok()?;
            args
        } else {
            Vec::new()
        };

        self.skip_newlines();

        let children = if self.check(&Token::Indent) {
            self.parse_view_children()
        } else {
            Vec::new()
        };

        Some(ComponentRef {
            name,
            args,
            children,
            span: start,
        })
    }

    // === Action / Fn / On declarations ===

    fn parse_action(&mut self) -> Option<ActionDecl> {
        let start = self.peek_span();
        self.expect(&Token::Action).ok()?;
        let (name, _) = self.expect_ident().ok()?;

        let params = if self.eat(&Token::LParen) {
            let p = self.parse_param_list()?;
            self.expect(&Token::RParen).ok()?;
            p
        } else {
            Vec::new()
        };

        self.expect_indent().ok()?;
        let body = self.parse_statement_block();
        self.expect_dedent();

        Some(ActionDecl {
            name,
            params,
            body,
            span: start,
        })
    }

    fn parse_fn(&mut self) -> Option<FnDecl> {
        let start = self.peek_span();
        self.expect(&Token::Fn).ok()?;
        let (name, _) = self.expect_ident().ok()?;

        let params = if self.eat(&Token::LParen) {
            let p = self.parse_param_list()?;
            self.expect(&Token::RParen).ok()?;
            p
        } else {
            Vec::new()
        };

        let return_type = if self.eat(&Token::Arrow) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        self.expect_indent().ok()?;
        let body = self.parse_statement_block();
        self.expect_dedent();

        Some(FnDecl {
            name,
            params,
            return_type,
            body,
            span: start,
        })
    }

    fn parse_on(&mut self) -> Option<OnDecl> {
        let start = self.peek_span();
        self.expect(&Token::On).ok()?;
        let (event, _) = self.expect_ident().ok()?;

        let params = if self.eat(&Token::LParen) {
            let p = self.parse_param_list()?;
            self.expect(&Token::RParen).ok()?;
            p
        } else {
            Vec::new()
        };

        self.expect_indent().ok()?;
        let body = self.parse_statement_block();
        self.expect_dedent();

        Some(OnDecl {
            event,
            params,
            body,
            span: start,
        })
    }

    fn parse_style(&mut self) -> Option<StyleDecl> {
        let start = self.peek_span();
        self.expect(&Token::Style).ok()?;
        let (name, _) = self.expect_ident().ok()?;

        self.expect_indent().ok()?;
        let (tokens, props) = self.parse_design_tokens_and_props();
        self.skip_newlines();
        self.expect_dedent();

        Some(StyleDecl {
            name,
            tokens,
            props,
            span: start,
        })
    }

    // === Statement blocks ===

    fn parse_statement_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.check(&Token::Dedent) || self.at_end() {
                break;
            }
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                self.recover_to_declaration();
            }
        }
        stmts
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.peek()? {
            Token::Let => {
                let start = self.peek_span();
                self.advance();
                let (name, _) = self.expect_ident().ok()?;
                let type_expr = if self.eat(&Token::Colon) {
                    Some(self.parse_type_expr()?)
                } else {
                    None
                };
                self.expect(&Token::Eq).ok()?;
                let value = self.parse_expression()?;
                self.skip_newlines();
                Some(Stmt::Let(name, type_expr, value, start))
            }
            Token::If => {
                let start = self.peek_span();
                self.advance();
                let cond = self.parse_expression()?;
                self.expect_indent().ok()?;
                let then_body = self.parse_statement_block();
                self.expect_dedent();
                let else_body = if self.eat(&Token::Else) {
                    self.expect_indent().ok()?;
                    let body = self.parse_statement_block();
                    self.expect_dedent();
                    Some(body)
                } else {
                    None
                };
                Some(Stmt::If(cond, then_body, else_body, start))
            }
            Token::When => {
                let start = self.peek_span();
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_indent().ok()?;
                let mut branches = Vec::new();
                loop {
                    self.skip_newlines();
                    if self.check(&Token::Dedent) || self.at_end() {
                        break;
                    }
                    if let Some(branch) = self.parse_when_stmt_branch() {
                        branches.push(branch);
                    } else {
                        self.recover_to_declaration();
                    }
                }
                self.expect_dedent();
                Some(Stmt::When(expr, branches, start))
            }
            Token::Navigate => {
                self.advance();
                let nav = self.parse_navigate_expr()?;
                self.skip_newlines();
                Some(Stmt::Navigate(nav))
            }
            Token::Return => {
                let start = self.peek_span();
                self.advance();
                let value = if self.check(&Token::Newline) || self.check(&Token::Dedent) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                self.skip_newlines();
                Some(Stmt::Return(value, start))
            }
            Token::Emit => {
                let start = self.peek_span();
                self.advance();
                let (name, _) = self.expect_ident().ok()?;
                let args = if self.eat(&Token::LParen) {
                    let a = self.parse_arg_list();
                    self.expect(&Token::RParen).ok()?;
                    a
                } else {
                    Vec::new()
                };
                self.skip_newlines();
                Some(Stmt::Emit(name, args, start))
            }
            Token::Ident(_) => {
                // Could be assignment (x = ...) or expression statement (f())
                let start = self.peek_span();
                let expr = self.parse_expression()?;

                if self.eat(&Token::Eq) {
                    // Assignment
                    let name = match expr {
                        Expr::Var(name, _) => name,
                        Expr::MemberAccess(_, field, _) => field, // simplified
                        _ => {
                            self.error(
                                crate::errors::ErrorCode::E0700,
                                "Invalid assignment target".to_string(),
                                start,
                            );
                            return None;
                        }
                    };
                    let value = self.parse_expression()?;
                    self.skip_newlines();
                    Some(Stmt::Assign(name, value, start))
                } else {
                    self.skip_newlines();
                    Some(Stmt::Expr(expr, start))
                }
            }
            _ => {
                let start = self.peek_span();
                let expr = self.parse_expression()?;
                self.skip_newlines();
                Some(Stmt::Expr(expr, start))
            }
        }
    }

    fn parse_when_stmt_branch(&mut self) -> Option<WhenStmtBranch> {
        let start = self.peek_span();
        self.expect(&Token::Is).ok()?;
        let pattern = self.parse_pattern()?;

        if self.eat(&Token::Arrow) {
            // Single-line: is pattern -> expr
            let expr = self.parse_expression()?;
            self.skip_newlines();
            Some(WhenStmtBranch {
                pattern,
                body: StmtOrExpr::Expr(expr),
                span: start,
            })
        } else {
            // Multi-line block
            self.expect_indent().ok()?;
            let stmts = self.parse_statement_block();
            self.expect_dedent();
            Some(WhenStmtBranch {
                pattern,
                body: StmtOrExpr::Stmt(stmts),
                span: start,
            })
        }
    }

    // === Patterns ===

    fn parse_pattern(&mut self) -> Option<Pattern> {
        match self.peek()? {
            Token::Dot => {
                let span = self.peek_span();
                self.advance();
                let (name, _) = self.expect_ident().ok()?;
                Some(Pattern::EnumVariant(name, span))
            }
            Token::Some_ => {
                let span = self.peek_span();
                self.advance();
                self.expect(&Token::LParen).ok()?;
                let (name, _) = self.expect_ident().ok()?;
                self.expect(&Token::RParen).ok()?;
                Some(Pattern::Some(name, span))
            }
            Token::Nil => {
                let span = self.peek_span();
                self.advance();
                Some(Pattern::Nil(span))
            }
            Token::StringLit(_) | Token::Integer(_) | Token::FloatLit(_) | Token::True
            | Token::False => {
                let expr = self.parse_expression()?;
                Some(Pattern::Literal(expr))
            }
            Token::Ident(_) => {
                let (name, span) = self.expect_ident().ok()?;
                Some(Pattern::Identifier(name, span))
            }
            Token::TypeIdent(_) => {
                let (name, span) = self.expect_type_ident().ok()?;
                Some(Pattern::Constructor(name, Vec::new(), span))
            }
            _ => {
                let span = self.peek_span();
                self.error(
                    crate::errors::ErrorCode::E0700,
                    format!("Expected pattern, found {}", self.peek_desc()),
                    span,
                );
                None
            }
        }
    }

    // === Action expressions (right side of ->) ===

    fn parse_action_expr(&mut self) -> Option<ActionExpr> {
        if self.check(&Token::Navigate) {
            self.advance();
            let nav = self.parse_navigate_expr()?;
            Some(ActionExpr::Navigate(nav))
        } else {
            let (name, span) = self.expect_ident().ok()?;
            let args = if self.eat(&Token::LParen) {
                let a = self.parse_arg_list();
                self.expect(&Token::RParen).ok()?;
                a
            } else {
                Vec::new()
            };
            Some(ActionExpr::Call(name, args, span))
        }
    }

    fn parse_navigate_expr(&mut self) -> Option<NavigateExpr> {
        if self.eat(&Token::Dot) {
            let (method, span) = self.expect_ident().ok()?;
            match method.as_str() {
                "back" => Some(NavigateExpr::Back(span)),
                "root" => Some(NavigateExpr::Root(span)),
                "dismiss" => Some(NavigateExpr::Dismiss(span)),
                "replace" => {
                    self.expect(&Token::LParen).ok()?;
                    let expr = self.parse_expression()?;
                    self.expect(&Token::RParen).ok()?;
                    Some(NavigateExpr::Replace(expr, span))
                }
                "modal" => {
                    self.expect(&Token::LParen).ok()?;
                    let expr = self.parse_expression()?;
                    self.expect(&Token::RParen).ok()?;
                    Some(NavigateExpr::Modal(expr, span))
                }
                _ => {
                    self.error(
                        crate::errors::ErrorCode::E0700,
                        format!("Unknown navigate method: {}", method),
                        span,
                    );
                    None
                }
            }
        } else {
            self.expect(&Token::LParen).ok()?;
            let expr = self.parse_expression()?;
            self.expect(&Token::RParen).ok()?;
            let span = self.peek_span();
            Some(NavigateExpr::To(expr, span))
        }
    }

    // === Type expressions ===

    fn parse_type_expr(&mut self) -> Option<TypeExpr> {
        let span = self.peek_span();

        match self.peek()?.clone() {
            // Collection types: list[T], map[K,V], set[T]
            Token::List | Token::Map | Token::Set => {
                let kind = match self.peek()? {
                    Token::List => "list",
                    Token::Map => "map",
                    Token::Set => "set",
                    _ => unreachable!(),
                }
                .to_string();
                self.advance();
                self.expect(&Token::LBracket).ok()?;
                let mut type_args = vec![self.parse_type_expr()?];
                while self.eat(&Token::Comma) {
                    type_args.push(self.parse_type_expr()?);
                }
                self.expect(&Token::RBracket).ok()?;
                Some(TypeExpr::Collection(kind, type_args, span))
            }
            // Optional
            Token::Optional => {
                self.advance();
                self.expect(&Token::LBracket).ok()?;
                let inner = self.parse_type_expr()?;
                self.expect(&Token::RBracket).ok()?;
                Some(TypeExpr::Optional(Box::new(inner), span))
            }
            // Enum
            Token::Enum => {
                self.advance();
                self.expect(&Token::LBracket).ok()?;
                let mut variants = Vec::new();
                loop {
                    if self.check(&Token::RBracket) {
                        break;
                    }
                    let (name, vspan) = self.expect_ident().ok()?;
                    let fields = if self.eat(&Token::LParen) {
                        let p = self.parse_param_list()?;
                        self.expect(&Token::RParen).ok()?;
                        p
                    } else {
                        Vec::new()
                    };
                    variants.push(EnumVariantExpr {
                        name,
                        fields,
                        span: vspan,
                    });
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RBracket).ok()?;
                Some(TypeExpr::Enum(variants, span))
            }
            // Function type: fn(T) -> U
            Token::Fn => {
                self.advance();
                self.expect(&Token::LParen).ok()?;
                let mut params = Vec::new();
                while !self.check(&Token::RParen) && !self.at_end() {
                    params.push(self.parse_type_expr()?);
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen).ok()?;
                let ret = if self.eat(&Token::Arrow) {
                    Some(Box::new(self.parse_type_expr()?))
                } else {
                    None
                };
                Some(TypeExpr::Function(params, ret, span))
            }
            // Action type
            Token::Action => {
                self.advance();
                let params = if self.eat(&Token::LParen) {
                    let mut p = Vec::new();
                    while !self.check(&Token::RParen) && !self.at_end() {
                        p.push(self.parse_type_expr()?);
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::RParen).ok()?;
                    p
                } else {
                    Vec::new()
                };
                Some(TypeExpr::Action(params, span))
            }
            // Named types (primitives + user types)
            Token::Text | Token::Int | Token::Float | Token::Bool | Token::Timestamp
            | Token::Duration | Token::Percent | Token::Secret | Token::Sanitized
            | Token::Email | Token::Url | Token::TokenType => {
                let name = match self.peek().unwrap() {
                    Token::Text => "text",
                    Token::Int => "int",
                    Token::Float => "float",
                    Token::Bool => "bool",
                    Token::Timestamp => "timestamp",
                    Token::Duration => "duration",
                    Token::Percent => "percent",
                    Token::Secret => "secret",
                    Token::Sanitized => "sanitized",
                    Token::Email => "email",
                    Token::Url => "url",
                    Token::TokenType => "token",
                    _ => unreachable!(),
                }.to_string();
                self.advance();
                Some(TypeExpr::Named(name, span))
            }
            Token::TypeIdent(name) => {
                self.advance();
                Some(TypeExpr::Named(name, span))
            }
            Token::Ident(name) => {
                self.advance();
                Some(TypeExpr::Named(name, span))
            }
            _ => {
                self.error(
                    crate::errors::ErrorCode::E0700,
                    format!("Expected type, found {}", self.peek_desc()),
                    span,
                );
                None
            }
        }
    }

    // === Parameters ===

    fn parse_param_list(&mut self) -> Option<Vec<Param>> {
        let mut params = Vec::new();
        loop {
            if self.check(&Token::RParen) || self.at_end() {
                break;
            }
            self.skip_newlines();
            let (name, span) = self.expect_ident().ok()?;
            self.expect(&Token::Colon).ok()?;
            let type_expr = self.parse_type_expr()?;
            let default = if self.eat(&Token::Eq) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            params.push(Param {
                name,
                type_expr,
                default,
                span,
            });
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        Some(params)
    }

    // === Expressions ===

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Option<Expr> {
        let mut left = self.parse_nil_coalesce()?;
        while self.eat(&Token::Pipe) {
            let span = self.peek_span();
            let right = self.parse_nil_coalesce()?;
            left = Expr::Pipe(Box::new(left), Box::new(right), span);
        }
        Some(left)
    }

    fn parse_nil_coalesce(&mut self) -> Option<Expr> {
        let mut left = self.parse_or()?;
        while self.eat(&Token::NilCoalesce) {
            let span = self.peek_span();
            let right = self.parse_or()?;
            left = Expr::NilCoalesce(Box::new(left), Box::new(right), span);
        }
        Some(left)
    }

    fn parse_or(&mut self) -> Option<Expr> {
        let mut left = self.parse_and()?;
        while self.eat(&Token::Or) {
            let span = self.peek_span();
            let right = self.parse_and()?;
            left = Expr::BinOp(Box::new(left), BinOp::Or, Box::new(right), span);
        }
        Some(left)
    }

    fn parse_and(&mut self) -> Option<Expr> {
        let mut left = self.parse_equality()?;
        while self.eat(&Token::And) {
            let span = self.peek_span();
            let right = self.parse_equality()?;
            left = Expr::BinOp(Box::new(left), BinOp::And, Box::new(right), span);
        }
        Some(left)
    }

    fn parse_equality(&mut self) -> Option<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            let span = self.peek_span();
            if self.eat(&Token::EqEq) {
                let right = self.parse_comparison()?;
                left = Expr::BinOp(Box::new(left), BinOp::Eq, Box::new(right), span);
            } else if self.eat(&Token::NotEq) {
                let right = self.parse_comparison()?;
                left = Expr::BinOp(Box::new(left), BinOp::NotEq, Box::new(right), span);
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_addition()?;
        loop {
            let span = self.peek_span();
            if self.eat(&Token::Lt) {
                let right = self.parse_addition()?;
                left = Expr::BinOp(Box::new(left), BinOp::Lt, Box::new(right), span);
            } else if self.eat(&Token::Gt) {
                let right = self.parse_addition()?;
                left = Expr::BinOp(Box::new(left), BinOp::Gt, Box::new(right), span);
            } else if self.eat(&Token::LtEq) {
                let right = self.parse_addition()?;
                left = Expr::BinOp(Box::new(left), BinOp::LtEq, Box::new(right), span);
            } else if self.eat(&Token::GtEq) {
                let right = self.parse_addition()?;
                left = Expr::BinOp(Box::new(left), BinOp::GtEq, Box::new(right), span);
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_addition(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplication()?;
        loop {
            let span = self.peek_span();
            if self.eat(&Token::Plus) {
                let right = self.parse_multiplication()?;
                left = Expr::BinOp(Box::new(left), BinOp::Add, Box::new(right), span);
            } else if self.eat(&Token::Minus) {
                let right = self.parse_multiplication()?;
                left = Expr::BinOp(Box::new(left), BinOp::Sub, Box::new(right), span);
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_multiplication(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let span = self.peek_span();
            if self.eat(&Token::Star) {
                let right = self.parse_unary()?;
                left = Expr::BinOp(Box::new(left), BinOp::Mul, Box::new(right), span);
            } else if self.eat(&Token::Slash) {
                let right = self.parse_unary()?;
                left = Expr::BinOp(Box::new(left), BinOp::Div, Box::new(right), span);
            } else if self.eat(&Token::Modulo) {
                let right = self.parse_unary()?;
                left = Expr::BinOp(Box::new(left), BinOp::Mod, Box::new(right), span);
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        let span = self.peek_span();
        if self.eat(&Token::Not) {
            let expr = self.parse_unary()?;
            Some(Expr::UnaryOp(UnaryOp::Not, Box::new(expr), span))
        } else if self.eat(&Token::Minus) {
            let expr = self.parse_unary()?;
            Some(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr), span))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.eat(&Token::Dot) {
                let span = self.peek_span();
                let (member, _) = self.expect_ident().ok()?;
                expr = Expr::MemberAccess(Box::new(expr), member, span);
            } else if self.eat(&Token::LParen) {
                let span = self.peek_span();
                // Check if named args or positional
                if self.is_named_call_ahead() {
                    let args = self.parse_named_args();
                    self.expect(&Token::RParen).ok()?;
                    expr = Expr::NamedCall(Box::new(expr), args, span);
                } else {
                    let args = self.parse_arg_list();
                    self.expect(&Token::RParen).ok()?;
                    expr = Expr::Call(Box::new(expr), args, span);
                }
            } else if self.eat(&Token::LBracket) {
                let span = self.peek_span();
                let index = self.parse_expression()?;
                self.expect(&Token::RBracket).ok()?;
                expr = Expr::Index(Box::new(expr), Box::new(index), span);
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let span = self.peek_span();

        match self.peek()?.clone() {
            Token::Integer(n) => {
                self.advance();
                Some(Expr::IntLit(n, span))
            }
            Token::FloatLit(f) => {
                self.advance();
                Some(Expr::FloatLit(f, span))
            }
            Token::StringLit(s) => {
                self.advance();
                Some(Expr::StringLit(s, span))
            }
            Token::True => {
                self.advance();
                Some(Expr::BoolLit(true, span))
            }
            Token::False => {
                self.advance();
                Some(Expr::BoolLit(false, span))
            }
            Token::Nil => {
                self.advance();
                Some(Expr::Nil(span))
            }
            Token::LParen => {
                self.advance();
                // Check for lambda: (params) => expr
                // Or just grouping: (expr)
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen).ok()?;
                Some(expr)
            }
            Token::LBracket => {
                // List literal: [1, 2, 3]
                self.advance();
                let mut items = Vec::new();
                while !self.check(&Token::RBracket) && !self.at_end() {
                    items.push(self.parse_expression()?);
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RBracket).ok()?;
                // Represent as a constructor for now
                Some(Expr::Constructor("list".to_string(),
                    items.into_iter().enumerate().map(|(i, e)| (format!("{}", i), e)).collect(),
                    span))
            }
            Token::If => {
                // Conditional expression: if a then b else c
                self.advance();
                let cond = self.parse_expression()?;
                self.expect(&Token::Then).ok()?;
                let then_expr = self.parse_expression()?;
                self.expect(&Token::Else).ok()?;
                let else_expr = self.parse_expression()?;
                Some(Expr::Conditional(
                    Box::new(cond),
                    Box::new(then_expr),
                    Box::new(else_expr),
                    span,
                ))
            }
            Token::TypeIdent(name) => {
                self.advance();
                // Constructor: TypeName(args) or just a type reference
                if self.eat(&Token::LParen) {
                    let args = self.parse_named_args();
                    self.expect(&Token::RParen).ok()?;
                    Some(Expr::Constructor(name, args, span))
                } else {
                    Some(Expr::Var(name, span))
                }
            }
            Token::Ident(name) => {
                self.advance();
                // Check for lambda: name => expr
                if self.check(&Token::FatArrow) {
                    self.advance();
                    let body = self.parse_expression()?;
                    let param = Param {
                        name: name.clone(),
                        type_expr: TypeExpr::Named("_".to_string(), span),
                        default: None,
                        span,
                    };
                    Some(Expr::Lambda(vec![param], Box::new(body), span))
                } else {
                    Some(Expr::Var(name, span))
                }
            }
            Token::Dot => {
                // Design token as expression: .accent, .bold
                self.advance();
                let mut segments = Vec::new();
                let (first, _) = self.expect_ident().ok()?;
                segments.push(first);
                while self.eat(&Token::Dot) {
                    if let Some(Token::Ident(_)) = self.peek() {
                        let (s, _) = self.expect_ident().ok()?;
                        segments.push(s);
                    } else {
                        break;
                    }
                }
                Some(Expr::DesignToken(segments, span))
            }
            _ => {
                self.error(
                    crate::errors::ErrorCode::E0701,
                    format!("Expected expression, found {}", self.peek_desc()),
                    span,
                );
                None
            }
        }
    }

    fn parse_arg_list(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        while !self.check(&Token::RParen) && !self.at_end() {
            self.skip_newlines();
            if let Some(expr) = self.parse_expression() {
                args.push(expr);
            } else {
                break;
            }
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        args
    }

    fn parse_named_args(&mut self) -> Vec<(String, Expr)> {
        let mut args = Vec::new();
        while !self.check(&Token::RParen) && !self.at_end() {
            self.skip_newlines();
            if let Some(Token::Ident(name)) = self.peek().cloned() {
                if self.tokens.get(self.pos + 1).map(|t| &t.value) == Some(&Token::Colon) {
                    self.advance(); // name
                    self.advance(); // colon
                    if let Some(value) = self.parse_expression() {
                        args.push((name, value));
                    }
                } else if let Some(expr) = self.parse_expression() {
                    args.push(("_".to_string(), expr));
                }
            } else if let Some(expr) = self.parse_expression() {
                args.push(("_".to_string(), expr));
            } else {
                break;
            }
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        args
    }

    // === Design tokens and props ===

    fn parse_design_tokens(&mut self) -> Vec<DesignToken> {
        let mut tokens = Vec::new();
        while self.check(&Token::Dot) {
            let span = self.peek_span();
            self.advance(); // eat the dot
            if let Some(first_segment) = self.try_parse_token_segment() {
                let mut segments = vec![first_segment];
                // Compound: .gap.md, etc.
                while self.check(&Token::Dot) {
                    // Peek ahead to see if next after dot is a valid segment
                    let saved = self.pos;
                    self.advance(); // eat dot
                    if let Some(seg) = self.try_parse_token_segment() {
                        segments.push(seg);
                    } else {
                        self.pos = saved; // put the dot back
                        break;
                    }
                }
                tokens.push(DesignToken { segments, span });
            } else {
                break;
            }
        }
        tokens
    }

    fn parse_design_tokens_and_props(&mut self) -> (Vec<DesignToken>, Vec<PropAssign>) {
        let mut tokens = Vec::new();
        let mut props = Vec::new();

        loop {
            if self.check(&Token::Dot) {
                // .accent, .bold, .md etc.
                tokens.extend(self.parse_design_tokens());
            } else if self.is_compound_design_token_ahead() {
                // gap.md, padding.lg, align.center, size.xl etc.
                if let Some(token) = self.parse_compound_design_token() {
                    tokens.push(token);
                } else {
                    break;
                }
            } else if self.is_prop_assign_ahead() {
                // name: value
                if let Some(prop) = self.parse_prop_assign() {
                    props.push(prop);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        (tokens, props)
    }

    /// Check if current position has a compound design token like `gap.md`.
    fn is_compound_design_token_ahead(&self) -> bool {
        if let Some(Token::Ident(name)) = self.peek() {
            if is_compound_token_prefix(name) {
                // Check next token is Dot
                if self.tokens.get(self.pos + 1).map(|t| &t.value) == Some(&Token::Dot) {
                    return true;
                }
            }
        }
        false
    }

    /// Parse a compound design token: `gap.md`, `padding.horizontal.lg`, etc.
    fn parse_compound_design_token(&mut self) -> Option<DesignToken> {
        let span = self.peek_span();
        let (first, _) = self.expect_ident().ok()?;
        let mut segments = vec![first];

        while self.eat(&Token::Dot) {
            if let Some(segment) = self.try_parse_token_segment() {
                segments.push(segment);
            } else {
                break;
            }
        }

        Some(DesignToken { segments, span })
    }

    /// Try to parse a design token segment — handles identifiers and
    /// numeric-prefixed tokens like `2xl`, `3xl`, `4xl`.
    fn try_parse_token_segment(&mut self) -> Option<String> {
        match self.peek() {
            Some(Token::Ident(_)) => {
                let (s, _) = self.expect_ident().ok()?;
                Some(s)
            }
            Some(Token::Integer(n)) => {
                // Check for numeric prefix: 2xl, 3xl, 4xl
                let n = *n;
                let next_is_ident = matches!(
                    self.tokens.get(self.pos + 1).map(|t| &t.value),
                    Some(Token::Ident(_))
                );
                if next_is_ident {
                    self.advance(); // eat integer
                    if let Some(Token::Ident(suffix)) = self.peek().cloned() {
                        self.advance(); // eat suffix
                        Some(format!("{}{}", n, suffix))
                    } else {
                        Some(n.to_string())
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_prop_assign(&mut self) -> Option<PropAssign> {
        let (name, span) = self.expect_ident().ok()?;
        self.expect(&Token::Colon).ok()?;
        let value = self.parse_expression()?;
        Some(PropAssign { name, value, span })
    }

    fn is_prop_assign_ahead(&self) -> bool {
        matches!(
            (self.tokens.get(self.pos).map(|t| &t.value), self.tokens.get(self.pos + 1).map(|t| &t.value)),
            (Some(Token::Ident(_)), Some(Token::Colon))
        )
    }

    fn is_named_call_ahead(&self) -> bool {
        // Look for pattern: ident : at start of arg list
        matches!(
            (self.tokens.get(self.pos).map(|t| &t.value), self.tokens.get(self.pos + 1).map(|t| &t.value)),
            (Some(Token::Ident(_)), Some(Token::Colon))
        )
    }
}

/// Check if a name is a known design token.
/// Check if name is a compound design token prefix (e.g., `gap`, `padding`, `margin`).
fn is_compound_token_prefix(name: &str) -> bool {
    matches!(
        name,
        "gap" | "padding" | "margin" | "size" | "width" | "height"
            | "radius" | "shadow" | "elevation" | "opacity"
            | "align" | "justify" | "max" | "min" | "direction"
    )
}

fn is_design_token_name(name: &str) -> bool {
    matches!(
        name,
        "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "display"
            | "primary" | "secondary" | "muted" | "accent" | "danger" | "warning"
            | "success" | "info" | "surface" | "background" | "divider"
            | "bold" | "medium" | "semibold" | "heavy" | "light" | "thin" | "regular" | "black"
            | "italic" | "mono" | "underline" | "strike" | "center" | "leading" | "trailing"
            | "uppercase" | "lowercase" | "capitalize"
            | "sharp" | "subtle" | "rounded" | "smooth" | "pill" | "circle"
            | "ease" | "spring" | "bounce" | "instant" | "fast" | "normal" | "slow"
            | "fill" | "fit" | "indeterminate"
            | "horizontal" | "vertical"
    )
}

/// Parse source code into an AST.
pub fn parse(source: &str) -> ParseResult {
    let lex_result = crate::lexer::lex(source);

    let mut parser = Parser::new(lex_result.tokens);
    let program = parser.parse_program();

    let mut errors = lex_result.errors;
    errors.extend(parser.errors);

    ParseResult { program, errors }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let result = parse("app Hello\n  screen Main\n    view\n      text \"Hello\"");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let program = result.program.unwrap();
        assert_eq!(program.app.name, "Hello");
        assert_eq!(program.app.members.len(), 1);
    }

    #[test]
    fn test_parse_model() {
        let result = parse("\
app Test
  model Todo
    title: text
    done: bool = false");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let program = result.program.unwrap();
        match &program.app.members[0] {
            AppMember::Model(m) => {
                assert_eq!(m.name, "Todo");
                assert_eq!(m.fields.len(), 2);
                assert_eq!(m.fields[0].name, "title");
                assert_eq!(m.fields[1].name, "done");
            }
            _ => panic!("Expected Model"),
        }
    }

    #[test]
    fn test_parse_screen_with_state() {
        let result = parse("\
app Test
  screen Main
    state count: int = 0
    view
      text \"hello\"");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let program = result.program.unwrap();
        match &program.app.members[0] {
            AppMember::Screen(s) => {
                assert_eq!(s.name, "Main");
                assert!(s.members.iter().any(|m| matches!(m, ScreenMember::State(_))));
                assert!(s.members.iter().any(|m| matches!(m, ScreenMember::View(_))));
            }
            _ => panic!("Expected Screen"),
        }
    }

    #[test]
    fn test_parse_component_with_props() {
        let result = parse("\
app Test
  component Card(title: text, onTap: action)
    view
      text title");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let program = result.program.unwrap();
        match &program.app.members[0] {
            AppMember::Component(c) => {
                assert_eq!(c.name, "Card");
                assert_eq!(c.props.len(), 2);
                assert_eq!(c.props[0].name, "title");
                assert_eq!(c.props[1].name, "onTap");
            }
            _ => panic!("Expected Component"),
        }
    }

    #[test]
    fn test_parse_action_block() {
        let result = parse("\
app Test
  screen Main
    state x: int = 0
    view
      text \"hi\"
    action increment
      x = x + 1");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_fn_with_return_type() {
        let result = parse("\
app Test
  screen Main
    view
      text \"hi\"
    fn double(x: int) -> int
      x * 2");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_button_with_action() {
        let result = parse("\
app Test
  screen Main
    view
      button \"Save\" .accent -> save()");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_if_in_view() {
        let result = parse("\
app Test
  screen Main
    view
      if true
        text \"yes\"
      else
        text \"no\"");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_each_in_view() {
        let result = parse("\
app Test
  screen Main
    state items: list[text] = []
    view
      each items as item
        text item");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_enum_type() {
        let result = parse("\
app Test
  model Item
    priority: enum[low, medium, high]");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_import() {
        let result = parse("\
import Button from \"@aura/ui\"

app Test
  screen Main
    view
      text \"hi\"");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        assert_eq!(result.program.unwrap().imports.len(), 1);
    }

    #[test]
    fn test_parse_multiple_screens() {
        let result = parse("\
app Test
  screen Home
    view
      text \"Home\"
  screen Settings
    view
      text \"Settings\"");
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let program = result.program.unwrap();
        let screens: Vec<_> = program.app.members.iter().filter(|m| matches!(m, AppMember::Screen(_))).collect();
        assert_eq!(screens.len(), 2);
    }
}
