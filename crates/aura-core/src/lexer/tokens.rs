//! Token definitions for the Aura lexer.

use logos::Logos;

/// Raw token produced by logos. Does not include Indent/Dedent —
/// those are synthesized during post-processing in `mod.rs`.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \r]+")]
pub enum RawToken {
    // === Reserved Words ===
    #[token("app")]
    App,
    #[token("screen")]
    Screen,
    #[token("view")]
    View,
    #[token("model")]
    Model,
    #[token("state")]
    State,
    #[token("action")]
    Action,
    #[token("each")]
    Each,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("when")]
    When,
    #[token("is")]
    Is,
    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("as")]
    As,
    #[token("theme")]
    Theme,
    #[token("style")]
    Style,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("enum")]
    Enum,
    #[token("fn")]
    Fn,
    #[token("return")]
    Return,
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("list")]
    List,
    #[token("map")]
    Map,
    #[token("set")]
    Set,
    #[token("optional")]
    Optional,
    #[token("component")]
    Component,
    #[token("navigate")]
    Navigate,
    #[token("back")]
    Back,
    #[token("emit")]
    Emit,
    #[token("on")]
    On,
    #[token("animate")]
    Animate,
    #[token("with")]
    With,
    #[token("where")]
    Where,
    #[token("in")]
    In,
    #[token("then")]
    Then,
    #[token("some")]
    Some_,
    #[token("slot")]
    Slot,
    #[token("api")]
    Api,
    #[token("route")]
    Route,
    #[token("platform")]
    Platform,
    #[token("palette")]
    Palette,
    #[token("variants")]
    Variants,

    // === Layout Keywords ===
    #[token("column")]
    Column,
    #[token("row")]
    Row,
    #[token("stack")]
    Stack,
    #[token("grid")]
    Grid,
    #[token("scroll")]
    Scroll,
    #[token("wrap")]
    Wrap,

    // === Widget Keywords ===
    #[token("text")]
    Text,
    #[token("heading")]
    Heading,
    #[token("image")]
    Image,
    #[token("icon")]
    Icon,
    #[token("badge")]
    Badge,
    #[token("divider")]
    Divider,
    #[token("spacer")]
    Spacer,
    #[token("progress")]
    Progress,
    #[token("avatar")]
    Avatar,
    #[token("button")]
    Button,
    #[token("fab")]
    Fab,

    // === Input Keywords ===
    #[token("textfield")]
    TextField,
    #[token("textarea")]
    TextArea,
    #[token("checkbox")]
    Checkbox,
    #[token("toggle")]
    Toggle,
    #[token("slider")]
    Slider,
    #[token("picker")]
    Picker,
    #[token("datepicker")]
    DatePicker,
    #[token("segmented")]
    Segmented,
    #[token("stepper")]
    Stepper,

    // === Type Keywords (Security) ===
    #[token("secret")]
    Secret,
    #[token("sanitized")]
    Sanitized,
    #[token("email")]
    Email,
    #[token("url")]
    Url,
    #[token("token")]
    TokenType,

    // === Type Keywords (Primitive) ===
    #[token("int")]
    Int,
    #[token("float")]
    Float,
    #[token("bool")]
    Bool,
    #[token("timestamp")]
    Timestamp,
    #[token("duration")]
    Duration,
    #[token("percent")]
    Percent,

    // === Operators (order matters: longer first for logos) ===
    #[token("...")]
    Spread,
    #[token("..")]
    DotDot,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("|>")]
    Pipe,
    #[token("|")]
    Bar,
    #[token("::")]
    ColonColon,
    #[token("??")]
    NilCoalesce,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Modulo,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,

    // === Punctuation ===
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    // === Literals ===
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<f64>().ok(), priority = 3)]
    FloatLit(f64),

    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok(), priority = 2)]
    Integer(i64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLit(String),

    // === Identifiers ===
    #[regex(r"[a-z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 1)]
    Ident(String),

    #[regex(r"[A-Z][a-zA-Z0-9]*", |lex| lex.slice().to_string(), priority = 1)]
    TypeIdent(String),

    // === Whitespace & Structure ===
    #[token("\n")]
    Newline,

    // === Comments (captured for trivia preservation) ===
    #[regex(r"//[^\n]*", |lex| lex.slice().to_string())]
    SingleLineComment(String),

    #[regex(r"/\*([^*]|\*[^/])*\*/", |lex| lex.slice().to_string())]
    MultiLineComment(String),
}

