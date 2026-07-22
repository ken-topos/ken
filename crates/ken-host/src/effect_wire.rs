//! Total private wire codec for one completed linked-artifact effect trace.

use crate::{
    CanonicalOutcomeV1, CanonicalReplyV1, CanonicalRequestV1, CapabilityDeniedV1,
    CapabilityTraceIdentity, ConsoleStreamV1, CreatePolicyV1, DirEntryV1, EffectEvent,
    FileErrorCauseV1, FileErrorIdentityV1, FileMetadataV1, FsCapabilityOperationV1, FsNodeKindV1,
    FsOpenModeV1, HostOpV1, IoErrorIdentityV1, ResourceBindingRole, ResourceErrorV1,
    ResourceKindV1, ResourceSettlementObservationV1, ResourceSettlementOutcomeV1,
    ResourceTraceIdentityV1, SemanticErrorV1, TerminalExitClass,
};

const MAGIC: &[u8; 8] = b"KETRACE2";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedEffectTrace {
    pub plan_hash: u64,
    pub target_abi_hash: [u8; 32],
    pub host_effect_abi_hash: [u8; 32],
    pub terminal_value: i64,
    pub terminal_error: Option<crate::TerminalErrorV1>,
    pub effect_trace: Vec<EffectEvent>,
    pub terminal_exit: TerminalExitClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectTraceWireError;

fn put_role(out: &mut Vec<u8>, role: ResourceBindingRole) {
    put_u8(
        out,
        match role {
            ResourceBindingRole::File => 0,
            ResourceBindingRole::Buffer => 1,
            ResourceBindingRole::Target => 2,
        },
    );
}

fn put_terminal_exit(out: &mut Vec<u8>, class: TerminalExitClass) {
    put_u8(
        out,
        match class {
            TerminalExitClass::NormalReturn => 0,
            TerminalExitClass::ReturnedError => 1,
            TerminalExitClass::ControlledTrap => 2,
        },
    );
}

fn put_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), EffectTraceWireError> {
    put_u64(
        out,
        u64::try_from(bytes.len()).map_err(|_| EffectTraceWireError)?,
    );
    out.extend_from_slice(bytes);
    Ok(())
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), EffectTraceWireError> {
    put_bytes(out, value.as_bytes())
}

fn stream_tag(stream: ConsoleStreamV1) -> u8 {
    match stream {
        ConsoleStreamV1::Stdin => 0,
        ConsoleStreamV1::Stdout => 1,
        ConsoleStreamV1::Stderr => 2,
    }
}

fn put_request(
    out: &mut Vec<u8>,
    request: &CanonicalRequestV1,
) -> Result<(), EffectTraceWireError> {
    match request {
        CanonicalRequestV1::ConsoleRead { stream, limit } => {
            put_u8(out, 0);
            put_u8(out, stream_tag(*stream));
            put_u64(out, *limit);
        }
        CanonicalRequestV1::ConsoleWrite { stream, bytes } => {
            put_u8(out, 1);
            put_u8(out, stream_tag(*stream));
            put_bytes(out, bytes)?;
        }
        CanonicalRequestV1::ConsoleFlush { stream } => {
            put_u8(out, 2);
            put_u8(out, stream_tag(*stream));
        }
        CanonicalRequestV1::ConsoleIsTerminal { stream } => {
            put_u8(out, 3);
            put_u8(out, stream_tag(*stream));
        }
        CanonicalRequestV1::ClockWallNow => put_u8(out, 4),
        CanonicalRequestV1::FsReadFile { path } => {
            put_u8(out, 5);
            put_bytes(out, path)?;
        }
        CanonicalRequestV1::FsWriteFile {
            path,
            create_policy,
            bytes,
        } => {
            put_u8(out, 6);
            put_bytes(out, path)?;
            put_u8(
                out,
                match create_policy {
                    CreatePolicyV1::CreateNew => 0,
                    CreatePolicyV1::CreateOrTruncate => 1,
                    CreatePolicyV1::CreateOrKeep => 2,
                },
            );
            put_bytes(out, bytes)?;
        }
        CanonicalRequestV1::FsAppendFile { path, bytes } => {
            put_u8(out, 7);
            put_bytes(out, path)?;
            put_bytes(out, bytes)?;
        }
        CanonicalRequestV1::FsMetadata { path } => {
            put_u8(out, 8);
            put_bytes(out, path)?;
        }
        CanonicalRequestV1::FsReadDirectory { path } => {
            put_u8(out, 9);
            put_bytes(out, path)?;
        }
        CanonicalRequestV1::FsCreateDirectory { recursive, path } => {
            put_u8(out, 10);
            put_u8(out, u8::from(*recursive));
            put_bytes(out, path)?;
        }
        CanonicalRequestV1::FsRemoveFile { path } => {
            put_u8(out, 11);
            put_bytes(out, path)?;
        }
        CanonicalRequestV1::FsRemoveDirectory { recursive, path } => {
            put_u8(out, 12);
            put_u8(out, u8::from(*recursive));
            put_bytes(out, path)?;
        }
        CanonicalRequestV1::FsRename {
            source,
            destination,
        } => {
            put_u8(out, 13);
            put_bytes(out, source)?;
            put_bytes(out, destination)?;
        }
        CanonicalRequestV1::FsChangeMode { path, mode } => {
            put_u8(out, 14);
            put_bytes(out, path)?;
            put_u16(out, *mode);
        }
        CanonicalRequestV1::FsOpen { path, mode } => {
            put_u8(out, 15);
            put_bytes(out, path)?;
            put_u8(
                out,
                match mode {
                    FsOpenModeV1::Read => 0,
                    FsOpenModeV1::Metadata => 1,
                    FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateNew) => 2,
                    FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrTruncate) => 3,
                    FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrKeep) => 4,
                },
            );
        }
        CanonicalRequestV1::FsHandleMetadata => put_u8(out, 16),
        CanonicalRequestV1::ResourceRelease => put_u8(out, 17),
        CanonicalRequestV1::FsReadAt {
            file_offset,
            buffer_start,
            length,
        } => {
            put_u8(out, 18);
            put_u64(out, *file_offset);
            put_u64(out, *buffer_start);
            put_u64(out, *length);
        }
        CanonicalRequestV1::FsWriteAt {
            file_offset,
            buffer_start,
            length,
        } => {
            put_u8(out, 19);
            put_u64(out, *file_offset);
            put_u64(out, *buffer_start);
            put_u64(out, *length);
        }
        CanonicalRequestV1::BufferAllocate { capacity } => {
            put_u8(out, 20);
            put_u64(out, *capacity);
        }
        CanonicalRequestV1::BufferFreeze { start, length } => {
            put_u8(out, 21);
            put_u64(out, *start);
            put_u64(out, *length);
        }
    }
    Ok(())
}

