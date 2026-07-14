//! Surface-level elaboration errors (`39 §5.6`).

use std::fmt;

use ken_kernel::{KernelError, Level};

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
    /// A forbidden documentary name in an anonymous boundary header.
    NamedBoundaryHeader { name: String, span: Span },
    /// A `capabilities` item named no supported effect family.
    UnknownCapabilityFamily { family: String, span: Span },
    /// The authority is not a constructor of the family's authority type.
    InvalidCapabilityAuthority {
        family: String,
        authority: String,
        span: Span,
    },
    /// A program declared two authorities for the same effect family.
    DuplicateCapabilityFamily { family: String, span: Span },
    /// Package boundaries do not carry runtime capabilities in this round.
    PackageCapabilitiesNotAllowed { span: Span },
    /// A program's reachable effect row names a family absent from its
    /// boundary capability declaration.
    MissingCapability { effect: String, span: Span },
    /// An unresolved name at the name-resolution stage (`39 §5.3`).
    UnboundName { name: String, span: Span },
    /// A `ConId` with no global declaration.
    UnresolvedCon { name: String, span: Span },
    /// A second top-level definition of a name already defined in the same
    /// compilation unit (`33 §3`, ADR 0014 MRES-5/MRES-7).
    DuplicateDefinition { name: String, span: Span },
    /// A cross-file import revisited a unit on the active import stack
    /// (`33 §3.2`, ADR 0014 MRES-2). `cycle` is the closed path in import-edge
    /// order, rooted at the entry unit (for example, `A`, `B`, `A`).
    ImportCycle { cycle: Vec<String>, span: Span },
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
    /// A constructor argument lives above its data family's universe.
    ///
    /// The kernel remains the authority for the rejection; this surface form
    /// adds source attribution and the existing explicit-family remedy.
    ConstructorUniverseViolation {
        data: String,
        constructor: String,
        argument_name: Option<String>,
        argument_index: usize,
        argument_level: Level,
        family_level: Level,
        span: Span,
    },
    /// A non-exhaustive match: `missing` names the first uncovered constructor (`34 §4`).
    ExhaustivenessError { missing: String, span: Span },
    /// A redundant match arm (`34 §5`): the arm's constructor was already covered.
    ReachabilityError { span: Span },
    /// An instance declared outside the module of its class AND its head-type
    /// (`33 §5.3`, `39 §6.1`). The orphan check is a syntactic, per-module
    /// predicate that keeps canonicity per-module-decidable.
    OrphanInstance {
        class: String,
        head_type: String,
        span: Span,
    },
    /// Two instances registered under the same `(class, head-type)` key
    /// (`39 §6.1`). Reports both candidate spans.
    OverlappingInstances {
        class: String,
        head_type: String,
        first_span: Span,
        second_span: Span,
    },
    /// A real implicit search selected a canonical dictionary whose defining
    /// package is outside the active boundary's direct-use set (`33 §5.5.1`).
    UnadmittedInstance {
        defining_package: String,
        class: String,
        head_type: String,
        instance_id: ken_kernel::GlobalId,
        span: Span,
    },
    /// The instance resolver found multiple candidates and cannot pick
    /// silently (`39 §6.2`, `39 §6.7`).
    AmbiguousInstance {
        class: String,
        head_type: String,
        span: Span,
    },
    /// No instance found for the requested `(class, head-type)` (`39 §6.7`).
    NoInstance {
        class: String,
        ty: String,
        span: Span,
    },
    /// The `sct_check` on the reified dictionary group rejected the resolution
    /// chain — i.e. search would not terminate (`39 §6.4`, `17 §4.2`).
    /// Detected at admission time; never a search-time hang.
    NonTerminatingInstances { span: Span },
    /// An unqualified import clash (`33 §3.3`): a top-level local and a
    /// selective import, or two selective imports, bind distinct declarations.
    /// `sources` names every colliding qualified origin (the reject must
    /// name both, never pick silently).
    AmbiguousReference {
        name: String,
        sources: Vec<String>,
        span: Span,
    },
    /// Catch-all for internal elaborator errors.
    Internal(String),
}

