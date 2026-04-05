//! # Aura Lexer
//!
//! Tokenizes `.aura` source files into a stream of tokens.
//! Uses the `logos` crate for raw tokenization, then post-processes
//! to synthesize Indent/Dedent tokens from indentation levels.
//!
//! ## Pipeline
//! ```text
//! source → logos (RawTokens) → indentation processor → Token stream
//! ```
//!
//! ## Indentation Rules
//! - 2-space indent unit (tabs are errors)
//! - Blank lines and comment-only lines are ignored for indentation
//! - Indent/Dedent tokens are inserted at the start of each significant line

mod tokens;

pub use tokens::{RawToken, Token};

use logos::Logos;

/// Maximum source file size in bytes (10 MB).
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// The indent unit in spaces.
pub const INDENT_UNIT: usize = 2;

/// Result of lexing a source file.
pub struct LexResult {
    pub tokens: Vec<Spanned<Token>>,
    pub errors: Vec<crate::errors::AuraError>,
    /// Preserved comments (trivia) with their source positions.
    pub comments: Vec<Spanned<String>>,
}

/// A token with its span in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

/// A source span (byte offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merge two spans into one covering both.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Lex a source string into tokens with Indent/Dedent.
///
/// The lexer does NOT stop at the first error — it collects all errors
/// and produces as many valid tokens as possible.
pub fn lex(source: &str) -> LexResult {
    let mut errors = Vec::new();

    // Check file size
    if source.len() > MAX_FILE_SIZE {
        errors.push(crate::errors::AuraError::new(
            crate::errors::ErrorCode::E0002,
            crate::errors::Severity::Error,
            format!(
                "File exceeds maximum size of {} MB ({} bytes)",
                MAX_FILE_SIZE / (1024 * 1024),
                source.len()
            ),
            Span::new(0, 0),
        ));
        return LexResult {
            tokens: Vec::new(),
            errors,
            comments: Vec::new(),
        };
    }

    // Phase 1: Raw tokenization with logos
    let raw_tokens = lex_raw(source, &mut errors);

    // Extract comments as trivia before indentation processing
    let mut comments = Vec::new();
    let mut non_comment_tokens = Vec::new();
    for tok in raw_tokens {
        match &tok.value {
            RawToken::SingleLineComment(s) | RawToken::MultiLineComment(s) => {
                comments.push(Spanned {
                    value: s.clone(),
                    span: tok.span,
                });
            }
            _ => non_comment_tokens.push(tok),
        }
    }

    // Phase 2: Synthesize Indent/Dedent from indentation
    let tokens = process_indentation(source, non_comment_tokens, &mut errors);

    LexResult {
        tokens,
        errors,
        comments,
    }
}

/// Phase 1: Tokenize with logos, producing raw tokens with spans.
fn lex_raw(source: &str, errors: &mut Vec<crate::errors::AuraError>) -> Vec<Spanned<RawToken>> {
    let mut raw_tokens = Vec::new();
    let mut lexer = RawToken::lexer(source);

    while let Some(result) = lexer.next() {
        let span = Span::new(lexer.span().start, lexer.span().end);
        match result {
            Ok(token) => {
                raw_tokens.push(Spanned { value: token, span });
            }
            Err(()) => {
                let slice = &source[span.start..span.end];
                // Check for tabs specifically
                if slice.contains('\t') {
                    errors.push(crate::errors::AuraError::new(
                        crate::errors::ErrorCode::E0010,
                        crate::errors::Severity::Error,
                        "Tab character found. Aura uses 2-space indentation.".to_string(),
                        span,
                    ));
                } else {
                    errors.push(crate::errors::AuraError::new(
                        crate::errors::ErrorCode::E0030,
                        crate::errors::Severity::Error,
                        format!("Unexpected character: {:?}", slice),
                        span,
                    ));
                }
            }
        }
    }

    raw_tokens
}