fn put_node_kind(out: &mut Vec<u8>, kind: FsNodeKindV1) {
    put_u8(
        out,
        match kind {
            FsNodeKindV1::File => 0,
            FsNodeKindV1::Directory => 1,
            FsNodeKindV1::Symlink => 2,
            FsNodeKindV1::Other => 3,
        },
    );
}

fn put_reply(out: &mut Vec<u8>, reply: &CanonicalReplyV1) -> Result<(), EffectTraceWireError> {
    match reply {
        CanonicalReplyV1::Unit => put_u8(out, 0),
        CanonicalReplyV1::Bool(value) => {
            put_u8(out, 1);
            put_u8(out, u8::from(*value));
        }
        CanonicalReplyV1::Bytes(bytes) => {
            put_u8(out, 2);
            put_bytes(out, bytes)?;
        }
        CanonicalReplyV1::ReadChunk(bytes) => {
            put_u8(out, 3);
            put_bytes(out, bytes)?;
        }
        CanonicalReplyV1::ReadEof => put_u8(out, 4),
        CanonicalReplyV1::Instant(bytes) => {
            put_u8(out, 5);
            put_bytes(out, bytes)?;
        }
        CanonicalReplyV1::FileMetadata(metadata) => {
            put_u8(out, 6);
            put_u64(out, metadata.size);
            put_node_kind(out, metadata.kind);
        }
        CanonicalReplyV1::DirectoryEntries(entries) => {
            put_u8(out, 7);
            put_u64(
                out,
                u64::try_from(entries.len()).map_err(|_| EffectTraceWireError)?,
            );
            for entry in entries {
                put_bytes(out, &entry.name)?;
                put_node_kind(out, entry.kind);
            }
        }
        CanonicalReplyV1::ResourceAcquired {
            schema_version,
            resource_kind,
            identity,
        } => {
            put_u8(out, 8);
            put_u16(out, *schema_version);
            put_resource_kind(out, *resource_kind);
            put_u64(out, identity.0);
        }
        CanonicalReplyV1::ResourceSettlement(settlement) => {
            put_u8(out, 9);
            put_settlement(out, settlement);
        }
        CanonicalReplyV1::ReadProgress(progress) => {
            put_u8(out, 10);
            match progress {
                crate::ReadProgressV1::ReadSome { span, transferred } => {
                    put_u8(out, 0);
                    put_u64(out, span.start());
                    put_u64(out, span.length());
                    put_u64(out, transferred.get());
                    put_u64(out, transferred.effective_request());
                }
                crate::ReadProgressV1::ReadEof => put_u8(out, 1),
            }
        }
        CanonicalReplyV1::WriteProgress(crate::WriteProgressV1::Wrote(transferred)) => {
            put_u8(out, 11);
            put_u64(out, transferred.get());
            put_u64(out, transferred.effective_request());
        }
    }
    Ok(())
}

fn put_resource_kind(out: &mut Vec<u8>, kind: ResourceKindV1) {
    match kind {
        ResourceKindV1::FsHandle => put_u8(out, 0),
        ResourceKindV1::Buffer => put_u8(out, 1),
    }
}

fn put_settlement(out: &mut Vec<u8>, settlement: &ResourceSettlementObservationV1) {
    put_u16(out, settlement.schema_version);
    put_resource_kind(out, settlement.resource_kind);
    put_u64(out, settlement.identity.0);
    match settlement.outcome {
        ResourceSettlementOutcomeV1::Released => put_u8(out, 0),
        ResourceSettlementOutcomeV1::ReleaseFailed(io) => {
            put_u8(out, 1);
            put_io_error(out, io);
        }
    }
}

fn put_io_error(out: &mut Vec<u8>, error: IoErrorIdentityV1) {
    let (tag, raw) = match error {
        IoErrorIdentityV1::NotFound => (0, None),
        IoErrorIdentityV1::PermissionDenied => (1, None),
        IoErrorIdentityV1::BrokenPipe => (2, None),
        IoErrorIdentityV1::Interrupted => (3, None),
        IoErrorIdentityV1::AlreadyExists => (4, None),
        IoErrorIdentityV1::InvalidInput => (5, None),
        IoErrorIdentityV1::IsDirectory => (6, None),
        IoErrorIdentityV1::NotDirectory => (7, None),
        IoErrorIdentityV1::NotEmpty => (8, None),
        IoErrorIdentityV1::Unsupported => (9, None),
        IoErrorIdentityV1::Other(raw) => (10, Some(raw)),
    };
    put_u8(out, tag);
    if let Some(raw) = raw {
        put_i32(out, raw);
    }
}

fn fs_operation_tag(operation: FsCapabilityOperationV1) -> u8 {
    match operation {
        FsCapabilityOperationV1::Read => 0,
        FsCapabilityOperationV1::Write => 1,
        FsCapabilityOperationV1::Append => 2,
        FsCapabilityOperationV1::Metadata => 3,
        FsCapabilityOperationV1::Enumerate => 4,
        FsCapabilityOperationV1::CreateDirectory => 5,
        FsCapabilityOperationV1::RemoveFile => 6,
        FsCapabilityOperationV1::RemoveDirectory => 7,
        FsCapabilityOperationV1::RenameSource => 8,
        FsCapabilityOperationV1::RenameDestination => 9,
        FsCapabilityOperationV1::ChangeMode => 10,
    }
}

fn put_denial(out: &mut Vec<u8>, denial: &CapabilityDeniedV1) {
    match denial {
        CapabilityDeniedV1::RightNotHeld {
            operation,
            held_rights,
        } => {
            put_u8(out, 0);
            put_u8(out, fs_operation_tag(*operation));
            put_u8(out, *held_rights);
        }
        CapabilityDeniedV1::AuthorityInsufficient => put_u8(out, 1),
        CapabilityDeniedV1::ScopeEscape => put_u8(out, 2),
        CapabilityDeniedV1::SymlinkDenied => put_u8(out, 3),
        CapabilityDeniedV1::MalformedCapability => put_u8(out, 4),
    }
}