/// The final token type exposed to the parser. Includes synthesized
/// Indent/Dedent tokens that RawToken does not have.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Structure
    Indent,
    Dedent,
    Newline,
    /// Preserved comment (trivia)
    Comment(String),

    // Keywords
    App,
    Screen,
    View,
    Model,
    State,
    Action,
    Each,
    If,
    Else,
    When,
    Is,
    Import,
    From,
    As,
    Theme,
    Style,
    True,
    False,
    Nil,
    And,
    Or,
    Not,
    Enum,
    Fn,
    Return,
    Let,
    Const,
    List,
    Map,
    Set,
    Optional,
    Component,
    Navigate,
    Back,
    Emit,
    On,
    Animate,
    With,
    Where,
    In,
    Then,
    Some_,
    Slot,
    Api,
    Route,
    Platform,
    Palette,
    Variants,

    // Layout
    Column,
    Row,
    Stack,
    Grid,
    Scroll,
    Wrap,

    // Widgets
    Text,
    Heading,
    Image,
    Icon,
    Badge,
    Divider,
    Spacer,
    Progress,
    Avatar,
    Button,
    Fab,

    // Inputs
    TextField,
    TextArea,
    Checkbox,
    Toggle,
    Slider,
    Picker,
    DatePicker,
    Segmented,
    Stepper,

    // Security types
    Secret,
    Sanitized,
    Email,
    Url,
    TokenType,

    // Primitive types
    Int,
    Float,
    Bool,
    Timestamp,
    Duration,
    Percent,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Modulo,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Eq,
    Arrow,
    FatArrow,
    Pipe,
    Bar,
    ColonColon,
    DotDot,
    Spread,
    NilCoalesce,

    // Punctuation
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Dot,

    // Literals
    Integer(i64),
    FloatLit(f64),
    StringLit(String),

    // Identifiers
    Ident(String),
    TypeIdent(String),
}