/// Compute the indentation level (in spaces) for a byte offset in source.
/// Returns the number of leading spaces on the line containing `offset`.
fn leading_spaces_at(source: &str, offset: usize) -> usize {
    // Find the start of the line containing this offset
    let line_start = source[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line = &source[line_start..];
    line.len() - line.trim_start_matches(' ').len()
}

/// Phase 2: Walk the raw token stream and insert Indent/Dedent tokens.
///
/// Algorithm (similar to Python's tokenizer):
/// - Maintain a stack of indentation levels, starting with [0].
/// - After each Newline token, check the indentation of the next non-blank line.
/// - If indent increases: push level, emit Indent.
/// - If indent decreases: pop levels until match, emit Dedent for each pop.
/// - At EOF: emit Dedent for each remaining level > 0.
fn process_indentation(
    source: &str,
    raw_tokens: Vec<Spanned<RawToken>>,
    errors: &mut Vec<crate::errors::AuraError>,
) -> Vec<Spanned<Token>> {
    let mut output: Vec<Spanned<Token>> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];

    // Group tokens by line. We process Newline tokens as line boundaries.
    let mut i = 0;
    let len = raw_tokens.len();

    // Skip leading newlines
    while i < len && raw_tokens[i].value == RawToken::Newline {
        i += 1;
    }

    // Process first line's indentation (should be 0 for top-level)
    if i < len {
        let first_indent = leading_spaces_at(source, raw_tokens[i].span.start);
        if first_indent != 0 {
            errors.push(crate::errors::AuraError::new(
                crate::errors::ErrorCode::E0011,
                crate::errors::Severity::Error,
                "First line must not be indented.".to_string(),
                Span::new(0, first_indent),
            ));
        }
    }

    while i < len {
        let raw = &raw_tokens[i];

        if raw.value == RawToken::Newline {
            // Emit the Newline
            output.push(Spanned {
                value: Token::Newline,
                span: raw.span,
            });

            // Skip consecutive newlines (blank lines)
            i += 1;
            while i < len && raw_tokens[i].value == RawToken::Newline {
                i += 1;
            }

            if i >= len {
                break;
            }

            // Determine indentation of next significant token
            let next_span = raw_tokens[i].span;
            let spaces = leading_spaces_at(source, next_span.start);
            let indent_span = Span::new(next_span.start.saturating_sub(spaces), next_span.start);

            // Validate indent is a multiple of INDENT_UNIT
            if spaces % INDENT_UNIT != 0 {
                errors.push(
                    crate::errors::AuraError::new(
                        crate::errors::ErrorCode::E0011,
                        crate::errors::Severity::Error,
                        format!(
                            "Inconsistent indentation: {} spaces (must be a multiple of {})",
                            spaces, INDENT_UNIT
                        ),
                        indent_span,
                    )
                    .with_help(format!(
                        "Use {} or {} spaces here.",
                        (spaces / INDENT_UNIT) * INDENT_UNIT,
                        ((spaces / INDENT_UNIT) + 1) * INDENT_UNIT
                    )),
                );
            }

            let current = *indent_stack.last().unwrap();

            if spaces > current {
                // Indent
                indent_stack.push(spaces);
                output.push(Spanned {
                    value: Token::Indent,
                    span: indent_span,
                });
            } else if spaces < current {
                // Dedent — possibly multiple levels
                while indent_stack.len() > 1 && *indent_stack.last().unwrap() > spaces {
                    indent_stack.pop();
                    output.push(Spanned {
                        value: Token::Dedent,
                        span: indent_span,
                    });
                }
                // Check we landed on a valid indent level
                if *indent_stack.last().unwrap() != spaces {
                    errors.push(crate::errors::AuraError::new(
                        crate::errors::ErrorCode::E0011,
                        crate::errors::Severity::Error,
                        format!(
                            "Dedent to {} spaces does not match any outer indentation level. \
                             Outer levels: {:?}",
                            spaces, indent_stack
                        ),
                        indent_span,
                    ));
                }
            }
            // If spaces == current: same level, no Indent/Dedent needed

            // Don't increment i — the token at i still needs to be emitted
            continue;
        }

        // Non-newline token: convert and emit
        output.push(Spanned {
            value: Token::from_raw(raw.value.clone()),
            span: raw.span,
        });
        i += 1;
    }

    // EOF: close all open indentation levels
    let eof_span = if let Some(last) = output.last() {
        Span::new(last.span.end, last.span.end)
    } else {
        Span::new(0, 0)
    };

    // Emit trailing Newline if the last token isn't one
    if output.last().map(|t| &t.value) != Some(&Token::Newline) && !output.is_empty() {
        output.push(Spanned {
            value: Token::Newline,
            span: eof_span,
        });
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        output.push(Spanned {
            value: Token::Dedent,
            span: eof_span,
        });
    }

    output
}