fn put_cause(out: &mut Vec<u8>, cause: &FileErrorCauseV1) {
    match cause {
        FileErrorCauseV1::Io(error) => {
            put_u8(out, 0);
            put_io_error(out, *error);
        }
        FileErrorCauseV1::Capability(error) => {
            put_u8(out, 1);
            put_denial(out, error);
        }
    }
}

fn put_error(out: &mut Vec<u8>, error: &SemanticErrorV1) -> Result<(), EffectTraceWireError> {
    match error {
        SemanticErrorV1::Io(error) => {
            put_u8(out, 0);
            put_io_error(out, *error);
        }
        SemanticErrorV1::File(error) => {
            put_u8(out, 1);
            put_u16(out, error.operation as u16);
            put_bytes(out, &error.relative_path)?;
            put_cause(out, &error.cause);
        }
        SemanticErrorV1::Capability(error) => {
            put_u8(out, 2);
            put_denial(out, error);
        }
        SemanticErrorV1::Resource(error) => {
            put_u8(out, 3);
            match error {
                ResourceErrorV1::Closed => put_u8(out, 0),
                ResourceErrorV1::MalformedResource => put_u8(out, 1),
                ResourceErrorV1::RightNotHeld { required, held } => {
                    put_u8(out, 2);
                    put_u8(out, *required);
                    put_u8(out, *held);
                }
                ResourceErrorV1::ReleaseFailed {
                    schema_version,
                    resource_kind,
                    identity,
                    io,
                } => {
                    put_u8(out, 3);
                    put_u16(out, *schema_version);
                    put_resource_kind(out, *resource_kind);
                    put_u64(out, identity.0);
                    put_io_error(out, *io);
                }
                ResourceErrorV1::ResourceKindMismatch { expected, actual } => {
                    put_u8(out, 4);
                    put_resource_kind(out, *expected);
                    put_resource_kind(out, *actual);
                }
                ResourceErrorV1::BufferLimit => put_u8(out, 5),
                ResourceErrorV1::InvalidOffset => put_u8(out, 6),
                ResourceErrorV1::InvalidBounds => put_u8(out, 7),
                ResourceErrorV1::NoProgress => put_u8(out, 8),
            }
        }
    }
    Ok(())
}

pub fn encode_linked_effect_trace(
    trace: &LinkedEffectTrace,
) -> Result<Vec<u8>, EffectTraceWireError> {
    if trace.terminal_exit
        != crate::terminal_exit_class(trace.terminal_value, trace.terminal_error.as_ref())
    {
        return Err(EffectTraceWireError);
    }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    put_u64(&mut out, trace.plan_hash);
    out.extend_from_slice(&trace.target_abi_hash);
    out.extend_from_slice(&trace.host_effect_abi_hash);
    put_i64(&mut out, trace.terminal_value);
    match &trace.terminal_error {
        None => put_u8(&mut out, 0),
        Some(crate::TerminalErrorV1::RootExecutionDenied) => put_u8(&mut out, 1),
        Some(crate::TerminalErrorV1::HomeRootResolutionFailed(failure)) => {
            put_u8(&mut out, 2);
            match failure {
                crate::HomeRootResolutionFailureV1::NoAccountRecord => put_u8(&mut out, 0),
                crate::HomeRootResolutionFailureV1::AccountRecordTooLarge => put_u8(&mut out, 1),
                crate::HomeRootResolutionFailureV1::AccountLookup(error) => {
                    put_u8(&mut out, 2);
                    put_io_error(&mut out, *error);
                }
                crate::HomeRootResolutionFailureV1::InvalidAccountRecord => put_u8(&mut out, 3),
                crate::HomeRootResolutionFailureV1::RootOpen(error) => {
                    put_u8(&mut out, 4);
                    put_io_error(&mut out, *error);
                }
                crate::HomeRootResolutionFailureV1::ScopeEscape => put_u8(&mut out, 5),
                crate::HomeRootResolutionFailureV1::SymlinkDenied => put_u8(&mut out, 6),
            }
        }
        Some(_) => return Err(EffectTraceWireError),
    }
    put_terminal_exit(&mut out, trace.terminal_exit);
    put_u64(
        &mut out,
        u64::try_from(trace.effect_trace.len()).map_err(|_| EffectTraceWireError)?,
    );
    for event in &trace.effect_trace {
        put_u64(&mut out, event.sequence);
        put_u16(&mut out, event.operation as u16);
        match &event.capability {
            None => put_u8(&mut out, 0),
            Some(identity) => {
                put_u8(&mut out, 1);
                put_string(&mut out, &identity.0)?;
            }
        }
        put_u64(
            &mut out,
            u64::try_from(event.resource_bindings.len()).map_err(|_| EffectTraceWireError)?,
        );
        for (role, identity) in &event.resource_bindings {
            put_role(&mut out, *role);
            put_u64(&mut out, identity.0);
        }
        put_request(&mut out, &event.request)?;
        match &event.outcome {
            CanonicalOutcomeV1::Success(reply) => {
                put_u8(&mut out, 0);
                put_reply(&mut out, reply)?;
            }
            CanonicalOutcomeV1::Error(error) => {
                put_u8(&mut out, 1);
                put_error(&mut out, error)?;
            }
        }
    }
    Ok(out)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Cursor<'a> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], EffectTraceWireError> {
        let end = self.position.checked_add(len).ok_or(EffectTraceWireError)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(EffectTraceWireError)?;
        self.position = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, EffectTraceWireError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, EffectTraceWireError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, EffectTraceWireError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i64(&mut self) -> Result<i64, EffectTraceWireError> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, EffectTraceWireError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn bytes(&mut self) -> Result<Vec<u8>, EffectTraceWireError> {
        let len = usize::try_from(self.u64()?).map_err(|_| EffectTraceWireError)?;
        Ok(self.take(len)?.to_vec())
    }
    fn string(&mut self) -> Result<String, EffectTraceWireError> {
        String::from_utf8(self.bytes()?).map_err(|_| EffectTraceWireError)
    }
}

