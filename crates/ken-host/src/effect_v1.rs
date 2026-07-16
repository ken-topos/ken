//! Canonical host-effect vocabulary shared by the interpreter, native runtime,
//! and the independent differential harness.
//!
//! These values are semantic observations, not the private raw-pointer ABI.
//! They deliberately exclude descriptors, pointers, inode identities, absolute
//! host roots, and diagnostic prose.

/// The closed Program-I host operation catalog.
///
/// Numeric values are explicit ABI identities. Declaration order is not an
/// ABI, and unknown numeric values must be rejected rather than coerced.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HostOpV1 {
    ConsoleRead = 0x0101,
    ConsoleWrite = 0x0102,
    ConsoleFlush = 0x0103,
    ConsoleIsTerminal = 0x0104,
    ClockWallNow = 0x0201,
    FsReadFile = 0x0301,
    FsWriteFile = 0x0302,
    FsAppendFile = 0x0303,
    FsMetadata = 0x0304,
    FsReadDirectory = 0x0305,
    FsCreateDirectory = 0x0306,
    FsRemoveFile = 0x0307,
    FsRemoveDirectory = 0x0308,
    FsRename = 0x0309,
}

impl HostOpV1 {
    pub const ALL: [Self; 14] = [
        Self::ConsoleRead,
        Self::ConsoleWrite,
        Self::ConsoleFlush,
        Self::ConsoleIsTerminal,
        Self::ClockWallNow,
        Self::FsReadFile,
        Self::FsWriteFile,
        Self::FsAppendFile,
        Self::FsMetadata,
        Self::FsReadDirectory,
        Self::FsCreateDirectory,
        Self::FsRemoveFile,
        Self::FsRemoveDirectory,
        Self::FsRename,
    ];

    pub const fn availability(self) -> HostOpAvailabilityV1 {
        if matches!(
            self,
            Self::ConsoleWrite
                | Self::ConsoleFlush
                | Self::ConsoleIsTerminal
                | Self::FsReadFile
                | Self::FsWriteFile
        ) {
            HostOpAvailabilityV1::NativeTested
        } else {
            HostOpAvailabilityV1::RepresentedUnavailable
        }
    }

    pub const fn is_ambient(self) -> bool {
        matches!(
            self,
            Self::ConsoleRead
                | Self::ConsoleWrite
                | Self::ConsoleFlush
                | Self::ConsoleIsTerminal
                | Self::ClockWallNow
        )
    }
}

/// PX5's intended promotion set. Membership is a plan, not evidence: every
/// operation remains `RepresentedUnavailable` until its artifact differential
/// gates promote it explicitly.
pub const PX5_PLANNED_NATIVE_TARGETS: [HostOpV1; 5] = [
    HostOpV1::ConsoleWrite,
    HostOpV1::ConsoleFlush,
    HostOpV1::ConsoleIsTerminal,
    HostOpV1::FsReadFile,
    HostOpV1::FsWriteFile,
];