/// Convenience: extract just token values from a LexResult (for testing).
pub fn token_values(result: &LexResult) -> Vec<&Token> {
    result.tokens.iter().map(|t| &t.value).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: lex and return just the token types (no spans).
    fn toks(source: &str) -> Vec<Token> {
        let result = lex(source);
        assert!(
            result.errors.is_empty(),
            "Unexpected errors: {:?}",
            result.errors
        );
        result.tokens.into_iter().map(|t| t.value).collect()
    }

    /// Helper: lex and return errors.
    fn errs(source: &str) -> Vec<crate::errors::ErrorCode> {
        lex(source).errors.into_iter().map(|e| e.code).collect()
    }

    // === Basic tokenization ===

    #[test]
    fn test_empty_source() {
        let result = lex("");
        assert!(result.tokens.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_single_line() {
        let tokens = toks("app Hello");
        assert_eq!(tokens[0], Token::App);
        assert_eq!(tokens[1], Token::TypeIdent("Hello".to_string()));
        assert_eq!(tokens[2], Token::Newline);
    }

    #[test]
    fn test_keywords_vs_identifiers() {
        let tokens = toks("application screening viewed");
        // These should be identifiers, not keywords, because they're longer
        assert_eq!(tokens[0], Token::Ident("application".to_string()));
        assert_eq!(tokens[1], Token::Ident("screening".to_string()));
        assert_eq!(tokens[2], Token::Ident("viewed".to_string()));
    }

    #[test]
    fn test_numeric_literals() {
        let tokens = toks("42 3.14 1_000");
        assert_eq!(tokens[0], Token::Integer(42));
        assert_eq!(tokens[1], Token::FloatLit(3.14));
        assert_eq!(tokens[2], Token::Integer(1000));
    }

    #[test]
    fn test_string_literal() {
        let tokens = toks(r#""hello world""#);
        assert_eq!(tokens[0], Token::StringLit("hello world".to_string()));
    }

    #[test]
    fn test_operators() {
        let tokens = toks("-> => |> == != <= >= ?? ..");
        assert_eq!(tokens[0], Token::Arrow);
        assert_eq!(tokens[1], Token::FatArrow);
        assert_eq!(tokens[2], Token::Pipe);
        assert_eq!(tokens[3], Token::EqEq);
        assert_eq!(tokens[4], Token::NotEq);
        assert_eq!(tokens[5], Token::LtEq);
        assert_eq!(tokens[6], Token::GtEq);
        assert_eq!(tokens[7], Token::NilCoalesce);
        assert_eq!(tokens[8], Token::DotDot);
    }

    #[test]
    fn test_type_identifiers() {
        let tokens = toks("Todo UserProfile");
        assert_eq!(tokens[0], Token::TypeIdent("Todo".to_string()));
        assert_eq!(tokens[1], Token::TypeIdent("UserProfile".to_string()));
    }

    // === Indentation ===

    #[test]
    fn test_simple_indent() {
        let tokens = toks("app Hello\n  screen Main");
        // app Hello NEWLINE INDENT screen Main NEWLINE DEDENT
        assert_eq!(tokens[0], Token::App);
        assert_eq!(tokens[1], Token::TypeIdent("Hello".to_string()));
        assert_eq!(tokens[2], Token::Newline);
        assert_eq!(tokens[3], Token::Indent);
        assert_eq!(tokens[4], Token::Screen);
        assert_eq!(tokens[5], Token::TypeIdent("Main".to_string()));
    }

    #[test]
    fn test_indent_dedent() {
        let tokens = toks("app Hello\n  screen Main\nconst X = 1");
        // app Hello NL INDENT screen Main NL DEDENT const X = 1 NL
        let indent_count = tokens.iter().filter(|t| **t == Token::Indent).count();
        let dedent_count = tokens.iter().filter(|t| **t == Token::Dedent).count();
        assert_eq!(indent_count, 1);
        assert_eq!(dedent_count, 1);
    }

    #[test]
    fn test_nested_indent() {
        let tokens = toks("app A\n  screen B\n    view\n      text C");
        let indent_count = tokens.iter().filter(|t| **t == Token::Indent).count();
        assert_eq!(indent_count, 3); // 3 nested levels
    }

    #[test]
    fn test_eof_dedents() {
        let tokens = toks("app A\n  screen B\n    view");
        // At EOF, all open indents should be closed
        let indent_count = tokens.iter().filter(|t| **t == Token::Indent).count();
        let dedent_count = tokens.iter().filter(|t| **t == Token::Dedent).count();
        assert_eq!(indent_count, dedent_count, "Indent/Dedent must be balanced");
    }

    #[test]
    fn test_multiple_dedent() {
        let tokens = toks("app A\n  screen B\n    view\n      text C\nconst X = 1");
        // Going from indent 3 back to 0 should produce 3 dedents
        let indent_count = tokens.iter().filter(|t| **t == Token::Indent).count();
        let dedent_count = tokens.iter().filter(|t| **t == Token::Dedent).count();
        assert_eq!(indent_count, 3);
        assert_eq!(dedent_count, 3);
    }

    #[test]
    fn test_blank_lines_ignored() {
        let tokens = toks("app A\n\n  screen B\n\n    view");
        // Blank lines should not affect indentation
        let indent_count = tokens.iter().filter(|t| **t == Token::Indent).count();
        assert_eq!(indent_count, 2);
    }

    #[test]
    fn test_same_level_no_indent_dedent() {
        let tokens = toks("app A\n  screen B\n  model C");
        // screen and model are at same level — only 1 indent, no extra dedent between them
        let indent_count = tokens.iter().filter(|t| **t == Token::Indent).count();
        assert_eq!(indent_count, 1);
    }

    // === Error cases ===

    #[test]
    fn test_rejects_oversized_file() {
        let source = "a".repeat(MAX_FILE_SIZE + 1);
        let result = lex(&source);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, crate::errors::ErrorCode::E0002);
    }

    #[test]
    fn test_rejects_tabs() {
        let codes = errs("\tapp Hello");
        assert!(codes.contains(&crate::errors::ErrorCode::E0010));
    }

    #[test]
    fn test_odd_indentation_error() {
        let codes = errs("app A\n   screen B");
        // 3 spaces is not a multiple of 2
        assert!(codes.contains(&crate::errors::ErrorCode::E0011));
    }

    // === Comments ===

    #[test]
    fn test_single_line_comment_skipped() {
        let tokens = toks("app Hello // this is a comment");
        assert_eq!(tokens[0], Token::App);
        assert_eq!(tokens[1], Token::TypeIdent("Hello".to_string()));
        assert_eq!(tokens[2], Token::Newline);
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_multi_line_comment_skipped() {
        let tokens = toks("app /* comment */ Hello");
        assert_eq!(tokens[0], Token::App);
        assert_eq!(tokens[1], Token::TypeIdent("Hello".to_string()));
    }

    // === Design token adjacency ===

    #[test]
    fn test_dot_tokens() {
        let tokens = toks("column .md .accent");
        assert_eq!(tokens[0], Token::Column);
        assert_eq!(tokens[1], Token::Dot);
        assert_eq!(tokens[2], Token::Ident("md".to_string()));
        assert_eq!(tokens[3], Token::Dot);
        assert_eq!(tokens[4], Token::Ident("accent".to_string()));
    }

    // === Real Aura program ===

    #[test]
    fn test_minimal_program() {
        let source = "\
app Hello
  screen Main
    view
      text \"Hello, Aura!\"";
        let result = lex(source);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

        let tokens: Vec<_> = result.tokens.iter().map(|t| &t.value).collect();
        // Verify key structure tokens are present
        assert!(tokens.contains(&&Token::App));
        assert!(tokens.contains(&&Token::Screen));
        assert!(tokens.contains(&&Token::View));
        assert!(tokens.contains(&&Token::Text));
        assert!(tokens.contains(&&Token::StringLit("Hello, Aura!".to_string())));

        // Verify indent/dedent balance
        let indents = tokens.iter().filter(|t| ***t == Token::Indent).count();
        let dedents = tokens.iter().filter(|t| ***t == Token::Dedent).count();
        assert_eq!(indents, dedents, "Indent/Dedent must balance");
        assert_eq!(indents, 3); // 3 levels of nesting
    }

    #[test]
    fn test_model_with_fields() {
        let source = "\
model Todo
  title: text
  done: bool = false";
        let result = lex(source);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

        let tokens: Vec<_> = result.tokens.iter().map(|t| &t.value).collect();
        assert!(tokens.contains(&&Token::Model));
        assert!(tokens.contains(&&Token::Colon));
        assert!(tokens.contains(&&Token::Text));
        assert!(tokens.contains(&&Token::Bool));
        assert!(tokens.contains(&&Token::False));
    }

    #[test]
    fn test_action_with_arrow() {
        let tokens = toks("button \"Save\" .accent -> save()");
        assert_eq!(tokens[0], Token::Button);
        assert_eq!(tokens[1], Token::StringLit("Save".to_string()));
        assert_eq!(tokens[2], Token::Dot);
        assert_eq!(tokens[3], Token::Ident("accent".to_string()));
        assert_eq!(tokens[4], Token::Arrow);
        assert_eq!(tokens[5], Token::Ident("save".to_string()));
        assert_eq!(tokens[6], Token::LParen);
        assert_eq!(tokens[7], Token::RParen);
    }
}
