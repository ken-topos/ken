//! Total private wire codec for one completed linked-artifact effect trace.

use crate::{
    CanonicalOutcomeV1, CanonicalReplyV1, CanonicalRequestV1, CapabilityDeniedV1,
    CapabilityTraceIdentity, ConsoleStreamV1, CreatePolicyV1, DirEntryV1, EffectEventV1,
    FileErrorCauseV1, FileErrorIdentityV1, FileMetadataV1, FsCapabilityOperationV1, FsNodeKindV1,
    FsOpenModeV1, HostOpV1, IoErrorIdentityV1, ResourceErrorV1, ResourceKindV1,
    ResourceSettlementObservationV1, ResourceSettlementOutcomeV1, ResourceTraceIdentityV1,
    SemanticErrorV1,
};

const MAGIC: &[u8; 8] = b"KETRACE2";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedEffectTraceV1 {
    pub plan_hash: u64,
    pub target_abi_hash: [u8; 32],
    pub host_effect_abi_hash: [u8; 32],
    pub terminal_value: i64,
    pub terminal_error: Option<crate::TerminalErrorV1>,
    pub effect_trace: Vec<EffectEventV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectTraceWireErrorV1;

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

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), EffectTraceWireErrorV1> {
    put_u64(
        out,
        u64::try_from(bytes.len()).map_err(|_| EffectTraceWireErrorV1)?,
    );
    out.extend_from_slice(bytes);
    Ok(())
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), EffectTraceWireErrorV1> {
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
) -> Result<(), EffectTraceWireErrorV1> {
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
                },
            );
        }
        CanonicalRequestV1::FsHandleMetadata => put_u8(out, 16),
        CanonicalRequestV1::ResourceRelease => put_u8(out, 17),
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

fn put_reply(out: &mut Vec<u8>, reply: &CanonicalReplyV1) -> Result<(), EffectTraceWireErrorV1> {
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
                u64::try_from(entries.len()).map_err(|_| EffectTraceWireErrorV1)?,
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
    }
    Ok(())
}

fn put_resource_kind(out: &mut Vec<u8>, kind: ResourceKindV1) {
    match kind {
        ResourceKindV1::FsHandle => put_u8(out, 0),
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

fn put_error(out: &mut Vec<u8>, error: &SemanticErrorV1) -> Result<(), EffectTraceWireErrorV1> {
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
            }
        }
    }
    Ok(())
}

