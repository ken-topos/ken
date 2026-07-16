//! The sole raw native host ABI boundary.
//!
//! Raw fields are validated here, then converted to the same typed request and
//! single safe dispatcher used by the interpreter-facing lane. The invocation
//! owns both the capability table and the append-only reply arena.

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::fs::OpenOptions;
use std::io::{self, Write};

use crate::{
    AUTH_FULL, AUTH_NONE, AUTH_PARTIAL, CanonicalOutcomeV1, CanonicalReplyV1, CanonicalRequestV1,
    Cap, CapabilityGrantV1, CapabilityTableV1, CapabilityTokenV1, ConsoleStreamV1, CreatePolicyV1,
    EffectEventV1, FileErrorCauseV1, FsHandle, FsIdentity, FsScope, HostEffectBackendV1, HostOpV1,
    IoErrorIdentityV1, OpenRequest, PathComponent, RootPath, RootedHandle, SymlinkPolicy,
    dispatch_host_op_v1,
};

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn ken_host_abi_v1_establish_sigpipe_ignore() -> std::ffi::c_int;
}

#[derive(Debug)]
struct ProcessPostureV1(());

fn establish_process_posture_v1() -> Result<ProcessPostureV1, ()> {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: this calls Ken's own no-argument C companion. The companion
        // obtains sigaction's layout and constants from the target headers,
        // installs SIG_IGN, retains no pointer, and returns only status.
        if unsafe { ken_host_abi_v1_establish_sigpipe_ignore() } == 0 {
            Ok(ProcessPostureV1(()))
        } else {
            Err(())
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(())
    }
}

#[repr(C)]
struct NativeInvocationV1 {
    process_input: *const c_void,
    host_context: *mut ProcessContext,
    capability: u64,
}

#[repr(C)]
pub(crate) struct HostInitResultV1 {
    context: *mut c_void,
    capability: u64,
    plan_hash: u64,
}

#[repr(C)]
struct SliceV1 {
    data: *const u8,
    len: usize,
}

#[repr(C)]
struct ConsoleWriteRequestV1 {
    stream: u64,
    bytes: SliceV1,
}

#[repr(C)]
#[allow(dead_code)] // Manifest-covered V1 lane; native execution is deferred.
struct ConsoleReadRequestV1 {
    stream: u64,
    limit: u64,
}

#[repr(C)]
struct ConsoleStreamRequestV1 {
    stream: u64,
}

#[repr(C)]
#[allow(dead_code)] // Manifest-covered V1 lane; native execution is deferred.
struct UnitRequestV1 {
    reserved: u8,
}

#[repr(C)]
struct FsReadFileRequestV1 {
    capability: u64,
    path: SliceV1,
}

#[repr(C)]
struct FsWriteFileRequestV1 {
    capability: u64,
    path: SliceV1,
    create_policy: u64,
    bytes: SliceV1,
}

#[repr(C)]
#[allow(dead_code)] // Manifest-covered V1 lane; native execution is deferred.
struct FsAppendFileRequestV1 {
    capability: u64,
    path: SliceV1,
    bytes: SliceV1,
}

#[repr(C)]
#[allow(dead_code)] // Manifest-covered V1 lane; native execution is deferred.
struct FsPathRequestV1 {
    capability: u64,
    path: SliceV1,
}

#[repr(C)]
#[allow(dead_code)] // Manifest-covered V1 lane; native execution is deferred.
struct FsRecursivePathRequestV1 {
    capability: u64,
    recursive: u64,
    path: SliceV1,
}

#[repr(C)]
#[allow(dead_code)] // Manifest-covered V1 lane; native execution is deferred.
struct FsRenameRequestV1 {
    capability: u64,
    source: SliceV1,
    destination: SliceV1,
}

#[repr(C)]
struct HostReplyV1 {
    tag: u64,
    detail: u64,
    bytes: SliceV1,
}

const REPLY_UNIT: u64 = 0;
const REPLY_BOOL: u64 = 1;
const REPLY_BYTES: u64 = 2;
const REPLY_ERROR: u64 = 3;

struct ProcessContext {
    _posture: ProcessPostureV1,
    host: ProcessHost,
    capabilities: CapabilityTableV1,
    response_arena: Vec<Box<[u8]>>,
    effect_trace: Vec<EffectEventV1>,
    observation: Option<std::fs::File>,
    plan_hash: u64,
    capability: CapabilityTokenV1,
}

struct ProcessHost;

