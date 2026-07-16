//! The sole raw native host ABI boundary.
//!
//! Raw fields are validated here, then converted to the same typed request and
//! single safe dispatcher used by the interpreter-facing lane. The invocation
//! owns both the capability table and the append-only reply arena.

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::io::{self, Write};

use crate::{
    dispatch_host_op_v1, CanonicalOutcomeV1, CanonicalReplyV1, CanonicalRequestV1, Cap,
    CapabilityGrantV1, CapabilityTableV1, CapabilityTokenV1, CapabilityTraceIdentity,
    ConsoleStreamV1, CreatePolicyV1, EffectEventV1, FileErrorCauseV1, FsHandle, FsIdentity,
    FsScope, HostEffectBackendV1, HostOpV1, IoErrorIdentityV1, OpenRequest, PathComponent,
    RootPath, RootedHandle, SymlinkPolicy, AUTH_FULL, AUTH_NONE, AUTH_PARTIAL,
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
struct ConsoleStreamRequestV1 {
    stream: u64,
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
        result.map_err(map_io_error)
    }

    fn console_flush(&mut self, stream: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1> {
        let result = match stream {
            ConsoleStreamV1::Stdout => io::stdout().lock().flush(),
            ConsoleStreamV1::Stderr => io::stderr().lock().flush(),
            ConsoleStreamV1::Stdin => Err(io::ErrorKind::InvalidInput.into()),
        };
        result.map_err(map_io_error)
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
    FileErrorCauseV1::Io(map_io_error(error.into_io_error()))
}

fn map_io_error(error: io::Error) -> IoErrorIdentityV1 {
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
    target_abi_hash: *const u8,
    host_effect_abi_hash: *const u8,
) -> *mut c_void {
    let cwd = SliceV1 {
        data: cwd,
        len: cwd_len,
    };
    let Some(cwd) = (unsafe { borrowed_slice(&cwd) }) else {
        return std::ptr::null_mut();
    };
    let Some(authority) = authority(authority_tag) else {
        return std::ptr::null_mut();
    };
    if target_abi_hash.is_null() || host_effect_abi_hash.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: both artifact-owned manifest arrays have the fixed V1 length.
    let target_hash = unsafe { std::slice::from_raw_parts(target_abi_hash, 32) };
    // SAFETY: both artifact-owned manifest arrays have the fixed V1 length.
    let effect_hash = unsafe { std::slice::from_raw_parts(host_effect_abi_hash, 32) };
    if target_hash != crate::TARGET_ABI_MANIFEST_HASH
        || effect_hash != crate::HOST_EFFECT_ABI_V1_HASH
    {
        return std::ptr::null_mut();
    }
    #[cfg(target_os = "linux")]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        std::path::PathBuf::from(std::ffi::OsString::from_vec(cwd.to_vec()))
    };
    #[cfg(not(target_os = "linux"))]
    let path = match std::str::from_utf8(cwd) {
        Ok(path) => std::path::PathBuf::from(path),
        Err(_) => return std::ptr::null_mut(),
    };
    let Ok(root_path) = RootPath::new(path) else {
        return std::ptr::null_mut();
    };
    let Some(context) =
        initialize_process_context(root_path, authority, establish_process_posture_v1())
    else {
        return std::ptr::null_mut();
    };
    Box::into_raw(context).cast()
}

fn initialize_process_context(
    root_path: RootPath,
    authority: crate::Authority,
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
    capabilities.insert(CapabilityGrantV1 {
        identity: CapabilityTraceIdentity("declared:FS".to_string()),
        capability: cap,
    });
    Some(Box::new(ProcessContext {
        _posture: posture,
        host: ProcessHost,
        capabilities,
        response_arena: Vec::new(),
        effect_trace: Vec::new(),
    }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_host_invocation_v1_destroy(context: *mut c_void) {
    if !context.is_null() {
        // SAFETY: the starter passes exactly the unique pointer returned by
        // init, after the Ken entry has returned and no reply can be retained.
        drop(unsafe { Box::from_raw(context.cast::<ProcessContext>()) });
    }
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

    fn context(directory: &std::path::Path) -> *mut c_void {
        #[cfg(target_os = "linux")]
        let cwd = {
            use std::os::unix::ffi::OsStrExt;
            directory.as_os_str().as_bytes().to_vec()
        };
        #[cfg(not(target_os = "linux"))]
        let cwd = directory.to_string_lossy().as_bytes().to_vec();
        unsafe {
            ken_host_invocation_v1_init(
                cwd.as_ptr(),
                cwd.len(),
                2,
                crate::TARGET_ABI_MANIFEST_HASH.as_ptr(),
                crate::HOST_EFFECT_ABI_V1_HASH.as_ptr(),
            )
        }
    }

    #[test]
    fn manifest_mismatch_fails_before_context_creation() {
        let mut wrong = crate::HOST_EFFECT_ABI_V1_HASH;
        wrong[0] ^= 1;
        let cwd = b".";
        let context = unsafe {
            ken_host_invocation_v1_init(
                cwd.as_ptr(),
                cwd.len(),
                2,
                crate::TARGET_ABI_MANIFEST_HASH.as_ptr(),
                wrong.as_ptr(),
            )
        };
        assert!(context.is_null());
    }

    #[test]
    fn posture_failure_prevents_context_publication() {
        let root = RootPath::new(std::env::current_dir().unwrap()).unwrap();
        assert!(initialize_process_context(root, AUTH_FULL, Err(())).is_none());
    }

    #[test]
    fn wrong_generation_is_capability_denied_before_filesystem_access() {
        let directory =
            std::env::temp_dir().join(format!("ken-px5-malformed-token-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let context = context(&directory);
        assert!(!context.is_null());
        let invocation = NativeInvocationV1 {
            process_input: std::ptr::null(),
            host_context: context.cast(),
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
        unsafe { ken_host_invocation_v1_destroy(context) };
        let _ = std::fs::remove_dir_all(directory);
    }
}