fn get_stream(cursor: &mut Cursor<'_>) -> Result<ConsoleStreamV1, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(ConsoleStreamV1::Stdin),
        1 => Ok(ConsoleStreamV1::Stdout),
        2 => Ok(ConsoleStreamV1::Stderr),
        _ => Err(EffectTraceWireError),
    }
}
fn get_role(cursor: &mut Cursor<'_>) -> Result<ResourceBindingRole, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(ResourceBindingRole::File),
        1 => Ok(ResourceBindingRole::Buffer),
        2 => Ok(ResourceBindingRole::Target),
        _ => Err(EffectTraceWireError),
    }
}
fn get_terminal_exit(cursor: &mut Cursor<'_>) -> Result<TerminalExitClass, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(TerminalExitClass::NormalReturn),
        1 => Ok(TerminalExitClass::ReturnedError),
        2 => Ok(TerminalExitClass::ControlledTrap),
        _ => Err(EffectTraceWireError),
    }
}
fn get_bool(cursor: &mut Cursor<'_>) -> Result<bool, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(EffectTraceWireError),
    }
}
fn get_node_kind(cursor: &mut Cursor<'_>) -> Result<FsNodeKindV1, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(FsNodeKindV1::File),
        1 => Ok(FsNodeKindV1::Directory),
        2 => Ok(FsNodeKindV1::Symlink),
        3 => Ok(FsNodeKindV1::Other),
        _ => Err(EffectTraceWireError),
    }
}
fn get_request(cursor: &mut Cursor<'_>) -> Result<CanonicalRequestV1, EffectTraceWireError> {
    Ok(match cursor.u8()? {
        0 => CanonicalRequestV1::ConsoleRead {
            stream: get_stream(cursor)?,
            limit: cursor.u64()?,
        },
        1 => CanonicalRequestV1::ConsoleWrite {
            stream: get_stream(cursor)?,
            bytes: cursor.bytes()?,
        },
        2 => CanonicalRequestV1::ConsoleFlush {
            stream: get_stream(cursor)?,
        },
        3 => CanonicalRequestV1::ConsoleIsTerminal {
            stream: get_stream(cursor)?,
        },
        4 => CanonicalRequestV1::ClockWallNow,
        5 => CanonicalRequestV1::FsReadFile {
            path: cursor.bytes()?,
        },
        6 => {
            let path = cursor.bytes()?;
            let create_policy = match cursor.u8()? {
                0 => CreatePolicyV1::CreateNew,
                1 => CreatePolicyV1::CreateOrTruncate,
                2 => CreatePolicyV1::CreateOrKeep,
                _ => return Err(EffectTraceWireError),
            };
            CanonicalRequestV1::FsWriteFile {
                path,
                create_policy,
                bytes: cursor.bytes()?,
            }
        }
        7 => CanonicalRequestV1::FsAppendFile {
            path: cursor.bytes()?,
            bytes: cursor.bytes()?,
        },
        8 => CanonicalRequestV1::FsMetadata {
            path: cursor.bytes()?,
        },
        9 => CanonicalRequestV1::FsReadDirectory {
            path: cursor.bytes()?,
        },
        10 => CanonicalRequestV1::FsCreateDirectory {
            recursive: get_bool(cursor)?,
            path: cursor.bytes()?,
        },
        11 => CanonicalRequestV1::FsRemoveFile {
            path: cursor.bytes()?,
        },
        12 => CanonicalRequestV1::FsRemoveDirectory {
            recursive: get_bool(cursor)?,
            path: cursor.bytes()?,
        },
        13 => CanonicalRequestV1::FsRename {
            source: cursor.bytes()?,
            destination: cursor.bytes()?,
        },
        14 => CanonicalRequestV1::FsChangeMode {
            path: cursor.bytes()?,
            mode: cursor.u16()?,
        },
        15 => CanonicalRequestV1::FsOpen {
            path: cursor.bytes()?,
            mode: match cursor.u8()? {
                0 => FsOpenModeV1::Read,
                1 => FsOpenModeV1::Metadata,
                2 => FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateNew),
                3 => FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrTruncate),
                4 => FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrKeep),
                _ => return Err(EffectTraceWireError),
            },
        },
        16 => CanonicalRequestV1::FsHandleMetadata,
        17 => CanonicalRequestV1::ResourceRelease,
        18 => CanonicalRequestV1::FsReadAt {
            file_offset: cursor.u64()?,
            buffer_start: cursor.u64()?,
            length: cursor.u64()?,
        },
        19 => CanonicalRequestV1::FsWriteAt {
            file_offset: cursor.u64()?,
            buffer_start: cursor.u64()?,
            length: cursor.u64()?,
        },
        20 => CanonicalRequestV1::BufferAllocate {
            capacity: cursor.u64()?,
        },
        21 => CanonicalRequestV1::BufferFreeze {
            start: cursor.u64()?,
            length: cursor.u64()?,
        },
        _ => return Err(EffectTraceWireError),
    })
}

/// Fail-closed correlation the reply decoder cannot perform on its own: it
/// sees only a cursor, so `TransferCountV1::new` can prove `0 < count <=
/// effective_request` but nothing bounds `effective_request` against the
/// caller's own raw request length. A crafted trace with a valid count/
/// effective pair but an inflated `effective_request` would otherwise
/// decode. Reject mismatched request/reply shapes outright rather than
/// inferring a budget from a request the reply doesn't correspond to.
fn validate_transfer_request_bound(
    request: &CanonicalRequestV1,
    outcome: &CanonicalOutcomeV1,
) -> Result<(), EffectTraceWireError> {
    match outcome.transfer_request_bound() {
        None => Ok(()),
        Some(crate::TransferRequestBoundV1::ReadAt(transferred)) => match request {
            CanonicalRequestV1::FsReadAt { length, .. }
                if transferred.effective_request() <= *length =>
            {
                Ok(())
            }
            _ => Err(EffectTraceWireError),
        },
        Some(crate::TransferRequestBoundV1::WriteAt(transferred)) => match request {
            CanonicalRequestV1::FsWriteAt { length, .. }
                if transferred.effective_request() <= *length =>
            {
                Ok(())
            }
            _ => Err(EffectTraceWireError),
        },
    }
}

