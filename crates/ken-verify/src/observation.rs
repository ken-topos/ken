//! Exact canonical mismatch diagnostics and deliberately weak controls.

use std::fmt;

use crate::RootSnapshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservationField {
    Stdout,
    Stderr,
    ExitStatus,
    CanonicalImported(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldMismatch {
    pub field: ObservationField,
    pub interpreter: String,
    pub native: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationMismatch {
    pub mismatches: Vec<FieldMismatch>,
}

impl fmt::Display for ObservationMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, mismatch) in self.mismatches.iter().enumerate() {
            if index != 0 {
                formatter.write_str("; ")?;
            }
            write!(
                formatter,
                "{:?}: interpreter={}, native={}",
                mismatch.field, mismatch.interpreter, mismatch.native
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for ObservationMismatch {}

/// Real-root and handler evidence used by denial-before-action gates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneActionEvidence {
    pub root_before: RootSnapshot,
    pub root_after: RootSnapshot,
    /// `Some` only where the interpreter HostHandler seam directly counted
    /// post-resolution actions. The linked artifact exposes no such counter.
    pub fs_actions_after_resolve: Option<u64>,
}

/// Deliberately weak negative control representing a return/runner-only judge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerOnlyProxy<T> {
    pub scenario_identity: String,
    pub returned_value: T,
}

impl<T: PartialEq> RunnerOnlyProxy<T> {
    pub fn agrees(&self, other: &Self) -> bool {
        self == other
    }
}
