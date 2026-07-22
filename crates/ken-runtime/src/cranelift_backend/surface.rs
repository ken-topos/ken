//! Outward data surface of the Cranelift backend: reports, evidence, errors,
//! and the seed environment, together with their `Display`/`Error`/`From`
//! impls and the error constructors.
//!
//! RT-SPLIT slice 1 of 7. Pure move out of the flat `cranelift_backend`
//! module; no logic, signature, or rename changes. This module is a leaf --
//! it depends on nothing else in the decomposition.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    KenCheckedProofErasureBoundaryReport, RuntimeArtifactValidationError,
    RuntimeArtifactValidationReport, RuntimeGroundValue, RuntimeIrRunReport, RuntimeObservation,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraneliftRunReport {
    pub example: String,
    pub observation: RuntimeObservation,
    pub verifier_passed: bool,
    pub native_returned: Option<i64>,
    pub trust: NativeTrustReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraneliftObjectArtifact {
    pub example: String,
    pub entry_symbol: String,
    pub object_bytes: Vec<u8>,
    pub object_hash: u64,
    pub platform_target: String,
    pub backend_name: String,
    pub verifier_passed: bool,
    pub assumptions: BTreeSet<String>,
    pub unsupported: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeTrustReport {
    pub backend: &'static str,
    pub fidelity: NativeFidelity,
    pub verifier_passed: bool,
    pub artifact_validation: Option<RuntimeArtifactValidationReport>,
    pub ken_checked_proof_erasure_boundary: Option<KenCheckedProofErasureBoundaryReport>,
    pub toolchain: NativeToolchainReport,
    pub evidence: NativeRunEvidence,
    pub assumptions: BTreeSet<String>,
    pub unsupported: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeToolchainReport {
    pub cranelift: NativeEvidenceFact,
    pub linker: NativeEvidenceFact,
    pub runtime: NativeEvidenceFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeEvidenceFact {
    Available {
        value: String,
        evidence_source: String,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeRunEvidence {
    pub package_identity: Option<String>,
    pub core_semantic_hash: Option<u64>,
    pub runtime_artifact_hash: Option<u64>,
    pub evidence_sources: BTreeMap<String, String>,
    pub unavailable: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpreterOracleObservation {
    pub artifact: NativeArtifactIdentity,
    pub observation: RuntimeObservation,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDifferentialReport {
    pub example: String,
    pub artifact: NativeArtifactIdentity,
    pub oracle: InterpreterOracleObservation,
    pub native: Option<CraneliftRunReport>,
    pub verdict: NativeDifferentialVerdict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRuntimeIrComparisonReport {
    pub example: String,
    pub artifact: NativeArtifactIdentity,
    pub runtime_ir: RuntimeIrRunReport,
    pub native: Option<CraneliftRunReport>,
    pub verdict: NativeRuntimeIrComparisonVerdict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeArtifactIdentity {
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub runtime_artifact_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeDifferentialVerdict {
    F1InterpreterAgreement {
        stage: NativeDifferentialStage,
    },
    Unsupported {
        stage: NativeDifferentialStage,
        construct: &'static str,
        reason: String,
    },
    Mismatch {
        stage: NativeDifferentialStage,
        interpreter: RuntimeObservation,
        native: RuntimeObservation,
    },
    BackendFailure {
        stage: NativeDifferentialStage,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeRuntimeIrComparisonVerdict {
    RuntimeIrNativeAgreement {
        stage: NativeDifferentialStage,
    },
    Unsupported {
        stage: NativeDifferentialStage,
        construct: &'static str,
        reason: String,
    },
    Mismatch {
        stage: NativeDifferentialStage,
        runtime_ir: RuntimeObservation,
        native: RuntimeObservation,
    },
    BackendFailure {
        stage: NativeDifferentialStage,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeDifferentialStage {
    BoundaryPreflight,
    NativeLoweringOrExecution,
    InterpreterNativeCompare,
    RuntimeIrNativeCompare,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeFidelity {
    F0NativeExample,
    F1SeedObservationAgreement,
    F1InterpreterDifferentialAgreement,
    F1RuntimeIrEvaluatorAgreement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CraneliftBackendError {
    Unsupported(UnsupportedLowering),
    Backend(BackendFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidatedNativeRunError {
    Validation(RuntimeArtifactValidationError),
    Backend(CraneliftBackendError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedLowering {
    pub construct: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendFailure {
    Target(String),
    Verifier(String),
    Module(String),
    NativeResultDecode { token: i64 },
}

impl fmt::Display for CraneliftBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CraneliftBackendError::Unsupported(err) => {
                write!(f, "unsupported runtime-IR lowering: {err}")
            }
            CraneliftBackendError::Backend(err) => write!(f, "Cranelift backend failure: {err}"),
        }
    }
}

impl std::error::Error for CraneliftBackendError {}

impl fmt::Display for ValidatedNativeRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidatedNativeRunError::Validation(err) => {
                write!(f, "runtime artifact validation failed: {err}")
            }
            ValidatedNativeRunError::Backend(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ValidatedNativeRunError {}

impl From<RuntimeArtifactValidationError> for ValidatedNativeRunError {
    fn from(err: RuntimeArtifactValidationError) -> Self {
        ValidatedNativeRunError::Validation(err)
    }
}

impl From<CraneliftBackendError> for ValidatedNativeRunError {
    fn from(err: CraneliftBackendError) -> Self {
        ValidatedNativeRunError::Backend(err)
    }
}

impl fmt::Display for UnsupportedLowering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.construct, self.reason)
    }
}

impl fmt::Display for BackendFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendFailure::Target(msg) => write!(f, "target setup failed: {msg}"),
            BackendFailure::Verifier(msg) => write!(f, "verifier rejected function: {msg}"),
            BackendFailure::Module(msg) => write!(f, "module operation failed: {msg}"),
            BackendFailure::NativeResultDecode { token } => {
                write!(f, "native result token {token} is not in the result table")
            }
        }
    }
}

/// Diagnostic-rendering coverage for the backend's outward error surface.
///
/// These types are the crate's user-facing failure vocabulary, reachable as
/// `ken_runtime::<name>` through the `lib.rs` glob re-export. Before these
/// tests, six of the seven format strings below occurred exactly once each in
/// the whole workspace — at their own `write!` site — so any edit to a
/// rendered message was unobservable to the suite. (The seventh,
/// `"unsupported runtime-IR lowering: "`, is pinned indirectly by
/// `native_process_entrypoint.rs`'s report assertion.)
///
/// The two properties worth pinning are not the individual strings but the
/// *composition rules*, which is where a plausible "tidy-up" edit would land:
/// `CraneliftBackendError` prefixes both of its arms, while
/// `ValidatedNativeRunError` prefixes `Validation` and deliberately delegates
/// `Backend` verbatim. Each is asserted against its opposite so neither can be
/// normalized into the other without a failure.
#[cfg(test)]
mod surface_diagnostics_tests {
    use super::*;
    use crate::artifact_validation::{
        RuntimeArtifactValidationError, RuntimeArtifactValidationStage,
    };

    fn validation_error() -> RuntimeArtifactValidationError {
        RuntimeArtifactValidationError {
            stage: RuntimeArtifactValidationStage::ClaimMismatch,
            fact: "core_semantic_hash",
            reason: "recomputed claim disagrees".to_string(),
        }
    }

    #[test]
    fn backend_failure_renders_every_variant_distinctly() {
        // Exhaustive by construction: this `match` has no `_` arm, so adding a
        // `BackendFailure` variant is a compile error here rather than a
        // silently unrendered case.
        let cases = [
            BackendFailure::Target("no isa for wasm64".to_string()),
            BackendFailure::Verifier("inst12 has no type".to_string()),
            BackendFailure::Module("duplicate symbol".to_string()),
            BackendFailure::NativeResultDecode { token: -7 },
        ];
        for case in &cases {
            let rendered = case.to_string();
            let expected = match case {
                BackendFailure::Target(msg) => format!("target setup failed: {msg}"),
                BackendFailure::Verifier(msg) => format!("verifier rejected function: {msg}"),
                BackendFailure::Module(msg) => format!("module operation failed: {msg}"),
                BackendFailure::NativeResultDecode { token } => {
                    format!("native result token {token} is not in the result table")
                }
            };
            assert_eq!(rendered, expected, "BackendFailure rendering drifted");
        }

        // The variants must stay mutually distinguishable in text: a reader of
        // a log line has only this string to go on.
        let rendered: Vec<String> = cases.iter().map(|c| c.to_string()).collect();
        let distinct: BTreeSet<&String> = rendered.iter().collect();
        assert_eq!(
            distinct.len(),
            rendered.len(),
            "two BackendFailure variants render identically: {rendered:?}"
        );
    }

    #[test]
    fn unsupported_lowering_renders_construct_then_reason() {
        let err = UnsupportedLowering {
            construct: "Effect",
            reason: "no native lowering".to_string(),
        };
        assert_eq!(err.to_string(), "Effect: no native lowering");
    }

    #[test]
    fn cranelift_backend_error_prefixes_both_arms() {
        let unsupported = CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Effect",
            reason: "no native lowering".to_string(),
        });
        assert_eq!(
            unsupported.to_string(),
            "unsupported runtime-IR lowering: Effect: no native lowering"
        );

        let backend =
            CraneliftBackendError::Backend(BackendFailure::Module("duplicate symbol".to_string()));
        assert_eq!(
            backend.to_string(),
            "Cranelift backend failure: module operation failed: duplicate symbol"
        );
    }

    #[test]
    fn validated_run_error_prefixes_validation_but_delegates_backend_verbatim() {
        // The asymmetry is deliberate and is the whole point of this test: an
        // edit that gave `Backend` a prefix "for consistency", or dropped
        // `Validation`'s, would leave every other test in the crate green.
        let validation = ValidatedNativeRunError::Validation(validation_error());
        assert_eq!(
            validation.to_string(),
            format!("runtime artifact validation failed: {}", validation_error()),
            "the Validation arm must prefix"
        );

        let inner =
            CraneliftBackendError::Backend(BackendFailure::Target("no isa for wasm64".to_string()));
        let backend = ValidatedNativeRunError::Backend(inner.clone());
        assert_eq!(
            backend.to_string(),
            inner.to_string(),
            "the Backend arm must delegate verbatim, adding no prefix of its own"
        );
        assert!(
            !backend
                .to_string()
                .starts_with("runtime artifact validation failed"),
            "the Backend arm must not acquire the Validation arm's prefix"
        );
    }

    #[test]
    fn validated_run_error_from_impls_select_the_matching_arm() {
        // Both `From` impls are unreferenced outside their own declaration, so
        // nothing else in the suite would notice if they were crossed.
        let from_validation: ValidatedNativeRunError = validation_error().into();
        assert_eq!(
            from_validation,
            ValidatedNativeRunError::Validation(validation_error())
        );

        let inner = CraneliftBackendError::Backend(BackendFailure::Verifier("inst12".to_string()));
        let from_backend: ValidatedNativeRunError = inner.clone().into();
        assert_eq!(from_backend, ValidatedNativeRunError::Backend(inner));
    }

    #[test]
    fn backend_errors_are_usable_as_std_error_trait_objects() {
        // `impl std::error::Error` on these types is load-bearing for callers
        // that box them; nothing else in the crate exercises it.
        let boxed: Box<dyn std::error::Error> = Box::new(CraneliftBackendError::Backend(
            BackendFailure::Module("duplicate symbol".to_string()),
        ));
        assert_eq!(
            boxed.to_string(),
            "Cranelift backend failure: module operation failed: duplicate symbol"
        );

        let boxed: Box<dyn std::error::Error> =
            Box::new(ValidatedNativeRunError::Validation(validation_error()));
        assert!(boxed
            .to_string()
            .starts_with("runtime artifact validation failed: "));
    }

    #[test]
    fn error_constructors_build_the_arms_they_name() {
        assert_eq!(
            unsupported("Effect", "no native lowering"),
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Effect",
                reason: "no native lowering".to_string(),
            })
        );
        assert_eq!(
            backend(BackendFailure::Target("no isa".to_string())),
            CraneliftBackendError::Backend(BackendFailure::Target("no isa".to_string()))
        );
        assert_eq!(
            backend_module("duplicate symbol".to_string()),
            CraneliftBackendError::Backend(BackendFailure::Module("duplicate symbol".to_string())),
            "backend_module must select the Module variant, not Target or Verifier"
        );
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeSeedEnvironment {
    pub(super) values: BTreeMap<String, RuntimeGroundValue>,
}

impl NativeSeedEnvironment {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn nc5_seed() -> Self {
        let mut values = BTreeMap::new();
        values.insert(
            "decl:fixture::Local::y".to_string(),
            RuntimeGroundValue::Int((2).into()),
        );
        Self { values }
    }

    pub fn insert(&mut self, symbol: impl Into<String>, value: RuntimeGroundValue) {
        self.values.insert(symbol.into(), value);
    }
}
pub(super) fn unsupported(
    construct: &'static str,
    reason: impl Into<String>,
) -> CraneliftBackendError {
    CraneliftBackendError::Unsupported(UnsupportedLowering {
        construct,
        reason: reason.into(),
    })
}

pub(super) fn backend(failure: BackendFailure) -> CraneliftBackendError {
    CraneliftBackendError::Backend(failure)
}

pub(crate) fn backend_module(reason: String) -> CraneliftBackendError {
    backend(BackendFailure::Module(reason))
}