pub fn encode_linked_effect_trace_v1(
    trace: &LinkedEffectTraceV1,
) -> Result<Vec<u8>, EffectTraceWireErrorV1> {
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
        Some(_) => return Err(EffectTraceWireErrorV1),
    }
    put_u64(
        &mut out,
        u64::try_from(trace.effect_trace.len()).map_err(|_| EffectTraceWireErrorV1)?,
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
        match event.resource {
            None => put_u8(&mut out, 0),
            Some(identity) => {
                put_u8(&mut out, 1);
                put_u64(&mut out, identity.0);
            }
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

    fn take(&mut self, len: usize) -> Result<&'a [u8], EffectTraceWireErrorV1> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(EffectTraceWireErrorV1)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(EffectTraceWireErrorV1)?;
        self.position = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, EffectTraceWireErrorV1> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, EffectTraceWireErrorV1> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, EffectTraceWireErrorV1> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i64(&mut self) -> Result<i64, EffectTraceWireErrorV1> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, EffectTraceWireErrorV1> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn bytes(&mut self) -> Result<Vec<u8>, EffectTraceWireErrorV1> {
        let len = usize::try_from(self.u64()?).map_err(|_| EffectTraceWireErrorV1)?;
        Ok(self.take(len)?.to_vec())
    }
    fn string(&mut self) -> Result<String, EffectTraceWireErrorV1> {
        String::from_utf8(self.bytes()?).map_err(|_| EffectTraceWireErrorV1)
    }
}

fn get_stream(cursor: &mut Cursor<'_>) -> Result<ConsoleStreamV1, EffectTraceWireErrorV1> {
    match cursor.u8()? {
        0 => Ok(ConsoleStreamV1::Stdin),
        1 => Ok(ConsoleStreamV1::Stdout),
        2 => Ok(ConsoleStreamV1::Stderr),
        _ => Err(EffectTraceWireErrorV1),
    }
}
fn get_bool(cursor: &mut Cursor<'_>) -> Result<bool, EffectTraceWireErrorV1> {
    match cursor.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(EffectTraceWireErrorV1),
    }
}
fn get_node_kind(cursor: &mut Cursor<'_>) -> Result<FsNodeKindV1, EffectTraceWireErrorV1> {
    match cursor.u8()? {
        0 => Ok(FsNodeKindV1::File),
        1 => Ok(FsNodeKindV1::Directory),
        2 => Ok(FsNodeKindV1::Symlink),
        3 => Ok(FsNodeKindV1::Other),
        _ => Err(EffectTraceWireErrorV1),
    }
}
fn get_request(cursor: &mut Cursor<'_>) -> Result<CanonicalRequestV1, EffectTraceWireErrorV1> {
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
                _ => return Err(EffectTraceWireErrorV1),
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
                _ => return Err(EffectTraceWireErrorV1),
            },
        },
        16 => CanonicalRequestV1::FsHandleMetadata,
        17 => CanonicalRequestV1::ResourceRelease,
        _ => return Err(EffectTraceWireErrorV1),
    })
}

fn get_reply(cursor: &mut Cursor<'_>) -> Result<CanonicalReplyV1, EffectTraceWireErrorV1> {
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
            let count = usize::try_from(cursor.u64()?).map_err(|_| EffectTraceWireErrorV1)?;
            if count > cursor.remaining() / 9 {
                return Err(EffectTraceWireErrorV1);
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
        _ => return Err(EffectTraceWireErrorV1),
    })
}

fn get_resource_kind(cursor: &mut Cursor<'_>) -> Result<ResourceKindV1, EffectTraceWireErrorV1> {
    match cursor.u8()? {
        0 => Ok(ResourceKindV1::FsHandle),
        _ => Err(EffectTraceWireErrorV1),
    }
}

fn get_settlement(
    cursor: &mut Cursor<'_>,
) -> Result<ResourceSettlementObservationV1, EffectTraceWireErrorV1> {
    let schema_version = cursor.u16()?;
    let resource_kind = get_resource_kind(cursor)?;
    let identity = ResourceTraceIdentityV1(cursor.u64()?);
    let outcome = match cursor.u8()? {
        0 => ResourceSettlementOutcomeV1::Released,
        1 => ResourceSettlementOutcomeV1::ReleaseFailed(get_io_error(cursor)?),
        _ => return Err(EffectTraceWireErrorV1),
    };
    Ok(ResourceSettlementObservationV1 {
        schema_version,
        resource_kind,
        identity,
        outcome,
    })
}
fn get_io_error(cursor: &mut Cursor<'_>) -> Result<IoErrorIdentityV1, EffectTraceWireErrorV1> {
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
        _ => return Err(EffectTraceWireErrorV1),
    })
}
fn get_fs_operation(
    cursor: &mut Cursor<'_>,
) -> Result<FsCapabilityOperationV1, EffectTraceWireErrorV1> {
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
        _ => Err(EffectTraceWireErrorV1),
    }
}
fn get_denial(cursor: &mut Cursor<'_>) -> Result<CapabilityDeniedV1, EffectTraceWireErrorV1> {
    Ok(match cursor.u8()? {
        0 => CapabilityDeniedV1::RightNotHeld {
            operation: get_fs_operation(cursor)?,
            held_rights: cursor.u8()?,
        },
        1 => CapabilityDeniedV1::AuthorityInsufficient,
        2 => CapabilityDeniedV1::ScopeEscape,
        3 => CapabilityDeniedV1::SymlinkDenied,
        4 => CapabilityDeniedV1::MalformedCapability,
        _ => return Err(EffectTraceWireErrorV1),
    })
}
fn get_cause(cursor: &mut Cursor<'_>) -> Result<FileErrorCauseV1, EffectTraceWireErrorV1> {
    match cursor.u8()? {
        0 => Ok(FileErrorCauseV1::Io(get_io_error(cursor)?)),
        1 => Ok(FileErrorCauseV1::Capability(get_denial(cursor)?)),
        _ => Err(EffectTraceWireErrorV1),
    }
}
fn get_error(cursor: &mut Cursor<'_>) -> Result<SemanticErrorV1, EffectTraceWireErrorV1> {
    Ok(match cursor.u8()? {
        0 => SemanticErrorV1::Io(get_io_error(cursor)?),
        1 => SemanticErrorV1::File(FileErrorIdentityV1 {
            operation: HostOpV1::try_from(cursor.u16()?).map_err(|_| EffectTraceWireErrorV1)?,
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
            _ => return Err(EffectTraceWireErrorV1),
        }),
        _ => return Err(EffectTraceWireErrorV1),
    })
}

