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
        HostOpAvailabilityV1::RepresentedUnavailable
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
        assert!(HostOpV1::ALL.into_iter().all(|operation| {
            operation.availability() == HostOpAvailabilityV1::RepresentedUnavailable
        }));
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