fn get_reply(cursor: &mut Cursor<'_>) -> Result<CanonicalReplyV1, EffectTraceWireError> {
    Ok(match cursor.u8()? {
        0 => CanonicalReplyV1::Unit,
        1 => CanonicalReplyV1::Bool(get_bool(cursor)?),
        2 => CanonicalReplyV1::Bytes(cursor.bytes()?),
        3 => CanonicalReplyV1::ReadChunk(cursor.bytes()?),
        4 => CanonicalReplyV1::ReadEof,
        5 => CanonicalReplyV1::Instant(cursor.bytes()?),
        6 => CanonicalReplyV1::FileMetadata(FileMetadataV1 {
            size: cursor.u64()?,
            kind: get_node_kind(cursor)?,
        }),
        7 => {
            let count = usize::try_from(cursor.u64()?).map_err(|_| EffectTraceWireError)?;
            if count > cursor.remaining() / 9 {
                return Err(EffectTraceWireError);
            }
            let mut entries = Vec::with_capacity(count);
            for _ in 0..count {
                entries.push(DirEntryV1 {
                    name: cursor.bytes()?,
                    kind: get_node_kind(cursor)?,
                });
            }
            CanonicalReplyV1::DirectoryEntries(entries)
        }
        8 => CanonicalReplyV1::ResourceAcquired {
            schema_version: cursor.u16()?,
            resource_kind: get_resource_kind(cursor)?,
            identity: ResourceTraceIdentityV1(cursor.u64()?),
        },
        9 => CanonicalReplyV1::ResourceSettlement(get_settlement(cursor)?),
        10 => {
            let progress = match cursor.u8()? {
                0 => {
                    let start = cursor.u64()?;
                    let length = cursor.u64()?;
                    let transferred = cursor.u64()?;
                    let effective_request = cursor.u64()?;
                    if length != transferred {
                        return Err(EffectTraceWireError);
                    }
                    let transferred = crate::TransferCountV1::new(transferred, effective_request)
                        .ok_or(EffectTraceWireError)?;
                    crate::ReadProgressV1::ReadSome {
                        span: crate::BufferSpanV1 { start, length },
                        transferred,
                    }
                }
                1 => crate::ReadProgressV1::ReadEof,
                _ => return Err(EffectTraceWireError),
            };
            CanonicalReplyV1::ReadProgress(progress)
        }
        11 => {
            let count = cursor.u64()?;
            let effective_request = cursor.u64()?;
            let transferred = crate::TransferCountV1::new(count, effective_request)
                .ok_or(EffectTraceWireError)?;
            CanonicalReplyV1::WriteProgress(crate::WriteProgressV1::Wrote(transferred))
        }
        _ => return Err(EffectTraceWireError),
    })
}

fn get_resource_kind(cursor: &mut Cursor<'_>) -> Result<ResourceKindV1, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(ResourceKindV1::FsHandle),
        1 => Ok(ResourceKindV1::Buffer),
        _ => Err(EffectTraceWireError),
    }
}

fn get_settlement(
    cursor: &mut Cursor<'_>,
) -> Result<ResourceSettlementObservationV1, EffectTraceWireError> {
    let schema_version = cursor.u16()?;
    let resource_kind = get_resource_kind(cursor)?;
    let identity = ResourceTraceIdentityV1(cursor.u64()?);
    let outcome = match cursor.u8()? {
        0 => ResourceSettlementOutcomeV1::Released,
        1 => ResourceSettlementOutcomeV1::ReleaseFailed(get_io_error(cursor)?),
        _ => return Err(EffectTraceWireError),
    };
    Ok(ResourceSettlementObservationV1 {
        schema_version,
        resource_kind,
        identity,
        outcome,
    })
}
fn get_io_error(cursor: &mut Cursor<'_>) -> Result<IoErrorIdentityV1, EffectTraceWireError> {
    Ok(match cursor.u8()? {
        0 => IoErrorIdentityV1::NotFound,
        1 => IoErrorIdentityV1::PermissionDenied,
        2 => IoErrorIdentityV1::BrokenPipe,
        3 => IoErrorIdentityV1::Interrupted,
        4 => IoErrorIdentityV1::AlreadyExists,
        5 => IoErrorIdentityV1::InvalidInput,
        6 => IoErrorIdentityV1::IsDirectory,
        7 => IoErrorIdentityV1::NotDirectory,
        8 => IoErrorIdentityV1::NotEmpty,
        9 => IoErrorIdentityV1::Unsupported,
        10 => IoErrorIdentityV1::Other(cursor.i32()?),
        _ => return Err(EffectTraceWireError),
    })
}
fn get_fs_operation(
    cursor: &mut Cursor<'_>,
) -> Result<FsCapabilityOperationV1, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(FsCapabilityOperationV1::Read),
        1 => Ok(FsCapabilityOperationV1::Write),
        2 => Ok(FsCapabilityOperationV1::Append),
        3 => Ok(FsCapabilityOperationV1::Metadata),
        4 => Ok(FsCapabilityOperationV1::Enumerate),
        5 => Ok(FsCapabilityOperationV1::CreateDirectory),
        6 => Ok(FsCapabilityOperationV1::RemoveFile),
        7 => Ok(FsCapabilityOperationV1::RemoveDirectory),
        8 => Ok(FsCapabilityOperationV1::RenameSource),
        9 => Ok(FsCapabilityOperationV1::RenameDestination),
        10 => Ok(FsCapabilityOperationV1::ChangeMode),
        _ => Err(EffectTraceWireError),
    }
}
fn get_denial(cursor: &mut Cursor<'_>) -> Result<CapabilityDeniedV1, EffectTraceWireError> {
    Ok(match cursor.u8()? {
        0 => CapabilityDeniedV1::RightNotHeld {
            operation: get_fs_operation(cursor)?,
            held_rights: cursor.u8()?,
        },
        1 => CapabilityDeniedV1::AuthorityInsufficient,
        2 => CapabilityDeniedV1::ScopeEscape,
        3 => CapabilityDeniedV1::SymlinkDenied,
        4 => CapabilityDeniedV1::MalformedCapability,
        _ => return Err(EffectTraceWireError),
    })
}
fn get_cause(cursor: &mut Cursor<'_>) -> Result<FileErrorCauseV1, EffectTraceWireError> {
    match cursor.u8()? {
        0 => Ok(FileErrorCauseV1::Io(get_io_error(cursor)?)),
        1 => Ok(FileErrorCauseV1::Capability(get_denial(cursor)?)),
        _ => Err(EffectTraceWireError),
    }
}
fn get_error(cursor: &mut Cursor<'_>) -> Result<SemanticErrorV1, EffectTraceWireError> {
    Ok(match cursor.u8()? {
        0 => SemanticErrorV1::Io(get_io_error(cursor)?),
        1 => SemanticErrorV1::File(FileErrorIdentityV1 {
            operation: HostOpV1::try_from(cursor.u16()?).map_err(|_| EffectTraceWireError)?,
            relative_path: cursor.bytes()?,
            cause: get_cause(cursor)?,
        }),
        2 => SemanticErrorV1::Capability(get_denial(cursor)?),
        3 => SemanticErrorV1::Resource(match cursor.u8()? {
            0 => ResourceErrorV1::Closed,
            1 => ResourceErrorV1::MalformedResource,
            2 => ResourceErrorV1::RightNotHeld {
                required: cursor.u8()?,
                held: cursor.u8()?,
            },
            3 => ResourceErrorV1::ReleaseFailed {
                schema_version: cursor.u16()?,
                resource_kind: get_resource_kind(cursor)?,
                identity: ResourceTraceIdentityV1(cursor.u64()?),
                io: get_io_error(cursor)?,
            },
            4 => ResourceErrorV1::ResourceKindMismatch {
                expected: get_resource_kind(cursor)?,
                actual: get_resource_kind(cursor)?,
            },
            5 => ResourceErrorV1::BufferLimit,
            6 => ResourceErrorV1::InvalidOffset,
            7 => ResourceErrorV1::InvalidBounds,
            8 => ResourceErrorV1::NoProgress,
            _ => return Err(EffectTraceWireError),
        }),
        _ => return Err(EffectTraceWireError),
    })
}

