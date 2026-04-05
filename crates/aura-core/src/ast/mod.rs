//! # Aura AST (Abstract Syntax Tree)
//!
//! The AST is the direct output of the parser. It preserves the full structure
//! of the source code, including location information for error reporting.

use crate::lexer::Span;
/// A complete Aura program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub imports: Vec<ImportDecl>,
    pub app: AppDecl,
}

/// An import declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub spec: ImportSpec,
    pub source: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportSpec {
    /// `import Foo from "..."`
    Named(String),
    /// `import { Foo, Bar } from "..."`
    Destructured(Vec<String>),
    /// `import * as Foo from "..."`
    Wildcard(String),
}

/// The top-level app declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct AppDecl {
    pub name: String,
    pub members: Vec<AppMember>,
    pub span: Span,
}

/// A member of an app declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMember {
    ThemeRef(ThemeRef),
    NavigationDecl(NavigationDecl),
    RouteDecl(RouteDecl),
    Model(ModelDecl),
    Screen(ScreenDecl),
    Component(ComponentDecl),
    Style(StyleDecl),
    Const(ConstDecl),
    Fn(FnDecl),
    State(StateDecl),
    Theme(ThemeDecl),
}

/// A theme reference: `theme: modern.dark`
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeRef {
    pub value: Expr,
    pub span: Span,
}

/// Navigation mode declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct NavigationDecl {
    pub mode: String, // "stack", "tabs"
    pub span: Span,
}

/// Route declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteDecl {
    pub pattern: String,
    pub screen: String,
    pub params: Vec<(String, Expr)>,
    pub span: Span,
}

/// A model (data structure) declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub span: Span,
}

/// A field in a model.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub type_expr: TypeExpr,
    pub default: Option<Expr>,
    pub span: Span,
}

/// A type expression in source code.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// Simple name: `text`, `int`, `Todo`, `secret`
    Named(String, Span),
    /// Collection: `list[Todo]`, `map[text, int]`
    Collection(String, Vec<TypeExpr>, Span),
    /// Optional: `optional[text]`
    Optional(Box<TypeExpr>, Span),
    /// Enum: `enum[low, medium, high]`
    Enum(Vec<EnumVariantExpr>, Span),
    /// Function: `fn(int, int) -> int`
    Function(Vec<TypeExpr>, Option<Box<TypeExpr>>, Span),
    /// Action: `action` or `action(text)`
    Action(Vec<TypeExpr>, Span),
    /// Union: `text | int | bool`
    Union(Vec<TypeExpr>, Span),
}

/// An enum variant in source.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantExpr {
    pub name: String,
    pub fields: Vec<Param>,
    pub span: Span,
}

/// A screen declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub modifiers: Vec<ScreenModifier>,
    pub members: Vec<ScreenMember>,
    pub span: Span,
}

/// Screen modifiers (tab icon, label).
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenModifier {
    Tab(String),
    Label(String),
}

/// A member of a screen.
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenMember {
    State(StateDecl),
    View(ViewDecl),
    Action(ActionDecl),
    Fn(FnDecl),
    On(OnDecl),
    Style(StyleDecl),
}

/// A component declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDecl {
    pub name: String,
    pub props: Vec<Param>,
    pub members: Vec<ScreenMember>, // Same member types as screen
    pub span: Span,
}

/// A parameter (used in functions, actions, components, screens).
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_expr: TypeExpr,
    pub default: Option<Expr>,
    pub span: Span,
}

/// A state declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct StateDecl {
    pub name: String,
    pub type_expr: TypeExpr,
    pub default: Option<Expr>,
    pub span: Span,
}

/// A view declaration (the UI tree).
#[derive(Debug, Clone, PartialEq)]
pub struct ViewDecl {
    pub body: Vec<ViewElement>,
    pub span: Span,
}

/// An element in a view tree.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewElement {
    /// Layout container: column, row, stack, grid, scroll, wrap
    Layout(LayoutElement),
    /// Widget: text, heading, image, icon, badge, etc.
    Widget(WidgetElement),
    /// Input: textfield, checkbox, toggle, slider, etc.
    Input(InputElement),
    /// Button (special: has action trigger)
    Button(ButtonElement),
    /// Conditional: if/else
    If(IfView),
    /// Loop: each ... as ...
    Each(EachView),
    /// Pattern match: when ... is ...
    When(WhenView),
    /// Component reference
    ComponentRef(ComponentRef),
    /// Spacer
    Spacer(Span),
    /// Divider
    Divider(Vec<DesignToken>, Span),
    /// Slot (content projection)
    Slot(Span),
}

/// A layout element (column, row, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutElement {
    pub kind: LayoutKind,
    pub tokens: Vec<DesignToken>,
    pub props: Vec<PropAssign>,
    pub children: Vec<ViewElement>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutKind {
    Column,
    Row,
    Stack,
    Grid,
    Scroll,
    Wrap,
}

/// A widget element (text, heading, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetElement {
    pub kind: WidgetKind,
    pub args: Vec<Expr>,
    pub tokens: Vec<DesignToken>,
    pub props: Vec<PropAssign>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    Text,
    Heading,
    Image,
    Icon,
    Badge,
    Progress,
    Avatar,
}

/// An input element.
#[derive(Debug, Clone, PartialEq)]
pub struct InputElement {
    pub kind: InputKind,
    pub binding: String,
    pub tokens: Vec<DesignToken>,
    pub props: Vec<PropAssign>,
    pub action: Option<ActionExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    TextField,
    TextArea,
    Checkbox,
    Toggle,
    Slider,
    Picker,
    DatePicker,
    Segmented,
    Stepper,
}

