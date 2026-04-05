//! # Aura Error System
//!
//! Errors designed for two audiences: human developers and AI coding agents.
//! Features:
//! - Error poisoning (suppress cascade errors from one root cause)
//! - Confidence-scored fix suggestions (AI agents auto-apply high-confidence fixes)
//! - Structured JSON output for machine consumption

use crate::lexer::Span;

/// An Aura compiler error or warning.
#[derive(Debug, Clone)]
pub struct AuraError {
    /// Error code (e.g., E0103)
    pub code: ErrorCode,
    /// Severity level
    pub severity: Severity,
    /// Human-readable error message
    pub message: String,
    /// Location in source
    pub span: Span,
    /// Human-readable help text
    pub help: Option<String>,
    /// Machine-applicable fix with confidence score
    pub fix: Option<Fix>,
    /// Number of downstream errors suppressed by poisoning
    pub suppressed: usize,
    /// Whether this error's AST node is poisoned
    pub poisoned: bool,
}

impl AuraError {
    pub fn new(code: ErrorCode, severity: Severity, message: String, span: Span) -> Self {
        Self {
            code,
            severity,
            message,
            span,
            help: None,
            fix: None,
            suppressed: 0,
            poisoned: false,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }
}

/// Severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Prevents compilation.
    Error,
    /// Compiles but likely wrong.
    Warning,
    /// Suggestion for improvement.
    Info,
}

/// A machine-applicable fix suggestion.
#[derive(Debug, Clone)]
pub struct Fix {
    /// What to do
    pub action: FixAction,
    /// Where in the source to apply the fix
    pub span: Span,
    /// The replacement text
    pub replacement: String,
    /// Confidence score (0.0 - 1.0)
    /// - 0.95-1.0: auto-apply
    /// - 0.8-0.95: auto-apply with log
    /// - 0.5-0.8: present for confirmation
    /// - 0.0-0.5: present multiple alternatives
    pub confidence: f64,
}

/// The kind of fix action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixAction {
    /// Replace the span with new text
    Replace,
    /// Insert text at the span's start position
    Insert,
    /// Delete the span
    Delete,
}

/// Error codes organized by category.
///
/// See spec/language.md Section 12.6 for the complete reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Lexer errors (E0001-E0099)
    E0001, // Invalid UTF-8
    E0002, // File too large
    E0010, // Tab character
    E0011, // Inconsistent indentation
    E0020, // Unterminated string
    E0021, // Invalid escape sequence
    E0030, // Invalid numeric literal / unexpected character

    // Type errors (E0100-E0199)
    E0100, // Nil for non-optional
    E0101, // Type mismatch
    E0102, // Cannot infer type
    E0103, // Undefined variable
    E0104, // Undefined type
    E0105, // Duplicate field
    E0106, // Missing required field
    E0107, // Circular dependency
    E0108, // Cannot call non-function
    E0109, // Wrong arg count
    E0110, // Optional access without nil check

    // Security type errors (E0200-E0299)
    E0200, // Secret in API response
    E0201, // Secret in log
    E0202, // Secret in interpolation
    E0203, // Secret in == comparison
    E0204, // Secret in serialization
    E0210, // Unsanitized text in view
    E0220, // Invalid email format
    E0230, // Invalid URL format
    E0240, // Expired token access

    // State errors (E0300-E0399)
    E0300, // State mutation outside action
    E0301, // State mutation in view
    E0302, // State mutation in pure function
    E0310, // Undeclared state

    // Design errors (E0400-E0499)
    E0400, // Unknown design token
    E0401, // Incompatible tokens
    E0410, // Theme not found
    E0411, // Theme missing field

    // Platform errors (E0500-E0599)
    E0500, // API not available
    E0501, // Missing platform guard
    E0510, // SDK not installed

    // Navigation errors (E0600-E0699)
    E0600, // Unknown screen
    E0601, // Missing screen param
    E0602, // Circular navigation

    // Parse errors (E0700-E0799)
    E0700, // Unexpected token
    E0701, // Expected expression
    E0702, // Expected identifier
    E0710, // Max nesting depth
    E0720, // Unexpected EOF

    // Internal errors (E0900-E0999)
    E0900, // Internal compiler error
    E0901, // Codegen failure
    E0902, // IR lowering failure
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Compute edit distance between two strings for "did you mean?" suggestions.
pub fn suggest_similar<'a>(
    name: &str,
    candidates: &[&'a str],
    max_distance: usize,
) -> Option<(&'a str, f64)> {
    let mut best: Option<(&str, usize)> = None;

    for &candidate in candidates {
        let dist = strsim::levenshtein(name, candidate);
        if dist <= max_distance {
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((candidate, dist));
            }
        }
    }

    best.map(|(name, dist)| {
        // Confidence: closer edit distance → higher confidence
        let max_len = name.len().max(dist);
        let confidence = if max_len == 0 {
            1.0
        } else {
            1.0 - (dist as f64 / max_len as f64)
        };
        (name, confidence.clamp(0.0, 1.0))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_similar_exact_match() {
        let result = suggest_similar("todos", &["todos", "filter", "count"], 2);
        assert_eq!(result.map(|(name, _)| name), Some("todos"));
    }

    #[test]
    fn test_suggest_similar_typo() {
        let result = suggest_similar("todoos", &["todos", "filter", "count"], 2);
        assert_eq!(result.map(|(name, _)| name), Some("todos"));
    }

    #[test]
    fn test_suggest_similar_no_match() {
        let result = suggest_similar("xyz", &["todos", "filter", "count"], 2);
        assert!(result.is_none());
    }

    #[test]
    fn test_confidence_scoring() {
        let result = suggest_similar("todoos", &["todos"], 2);
        let (_, confidence) = result.unwrap();
        assert!(
            confidence > 0.7,
            "Expected confidence > 0.7, got {}",
            confidence
        );
    }
}