impl ProcessHost {
    fn reject_symlink(parent: &RootedHandle, leaf: &PathComponent) -> Result<(), FileErrorCauseV1> {
        if crate::readlink_at(parent, leaf).is_ok() {
            Err(FileErrorCauseV1::Capability(
                crate::CapabilityDeniedV1::SymlinkDenied,
            ))
        } else {
            Ok(())
        }
    }

    fn components<'a>(path: &'a [u8]) -> Result<Vec<&'a [u8]>, FileErrorCauseV1> {
        if path.starts_with(b"/") || path.contains(&0) {
            return Err(FileErrorCauseV1::Capability(
                crate::CapabilityDeniedV1::ScopeEscape,
            ));
        }
        let mut out = Vec::new();
        for component in path.split(|byte| *byte == b'/') {
            if component.is_empty() || component == b"." {
                continue;
            }
            if component == b".." {
                return Err(FileErrorCauseV1::Capability(
                    crate::CapabilityDeniedV1::ScopeEscape,
                ));
            }
            out.push(component);
        }
        if out.is_empty() {
            return Err(FileErrorCauseV1::Io(IoErrorIdentityV1::InvalidInput));
        }
        Ok(out)
    }

    fn root(grant: &CapabilityGrantV1) -> Result<RootedHandle, FileErrorCauseV1> {
        match &grant.capability.scope().root {
            FsHandle::Posix(root) => Ok(root.clone()),
            FsHandle::Virtual(_) => Err(FileErrorCauseV1::Capability(
                crate::CapabilityDeniedV1::ScopeEscape,
            )),
        }
    }

    fn parent(
        grant: &CapabilityGrantV1,
        path: &[u8],
    ) -> Result<(RootedHandle, PathComponent), FileErrorCauseV1> {
        let components = Self::components(path)?;
        let mut current = Self::root(grant)?;
        for component in &components[..components.len() - 1] {
            let component = PathComponent::new(component).map_err(host_error)?;
            Self::reject_symlink(&current, &component)?;
            current = crate::open_at(&current, &component, OpenRequest::ReadDirectory)
                .map_err(host_error)?;
        }
        let leaf = PathComponent::new(components[components.len() - 1]).map_err(host_error)?;
        Self::reject_symlink(&current, &leaf)?;
        Ok((current, leaf))
    }
}

impl HostEffectBackendV1 for ProcessHost {
    fn console_write(
        &mut self,
        stream: ConsoleStreamV1,
        bytes: &[u8],
    ) -> Result<(), IoErrorIdentityV1> {
        let result = match stream {
            ConsoleStreamV1::Stdout => io::stdout().lock().write_all(bytes),
            ConsoleStreamV1::Stderr => io::stderr().lock().write_all(bytes),
            ConsoleStreamV1::Stdin => Err(io::ErrorKind::InvalidInput.into()),
        };
        result.map_err(|error| crate::io_error_identity_v1(&error))
    }