/// A button element.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonElement {
    pub style: Option<String>,
    pub label: Expr,
    pub tokens: Vec<DesignToken>,
    pub props: Vec<PropAssign>,
    pub action: ActionExpr,
    pub span: Span,
}

/// A design token (e.g., .md, .accent, .bold).
#[derive(Debug, Clone, PartialEq)]
pub struct DesignToken {
    pub segments: Vec<String>, // ["gap", "md"] for gap.md
    pub span: Span,
}

/// A property assignment (e.g., placeholder: "Search").
#[derive(Debug, Clone, PartialEq)]
pub struct PropAssign {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

/// An action expression (right side of `->` in views).
#[derive(Debug, Clone, PartialEq)]
pub enum ActionExpr {
    Call(String, Vec<Expr>, Span),
    Navigate(NavigateExpr),
    Lambda(Vec<Param>, Box<Expr>, Span),
}

/// A navigate expression.
#[derive(Debug, Clone, PartialEq)]
pub enum NavigateExpr {
    To(Expr, Span),
    Back(Span),
    Root(Span),
    Replace(Expr, Span),
    Modal(Expr, Span),
    Dismiss(Span),
}

/// Conditional view.
#[derive(Debug, Clone, PartialEq)]
pub struct IfView {
    pub condition: Expr,
    pub then_body: Vec<ViewElement>,
    pub else_body: Option<Vec<ViewElement>>,
    pub span: Span,
}

/// Loop view.
#[derive(Debug, Clone, PartialEq)]
pub struct EachView {
    pub iterable: Expr,
    pub item_name: String,
    pub index_name: Option<String>,
    pub body: Vec<ViewElement>,
    pub span: Span,
}

/// Pattern-matching view.
#[derive(Debug, Clone, PartialEq)]
pub struct WhenView {
    pub expression: Expr,
    pub branches: Vec<WhenBranch>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenBranch {
    pub pattern: Pattern,
    pub body: Vec<ViewElement>,
    pub span: Span,
}

/// A pattern for matching.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Expr),
    Identifier(String, Span),
    EnumVariant(String, Span),
    Some(String, Span),
    Nil(Span),
    Constructor(String, Vec<(String, String)>, Span),
}

/// Component reference in a view.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentRef {
    pub name: String,
    pub args: Vec<(String, Expr)>,
    pub children: Vec<ViewElement>,
    pub span: Span,
}

/// An action declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// A function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// An event handler declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct OnDecl {
    pub event: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// A style declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct StyleDecl {
    pub name: String,
    pub tokens: Vec<DesignToken>,
    pub props: Vec<PropAssign>,
    pub span: Span,
}

/// A constant declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: String,
    pub type_expr: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

/// A theme declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeDecl {
    pub name: String,
    pub properties: Vec<ThemeProperty>,
    pub palette: Vec<PaletteEntry>,
    pub variants: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeProperty {
    pub path: Vec<String>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteEntry {
    pub path: Vec<String>,
    pub value: String,
    pub span: Span,
}

/// A statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Variable assignment: `x = expr`
    Assign(String, Expr, Span),
    /// Let binding: `let x = expr`
    Let(String, Option<TypeExpr>, Expr, Span),
    /// If statement
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>, Span),
    /// When statement
    When(Expr, Vec<WhenStmtBranch>, Span),
    /// Navigate
    Navigate(NavigateExpr),
    /// Emit event
    Emit(String, Vec<Expr>, Span),
    /// Return
    Return(Option<Expr>, Span),
    /// Expression statement (function call, etc.)
    Expr(Expr, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenStmtBranch {
    pub pattern: Pattern,
    pub body: StmtOrExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtOrExpr {
    Stmt(Vec<Stmt>),
    Expr(Expr),
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    IntLit(i64, Span),
    /// Float literal
    FloatLit(f64, Span),
    /// String literal (may contain interpolations)
    StringLit(String, Span),
    /// Percent literal
    PercentLit(f64, Span),
    /// Boolean literal
    BoolLit(bool, Span),
    /// Nil literal
    Nil(Span),
    /// Variable reference
    Var(String, Span),
    /// Member access: `a.b`
    MemberAccess(Box<Expr>, String, Span),
    /// Function call: `f(a, b)`
    Call(Box<Expr>, Vec<Expr>, Span),
    /// Named argument call: `F(x: 1, y: 2)`
    NamedCall(Box<Expr>, Vec<(String, Expr)>, Span),
    /// Index access: `a[i]`
    Index(Box<Expr>, Box<Expr>, Span),
    /// Binary operation: `a + b`
    BinOp(Box<Expr>, BinOp, Box<Expr>, Span),
    /// Unary operation: `not x`, `-x`
    UnaryOp(UnaryOp, Box<Expr>, Span),
    /// Lambda: `x => x + 1`
    Lambda(Vec<Param>, Box<Expr>, Span),
    /// Constructor: `Todo(title: "Buy milk")`
    Constructor(String, Vec<(String, Expr)>, Span),
    /// Pipe: `x |> f |> g`
    Pipe(Box<Expr>, Box<Expr>, Span),
    /// Conditional expression: `if a then b else c`
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    /// Nil coalesce: `a ?? b`
    NilCoalesce(Box<Expr>, Box<Expr>, Span),
    /// Design token reference: `.accent`, `.bold`
    DesignToken(Vec<String>, Span),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Range, // ..
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}
