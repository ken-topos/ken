//! Kernel errors — minimal, precise reasons (`18 §4`).
//!
//! The kernel answers typed/not-typed, equal/not-equal with a *minimal* reason
//! (the failing subterm and, where relevant, the two types that did not
//! convert). Turning that into a human/agent diagnostic is V4's job, not the
//! kernel's (`18 §4`). [`KernelError`] carries just enough to localise a
//! failure; the `Display` render is for tests, not users.

use crate::term::{Level, Term};

/// A kernel check failure. `Debug`-rendered terms/levels are short; the kernel
/// never produces a partial or `unknown` result (`18 §4`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// A de Bruijn index does not resolve to an in-scope binding (raw-wf,
    /// `11 §6`). `(index, context_depth)`.
    VarOutOfScope { index: usize, depth: usize },

    /// A `[K2]`-reserved former appeared in `check`/`infer` — K1 rejects it as
    /// unrecognised (`11 §6`, `12 §5`).
    K2ReservedFormer,

    /// A universe-level loop / mismatch — the `Type : Type` family (`12 §1`).
    /// Soundness anchor: `Type ℓ : Type ℓ` is rejected.
    UniverseInconsistency { expected: Level, found: Level },

    /// `check` failed: `t` does not have the expected type (`18 §3`).
    TypeMismatch {
        expected: Box<Term>,
        found: Box<Term>,
    },

    /// Application of a non-function (head is not a Π-type) (`13 §1`).
    NotAFunction { head: Box<Term> },

    /// Projection of a non-Σ-type (`13 §2`).
    NotASigma { head: Box<Term> },

    /// Strict-positivity violation at admission (`14 §2`, §8).
    PositivityViolation(String),

    /// A declaration signature or arity is ill-formed (`14 §1`).
    IllFormedDecl(String),

    /// Wrong number of level arguments at a polymorphic use (`12 §4`).
    LevelArityMismatch { expected: usize, found: usize },

    /// A motive or method does not have the shape the eliminator requires
    /// (`14 §3`, §7).
    BadEliminator(String),

    /// A primitive-reduction or trusted-base accounting failure (`14 §5`).
    Primitive(String),

    /// A labelled failure for checks that need a short prose reason.
    Msg(String),

    /// SCT gate rejected the definition group as non-terminating (`17 §4`).
    ScfFailed(String),
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelError::VarOutOfScope { index, depth } => write!(
                f,
                "de Bruijn index {index} out of scope (context depth {depth})"
            ),
            KernelError::K2ReservedFormer => write!(
                f,
                "K2-reserved former not implemented in K1 (check/infer reject)"
            ),
            KernelError::UniverseInconsistency { expected, found } => write!(
                f,
                "universe inconsistency: expected Type {:?}, found Type {:?} (no Type:Type)",
                expected, found
            ),
            KernelError::TypeMismatch { expected, found } => write!(
                f,
                "type mismatch: expected {:?}, found {:?}",
                expected, found
            ),
            KernelError::NotAFunction { head } => {
                write!(f, "application of a non-function: {:?}", head)
            }
            KernelError::NotASigma { head } => {
                write!(f, "projection of a non-Σ-type: {:?}", head)
            }
            KernelError::PositivityViolation(s) => {
                write!(f, "strict-positivity violation: {s}")
            }
            KernelError::IllFormedDecl(s) => write!(f, "ill-formed declaration: {s}"),
            KernelError::LevelArityMismatch { expected, found } => write!(
                f,
                "wrong number of level arguments: expected {expected}, found {found}"
            ),
            KernelError::BadEliminator(s) => write!(f, "bad eliminator: {s}"),
            KernelError::Primitive(s) => write!(f, "primitive: {s}"),
            KernelError::Msg(s) => write!(f, "{s}"),
            KernelError::ScfFailed(s) => write!(f, "SCT gate rejected: {s}"),
        }
    }
}

impl std::error::Error for KernelError {}

/// The kernel's checking result.
pub type KernelResult<T> = Result<T, KernelError>;
