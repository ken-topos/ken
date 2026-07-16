//! Exact comparator and mutation net over PX5's imported V1 vocabulary.

use ken_host::{
    CanonicalOutcomeV1, CanonicalReplyV1, CanonicalRequestV1, CapabilityDeniedV1,
    CapabilityTraceIdentity, CreatePolicyV1, DirEntryV1, EffectEventV1, EffectObservationV1,
    FileErrorCauseV1, FileErrorIdentityV1, FsCapabilityOperationV1, FsDeltaV1, FsNodeKindV1,
    FsNodeObservationV1, HostOpV1, IoErrorIdentityV1, SemanticErrorV1, TerminalErrorV1,
};

use crate::{
    FieldMismatch, LaneActionEvidence, ObservationField, ObservationMismatch, RunnerOnlyProxy,
};

/// Compare every canonical field exactly. Equality is the imported enum/value
/// identity: no message-string comparison, path normalization, trace sorting,
/// or comparator-side repair is permitted.
pub fn compare_canonical_exact(
    interpreter: &EffectObservationV1,
    native: &EffectObservationV1,
) -> Result<(), ObservationMismatch> {
    let mut mismatches = Vec::new();
    push_mismatch(
        &mut mismatches,
        ObservationField::Stdout,
        &interpreter.stdout,
        &native.stdout,
    );
    push_mismatch(
        &mut mismatches,
        ObservationField::Stderr,
        &interpreter.stderr,
        &native.stderr,
    );
    push_mismatch(
        &mut mismatches,
        ObservationField::CanonicalImported("filesystem_delta"),
        &interpreter.filesystem_delta,
        &native.filesystem_delta,
    );
    push_mismatch(
        &mut mismatches,
        ObservationField::CanonicalImported("terminal_error"),
        &interpreter.terminal_error,
        &native.terminal_error,
    );
    push_mismatch(
        &mut mismatches,
        ObservationField::CanonicalImported("ordered_effect_trace"),
        &interpreter.effect_trace,
        &native.effect_trace,
    );
    push_mismatch(
        &mut mismatches,
        ObservationField::ExitStatus,
        &interpreter.exit_status,
        &native.exit_status,
    );
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(ObservationMismatch { mismatches })
    }
}