pub fn decode_linked_effect_trace(bytes: &[u8]) -> Result<LinkedEffectTrace, EffectTraceWireError> {
    let mut cursor = Cursor { bytes, position: 0 };
    if cursor.take(MAGIC.len())? != MAGIC {
        return Err(EffectTraceWireError);
    }
    let plan_hash = cursor.u64()?;
    let target_abi_hash = cursor.take(32)?.try_into().unwrap();
    let host_effect_abi_hash = cursor.take(32)?.try_into().unwrap();
    let terminal_value = cursor.i64()?;
    let terminal_error = match cursor.u8()? {
        0 => None,
        1 => Some(crate::TerminalErrorV1::RootExecutionDenied),
        2 => Some(crate::TerminalErrorV1::HomeRootResolutionFailed(
            match cursor.u8()? {
                0 => crate::HomeRootResolutionFailureV1::NoAccountRecord,
                1 => crate::HomeRootResolutionFailureV1::AccountRecordTooLarge,
                2 => crate::HomeRootResolutionFailureV1::AccountLookup(get_io_error(&mut cursor)?),
                3 => crate::HomeRootResolutionFailureV1::InvalidAccountRecord,
                4 => crate::HomeRootResolutionFailureV1::RootOpen(get_io_error(&mut cursor)?),
                5 => crate::HomeRootResolutionFailureV1::ScopeEscape,
                6 => crate::HomeRootResolutionFailureV1::SymlinkDenied,
                _ => return Err(EffectTraceWireError),
            },
        )),
        _ => return Err(EffectTraceWireError),
    };
    let terminal_exit = get_terminal_exit(&mut cursor)?;
    let count = usize::try_from(cursor.u64()?).map_err(|_| EffectTraceWireError)?;
    // Minimum event: sequence + operation + capability tag + binding count +
    // request tag + outcome tag + reply/error tag.
    if count > cursor.remaining() / 22 {
        return Err(EffectTraceWireError);
    }
    let mut effect_trace = Vec::with_capacity(count);
    for _ in 0..count {
        let sequence = cursor.u64()?;
        let operation = HostOpV1::try_from(cursor.u16()?).map_err(|_| EffectTraceWireError)?;
        let capability = match cursor.u8()? {
            0 => None,
            1 => Some(CapabilityTraceIdentity(cursor.string()?)),
            _ => return Err(EffectTraceWireError),
        };
        let binding_count = usize::try_from(cursor.u64()?).map_err(|_| EffectTraceWireError)?;
        if binding_count > cursor.remaining() / 9 {
            return Err(EffectTraceWireError);
        }
        let mut resource_bindings = Vec::with_capacity(binding_count);
        for _ in 0..binding_count {
            resource_bindings.push((
                get_role(&mut cursor)?,
                ResourceTraceIdentityV1(cursor.u64()?),
            ));
        }
        let request = get_request(&mut cursor)?;
        let outcome = match cursor.u8()? {
            0 => CanonicalOutcomeV1::Success(get_reply(&mut cursor)?),
            1 => CanonicalOutcomeV1::Error(get_error(&mut cursor)?),
            _ => return Err(EffectTraceWireError),
        };
        validate_transfer_request_bound(&request, &outcome)?;
        effect_trace.push(EffectEvent {
            sequence,
            operation,
            capability,
            resource_bindings,
            request,
            outcome,
        });
    }
    if cursor.position != bytes.len()
        || terminal_exit != crate::terminal_exit_class(terminal_value, terminal_error.as_ref())
    {
        return Err(EffectTraceWireError);
    }
    Ok(LinkedEffectTrace {
        plan_hash,
        target_abi_hash,
        host_effect_abi_hash,
        terminal_value,
        terminal_error,
        effect_trace,
        terminal_exit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn representative_trace() -> LinkedEffectTrace {
        LinkedEffectTrace {
            plan_hash: 7,
            target_abi_hash: [3; 32],
            host_effect_abi_hash: [5; 32],
            terminal_value: 61,
            terminal_error: None,
            terminal_exit: TerminalExitClass::ReturnedError,
            effect_trace: vec![
                EffectEvent {
                    sequence: 0,
                    operation: HostOpV1::ConsoleWrite,
                    capability: None,
                    resource_bindings: Vec::new(),
                    request: CanonicalRequestV1::ConsoleWrite {
                        stream: ConsoleStreamV1::Stdout,
                        bytes: vec![0, 0xff],
                    },
                    outcome: CanonicalOutcomeV1::Error(SemanticErrorV1::Io(
                        IoErrorIdentityV1::BrokenPipe,
                    )),
                },
                EffectEvent {
                    sequence: 1,
                    operation: HostOpV1::FsReadFile,
                    capability: Some(CapabilityTraceIdentity("declared:FS".into())),
                    resource_bindings: Vec::new(),
                    request: CanonicalRequestV1::FsReadFile {
                        path: vec![b'f', 0xff],
                    },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(vec![1, 2])),
                },
                EffectEvent {
                    sequence: 2,
                    operation: HostOpV1::FsChangeMode,
                    capability: Some(CapabilityTraceIdentity("declared:FS".into())),
                    resource_bindings: Vec::new(),
                    request: CanonicalRequestV1::FsChangeMode {
                        path: vec![b'm', 0xfe],
                        mode: 0o640,
                    },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::Unit),
                },
                EffectEvent {
                    sequence: 3,
                    operation: HostOpV1::FsOpen,
                    capability: Some(CapabilityTraceIdentity("declared:FS".into())),
                    resource_bindings: vec![(
                        ResourceBindingRole::Target,
                        ResourceTraceIdentityV1(1),
                    )],
                    request: CanonicalRequestV1::FsOpen {
                        path: vec![b'r', 0xff],
                        mode: FsOpenModeV1::Metadata,
                    },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::ResourceAcquired {
                        schema_version: 1,
                        resource_kind: ResourceKindV1::FsHandle,
                        identity: ResourceTraceIdentityV1(1),
                    }),
                },
                EffectEvent {
                    sequence: 4,
                    operation: HostOpV1::ResourceRelease,
                    capability: None,
                    resource_bindings: vec![(
                        ResourceBindingRole::Target,
                        ResourceTraceIdentityV1(1),
                    )],
                    request: CanonicalRequestV1::ResourceRelease,
                    outcome: CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(
                        ResourceErrorV1::ReleaseFailed {
                            schema_version: 1,
                            resource_kind: ResourceKindV1::FsHandle,
                            identity: ResourceTraceIdentityV1(1),
                            io: IoErrorIdentityV1::Other(5),
                        },
                    )),
                },
                EffectEvent {
                    sequence: 5,
                    operation: HostOpV1::BufferAllocate,
                    capability: None,
                    resource_bindings: vec![(
                        ResourceBindingRole::Target,
                        ResourceTraceIdentityV1(2),
                    )],
                    request: CanonicalRequestV1::BufferAllocate { capacity: 8 },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::ResourceAcquired {
                        schema_version: 1,
                        resource_kind: ResourceKindV1::Buffer,
                        identity: ResourceTraceIdentityV1(2),
                    }),
                },
                EffectEvent {
                    sequence: 6,
                    operation: HostOpV1::FsReadAt,
                    capability: None,
                    resource_bindings: vec![
                        (ResourceBindingRole::File, ResourceTraceIdentityV1(1)),
                        (ResourceBindingRole::Buffer, ResourceTraceIdentityV1(2)),
                    ],
                    request: CanonicalRequestV1::FsReadAt {
                        file_offset: 9,
                        buffer_start: 1,
                        length: 4,
                    },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::ReadProgress(
                        crate::ReadProgressV1::ReadSome {
                            span: crate::BufferSpanV1 {
                                start: 1,
                                length: 2,
                            },
                            transferred: crate::TransferCountV1::new(2, 3)
                                .expect("2 <= effective 3"),
                        },
                    )),
                },
                EffectEvent {
                    sequence: 7,
                    operation: HostOpV1::FsWriteAt,
                    capability: None,
                    resource_bindings: Vec::new(),
                    request: CanonicalRequestV1::FsWriteAt {
                        file_offset: u64::MAX,
                        buffer_start: 0,
                        length: 1,
                    },
                    outcome: CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(
                        ResourceErrorV1::ResourceKindMismatch {
                            expected: ResourceKindV1::FsHandle,
                            actual: ResourceKindV1::Buffer,
                        },
                    )),
                },
                EffectEvent {
                    sequence: 8,
                    operation: HostOpV1::BufferFreeze,
                    capability: None,
                    resource_bindings: vec![(
                        ResourceBindingRole::Target,
                        ResourceTraceIdentityV1(2),
                    )],
                    request: CanonicalRequestV1::BufferFreeze {
                        start: 1,
                        length: 2,
                    },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(vec![0xff, 0])),
                },
            ],
        }
    }

    #[test]
    fn linked_trace_codec_roundtrips_identity_bytes_and_outcomes() {
        let expected = representative_trace();
        let encoded = encode_linked_effect_trace(&expected).unwrap();
        assert_eq!(decode_linked_effect_trace(&encoded), Ok(expected));
    }

    #[test]
    fn home_failure_wire_distinguishes_lookup_from_root_open_and_keeps_identity() {
        let mut lookup = representative_trace();
        lookup.terminal_error = Some(crate::TerminalErrorV1::HomeRootResolutionFailed(
            crate::HomeRootResolutionFailureV1::AccountLookup(IoErrorIdentityV1::Other(5)),
        ));
        lookup.terminal_exit = TerminalExitClass::ControlledTrap;
        let mut root_open = representative_trace();
        root_open.terminal_error = Some(crate::TerminalErrorV1::HomeRootResolutionFailed(
            crate::HomeRootResolutionFailureV1::RootOpen(IoErrorIdentityV1::Other(5)),
        ));
        root_open.terminal_exit = TerminalExitClass::ControlledTrap;

        let lookup_wire = encode_linked_effect_trace(&lookup).unwrap();
        let root_open_wire = encode_linked_effect_trace(&root_open).unwrap();
        assert_ne!(lookup_wire, root_open_wire);
        assert_eq!(decode_linked_effect_trace(&lookup_wire), Ok(lookup));
        assert_eq!(decode_linked_effect_trace(&root_open_wire), Ok(root_open));
    }

    #[test]
    fn linked_trace_decoder_rejects_magic_truncation_trailing_and_count_drift() {
        let encoded = encode_linked_effect_trace(&representative_trace()).unwrap();

        let mut wrong_magic = encoded.clone();
        wrong_magic[0] ^= 1;
        assert_eq!(
            decode_linked_effect_trace(&wrong_magic),
            Err(EffectTraceWireError)
        );
        assert_eq!(
            decode_linked_effect_trace(&encoded[..encoded.len() - 1]),
            Err(EffectTraceWireError)
        );
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert_eq!(
            decode_linked_effect_trace(&trailing),
            Err(EffectTraceWireError)
        );
        let mut impossible_count = encoded;
        impossible_count[88..96].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            decode_linked_effect_trace(&impossible_count),
            Err(EffectTraceWireError)
        );
    }

    #[test]
    fn sole_codec_rejects_terminal_role_length_and_class_drift() {
        let trace = LinkedEffectTrace {
            plan_hash: 7,
            target_abi_hash: [3; 32],
            host_effect_abi_hash: [5; 32],
            terminal_value: 0,
            terminal_error: None,
            effect_trace: vec![EffectEvent {
                sequence: 0,
                operation: HostOpV1::ResourceRelease,
                capability: None,
                resource_bindings: vec![(ResourceBindingRole::Target, ResourceTraceIdentityV1(19))],
                request: CanonicalRequestV1::ResourceRelease,
                outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::Unit),
            }],
            terminal_exit: TerminalExitClass::NormalReturn,
        };
        let encoded = encode_linked_effect_trace(&trace).unwrap();

        let mut wrong_terminal_tag = encoded.clone();
        wrong_terminal_tag[89] = 3;
        assert_eq!(
            decode_linked_effect_trace(&wrong_terminal_tag),
            Err(EffectTraceWireError)
        );

        let mut wrong_role = encoded.clone();
        wrong_role[117] = 3;
        assert_eq!(
            decode_linked_effect_trace(&wrong_role),
            Err(EffectTraceWireError)
        );

        let mut impossible_bindings = encoded.clone();
        impossible_bindings[109..117].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            decode_linked_effect_trace(&impossible_bindings),
            Err(EffectTraceWireError)
        );

        let mut mismatched_class = trace;
        mismatched_class.terminal_exit = TerminalExitClass::ReturnedError;
        assert_eq!(
            encode_linked_effect_trace(&mismatched_class),
            Err(EffectTraceWireError)
        );
    }

    fn single_read_some_trace(request_length: u64) -> LinkedEffectTrace {
        LinkedEffectTrace {
            plan_hash: 7,
            target_abi_hash: [3; 32],
            host_effect_abi_hash: [5; 32],
            terminal_value: 0,
            terminal_error: None,
            effect_trace: vec![EffectEvent {
                sequence: 0,
                operation: HostOpV1::FsReadAt,
                capability: None,
                resource_bindings: Vec::new(),
                request: CanonicalRequestV1::FsReadAt {
                    file_offset: 0,
                    buffer_start: 0,
                    length: request_length,
                },
                outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::ReadProgress(
                    crate::ReadProgressV1::ReadSome {
                        span: crate::BufferSpanV1 {
                            start: 0,
                            length: 4,
                        },
                        transferred: crate::TransferCountV1::new(4, 4).expect("4 <= effective 4"),
                    },
                )),
            }],
            terminal_exit: TerminalExitClass::NormalReturn,
        }
    }

    /// The gap the Architect found in `dec_3fz801fangyp`'s first review: the
    /// reply decoder alone can only prove `0 < count <= effective_request`
    /// (`TransferCountV1::new`'s own invariant, exercised by the first two
    /// mutations below); it cannot bound `effective_request` against the
    /// caller's own raw request length without `validate_transfer_request_
    /// bound` correlating the decoded request. `effective_request` is the
    /// trailing 8 bytes of this single-event trace's `ReadSome` payload
    /// (the last thing `put_reply` writes), so mutating the tail exercises
    /// exactly that field.
    #[test]
    fn wire_decoder_rejects_read_some_effective_request_zero_below_count_and_above_raw() {
        let encoded = encode_linked_effect_trace(&single_read_some_trace(8)).unwrap();
        assert!(
            decode_linked_effect_trace(&encoded).is_ok(),
            "baseline must be valid"
        );
        let tail = encoded.len() - 8;

        let mut effective_zero = encoded.clone();
        effective_zero[tail..].copy_from_slice(&0u64.to_le_bytes());
        assert_eq!(
            decode_linked_effect_trace(&effective_zero),
            Err(EffectTraceWireError),
            "effective_request == 0"
        );

        let mut effective_below_count = encoded.clone();
        effective_below_count[tail..].copy_from_slice(&3u64.to_le_bytes());
        assert_eq!(
            decode_linked_effect_trace(&effective_below_count),
            Err(EffectTraceWireError),
            "effective_request(3) < count(4)"
        );

        let mut effective_above_raw = encoded;
        effective_above_raw[tail..].copy_from_slice(&9u64.to_le_bytes());
        assert_eq!(
            decode_linked_effect_trace(&effective_above_raw),
            Err(EffectTraceWireError),
            "effective_request(9) > raw request.length(8) — the fold-in fix"
        );
    }

    /// Boundary constraint 4 (`dec_1m6xdwjp2ttyn`): applies to both
    /// `ReadSome` and `Wrote`. Confirms the `effective_request > raw`
    /// correlation fix is symmetric, not `ReadSome`-only.
    #[test]
    fn wire_decoder_rejects_wrote_effective_request_above_raw_request() {
        let trace = LinkedEffectTrace {
            plan_hash: 7,
            target_abi_hash: [3; 32],
            host_effect_abi_hash: [5; 32],
            terminal_value: 0,
            terminal_error: None,
            effect_trace: vec![EffectEvent {
                sequence: 0,
                operation: HostOpV1::FsWriteAt,
                capability: None,
                resource_bindings: Vec::new(),
                request: CanonicalRequestV1::FsWriteAt {
                    file_offset: 0,
                    buffer_start: 0,
                    length: 8,
                },
                outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::WriteProgress(
                    crate::WriteProgressV1::Wrote(
                        crate::TransferCountV1::new(4, 4).expect("4 <= effective 4"),
                    ),
                )),
            }],
            terminal_exit: TerminalExitClass::NormalReturn,
        };
        let encoded = encode_linked_effect_trace(&trace).unwrap();
        assert!(
            decode_linked_effect_trace(&encoded).is_ok(),
            "baseline must be valid"
        );

        let mut effective_above_raw = encoded;
        let tail = effective_above_raw.len() - 8;
        effective_above_raw[tail..].copy_from_slice(&9u64.to_le_bytes());
        assert_eq!(
            decode_linked_effect_trace(&effective_above_raw),
            Err(EffectTraceWireError),
            "effective_request(9) > raw request.length(8)"
        );
    }
}
