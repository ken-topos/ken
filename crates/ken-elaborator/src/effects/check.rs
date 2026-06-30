//! Effect discipline checks: escape gate (§1.4), capability gate (§2.5),
//! cross-space alias (§4.4), and tail-resumptive handler (§5.2).

use super::algebra::cap_set;
use super::infer::EffectDecl;
use super::row::{EffectName, EffectRow};

// ----- error type -----

/// A static error from the effect discipline (`36 §7.3`).
///
/// All variants are **elaboration-phase** errors caught before the kernel;
/// the kernel sees only the pure denotation (§7.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectError {
    /// `ρ_inf ⊄ ρ_decl` — an inferred effect escapes the declared bound (§1.4).
    EffectEscapes {
        decl_name: String,
        /// Each escaping effect paired with the call/perform site that
        /// introduced it (§1.4: the check names the witness, not just the diff).
        witnesses: Vec<(EffectName, String)>,
    },
    /// A `perform_E op` with no `Cap E` in scope (§2.5, §7.3.2).
    MissingCapability {
        decl_name: String,
        effect: EffectName,
    },
    /// Direct access to another space's `mut` cell — violates shared-nothing
    /// isolation (§4.4).
    CrossSpaceAlias {
        decl_name: String,
        target_space: String,
    },
    /// A handler resumes more than once or not in tail position (§5.2, OQ-9).
    NonTailResumptive {
        decl_name: String,
    },
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EffectEscapes { decl_name, witnesses } => {
                write!(f, "EffectEscapes in '{}': ", decl_name)?;
                for (i, (eff, site)) in witnesses.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} (via {})", eff, site)?;
                }
                Ok(())
            }
            Self::MissingCapability { decl_name, effect } => {
                write!(
                    f,
                    "MissingCapability in '{}': no Cap {} in scope",
                    decl_name, effect
                )
            }
            Self::CrossSpaceAlias { decl_name, target_space } => {
                write!(
                    f,
                    "CrossSpaceAlias in '{}': direct access to space '{}'",
                    decl_name, target_space
                )
            }
            Self::NonTailResumptive { decl_name } => {
                write!(
                    f,
                    "NonTailResumptive in '{}': handler resumes more than once \
                     or not in tail position",
                    decl_name
                )
            }
        }
    }
}

impl std::error::Error for EffectError {}

// ----- §1.4 escape check -----

/// Witness map: for each inferred effect, which call site introduced it.
///
/// Callers populate this so the escape error can name a source witness
/// (§1.4: the error is not just a set difference, it includes a site).
pub type WitnessMap = std::collections::HashMap<EffectName, String>;

/// Check `ρ_inf ⊆ ρ_decl`; produce `EffectEscapes` with witnesses on failure
/// (§1.4, frame AC1).
///
/// `witnesses`: maps each effect name to the call/perform site that introduced
/// it into the inferred row. Used to populate the error's witness list.
pub fn check_escape(
    decl: &EffectDecl,
    inferred: &EffectRow,
    witnesses: &WitnessMap,
) -> Result<(), EffectError> {
    let declared = decl
        .declared_row
        .as_ref()
        .cloned()
        .unwrap_or_else(EffectRow::empty);

    if inferred.is_subset_of(&declared) {
        return Ok(());
    }

    let escaping = inferred.minus(&declared);
    let ws: Vec<(EffectName, String)> = escaping
        .effects()
        .map(|e| {
            let site = witnesses
                .get(e.as_str())
                .cloned()
                .unwrap_or_else(|| "<unknown>".into());
            (e.clone(), site)
        })
        .collect();

    Err(EffectError::EffectEscapes {
        decl_name: decl.name.clone(),
        witnesses: ws,
    })
}

// ----- §2.5 capability check -----

/// Check that every effect in `performed_effects` has a corresponding `Cap E`
/// in `cap_params` or is provided by an enclosing handler (`handler_caps`)
/// (§2.5, §7.3.2, frame AC3).
pub fn check_capabilities(
    decl: &EffectDecl,
    performed_effects: &EffectRow,
    handler_caps: &EffectRow,
) -> Result<(), EffectError> {
    let available = cap_set(&decl.cap_params).join(handler_caps);
    for e in performed_effects.effects() {
        if !available.contains(e.as_str()) {
            return Err(EffectError::MissingCapability {
                decl_name: decl.name.clone(),
                effect: e.clone(),
            });
        }
    }
    Ok(())
}

/// Check capability availability given explicit cap params (convenience form
/// for cases where no enclosing handler is present).
pub fn check_capabilities_no_handler(
    decl: &EffectDecl,
    performed_effects: &EffectRow,
) -> Result<(), EffectError> {
    check_capabilities(decl, performed_effects, &EffectRow::empty())
}

// ----- §4.4 cross-space alias check -----

/// A cross-space access attempt: `from_space` body directly reads/writes a
/// `mut` cell of `to_space`.
#[derive(Debug, Clone)]
pub struct CrossSpaceAccess {
    pub from_space: String,
    pub to_space: String,
}

/// Check that no space directly aliases another space's mutable cells (§4.4).
///
/// Returns the first `CrossSpaceAlias` error found, or `Ok(())`.
pub fn check_cross_space(
    accesses: &[CrossSpaceAccess],
) -> Result<(), EffectError> {
    for a in accesses {
        if a.from_space != a.to_space {
            return Err(EffectError::CrossSpaceAlias {
                decl_name: a.from_space.clone(),
                target_space: a.to_space.clone(),
            });
        }
    }
    Ok(())
}

// ----- §5.2 tail-resumptive handler check -----

/// Whether a handler invocation is tail-resumptive (§5.2, OQ-9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeKind {
    /// Resume once, in tail position — permitted (§5.2).
    TailOnce,
    /// Resume more than once — rejected (`NonTailResumptive`, §7.3.3).
    MultiShot,
    /// Resume in a non-tail position — rejected (`NonTailResumptive`, §7.3.3).
    NonTail,
    /// No resume (stop) — permitted (§5.2: "at most once").
    Stop,
}

/// Check that a handler is tail-resumptive (§5.2, OQ-9).
pub fn check_tail_resumptive(
    decl_name: &str,
    resume_kind: ResumeKind,
) -> Result<(), EffectError> {
    match resume_kind {
        ResumeKind::TailOnce | ResumeKind::Stop => Ok(()),
        ResumeKind::MultiShot | ResumeKind::NonTail => {
            Err(EffectError::NonTailResumptive {
                decl_name: decl_name.to_string(),
            })
        }
    }
}

// ----- combined check: full §1.4 + §2.5 pass -----

/// Run the full effect-discipline check on one declaration:
/// 1. Escape check (`ρ_inf ⊆ ρ_decl`).
/// 2. Capability check (each performed effect has `Cap E` in scope).
///
/// Returns the first error found, or `Ok(())`.
pub fn check_decl(
    decl: &EffectDecl,
    inferred: &EffectRow,
    witnesses: &WitnessMap,
    handler_caps: &EffectRow,
) -> Result<(), EffectError> {
    check_escape(decl, inferred, witnesses)?;
    let performed = EffectRow::from_effects(decl.performed_effects.iter().cloned());
    check_capabilities(decl, &performed, handler_caps)?;
    Ok(())
}