fn push_mismatch<T: std::fmt::Debug + PartialEq>(
    mismatches: &mut Vec<FieldMismatch>,
    field: ObservationField,
    interpreter: &T,
    native: &T,
) {
    if interpreter != native {
        mismatches.push(FieldMismatch {
            field,
            interpreter: format!("{interpreter:?}"),
            native: format!("{native:?}"),
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalMutation {
    SilentSkip,
    DuplicatedResume,
    ReorderedEvents,
    StdoutStderrSwap,
    PathByteNormalization,
    WeakenedErrorIdentity,
    WrongCapabilityToken,
    DeniedBeforeHostAction,
    FilesystemMutationWithoutTrace,
    TraceWithoutFilesystemMutation,
    TargetEffectManifestMismatch,
    OperationStatusTransition,
    FileMetadataSize,
    FileMetadataKind,
    DirectoryEntryName,
    DirectoryEntryKind,
    UnknownFamilyIdentity,
    UnknownOperationIdentity,
    UnknownRawOperationId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalMutationError {
    pub mutation: CanonicalMutation,
    pub reason: &'static str,
}

/// One canonical observation and its deliberately weaker same-scenario return
/// proxy. Applying a mutation changes only the external observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalMutationControl<T> {
    pub observation: EffectObservationV1,
    pub runner_only: RunnerOnlyProxy<T>,
}

impl<T> CanonicalMutationControl<T> {
    pub fn apply(&mut self, mutation: CanonicalMutation) -> Result<(), CanonicalMutationError> {
        apply_canonical_mutation(&mut self.observation, mutation)
    }
}

/// Inject one discriminator directly into the imported canonical values.
pub fn apply_canonical_mutation(
    observation: &mut EffectObservationV1,
    mutation: CanonicalMutation,
) -> Result<(), CanonicalMutationError> {
    let missing = |reason| CanonicalMutationError { mutation, reason };
    match mutation {
        CanonicalMutation::SilentSkip => {
            if observation.effect_trace.is_empty() {
                return Err(missing("silent-skip control needs an effect event"));
            }
            observation.effect_trace.remove(0);
        }
        CanonicalMutation::DuplicatedResume => {
            let event = observation
                .effect_trace
                .first()
                .cloned()
                .ok_or_else(|| missing("duplicate-resume control needs an effect event"))?;
            observation.effect_trace.insert(1, event);
        }
        CanonicalMutation::ReorderedEvents => {
            if observation.effect_trace.len() < 2 {
                return Err(missing("reorder control needs two effect events"));
            }
            observation.effect_trace.swap(0, 1);
        }
        CanonicalMutation::StdoutStderrSwap => {
            std::mem::swap(&mut observation.stdout, &mut observation.stderr);
        }
        CanonicalMutation::PathByteNormalization => {
            let path = request_path_with_marker_mut(&mut observation.effect_trace)
                .or_else(|| first_delta_path_mut(&mut observation.filesystem_delta))
                .ok_or_else(|| missing("path control needs a raw request or delta path"))?;
            let marker = path
                .windows(3)
                .position(|window| window == b"/./")
                .ok_or_else(|| missing("path control needs a raw /./ marker"))?;
            path.drain(marker..marker + 2);
        }
        CanonicalMutation::WeakenedErrorIdentity => {
            let outcome = first_error_outcome_mut(&mut observation.effect_trace)
                .ok_or_else(|| missing("error control needs a semantic error"))?;
            *outcome = CanonicalOutcomeV1::Error(SemanticErrorV1::Io(IoErrorIdentityV1::Other(0)));
        }
        CanonicalMutation::WrongCapabilityToken => {
            let index = first_capability_event_index(&observation.effect_trace)
                .ok_or_else(|| missing("wrong-cap control needs a capability event"))?;
            let mut event = observation.effect_trace[index].clone();
            // Raw generational tokens are deliberately absent from the
            // canonical observation. Preserve the stable ProgramCaps trace
            // identity and discriminate the malformed token by enum outcome.
            event.outcome = CanonicalOutcomeV1::Error(SemanticErrorV1::Capability(
                CapabilityDeniedV1::MalformedCapability,
            ));
            observation.effect_trace.clear();
            observation.effect_trace.push(event);
            observation.filesystem_delta.clear();
        }
        CanonicalMutation::DeniedBeforeHostAction => {
            let index = first_capability_event_index(&observation.effect_trace)
                .ok_or_else(|| missing("denial control needs a capability event"))?;
            let mut event = observation.effect_trace[index].clone();
            event.outcome = CanonicalOutcomeV1::Error(SemanticErrorV1::Capability(
                CapabilityDeniedV1::RightNotHeld {
                    operation: FsCapabilityOperationV1::Write,
                    held_rights: 0,
                },
            ));
            observation.effect_trace.clear();
            observation.effect_trace.push(event);
            observation.filesystem_delta.clear();
        }
        CanonicalMutation::FilesystemMutationWithoutTrace => {
            observation.filesystem_delta.push(FsDeltaV1::Created {
                relative_path: b"mutation-without-trace".to_vec(),
                node: file_node(b"mutated"),
            });
        }
        CanonicalMutation::TraceWithoutFilesystemMutation => {
            observation.filesystem_delta.clear();
            let sequence = observation
                .effect_trace
                .last()
                .map(|event| event.sequence + 1)
                .unwrap_or(0);
            observation.effect_trace.push(EffectEventV1 {
                sequence,
                operation: HostOpV1::FsWriteFile,
                capability: Some(CapabilityTraceIdentity("program_caps.fs".to_string())),
                request: CanonicalRequestV1::FsWriteFile {
                    path: b"trace-without-mutation".to_vec(),
                    create_policy: CreatePolicyV1::CreateOrTruncate,
                    bytes: b"unobserved".to_vec(),
                },
                outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::Unit),
            });
        }
        CanonicalMutation::TargetEffectManifestMismatch => {
            observation.terminal_error = Some(TerminalErrorV1::HostEffectAbiMismatch);
            observation.effect_trace.clear();
            observation.filesystem_delta.clear();
        }
        CanonicalMutation::OperationStatusTransition => {
            observation.terminal_error = Some(TerminalErrorV1::OperationUnavailable(
                HostOpV1::FsAppendFile,
            ));
        }
        CanonicalMutation::FileMetadataSize => {
            let metadata = first_metadata_reply_mut(&mut observation.effect_trace)
                .ok_or_else(|| missing("metadata-size control needs FileMetadata"))?;
            metadata.size = metadata.size.wrapping_add(1);
        }
        CanonicalMutation::FileMetadataKind => {
            let metadata = first_metadata_reply_mut(&mut observation.effect_trace)
                .ok_or_else(|| missing("metadata-kind control needs FileMetadata"))?;
            metadata.kind = different_kind(metadata.kind);
        }
        CanonicalMutation::DirectoryEntryName => {
            let entry = first_directory_entry_mut(&mut observation.effect_trace)
                .ok_or_else(|| missing("directory-name control needs DirectoryEntries"))?;
            entry.name.push(0xff);
        }
        CanonicalMutation::DirectoryEntryKind => {
            let entry = first_directory_entry_mut(&mut observation.effect_trace)
                .ok_or_else(|| missing("directory-kind control needs DirectoryEntries"))?;
            entry.kind = different_kind(entry.kind);
        }
        CanonicalMutation::UnknownFamilyIdentity => {
            observation.terminal_error = Some(TerminalErrorV1::UnknownFamily {
                family: "different-family".to_string(),
            });
        }
        CanonicalMutation::UnknownOperationIdentity => {
            let raw_operation_id = match observation.terminal_error {
                Some(TerminalErrorV1::UnknownOperation {
                    raw_operation_id, ..
                }) => raw_operation_id,
                _ => 0xffff,
            };
            observation.terminal_error = Some(TerminalErrorV1::UnknownOperation {
                family: "different-family".to_string(),
                raw_operation_id,
            });
        }
        CanonicalMutation::UnknownRawOperationId => {
            let (family, raw_operation_id) = match &observation.terminal_error {
                Some(TerminalErrorV1::UnknownOperation {
                    family,
                    raw_operation_id,
                }) => (family.clone(), raw_operation_id.wrapping_add(1)),
                _ => ("FS".to_string(), 0xffff),
            };
            observation.terminal_error = Some(TerminalErrorV1::UnknownOperation {
                family,
                raw_operation_id,
            });
        }
    }
    Ok(())
}

/// Verify that a semantic capability denial occurred before any observable
/// host action. The runner seam and real-root state are both required.
pub fn denial_precedes_host_action(
    capture: &LaneActionEvidence,
    observation: &EffectObservationV1,
) -> bool {
    capture.fs_actions_after_resolve == Some(0)
        && capture.root_before == capture.root_after
        && observation.filesystem_delta.is_empty()
        && observation
            .effect_trace
            .iter()
            .filter(|event| is_fs_operation(event.operation))
            .all(|event| is_capability_denial(&event.outcome))
        && observation
            .effect_trace
            .iter()
            .any(|event| is_fs_operation(event.operation) && is_capability_denial(&event.outcome))
}

pub fn is_fail_closed_manifest_mismatch(observation: &EffectObservationV1) -> bool {
    observation.effect_trace.is_empty()
        && observation.filesystem_delta.is_empty()
        && matches!(
            observation.terminal_error,
            Some(TerminalErrorV1::TargetAbiMismatch | TerminalErrorV1::HostEffectAbiMismatch)
        )
}

fn first_delta_path_mut(deltas: &mut [FsDeltaV1]) -> Option<&mut Vec<u8>> {
    deltas.first_mut().map(|delta| match delta {
        FsDeltaV1::Created { relative_path, .. }
        | FsDeltaV1::Removed { relative_path, .. }
        | FsDeltaV1::Modified { relative_path, .. } => relative_path,
    })
}

fn request_path_mut(request: &mut CanonicalRequestV1) -> Option<&mut Vec<u8>> {
    match request {
        CanonicalRequestV1::FsReadFile { path }
        | CanonicalRequestV1::FsWriteFile { path, .. }
        | CanonicalRequestV1::FsAppendFile { path, .. }
        | CanonicalRequestV1::FsMetadata { path }
        | CanonicalRequestV1::FsReadDirectory { path }
        | CanonicalRequestV1::FsCreateDirectory { path, .. }
        | CanonicalRequestV1::FsRemoveFile { path }
        | CanonicalRequestV1::FsRemoveDirectory { path, .. } => Some(path),
        CanonicalRequestV1::FsRename { source, .. } => Some(source),
        _ => None,
    }
}

fn request_path_with_marker_mut(events: &mut [EffectEventV1]) -> Option<&mut Vec<u8>> {
    events.iter_mut().find_map(|event| {
        let path = request_path_mut(&mut event.request)?;
        path.windows(3)
            .any(|window| window == b"/./")
            .then_some(path)
    })
}

fn first_error_outcome_mut(events: &mut [EffectEventV1]) -> Option<&mut CanonicalOutcomeV1> {
    events
        .iter_mut()
        .find(|event| matches!(event.outcome, CanonicalOutcomeV1::Error(_)))
        .map(|event| &mut event.outcome)
}

fn first_capability_event_index(events: &[EffectEventV1]) -> Option<usize> {
    events.iter().position(|event| event.capability.is_some())
}

fn is_capability_denial(outcome: &CanonicalOutcomeV1) -> bool {
    matches!(
        outcome,
        CanonicalOutcomeV1::Error(SemanticErrorV1::Capability(_))
            | CanonicalOutcomeV1::Error(SemanticErrorV1::File(FileErrorIdentityV1 {
                cause: FileErrorCauseV1::Capability(_),
                ..
            }))
    )
}

fn is_fs_operation(operation: HostOpV1) -> bool {
    matches!(
        operation,
        HostOpV1::FsReadFile
            | HostOpV1::FsWriteFile
            | HostOpV1::FsAppendFile
            | HostOpV1::FsMetadata
            | HostOpV1::FsReadDirectory
            | HostOpV1::FsCreateDirectory
            | HostOpV1::FsRemoveFile
            | HostOpV1::FsRemoveDirectory
            | HostOpV1::FsRename
    )
}

fn first_metadata_reply_mut(events: &mut [EffectEventV1]) -> Option<&mut ken_host::FileMetadataV1> {
    events
        .iter_mut()
        .find_map(|event| match &mut event.outcome {
            CanonicalOutcomeV1::Success(CanonicalReplyV1::FileMetadata(metadata)) => Some(metadata),
            _ => None,
        })
}

fn first_directory_entry_mut(events: &mut [EffectEventV1]) -> Option<&mut DirEntryV1> {
    events
        .iter_mut()
        .find_map(|event| match &mut event.outcome {
            CanonicalOutcomeV1::Success(CanonicalReplyV1::DirectoryEntries(entries)) => {
                entries.first_mut()
            }
            _ => None,
        })
}

fn different_kind(kind: FsNodeKindV1) -> FsNodeKindV1 {
    match kind {
        FsNodeKindV1::File => FsNodeKindV1::Directory,
        FsNodeKindV1::Directory | FsNodeKindV1::Symlink | FsNodeKindV1::Other => FsNodeKindV1::File,
    }
}

fn file_node(bytes: &[u8]) -> FsNodeObservationV1 {
    FsNodeObservationV1 {
        kind: FsNodeKindV1::File,
        file_bytes: Some(bytes.to_vec()),
        symlink_target: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RootSnapshot, SnapshotNode, SnapshotNodeKind};
    use ken_host::FileMetadataV1;

    fn event(
        sequence: u64,
        operation: HostOpV1,
        capability: bool,
        request: CanonicalRequestV1,
        outcome: CanonicalOutcomeV1,
    ) -> EffectEventV1 {
        EffectEventV1 {
            sequence,
            operation,
            capability: capability.then(|| CapabilityTraceIdentity("program_caps.fs".to_string())),
            request,
            outcome,
        }
    }

    fn baseline() -> EffectObservationV1 {
        EffectObservationV1 {
            stdout: b"stdout".to_vec(),
            stderr: b"stderr".to_vec(),
            filesystem_delta: vec![FsDeltaV1::Modified {
                relative_path: vec![b'a', b'/', b'.', b'/', b'b', 0xff],
                before: file_node(b"before"),
                after: file_node(b"after"),
            }],
            terminal_error: Some(TerminalErrorV1::UnknownOperation {
                family: "FS".to_string(),
                raw_operation_id: 0xfff0,
            }),
            effect_trace: vec![
                event(
                    0,
                    HostOpV1::ConsoleWrite,
                    false,
                    CanonicalRequestV1::ConsoleWrite {
                        stream: ken_host::ConsoleStreamV1::Stdout,
                        bytes: b"stdout".to_vec(),
                    },
                    CanonicalOutcomeV1::Success(CanonicalReplyV1::Unit),
                ),
                event(
                    1,
                    HostOpV1::FsMetadata,
                    true,
                    CanonicalRequestV1::FsMetadata {
                        path: b"metadata".to_vec(),
                    },
                    CanonicalOutcomeV1::Success(CanonicalReplyV1::FileMetadata(FileMetadataV1 {
                        size: 7,
                        kind: FsNodeKindV1::File,
                    })),
                ),
                event(
                    2,
                    HostOpV1::FsReadDirectory,
                    true,
                    CanonicalRequestV1::FsReadDirectory {
                        path: b"directory".to_vec(),
                    },
                    CanonicalOutcomeV1::Success(CanonicalReplyV1::DirectoryEntries(vec![
                        DirEntryV1 {
                            name: vec![b'f', 0xff],
                            kind: FsNodeKindV1::File,
                        },
                    ])),
                ),
                event(
                    3,
                    HostOpV1::FsReadFile,
                    true,
                    CanonicalRequestV1::FsReadFile {
                        path: b"missing".to_vec(),
                    },
                    CanonicalOutcomeV1::Error(SemanticErrorV1::File(FileErrorIdentityV1 {
                        operation: HostOpV1::FsReadFile,
                        relative_path: b"missing".to_vec(),
                        cause: FileErrorCauseV1::Io(IoErrorIdentityV1::NotFound),
                    })),
                ),
            ],
            exit_status: 1,
        }
    }

    fn control() -> CanonicalMutationControl<i64> {
        CanonicalMutationControl {
            observation: baseline(),
            runner_only: RunnerOnlyProxy {
                scenario_identity: "same-checked-entry-and-input".to_string(),
                returned_value: 7,
            },
        }
    }

    #[test]
    fn every_canonical_mutation_bites_while_same_runner_proxy_stays_green() {
        for mutation in [
            CanonicalMutation::SilentSkip,
            CanonicalMutation::DuplicatedResume,
            CanonicalMutation::ReorderedEvents,
            CanonicalMutation::StdoutStderrSwap,
            CanonicalMutation::PathByteNormalization,
            CanonicalMutation::WeakenedErrorIdentity,
            CanonicalMutation::WrongCapabilityToken,
            CanonicalMutation::DeniedBeforeHostAction,
            CanonicalMutation::FilesystemMutationWithoutTrace,
            CanonicalMutation::TraceWithoutFilesystemMutation,
            CanonicalMutation::TargetEffectManifestMismatch,
            CanonicalMutation::OperationStatusTransition,
            CanonicalMutation::FileMetadataSize,
            CanonicalMutation::FileMetadataKind,
            CanonicalMutation::DirectoryEntryName,
            CanonicalMutation::DirectoryEntryKind,
            CanonicalMutation::UnknownFamilyIdentity,
            CanonicalMutation::UnknownOperationIdentity,
            CanonicalMutation::UnknownRawOperationId,
        ] {
            let oracle = control();
            let mut subject = oracle.clone();
            subject
                .apply(mutation)
                .expect("mutation fixture is complete");
            assert!(
                compare_canonical_exact(&oracle.observation, &subject.observation).is_err(),
                "canonical mutation {mutation:?} must be detected"
            );
            assert!(
                oracle.runner_only.agrees(&subject.runner_only),
                "same-scenario return proxy must stay green for {mutation:?}"
            );
        }
    }

    #[test]
    fn comparator_reports_enum_trace_delta_and_corrected_payload_identity() {
        for mutation in [
            CanonicalMutation::WeakenedErrorIdentity,
            CanonicalMutation::ReorderedEvents,
            CanonicalMutation::PathByteNormalization,
            CanonicalMutation::FileMetadataSize,
            CanonicalMutation::FileMetadataKind,
            CanonicalMutation::DirectoryEntryName,
            CanonicalMutation::DirectoryEntryKind,
            CanonicalMutation::UnknownFamilyIdentity,
            CanonicalMutation::UnknownOperationIdentity,
            CanonicalMutation::UnknownRawOperationId,
        ] {
            let oracle = baseline();
            let mut subject = oracle.clone();
            apply_canonical_mutation(&mut subject, mutation).expect("complete fixture");
            let error = compare_canonical_exact(&oracle, &subject).expect_err("must differ");
            assert!(!error.mismatches.is_empty(), "diagnostic for {mutation:?}");
        }
    }

    #[test]
    fn wrong_capability_and_denial_are_proven_before_host_action() {
        for mutation in [
            CanonicalMutation::WrongCapabilityToken,
            CanonicalMutation::DeniedBeforeHostAction,
        ] {
            let mut observation = baseline();
            let stable_identity = observation
                .effect_trace
                .iter()
                .find_map(|event| event.capability.clone())
                .expect("fixture capability identity");
            apply_canonical_mutation(&mut observation, mutation).expect("denial mutation");
            assert_eq!(
                observation.effect_trace[0].capability,
                Some(stable_identity),
                "raw tokens never replace the stable ProgramCaps trace identity"
            );
            let mut capture = LaneActionEvidence {
                root_before: RootSnapshot::default(),
                root_after: RootSnapshot::default(),
                fs_actions_after_resolve: Some(0),
            };
            assert!(denial_precedes_host_action(&capture, &observation));
            capture.fs_actions_after_resolve = Some(1);
            assert!(!denial_precedes_host_action(&capture, &observation));
            capture.fs_actions_after_resolve = Some(0);
            capture.root_after.nodes.push(SnapshotNode {
                relative_path: b"acted".to_vec(),
                kind: SnapshotNodeKind::File,
                bytes: Vec::new(),
            });
            assert!(!denial_precedes_host_action(&capture, &observation));
        }
    }

    #[test]
    fn target_and_effect_manifest_mismatch_fail_closed() {
        for terminal_error in [
            TerminalErrorV1::TargetAbiMismatch,
            TerminalErrorV1::HostEffectAbiMismatch,
        ] {
            let mut observation = baseline();
            observation.terminal_error = Some(terminal_error);
            observation.effect_trace.clear();
            observation.filesystem_delta.clear();
            assert!(is_fail_closed_manifest_mismatch(&observation));
        }
        let mut not_closed = baseline();
        not_closed.terminal_error = Some(TerminalErrorV1::HostEffectAbiMismatch);
        assert!(!is_fail_closed_manifest_mismatch(&not_closed));
    }
}
