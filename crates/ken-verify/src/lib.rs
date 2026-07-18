//! Independent interpreter/native external-effect differential harness.
//!
//! This crate is a tested, target-validated judge. It is not a proof surface.
//! The passing interpreter lane uses a real [`ken_interp::PosixHost`] rooted at
//! one of two byte-identical temporary roots. The native lane executes the
//! exact PX5 bound checked-source artifact against the other root through
//! `run_bound_process_effect_observation`. `CaptureHost` is only an explicit
//! insufficient negative control, never a passing substrate.
//!
//! Runtime owns `EffectObservation`, `EffectEvent`, canonical error identity,
//! and `FsDeltaV1`. This crate intentionally does not define substitutes for
//! those types. [`canonical::compare_canonical_exact`] consumes their concrete
//! `ken_host` re-export directly.

#![forbid(unsafe_code)]

pub mod canonical;
pub mod catalog;
pub mod filesystem;
pub mod host;
pub mod observation;
pub mod scenario;

pub use canonical::*;
pub use catalog::*;
pub use filesystem::*;
pub use host::*;
pub use observation::*;
pub use scenario::*;

/// Honest trust disclosure attached to PX6 evidence.
pub const TRUST_DISCLOSURE: &str = "tested/target-validated harness; zero kernel rules, zero Ken postulates, no proof-of-confinement claim";

/// The passing-lane substrate is intentionally explicit and machine-checkable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceSubstrate {
    TwinRealRootsAndProducedArtifact,
    UnitOrNegativeControl,
}
