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
    FsOpen = 0x030B,
    FsHandleMetadata = 0x030C,
    FsReadAt = 0x030D,
    FsWriteAt = 0x030E,
    ResourceRelease = 0x0401,
    BufferAllocate = 0x0402,
    BufferFreeze = 0x0403,
}

impl HostOpV1 {
    pub const ALL: [Self; 22] = [
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
        Self::FsOpen,
        Self::FsHandleMetadata,
        Self::FsReadAt,
        Self::FsWriteAt,
        Self::ResourceRelease,
        Self::BufferAllocate,
        Self::BufferFreeze,
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
                | Self::FsOpen
                | Self::FsHandleMetadata
                | Self::ResourceRelease
                | Self::FsReadAt
                | Self::FsWriteAt
                | Self::BufferAllocate
                | Self::BufferFreeze
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

pub const NATIVE_TESTED_TARGETS_V1: [HostOpV1; 13] = [
    HostOpV1::ConsoleWrite,
    HostOpV1::ConsoleFlush,
    HostOpV1::ConsoleIsTerminal,
    HostOpV1::FsReadFile,
    HostOpV1::FsWriteFile,
    HostOpV1::FsChangeMode,
    HostOpV1::FsOpen,
    HostOpV1::FsHandleMetadata,
    HostOpV1::FsReadAt,
    HostOpV1::FsWriteAt,
    HostOpV1::ResourceRelease,
    HostOpV1::BufferAllocate,
    HostOpV1::BufferFreeze,
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
    pub resource_token_size: u16,
    pub resource_token_align: u16,
    pub response_arena_lifetime_version: u16,
    pub trace_schema_version: u16,
    pub filesystem_observation_schema_version: u16,
    pub resource_observation_schema_version: u16,
    pub buffer_limits: BufferLimitsV1,
    pub manifest_hash: [u8; 32],
}

pub const HOST_EFFECT_ABI_V1: HostEffectAbiV1 = HostEffectAbiV1 {
    schema_version: HOST_EFFECT_ABI_V1_SCHEMA_VERSION,
    operation_count: HOST_EFFECT_ABI_V1_CATALOG.len() as u16,
    native_tested_count: NATIVE_TESTED_TARGETS_V1.len() as u16,
    capability_token_size: std::mem::size_of::<CapabilityTokenV1>() as u16,
    capability_token_align: std::mem::align_of::<CapabilityTokenV1>() as u16,
    resource_token_size: std::mem::size_of::<ResourceTokenV1>() as u16,
    resource_token_align: std::mem::align_of::<ResourceTokenV1>() as u16,
    response_arena_lifetime_version: 1,
    trace_schema_version: 1,
    filesystem_observation_schema_version: 2,
    resource_observation_schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
    buffer_limits: DEFAULT_BUFFER_LIMITS_V1,
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
    pub reply_effective_request_offset: u32,
    pub reply_bytes_data_offset: u32,
    pub reply_bytes_len_offset: u32,
    pub reply_resource_error_schema_offset: u32,
    pub reply_resource_error_kind_offset: u32,
    pub reply_resource_error_identity_offset: u32,
    pub reply_resource_error_io_offset: u32,
    pub reply_resource_error_required_offset: u32,
    pub reply_resource_error_held_offset: u32,
    pub reply_resource_error_expected_kind_offset: u32,
    pub reply_resource_error_actual_kind_offset: u32,
    pub reply_unit_tag: u64,
    pub reply_bool_tag: u64,
    pub reply_bytes_tag: u64,
    pub reply_error_tag: u64,
    pub reply_resource_tag: u64,
    pub reply_metadata_tag: u64,
    pub reply_resource_error_tag: u64,
    pub reply_read_progress_tag: u64,
    pub reply_write_progress_tag: u64,
    pub resource_error_closed: u64,
    pub resource_error_malformed: u64,
    pub resource_error_right_not_held: u64,
    pub resource_error_release_failed: u64,
    pub resource_error_kind_mismatch: u64,
    pub resource_error_buffer_limit: u64,
    pub resource_error_invalid_offset: u64,
    pub resource_error_invalid_bounds: u64,
    pub resource_error_no_progress: u64,
    pub resource_kind_fs_handle: u64,
    pub resource_kind_buffer: u64,
    pub resource_error_reply_schema: u64,
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
        HostOpV1::FsOpen => {
            let path = slice("path")?;
            vec![
                checked_u32(field("capability")?)?,
                path[0],
                path[1],
                checked_u32(field("mode")?)?,
            ]
        }
        HostOpV1::FsHandleMetadata | HostOpV1::ResourceRelease => {
            vec![checked_u32(field("resource")?)?]
        }
        HostOpV1::FsReadAt => vec![
            checked_u32(field("file")?)?,
            checked_u32(field("buffer")?)?,
            checked_u32(field("file_offset")?)?,
            checked_u32(field("buffer_start")?)?,
            checked_u32(field("length")?)?,
        ],
        // PX8-SPAN-PROV: `FsWriteAt` carries the span's originating buffer
        // acquisition (`span_origin`) so the shared dispatcher can admit it.
        HostOpV1::FsWriteAt => vec![
            checked_u32(field("file")?)?,
            checked_u32(field("buffer")?)?,
            checked_u32(field("file_offset")?)?,
            checked_u32(field("buffer_start")?)?,
            checked_u32(field("length")?)?,
            checked_u32(field("span_origin")?)?,
        ],
        HostOpV1::BufferAllocate => vec![checked_u32(field("capacity")?)?],
        HostOpV1::BufferFreeze => vec![
            checked_u32(field("resource")?)?,
            checked_u32(field("start")?)?,
            checked_u32(field("length")?)?,
            checked_u32(field("span_origin")?)?,
        ],
        _ => return Err(TerminalErrorV1::OperationUnavailable(operation)),
    };
    let reply_bytes = offset("HostReplyV1", "bytes")?;
    let reply_resource_error = offset("HostReplyV1", "resource_error")?;
    let resource_error_field = |name: &str| offset("ResourceErrorReplyV1", name);
    Ok(HostEffectWireLayoutV1 {
        request_size: checked_u32(size(request)?)?,
        request_align_shift: align_shift(align(request)?)?,
        request_offsets,
        reply_size: checked_u32(size("HostReplyV1")?)?,
        reply_align_shift: align_shift(align("HostReplyV1")?)?,
        reply_tag_offset: checked_u32(offset("HostReplyV1", "tag")?)?,
        reply_detail_offset: checked_u32(offset("HostReplyV1", "detail")?)?,
        reply_effective_request_offset: checked_u32(offset("HostReplyV1", "effective_request")?)?,
        reply_bytes_data_offset: checked_u32(reply_bytes + slice_data)?,
        reply_bytes_len_offset: checked_u32(reply_bytes + slice_len)?,
        reply_resource_error_schema_offset: checked_u32(
            reply_resource_error + resource_error_field("schema_version")?,
        )?,
        reply_resource_error_kind_offset: checked_u32(
            reply_resource_error + resource_error_field("resource_kind")?,
        )?,
        reply_resource_error_identity_offset: checked_u32(
            reply_resource_error + resource_error_field("identity")?,
        )?,
        reply_resource_error_io_offset: checked_u32(
            reply_resource_error + resource_error_field("io")?,
        )?,
        reply_resource_error_required_offset: checked_u32(
            reply_resource_error + resource_error_field("required")?,
        )?,
        reply_resource_error_held_offset: checked_u32(
            reply_resource_error + resource_error_field("held")?,
        )?,
        reply_resource_error_expected_kind_offset: checked_u32(
            reply_resource_error + resource_error_field("expected_kind")?,
        )?,
        reply_resource_error_actual_kind_offset: checked_u32(
            reply_resource_error + resource_error_field("actual_kind")?,
        )?,
        reply_unit_tag: generated_binding("tag", "reply.unit")?,
        reply_bool_tag: generated_binding("tag", "reply.bool")?,
        reply_bytes_tag: generated_binding("tag", "reply.bytes")?,
        reply_error_tag: generated_binding("tag", "reply.error")?,
        reply_resource_tag: generated_binding("tag", "reply.resource")?,
        reply_metadata_tag: generated_binding("tag", "reply.metadata")?,
        reply_resource_error_tag: generated_binding("tag", "reply.resource_error")?,
        reply_read_progress_tag: generated_binding("tag", "reply.read_progress")?,
        reply_write_progress_tag: generated_binding("tag", "reply.write_progress")?,
        resource_error_closed: generated_binding("error", "resource.Closed")?,
        resource_error_malformed: generated_binding("error", "resource.MalformedResource")?,
        resource_error_right_not_held: generated_binding("error", "resource.RightNotHeld")?,
        resource_error_release_failed: generated_binding("error", "resource.ReleaseFailed")?,
        resource_error_kind_mismatch: generated_binding("error", "resource.ResourceKindMismatch")?,
        resource_error_buffer_limit: generated_binding("error", "resource.BufferLimit")?,
        resource_error_invalid_offset: generated_binding("error", "resource.InvalidOffset")?,
        resource_error_invalid_bounds: generated_binding("error", "resource.InvalidBounds")?,
        resource_error_no_progress: generated_binding("error", "resource.NoProgress")?,
        resource_kind_fs_handle: generated_binding("tag", "resource_kind.FsHandle")?,
        resource_kind_buffer: generated_binding("tag", "resource_kind.Buffer")?,
        resource_error_reply_schema: generated_binding("lifetime", "resource_error_reply_schema")?,
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

/// Opaque resource carrier used only at the private host boundary.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceTokenV1 {
    slot: u32,
    generation: u32,
}

impl ResourceTokenV1 {
    pub fn erased_identity(self) -> u64 {
        (u64::from(self.generation) << 32) | u64::from(self.slot)
    }

    pub(crate) fn from_erased_identity(identity: u64) -> Self {
        Self {
            slot: identity as u32,
            generation: (identity >> 32) as u32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKindV1 {
    FsHandle,
    Buffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceTraceIdentityV1(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FsOpenModeV1 {
    Read,
    Metadata,
    WriteCreate(CreatePolicyV1),
}

impl FsOpenModeV1 {
    pub const fn required_right(self) -> crate::RightSet {
        match self {
            Self::Read => crate::RightSet::READ,
            Self::Metadata => crate::RightSet::METADATA,
            Self::WriteCreate(_) => crate::RightSet::WRITE.union(crate::RightSet::CREATE),
        }
    }

    const fn capability_operation(self) -> crate::FsCapabilityOperation {
        match self {
            Self::Read => crate::FsCapabilityOperation::Read,
            Self::Metadata => crate::FsCapabilityOperation::Metadata,
            Self::WriteCreate(_) => crate::FsCapabilityOperation::Write,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceErrorV1 {
    Closed,
    MalformedResource,
    ResourceKindMismatch {
        expected: ResourceKindV1,
        actual: ResourceKindV1,
    },
    RightNotHeld {
        required: u8,
        held: u8,
    },
    ReleaseFailed {
        schema_version: u16,
        resource_kind: ResourceKindV1,
        identity: ResourceTraceIdentityV1,
        io: IoErrorIdentityV1,
    },
    BufferLimit,
    InvalidOffset,
    InvalidBounds,
    NoProgress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferLimitsV1 {
    pub per_buffer_max_capacity: u64,
    pub invocation_max_live_capacity: u64,
}

pub const DEFAULT_BUFFER_LIMITS_V1: BufferLimitsV1 = BufferLimitsV1 {
    per_buffer_max_capacity: 1024 * 1024,
    invocation_max_live_capacity: 16 * 1024 * 1024,
};

impl BufferLimitsV1 {
    pub const fn new(
        per_buffer_max_capacity: u64,
        invocation_max_live_capacity: u64,
    ) -> Option<Self> {
        if per_buffer_max_capacity == 0
            || invocation_max_live_capacity == 0
            || per_buffer_max_capacity > invocation_max_live_capacity
        {
            None
        } else {
            Some(Self {
                per_buffer_max_capacity,
                invocation_max_live_capacity,
            })
        }
    }
}

impl Default for BufferLimitsV1 {
    fn default() -> Self {
        DEFAULT_BUFFER_LIMITS_V1
    }
}

#[derive(Debug)]
pub struct BufferRegionV1 {
    bytes: Vec<u8>,
    initialized_start: usize,
    initialized_len: usize,
}

impl BufferRegionV1 {
    fn new(capacity: usize) -> Self {
        Self {
            bytes: vec![0; capacity],
            initialized_start: 0,
            initialized_len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.bytes.len()
    }

    fn clear_window(&mut self) {
        self.initialized_start = 0;
        self.initialized_len = 0;
    }

    fn install_window(&mut self, start: usize, len: usize) {
        self.initialized_start = start;
        self.initialized_len = len;
    }

    fn initialized_slice(&self, start: usize, len: usize) -> Result<&[u8], ResourceErrorV1> {
        let end = start
            .checked_add(len)
            .ok_or(ResourceErrorV1::InvalidBounds)?;
        let live_end = self
            .initialized_start
            .checked_add(self.initialized_len)
            .ok_or(ResourceErrorV1::InvalidBounds)?;
        if start < self.initialized_start || end > live_end {
            return Err(ResourceErrorV1::InvalidBounds);
        }
        Ok(&self.bytes[start..end])
    }
}

#[derive(Debug)]
pub enum ResourceOwnerV1 {
    FsHandle(crate::ResourceHandleV1),
    Buffer(BufferRegionV1),
}

pub const RESOURCE_OBSERVATION_SCHEMA_VERSION_V1: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceSettlementOutcomeV1 {
    Released,
    ReleaseFailed(IoErrorIdentityV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceSettlementObservationV1 {
    pub schema_version: u16,
    pub resource_kind: ResourceKindV1,
    pub identity: ResourceTraceIdentityV1,
    pub outcome: ResourceSettlementOutcomeV1,
}

#[derive(Debug)]
enum ResourceSlotStateV1 {
    Live {
        owner: ResourceOwnerV1,
        kind: ResourceKindV1,
        rights: crate::RightSet,
        identity: ResourceTraceIdentityV1,
    },
    Vacant {
        last_identity: ResourceTraceIdentityV1,
    },
    Retired {
        last_identity: Option<ResourceTraceIdentityV1>,
    },
}

#[derive(Debug)]
struct ResourceSlotV1 {
    generation: u32,
    state: ResourceSlotStateV1,
}

#[derive(Debug, Default)]
pub struct ResourceTableV1 {
    slots: Vec<ResourceSlotV1>,
    next_acquisition_identity: u64,
    buffer_limits: BufferLimitsV1,
    live_buffer_capacity: u64,
}

#[derive(Debug)]
pub struct PendingResourceCloseV1 {
    owner: ResourceOwnerV1,
    kind: ResourceKindV1,
    identity: ResourceTraceIdentityV1,
}

impl ResourceTableV1 {
    pub fn with_buffer_limits(limits: BufferLimitsV1) -> Self {
        Self {
            slots: Vec::new(),
            next_acquisition_identity: 0,
            buffer_limits: limits,
            live_buffer_capacity: 0,
        }
    }

    fn insert_owner(
        &mut self,
        owner: ResourceOwnerV1,
        kind: ResourceKindV1,
        rights: crate::RightSet,
    ) -> (ResourceTokenV1, ResourceTraceIdentityV1) {
        self.next_acquisition_identity = self
            .next_acquisition_identity
            .checked_add(1)
            .expect("resource acquisition identity exhausted");
        let identity = ResourceTraceIdentityV1(self.next_acquisition_identity);
        if let Some((slot_index, slot)) = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| matches!(slot.state, ResourceSlotStateV1::Vacant { .. }))
        {
            slot.state = ResourceSlotStateV1::Live {
                owner,
                kind,
                rights,
                identity,
            };
            return (
                ResourceTokenV1 {
                    slot: slot_index as u32,
                    generation: slot.generation,
                },
                identity,
            );
        }
        let slot = u32::try_from(self.slots.len()).expect("resource table exceeds u32");
        let generation = 1;
        self.slots.push(ResourceSlotV1 {
            generation,
            state: ResourceSlotStateV1::Live {
                owner,
                kind,
                rights,
                identity,
            },
        });
        (ResourceTokenV1 { slot, generation }, identity)
    }

    pub fn insert_fs_handle(
        &mut self,
        owner: crate::ResourceHandleV1,
        rights: crate::RightSet,
    ) -> (ResourceTokenV1, ResourceTraceIdentityV1) {
        self.insert_owner(
            ResourceOwnerV1::FsHandle(owner),
            ResourceKindV1::FsHandle,
            rights,
        )
    }

    pub fn insert_buffer(
        &mut self,
        capacity: u64,
    ) -> Result<(ResourceTokenV1, ResourceTraceIdentityV1), ResourceErrorV1> {
        let total = self
            .live_buffer_capacity
            .checked_add(capacity)
            .ok_or(ResourceErrorV1::BufferLimit)?;
        if capacity == 0
            || capacity > self.buffer_limits.per_buffer_max_capacity
            || total > self.buffer_limits.invocation_max_live_capacity
        {
            return Err(ResourceErrorV1::BufferLimit);
        }
        let capacity = usize::try_from(capacity).map_err(|_| ResourceErrorV1::BufferLimit)?;
        let inserted = self.insert_owner(
            ResourceOwnerV1::Buffer(BufferRegionV1::new(capacity)),
            ResourceKindV1::Buffer,
            crate::RightSet::from_bits(0),
        );
        self.live_buffer_capacity = total;
        Ok(inserted)
    }

    pub fn resolve_fs_handle(
        &self,
        token: ResourceTokenV1,
        required: crate::RightSet,
    ) -> Result<(&crate::ResourceHandleV1, ResourceTraceIdentityV1), ResourceErrorV1> {
        let slot = self.lookup(token)?;
        let (owner, kind, rights, identity) = match &slot.state {
            ResourceSlotStateV1::Live {
                owner,
                kind,
                rights,
                identity,
                ..
            } => (owner, kind, rights, identity),
            ResourceSlotStateV1::Vacant { .. } => return Err(ResourceErrorV1::MalformedResource),
            ResourceSlotStateV1::Retired { .. } => return Err(ResourceErrorV1::Closed),
        };
        if *kind != ResourceKindV1::FsHandle {
            return Err(ResourceErrorV1::ResourceKindMismatch {
                expected: ResourceKindV1::FsHandle,
                actual: *kind,
            });
        }
        if !rights.contains(required) {
            return Err(ResourceErrorV1::RightNotHeld {
                required: required.bits(),
                held: rights.bits(),
            });
        }
        let ResourceOwnerV1::FsHandle(owner) = owner else {
            return Err(ResourceErrorV1::MalformedResource);
        };
        Ok((owner, *identity))
    }

    pub fn resolve_buffer(
        &self,
        token: ResourceTokenV1,
    ) -> Result<(&BufferRegionV1, ResourceTraceIdentityV1), ResourceErrorV1> {
        let slot = self.lookup(token)?;
        let (owner, kind, identity) = match &slot.state {
            ResourceSlotStateV1::Live {
                owner,
                kind,
                identity,
                ..
            } => (owner, kind, identity),
            ResourceSlotStateV1::Vacant { .. } => return Err(ResourceErrorV1::MalformedResource),
            ResourceSlotStateV1::Retired { .. } => return Err(ResourceErrorV1::Closed),
        };
        if *kind != ResourceKindV1::Buffer {
            return Err(ResourceErrorV1::ResourceKindMismatch {
                expected: ResourceKindV1::Buffer,
                actual: *kind,
            });
        }
        let ResourceOwnerV1::Buffer(buffer) = owner else {
            return Err(ResourceErrorV1::MalformedResource);
        };
        Ok((buffer, *identity))
    }

    pub fn identity(
        &self,
        token: ResourceTokenV1,
    ) -> Result<ResourceTraceIdentityV1, ResourceErrorV1> {
        if token.generation == 0 {
            return Err(ResourceErrorV1::MalformedResource);
        }
        let Some(slot) = self.slots.get(token.slot as usize) else {
            return Err(ResourceErrorV1::MalformedResource);
        };
        match &slot.state {
            ResourceSlotStateV1::Live { identity, .. } if token.generation == slot.generation => {
                Ok(*identity)
            }
            ResourceSlotStateV1::Live { .. } if token.generation < slot.generation => {
                Err(ResourceErrorV1::Closed)
            }
            ResourceSlotStateV1::Live { .. } => Err(ResourceErrorV1::MalformedResource),
            ResourceSlotStateV1::Vacant { last_identity }
                if token.generation.checked_add(1) == Some(slot.generation) =>
            {
                Ok(*last_identity)
            }
            ResourceSlotStateV1::Retired {
                last_identity: Some(last_identity),
            } if token.generation == slot.generation => Ok(*last_identity),
            ResourceSlotStateV1::Vacant { .. } | ResourceSlotStateV1::Retired { .. }
                if token.generation < slot.generation =>
            {
                Err(ResourceErrorV1::Closed)
            }
            ResourceSlotStateV1::Vacant { .. } => Err(ResourceErrorV1::MalformedResource),
            ResourceSlotStateV1::Retired { .. } => Err(ResourceErrorV1::MalformedResource),
        }
    }

    fn lookup(&self, token: ResourceTokenV1) -> Result<&ResourceSlotV1, ResourceErrorV1> {
        if token.generation == 0 {
            return Err(ResourceErrorV1::MalformedResource);
        }
        let Some(slot) = self.slots.get(token.slot as usize) else {
            return Err(ResourceErrorV1::MalformedResource);
        };
        if token.generation < slot.generation {
            return Err(ResourceErrorV1::Closed);
        }
        if token.generation > slot.generation {
            return Err(ResourceErrorV1::MalformedResource);
        }
        Ok(slot)
    }

    pub fn begin_release(
        &mut self,
        token: ResourceTokenV1,
    ) -> Result<PendingResourceCloseV1, ResourceErrorV1> {
        if token.generation == 0 {
            return Err(ResourceErrorV1::MalformedResource);
        }
        let Some(slot) = self.slots.get_mut(token.slot as usize) else {
            return Err(ResourceErrorV1::MalformedResource);
        };
        if token.generation < slot.generation {
            return Err(ResourceErrorV1::Closed);
        }
        if token.generation > slot.generation {
            return Err(ResourceErrorV1::MalformedResource);
        }
        match &slot.state {
            ResourceSlotStateV1::Vacant { .. } => return Err(ResourceErrorV1::MalformedResource),
            ResourceSlotStateV1::Retired { .. } => return Err(ResourceErrorV1::Closed),
            ResourceSlotStateV1::Live { .. } => {}
        }
        let state = std::mem::replace(
            &mut slot.state,
            ResourceSlotStateV1::Retired {
                last_identity: None,
            },
        );
        let ResourceSlotStateV1::Live {
            owner,
            kind,
            identity,
            ..
        } = state
        else {
            return Err(ResourceErrorV1::Closed);
        };
        if let ResourceOwnerV1::Buffer(buffer) = &owner {
            self.live_buffer_capacity = self
                .live_buffer_capacity
                .checked_sub(buffer.capacity() as u64)
                .expect("live buffer capacity accounting underflow");
        }
        match slot.generation.checked_add(1) {
            Some(next) => {
                slot.generation = next;
                slot.state = ResourceSlotStateV1::Vacant {
                    last_identity: identity,
                };
            }
            None => {
                slot.state = ResourceSlotStateV1::Retired {
                    last_identity: Some(identity),
                }
            }
        }
        Ok(PendingResourceCloseV1 {
            owner,
            kind,
            identity,
        })
    }

    pub fn finish_release_with(
        pending: PendingResourceCloseV1,
        close: impl FnOnce(crate::ResourceHandleV1) -> Result<(), IoErrorIdentityV1>,
    ) -> Result<ResourceSettlementObservationV1, ResourceErrorV1> {
        let PendingResourceCloseV1 {
            owner,
            kind,
            identity,
        } = pending;
        let closed = match owner {
            ResourceOwnerV1::FsHandle(handle) => close(handle),
            ResourceOwnerV1::Buffer(_) => Ok(()),
        };
        match closed {
            Ok(()) => Ok(ResourceSettlementObservationV1 {
                schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                resource_kind: kind,
                identity,
                outcome: ResourceSettlementOutcomeV1::Released,
            }),
            Err(io) => Err(ResourceErrorV1::ReleaseFailed {
                schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                resource_kind: kind,
                identity,
                io,
            }),
        }
    }

    pub fn finalize_all_with(
        &mut self,
        mut close: impl FnMut(crate::ResourceHandleV1) -> Result<(), IoErrorIdentityV1>,
    ) -> Vec<ResourceSettlementObservationV1> {
        let tokens = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, entry)| {
                matches!(entry.state, ResourceSlotStateV1::Live { .. }).then_some(ResourceTokenV1 {
                    slot: slot as u32,
                    generation: entry.generation,
                })
            })
            .collect::<Vec<_>>();
        tokens
            .into_iter()
            .filter_map(|token| {
                let pending = self.begin_release(token).ok()?;
                let kind = pending.kind;
                let identity = pending.identity;
                let closed = match pending.owner {
                    ResourceOwnerV1::FsHandle(handle) => close(handle),
                    ResourceOwnerV1::Buffer(_) => Ok(()),
                };
                Some(match closed {
                    Ok(()) => ResourceSettlementObservationV1 {
                        schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                        resource_kind: kind,
                        identity,
                        outcome: ResourceSettlementOutcomeV1::Released,
                    },
                    Err(io) => ResourceSettlementObservationV1 {
                        schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                        resource_kind: kind,
                        identity,
                        outcome: ResourceSettlementOutcomeV1::ReleaseFailed(io),
                    },
                })
            })
            .collect()
    }

    fn two_live_slots_mut(
        &mut self,
        first: ResourceTokenV1,
        second: ResourceTokenV1,
    ) -> Result<(&mut ResourceSlotV1, &mut ResourceSlotV1), ResourceErrorV1> {
        self.lookup(first)?;
        self.lookup(second)?;
        if first.slot == second.slot {
            let slot = &self.slots[first.slot as usize];
            let ResourceSlotStateV1::Live { kind: actual, .. } = &slot.state else {
                return Err(ResourceErrorV1::Closed);
            };
            return Err(ResourceErrorV1::ResourceKindMismatch {
                expected: if *actual == ResourceKindV1::FsHandle {
                    ResourceKindV1::Buffer
                } else {
                    ResourceKindV1::FsHandle
                },
                actual: *actual,
            });
        }
        let (low, high, swapped) = if first.slot < second.slot {
            (first, second, false)
        } else {
            (second, first, true)
        };
        let (left, right) = self.slots.split_at_mut(high.slot as usize);
        let low_slot = &mut left[low.slot as usize];
        let high_slot = &mut right[0];
        if swapped {
            Ok((high_slot, low_slot))
        } else {
            Ok((low_slot, high_slot))
        }
    }

    pub fn with_fs_and_buffer_mut<R>(
        &mut self,
        file: ResourceTokenV1,
        file_right: crate::RightSet,
        buffer: ResourceTokenV1,
        f: impl FnOnce(
            &crate::ResourceHandleV1,
            ResourceTraceIdentityV1,
            &mut BufferRegionV1,
            ResourceTraceIdentityV1,
        ) -> Result<R, SemanticErrorV1>,
    ) -> Result<R, SemanticErrorV1> {
        let (file_slot, buffer_slot) = self
            .two_live_slots_mut(file, buffer)
            .map_err(SemanticErrorV1::Resource)?;
        let ResourceSlotStateV1::Live {
            owner: file_owner,
            kind: file_kind,
            rights,
            identity: file_identity,
        } = &mut file_slot.state
        else {
            return Err(SemanticErrorV1::Resource(ResourceErrorV1::Closed));
        };
        if *file_kind != ResourceKindV1::FsHandle {
            return Err(SemanticErrorV1::Resource(
                ResourceErrorV1::ResourceKindMismatch {
                    expected: ResourceKindV1::FsHandle,
                    actual: *file_kind,
                },
            ));
        }
        if !rights.contains(file_right) {
            return Err(SemanticErrorV1::Resource(ResourceErrorV1::RightNotHeld {
                required: file_right.bits(),
                held: rights.bits(),
            }));
        }
        let ResourceOwnerV1::FsHandle(file_owner) = file_owner else {
            return Err(SemanticErrorV1::Resource(
                ResourceErrorV1::MalformedResource,
            ));
        };
        let ResourceSlotStateV1::Live {
            owner: buffer_owner,
            kind: buffer_kind,
            identity: buffer_identity,
            ..
        } = &mut buffer_slot.state
        else {
            return Err(SemanticErrorV1::Resource(ResourceErrorV1::Closed));
        };
        if *buffer_kind != ResourceKindV1::Buffer {
            return Err(SemanticErrorV1::Resource(
                ResourceErrorV1::ResourceKindMismatch {
                    expected: ResourceKindV1::Buffer,
                    actual: *buffer_kind,
                },
            ));
        }
        let ResourceOwnerV1::Buffer(buffer_owner) = buffer_owner else {
            return Err(SemanticErrorV1::Resource(
                ResourceErrorV1::MalformedResource,
            ));
        };
        f(file_owner, *file_identity, buffer_owner, *buffer_identity)
    }

    #[cfg(test)]
    fn force_generation_for_test(
        &mut self,
        token: ResourceTokenV1,
        generation: u32,
    ) -> ResourceTokenV1 {
        let slot = &mut self.slots[token.slot as usize];
        slot.generation = generation;
        ResourceTokenV1 {
            slot: token.slot,
            generation,
        }
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

    fn fs_open_resource(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
        _mode: FsOpenModeV1,
    ) -> Result<crate::ResourceHandleV1, FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }

    fn fs_resource_metadata(
        &mut self,
        handle: &crate::ResourceHandleV1,
    ) -> Result<FileMetadataV1, IoErrorIdentityV1> {
        crate::resource_metadata_v1(handle)
            .map(|metadata| FileMetadataV1 {
                size: metadata.size,
                kind: match metadata.kind {
                    crate::FileKind::File => FsNodeKindV1::File,
                    crate::FileKind::Directory => FsNodeKindV1::Directory,
                    crate::FileKind::Symlink => FsNodeKindV1::Symlink,
                    crate::FileKind::Other => FsNodeKindV1::Other,
                },
            })
            .map_err(|error| io_error_identity_v1(&error.into_io_error()))
    }

    fn fs_resource_read_at(
        &mut self,
        handle: &crate::ResourceHandleV1,
        offset: u64,
        bytes: &mut [u8],
    ) -> Result<usize, IoErrorIdentityV1> {
        crate::resource_read_at_v1(handle, offset, bytes)
            .map_err(|error| io_error_identity_v1(&error.into_io_error()))
    }

    fn fs_resource_write_at(
        &mut self,
        handle: &crate::ResourceHandleV1,
        offset: u64,
        bytes: &[u8],
    ) -> Result<usize, IoErrorIdentityV1> {
        crate::resource_write_at_v1(handle, offset, bytes)
            .map_err(|error| io_error_identity_v1(&error.into_io_error()))
    }

    fn resource_close(&mut self, handle: crate::ResourceHandleV1) -> Result<(), IoErrorIdentityV1> {
        crate::close_resource_v1(handle)
            .map_err(|error| io_error_identity_v1(&error.into_io_error()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDispatchReplyV1 {
    pub capability_identity: Option<CapabilityTraceIdentity>,
    pub resource_token: Option<ResourceTokenV1>,
    pub resource_bindings: Vec<(ResourceBindingRole, ResourceTraceIdentityV1)>,
    pub outcome: CanonicalOutcomeV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceInputsV1 {
    None,
    Target(ResourceTokenV1),
    FileBuffer {
        file: ResourceTokenV1,
        buffer: ResourceTokenV1,
    },
    /// `BufferFreeze`/`spanBytes` consumer: the target buffer plus the exact
    /// buffer acquisition that minted the supplied span (PX8-SPAN-PROV). The
    /// dispatcher admits only when `span_origin == target` on the full opaque
    /// token, before any byte exposure.
    BufferSpanTarget {
        target: ResourceTokenV1,
        span_origin: ResourceTokenV1,
    },
    /// `FsWriteAt` consumer: the file, the target buffer, and the span's
    /// originating buffer acquisition (PX8-SPAN-PROV). The dispatcher admits
    /// only when `span_origin == target_buffer`, after host-width admission and
    /// before any backend write.
    FileBufferSpan {
        file: ResourceTokenV1,
        target_buffer: ResourceTokenV1,
        span_origin: ResourceTokenV1,
    },
}

/// The only V1 semantic operation switch. Validation and capability denial
/// happen before a backend leaf is invoked.
pub fn dispatch_host_op_v1<B: HostEffectBackendV1>(
    backend: &mut B,
    capabilities: &CapabilityTableV1,
    resources: &mut ResourceTableV1,
    operation: HostOpV1,
    capability: Option<CapabilityTokenV1>,
    resource: ResourceInputsV1,
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
        HostOpV1::FsOpen => {
            let CanonicalRequestV1::FsOpen { mode, .. } = request else {
                return Err(TerminalErrorV1::MalformedHostAbiField);
            };
            Some((
                mode.capability_operation(),
                if matches!(mode, FsOpenModeV1::WriteCreate(_)) {
                    crate::AUTH_FULL
                } else {
                    crate::AUTH_PARTIAL
                },
            ))
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
    let resource_shape_matches = matches!(
        (operation, resource),
        (
            HostOpV1::FsHandleMetadata | HostOpV1::ResourceRelease,
            ResourceInputsV1::Target(_)
        ) | (
            HostOpV1::BufferFreeze,
            ResourceInputsV1::BufferSpanTarget { .. }
        ) | (
            HostOpV1::FsReadAt,
            ResourceInputsV1::FileBuffer { .. }
        ) | (
            HostOpV1::FsWriteAt,
            ResourceInputsV1::FileBufferSpan { .. }
        ) | (
            HostOpV1::BufferAllocate
                | HostOpV1::ConsoleRead
                | HostOpV1::ConsoleWrite
                | HostOpV1::ConsoleFlush
                | HostOpV1::ConsoleIsTerminal
                | HostOpV1::ClockWallNow
                | HostOpV1::FsReadFile
                | HostOpV1::FsWriteFile
                | HostOpV1::FsAppendFile
                | HostOpV1::FsMetadata
                | HostOpV1::FsReadDirectory
                | HostOpV1::FsCreateDirectory
                | HostOpV1::FsRemoveFile
                | HostOpV1::FsRemoveDirectory
                | HostOpV1::FsRename
                | HostOpV1::FsChangeMode
                | HostOpV1::FsOpen,
            ResourceInputsV1::None
        )
    );
    if !resource_shape_matches {
        return Ok(resource_denied(
            operation,
            request,
            ResourceErrorV1::MalformedResource,
        ));
    }
    let mut minted_resource = None;
    let mut resource_bindings = Vec::new();
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
        (HostOpV1::FsOpen, CanonicalRequestV1::FsOpen { path, mode }) => backend
            .fs_open_resource(grant.expect("validated FS capability"), path, *mode)
            .map(|owner| {
                let (token, identity) = resources.insert_fs_handle(owner, mode.required_right());
                minted_resource = Some(token);
                resource_bindings.push((ResourceBindingRole::Target, identity));
                CanonicalReplyV1::ResourceAcquired {
                    schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                    resource_kind: ResourceKindV1::FsHandle,
                    identity,
                }
            })
            .map_err(|cause| file_error(operation, path, cause)),
        (HostOpV1::FsHandleMetadata, CanonicalRequestV1::FsHandleMetadata) => {
            let ResourceInputsV1::Target(token) = resource else {
                unreachable!("resource shape validated")
            };
            match resources.resolve_fs_handle(token, crate::RightSet::METADATA) {
                Ok((handle, identity)) => {
                    resource_bindings.push((ResourceBindingRole::Target, identity));
                    backend
                        .fs_resource_metadata(handle)
                        .map(CanonicalReplyV1::FileMetadata)
                        .map_err(SemanticErrorV1::Io)
                }
                Err(error) => Err(SemanticErrorV1::Resource(error)),
            }
        }
        (HostOpV1::BufferAllocate, CanonicalRequestV1::BufferAllocate { capacity }) => {
            match resources.insert_buffer(*capacity) {
                Ok((token, identity)) => {
                    minted_resource = Some(token);
                    resource_bindings.push((ResourceBindingRole::Target, identity));
                    Ok(CanonicalReplyV1::ResourceAcquired {
                        schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                        resource_kind: ResourceKindV1::Buffer,
                        identity,
                    })
                }
                Err(error) => Err(SemanticErrorV1::Resource(error)),
            }
        }
        (HostOpV1::BufferFreeze, CanonicalRequestV1::BufferFreeze { start, length }) => {
            let ResourceInputsV1::BufferSpanTarget {
                target,
                span_origin,
            } = resource
            else {
                unreachable!("resource shape validated")
            };
            match resources.resolve_buffer(target) {
                Ok((buffer, identity)) => {
                    resource_bindings.push((ResourceBindingRole::Target, identity));
                    if span_origin != target {
                        // PX8-SPAN-PROV: the span was minted by a different
                        // buffer acquisition. Reject before exposing any bytes.
                        // Acquisition mismatch shares `InvalidBounds` with
                        // numeric live-window invalidity (`38 §1.7.1`); their
                        // relative order is not publicly observable, so the
                        // check may precede the numeric coordinate check.
                        Err(SemanticErrorV1::Resource(ResourceErrorV1::InvalidBounds))
                    } else {
                        let start =
                            usize::try_from(*start).map_err(|_| ResourceErrorV1::InvalidBounds);
                        let length =
                            usize::try_from(*length).map_err(|_| ResourceErrorV1::InvalidBounds);
                        match start.and_then(|start| length.map(|length| (start, length))) {
                            Ok((start, length)) => buffer
                                .initialized_slice(start, length)
                                .map(|bytes| CanonicalReplyV1::Bytes(bytes.to_vec()))
                                .map_err(SemanticErrorV1::Resource),
                            Err(error) => Err(SemanticErrorV1::Resource(error)),
                        }
                    }
                }
                Err(error) => Err(SemanticErrorV1::Resource(error)),
            }
        }
        (
            HostOpV1::FsReadAt,
            CanonicalRequestV1::FsReadAt {
                file_offset,
                buffer_start,
                length,
            },
        ) => {
            let ResourceInputsV1::FileBuffer { file, buffer } = resource else {
                unreachable!("resource shape validated")
            };
            resources.with_fs_and_buffer_mut(
                file,
                crate::RightSet::READ,
                buffer,
                |handle, file_identity, region, buffer_identity| {
                    resource_bindings.extend([
                        (ResourceBindingRole::File, file_identity),
                        (ResourceBindingRole::Buffer, buffer_identity),
                    ]);
                    let (start, effective) =
                        checked_buffer_range(region.capacity(), *buffer_start, *length)
                            .map_err(SemanticErrorV1::Resource)?;
                    file_offset
                        .checked_add(effective as u64)
                        .ok_or(ResourceErrorV1::InvalidOffset)
                        .map_err(SemanticErrorV1::Resource)?;
                    region.clear_window();
                    let read = backend
                        .fs_resource_read_at(
                            handle,
                            *file_offset,
                            &mut region.bytes[start..start + effective],
                        )
                        .map_err(|error| {
                            region.clear_window();
                            SemanticErrorV1::Io(error)
                        })?;
                    if read > effective {
                        region.clear_window();
                        return Err(SemanticErrorV1::Resource(ResourceErrorV1::InvalidBounds));
                    }
                    if read == 0 {
                        Ok(CanonicalReplyV1::ReadProgress(ReadProgressV1::ReadEof))
                    } else {
                        region.install_window(start, read);
                        Ok(CanonicalReplyV1::ReadProgress(ReadProgressV1::ReadSome {
                            span: BufferSpanV1 {
                                start: start as u64,
                                length: read as u64,
                            },
                            transferred: TransferCountV1::new(read as u64, effective as u64)
                                .expect("positive bounded backend read"),
                        }))
                    }
                },
            )
        }
        (
            HostOpV1::FsWriteAt,
            CanonicalRequestV1::FsWriteAt {
                file_offset,
                buffer_start,
                length,
            },
        ) => {
            let ResourceInputsV1::FileBufferSpan {
                file,
                target_buffer,
                span_origin,
            } = resource
            else {
                unreachable!("resource shape validated")
            };
            resources.with_fs_and_buffer_mut(
                file,
                crate::RightSet::WRITE,
                target_buffer,
                |handle, file_identity, region, buffer_identity| {
                    resource_bindings.extend([
                        (ResourceBindingRole::File, file_identity),
                        (ResourceBindingRole::Buffer, buffer_identity),
                    ]);
                    let (start, effective) =
                        checked_buffer_range(region.capacity(), *buffer_start, *length)
                            .map_err(SemanticErrorV1::Resource)?;
                    file_offset
                        .checked_add(effective as u64)
                        .ok_or(ResourceErrorV1::InvalidOffset)
                        .map_err(SemanticErrorV1::Resource)?;
                    if span_origin != target_buffer {
                        // PX8-SPAN-PROV: foreign-acquisition span. Reject after
                        // existing host-width admission (`InvalidOffset`) and
                        // before exposing bytes or issuing any backend write, so
                        // a mismatch records zero backend calls (`38 §1.7.1`).
                        return Err(SemanticErrorV1::Resource(ResourceErrorV1::InvalidBounds));
                    }
                    let bytes = region
                        .initialized_slice(start, effective)
                        .map_err(SemanticErrorV1::Resource)?;
                    let written = backend
                        .fs_resource_write_at(handle, *file_offset, bytes)
                        .map_err(SemanticErrorV1::Io)?;
                    if written == 0 {
                        return Err(SemanticErrorV1::Resource(ResourceErrorV1::NoProgress));
                    }
                    if written > effective {
                        return Err(SemanticErrorV1::Resource(ResourceErrorV1::InvalidBounds));
                    }
                    Ok(CanonicalReplyV1::WriteProgress(WriteProgressV1::Wrote(
                        TransferCountV1::new(written as u64, effective as u64)
                            .expect("positive bounded backend write"),
                    )))
                },
            )
        }
        (HostOpV1::ResourceRelease, CanonicalRequestV1::ResourceRelease) => {
            let ResourceInputsV1::Target(token) = resource else {
                unreachable!("resource shape validated")
            };
            match resources.begin_release(token) {
                Ok(pending) => {
                    resource_bindings.push((ResourceBindingRole::Target, pending.identity));
                    ResourceTableV1::finish_release_with(pending, |owner| {
                        backend.resource_close(owner)
                    })
                    .map(CanonicalReplyV1::ResourceSettlement)
                    .map_err(SemanticErrorV1::Resource)
                }
                Err(error) => Err(SemanticErrorV1::Resource(error)),
            }
        }
        _ => return Err(TerminalErrorV1::MalformedHostAbiField),
    };
    Ok(HostDispatchReplyV1 {
        capability_identity: grant.map(|grant| grant.identity.clone()),
        resource_token: minted_resource,
        resource_bindings,
        outcome: match outcome {
            Ok(reply) => CanonicalOutcomeV1::Success(reply),
            Err(error) => CanonicalOutcomeV1::Error(error),
        },
    })
}

fn checked_buffer_range(
    capacity: usize,
    start: u64,
    length: u64,
) -> Result<(usize, usize), ResourceErrorV1> {
    let start = usize::try_from(start).map_err(|_| ResourceErrorV1::InvalidBounds)?;
    if start > capacity {
        return Err(ResourceErrorV1::InvalidBounds);
    }
    let requested = usize::try_from(length).unwrap_or(usize::MAX);
    let effective = requested.min(capacity - start);
    if effective == 0 {
        return Err(ResourceErrorV1::InvalidBounds);
    }
    Ok((start, effective))
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
        CanonicalRequestV1::FsChangeMode { path, .. } | CanonicalRequestV1::FsOpen { path, .. } => {
            path.clone()
        }
        _ => Vec::new(),
    };
    HostDispatchReplyV1 {
        capability_identity: None,
        resource_token: None,
        resource_bindings: Vec::new(),
        outcome: CanonicalOutcomeV1::Error(file_error(
            operation,
            &path,
            FileErrorCauseV1::Capability(error),
        )),
    }
}

fn resource_denied(
    _operation: HostOpV1,
    _request: &CanonicalRequestV1,
    error: ResourceErrorV1,
) -> HostDispatchReplyV1 {
    HostDispatchReplyV1 {
        capability_identity: None,
        resource_token: None,
        resource_bindings: Vec::new(),
        outcome: CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(error)),
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
    FsOpen {
        path: Vec<u8>,
        mode: FsOpenModeV1,
    },
    FsHandleMetadata,
    FsReadAt {
        file_offset: u64,
        buffer_start: u64,
        length: u64,
    },
    FsWriteAt {
        file_offset: u64,
        buffer_start: u64,
        length: u64,
    },
    BufferAllocate {
        capacity: u64,
    },
    BufferFreeze {
        start: u64,
        length: u64,
    },
    ResourceRelease,
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
    Resource(ResourceErrorV1),
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
    ResourceAcquired {
        schema_version: u16,
        resource_kind: ResourceKindV1,
        identity: ResourceTraceIdentityV1,
    },
    ResourceSettlement(ResourceSettlementObservationV1),
    ReadProgress(ReadProgressV1),
    WriteProgress(WriteProgressV1),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferSpanV1 {
    pub(crate) start: u64,
    pub(crate) length: u64,
}

impl BufferSpanV1 {
    pub fn start(self) -> u64 {
        self.start
    }

    pub fn length(self) -> u64 {
        self.length
    }
}

/// Inseparable validated carrier (Architect ruling `dec_1m6xdwjp2ttyn`,
/// Option 1): the effective (post-clamp) request travels inside the count
/// itself, never as a loose sibling value that can drift from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferCountV1 {
    transferred: u64,
    effective_request: u64,
}

impl TransferCountV1 {
    pub(crate) fn new(transferred: u64, effective_request: u64) -> Option<Self> {
        (transferred > 0 && transferred <= effective_request).then_some(Self {
            transferred,
            effective_request,
        })
    }

    pub fn get(self) -> u64 {
        self.transferred
    }

    /// The post-clamp request this count was minted against. `remaining`
    /// (both reifiers) is `effective_request - get()` — never derived from
    /// the raw pre-clamp request length.
    pub fn effective_request(self) -> u64 {
        self.effective_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadProgressV1 {
    ReadSome {
        span: BufferSpanV1,
        transferred: TransferCountV1,
    },
    ReadEof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteProgressV1 {
    Wrote(TransferCountV1),
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

/// Sole population classifier for BUDGET-EXHAUST (`dec_7kcbc14ybndbq`): the
/// set of outcome shapes that carry a `TransferCountV1` budget to bound. No
/// arm at any level (`CanonicalOutcomeV1`, `CanonicalReplyV1`,
/// `ReadProgressV1`, `WriteProgressV1`, `SemanticErrorV1`) is a `_`
/// catch-all, so a future budget-carrying reply variant is a compile error
/// here rather than an absence a reviewer must notice. Population is one
/// fact; `effect_wire`'s and `ken-interp`'s validators stay independent
/// consumers of it (wire admission and interpreter reification are two
/// boundaries).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferRequestBoundV1 {
    ReadAt(TransferCountV1),
    WriteAt(TransferCountV1),
}

impl CanonicalOutcomeV1 {
    pub fn transfer_request_bound(&self) -> Option<TransferRequestBoundV1> {
        match self {
            CanonicalOutcomeV1::Success(reply) => match reply {
                CanonicalReplyV1::Unit
                | CanonicalReplyV1::Bool(_)
                | CanonicalReplyV1::Bytes(_)
                | CanonicalReplyV1::ReadChunk(_)
                | CanonicalReplyV1::ReadEof
                | CanonicalReplyV1::Instant(_)
                | CanonicalReplyV1::FileMetadata(_)
                | CanonicalReplyV1::DirectoryEntries(_)
                | CanonicalReplyV1::ResourceAcquired { .. }
                | CanonicalReplyV1::ResourceSettlement(_) => None,
                CanonicalReplyV1::ReadProgress(progress) => match progress {
                    ReadProgressV1::ReadEof => None,
                    ReadProgressV1::ReadSome { transferred, .. } => {
                        Some(TransferRequestBoundV1::ReadAt(*transferred))
                    }
                },
                CanonicalReplyV1::WriteProgress(progress) => match progress {
                    WriteProgressV1::Wrote(transferred) => {
                        Some(TransferRequestBoundV1::WriteAt(*transferred))
                    }
                },
            },
            CanonicalOutcomeV1::Error(error) => match error {
                SemanticErrorV1::Io(_)
                | SemanticErrorV1::File(_)
                | SemanticErrorV1::Capability(_)
                | SemanticErrorV1::Resource(_) => None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceBindingRole {
    File,
    Buffer,
    Target,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectEvent {
    pub sequence: u64,
    pub operation: HostOpV1,
    pub capability: Option<CapabilityTraceIdentity>,
    pub resource_bindings: Vec<(ResourceBindingRole, ResourceTraceIdentityV1)>,
    pub request: CanonicalRequestV1,
    pub outcome: CanonicalOutcomeV1,
}

/// Construct the canonical runtime event after semantic dispatch has
/// produced its reply and before an executor reifies that reply.
pub fn effect_event_from_dispatch(
    sequence: u64,
    operation: HostOpV1,
    request: CanonicalRequestV1,
    reply: &HostDispatchReplyV1,
) -> EffectEvent {
    EffectEvent {
        sequence,
        operation,
        capability: reply.capability_identity.clone(),
        resource_bindings: reply.resource_bindings.clone(),
        request,
        outcome: reply.outcome.clone(),
    }
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

/// Observed process termination, computed before exit-code normalization.
///
/// This is an exported observation, never a Ward verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TerminalExitClass {
    NormalReturn,
    ReturnedError,
    ControlledTrap,
}

pub fn terminal_exit_class(
    terminal_value: i64,
    terminal_error: Option<&TerminalErrorV1>,
) -> TerminalExitClass {
    if terminal_error.is_some() || terminal_value < 0 {
        TerminalExitClass::ControlledTrap
    } else if terminal_value == 0 {
        TerminalExitClass::NormalReturn
    } else {
        TerminalExitClass::ReturnedError
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectObservation {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub filesystem_delta: Vec<FsDeltaV1>,
    pub terminal_error: Option<TerminalErrorV1>,
    pub effect_trace: Vec<EffectEvent>,
    pub terminal_exit: TerminalExitClass,
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
    fn all_pre_resource_operations_share_one_semantic_dispatch() {
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
        let mut resources = ResourceTableV1::default();
        for (operation, request) in requests {
            let capability = (!operation.is_ambient()).then_some(token);
            dispatch_host_op_v1(
                &mut backend,
                &capabilities,
                &mut resources,
                operation,
                capability,
                ResourceInputsV1::None,
                &request,
            )
            .unwrap();
        }
        assert_eq!(backend.0, HostOpV1::ALL[..15]);
    }

    #[test]
    fn catalog_is_closed_and_availability_is_exact() {
        assert_eq!(HostOpV1::ALL.len(), 22);
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
            "FsOpen|030b|native|FsOpenRequestV1|3|HostReplyV1|1",
            "FsHandleMetadata|030c|native|ResourceRequestV1|1|HostReplyV1|1",
            "FsReadAt|030d|native|FsPositionedRequestV1|5|HostReplyV1|1",
            "FsWriteAt|030e|native|FsWriteAtRequestV1|6|HostReplyV1|1",
            "ResourceRelease|0401|native|ResourceRequestV1|1|HostReplyV1|1",
            "BufferAllocate|0402|native|BufferAllocateRequestV1|1|HostReplyV1|1",
            "BufferFreeze|0403|native|BufferFreezeRequestV1|4|HostReplyV1|1",
            "lifetime=filesystem_observation_schema|2",
            "lifetime=resource_observation_schema|1",
            "lifetime=resource_error_reply_schema|1",
            "limit=buffer.per_buffer_max_capacity|1048576",
            "limit=buffer.invocation_max_live_capacity|16777216",
            "layout=OFFSET_FsChangeModeRequestV1_mode|24",
            "layout=OFFSET_ResourceRequestV1_resource|0",
            "layout=SIZE_HostReplyV1|",
            "layout=SIZE_ResourceErrorReplyV1|64",
            "layout=OFFSET_HostReplyV1_resource_error|32",
            "layout=OFFSET_HostReplyV1_effective_request|96",
            "error=io.BrokenPipe|3",
            "error=resource.ReleaseFailed|3",
            "error=resource.ResourceKindMismatch|4",
            "tag=reply.error|3",
            "tag=reply.resource_error|6",
            "tag=resource_kind.FsHandle|0",
            "tag=resource_kind.Buffer|1",
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
        assert_eq!(
            fact("SIZE_ResourceTokenV1"),
            std::mem::size_of::<ResourceTokenV1>() as u64
        );
        assert_eq!(
            fact("ALIGN_ResourceTokenV1"),
            std::mem::align_of::<ResourceTokenV1>() as u64
        );
        assert_eq!(
            fact("OFFSET_ResourceTokenV1_slot"),
            std::mem::offset_of!(ResourceTokenV1, slot) as u64
        );
        assert_eq!(
            fact("OFFSET_ResourceTokenV1_generation"),
            std::mem::offset_of!(ResourceTokenV1, generation) as u64
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

    #[cfg(target_os = "linux")]
    fn resource_fixture(name: &str) -> (std::path::PathBuf, crate::ResourceHandleV1) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let root = std::env::temp_dir().join(format!(
            "ken-px7r-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            name
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(name), b"resource-bytes").unwrap();
        let root_path = crate::RootPath::new(&root).unwrap();
        let parent = crate::open_root(&root_path).unwrap();
        let leaf = crate::PathComponent::new(name.as_bytes()).unwrap();
        let handle = crate::open_resource_at_v1(&parent, &leaf, crate::OpenRequest::Read).unwrap();
        (root, handle)
    }

    #[cfg(target_os = "linux")]
    #[test]
    /// Caller-control only: the injected error proves invalidation, mapping,
    /// and no retry; it is not evidence that the OS produced a close error.
    fn caller_control_release_invalidates_before_close_and_never_retries() {
        let (root, owner) = resource_fixture("first");
        let mut table = ResourceTableV1::default();
        let (token, identity) = table.insert_fs_handle(owner, crate::RightSet::METADATA);
        assert_eq!(identity, ResourceTraceIdentityV1(1));
        assert!(table
            .resolve_fs_handle(token, crate::RightSet::METADATA)
            .is_ok());

        let pending = table.begin_release(token).unwrap();
        assert!(matches!(
            table.resolve_fs_handle(token, crate::RightSet::METADATA),
            Err(ResourceErrorV1::Closed)
        ));
        let calls = std::cell::Cell::new(0);
        let result = ResourceTableV1::finish_release_with(pending, |owner| {
            calls.set(calls.get() + 1);
            drop(owner);
            Err(IoErrorIdentityV1::Other(5))
        });
        assert_eq!(calls.get(), 1);
        assert_eq!(
            result,
            Err(ResourceErrorV1::ReleaseFailed {
                schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                resource_kind: ResourceKindV1::FsHandle,
                identity,
                io: IoErrorIdentityV1::Other(5),
            })
        );
        assert!(matches!(
            table.begin_release(token),
            Err(ResourceErrorV1::Closed)
        ));
        assert_eq!(calls.get(), 1, "closed descriptors are never retried");
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Caller-control only: this proves explicit finalization preserves every
    /// close result after invalidation; it does not claim the OS produced the
    /// injected error.
    #[cfg(target_os = "linux")]
    #[test]
    fn explicit_finalizer_settles_every_live_owner_once_and_records_failure() {
        let (root_a, owner_a) = resource_fixture("final-a");
        let (root_b, owner_b) = resource_fixture("final-b");
        let mut table = ResourceTableV1::default();
        let (token_a, identity_a) = table.insert_fs_handle(owner_a, crate::RightSet::METADATA);
        let (token_b, identity_b) = table.insert_fs_handle(owner_b, crate::RightSet::METADATA);
        let calls = std::cell::Cell::new(0);
        let settlements = table.finalize_all_with(|owner| {
            let call = calls.get();
            calls.set(call + 1);
            drop(owner);
            if call == 0 {
                Ok(())
            } else {
                Err(IoErrorIdentityV1::Other(5))
            }
        });
        assert_eq!(calls.get(), 2);
        assert_eq!(
            settlements,
            vec![
                ResourceSettlementObservationV1 {
                    schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                    resource_kind: ResourceKindV1::FsHandle,
                    identity: identity_a,
                    outcome: ResourceSettlementOutcomeV1::Released,
                },
                ResourceSettlementObservationV1 {
                    schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                    resource_kind: ResourceKindV1::FsHandle,
                    identity: identity_b,
                    outcome: ResourceSettlementOutcomeV1::ReleaseFailed(IoErrorIdentityV1::Other(
                        5
                    ),),
                },
            ]
        );
        for token in [token_a, token_b] {
            assert!(matches!(
                table.resolve_fs_handle(token, crate::RightSet::METADATA),
                Err(ResourceErrorV1::Closed)
            ));
        }
        assert!(table.finalize_all_with(|_| unreachable!()).is_empty());
        assert_eq!(calls.get(), 2, "explicit finalization never retries");
        for root in [root_a, root_b] {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn stale_slot_reuse_wrap_retirement_and_real_consumer_are_distinct() {
        let (root_a, owner_a) = resource_fixture("a");
        let (root_b, owner_b) = resource_fixture("b");
        let (root_c, owner_c) = resource_fixture("c");
        let mut table = ResourceTableV1::default();
        let (stale, _) = table.insert_fs_handle(owner_a, crate::RightSet::METADATA);
        let metadata = crate::resource_metadata_v1(
            table
                .resolve_fs_handle(stale, crate::RightSet::METADATA)
                .unwrap()
                .0,
        )
        .unwrap();
        assert_eq!(metadata.size, 14);
        let pending = table.begin_release(stale).unwrap();
        ResourceTableV1::finish_release_with(pending, |owner| {
            crate::close_resource_v1(owner)
                .map_err(|error| io_error_identity_v1(&error.into_io_error()))
        })
        .unwrap();
        assert!(matches!(
            table.resolve_fs_handle(stale, crate::RightSet::METADATA),
            Err(ResourceErrorV1::Closed)
        ));

        let never_minted = ResourceTokenV1 {
            slot: stale.slot,
            generation: stale.generation + 1,
        };
        assert_eq!(
            table.identity(never_minted),
            Err(ResourceErrorV1::MalformedResource)
        );
        assert!(matches!(
            table.resolve_fs_handle(never_minted, crate::RightSet::METADATA),
            Err(ResourceErrorV1::MalformedResource)
        ));
        assert!(matches!(
            table.begin_release(never_minted),
            Err(ResourceErrorV1::MalformedResource)
        ));

        let (reused, second_identity) = table.insert_fs_handle(owner_b, crate::RightSet::METADATA);
        assert_eq!(reused.slot, stale.slot);
        assert_ne!(reused.generation, stale.generation);
        assert_eq!(second_identity, ResourceTraceIdentityV1(2));
        assert!(matches!(
            table.resolve_fs_handle(reused, crate::RightSet::READ),
            Err(ResourceErrorV1::RightNotHeld { .. })
        ));

        let wrapped = table.force_generation_for_test(reused, u32::MAX);
        let pending = table.begin_release(wrapped).unwrap();
        ResourceTableV1::finish_release_with(pending, |owner| {
            crate::close_resource_v1(owner)
                .map_err(|error| io_error_identity_v1(&error.into_io_error()))
        })
        .unwrap();
        let (after_wrap, third_identity) =
            table.insert_fs_handle(owner_c, crate::RightSet::METADATA);
        assert_ne!(
            after_wrap.slot, wrapped.slot,
            "wrapped slots retire permanently"
        );
        assert_eq!(third_identity, ResourceTraceIdentityV1(3));
        assert_eq!(
            table.identity(ResourceTokenV1 {
                slot: u32::MAX,
                generation: 1,
            }),
            Err(ResourceErrorV1::MalformedResource)
        );
        assert_eq!(
            table.identity(ResourceTokenV1 {
                slot: after_wrap.slot,
                generation: 0,
            }),
            Err(ResourceErrorV1::MalformedResource)
        );
        let pending = table.begin_release(after_wrap).unwrap();
        ResourceTableV1::finish_release_with(pending, |owner| {
            crate::close_resource_v1(owner)
                .map_err(|error| io_error_identity_v1(&error.into_io_error()))
        })
        .unwrap();
        for root in [root_a, root_b, root_c] {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn resource_owner_and_close_allowance_are_structurally_confined() {
        // Structural source-text proxy for two Rust-type-system-level shape
        // properties with no cheaper elaborated/runtime check available in
        // stable Rust short of a compile-fail (trybuild) harness: no shared-
        // ownership escape hatch on `ResourceHandleV1`, and a single confined
        // unsafe raw-fd close site. A semantically-preserving rename/refactor
        // could in principle evade these substring matches; verified
        // (Q-RESIDUE, 2026-07-21) that flipping each guarded shape in place
        // makes the corresponding assertion fail, so the scan does
        // discriminate the violations it names.
        //
        // The third property this test used to check here -- that resource
        // settlement is recorded before the observation is written -- is now
        // proven behaviorally, through the real unsafe entrypoint, by
        // `abi_v1.rs`'s
        // `resource_settlement_is_recorded_before_observation_is_written`,
        // which decodes the actual emitted trace instead of comparing
        // source-text byte offsets of the two call sites.
        let lib = include_str!("lib.rs");
        let close = include_str!("resource_close_v1.rs");
        assert!(lib.contains("pub struct ResourceHandleV1"));
        assert!(!lib.contains("impl Clone for ResourceHandleV1"));
        assert!(!lib.contains("ResourceHandleV1 {\n    inner: Arc"));
        assert_eq!(
            close
                .matches("unsafe { rustix::io::try_close(raw_fd) }")
                .count(),
            1
        );
        assert_eq!(close.matches("#![allow(unsafe_code)]").count(), 1);
        assert!(!close.contains("ManuallyDrop"));
        assert!(!close.contains("from_raw_fd"));
    }

    #[cfg(target_os = "linux")]
    struct RealResourceBackend {
        root: crate::RootedHandle,
    }

    #[cfg(target_os = "linux")]
    impl HostEffectBackendV1 for RealResourceBackend {
        fn console_write(&mut self, _: ConsoleStreamV1, _: &[u8]) -> Result<(), IoErrorIdentityV1> {
            unreachable!()
        }

        fn console_flush(&mut self, _: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1> {
            unreachable!()
        }

        fn console_is_terminal(&mut self, _: ConsoleStreamV1) -> bool {
            unreachable!()
        }

        fn fs_read_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
        ) -> Result<Vec<u8>, FileErrorCauseV1> {
            unreachable!()
        }

        fn fs_write_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: CreatePolicyV1,
            _: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            unreachable!()
        }

        fn fs_open_resource(
            &mut self,
            _: &CapabilityGrantV1,
            path: &[u8],
            _: FsOpenModeV1,
        ) -> Result<crate::ResourceHandleV1, FileErrorCauseV1> {
            crate::open_resource_at_v1(
                &self.root,
                &crate::PathComponent::new(path).map_err(|error| {
                    FileErrorCauseV1::Io(io_error_identity_v1(&error.into_io_error()))
                })?,
                crate::OpenRequest::Read,
            )
            .map_err(|error| FileErrorCauseV1::Io(io_error_identity_v1(&error.into_io_error())))
        }
    }

    #[cfg(target_os = "linux")]
    fn resource_lane(root: &std::path::Path) -> Vec<EffectEvent> {
        let root_path = crate::RootPath::new(root).unwrap();
        let root_handle = crate::open_root(&root_path).unwrap();
        let metadata = crate::metadata(&root_handle).unwrap();
        let cap = crate::Cap::mint_scoped(
            crate::AUTH_PARTIAL,
            "FS",
            crate::FsScope::root(
                crate::RightSet::METADATA,
                crate::FsHandle::Posix(root_handle.clone()),
                crate::FsIdentity::Posix {
                    device: metadata.identity.device,
                    inode: metadata.identity.inode,
                },
                crate::SymlinkPolicy::NoFollow,
            ),
        );
        let mut capabilities = CapabilityTableV1::default();
        let capability = capabilities.insert(CapabilityGrantV1 {
            identity: crate::program_caps_fs_trace_identity_v1(),
            capability: cap,
        });
        let mut resources = ResourceTableV1::default();
        let mut backend = RealResourceBackend { root: root_handle };
        let mut events = Vec::new();
        let open_request = CanonicalRequestV1::FsOpen {
            path: b"held.bin".to_vec(),
            mode: FsOpenModeV1::Metadata,
        };
        let open = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &open_request,
        )
        .unwrap();
        let token = open.resource_token.unwrap();
        events.push(effect_event_from_dispatch(
            0,
            HostOpV1::FsOpen,
            open_request,
            &open,
        ));
        for (sequence, operation, request) in [
            (
                1,
                HostOpV1::FsHandleMetadata,
                CanonicalRequestV1::FsHandleMetadata,
            ),
            (
                2,
                HostOpV1::ResourceRelease,
                CanonicalRequestV1::ResourceRelease,
            ),
        ] {
            let reply = dispatch_host_op_v1(
                &mut backend,
                &capabilities,
                &mut resources,
                operation,
                None,
                ResourceInputsV1::Target(token),
                &request,
            )
            .unwrap();
            events.push(effect_event_from_dispatch(
                sequence, operation, request, &reply,
            ));
        }
        events
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn distinct_host_roots_have_byte_identical_resource_lifecycle_observations() {
        let left = std::env::temp_dir().join(format!("ken-px7r-left-{}", std::process::id()));
        let right = std::env::temp_dir().join(format!("ken-px7r-right-{}", std::process::id()));
        for root in [&left, &right] {
            let _ = std::fs::remove_dir_all(root);
            std::fs::create_dir_all(root).unwrap();
            std::fs::write(root.join("held.bin"), b"same-bytes").unwrap();
        }
        let left_events = resource_lane(&left);
        let right_events = resource_lane(&right);
        assert_eq!(left_events, right_events);
        assert_eq!(
            left_events
                .iter()
                .map(|event| event.resource_bindings.clone())
                .collect::<Vec<_>>(),
            vec![vec![(ResourceBindingRole::Target, ResourceTraceIdentityV1(1))]; 3]
        );
        std::fs::remove_dir_all(left).unwrap();
        std::fs::remove_dir_all(right).unwrap();
    }

    #[test]
    fn fs_open_denial_remains_a_capability_error_before_resource_mint() {
        let mut capabilities = CapabilityTableV1::default();
        let capability = capabilities.insert(CapabilityGrantV1 {
            identity: crate::program_caps_fs_trace_identity_v1(),
            capability: crate::Cap::mint(crate::AUTH_NONE, "FS"),
        });
        let mut resources = ResourceTableV1::default();
        let mut backend = AllOpsBackend::default();
        let request = CanonicalRequestV1::FsOpen {
            path: b"denied".to_vec(),
            mode: FsOpenModeV1::Metadata,
        };
        let reply = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &request,
        )
        .unwrap();
        assert!(matches!(
            reply.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::File(FileErrorIdentityV1 {
                cause: FileErrorCauseV1::Capability(_),
                ..
            }))
        ));
        assert!(reply.resource_token.is_none());
        assert!(backend.0.is_empty(), "denial precedes every host leaf");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn fs_open_attenuates_the_live_handle_to_the_requested_mode() {
        let root =
            std::env::temp_dir().join(format!("ken-px7r-attenuation-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("held.bin"), b"held-resource").unwrap();
        let rooted = crate::open_root(&crate::RootPath::new(&root).unwrap()).unwrap();
        let metadata = crate::metadata(&rooted).unwrap();
        let cap = crate::Cap::mint_scoped(
            crate::AUTH_PARTIAL,
            "FS",
            crate::FsScope::root(
                crate::RightSet::READ.union(crate::RightSet::METADATA),
                crate::FsHandle::Posix(rooted.clone()),
                crate::FsIdentity::Posix {
                    device: metadata.identity.device,
                    inode: metadata.identity.inode,
                },
                crate::SymlinkPolicy::NoFollow,
            ),
        );
        let mut capabilities = CapabilityTableV1::default();
        let capability = capabilities.insert(CapabilityGrantV1 {
            identity: crate::program_caps_fs_trace_identity_v1(),
            capability: cap,
        });
        let mut resources = ResourceTableV1::default();
        let mut backend = RealResourceBackend { root: rooted };
        let opened = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &CanonicalRequestV1::FsOpen {
                path: b"held.bin".to_vec(),
                mode: FsOpenModeV1::Read,
            },
        )
        .unwrap();
        let token = opened.resource_token.unwrap();
        let denied = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsHandleMetadata,
            None,
            ResourceInputsV1::Target(token),
            &CanonicalRequestV1::FsHandleMetadata,
        )
        .unwrap();
        assert_eq!(
            denied.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::RightNotHeld {
                required: crate::RightSet::METADATA.bits(),
                held: crate::RightSet::READ.bits(),
            },))
        );
        let released = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::ResourceRelease,
            None,
            ResourceInputsV1::Target(token),
            &CanonicalRequestV1::ResourceRelease,
        )
        .unwrap();
        assert!(matches!(
            released.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::ResourceSettlement(_))
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    struct PositionedBackend {
        root: crate::RootedHandle,
        write_limit: Option<usize>,
    }

    #[cfg(target_os = "linux")]
    impl HostEffectBackendV1 for PositionedBackend {
        fn console_write(&mut self, _: ConsoleStreamV1, _: &[u8]) -> Result<(), IoErrorIdentityV1> {
            unreachable!()
        }

        fn console_flush(&mut self, _: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1> {
            unreachable!()
        }

        fn console_is_terminal(&mut self, _: ConsoleStreamV1) -> bool {
            unreachable!()
        }

        fn fs_read_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
        ) -> Result<Vec<u8>, FileErrorCauseV1> {
            unreachable!()
        }

        fn fs_write_file(
            &mut self,
            _: &CapabilityGrantV1,
            _: &[u8],
            _: CreatePolicyV1,
            _: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            unreachable!()
        }

        fn fs_open_resource(
            &mut self,
            _: &CapabilityGrantV1,
            path: &[u8],
            mode: FsOpenModeV1,
        ) -> Result<crate::ResourceHandleV1, FileErrorCauseV1> {
            let leaf = crate::PathComponent::new(path).map_err(|error| {
                FileErrorCauseV1::Io(io_error_identity_v1(&error.into_io_error()))
            })?;
            let request = match mode {
                FsOpenModeV1::Read | FsOpenModeV1::Metadata => crate::OpenRequest::Read,
                FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateNew) => {
                    crate::OpenRequest::CreateNew
                }
                FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrTruncate) => {
                    crate::OpenRequest::CreateOrTruncate
                }
                FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrKeep) => {
                    crate::OpenRequest::CreateOrKeep
                }
            };
            crate::open_resource_at_v1(&self.root, &leaf, request)
                .map_err(|error| FileErrorCauseV1::Io(io_error_identity_v1(&error.into_io_error())))
        }

        fn fs_resource_write_at(
            &mut self,
            handle: &crate::ResourceHandleV1,
            offset: u64,
            bytes: &[u8],
        ) -> Result<usize, IoErrorIdentityV1> {
            let limit = self.write_limit.unwrap_or(bytes.len()).min(bytes.len());
            if limit == 0 {
                return Ok(0);
            }
            crate::resource_write_at_v1(handle, offset, &bytes[..limit])
                .map_err(|error| io_error_identity_v1(&error.into_io_error()))
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn bounded_positioned_io_reaches_progress_mismatch_and_ordered_bindings() {
        let root = std::env::temp_dir().join(format!("ken-px8r-positioned-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("source.bin"), b"abcdefghij").unwrap();
        std::fs::write(root.join("target.bin"), b"0123456789").unwrap();
        let rooted = crate::open_root(&crate::RootPath::new(&root).unwrap()).unwrap();
        let metadata = crate::metadata(&rooted).unwrap();
        let cap = crate::Cap::mint_scoped(
            crate::AUTH_FULL,
            "FS",
            crate::FsScope::root(
                crate::RightSet::ALL,
                crate::FsHandle::Posix(rooted.clone()),
                crate::FsIdentity::Posix {
                    device: metadata.identity.device,
                    inode: metadata.identity.inode,
                },
                crate::SymlinkPolicy::NoFollow,
            ),
        );
        let mut capabilities = CapabilityTableV1::default();
        let capability = capabilities.insert(CapabilityGrantV1 {
            identity: crate::program_caps_fs_trace_identity_v1(),
            capability: cap,
        });
        let mut resources = ResourceTableV1::with_buffer_limits(BufferLimitsV1::new(4, 6).unwrap());
        let mut backend = PositionedBackend {
            root: rooted,
            write_limit: None,
        };

        let source = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &CanonicalRequestV1::FsOpen {
                path: b"source.bin".to_vec(),
                mode: FsOpenModeV1::Read,
            },
        )
        .unwrap();
        let source_token = source.resource_token.unwrap();
        let source_identity = source.resource_bindings[0].1;
        let buffer = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::BufferAllocate,
            None,
            ResourceInputsV1::None,
            &CanonicalRequestV1::BufferAllocate { capacity: 4 },
        )
        .unwrap();
        let buffer_token = buffer.resource_token.unwrap();
        let buffer_identity = buffer.resource_bindings[0].1;
        assert_eq!(
            buffer.resource_bindings,
            vec![(ResourceBindingRole::Target, buffer_identity)]
        );
        let over_limit = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::BufferAllocate,
            None,
            ResourceInputsV1::None,
            &CanonicalRequestV1::BufferAllocate { capacity: 3 },
        )
        .unwrap();
        assert_eq!(
            over_limit.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::BufferLimit))
        );
        assert!(over_limit.resource_bindings.is_empty());

        let read_request = CanonicalRequestV1::FsReadAt {
            file_offset: 3,
            buffer_start: 0,
            length: 8,
        };
        let read = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsReadAt,
            None,
            ResourceInputsV1::FileBuffer {
                file: source_token,
                buffer: buffer_token,
            },
            &read_request,
        )
        .unwrap();
        assert!(matches!(
            read.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::ReadProgress(
                ReadProgressV1::ReadSome { ref span, ref transferred }
            )) if span.start == 0
               && span.length == 4
               && transferred.get() == 4
               && transferred.effective_request() == 4
        ));
        assert_eq!(
            read.resource_bindings,
            vec![
                (ResourceBindingRole::File, source_identity),
                (ResourceBindingRole::Buffer, buffer_identity),
            ]
        );
        let event = effect_event_from_dispatch(7, HostOpV1::FsReadAt, read_request, &read);
        assert_eq!(event.sequence, 7);
        assert_eq!(event.resource_bindings, read.resource_bindings);

        let freeze = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::BufferFreeze,
            None,
            ResourceInputsV1::BufferSpanTarget {
                target: buffer_token,
                span_origin: buffer_token,
            },
            &CanonicalRequestV1::BufferFreeze {
                start: 0,
                length: 4,
            },
        )
        .unwrap();
        assert_eq!(
            freeze.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(b"defg".to_vec()))
        );

        let target = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &CanonicalRequestV1::FsOpen {
                path: b"target.bin".to_vec(),
                mode: FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrKeep),
            },
        )
        .unwrap();
        let target_token = target
            .resource_token
            .unwrap_or_else(|| panic!("target open failed: {:?}", target.outcome));
        let target_identity = target.resource_bindings[0].1;
        let write_request = CanonicalRequestV1::FsWriteAt {
            file_offset: 2,
            buffer_start: 0,
            length: 4,
        };
        let full_write = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsWriteAt,
            None,
            ResourceInputsV1::FileBufferSpan {
                file: target_token,
                target_buffer: buffer_token,
                span_origin: buffer_token,
            },
            &write_request,
        )
        .unwrap();
        assert!(matches!(
            full_write.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::WriteProgress(
                WriteProgressV1::Wrote(ref count)
            )) if count.get() == 4
        ));
        assert_eq!(
            full_write.resource_bindings,
            vec![
                (ResourceBindingRole::File, target_identity),
                (ResourceBindingRole::Buffer, buffer_identity),
            ]
        );
        assert_eq!(
            std::fs::read(root.join("target.bin")).unwrap(),
            b"01defg6789"
        );

        std::fs::write(root.join("truncate.bin"), b"must-disappear").unwrap();
        let truncate = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &CanonicalRequestV1::FsOpen {
                path: b"truncate.bin".to_vec(),
                mode: FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrTruncate),
            },
        )
        .unwrap();
        let truncate_token = truncate.resource_token.unwrap();
        assert!(std::fs::read(root.join("truncate.bin")).unwrap().is_empty());
        for (file_offset, buffer_start) in [(0, 0), (2, 2)] {
            let reply = dispatch_host_op_v1(
                &mut backend,
                &capabilities,
                &mut resources,
                HostOpV1::FsWriteAt,
                None,
                ResourceInputsV1::FileBufferSpan {
                    file: truncate_token,
                    target_buffer: buffer_token,
                    span_origin: buffer_token,
                },
                &CanonicalRequestV1::FsWriteAt {
                    file_offset,
                    buffer_start,
                    length: 2,
                },
            )
            .unwrap();
            assert!(matches!(
                reply.outcome,
                CanonicalOutcomeV1::Success(CanonicalReplyV1::WriteProgress(
                    WriteProgressV1::Wrote(count)
                )) if count.get() == 2
            ));
        }
        assert_eq!(std::fs::read(root.join("truncate.bin")).unwrap(), b"defg");

        backend.write_limit = Some(2);
        let short_write = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsWriteAt,
            None,
            ResourceInputsV1::FileBufferSpan {
                file: target_token,
                target_buffer: buffer_token,
                span_origin: buffer_token,
            },
            &CanonicalRequestV1::FsWriteAt {
                file_offset: 6,
                buffer_start: 0,
                length: 4,
            },
        )
        .unwrap();
        assert!(matches!(
            short_write.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::WriteProgress(
                WriteProgressV1::Wrote(ref count)
            )) if count.get() == 2 && count.effective_request() == 4
        ));
        backend.write_limit = Some(0);
        let zero_write = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsWriteAt,
            None,
            ResourceInputsV1::FileBufferSpan {
                file: target_token,
                target_buffer: buffer_token,
                span_origin: buffer_token,
            },
            &CanonicalRequestV1::FsWriteAt {
                file_offset: 8,
                buffer_start: 0,
                length: 4,
            },
        )
        .unwrap();
        assert_eq!(
            zero_write.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::NoProgress))
        );
        assert_eq!(
            zero_write.resource_bindings,
            vec![
                (ResourceBindingRole::File, target_identity),
                (ResourceBindingRole::Buffer, buffer_identity),
            ]
        );

        let short_read = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsReadAt,
            None,
            ResourceInputsV1::FileBuffer {
                file: source_token,
                buffer: buffer_token,
            },
            &CanonicalRequestV1::FsReadAt {
                file_offset: 9,
                buffer_start: 0,
                length: 4,
            },
        )
        .unwrap();
        assert!(matches!(
            short_read.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::ReadProgress(
                ReadProgressV1::ReadSome { ref transferred, .. }
            )) if transferred.get() == 1 && transferred.effective_request() == 4
        ));
        let eof = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsReadAt,
            None,
            ResourceInputsV1::FileBuffer {
                file: source_token,
                buffer: buffer_token,
            },
            &CanonicalRequestV1::FsReadAt {
                file_offset: 10,
                buffer_start: 0,
                length: 4,
            },
        )
        .unwrap();
        assert_eq!(
            eof.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::ReadProgress(ReadProgressV1::ReadEof))
        );

        let buffer_on_file = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::BufferFreeze,
            None,
            ResourceInputsV1::BufferSpanTarget {
                target: source_token,
                span_origin: source_token,
            },
            &CanonicalRequestV1::BufferFreeze {
                start: 0,
                length: 1,
            },
        )
        .unwrap();
        assert_eq!(
            buffer_on_file.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(
                ResourceErrorV1::ResourceKindMismatch {
                    expected: ResourceKindV1::Buffer,
                    actual: ResourceKindV1::FsHandle,
                }
            ))
        );
        assert!(buffer_on_file.resource_bindings.is_empty());
        let file_on_buffer = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsHandleMetadata,
            None,
            ResourceInputsV1::Target(buffer_token),
            &CanonicalRequestV1::FsHandleMetadata,
        )
        .unwrap();
        assert_eq!(
            file_on_buffer.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(
                ResourceErrorV1::ResourceKindMismatch {
                    expected: ResourceKindV1::FsHandle,
                    actual: ResourceKindV1::Buffer,
                }
            ))
        );
        assert!(file_on_buffer.resource_bindings.is_empty());
        let right_denied = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsWriteAt,
            None,
            ResourceInputsV1::FileBufferSpan {
                file: source_token,
                target_buffer: buffer_token,
                span_origin: buffer_token,
            },
            &write_request,
        )
        .unwrap();
        assert!(matches!(
            right_denied.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(
                ResourceErrorV1::RightNotHeld { .. }
            ))
        ));

        let released = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::ResourceRelease,
            None,
            ResourceInputsV1::Target(buffer_token),
            &CanonicalRequestV1::ResourceRelease,
        )
        .unwrap();
        assert_eq!(
            released.resource_bindings,
            vec![(ResourceBindingRole::Target, buffer_identity)]
        );
        let closed = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::BufferFreeze,
            None,
            ResourceInputsV1::BufferSpanTarget {
                target: buffer_token,
                span_origin: buffer_token,
            },
            &CanonicalRequestV1::BufferFreeze {
                start: 0,
                length: 1,
            },
        )
        .unwrap();
        assert_eq!(
            closed.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::Closed))
        );
        assert!(closed.resource_bindings.is_empty());
        resources.finalize_all_with(|owner| {
            crate::close_resource_v1(owner)
                .map_err(|error| io_error_identity_v1(&error.into_io_error()))
        });
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn foreign_acquisition_span_rejects_on_both_consumers_before_bytes_or_backend() {
        // PX8-SPAN-PROV: two capacity-8 buffers A and B receive the same numeric
        // window [2,6) but distinct bytes (AAAA vs BBBB). A span minted from A
        // (`span_origin = token_a`) applied to a B-targeted freeze/write is a
        // foreign-acquisition span: the shared dispatcher rejects it with
        // InvalidBounds before exposing any bytes (freeze) or issuing any backend
        // write (write), while the own-span control (`span_origin = token_b`)
        // succeeds on the identical numeric shape. Acquisition is the only varied
        // field: capacity, start, length, and live window are all equal, so this
        // fails a numeric-only admission and its own-span controls fail an
        // always-reject one (AC-8 discriminator).
        let root =
            std::env::temp_dir().join(format!("ken-spanprov-unit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("source.bin"), b"AAAABBBB").unwrap();
        std::fs::write(root.join("target.bin"), b"00000000").unwrap();
        let rooted = crate::open_root(&crate::RootPath::new(&root).unwrap()).unwrap();
        let metadata = crate::metadata(&rooted).unwrap();
        let cap = crate::Cap::mint_scoped(
            crate::AUTH_FULL,
            "FS",
            crate::FsScope::root(
                crate::RightSet::ALL,
                crate::FsHandle::Posix(rooted.clone()),
                crate::FsIdentity::Posix {
                    device: metadata.identity.device,
                    inode: metadata.identity.inode,
                },
                crate::SymlinkPolicy::NoFollow,
            ),
        );
        let mut capabilities = CapabilityTableV1::default();
        let capability = capabilities.insert(CapabilityGrantV1 {
            identity: crate::program_caps_fs_trace_identity_v1(),
            capability: cap,
        });
        let mut resources =
            ResourceTableV1::with_buffer_limits(BufferLimitsV1::new(8, 16).unwrap());
        let mut backend = PositionedBackend {
            root: rooted,
            write_limit: None,
        };

        let source = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &CanonicalRequestV1::FsOpen {
                path: b"source.bin".to_vec(),
                mode: FsOpenModeV1::Read,
            },
        )
        .unwrap();
        let source_token = source.resource_token.unwrap();
        let target = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &CanonicalRequestV1::FsOpen {
                path: b"target.bin".to_vec(),
                mode: FsOpenModeV1::WriteCreate(CreatePolicyV1::CreateOrKeep),
            },
        )
        .unwrap();
        let target_token = target.resource_token.unwrap();

        let allocate = |resources: &mut ResourceTableV1, backend: &mut PositionedBackend| {
            dispatch_host_op_v1(
                backend,
                &capabilities,
                resources,
                HostOpV1::BufferAllocate,
                None,
                ResourceInputsV1::None,
                &CanonicalRequestV1::BufferAllocate { capacity: 8 },
            )
            .unwrap()
            .resource_token
            .unwrap()
        };
        let token_a = allocate(&mut resources, &mut backend);
        let token_b = allocate(&mut resources, &mut backend);

        // Install the same window [2,6) in both buffers with distinct bytes.
        for (buffer, file_offset) in [(token_a, 0u64), (token_b, 4u64)] {
            let read = dispatch_host_op_v1(
                &mut backend,
                &capabilities,
                &mut resources,
                HostOpV1::FsReadAt,
                None,
                ResourceInputsV1::FileBuffer {
                    file: source_token,
                    buffer,
                },
                &CanonicalRequestV1::FsReadAt {
                    file_offset,
                    buffer_start: 2,
                    length: 4,
                },
            )
            .unwrap();
            assert!(matches!(
                read.outcome,
                CanonicalOutcomeV1::Success(CanonicalReplyV1::ReadProgress(
                    ReadProgressV1::ReadSome { .. }
                ))
            ));
        }

        let freeze = |resources: &mut ResourceTableV1,
                      backend: &mut PositionedBackend,
                      span_origin: ResourceTokenV1| {
            dispatch_host_op_v1(
                backend,
                &capabilities,
                resources,
                HostOpV1::BufferFreeze,
                None,
                ResourceInputsV1::BufferSpanTarget {
                    target: token_b,
                    span_origin,
                },
                &CanonicalRequestV1::BufferFreeze {
                    start: 2,
                    length: 4,
                },
            )
            .unwrap()
            .outcome
        };
        // Foreign freeze: InvalidBounds, no bytes exposed.
        assert_eq!(
            freeze(&mut resources, &mut backend, token_a),
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::InvalidBounds))
        );
        // Own-span control: exactly B's window bytes.
        assert_eq!(
            freeze(&mut resources, &mut backend, token_b),
            CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(b"BBBB".to_vec()))
        );

        let write = |resources: &mut ResourceTableV1,
                     backend: &mut PositionedBackend,
                     span_origin: ResourceTokenV1| {
            dispatch_host_op_v1(
                backend,
                &capabilities,
                resources,
                HostOpV1::FsWriteAt,
                None,
                ResourceInputsV1::FileBufferSpan {
                    file: target_token,
                    target_buffer: token_b,
                    span_origin,
                },
                &CanonicalRequestV1::FsWriteAt {
                    file_offset: 0,
                    buffer_start: 2,
                    length: 4,
                },
            )
            .unwrap()
            .outcome
        };
        // Foreign write: InvalidBounds and zero backend writes (target unchanged).
        assert_eq!(
            write(&mut resources, &mut backend, token_a),
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::InvalidBounds))
        );
        assert_eq!(
            std::fs::read(root.join("target.bin")).unwrap(),
            b"00000000",
            "foreign-span write must issue zero backend writes"
        );
        // Own-span control: one backend write of exactly B's bytes.
        assert!(matches!(
            write(&mut resources, &mut backend, token_b),
            CanonicalOutcomeV1::Success(CanonicalReplyV1::WriteProgress(WriteProgressV1::Wrote(
                ref count
            ))) if count.get() == 4
        ));
        assert_eq!(std::fs::read(root.join("target.bin")).unwrap(), b"BBBB0000");

        resources.finalize_all_with(|owner| {
            crate::close_resource_v1(owner)
                .map_err(|error| io_error_identity_v1(&error.into_io_error()))
        });
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn released_acquisition_span_is_not_revived_by_slot_reuse() {
        // PX8-SPAN-PROV / SP-C: acquire buffer A (`token_a`), release it, then
        // allocate B which reuses A's vacated resource-table slot with a newer
        // acquisition generation. A span minted from A (`span_origin = token_a`)
        // applied to the reused B is rejected with InvalidBounds: slot identity
        // alone aliases the two acquisitions, but the full acquisition token
        // (slot+generation) does not, so release/reallocation is a permanent
        // verdict flip. A fresh span from B (`span_origin = token_b`) succeeds.
        let root = std::env::temp_dir()
            .join(format!("ken-spanprov-reuse-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("source.bin"), b"AAAABBBB").unwrap();
        let rooted = crate::open_root(&crate::RootPath::new(&root).unwrap()).unwrap();
        let metadata = crate::metadata(&rooted).unwrap();
        let cap = crate::Cap::mint_scoped(
            crate::AUTH_FULL,
            "FS",
            crate::FsScope::root(
                crate::RightSet::ALL,
                crate::FsHandle::Posix(rooted.clone()),
                crate::FsIdentity::Posix {
                    device: metadata.identity.device,
                    inode: metadata.identity.inode,
                },
                crate::SymlinkPolicy::NoFollow,
            ),
        );
        let mut capabilities = CapabilityTableV1::default();
        let capability = capabilities.insert(CapabilityGrantV1 {
            identity: crate::program_caps_fs_trace_identity_v1(),
            capability: cap,
        });
        let mut resources =
            ResourceTableV1::with_buffer_limits(BufferLimitsV1::new(8, 16).unwrap());
        let mut backend = PositionedBackend {
            root: rooted,
            write_limit: None,
        };
        let source = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsOpen,
            Some(capability),
            ResourceInputsV1::None,
            &CanonicalRequestV1::FsOpen {
                path: b"source.bin".to_vec(),
                mode: FsOpenModeV1::Read,
            },
        )
        .unwrap();
        let source_token = source.resource_token.unwrap();

        let token_a = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::BufferAllocate,
            None,
            ResourceInputsV1::None,
            &CanonicalRequestV1::BufferAllocate { capacity: 8 },
        )
        .unwrap()
        .resource_token
        .unwrap();
        dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::ResourceRelease,
            None,
            ResourceInputsV1::Target(token_a),
            &CanonicalRequestV1::ResourceRelease,
        )
        .unwrap();
        let token_b = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::BufferAllocate,
            None,
            ResourceInputsV1::None,
            &CanonicalRequestV1::BufferAllocate { capacity: 8 },
        )
        .unwrap()
        .resource_token
        .unwrap();
        // The reused acquisition inhabits A's vacated slot with a newer
        // generation, so it is a distinct opaque token.
        assert_ne!(
            token_b, token_a,
            "reallocation mints a distinct acquisition token"
        );
        assert_eq!(
            token_b.erased_identity() & 0xffff_ffff,
            token_a.erased_identity() & 0xffff_ffff,
            "the vacated resource-table slot is reused"
        );
        assert_ne!(
            token_b.erased_identity(),
            token_a.erased_identity(),
            "the reused acquisition has a newer generation"
        );

        // Install B's live window [2,6) = BBBB so the own-span control has bytes.
        let read = dispatch_host_op_v1(
            &mut backend,
            &capabilities,
            &mut resources,
            HostOpV1::FsReadAt,
            None,
            ResourceInputsV1::FileBuffer {
                file: source_token,
                buffer: token_b,
            },
            &CanonicalRequestV1::FsReadAt {
                file_offset: 4,
                buffer_start: 2,
                length: 4,
            },
        )
        .unwrap();
        assert!(matches!(
            read.outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::ReadProgress(
                ReadProgressV1::ReadSome { .. }
            ))
        ));

        let freeze = |resources: &mut ResourceTableV1,
                      backend: &mut PositionedBackend,
                      span_origin: ResourceTokenV1| {
            dispatch_host_op_v1(
                backend,
                &capabilities,
                resources,
                HostOpV1::BufferFreeze,
                None,
                ResourceInputsV1::BufferSpanTarget {
                    target: token_b,
                    span_origin,
                },
                &CanonicalRequestV1::BufferFreeze {
                    start: 2,
                    length: 4,
                },
            )
            .unwrap()
            .outcome
        };
        // The old acquisition's span does not survive release/reallocation.
        assert_eq!(
            freeze(&mut resources, &mut backend, token_a),
            CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::InvalidBounds))
        );
        // A fresh span from the reused acquisition succeeds.
        assert_eq!(
            freeze(&mut resources, &mut backend, token_b),
            CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(b"BBBB".to_vec()))
        );

        resources.finalize_all_with(|owner| {
            crate::close_resource_v1(owner)
                .map_err(|error| io_error_identity_v1(&error.into_io_error()))
        });
        std::fs::remove_dir_all(root).unwrap();
    }
}
