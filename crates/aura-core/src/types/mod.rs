//! # Aura Type System
//!
//! Structural type system with inference and built-in security types.
//! Types are inferred where possible, annotated where required.

use serde::{Deserialize, Serialize};

/// The top-level type representation in Aura.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuraType {
    /// Primitive types (text, int, float, bool, timestamp, duration, percent)
    Primitive(PrimitiveType),
    /// Security types (secret, sanitized, email, url, token)
    Security(SecurityType),
    /// List collection
    List(Box<AuraType>),
    /// Set collection
    Set(Box<AuraType>),
    /// Map collection
    Map(Box<AuraType>, Box<AuraType>),
    /// Optional wrapper
    Optional(Box<AuraType>),
    /// Enum (inline or named)
    Enum(Vec<EnumVariant>),
    /// Named model type
    Named(String),
    /// Function type
    Function(FunctionType),
    /// Union type: A | B (value is one of several types)
    Union(Vec<AuraType>),
    /// Action type (for event handlers)
    Action(Vec<AuraType>),
    /// Generic type parameter (e.g., T in fn first[T](list[T]) -> T)
    TypeParam(String),
    /// Conditional type: if T is subtype of U, resolve to A, else B.
    /// Like TypeScript's `T extends U ? A : B`
    Conditional {
        check: Box<AuraType>,
        extends: Box<AuraType>,
        then_type: Box<AuraType>,
        else_type: Box<AuraType>,
    },
    /// Type variable (for inference)
    Var(usize),
    /// Error/poison type (for error recovery — propagates without further errors)
    Poison,
}

/// Primitive types built into the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    Text,
    Int,
    Float,
    Bool,
    Timestamp,
    Duration,
    Percent,
}

/// Security types with compile-time enforcement.
///
/// These are NOT library types — they are grammar-level types with
/// special compiler rules (see spec Section 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityType {
    /// Auto-hashed, never logged/serialized, no == comparison
    Secret,
    /// XSS-safe, length-limited, validated
    Sanitized,
    /// Format-validated email address
    Email,
    /// Format-validated URL (javascript:/data: blocked by default)
    Url,
    /// Expiring auth token, never logged/serialized
    Token,
}

/// An enum variant with optional associated data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<(String, AuraType)>,
}

/// A function type: parameter types → return type, with optional type parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionType {
    /// Type parameters (e.g., [T, U] in fn map[T, U])
    pub type_params: Vec<String>,
    pub params: Vec<AuraType>,
    pub return_type: Box<AuraType>,
}

impl AuraType {
    /// Returns true if this type is poisoned (error recovery).
    pub fn is_poison(&self) -> bool {
        matches!(self, AuraType::Poison)
    }

    /// Returns true if this type is optional.
    pub fn is_optional(&self) -> bool {
        matches!(self, AuraType::Optional(_))
    }

    /// Returns true if this is a security type with logging restrictions.
    pub fn is_no_log(&self) -> bool {
        matches!(
            self,
            AuraType::Security(SecurityType::Secret) | AuraType::Security(SecurityType::Token)
        )
    }

    /// Returns true if this is a security type that cannot be serialized.
    pub fn is_no_serialize(&self) -> bool {
        matches!(
            self,
            AuraType::Security(SecurityType::Secret) | AuraType::Security(SecurityType::Token)
        )
    }

    /// Human-readable type name for error messages.
    pub fn display_name(&self) -> String {
        match self {
            AuraType::Primitive(p) => match p {
                PrimitiveType::Text => "text".to_string(),
                PrimitiveType::Int => "int".to_string(),
                PrimitiveType::Float => "float".to_string(),
                PrimitiveType::Bool => "bool".to_string(),
                PrimitiveType::Timestamp => "timestamp".to_string(),
                PrimitiveType::Duration => "duration".to_string(),
                PrimitiveType::Percent => "percent".to_string(),
            },
            AuraType::Security(s) => match s {
                SecurityType::Secret => "secret".to_string(),
                SecurityType::Sanitized => "sanitized".to_string(),
                SecurityType::Email => "email".to_string(),
                SecurityType::Url => "url".to_string(),
                SecurityType::Token => "token".to_string(),
            },
            AuraType::List(inner) => format!("list[{}]", inner.display_name()),
            AuraType::Set(inner) => format!("set[{}]", inner.display_name()),
            AuraType::Map(k, v) => format!("map[{}, {}]", k.display_name(), v.display_name()),
            AuraType::Optional(inner) => format!("optional[{}]", inner.display_name()),
            AuraType::Enum(variants) => {
                let names: Vec<_> = variants.iter().map(|v| v.name.as_str()).collect();
                format!("enum[{}]", names.join(", "))
            }
            AuraType::Named(name) => name.clone(),
            AuraType::Function(ft) => {
                let params: Vec<_> = ft.params.iter().map(|p| p.display_name()).collect();
                format!(
                    "fn({}) -> {}",
                    params.join(", "),
                    ft.return_type.display_name()
                )
            }
            AuraType::Action(params) => {
                if params.is_empty() {
                    "action".to_string()
                } else {
                    let p: Vec<_> = params.iter().map(|p| p.display_name()).collect();
                    format!("action({})", p.join(", "))
                }
            }
            AuraType::Union(types) => {
                let parts: Vec<_> = types.iter().map(|t| t.display_name()).collect();
                parts.join(" | ")
            }
            AuraType::TypeParam(name) => name.clone(),
            AuraType::Conditional {
                check,
                extends,
                then_type,
                else_type,
            } => {
                format!(
                    "{} extends {} ? {} : {}",
                    check.display_name(),
                    extends.display_name(),
                    then_type.display_name(),
                    else_type.display_name()
                )
            }
            AuraType::Var(id) => format!("?T{}", id),
            AuraType::Poison => "<error>".to_string(),
        }
    }
}

impl std::fmt::Display for AuraType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_names() {
        assert_eq!(
            AuraType::Primitive(PrimitiveType::Text).display_name(),
            "text"
        );
        assert_eq!(
            AuraType::Security(SecurityType::Secret).display_name(),
            "secret"
        );
        assert_eq!(
            AuraType::List(Box::new(AuraType::Named("Todo".to_string()))).display_name(),
            "list[Todo]"
        );
        assert_eq!(
            AuraType::Optional(Box::new(AuraType::Primitive(PrimitiveType::Text))).display_name(),
            "optional[text]"
        );
    }

    #[test]
    fn test_security_type_restrictions() {
        assert!(AuraType::Security(SecurityType::Secret).is_no_log());
        assert!(AuraType::Security(SecurityType::Token).is_no_log());
        assert!(!AuraType::Security(SecurityType::Email).is_no_log());
        assert!(AuraType::Security(SecurityType::Secret).is_no_serialize());
        assert!(!AuraType::Security(SecurityType::Sanitized).is_no_serialize());
    }

    #[test]
    fn test_poison_propagation() {
        assert!(AuraType::Poison.is_poison());
        assert!(!AuraType::Primitive(PrimitiveType::Int).is_poison());
    }
}
