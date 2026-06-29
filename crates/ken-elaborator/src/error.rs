//! Surface-level elaboration errors (`39 §5.6`).

use std::fmt;

use ken_kernel::KernelError;

/// A source span (byte offsets, 0-based).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn zero() -> Self {
        Self::default()
    }
    pub fn merge(a: &Self, b: &Self) -> Self {
        Self {
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }
    }
}

/// A V0 elaboration error (`39 §5.6`): parse, name-resolution, or type error.
#[derive(Debug, Clone)]
pub enum ElabError {
    /// A lexer/parser failure (`31 §8`, `32 §8`).
    ParseError { msg: String, span: Span },
    /// An unresolved name at the name-resolution stage (`39 §5.3`).
    UnboundName { name: String, span: Span },
    /// A `ConId` with no global declaration.
    UnresolvedCon { name: String, span: Span },
    /// The elaborator surfaced a kernel type-mismatch (`39 §5.6`).
    TypeMismatch { span: Span, reason: String },
    /// A λ was checked against a non-Π type — V0 structural rejection (`39 §5.6`).
    LambdaVsNonFunction { span: Span },
    /// A non-Π head in application position (`39 §5.6`).
    NotAFunction { span: Span },
    /// Level meta unification failed (unsolvable constraint).
    LevelConflict { span: Span },
    /// The kernel rejected the emitted term (wrapped kernel error).
    KernelRejected { error: KernelError, span: Span },
    /// Catch-all for internal elaborator errors.
    Internal(String),
}

impl fmt::Display for ElabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElabError::ParseError { msg, span } => {
                write!(f, "parse error at {}-{}: {}", span.start, span.end, msg)
            }
            ElabError::UnboundName { name, span } => {
                write!(f, "unbound name '{}' at {}-{}", name, span.start, span.end)
            }
            ElabError::UnresolvedCon { name, span } => {
                write!(f, "unresolved type '{}' at {}-{}", name, span.start, span.end)
            }
            ElabError::TypeMismatch { span, reason } => {
                write!(f, "type mismatch at {}-{}: {}", span.start, span.end, reason)
            }
            ElabError::LambdaVsNonFunction { span } => {
                write!(
                    f,
                    "lambda checked against non-function type at {}-{}",
                    span.start, span.end
                )
            }
            ElabError::NotAFunction { span } => {
                write!(f, "not a function at {}-{}", span.start, span.end)
            }
            ElabError::LevelConflict { span } => {
                write!(f, "level conflict at {}-{}", span.start, span.end)
            }
            ElabError::KernelRejected { error, span } => {
                write!(
                    f,
                    "kernel rejected at {}-{}: {}",
                    span.start, span.end, error
                )
            }
            ElabError::Internal(s) => write!(f, "internal error: {}", s),
        }
    }
}

impl std::error::Error for ElabError {}