impl fmt::Display for ElabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElabError::ParseError { msg, span } => {
                write!(f, "parse error at {}-{}: {}", span.start, span.end, msg)
            }
            ElabError::NamedBoundaryHeader { name, span } => write!(
                f,
                "named boundary header at {}-{}: '{}' is forbidden; \
                 program/package headers are anonymous",
                span.start, span.end, name,
            ),
            ElabError::UnknownCapabilityFamily { family, span } => write!(
                f,
                "unknown capability family '{}' at {}-{}",
                family, span.start, span.end,
            ),
            ElabError::InvalidCapabilityAuthority {
                family,
                authority,
                span,
            } => write!(
                f,
                "invalid authority '{}' for capability family '{}' at {}-{}",
                authority, family, span.start, span.end,
            ),
            ElabError::DuplicateCapabilityFamily { family, span } => write!(
                f,
                "duplicate capability family '{}' at {}-{}",
                family, span.start, span.end,
            ),
            ElabError::PackageCapabilitiesNotAllowed { span } => write!(
                f,
                "package capability clause at {}-{}: only program headers may \
                 declare capabilities",
                span.start, span.end,
            ),
            ElabError::MissingCapability { effect, span } => write!(
                f,
                "missing capability '{}' at {}-{}",
                effect, span.start, span.end,
            ),
            ElabError::UnboundName { name, span } => {
                write!(f, "unbound name '{}' at {}-{}", name, span.start, span.end)
            }
            ElabError::UnresolvedCon { name, span } => {
                write!(f, "unresolved type '{}' at {}-{}", name, span.start, span.end)
            }
            ElabError::DuplicateDefinition { name, span } => write!(
                f,
                "duplicate definition '{}' at {}-{}: name already defined in this compilation unit",
                name, span.start, span.end,
            ),
            ElabError::ImportCycle { cycle, span } => write!(
                f,
                "import cycle at {}-{}: {}",
                span.start,
                span.end,
                cycle.join(" → "),
            ),
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
            ElabError::ConstructorUniverseViolation {
                data,
                constructor,
                argument_name,
                argument_index,
                argument_level,
                family_level,
                span,
            } => {
                let argument = argument_name
                    .as_deref()
                    .map(|name| format!("'{}'", name))
                    .unwrap_or_else(|| format!("#{}", argument_index + 1));
                write!(
                    f,
                    "constructor argument {} of '{}' at {}-{} has universe {:?}, \
                     which exceeds data family '{}' universe {:?}; use the explicit \
                     family form `data {} : Type n where {{ … }}` with a sufficient n",
                    argument,
                    constructor,
                    span.start,
                    span.end,
                    argument_level,
                    data,
                    family_level,
                    data,
                )
            }
            ElabError::ExhaustivenessError { missing, span } => {
                write!(
                    f,
                    "non-exhaustive match at {}-{}: missing constructor '{}'",
                    span.start, span.end, missing
                )
            }
            ElabError::ReachabilityError { span } => {
                write!(
                    f,
                    "redundant match arm at {}-{}: constructor already covered",
                    span.start, span.end
                )
            }
            ElabError::OrphanInstance { class, head_type, span } => write!(
                f,
                "orphan instance at {}-{}: instance of '{}' for '{}' must be declared \
                 in the module of the class or the head type",
                span.start, span.end, class, head_type,
            ),
            ElabError::OverlappingInstances {
                class,
                head_type,
                first_span,
                second_span,
            } => write!(
                f,
                "overlapping instances at {}-{} and {}-{}: a canonical instance of '{}' for '{}' \
                 is already registered",
                first_span.start, first_span.end, second_span.start, second_span.end,
                class, head_type,
            ),
            ElabError::UnadmittedInstance {
                defining_package, class, head_type, instance_id, span,
            } => write!(
                f,
                "unadmitted instance at {}-{}: package '{}' defines {:?} for '{} {}'; \
                 add it to the boundary's admits list",
                span.start, span.end, defining_package, instance_id, class, head_type,
            ),
            ElabError::AmbiguousInstance { class, head_type, span } => write!(
                f,
                "ambiguous instance at {}-{}: multiple candidates for '{}' at '{}'; \
                 pass the dictionary explicitly",
                span.start, span.end, class, head_type,
            ),
            ElabError::NoInstance { class, ty, span } => write!(
                f,
                "no instance at {}-{}: no instance of '{}' found for '{}'",
                span.start, span.end, class, ty,
            ),
            ElabError::NonTerminatingInstances { span } => write!(
                f,
                "non-terminating instance chain at {}-{}: SCT rejected the \
                 reified resolution group (cyclic or non-decreasing)",
                span.start, span.end,
            ),
            ElabError::AmbiguousReference { name, sources, span } => {
                let mut sorted = sources.clone();
                sorted.sort();
                write!(
                    f,
                    "ambiguous reference to '{}' at {}-{}: resolves to both {} — qualify to disambiguate",
                    name, span.start, span.end, sorted.join(" and "),
                )
            }
            ElabError::Internal(s) => write!(f, "internal error: {}", s),
        }
    }
}

impl std::error::Error for ElabError {}
