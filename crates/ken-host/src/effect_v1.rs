//! Canonical host-effect vocabulary shared by the interpreter, native runtime,
//! and the independent differential harness.
//!
//! These values are semantic observations, not the private raw-pointer ABI.
//! They deliberately exclude descriptors, pointers, inode identities, absolute
//! host roots, and diagnostic prose.

use std::io;

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
    FsChangeMode = 0x030A,
}

impl HostOpV1 {
    pub const ALL: [Self; 15] = [
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
        Self::FsChangeMode,
    ];

    pub const fn availability(self) -> HostOpAvailabilityV1 {
        if matches!(
            self,
            Self::ConsoleWrite
                | Self::ConsoleFlush
                | Self::ConsoleIsTerminal
                | Self::FsReadFile
                | Self::FsWriteFile
                | Self::FsChangeMode
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

pub const NATIVE_TESTED_TARGETS_V1: [HostOpV1; 6] = [
    HostOpV1::ConsoleWrite,
    HostOpV1::ConsoleFlush,
    HostOpV1::ConsoleIsTerminal,
    HostOpV1::FsReadFile,
    HostOpV1::FsWriteFile,
    HostOpV1::FsChangeMode,
];

pub const HOST_EFFECT_ABI_V1_SCHEMA_VERSION: u32 = 1;
include!(concat!(env!("OUT_DIR"), "/host_effect_abi_v1.rs"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostEffectAbiV1 {
    pub schema_version: u32,
    pub operation_count: u16,
    pub native_tested_count: u16,
    pub capability_token_size: u16,
    pub capability_token_align: u16,
    pub response_arena_lifetime_version: u16,
    pub trace_schema_version: u16,
    pub filesystem_observation_schema_version: u16,
    pub manifest_hash: [u8; 32],
}

pub const HOST_EFFECT_ABI_V1: HostEffectAbiV1 = HostEffectAbiV1 {
    schema_version: HOST_EFFECT_ABI_V1_SCHEMA_VERSION,
    operation_count: HOST_EFFECT_ABI_V1_CATALOG.len() as u16,
    native_tested_count: NATIVE_TESTED_TARGETS_V1.len() as u16,
    capability_token_size: std::mem::size_of::<CapabilityTokenV1>() as u16,
    capability_token_align: std::mem::align_of::<CapabilityTokenV1>() as u16,
    response_arena_lifetime_version: 1,
    trace_schema_version: 1,
    filesystem_observation_schema_version: 2,
    manifest_hash: HOST_EFFECT_ABI_V1_HASH,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEffectWireLayoutV1 {
    pub request_size: u32,
    pub request_align_shift: u8,
    pub request_offsets: Vec<u32>,
    pub reply_size: u32,
    pub reply_align_shift: u8,
    pub reply_tag_offset: u32,
    pub reply_detail_offset: u32,
    pub reply_bytes_data_offset: u32,
    pub reply_bytes_len_offset: u32,
    pub reply_unit_tag: u64,
    pub reply_bool_tag: u64,
    pub reply_bytes_tag: u64,
    pub reply_error_tag: u64,
}

fn generated_layout_fact(name: &str) -> Result<u64, TerminalErrorV1> {
    HOST_EFFECT_ABI_V1_FACTS
        .iter()
        .find_map(|(fact, value)| (*fact == name).then_some(*value))
        .ok_or(TerminalErrorV1::HostEffectAbiMismatch)
}

fn generated_binding(kind: &str, name: &str) -> Result<u64, TerminalErrorV1> {
    HOST_EFFECT_ABI_V1_BINDINGS
        .iter()
        .find_map(|(bound_kind, bound_name, value)| {
            (*bound_kind == kind && *bound_name == name).then_some(*value)
        })
        .ok_or(TerminalErrorV1::HostEffectAbiMismatch)
}

fn checked_u32(value: u64) -> Result<u32, TerminalErrorV1> {
    u32::try_from(value).map_err(|_| TerminalErrorV1::HostEffectAbiMismatch)
}

fn align_shift(align: u64) -> Result<u8, TerminalErrorV1> {
    if align == 0 || !align.is_power_of_two() {
        return Err(TerminalErrorV1::HostEffectAbiMismatch);
    }
    u8::try_from(align.trailing_zeros()).map_err(|_| TerminalErrorV1::HostEffectAbiMismatch)
}

/// Returns the target-C-probed layout consumed by the live Cranelift emitter.
/// No request size, alignment, offset, or reply tag is restated in Runtime.
pub fn host_effect_wire_layout_v1(
    operation: HostOpV1,
) -> Result<HostEffectWireLayoutV1, TerminalErrorV1> {
    let row = HOST_EFFECT_ABI_V1_CATALOG
        .iter()
        .find(|row| row.1 == operation as u16)
        .ok_or(TerminalErrorV1::HostEffectAbiMismatch)?;
    let request = row.3;
    let size = |record: &str| generated_layout_fact(&format!("SIZE_{record}"));
    let align = |record: &str| generated_layout_fact(&format!("ALIGN_{record}"));
    let offset =
        |record: &str, field: &str| generated_layout_fact(&format!("OFFSET_{record}_{field}"));
    let slice_data = offset("SliceV1", "data")?;
    let slice_len = offset("SliceV1", "len")?;
    let field = |name: &str| offset(request, name);
    let slice = |name: &str| -> Result<[u32; 2], TerminalErrorV1> {
        let base = field(name)?;
        Ok([
            checked_u32(base + slice_data)?,
            checked_u32(base + slice_len)?,
        ])
    };
    let request_offsets = match operation {
        HostOpV1::ConsoleWrite => {
            let bytes = slice("bytes")?;
            vec![checked_u32(field("stream")?)?, bytes[0], bytes[1]]
        }
        HostOpV1::ConsoleFlush | HostOpV1::ConsoleIsTerminal => {
            vec![checked_u32(field("stream")?)?]
        }
        HostOpV1::FsReadFile => {
            let path = slice("path")?;
            vec![checked_u32(field("capability")?)?, path[0], path[1]]
        }
        HostOpV1::FsWriteFile => {
            let path = slice("path")?;
            let bytes = slice("bytes")?;
            vec![
                checked_u32(field("capability")?)?,
                path[0],
                path[1],
                checked_u32(field("create_policy")?)?,
                bytes[0],
                bytes[1],
            ]
        }
        HostOpV1::FsChangeMode => {
            let path = slice("path")?;
            vec![
                checked_u32(field("capability")?)?,
                path[0],
                path[1],
                checked_u32(field("mode")?)?,
            ]
        }
        _ => return Err(TerminalErrorV1::OperationUnavailable(operation)),
    };
    let reply_bytes = offset("HostReplyV1", "bytes")?;
    Ok(HostEffectWireLayoutV1 {
        request_size: checked_u32(size(request)?)?,
        request_align_shift: align_shift(align(request)?)?,
        request_offsets,
        reply_size: checked_u32(size("HostReplyV1")?)?,
        reply_align_shift: align_shift(align("HostReplyV1")?)?,
        reply_tag_offset: checked_u32(offset("HostReplyV1", "tag")?)?,
        reply_detail_offset: checked_u32(offset("HostReplyV1", "detail")?)?,
        reply_bytes_data_offset: checked_u32(reply_bytes + slice_data)?,
        reply_bytes_len_offset: checked_u32(reply_bytes + slice_len)?,
        reply_unit_tag: generated_binding("tag", "reply.unit")?,
        reply_bool_tag: generated_binding("tag", "reply.bool")?,
        reply_bytes_tag: generated_binding("tag", "reply.bytes")?,
        reply_error_tag: generated_binding("tag", "reply.error")?,
    })
}

pub fn verify_host_effect_wire_layout_v1(
    operation: HostOpV1,
    candidate: &HostEffectWireLayoutV1,
) -> Result<(), TerminalErrorV1> {
    if &host_effect_wire_layout_v1(operation)? == candidate {
        Ok(())
    } else {
        Err(TerminalErrorV1::HostEffectAbiMismatch)
    }
}

pub fn verify_host_effect_inventory_v1(
    producer: &[u16],
    registry: &[u16],
    observer: &[u16],
    consumers: &[u16],
) -> Result<(), TerminalErrorV1> {
    let closed = |values: &[u16]| {
        let mut values = values.to_vec();
        values.sort_unstable();
        values.dedup();
        values
    };
    let producer = closed(producer);
    if producer.len() == HostOpV1::ALL.len()
        && producer == closed(registry)
        && producer == closed(observer)
        && producer == closed(consumers)
    {
        Ok(())
    } else {
        Err(TerminalErrorV1::HostEffectAbiMismatch)
    }
}

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
pub const RIGHT_CHANGE_MODE_V1: u8 = crate::RightSet::CHANGE_MODE.bits();

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
    fn console_read(
        &mut self,
        _stream: ConsoleStreamV1,
        _limit: u64,
    ) -> Result<CanonicalReplyV1, IoErrorIdentityV1> {
        Err(IoErrorIdentityV1::Unsupported)
    }
    fn console_write(
        &mut self,
        stream: ConsoleStreamV1,
        bytes: &[u8],
    ) -> Result<(), IoErrorIdentityV1>;
    fn console_flush(&mut self, stream: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1>;
    fn console_is_terminal(&mut self, stream: ConsoleStreamV1) -> bool;
    fn clock_wall_now(&mut self) -> Vec<u8> {
        Vec::new()
    }
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
    fn fs_append_file(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
        _bytes: &[u8],
    ) -> Result<(), FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
    fn fs_metadata(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
    ) -> Result<FileMetadataV1, FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
    fn fs_read_directory(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
    ) -> Result<Vec<DirEntryV1>, FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
    fn fs_create_directory(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
        _recursive: bool,
    ) -> Result<(), FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
    fn fs_remove_file(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
    ) -> Result<(), FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
    fn fs_remove_directory(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
        _recursive: bool,
    ) -> Result<(), FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
    fn fs_rename(
        &mut self,
        _grant: &CapabilityGrantV1,
        _source: &[u8],
        _destination: &[u8],
    ) -> Result<(), FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
    fn fs_change_mode(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
        _mode: u16,
    ) -> Result<(), FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
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
    let required = match operation {
        HostOpV1::FsReadFile => Some((crate::FsCapabilityOperation::Read, crate::AUTH_PARTIAL)),
        HostOpV1::FsWriteFile => Some((crate::FsCapabilityOperation::Write, crate::AUTH_FULL)),
        HostOpV1::FsAppendFile => Some((crate::FsCapabilityOperation::Append, crate::AUTH_FULL)),
        HostOpV1::FsMetadata => Some((crate::FsCapabilityOperation::Metadata, crate::AUTH_PARTIAL)),
        HostOpV1::FsReadDirectory => {
            Some((crate::FsCapabilityOperation::Enumerate, crate::AUTH_PARTIAL))
        }
        HostOpV1::FsCreateDirectory => Some((
            crate::FsCapabilityOperation::CreateDirectory,
            crate::AUTH_FULL,
        )),
        HostOpV1::FsRemoveFile => {
            Some((crate::FsCapabilityOperation::RemoveFile, crate::AUTH_FULL))
        }
        HostOpV1::FsRemoveDirectory => Some((
            crate::FsCapabilityOperation::RemoveDirectory,
            crate::AUTH_FULL,
        )),
        HostOpV1::FsRename => Some((crate::FsCapabilityOperation::RenameSource, crate::AUTH_FULL)),
        HostOpV1::FsChangeMode => {
            Some((crate::FsCapabilityOperation::ChangeMode, crate::AUTH_FULL))
        }
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
        (HostOpV1::ConsoleRead, CanonicalRequestV1::ConsoleRead { stream, limit }) => backend
            .console_read(*stream, *limit)
            .map_err(SemanticErrorV1::Io),
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
        (HostOpV1::ClockWallNow, CanonicalRequestV1::ClockWallNow) => {
            Ok(CanonicalReplyV1::Instant(backend.clock_wall_now()))
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
        (HostOpV1::FsAppendFile, CanonicalRequestV1::FsAppendFile { path, bytes }) => backend
            .fs_append_file(grant.expect("validated FS capability"), path, bytes)
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(|cause| file_error(operation, path, cause)),
        (HostOpV1::FsMetadata, CanonicalRequestV1::FsMetadata { path }) => backend
            .fs_metadata(grant.expect("validated FS capability"), path)
            .map(CanonicalReplyV1::FileMetadata)
            .map_err(|cause| file_error(operation, path, cause)),
        (HostOpV1::FsReadDirectory, CanonicalRequestV1::FsReadDirectory { path }) => backend
            .fs_read_directory(grant.expect("validated FS capability"), path)
            .map(CanonicalReplyV1::DirectoryEntries)
            .map_err(|cause| file_error(operation, path, cause)),
        (
            HostOpV1::FsCreateDirectory,
            CanonicalRequestV1::FsCreateDirectory { recursive, path },
        ) => backend
            .fs_create_directory(grant.expect("validated FS capability"), path, *recursive)
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(|cause| file_error(operation, path, cause)),
        (HostOpV1::FsRemoveFile, CanonicalRequestV1::FsRemoveFile { path }) => backend
            .fs_remove_file(grant.expect("validated FS capability"), path)
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(|cause| file_error(operation, path, cause)),
        (
            HostOpV1::FsRemoveDirectory,
            CanonicalRequestV1::FsRemoveDirectory { recursive, path },
        ) => backend
            .fs_remove_directory(grant.expect("validated FS capability"), path, *recursive)
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(|cause| file_error(operation, path, cause)),
        (
            HostOpV1::FsRename,
            CanonicalRequestV1::FsRename {
                source,
                destination,
            },
        ) => backend
            .fs_rename(grant.expect("validated FS capability"), source, destination)
            .map(|()| CanonicalReplyV1::Unit)
            .map_err(|cause| file_error(operation, source, cause)),
        (HostOpV1::FsChangeMode, CanonicalRequestV1::FsChangeMode { path, mode }) => backend
            .fs_change_mode(grant.expect("validated FS capability"), path, *mode)
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
                    crate::FsCapabilityOperation::ChangeMode => FsCapabilityOperationV1::ChangeMode,
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
        CanonicalRequestV1::FsReadFile { path }
        | CanonicalRequestV1::FsWriteFile { path, .. }
        | CanonicalRequestV1::FsAppendFile { path, .. }
        | CanonicalRequestV1::FsMetadata { path }
        | CanonicalRequestV1::FsReadDirectory { path }
        | CanonicalRequestV1::FsCreateDirectory { path, .. }
        | CanonicalRequestV1::FsRemoveFile { path }
        | CanonicalRequestV1::FsRemoveDirectory { path, .. } => path.clone(),
        CanonicalRequestV1::FsRename { source, .. } => source.clone(),
        CanonicalRequestV1::FsChangeMode { path, .. } => path.clone(),
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

pub const PROGRAM_CAPS_FS_TRACE_IDENTITY_V1: &str = "FS";

pub fn program_caps_fs_trace_identity_v1() -> CapabilityTraceIdentity {
    CapabilityTraceIdentity(PROGRAM_CAPS_FS_TRACE_IDENTITY_V1.to_string())
}

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
    FsChangeMode {
        path: Vec<u8>,
        mode: u16,
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
    ChangeMode,
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

/// The one host-neutral mapping from Rust I/O errors into Ken's semantic
/// error identity. Both interpreter and native executors reify this value.
pub fn io_error_identity_v1(error: &io::Error) -> IoErrorIdentityV1 {
    match error.kind() {
        io::ErrorKind::NotFound => IoErrorIdentityV1::NotFound,
        io::ErrorKind::PermissionDenied => IoErrorIdentityV1::PermissionDenied,
        io::ErrorKind::BrokenPipe => IoErrorIdentityV1::BrokenPipe,
        io::ErrorKind::Interrupted => IoErrorIdentityV1::Interrupted,
        io::ErrorKind::AlreadyExists => IoErrorIdentityV1::AlreadyExists,
        io::ErrorKind::InvalidInput => IoErrorIdentityV1::InvalidInput,
        io::ErrorKind::IsADirectory => IoErrorIdentityV1::IsDirectory,
        io::ErrorKind::NotADirectory => IoErrorIdentityV1::NotDirectory,
        io::ErrorKind::DirectoryNotEmpty => IoErrorIdentityV1::NotEmpty,
        io::ErrorKind::Unsupported => IoErrorIdentityV1::Unsupported,
        _ => IoErrorIdentityV1::Other(error.raw_os_error().unwrap_or(0)),
    }
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
    pub mode: Option<u16>,
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
    RootExecutionDenied,
    HomeRootResolutionFailed(crate::HomeRootResolutionFailureV1),
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
    use sha2::{Digest, Sha256};

    #[derive(Default)]
    struct AllOpsBackend(Vec<HostOpV1>);

    impl HostEffectBackendV1 for AllOpsBackend {
        fn console_read(
            &mut self,
            _: ConsoleStreamV1,
            _: u64,
        ) -> Result<CanonicalReplyV1, IoErrorIdentityV1> {
            self.0.push(HostOpV1::ConsoleRead);
            Ok(CanonicalReplyV1::ReadEof)
        }
        fn console_write(&mut self, _: ConsoleStreamV1, _: &[u8]) -> Result<(), IoErrorIdentityV1> {
            self.0.push(HostOpV1::ConsoleWrite);
            Ok(())
        }
        fn console_flush(&mut self, _: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1> {
            self.0.push(HostOpV1::ConsoleFlush);
            Ok(())
        }
        fn console_is_terminal(&mut self, _: ConsoleStreamV1) -> bool {
            self.0.push(HostOpV1::ConsoleIsTerminal);
            false
        }
        fn clock_wall_now(&mut self) -> Vec<u8> {
            self.0.push(HostOpV1::ClockWallNow);
            vec![1]
        }
        fn fs_read_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
        ) -> Result<Vec<u8>, FileErrorCauseV1> {
            self.0.push(HostOpV1::FsReadFile);
            Ok(Vec::new())
        }
        fn fs_write_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: CreatePolicyV1,
            _: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            self.0.push(HostOpV1::FsWriteFile);
            Ok(())
        }
        fn fs_append_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            self.0.push(HostOpV1::FsAppendFile);
            Ok(())
        }
        fn fs_metadata(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
        ) -> Result<FileMetadataV1, FileErrorCauseV1> {
            self.0.push(HostOpV1::FsMetadata);
            Ok(FileMetadataV1 {
                size: 0,
                kind: FsNodeKindV1::File,
            })
        }
        fn fs_read_directory(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
        ) -> Result<Vec<DirEntryV1>, FileErrorCauseV1> {
            self.0.push(HostOpV1::FsReadDirectory);
            Ok(Vec::new())
        }
        fn fs_create_directory(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: bool,
        ) -> Result<(), FileErrorCauseV1> {
            self.0.push(HostOpV1::FsCreateDirectory);
            Ok(())
        }
        fn fs_remove_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            self.0.push(HostOpV1::FsRemoveFile);
            Ok(())
        }
        fn fs_remove_directory(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: bool,
        ) -> Result<(), FileErrorCauseV1> {
            self.0.push(HostOpV1::FsRemoveDirectory);
            Ok(())
        }
        fn fs_rename(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            self.0.push(HostOpV1::FsRename);
            Ok(())
        }
        fn fs_change_mode(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: u16,
        ) -> Result<(), FileErrorCauseV1> {
            self.0.push(HostOpV1::FsChangeMode);
            Ok(())
        }
    }

    #[test]
    fn all_fifteen_operations_share_one_semantic_dispatch() {
        let mut capabilities = CapabilityTableV1::default();
        let token = capabilities.insert(CapabilityGrantV1 {
            identity: CapabilityTraceIdentity("test:all".to_string()),
            capability: crate::Cap::mint_scoped(
                crate::AUTH_FULL,
                "FS",
                crate::FsScope::root(
                    crate::RightSet::from_bits(u8::MAX),
                    crate::FsHandle::Virtual(0),
                    crate::FsIdentity::Virtual(0),
                    crate::SymlinkPolicy::NoFollow,
                ),
            ),
        });
        let requests = [
            (
                HostOpV1::ConsoleRead,
                CanonicalRequestV1::ConsoleRead {
                    stream: ConsoleStreamV1::Stdin,
                    limit: 1,
                },
            ),
            (
                HostOpV1::ConsoleWrite,
                CanonicalRequestV1::ConsoleWrite {
                    stream: ConsoleStreamV1::Stdout,
                    bytes: vec![],
                },
            ),
            (
                HostOpV1::ConsoleFlush,
                CanonicalRequestV1::ConsoleFlush {
                    stream: ConsoleStreamV1::Stdout,
                },
            ),
            (
                HostOpV1::ConsoleIsTerminal,
                CanonicalRequestV1::ConsoleIsTerminal {
                    stream: ConsoleStreamV1::Stdout,
                },
            ),
            (HostOpV1::ClockWallNow, CanonicalRequestV1::ClockWallNow),
            (
                HostOpV1::FsReadFile,
                CanonicalRequestV1::FsReadFile {
                    path: b"a".to_vec(),
                },
            ),
            (
                HostOpV1::FsWriteFile,
                CanonicalRequestV1::FsWriteFile {
                    path: b"a".to_vec(),
                    create_policy: CreatePolicyV1::CreateNew,
                    bytes: vec![],
                },
            ),
            (
                HostOpV1::FsAppendFile,
                CanonicalRequestV1::FsAppendFile {
                    path: b"a".to_vec(),
                    bytes: vec![],
                },
            ),
            (
                HostOpV1::FsMetadata,
                CanonicalRequestV1::FsMetadata {
                    path: b"a".to_vec(),
                },
            ),
            (
                HostOpV1::FsReadDirectory,
                CanonicalRequestV1::FsReadDirectory {
                    path: b"a".to_vec(),
                },
            ),
            (
                HostOpV1::FsCreateDirectory,
                CanonicalRequestV1::FsCreateDirectory {
                    recursive: false,
                    path: b"a".to_vec(),
                },
            ),
            (
                HostOpV1::FsRemoveFile,
                CanonicalRequestV1::FsRemoveFile {
                    path: b"a".to_vec(),
                },
            ),
            (
                HostOpV1::FsRemoveDirectory,
                CanonicalRequestV1::FsRemoveDirectory {
                    recursive: false,
                    path: b"a".to_vec(),
                },
            ),
            (
                HostOpV1::FsRename,
                CanonicalRequestV1::FsRename {
                    source: b"a".to_vec(),
                    destination: b"b".to_vec(),
                },
            ),
            (
                HostOpV1::FsChangeMode,
                CanonicalRequestV1::FsChangeMode {
                    path: b"a".to_vec(),
                    mode: 0o640,
                },
            ),
        ];
        let mut backend = AllOpsBackend::default();
        for (operation, request) in requests {
            let capability = (!operation.is_ambient()).then_some(token);
            dispatch_host_op_v1(&mut backend, &capabilities, operation, capability, &request)
                .unwrap();
        }
        assert_eq!(backend.0, HostOpV1::ALL);
    }

    #[test]
    fn catalog_is_closed_and_availability_is_exact() {
        assert_eq!(HostOpV1::ALL.len(), 15);
        assert_eq!(
            HostOpV1::ALL
                .into_iter()
                .filter(|operation| operation.availability() == HostOpAvailabilityV1::NativeTested)
                .collect::<Vec<_>>(),
            NATIVE_TESTED_TARGETS_V1
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
    fn generated_manifest_closes_catalog_observer_and_consumer_sets() {
        let producer = HOST_EFFECT_ABI_V1_CATALOG
            .iter()
            .map(|row| row.1)
            .collect::<std::collections::BTreeSet<_>>();
        let registry = HostOpV1::ALL
            .into_iter()
            .map(|operation| operation as u16)
            .collect::<std::collections::BTreeSet<_>>();
        let observer = HOST_EFFECT_ABI_V1_CANONICAL
            .lines()
            .filter_map(|line| line.strip_prefix("operation="))
            .map(|line| u16::from_str_radix(line.split('|').nth(1).unwrap(), 16).unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        let consumers = HostOpV1::ALL
            .into_iter()
            .map(|operation| operation as u16)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(producer, registry);
        assert_eq!(producer, observer);
        assert_eq!(producer, consumers);
        let as_vec =
            |set: &std::collections::BTreeSet<u16>| set.iter().copied().collect::<Vec<_>>();
        assert_eq!(
            verify_host_effect_inventory_v1(
                &as_vec(&producer),
                &as_vec(&registry),
                &as_vec(&observer),
                &as_vec(&consumers),
            ),
            Ok(())
        );
        for row in HOST_EFFECT_ABI_V1_CATALOG {
            assert!(HOST_EFFECT_ABI_V1_FACTS
                .iter()
                .any(|(name, _)| *name == format!("SIZE_{}", row.3)));
            assert!(HOST_EFFECT_ABI_V1_FACTS
                .iter()
                .any(|(name, _)| *name == format!("ALIGN_{}", row.3)));
            assert!(HOST_EFFECT_ABI_V1_FACTS
                .iter()
                .any(|(name, _)| *name == format!("SIZE_{}", row.5)));
            assert!(HOST_EFFECT_ABI_V1_FACTS
                .iter()
                .any(|(name, _)| *name == format!("ALIGN_{}", row.5)));
        }

        let mut producer_only = producer.clone();
        producer_only.insert(0x7fff);
        assert!(verify_host_effect_inventory_v1(
            &as_vec(&producer_only),
            &as_vec(&registry),
            &as_vec(&observer),
            &as_vec(&consumers),
        )
        .is_err());
        let mut registry_only = registry.clone();
        registry_only.remove(&(HostOpV1::ConsoleWrite as u16));
        assert!(verify_host_effect_inventory_v1(
            &as_vec(&producer),
            &as_vec(&registry_only),
            &as_vec(&observer),
            &as_vec(&consumers),
        )
        .is_err());
        let mut observer_only = observer.clone();
        observer_only.insert(0x7ffe);
        assert!(verify_host_effect_inventory_v1(
            &as_vec(&producer),
            &as_vec(&registry),
            &as_vec(&observer_only),
            &as_vec(&consumers),
        )
        .is_err());
        let mut consumer_only = consumers.clone();
        consumer_only.remove(&(HostOpV1::FsRename as u16));
        assert!(verify_host_effect_inventory_v1(
            &as_vec(&producer),
            &as_vec(&registry),
            &as_vec(&observer),
            &as_vec(&consumer_only),
        )
        .is_err());
    }

    #[test]
    fn every_catalog_or_layout_value_mutation_changes_the_manifest_hash() {
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(HOST_EFFECT_ABI_V1_CANONICAL.as_bytes())),
            HOST_EFFECT_ABI_V1_HASH
        );
        for needle in [
            "ConsoleWrite|0102|native|ConsoleWriteRequestV1|2|HostReplyV1|1",
            "FsRename|0309|unavailable|FsRenameRequestV1|3|HostReplyV1|1",
            "FsChangeMode|030a|native|FsChangeModeRequestV1|3|HostReplyV1|1",
            "lifetime=filesystem_observation_schema|2",
            "layout=OFFSET_FsChangeModeRequestV1_mode|24",
            "layout=SIZE_HostReplyV1|",
            "error=io.BrokenPipe|3",
            "tag=reply.error|3",
        ] {
            assert!(HOST_EFFECT_ABI_V1_CANONICAL.contains(needle));
            let mutated = HOST_EFFECT_ABI_V1_CANONICAL.replacen(needle, "MUTATED", 1);
            assert_ne!(
                <[u8; 32]>::from(Sha256::digest(mutated.as_bytes())),
                HOST_EFFECT_ABI_V1_HASH
            );
        }
    }

    #[test]
    fn generated_manifest_binds_the_opaque_capability_layout() {
        let fact = |name: &str| {
            HOST_EFFECT_ABI_V1_FACTS
                .iter()
                .find_map(|(fact, value)| (*fact == name).then_some(*value))
                .unwrap()
        };
        assert_eq!(
            fact("SIZE_CapabilityTokenV1"),
            std::mem::size_of::<CapabilityTokenV1>() as u64
        );
        assert_eq!(
            fact("ALIGN_CapabilityTokenV1"),
            std::mem::align_of::<CapabilityTokenV1>() as u64
        );
        assert_eq!(
            fact("OFFSET_CapabilityTokenV1_slot"),
            std::mem::offset_of!(CapabilityTokenV1, slot) as u64
        );
        assert_eq!(
            fact("OFFSET_CapabilityTokenV1_generation"),
            std::mem::offset_of!(CapabilityTokenV1, generation) as u64
        );
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