    fn console_flush(&mut self, stream: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1> {
        let result = match stream {
            ConsoleStreamV1::Stdout => io::stdout().lock().flush(),
            ConsoleStreamV1::Stderr => io::stderr().lock().flush(),
            ConsoleStreamV1::Stdin => Err(io::ErrorKind::InvalidInput.into()),
        };
        result.map_err(|error| crate::io_error_identity_v1(&error))
    }

    fn console_is_terminal(&mut self, stream: ConsoleStreamV1) -> bool {
        use std::io::IsTerminal;
        match stream {
            ConsoleStreamV1::Stdin => io::stdin().is_terminal(),
            ConsoleStreamV1::Stdout => io::stdout().is_terminal(),
            ConsoleStreamV1::Stderr => io::stderr().is_terminal(),
        }
    }

    fn fs_read_file(
        &mut self,
        grant: &CapabilityGrantV1,
        path: &[u8],
    ) -> Result<Vec<u8>, FileErrorCauseV1> {
        let (parent, leaf) = Self::parent(grant, path)?;
        let handle = crate::open_at(&parent, &leaf, OpenRequest::Read).map_err(host_error)?;
        crate::read(&handle).map_err(host_error)
    }

    fn fs_write_file(
        &mut self,
        grant: &CapabilityGrantV1,
        path: &[u8],
        policy: CreatePolicyV1,
        bytes: &[u8],
    ) -> Result<(), FileErrorCauseV1> {
        let (parent, leaf) = Self::parent(grant, path)?;
        match crate::open_at(&parent, &leaf, OpenRequest::ReadWrite) {
            Ok(handle) => match policy {
                CreatePolicyV1::CreateNew => {
                    Err(FileErrorCauseV1::Io(IoErrorIdentityV1::AlreadyExists))
                }
                CreatePolicyV1::CreateOrKeep => Ok(()),
                CreatePolicyV1::CreateOrTruncate => {
                    crate::replace(&handle, bytes).map_err(host_error)
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let request = match policy {
                    CreatePolicyV1::CreateNew => OpenRequest::CreateNew,
                    CreatePolicyV1::CreateOrTruncate => OpenRequest::CreateOrTruncate,
                    CreatePolicyV1::CreateOrKeep => OpenRequest::CreateOrKeep,
                };
                match crate::open_at(&parent, &leaf, request) {
                    Ok(handle) => crate::write_new(&handle, bytes).map_err(host_error),
                    Err(error)
                        if policy == CreatePolicyV1::CreateOrKeep
                            && error.kind() == io::ErrorKind::AlreadyExists =>
                    {
                        Ok(())
                    }
                    Err(error) => Err(host_error(error)),
                }
            }
            Err(error) => Err(host_error(error)),
        }
    }
}

fn host_error(error: crate::HostError) -> FileErrorCauseV1 {
    let error = error.into_io_error();
    FileErrorCauseV1::Io(crate::io_error_identity_v1(&error))
}

fn authority(tag: u64) -> Option<crate::Authority> {
    match tag {
        0 => Some(AUTH_NONE),
        1 => Some(AUTH_PARTIAL),
        2 => Some(AUTH_FULL),
        _ => None,
    }
}

fn stream(tag: u64) -> Option<ConsoleStreamV1> {
    match tag {
        0 => Some(ConsoleStreamV1::Stdin),
        1 => Some(ConsoleStreamV1::Stdout),
        2 => Some(ConsoleStreamV1::Stderr),
        _ => None,
    }
}

fn require_native_operation_v1(operation: HostOpV1) -> Result<(), crate::TerminalErrorV1> {
    if operation.availability() == crate::HostOpAvailabilityV1::NativeTested {
        Ok(())
    } else {
        Err(crate::TerminalErrorV1::OperationUnavailable(operation))
    }
}

unsafe fn borrowed_slice<'a>(slice: &SliceV1) -> Option<&'a [u8]> {
    if slice.len == 0 {
        return Some(&[]);
    }
    if slice.data.is_null() || slice.len > isize::MAX as usize {
        return None;
    }
    // SAFETY: the generated caller owns this immutable slice for the complete
    // synchronous dispatch call, and null was rejected above.
    Some(unsafe { std::slice::from_raw_parts(slice.data, slice.len) })
}

/// Creates the one call-scoped host context. The returned pointer is opaque to
/// generated code and must be paired with `ken_host_invocation_v1_destroy`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_host_invocation_v1_init(
    cwd: *const u8,
    cwd_len: usize,
    authority_tag: u64,
    plan_hash: u64,
    target_abi_hash: *const u8,
    host_effect_abi_hash: *const u8,
    observation_path: *const u8,
    observation_path_len: usize,
    result: *mut HostInitResultV1,
) -> i64 {
    if result.is_null() || !result.is_aligned() {
        return -1;
    }
    // Clear the caller-owned result before any validation or host action.
    unsafe {
        result.write(HostInitResultV1 {
            context: std::ptr::null_mut(),
            capability: 0,
            plan_hash: 0,
        });
    }
    let cwd = SliceV1 {
        data: cwd,
        len: cwd_len,
    };
    let Some(cwd) = (unsafe { borrowed_slice(&cwd) }) else {
        return -1;
    };
    let Some(authority) = authority(authority_tag) else {
        return -1;
    };
    if plan_hash == 0 || target_abi_hash.is_null() || host_effect_abi_hash.is_null() {
        return -1;
    }
    // SAFETY: both artifact-owned manifest arrays have the fixed V1 length.
    let target_hash = unsafe { std::slice::from_raw_parts(target_abi_hash, 32) };
    // SAFETY: both artifact-owned manifest arrays have the fixed V1 length.
    let effect_hash = unsafe { std::slice::from_raw_parts(host_effect_abi_hash, 32) };
    if target_hash != crate::TARGET_ABI_MANIFEST_HASH
        || effect_hash != crate::HOST_EFFECT_ABI_V1_HASH
    {
        return -1;
    }
    #[cfg(target_os = "linux")]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        std::path::PathBuf::from(std::ffi::OsString::from_vec(cwd.to_vec()))
    };
    #[cfg(not(target_os = "linux"))]
    let path = match std::str::from_utf8(cwd) {
        Ok(path) => std::path::PathBuf::from(path),
        Err(_) => return -1,
    };
    let Ok(root_path) = RootPath::new(path) else {
        return -1;
    };
    let observation_path = SliceV1 {
        data: observation_path,
        len: observation_path_len,
    };
    let Some(observation_path) = (unsafe { borrowed_slice(&observation_path) }) else {
        return -1;
    };
    let Some(context) = initialize_process_context(
        root_path,
        authority,
        plan_hash,
        observation_path,
        establish_process_posture_v1(),
    ) else {
        return -1;
    };
    let capability = context.capability.erased_identity();
    let context = Box::into_raw(context).cast();
    unsafe {
        result.write(HostInitResultV1 {
            context,
            capability,
            plan_hash,
        });
    }
    0
}

