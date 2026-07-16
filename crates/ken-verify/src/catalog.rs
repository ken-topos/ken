//! Evidence-bearing confirmation gate over PX5's sealed catalog.

use ken_host::{EffectObservationV1, HostOpAvailabilityV1, HostOpV1};

use crate::{compare_canonical_exact, CanonicalDifferentialRun};

pub fn native_tested_lanes() -> Vec<HostOpV1> {
    HostOpV1::ALL
        .into_iter()
        .filter(|operation| operation.availability() == HostOpAvailabilityV1::NativeTested)
        .collect()
}

pub fn deferred_named_lanes() -> Vec<HostOpV1> {
    HostOpV1::ALL
        .into_iter()
        .filter(|operation| {
            operation.availability() == HostOpAvailabilityV1::RepresentedUnavailable
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeTestedEvidence {
    exact_artifact_executed: bool,
    canonical_observation_equal: bool,
    operation_observed_in_both_lanes: bool,
}

impl NativeTestedEvidence {
    pub fn from_run(operation: HostOpV1, run: &CanonicalDifferentialRun) -> Self {
        Self {
            exact_artifact_executed: run.exact_artifact_executed,
            canonical_observation_equal: run.compare_exact().is_ok(),
            operation_observed_in_both_lanes: [&run.interpreter, &run.native].into_iter().all(
                |observation| {
                    observation
                        .effect_trace
                        .iter()
                        .any(|event| event.operation == operation)
                },
            ),
        }
    }

    /// CaptureHost and hand-fed observations can exercise comparator units but
    /// can never manufacture exact-artifact evidence.
    pub fn unit_or_negative_control(
        operation: HostOpV1,
        interpreter: &EffectObservationV1,
        subject: &EffectObservationV1,
    ) -> Self {
        Self {
            exact_artifact_executed: false,
            canonical_observation_equal: compare_canonical_exact(interpreter, subject).is_ok(),
            operation_observed_in_both_lanes: [interpreter, subject].into_iter().all(
                |observation| {
                    observation
                        .effect_trace
                        .iter()
                        .any(|event| event.operation == operation)
                },
            ),
        }
    }

    pub fn permits_confirmation(self) -> bool {
        self.exact_artifact_executed
            && self.canonical_observation_equal
            && self.operation_observed_in_both_lanes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusTransitionError {
    OutsideNativeTestedSet(HostOpV1),
    MissingExactArtifactEvidence(HostOpV1),
    ExternalObservationMismatch(HostOpV1),
    OperationNotObserved(HostOpV1),
}

/// Confirm that PX5's catalog transition is backed by exact PX6 artifact
/// evidence. Deferred operations cannot be confirmed through this gate.
pub fn confirm_native_tested_transition(
    operation: HostOpV1,
    evidence: NativeTestedEvidence,
) -> Result<HostOpAvailabilityV1, StatusTransitionError> {
    if !ken_host::NATIVE_TESTED_TARGETS_V1.contains(&operation)
        || operation.availability() != HostOpAvailabilityV1::NativeTested
    {
        return Err(StatusTransitionError::OutsideNativeTestedSet(operation));
    }
    if !evidence.exact_artifact_executed {
        return Err(StatusTransitionError::MissingExactArtifactEvidence(
            operation,
        ));
    }
    if !evidence.canonical_observation_equal {
        return Err(StatusTransitionError::ExternalObservationMismatch(
            operation,
        ));
    }
    if !evidence.operation_observed_in_both_lanes {
        return Err(StatusTransitionError::OperationNotObserved(operation));
    }
    Ok(HostOpAvailabilityV1::NativeTested)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_catalog_has_exact_six_native_and_nine_named_unavailable() {
        assert_eq!(HostOpV1::ALL.len(), 15);
        assert_eq!(native_tested_lanes(), ken_host::NATIVE_TESTED_TARGETS_V1);
        assert_eq!(deferred_named_lanes().len(), 9);
        assert!(deferred_named_lanes().into_iter().all(|operation| {
            operation.availability() == HostOpAvailabilityV1::RepresentedUnavailable
                && !ken_host::NATIVE_TESTED_TARGETS_V1.contains(&operation)
        }));
    }

    #[test]
    fn confirmation_requires_real_artifact_equality_and_observed_operation() {
        let operation = HostOpV1::ConsoleFlush;
        let evidence =
            |exact_artifact_executed,
             canonical_observation_equal,
             operation_observed_in_both_lanes| NativeTestedEvidence {
                exact_artifact_executed,
                canonical_observation_equal,
                operation_observed_in_both_lanes,
            };
        assert_eq!(
            confirm_native_tested_transition(operation, evidence(false, true, true)),
            Err(StatusTransitionError::MissingExactArtifactEvidence(
                operation
            ))
        );
        assert_eq!(
            confirm_native_tested_transition(operation, evidence(true, false, true)),
            Err(StatusTransitionError::ExternalObservationMismatch(
                operation
            ))
        );
        assert_eq!(
            confirm_native_tested_transition(operation, evidence(true, true, false)),
            Err(StatusTransitionError::OperationNotObserved(operation))
        );
        assert_eq!(
            confirm_native_tested_transition(operation, evidence(true, true, true)),
            Ok(HostOpAvailabilityV1::NativeTested)
        );
    }
}