pub const HOST_EFFECT_ABI_V1_SCHEMA_VERSION: u32 = 1;
pub const HOST_EFFECT_ABI_V1_HASH: [u8; 32] = [
    0x6b, 0x65, 0x6e, 0x2d, 0x68, 0x6f, 0x73, 0x74, 0x2d, 0x65, 0x66, 0x66, 0x65, 0x63, 0x74, 0x2d,
    0x76, 0x31, 0x00, 0x0e, 0x00, 0x05, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostEffectAbiV1 {
    pub schema_version: u32,
    pub operation_count: u16,
    pub native_tested_count: u16,
    pub capability_token_size: u16,
    pub capability_token_align: u16,
    pub response_arena_lifetime_version: u16,
    pub trace_schema_version: u16,
    pub manifest_hash: [u8; 32],
}

pub const HOST_EFFECT_ABI_V1: HostEffectAbiV1 = HostEffectAbiV1 {
    schema_version: HOST_EFFECT_ABI_V1_SCHEMA_VERSION,
    operation_count: 14,
    native_tested_count: 5,
    capability_token_size: std::mem::size_of::<CapabilityTokenV1>() as u16,
    capability_token_align: std::mem::align_of::<CapabilityTokenV1>() as u16,
    response_arena_lifetime_version: 1,
    trace_schema_version: 1,
    manifest_hash: HOST_EFFECT_ABI_V1_HASH,
};

pub fn assert_host_effect_abi_identity(hash: [u8; 32]) -> Result<(), TerminalErrorV1> {
    if hash == HOST_EFFECT_ABI_V1_HASH {
        Ok(())
    } else {
        Err(TerminalErrorV1::HostEffectAbiMismatch)
    }
}

impl TryFrom<u16> for HostOpV1 {
    type Error = UnknownHostOpV1;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::ALL
            .into_iter()
            .find(|operation| *operation as u16 == value)
            .ok_or(UnknownHostOpV1(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownHostOpV1(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostOpAvailabilityV1 {
    NativeTested,
    RepresentedUnavailable,
}

/// Opaque capability carrier used at the private native host boundary.
/// Generated code may copy this value only into a dispatch request.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CapabilityTokenV1 {
    slot: u32,
    generation: u32,
}

impl CapabilityTokenV1 {
    pub fn erased_identity(self) -> u64 {
        (u64::from(self.generation) << 32) | u64::from(self.slot)
    }

    /// Rehydrates only the private host-wire representation. Ken code never
    /// receives this constructor or either component.
    pub(crate) fn from_erased_identity(identity: u64) -> Self {
        Self {
            slot: identity as u32,
            generation: (identity >> 32) as u32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityGrantV1 {
    pub identity: CapabilityTraceIdentity,
    pub capability: crate::Cap,
}

pub const RIGHT_READ_V1: u8 = crate::RightSet::READ.bits();
pub const RIGHT_WRITE_V1: u8 = crate::RightSet::WRITE.union(crate::RightSet::CREATE).bits();

#[derive(Clone, Debug, Default)]
pub struct CapabilityTableV1 {
    slots: Vec<CapabilitySlotV1>,
}

#[derive(Clone, Debug)]
struct CapabilitySlotV1 {
    generation: u32,
    grant: CapabilityGrantV1,
}

impl CapabilityTableV1 {
    /// Minting is runner-only. Tokens are never constructible from Ken data.
    pub fn insert(&mut self, grant: CapabilityGrantV1) -> CapabilityTokenV1 {
        let slot = u32::try_from(self.slots.len()).expect("capability table exceeds u32");
        let generation = 1;
        self.slots.push(CapabilitySlotV1 { generation, grant });
        CapabilityTokenV1 { slot, generation }
    }

    pub fn resolve(
        &self,
        token: CapabilityTokenV1,
    ) -> Result<&CapabilityGrantV1, CapabilityDeniedV1> {
        self.slots
            .get(token.slot as usize)
            .filter(|slot| slot.generation == token.generation)
            .map(|slot| &slot.grant)
            .ok_or(CapabilityDeniedV1::MalformedCapability)
    }
}

/// Host leaves behind the single semantic dispatcher.
pub trait HostEffectBackendV1 {
    fn console_write(
        &mut self,
        stream: ConsoleStreamV1,
        bytes: &[u8],
    ) -> Result<(), IoErrorIdentityV1>;
    fn console_flush(&mut self, stream: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1>;
    fn console_is_terminal(&mut self, stream: ConsoleStreamV1) -> bool;
    fn fs_read_file(
        &mut self,
        grant: &CapabilityGrantV1,
        path: &[u8],
    ) -> Result<Vec<u8>, FileErrorCauseV1>;
    fn fs_write_file(
        &mut self,
        grant: &CapabilityGrantV1,
        path: &[u8],
        create_policy: CreatePolicyV1,
        bytes: &[u8],
    ) -> Result<(), FileErrorCauseV1>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDispatchReplyV1 {
    pub capability_identity: Option<CapabilityTraceIdentity>,
    pub outcome: CanonicalOutcomeV1,
}

/// The only V1 semantic operation switch. Validation and capability denial
/// happen before a backend leaf is invoked.
pub fn dispatch_host_op_v1<B: HostEffectBackendV1>(
    backend: &mut B,
    capabilities: &CapabilityTableV1,
    operation: HostOpV1,
    capability: Option<CapabilityTokenV1>,
    request: &CanonicalRequestV1,
) -> Result<HostDispatchReplyV1, TerminalErrorV1> {
    if operation.availability() != HostOpAvailabilityV1::NativeTested {
        return Err(TerminalErrorV1::OperationUnavailable(operation));
    }
    let required = match operation {
        HostOpV1::FsReadFile => Some((crate::FsCapabilityOperation::Read, crate::AUTH_PARTIAL)),
        HostOpV1::FsWriteFile => Some((crate::FsCapabilityOperation::Write, crate::AUTH_FULL)),
        _ => None,
    };
    let grant = match (required, capability) {
        (None, None) => None,
        (None, Some(_)) | (Some(_), None) => {
            return Ok(denied(
                operation,
                request,
                CapabilityDeniedV1::MalformedCapability,
            ))
        }
        (Some((op, authority)), Some(token)) => match capabilities.resolve(token) {
            Ok(grant) => {
                match crate::capability::check_fs_capability(&grant.capability, op, authority) {
                    Ok(_) => Some(grant),
                    Err(error) => {
                        return Ok(denied(operation, request, map_capability_denial(error)))
                    }
                }
            }
            Err(error) => return Ok(denied(operation, request, error)),
        },
    };
    let outcome = match (operation, request) {
        (HostOpV1::ConsoleWrite, CanonicalRequestV1::ConsoleWrite { stream, bytes }) => backend
            .console_write(*stream, bytes)
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(SemanticErrorV1::Io),
        (HostOpV1::ConsoleFlush, CanonicalRequestV1::ConsoleFlush { stream }) => backend
            .console_flush(*stream)
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(SemanticErrorV1::Io),
        (HostOpV1::ConsoleIsTerminal, CanonicalRequestV1::ConsoleIsTerminal { stream }) => {
            Ok(CanonicalReplyV1::Bool(backend.console_is_terminal(*stream)))
        }
        (HostOpV1::FsReadFile, CanonicalRequestV1::FsReadFile { path }) => backend
            .fs_read_file(grant.expect("validated FS capability"), path)
            .map(CanonicalReplyV1::Bytes)
            .map_err(|cause| file_error(operation, path, cause)),
        (
            HostOpV1::FsWriteFile,
            CanonicalRequestV1::FsWriteFile {
                path,
                create_policy,
                bytes,
            },
        ) => backend
            .fs_write_file(
                grant.expect("validated FS capability"),
                path,
                *create_policy,
                bytes,
            )
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(|cause| file_error(operation, path, cause)),
        _ => return Err(TerminalErrorV1::MalformedHostAbiField),
    };
    Ok(HostDispatchReplyV1 {
        capability_identity: grant.map(|grant| grant.identity.clone()),
        outcome: match outcome {
            Ok(reply) => CanonicalOutcomeV1::Success(reply),
            Err(error) => CanonicalOutcomeV1::Error(error),
        },
    })
}

fn map_capability_denial(error: crate::CapabilityDenied) -> CapabilityDeniedV1 {
    match error {
        crate::CapabilityDenied::RightNotHeld { op, held_rights } => {
            CapabilityDeniedV1::RightNotHeld {
                operation: match op {
                    crate::FsCapabilityOperation::Read => FsCapabilityOperationV1::Read,
                    crate::FsCapabilityOperation::Write => FsCapabilityOperationV1::Write,
                    crate::FsCapabilityOperation::Append => FsCapabilityOperationV1::Append,
                    crate::FsCapabilityOperation::Metadata => FsCapabilityOperationV1::Metadata,
                    crate::FsCapabilityOperation::Enumerate => FsCapabilityOperationV1::Enumerate,
                    crate::FsCapabilityOperation::CreateDirectory => {
                        FsCapabilityOperationV1::CreateDirectory
                    }
                    crate::FsCapabilityOperation::RemoveFile => FsCapabilityOperationV1::RemoveFile,
                    crate::FsCapabilityOperation::RemoveDirectory => {
                        FsCapabilityOperationV1::RemoveDirectory
                    }
                    crate::FsCapabilityOperation::RenameSource => {
                        FsCapabilityOperationV1::RenameSource
                    }
                    crate::FsCapabilityOperation::RenameDestination => {
                        FsCapabilityOperationV1::RenameDestination
                    }
                },
                held_rights,
            }
        }
        crate::CapabilityDenied::ScopeEscape => CapabilityDeniedV1::ScopeEscape,
        crate::CapabilityDenied::SymlinkDenied => CapabilityDeniedV1::SymlinkDenied,
        crate::CapabilityDenied::AuthorityInsufficient => CapabilityDeniedV1::AuthorityInsufficient,
        crate::CapabilityDenied::MalformedCapability => CapabilityDeniedV1::MalformedCapability,
    }
}

fn file_error(operation: HostOpV1, path: &[u8], cause: FileErrorCauseV1) -> SemanticErrorV1 {
    SemanticErrorV1::File(FileErrorIdentityV1 {
        operation,
        relative_path: path.to_vec(),
        cause,
    })
}

fn denied(
    operation: HostOpV1,
    request: &CanonicalRequestV1,
    error: CapabilityDeniedV1,
) -> HostDispatchReplyV1 {
    let path = match request {
        CanonicalRequestV1::FsReadFile { path } | CanonicalRequestV1::FsWriteFile { path, .. } => {
            path.clone()
        }
        _ => Vec::new(),
    };
    HostDispatchReplyV1 {
        capability_identity: None,
        outcome: CanonicalOutcomeV1::Error(file_error(
            operation,
            &path,
            FileErrorCauseV1::Capability(error),
        )),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConsoleStreamV1 {
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CreatePolicyV1 {
    CreateNew,
    CreateOrTruncate,
    CreateOrKeep,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CapabilityTraceIdentity(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalRequestV1 {
    ConsoleRead {
        stream: ConsoleStreamV1,
        limit: u64,
    },
    ConsoleWrite {
        stream: ConsoleStreamV1,
        bytes: Vec<u8>,
    },
    ConsoleFlush {
        stream: ConsoleStreamV1,
    },
    ConsoleIsTerminal {
        stream: ConsoleStreamV1,
    },
    ClockWallNow,
    FsReadFile {
        path: Vec<u8>,
    },
    FsWriteFile {
        path: Vec<u8>,
        create_policy: CreatePolicyV1,
        bytes: Vec<u8>,
    },
    FsAppendFile {
        path: Vec<u8>,
        bytes: Vec<u8>,
    },
    FsMetadata {
        path: Vec<u8>,
    },
    FsReadDirectory {
        path: Vec<u8>,
    },
    FsCreateDirectory {
        recursive: bool,
        path: Vec<u8>,
    },
    FsRemoveFile {
        path: Vec<u8>,
    },
    FsRemoveDirectory {
        recursive: bool,
        path: Vec<u8>,
    },
    FsRename {
        source: Vec<u8>,
        destination: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FsCapabilityOperationV1 {
    Read,
    Write,
    Append,
    Metadata,
    Enumerate,
    CreateDirectory,
    RemoveFile,
    RemoveDirectory,
    RenameSource,
    RenameDestination,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IoErrorIdentityV1 {
    NotFound,
    PermissionDenied,
    BrokenPipe,
    Interrupted,
    AlreadyExists,
    InvalidInput,
    IsDirectory,
    NotDirectory,
    NotEmpty,
    Unsupported,
    Other(i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityDeniedV1 {
    RightNotHeld {
        operation: FsCapabilityOperationV1,
        held_rights: u8,
    },
    AuthorityInsufficient,
    ScopeEscape,
    SymlinkDenied,
    MalformedCapability,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileErrorIdentityV1 {
    pub operation: HostOpV1,
    pub relative_path: Vec<u8>,
    pub cause: FileErrorCauseV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileErrorCauseV1 {
    Io(IoErrorIdentityV1),
    Capability(CapabilityDeniedV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticErrorV1 {
    Io(IoErrorIdentityV1),
    File(FileErrorIdentityV1),
    Capability(CapabilityDeniedV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalReplyV1 {
    Unit,
    Bool(bool),
    Bytes(Vec<u8>),
    ReadChunk(Vec<u8>),
    ReadEof,
    Instant(Vec<u8>),
    FileMetadata(FileMetadataV1),
    DirectoryEntries(Vec<DirEntryV1>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileMetadataV1 {
    pub size: u64,
    pub kind: FsNodeKindV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntryV1 {
    pub name: Vec<u8>,
    pub kind: FsNodeKindV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalOutcomeV1 {
    Success(CanonicalReplyV1),
    Error(SemanticErrorV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectEventV1 {
    pub sequence: u64,
    pub operation: HostOpV1,
    pub capability: Option<CapabilityTraceIdentity>,
    pub request: CanonicalRequestV1,
    pub outcome: CanonicalOutcomeV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FsNodeKindV1 {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsNodeObservationV1 {
    pub kind: FsNodeKindV1,
    pub file_bytes: Option<Vec<u8>>,
    pub symlink_target: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsDeltaV1 {
    Created {
        relative_path: Vec<u8>,
        node: FsNodeObservationV1,
    },
    Removed {
        relative_path: Vec<u8>,
        node: FsNodeObservationV1,
    },
    Modified {
        relative_path: Vec<u8>,
        before: FsNodeObservationV1,
        after: FsNodeObservationV1,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalErrorV1 {
    UnknownFamily {
        family: String,
    },
    UnknownOperation {
        family: String,
        raw_operation_id: u16,
    },
    UnknownTree,
    MalformedTree,
    MalformedHostAbiField,
    TargetAbiMismatch,
    HostEffectAbiMismatch,
    OperationUnavailable(HostOpV1),
    RuntimeTrap(u16),
    DriverFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectObservationV1 {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub filesystem_delta: Vec<FsDeltaV1>,
    pub terminal_error: Option<TerminalErrorV1>,
    pub effect_trace: Vec<EffectEventV1>,
    pub exit_status: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_closed_and_availability_is_exact() {
        assert_eq!(HostOpV1::ALL.len(), 14);
        assert_eq!(
            HostOpV1::ALL
                .into_iter()
                .filter(|operation| operation.availability() == HostOpAvailabilityV1::NativeTested)
                .collect::<Vec<_>>(),
            PX5_PLANNED_NATIVE_TARGETS
        );
        assert_eq!(
            PX5_PLANNED_NATIVE_TARGETS,
            [
                HostOpV1::ConsoleWrite,
                HostOpV1::ConsoleFlush,
                HostOpV1::ConsoleIsTerminal,
                HostOpV1::FsReadFile,
                HostOpV1::FsWriteFile,
            ]
        );
        for operation in HostOpV1::ALL {
            assert_eq!(HostOpV1::try_from(operation as u16), Ok(operation));
        }
        assert_eq!(HostOpV1::try_from(0), Err(UnknownHostOpV1(0)));
    }

    #[test]
    fn canonical_payloads_and_unknown_identities_preserve_discriminators() {
        assert_ne!(
            CanonicalReplyV1::FileMetadata(FileMetadataV1 {
                size: 1,
                kind: FsNodeKindV1::File,
            }),
            CanonicalReplyV1::FileMetadata(FileMetadataV1 {
                size: 2,
                kind: FsNodeKindV1::File,
            })
        );
        assert_ne!(
            CanonicalReplyV1::FileMetadata(FileMetadataV1 {
                size: 1,
                kind: FsNodeKindV1::File,
            }),
            CanonicalReplyV1::FileMetadata(FileMetadataV1 {
                size: 1,
                kind: FsNodeKindV1::Directory,
            })
        );
        assert_ne!(
            CanonicalReplyV1::DirectoryEntries(vec![DirEntryV1 {
                name: b"a".to_vec(),
                kind: FsNodeKindV1::File,
            }]),
            CanonicalReplyV1::DirectoryEntries(vec![DirEntryV1 {
                name: b"b".to_vec(),
                kind: FsNodeKindV1::File,
            }])
        );
        assert_ne!(
            CanonicalReplyV1::DirectoryEntries(vec![DirEntryV1 {
                name: b"a".to_vec(),
                kind: FsNodeKindV1::File,
            }]),
            CanonicalReplyV1::DirectoryEntries(vec![DirEntryV1 {
                name: b"a".to_vec(),
                kind: FsNodeKindV1::Directory,
            }])
        );
        assert_ne!(
            TerminalErrorV1::UnknownFamily {
                family: "FS".to_owned(),
            },
            TerminalErrorV1::UnknownFamily {
                family: "Console".to_owned(),
            }
        );
        assert_ne!(
            TerminalErrorV1::UnknownOperation {
                family: "FS".to_owned(),
                raw_operation_id: 1,
            },
            TerminalErrorV1::UnknownOperation {
                family: "FS".to_owned(),
                raw_operation_id: 2,
            }
        );
    }
}