fn initialize_process_context(
    root_path: RootPath,
    authority: crate::Authority,
    plan_hash: u64,
    observation_path: &[u8],
    posture: Result<ProcessPostureV1, ()>,
) -> Option<Box<ProcessContext>> {
    let posture = posture.ok()?;
    let Ok(root) = crate::open_root(&root_path) else {
        return None;
    };
    let Ok(metadata) = crate::metadata(&root) else {
        return None;
    };
    let cap = Cap::mint_scoped(
        authority,
        "FS",
        FsScope::root(
            crate::rights_for_authority(authority),
            FsHandle::Posix(root),
            FsIdentity::Posix {
                device: metadata.identity.device,
                inode: metadata.identity.inode,
            },
            SymlinkPolicy::NoFollow,
        ),
    );
    let mut capabilities = CapabilityTableV1::default();
    let capability = capabilities.insert(CapabilityGrantV1 {
        identity: crate::program_caps_fs_trace_identity_v1(),
        capability: cap,
    });
    let observation = if observation_path.is_empty() {
        None
    } else {
        #[cfg(target_os = "linux")]
        let path = {
            use std::os::unix::ffi::OsStringExt;
            std::path::PathBuf::from(std::ffi::OsString::from_vec(observation_path.to_vec()))
        };
        #[cfg(not(target_os = "linux"))]
        let path = std::path::PathBuf::from(std::str::from_utf8(observation_path).ok()?);
        Some(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path)
                .ok()?,
        )
    };
    Some(Box::new(ProcessContext {
        _posture: posture,
        host: ProcessHost,
        capabilities,
        response_arena: Vec::new(),
        effect_trace: Vec::new(),
        observation,
        plan_hash,
        capability,
    }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_host_invocation_v1_destroy(context: *mut c_void) {
    // SAFETY: destroy is the fail-closed lifecycle fallback for callers that
    // cannot provide a terminal value; it still emits any completed events.
    let _ = unsafe { ken_host_invocation_v1_finish(context, -5) };
}

/// Completes the call-scoped observation before releasing reply storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_host_invocation_v1_finish(
    context: *mut c_void,
    terminal_value: i64,
) -> i64 {
    if context.is_null() || !context.cast::<ProcessContext>().is_aligned() {
        return -1;
    }
    // SAFETY: the starter supplies the unique init result exactly once after
    // Ken entry returns; no borrowed response is used after this call.
    let mut context = unsafe { Box::from_raw(context.cast::<ProcessContext>()) };
    if let Some(mut sink) = context.observation.take() {
        if write_observation_v1(&mut sink, &context, terminal_value).is_err() {
            return -1;
        }
    }
    0
}

fn write_observation_v1(
    sink: &mut impl Write,
    context: &ProcessContext,
    terminal_value: i64,
) -> io::Result<()> {
    let bytes = crate::encode_linked_effect_trace_v1(&crate::LinkedEffectTraceV1 {
        plan_hash: context.plan_hash,
        target_abi_hash: crate::TARGET_ABI_MANIFEST_HASH,
        host_effect_abi_hash: crate::HOST_EFFECT_ABI_V1_HASH,
        terminal_value,
        effect_trace: context.effect_trace.clone(),
    })
    .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
    sink.write_all(&bytes)?;
    sink.flush()
}

fn set_reply(reply: &mut HostReplyV1, outcome: CanonicalOutcomeV1, context: &mut ProcessContext) {
    reply.detail = 0;
    reply.bytes = SliceV1 {
        data: std::ptr::null(),
        len: 0,
    };
    match outcome {
        CanonicalOutcomeV1::Success(CanonicalReplyV1::Unit) => reply.tag = REPLY_UNIT,
        CanonicalOutcomeV1::Success(CanonicalReplyV1::Bool(value)) => {
            reply.tag = REPLY_BOOL;
            reply.detail = u64::from(value);
        }
        CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(bytes)) => {
            context.response_arena.push(bytes.into_boxed_slice());
            let bytes = context
                .response_arena
                .last()
                .expect("response was appended");
            reply.tag = REPLY_BYTES;
            reply.bytes = SliceV1 {
                data: bytes.as_ptr(),
                len: bytes.len(),
            };
        }
        CanonicalOutcomeV1::Error(error) => {
            reply.tag = REPLY_ERROR;
            reply.detail = match error {
                crate::SemanticErrorV1::Io(error) => io_error_tag(error),
                crate::SemanticErrorV1::File(error) => match error.cause {
                    FileErrorCauseV1::Io(error) => io_error_tag(error),
                    FileErrorCauseV1::Capability(_) => 2,
                },
                crate::SemanticErrorV1::Capability(_) => 2,
            };
        }
        CanonicalOutcomeV1::Success(_) => reply.tag = REPLY_ERROR,
    }
}