pub fn decode_linked_effect_trace_v1(
    bytes: &[u8],
) -> Result<LinkedEffectTraceV1, EffectTraceWireErrorV1> {
    let mut cursor = Cursor { bytes, position: 0 };
    if cursor.take(MAGIC.len())? != MAGIC {
        return Err(EffectTraceWireErrorV1);
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
                _ => return Err(EffectTraceWireErrorV1),
            },
        )),
        _ => return Err(EffectTraceWireErrorV1),
    };
    let count = usize::try_from(cursor.u64()?).map_err(|_| EffectTraceWireErrorV1)?;
    if count > cursor.remaining() / 13 {
        return Err(EffectTraceWireErrorV1);
    }
    let mut effect_trace = Vec::with_capacity(count);
    for _ in 0..count {
        let sequence = cursor.u64()?;
        let operation = HostOpV1::try_from(cursor.u16()?).map_err(|_| EffectTraceWireErrorV1)?;
        let capability = match cursor.u8()? {
            0 => None,
            1 => Some(CapabilityTraceIdentity(cursor.string()?)),
            _ => return Err(EffectTraceWireErrorV1),
        };
        let resource = match cursor.u8()? {
            0 => None,
            1 => Some(ResourceTraceIdentityV1(cursor.u64()?)),
            _ => return Err(EffectTraceWireErrorV1),
        };
        let request = get_request(&mut cursor)?;
        let outcome = match cursor.u8()? {
            0 => CanonicalOutcomeV1::Success(get_reply(&mut cursor)?),
            1 => CanonicalOutcomeV1::Error(get_error(&mut cursor)?),
            _ => return Err(EffectTraceWireErrorV1),
        };
        effect_trace.push(EffectEventV1 {
            sequence,
            operation,
            capability,
            resource,
            request,
            outcome,
        });
    }
    if cursor.position != bytes.len() {
        return Err(EffectTraceWireErrorV1);
    }
    Ok(LinkedEffectTraceV1 {
        plan_hash,
        target_abi_hash,
        host_effect_abi_hash,
        terminal_value,
        terminal_error,
        effect_trace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn representative_trace() -> LinkedEffectTraceV1 {
        LinkedEffectTraceV1 {
            plan_hash: 7,
            target_abi_hash: [3; 32],
            host_effect_abi_hash: [5; 32],
            terminal_value: 61,
            terminal_error: None,
            effect_trace: vec![
                EffectEventV1 {
                    sequence: 0,
                    operation: HostOpV1::ConsoleWrite,
                    capability: None,
                    resource: None,
                    request: CanonicalRequestV1::ConsoleWrite {
                        stream: ConsoleStreamV1::Stdout,
                        bytes: vec![0, 0xff],
                    },
                    outcome: CanonicalOutcomeV1::Error(SemanticErrorV1::Io(
                        IoErrorIdentityV1::BrokenPipe,
                    )),
                },
                EffectEventV1 {
                    sequence: 1,
                    operation: HostOpV1::FsReadFile,
                    capability: Some(CapabilityTraceIdentity("declared:FS".into())),
                    resource: None,
                    request: CanonicalRequestV1::FsReadFile {
                        path: vec![b'f', 0xff],
                    },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(vec![1, 2])),
                },
                EffectEventV1 {
                    sequence: 2,
                    operation: HostOpV1::FsChangeMode,
                    capability: Some(CapabilityTraceIdentity("declared:FS".into())),
                    resource: None,
                    request: CanonicalRequestV1::FsChangeMode {
                        path: vec![b'm', 0xfe],
                        mode: 0o640,
                    },
                    outcome: CanonicalOutcomeV1::Success(CanonicalReplyV1::Unit),
                },
                EffectEventV1 {
                    sequence: 3,
                    operation: HostOpV1::FsOpen,
                    capability: Some(CapabilityTraceIdentity("declared:FS".into())),
                    resource: Some(ResourceTraceIdentityV1(1)),
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
                EffectEventV1 {
                    sequence: 4,
                    operation: HostOpV1::ResourceRelease,
                    capability: None,
                    resource: Some(ResourceTraceIdentityV1(1)),
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
            ],
        }
    }

    #[test]
    fn linked_trace_codec_roundtrips_identity_bytes_and_outcomes() {
        let expected = representative_trace();
        let encoded = encode_linked_effect_trace_v1(&expected).unwrap();
        assert_eq!(decode_linked_effect_trace_v1(&encoded), Ok(expected));
    }

    #[test]
    fn home_failure_wire_distinguishes_lookup_from_root_open_and_keeps_identity() {
        let mut lookup = representative_trace();
        lookup.terminal_error = Some(crate::TerminalErrorV1::HomeRootResolutionFailed(
            crate::HomeRootResolutionFailureV1::AccountLookup(IoErrorIdentityV1::Other(5)),
        ));
        let mut root_open = representative_trace();
        root_open.terminal_error = Some(crate::TerminalErrorV1::HomeRootResolutionFailed(
            crate::HomeRootResolutionFailureV1::RootOpen(IoErrorIdentityV1::Other(5)),
        ));

        let lookup_wire = encode_linked_effect_trace_v1(&lookup).unwrap();
        let root_open_wire = encode_linked_effect_trace_v1(&root_open).unwrap();
        assert_ne!(lookup_wire, root_open_wire);
        assert_eq!(decode_linked_effect_trace_v1(&lookup_wire), Ok(lookup));
        assert_eq!(
            decode_linked_effect_trace_v1(&root_open_wire),
            Ok(root_open)
        );
    }

    #[test]
    fn linked_trace_decoder_rejects_magic_truncation_trailing_and_count_drift() {
        let encoded = encode_linked_effect_trace_v1(&representative_trace()).unwrap();

        let mut wrong_magic = encoded.clone();
        wrong_magic[0] ^= 1;
        assert_eq!(
            decode_linked_effect_trace_v1(&wrong_magic),
            Err(EffectTraceWireErrorV1)
        );
        assert_eq!(
            decode_linked_effect_trace_v1(&encoded[..encoded.len() - 1]),
            Err(EffectTraceWireErrorV1)
        );
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert_eq!(
            decode_linked_effect_trace_v1(&trailing),
            Err(EffectTraceWireErrorV1)
        );
        let mut impossible_count = encoded;
        impossible_count[88..96].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            decode_linked_effect_trace_v1(&impossible_count),
            Err(EffectTraceWireErrorV1)
        );
    }
}