impl Token {
    /// Convert a RawToken into a Token.
    pub fn from_raw(raw: RawToken) -> Self {
        match raw {
            RawToken::App => Token::App,
            RawToken::Screen => Token::Screen,
            RawToken::View => Token::View,
            RawToken::Model => Token::Model,
            RawToken::State => Token::State,
            RawToken::Action => Token::Action,
            RawToken::Each => Token::Each,
            RawToken::If => Token::If,
            RawToken::Else => Token::Else,
            RawToken::When => Token::When,
            RawToken::Is => Token::Is,
            RawToken::Import => Token::Import,
            RawToken::From => Token::From,
            RawToken::As => Token::As,
            RawToken::Theme => Token::Theme,
            RawToken::Style => Token::Style,
            RawToken::True => Token::True,
            RawToken::False => Token::False,
            RawToken::Nil => Token::Nil,
            RawToken::And => Token::And,
            RawToken::Or => Token::Or,
            RawToken::Not => Token::Not,
            RawToken::Enum => Token::Enum,
            RawToken::Fn => Token::Fn,
            RawToken::Return => Token::Return,
            RawToken::Let => Token::Let,
            RawToken::Const => Token::Const,
            RawToken::List => Token::List,
            RawToken::Map => Token::Map,
            RawToken::Set => Token::Set,
            RawToken::Optional => Token::Optional,
            RawToken::Component => Token::Component,
            RawToken::Navigate => Token::Navigate,
            RawToken::Back => Token::Back,
            RawToken::Emit => Token::Emit,
            RawToken::On => Token::On,
            RawToken::Animate => Token::Animate,
            RawToken::With => Token::With,
            RawToken::Where => Token::Where,
            RawToken::In => Token::In,
            RawToken::Then => Token::Then,
            RawToken::Some_ => Token::Some_,
            RawToken::Slot => Token::Slot,
            RawToken::Api => Token::Api,
            RawToken::Route => Token::Route,
            RawToken::Platform => Token::Platform,
            RawToken::Palette => Token::Palette,
            RawToken::Variants => Token::Variants,
            RawToken::Column => Token::Column,
            RawToken::Row => Token::Row,
            RawToken::Stack => Token::Stack,
            RawToken::Grid => Token::Grid,
            RawToken::Scroll => Token::Scroll,
            RawToken::Wrap => Token::Wrap,
            RawToken::Text => Token::Text,
            RawToken::Heading => Token::Heading,
            RawToken::Image => Token::Image,
            RawToken::Icon => Token::Icon,
            RawToken::Badge => Token::Badge,
            RawToken::Divider => Token::Divider,
            RawToken::Spacer => Token::Spacer,
            RawToken::Progress => Token::Progress,
            RawToken::Avatar => Token::Avatar,
            RawToken::Button => Token::Button,
            RawToken::Fab => Token::Fab,
            RawToken::TextField => Token::TextField,
            RawToken::TextArea => Token::TextArea,
            RawToken::Checkbox => Token::Checkbox,
            RawToken::Toggle => Token::Toggle,
            RawToken::Slider => Token::Slider,
            RawToken::Picker => Token::Picker,
            RawToken::DatePicker => Token::DatePicker,
            RawToken::Segmented => Token::Segmented,
            RawToken::Stepper => Token::Stepper,
            RawToken::Secret => Token::Secret,
            RawToken::Sanitized => Token::Sanitized,
            RawToken::Email => Token::Email,
            RawToken::Url => Token::Url,
            RawToken::TokenType => Token::TokenType,
            RawToken::Int => Token::Int,
            RawToken::Float => Token::Float,
            RawToken::Bool => Token::Bool,
            RawToken::Timestamp => Token::Timestamp,
            RawToken::Duration => Token::Duration,
            RawToken::Percent => Token::Percent,
            RawToken::Plus => Token::Plus,
            RawToken::Minus => Token::Minus,
            RawToken::Star => Token::Star,
            RawToken::Slash => Token::Slash,
            RawToken::Modulo => Token::Modulo,
            RawToken::EqEq => Token::EqEq,
            RawToken::NotEq => Token::NotEq,
            RawToken::Lt => Token::Lt,
            RawToken::Gt => Token::Gt,
            RawToken::LtEq => Token::LtEq,
            RawToken::GtEq => Token::GtEq,
            RawToken::Eq => Token::Eq,
            RawToken::Arrow => Token::Arrow,
            RawToken::FatArrow => Token::FatArrow,
            RawToken::Pipe => Token::Pipe,
            RawToken::Bar => Token::Bar,
            RawToken::ColonColon => Token::ColonColon,
            RawToken::DotDot => Token::DotDot,
            RawToken::Spread => Token::Spread,
            RawToken::NilCoalesce => Token::NilCoalesce,
            RawToken::LParen => Token::LParen,
            RawToken::RParen => Token::RParen,
            RawToken::LBracket => Token::LBracket,
            RawToken::RBracket => Token::RBracket,
            RawToken::LBrace => Token::LBrace,
            RawToken::RBrace => Token::RBrace,
            RawToken::Colon => Token::Colon,
            RawToken::Comma => Token::Comma,
            RawToken::Dot => Token::Dot,
            RawToken::Integer(n) => Token::Integer(n),
            RawToken::FloatLit(f) => Token::FloatLit(f),
            RawToken::StringLit(s) => Token::StringLit(s),
            RawToken::Ident(s) => Token::Ident(s),
            RawToken::TypeIdent(s) => Token::TypeIdent(s),
            RawToken::Newline => Token::Newline,
            RawToken::SingleLineComment(s) => Token::Comment(s),
            RawToken::MultiLineComment(s) => Token::Comment(s),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Indent => write!(f, "INDENT"),
            Token::Dedent => write!(f, "DEDENT"),
            Token::Newline => write!(f, "NEWLINE"),
            Token::App => write!(f, "app"),
            Token::Screen => write!(f, "screen"),
            Token::View => write!(f, "view"),
            Token::Model => write!(f, "model"),
            Token::State => write!(f, "state"),
            Token::Action => write!(f, "action"),
            Token::Component => write!(f, "component"),
            Token::Fn => write!(f, "fn"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Each => write!(f, "each"),
            Token::When => write!(f, "when"),
            Token::Is => write!(f, "is"),
            Token::Let => write!(f, "let"),
            Token::Const => write!(f, "const"),
            Token::Import => write!(f, "import"),
            Token::From => write!(f, "from"),
            Token::As => write!(f, "as"),
            Token::Return => write!(f, "return"),
            Token::Navigate => write!(f, "navigate"),
            Token::Theme => write!(f, "theme"),
            Token::Ident(s) | Token::TypeIdent(s) => write!(f, "{}", s),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            Token::Integer(n) => write!(f, "{}", n),
            Token::FloatLit(n) => write!(f, "{}", n),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::Pipe => write!(f, "|>"),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            _ => write!(f, "{:?}", self),
        }
    }
}