fn io_error_tag(error: IoErrorIdentityV1) -> u64 {
    match error {
        IoErrorIdentityV1::NotFound => 0,
        IoErrorIdentityV1::PermissionDenied => 1,
        IoErrorIdentityV1::BrokenPipe => 3,
        IoErrorIdentityV1::Interrupted => 4,
        IoErrorIdentityV1::AlreadyExists => 5,
        IoErrorIdentityV1::InvalidInput => 6,
        IoErrorIdentityV1::IsDirectory => 7,
        IoErrorIdentityV1::NotDirectory => 8,
        IoErrorIdentityV1::NotEmpty => 9,
        IoErrorIdentityV1::Unsupported => 10,
        IoErrorIdentityV1::Other(raw) => (u64::from(raw as u32) << 32) | 11,
    }
}

/// The single private V1 dispatch symbol linked into produced executables.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_host_dispatch_v1(
    invocation: *const c_void,
    op: u64,
    request: *const c_void,
    request_size: usize,
    reply: *mut c_void,
) -> i64 {
    if invocation.is_null()
        || request.is_null()
        || reply.is_null()
        || !invocation.cast::<NativeInvocationV1>().is_aligned()
        || !reply.cast::<HostReplyV1>().is_aligned()
    {
        return -1;
    }
    // SAFETY: the starter owns this immutable invocation through entry return.
    let invocation = unsafe { &*(invocation.cast::<NativeInvocationV1>()) };
    if invocation.host_context.is_null() || !invocation.host_context.is_aligned() {
        return -1;
    }
    // SAFETY: init created this unique context; generated calls are sequential.
    let context = unsafe { &mut *invocation.host_context };
    let Ok(op) = u16::try_from(op)
        .ok()
        .and_then(|op| HostOpV1::try_from(op).ok())
        .ok_or(())
    else {
        return -3;
    };
    if require_native_operation_v1(op).is_err() {
        return -(0x1_0000_i64 + i64::from(op as u16));
    }
    let (capability, request) = match op {
        HostOpV1::ConsoleWrite if request_size == std::mem::size_of::<ConsoleWriteRequestV1>() => {
            if !request.cast::<ConsoleWriteRequestV1>().is_aligned() {
                return -1;
            }
            let wire = unsafe { &*(request.cast::<ConsoleWriteRequestV1>()) };
            let (Some(stream), Some(bytes)) =
                (stream(wire.stream), unsafe { borrowed_slice(&wire.bytes) })
            else {
                return -1;
            };
            (
                None,
                CanonicalRequestV1::ConsoleWrite {
                    stream,
                    bytes: bytes.to_vec(),
                },
            )
        }
        HostOpV1::ConsoleFlush | HostOpV1::ConsoleIsTerminal
            if request_size == std::mem::size_of::<ConsoleStreamRequestV1>() =>
        {
            if !request.cast::<ConsoleStreamRequestV1>().is_aligned() {
                return -1;
            }
            let wire = unsafe { &*(request.cast::<ConsoleStreamRequestV1>()) };
            let Some(stream) = stream(wire.stream) else {
                return -1;
            };
            let request = if op == HostOpV1::ConsoleFlush {
                CanonicalRequestV1::ConsoleFlush { stream }
            } else {
                CanonicalRequestV1::ConsoleIsTerminal { stream }
            };
            (None, request)
        }
        HostOpV1::FsReadFile if request_size == std::mem::size_of::<FsReadFileRequestV1>() => {
            if !request.cast::<FsReadFileRequestV1>().is_aligned() {
                return -1;
            }
            let wire = unsafe { &*(request.cast::<FsReadFileRequestV1>()) };
            let Some(path) = (unsafe { borrowed_slice(&wire.path) }) else {
                return -1;
            };
            (
                Some(CapabilityTokenV1::from_erased_identity(wire.capability)),
                CanonicalRequestV1::FsReadFile {
                    path: path.to_vec(),
                },
            )
        }
        HostOpV1::FsWriteFile if request_size == std::mem::size_of::<FsWriteFileRequestV1>() => {
            if !request.cast::<FsWriteFileRequestV1>().is_aligned() {
                return -1;
            }
            let wire = unsafe { &*(request.cast::<FsWriteFileRequestV1>()) };
            let (Some(path), Some(bytes)) = (unsafe { borrowed_slice(&wire.path) }, unsafe {
                borrowed_slice(&wire.bytes)
            }) else {
                return -1;
            };
            let policy = match wire.create_policy {
                0 => CreatePolicyV1::CreateNew,
                1 => CreatePolicyV1::CreateOrTruncate,
                2 => CreatePolicyV1::CreateOrKeep,
                _ => return -1,
            };
            (
                Some(CapabilityTokenV1::from_erased_identity(wire.capability)),
                CanonicalRequestV1::FsWriteFile {
                    path: path.to_vec(),
                    create_policy: policy,
                    bytes: bytes.to_vec(),
                },
            )
        }
        _ => return -3,
    };
    let result = dispatch_host_op_v1(
        &mut context.host,
        &context.capabilities,
        op,
        capability,
        &request,
    );
    let Ok(result) = result else {
        return -3;
    };
    context.effect_trace.push(EffectEventV1 {
        sequence: context.effect_trace.len() as u64,
        operation: op,
        capability: result.capability_identity,
        request,
        outcome: result.outcome.clone(),
    });
    // SAFETY: generated code supplies an aligned writable HostReplyV1 slot.
    set_reply(
        unsafe { &mut *(reply.cast::<HostReplyV1>()) },
        result.outcome,
        context,
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn effect_fact(name: &str) -> u64 {
        crate::HOST_EFFECT_ABI_V1_FACTS
            .iter()
            .find_map(|(fact, value)| (*fact == name).then_some(*value))
            .unwrap_or_else(|| panic!("missing generated effect ABI fact {name}"))
    }

    fn effect_binding(kind: &str, name: &str) -> u64 {
        crate::HOST_EFFECT_ABI_V1_BINDINGS
            .iter()
            .find_map(|(bound_kind, bound_name, value)| {
                (*bound_kind == kind && *bound_name == name).then_some(*value)
            })
            .unwrap_or_else(|| panic!("missing generated effect ABI binding {kind}:{name}"))
    }

    #[test]
    fn generated_effect_layout_matches_every_live_wire_record() {
        macro_rules! size_align {
            ($name:literal, $ty:ty) => {
                assert_eq!(
                    effect_fact(concat!("SIZE_", $name)),
                    std::mem::size_of::<$ty>() as u64
                );
                assert_eq!(
                    effect_fact(concat!("ALIGN_", $name)),
                    std::mem::align_of::<$ty>() as u64
                );
            };
        }
        size_align!("SliceV1", SliceV1);
        size_align!("CapabilityTokenV1", CapabilityTokenV1);
        size_align!("NativeInvocationV1", NativeInvocationV1);
        size_align!("HostInitResultV1", HostInitResultV1);
        size_align!("ConsoleWriteRequestV1", ConsoleWriteRequestV1);
        size_align!("ConsoleReadRequestV1", ConsoleReadRequestV1);
        size_align!("ConsoleStreamRequestV1", ConsoleStreamRequestV1);
        size_align!("UnitRequestV1", UnitRequestV1);
        size_align!("FsReadFileRequestV1", FsReadFileRequestV1);
        size_align!("FsWriteFileRequestV1", FsWriteFileRequestV1);
        size_align!("FsAppendFileRequestV1", FsAppendFileRequestV1);
        size_align!("FsPathRequestV1", FsPathRequestV1);
        size_align!("FsRecursivePathRequestV1", FsRecursivePathRequestV1);
        size_align!("FsRenameRequestV1", FsRenameRequestV1);
        size_align!("HostReplyV1", HostReplyV1);
        macro_rules! offset {
            ($record:literal, $ty:ty, $field:ident) => {
                assert_eq!(
                    effect_fact(concat!("OFFSET_", $record, "_", stringify!($field))),
                    std::mem::offset_of!($ty, $field) as u64
                );
            };
        }
        offset!("SliceV1", SliceV1, data);
        offset!("SliceV1", SliceV1, len);
        offset!("NativeInvocationV1", NativeInvocationV1, process_input);
        offset!("NativeInvocationV1", NativeInvocationV1, host_context);
        offset!("NativeInvocationV1", NativeInvocationV1, capability);
        offset!("HostInitResultV1", HostInitResultV1, context);
        offset!("HostInitResultV1", HostInitResultV1, capability);
        offset!("HostInitResultV1", HostInitResultV1, plan_hash);
        offset!("ConsoleWriteRequestV1", ConsoleWriteRequestV1, stream);
        offset!("ConsoleWriteRequestV1", ConsoleWriteRequestV1, bytes);
        offset!("ConsoleReadRequestV1", ConsoleReadRequestV1, stream);
        offset!("ConsoleReadRequestV1", ConsoleReadRequestV1, limit);
        offset!("ConsoleStreamRequestV1", ConsoleStreamRequestV1, stream);
        offset!("UnitRequestV1", UnitRequestV1, reserved);
        offset!("FsReadFileRequestV1", FsReadFileRequestV1, capability);
        offset!("FsReadFileRequestV1", FsReadFileRequestV1, path);
        offset!("FsWriteFileRequestV1", FsWriteFileRequestV1, capability);
        offset!("FsWriteFileRequestV1", FsWriteFileRequestV1, path);
        offset!("FsWriteFileRequestV1", FsWriteFileRequestV1, create_policy);
        offset!("FsWriteFileRequestV1", FsWriteFileRequestV1, bytes);
        offset!("FsAppendFileRequestV1", FsAppendFileRequestV1, capability);
        offset!("FsAppendFileRequestV1", FsAppendFileRequestV1, path);
        offset!("FsAppendFileRequestV1", FsAppendFileRequestV1, bytes);
        offset!("FsPathRequestV1", FsPathRequestV1, capability);
        offset!("FsPathRequestV1", FsPathRequestV1, path);
        offset!(
            "FsRecursivePathRequestV1",
            FsRecursivePathRequestV1,
            capability
        );
        offset!(
            "FsRecursivePathRequestV1",
            FsRecursivePathRequestV1,
            recursive
        );
        offset!("FsRecursivePathRequestV1", FsRecursivePathRequestV1, path);
        offset!("FsRenameRequestV1", FsRenameRequestV1, capability);
        offset!("FsRenameRequestV1", FsRenameRequestV1, source);
        offset!("FsRenameRequestV1", FsRenameRequestV1, destination);
        offset!("HostReplyV1", HostReplyV1, tag);
        offset!("HostReplyV1", HostReplyV1, detail);
        offset!("HostReplyV1", HostReplyV1, bytes);
        assert_eq!(crate::HOST_EFFECT_ABI_V1_CATALOG.len(), 14);
        for operation in HostOpV1::ALL {
            let row = crate::HOST_EFFECT_ABI_V1_CATALOG
                .iter()
                .find(|row| row.1 == operation as u16)
                .expect("every HostOpV1 has one generated catalog row");
            assert_eq!(
                row.2 == "native",
                operation.availability() == crate::HostOpAvailabilityV1::NativeTested
            );
        }
        assert_eq!(effect_binding("tag", "reply.unit"), REPLY_UNIT);
        assert_eq!(effect_binding("tag", "reply.bool"), REPLY_BOOL);
        assert_eq!(effect_binding("tag", "reply.bytes"), REPLY_BYTES);
        assert_eq!(effect_binding("tag", "reply.error"), REPLY_ERROR);
        assert_eq!(effect_binding("error", "io.BrokenPipe"), 3);
        assert_eq!(effect_binding("error", "io.Other"), 11);
    }

    #[test]
    fn every_deferred_operation_has_its_own_named_native_boundary_rejection() {
        for operation in HostOpV1::ALL {
            let status = require_native_operation_v1(operation);
            assert_eq!(
                status.is_ok(),
                operation.availability() == crate::HostOpAvailabilityV1::NativeTested
            );
            if let Err(crate::TerminalErrorV1::OperationUnavailable(rejected)) = status {
                assert_eq!(rejected, operation);
            }
        }
    }

    fn context(directory: &std::path::Path) -> HostInitResultV1 {
        #[cfg(target_os = "linux")]
        let cwd = {
            use std::os::unix::ffi::OsStrExt;
            directory.as_os_str().as_bytes().to_vec()
        };
        #[cfg(not(target_os = "linux"))]
        let cwd = directory.to_string_lossy().as_bytes().to_vec();
        let mut result = HostInitResultV1 {
            context: std::ptr::null_mut(),
            capability: 0,
            plan_hash: 0,
        };
        let status = unsafe {
            ken_host_invocation_v1_init(
                cwd.as_ptr(),
                cwd.len(),
                2,
                1,
                crate::TARGET_ABI_MANIFEST_HASH.as_ptr(),
                crate::HOST_EFFECT_ABI_V1_HASH.as_ptr(),
                std::ptr::null(),
                0,
                &mut result,
            )
        };
        assert_eq!(status, 0);
        assert_ne!(result.capability, 0);
        result
    }

    #[test]
    fn manifest_mismatch_fails_before_context_creation() {
        let mut wrong = crate::HOST_EFFECT_ABI_V1_HASH;
        wrong[0] ^= 1;
        let cwd = b".";
        let mut result = HostInitResultV1 {
            context: std::ptr::null_mut(),
            capability: 0,
            plan_hash: 0,
        };
        let status = unsafe {
            ken_host_invocation_v1_init(
                cwd.as_ptr(),
                cwd.len(),
                2,
                1,
                crate::TARGET_ABI_MANIFEST_HASH.as_ptr(),
                wrong.as_ptr(),
                std::ptr::null(),
                0,
                &mut result,
            )
        };
        assert_ne!(status, 0);
        assert!(result.context.is_null());
    }

    #[test]
    fn zero_plan_binding_clears_the_complete_init_result() {
        let cwd = b".";
        let mut result = HostInitResultV1 {
            context: usize::MAX as *mut c_void,
            capability: u64::MAX,
            plan_hash: u64::MAX,
        };
        let status = unsafe {
            ken_host_invocation_v1_init(
                cwd.as_ptr(),
                cwd.len(),
                2,
                0,
                crate::TARGET_ABI_MANIFEST_HASH.as_ptr(),
                crate::HOST_EFFECT_ABI_V1_HASH.as_ptr(),
                std::ptr::null(),
                0,
                &mut result,
            )
        };
        assert_ne!(status, 0);
        assert!(result.context.is_null());
        assert_eq!(result.capability, 0);
        assert_eq!(result.plan_hash, 0);
    }

    #[test]
    fn posture_failure_prevents_context_publication() {
        let root = RootPath::new(std::env::current_dir().unwrap()).unwrap();
        assert!(initialize_process_context(root, AUTH_FULL, 1, &[], Err(())).is_none());
    }

    #[test]
    fn wrong_generation_is_capability_denied_before_filesystem_access() {
        let directory =
            std::env::temp_dir().join(format!("ken-px5-malformed-token-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let initialized = context(&directory);
        assert!(!initialized.context.is_null());
        let invocation = NativeInvocationV1 {
            process_input: std::ptr::null(),
            host_context: initialized.context.cast(),
            capability: (2_u64 << 32),
        };
        let path = b"must-not-be-read";
        let request = FsReadFileRequestV1 {
            capability: invocation.capability,
            path: SliceV1 {
                data: path.as_ptr(),
                len: path.len(),
            },
        };
        let mut reply = HostReplyV1 {
            tag: u64::MAX,
            detail: u64::MAX,
            bytes: SliceV1 {
                data: std::ptr::null(),
                len: 0,
            },
        };
        let status = unsafe {
            ken_host_dispatch_v1(
                (&invocation as *const NativeInvocationV1).cast(),
                HostOpV1::FsReadFile as u64,
                (&request as *const FsReadFileRequestV1).cast(),
                std::mem::size_of::<FsReadFileRequestV1>(),
                (&mut reply as *mut HostReplyV1).cast(),
            )
        };
        assert_eq!(status, 0);
        assert_eq!(reply.tag, REPLY_ERROR);
        assert_eq!(reply.detail, 2);
        assert!(!directory.join("must-not-be-read").exists());
        let context = unsafe { &*initialized.context.cast::<ProcessContext>() };
        let [event] = context.effect_trace.as_slice() else {
            panic!("one denied dispatch must emit exactly one event")
        };
        assert_eq!(event.capability, None);
        assert!(matches!(
            &event.outcome,
            CanonicalOutcomeV1::Error(crate::SemanticErrorV1::File(crate::FileErrorIdentityV1 {
                cause: FileErrorCauseV1::Capability(crate::CapabilityDeniedV1::MalformedCapability),
                ..
            }))
        ));
        unsafe { ken_host_invocation_v1_destroy(initialized.context) };
        let _ = std::fs::remove_dir_all(directory);
    }
}
