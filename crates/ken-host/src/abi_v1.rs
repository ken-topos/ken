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
    dispatch_host_op_v1, CanonicalOutcomeV1, CanonicalReplyV1, CanonicalRequestV1, Cap,
    CapabilityGrantV1, CapabilityTableV1, CapabilityTokenV1, ConsoleStreamV1, CreatePolicyV1,
    EffectEventV1, FileErrorCauseV1, FsHandle, FsRootSpec, HostEffectBackendV1, HostOpV1,
    IoErrorIdentityV1, OpenRequest, PathComponent, RootPath, RootedHandle, SymlinkPolicy,
    AUTH_FULL, AUTH_NONE, AUTH_PARTIAL,
};

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn ken_host_abi_v1_establish_sigpipe_ignore() -> std::ffi::c_int;
}

#[derive(Debug)]
struct ProcessPostureV1(());

/// Immutable observation of the process effective UID at startup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveUidSnapshotV1(u32);

impl EffectiveUidSnapshotV1 {
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
    pub fn is_root(self) -> bool {
        self.0 == 0
    }

    /// Constructs a scripted host observation for discriminator tests.
    /// Production runners use `observe_effective_uid_v1` instead.
    #[doc(hidden)]
    pub const fn scripted(raw: u32) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootExecutionDeniedV1;

/// The single pure effective-root admission check shared by both executors.
pub fn admit_root_execution(
    effective_uid: EffectiveUidSnapshotV1,
    allow_root_execution: bool,
) -> Result<(), RootExecutionDeniedV1> {
    if effective_uid.is_root() && !allow_root_execution {
        Err(RootExecutionDeniedV1)
    } else {
        Ok(())
    }
}

/// Reads the v1 privilege predicate once through Ken's audited rustix boundary.
/// This is runtime-trusted and target-validated evidence, never a Ken proof.
pub fn observe_effective_uid_v1() -> Result<EffectiveUidSnapshotV1, ()> {
    #[cfg(test)]
    if let Some(raw) = SCRIPTED_EFFECTIVE_UID_V1.with(std::cell::Cell::get) {
        return Ok(EffectiveUidSnapshotV1::scripted(raw));
    }
    #[cfg(target_os = "linux")]
    {
        Ok(EffectiveUidSnapshotV1(rustix::process::geteuid().as_raw()))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(())
    }
}

#[cfg(test)]
thread_local! {
    static SCRIPTED_EFFECTIVE_UID_V1: std::cell::Cell<Option<u32>> = const {
        std::cell::Cell::new(None)
    };
}

#[cfg(test)]
fn with_scripted_effective_uid_v1<T>(raw: u32, run: impl FnOnce() -> T) -> T {
    SCRIPTED_EFFECTIVE_UID_V1.with(|slot| {
        let previous = slot.replace(Some(raw));
        let result = run();
        slot.set(previous);
        result
    })
}

fn establish_process_posture_v1(
    effective_uid: Result<EffectiveUidSnapshotV1, ()>,
    allow_root_execution: bool,
) -> Result<ProcessPostureV1, PostureErrorV1> {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: this calls Ken's own no-argument C companion. The companion
        // obtains sigaction's layout and constants from the target headers,
        // installs SIG_IGN, retains no pointer, and returns only status.
        if unsafe { ken_host_abi_v1_establish_sigpipe_ignore() } == 0 {
            admit_root_execution(
                effective_uid.map_err(|_| PostureErrorV1::HostPostureUnavailable)?,
                allow_root_execution,
            )
            .map_err(|_| PostureErrorV1::RootExecutionDenied)?;
            Ok(ProcessPostureV1(()))
        } else {
            Err(PostureErrorV1::HostPostureUnavailable)
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (effective_uid, allow_root_execution);
        Err(PostureErrorV1::HostPostureUnavailable)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostureErrorV1 {
    HostPostureUnavailable,
    RootExecutionDenied,
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
struct FsChangeModeRequestV1 {
    capability: u64,
    path: SliceV1,
    mode: u16,
}

#[repr(C)]
struct FsOpenRequestV1 {
    capability: u64,
    path: SliceV1,
    mode: u64,
}

#[repr(C)]
struct ResourceRequestV1 {
    resource: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ResourceErrorReplyV1 {
    schema_version: u64,
    resource_kind: u64,
    identity: u64,
    io: u64,
    required: u64,
    held: u64,
}

#[repr(C)]
struct HostReplyV1 {
    tag: u64,
    detail: u64,
    bytes: SliceV1,
    resource_error: ResourceErrorReplyV1,
}

const REPLY_UNIT: u64 = 0;
const REPLY_BOOL: u64 = 1;
const REPLY_BYTES: u64 = 2;
const REPLY_ERROR: u64 = 3;
const REPLY_RESOURCE: u64 = 4;
const REPLY_METADATA: u64 = 5;
const REPLY_RESOURCE_ERROR: u64 = 6;

struct ProcessContext {
    _posture: ProcessPostureV1,
    host: ProcessHost,
    capabilities: CapabilityTableV1,
    resources: crate::ResourceTableV1,
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

    fn fs_change_mode(
        &mut self,
        grant: &CapabilityGrantV1,
        path: &[u8],
        mode: u16,
    ) -> Result<(), FileErrorCauseV1> {
        let (parent, leaf) = Self::parent(grant, path)?;
        let handle = crate::open_at(&parent, &leaf, OpenRequest::Read).map_err(host_error)?;
        crate::change_mode(&handle, mode).map_err(host_error)
    }

    fn fs_open_resource(
        &mut self,
        grant: &CapabilityGrantV1,
        path: &[u8],
        _mode: crate::FsOpenModeV1,
    ) -> Result<crate::ResourceHandleV1, FileErrorCauseV1> {
        let (parent, leaf) = Self::parent(grant, path)?;
        crate::open_resource_at_v1(&parent, &leaf, OpenRequest::Read).map_err(host_error)
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
    fs_root_tag: u64,
    fs_root: *const u8,
    fs_root_len: usize,
    authority_tag: u64,
    plan_hash: u64,
    allow_root_execution: u64,
    root_denied_exit_status: i64,
    target_abi_hash: *const u8,
    host_effect_abi_hash: *const u8,
    observation_path: *const u8,
    observation_path_len: usize,
    result: *mut HostInitResultV1,
) -> i64 {
    if result.is_null() || !result.is_aligned() {
        return -1;
    }
    let cwd = SliceV1 {
        data: cwd,
        len: cwd_len,
    };
    let fs_root = SliceV1 {
        data: fs_root,
        len: fs_root_len,
    };
    let cwd_bytes;
    let fs_root_bytes;
    // Clear the caller-owned result before any validation or host action.
    unsafe {
        result.write(HostInitResultV1 {
            context: std::ptr::null_mut(),
            capability: 0,
            plan_hash: 0,
        });
        cwd_bytes = borrowed_slice(&cwd);
        fs_root_bytes = borrowed_slice(&fs_root);
    }
    let Some(cwd) = cwd_bytes else {
        return -1;
    };
    let Some(fs_root) = fs_root_bytes else {
        return -1;
    };
    let fs_root_spec = match fs_root_tag {
        0 => FsRootSpec::Absolute(fs_root.to_vec()),
        1 => FsRootSpec::ExecutionStartCwd(fs_root.to_vec()),
        2 => FsRootSpec::EffectiveUserHome(fs_root.to_vec()),
        _ => return -1,
    };
    let Some(authority) = authority(authority_tag) else {
        return -1;
    };
    if plan_hash == 0
        || allow_root_execution > 1
        || target_abi_hash.is_null()
        || host_effect_abi_hash.is_null()
    {
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
    let effective_uid = observe_effective_uid_v1();
    let posture = match establish_process_posture_v1(effective_uid, allow_root_execution != 0) {
        Err(PostureErrorV1::RootExecutionDenied) => {
            if write_startup_terminal_observation_v1(
                observation_path,
                plan_hash,
                root_denied_exit_status,
                crate::TerminalErrorV1::RootExecutionDenied,
            )
            .is_err()
            {
                return -1;
            }
            return 1;
        }
        other => other,
    };
    let context = match initialize_process_context(
        root_path,
        fs_root_spec,
        authority,
        plan_hash,
        observation_path,
        posture,
        effective_uid.ok(),
    ) {
        Ok(context) => context,
        Err(ProcessContextInitError::Home(failure)) => {
            if write_startup_terminal_observation_v1(
                observation_path,
                plan_hash,
                root_denied_exit_status,
                crate::TerminalErrorV1::HomeRootResolutionFailed(failure),
            )
            .is_err()
            {
                return -1;
            }
            return 1;
        }
        Err(ProcessContextInitError::Ordinary) => return -1,
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
    execution_start_cwd: RootPath,
    fs_root_spec: FsRootSpec,
    authority: crate::Authority,
    plan_hash: u64,
    observation_path: &[u8],
    posture: Result<ProcessPostureV1, PostureErrorV1>,
    effective_uid: Option<EffectiveUidSnapshotV1>,
) -> Result<Box<ProcessContext>, ProcessContextInitError> {
    initialize_process_context_with_lookup(
        execution_start_cwd,
        fs_root_spec,
        authority,
        plan_hash,
        observation_path,
        posture,
        effective_uid,
        &crate::account_db_v1::LibcAccountHomeLookupV1,
    )
}

fn initialize_process_context_with_lookup(
    execution_start_cwd: RootPath,
    fs_root_spec: FsRootSpec,
    authority: crate::Authority,
    plan_hash: u64,
    observation_path: &[u8],
    posture: Result<ProcessPostureV1, PostureErrorV1>,
    effective_uid: Option<EffectiveUidSnapshotV1>,
    home_lookup: &impl crate::account_db_v1::AccountHomeLookupV1,
) -> Result<Box<ProcessContext>, ProcessContextInitError> {
    let posture = posture.map_err(|_| ProcessContextInitError::Ordinary)?;
    let Ok(cwd_root) = crate::open_root(&execution_start_cwd) else {
        return Err(ProcessContextInitError::Ordinary);
    };
    let scope = crate::resolve_fs_root_spec_with_lookup_v1(
        &fs_root_spec,
        &cwd_root,
        effective_uid.ok_or(ProcessContextInitError::Ordinary)?,
        crate::rights_for_authority(authority),
        SymlinkPolicy::NoFollow,
        home_lookup,
    )
    .map_err(|error| match error {
        crate::FsRootResolveError::HomeRootResolution(failure) => {
            ProcessContextInitError::Home(failure)
        }
        _ => ProcessContextInitError::Ordinary,
    })?;
    let cap = Cap::mint_scoped(authority, "FS", scope);
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
                .map_err(|_| ProcessContextInitError::Ordinary)?,
        )
    };
    Ok(Box::new(ProcessContext {
        _posture: posture,
        host: ProcessHost,
        capabilities,
        resources: crate::ResourceTableV1::default(),
        response_arena: Vec::new(),
        effect_trace: Vec::new(),
        observation,
        plan_hash,
        capability,
    }))
}

#[derive(Debug)]
enum ProcessContextInitError {
    Ordinary,
    Home(crate::HomeRootResolutionFailureV1),
}

fn write_startup_terminal_observation_v1(
    observation_path: &[u8],
    plan_hash: u64,
    exit_status: i64,
    terminal_error: crate::TerminalErrorV1,
) -> io::Result<()> {
    if observation_path.is_empty() {
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        std::path::PathBuf::from(std::ffi::OsString::from_vec(observation_path.to_vec()))
    };
    #[cfg(not(target_os = "linux"))]
    let path = std::path::PathBuf::from(
        std::str::from_utf8(observation_path)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?,
    );
    let mut sink = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    let bytes = crate::encode_linked_effect_trace_v1(&crate::LinkedEffectTraceV1 {
        plan_hash,
        target_abi_hash: crate::TARGET_ABI_MANIFEST_HASH,
        host_effect_abi_hash: crate::HOST_EFFECT_ABI_V1_HASH,
        terminal_value: exit_status,
        terminal_error: Some(terminal_error),
        effect_trace: Vec::new(),
    })
    .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
    sink.write_all(&bytes)?;
    sink.flush()
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
    context.finalize_resources();
    if let Some(mut sink) = context.observation.take() {
        if write_observation_v1(&mut sink, &context, terminal_value).is_err() {
            return -1;
        }
    }
    0
}

impl ProcessContext {
    fn finalize_resources(&mut self) {
        let settlements = self
            .resources
            .finalize_all_with(|owner| self.host.resource_close(owner));
        for settlement in settlements {
            let outcome = match &settlement.outcome {
                crate::ResourceSettlementOutcomeV1::Released => CanonicalOutcomeV1::Success(
                    CanonicalReplyV1::ResourceSettlement(settlement.clone()),
                ),
                crate::ResourceSettlementOutcomeV1::ReleaseFailed(io) => CanonicalOutcomeV1::Error(
                    crate::SemanticErrorV1::Resource(crate::ResourceErrorV1::ReleaseFailed {
                        schema_version: settlement.schema_version,
                        resource_kind: settlement.resource_kind,
                        identity: settlement.identity,
                        io: *io,
                    }),
                ),
            };
            self.effect_trace.push(EffectEventV1 {
                sequence: self.effect_trace.len() as u64,
                operation: HostOpV1::ResourceRelease,
                capability: None,
                resource: Some(settlement.identity),
                request: CanonicalRequestV1::ResourceRelease,
                outcome,
            });
        }
    }
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
        terminal_error: None,
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
    reply.resource_error = ResourceErrorReplyV1::default();
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
        CanonicalOutcomeV1::Success(CanonicalReplyV1::ResourceAcquired { .. }) => {
            reply.tag = REPLY_RESOURCE;
        }
        CanonicalOutcomeV1::Success(CanonicalReplyV1::ResourceSettlement(_)) => {
            reply.tag = REPLY_UNIT;
        }
        CanonicalOutcomeV1::Success(CanonicalReplyV1::FileMetadata(metadata)) => {
            reply.tag = REPLY_METADATA;
            reply.detail = metadata.size;
            reply.bytes.len = match metadata.kind {
                crate::FsNodeKindV1::File => 0,
                crate::FsNodeKindV1::Directory => 1,
                crate::FsNodeKindV1::Symlink => 2,
                crate::FsNodeKindV1::Other => 3,
            };
        }
        CanonicalOutcomeV1::Error(crate::SemanticErrorV1::Resource(error)) => {
            reply.tag = REPLY_RESOURCE_ERROR;
            match error {
                crate::ResourceErrorV1::Closed => reply.detail = 0,
                crate::ResourceErrorV1::MalformedResource => reply.detail = 1,
                crate::ResourceErrorV1::RightNotHeld { required, held } => {
                    reply.detail = 2;
                    reply.resource_error.schema_version =
                        u64::from(crate::RESOURCE_OBSERVATION_SCHEMA_VERSION_V1);
                    reply.resource_error.required = u64::from(required);
                    reply.resource_error.held = u64::from(held);
                }
                crate::ResourceErrorV1::ReleaseFailed {
                    schema_version,
                    resource_kind,
                    identity,
                    io,
                } => {
                    reply.detail = 3;
                    reply.resource_error.schema_version = u64::from(schema_version);
                    reply.resource_error.resource_kind = match resource_kind {
                        crate::ResourceKindV1::FsHandle => 0,
                    };
                    reply.resource_error.identity = identity.0;
                    reply.resource_error.io = io_error_tag(io);
                }
            }
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
                crate::SemanticErrorV1::Resource(_) => unreachable!("resource errors have a distinct reply tag"),
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

#[cfg(test)]
fn io_error_from_tag(encoded: u64) -> Option<IoErrorIdentityV1> {
    let discriminator = encoded & 0xff;
    if discriminator == 11 {
        if encoded & 0x0000_0000_ffff_ff00 != 0 {
            return None;
        }
        return Some(IoErrorIdentityV1::Other((encoded >> 32) as u32 as i32));
    }
    if encoded >> 8 != 0 {
        return None;
    }
    Some(match discriminator {
        0 => IoErrorIdentityV1::NotFound,
        1 => IoErrorIdentityV1::PermissionDenied,
        3 => IoErrorIdentityV1::BrokenPipe,
        4 => IoErrorIdentityV1::Interrupted,
        5 => IoErrorIdentityV1::AlreadyExists,
        6 => IoErrorIdentityV1::InvalidInput,
        7 => IoErrorIdentityV1::IsDirectory,
        8 => IoErrorIdentityV1::NotDirectory,
        9 => IoErrorIdentityV1::NotEmpty,
        10 => IoErrorIdentityV1::Unsupported,
        _ => return None,
    })
}

/// Canonical fail-closed decoder shared by the projection mutation oracle.
/// The linked consumer emits equivalent guards from the generated layout.
#[cfg(test)]
fn decode_resource_error_reply(
    discriminator: u64,
    payload: ResourceErrorReplyV1,
) -> Option<crate::ResourceErrorV1> {
    let all_zero = payload == ResourceErrorReplyV1::default();
    match discriminator {
        0 if all_zero => Some(crate::ResourceErrorV1::Closed),
        1 if all_zero => Some(crate::ResourceErrorV1::MalformedResource),
        2
            if payload.schema_version
                == u64::from(crate::RESOURCE_OBSERVATION_SCHEMA_VERSION_V1)
                && payload.resource_kind == 0
                && payload.identity == 0
                && payload.io == 0
                && payload.required <= u64::from(u8::MAX)
                && payload.held <= u64::from(u8::MAX) =>
        {
            Some(crate::ResourceErrorV1::RightNotHeld {
                required: payload.required as u8,
                held: payload.held as u8,
            })
        }
        3
            if payload.schema_version
                == u64::from(crate::RESOURCE_OBSERVATION_SCHEMA_VERSION_V1)
                && payload.resource_kind == 0
                && payload.required == 0
                && payload.held == 0 =>
        {
            Some(crate::ResourceErrorV1::ReleaseFailed {
                schema_version: crate::RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
                resource_kind: crate::ResourceKindV1::FsHandle,
                identity: crate::ResourceTraceIdentityV1(payload.identity),
                io: io_error_from_tag(payload.io)?,
            })
        }
        _ => None,
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
    let (capability, resource, request) = match op {
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
            (None, None, request)
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
                None,
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
                None,
                CanonicalRequestV1::FsWriteFile {
                    path: path.to_vec(),
                    create_policy: policy,
                    bytes: bytes.to_vec(),
                },
            )
        }
        HostOpV1::FsChangeMode if request_size == std::mem::size_of::<FsChangeModeRequestV1>() => {
            if !request.cast::<FsChangeModeRequestV1>().is_aligned() {
                return -1;
            }
            let wire = unsafe { &*(request.cast::<FsChangeModeRequestV1>()) };
            let Some(path) = (unsafe { borrowed_slice(&wire.path) }) else {
                return -1;
            };
            if wire.mode & !0o7777 != 0 {
                let outcome = CanonicalOutcomeV1::Error(crate::SemanticErrorV1::File(
                    crate::FileErrorIdentityV1 {
                        operation: HostOpV1::FsChangeMode,
                        relative_path: path.to_vec(),
                        cause: FileErrorCauseV1::Io(IoErrorIdentityV1::InvalidInput),
                    },
                ));
                set_reply(
                    unsafe { &mut *(reply.cast::<HostReplyV1>()) },
                    outcome,
                    context,
                );
                return 0;
            }
            (
                Some(CapabilityTokenV1::from_erased_identity(wire.capability)),
                None,
                CanonicalRequestV1::FsChangeMode {
                    path: path.to_vec(),
                    mode: wire.mode,
                },
            )
        }
        HostOpV1::FsOpen if request_size == std::mem::size_of::<FsOpenRequestV1>() => {
            if !request.cast::<FsOpenRequestV1>().is_aligned() {
                return -1;
            }
            let wire = unsafe { &*(request.cast::<FsOpenRequestV1>()) };
            let Some(path) = (unsafe { borrowed_slice(&wire.path) }) else {
                return -1;
            };
            let mode = match wire.mode {
                0 => crate::FsOpenModeV1::Read,
                1 => crate::FsOpenModeV1::Metadata,
                _ => return -1,
            };
            (
                Some(CapabilityTokenV1::from_erased_identity(wire.capability)),
                None,
                CanonicalRequestV1::FsOpen {
                    path: path.to_vec(),
                    mode,
                },
            )
        }
        HostOpV1::FsHandleMetadata | HostOpV1::ResourceRelease
            if request_size == std::mem::size_of::<ResourceRequestV1>() =>
        {
            if !request.cast::<ResourceRequestV1>().is_aligned() {
                return -1;
            }
            let wire = unsafe { &*(request.cast::<ResourceRequestV1>()) };
            let request = if op == HostOpV1::FsHandleMetadata {
                CanonicalRequestV1::FsHandleMetadata
            } else {
                CanonicalRequestV1::ResourceRelease
            };
            (
                None,
                Some(crate::ResourceTokenV1::from_erased_identity(wire.resource)),
                request,
            )
        }
        _ => return -3,
    };
    let result = dispatch_host_op_v1(
        &mut context.host,
        &context.capabilities,
        &mut context.resources,
        op,
        capability,
        resource,
        &request,
    );
    let Ok(result) = result else {
        return -3;
    };
    context.effect_trace.push(EffectEventV1 {
        sequence: context.effect_trace.len() as u64,
        operation: op,
        capability: result.capability_identity,
        resource: result.resource_identity,
        request,
        outcome: result.outcome.clone(),
    });
    // SAFETY: generated code supplies an aligned writable HostReplyV1 slot.
    let token = result.resource_token;
    set_reply(
        unsafe { &mut *(reply.cast::<HostReplyV1>()) },
        result.outcome,
        context,
    );
    if let Some(token) = token {
        unsafe { &mut *(reply.cast::<HostReplyV1>()) }.detail = token.erased_identity();
    }
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
        size_align!("ResourceTokenV1", crate::ResourceTokenV1);
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
        size_align!("FsChangeModeRequestV1", FsChangeModeRequestV1);
        size_align!("FsOpenRequestV1", FsOpenRequestV1);
        size_align!("ResourceRequestV1", ResourceRequestV1);
        size_align!("ResourceErrorReplyV1", ResourceErrorReplyV1);
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
        offset!("FsChangeModeRequestV1", FsChangeModeRequestV1, capability);
        offset!("FsChangeModeRequestV1", FsChangeModeRequestV1, path);
        offset!("FsChangeModeRequestV1", FsChangeModeRequestV1, mode);
        offset!("FsOpenRequestV1", FsOpenRequestV1, capability);
        offset!("FsOpenRequestV1", FsOpenRequestV1, path);
        offset!("FsOpenRequestV1", FsOpenRequestV1, mode);
        offset!("ResourceRequestV1", ResourceRequestV1, resource);
        offset!("ResourceErrorReplyV1", ResourceErrorReplyV1, schema_version);
        offset!("ResourceErrorReplyV1", ResourceErrorReplyV1, resource_kind);
        offset!("ResourceErrorReplyV1", ResourceErrorReplyV1, identity);
        offset!("ResourceErrorReplyV1", ResourceErrorReplyV1, io);
        offset!("ResourceErrorReplyV1", ResourceErrorReplyV1, required);
        offset!("ResourceErrorReplyV1", ResourceErrorReplyV1, held);
        offset!("HostReplyV1", HostReplyV1, tag);
        offset!("HostReplyV1", HostReplyV1, detail);
        offset!("HostReplyV1", HostReplyV1, bytes);
        offset!("HostReplyV1", HostReplyV1, resource_error);
        assert_eq!(crate::HOST_EFFECT_ABI_V1_CATALOG.len(), 18);
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
        assert_eq!(effect_binding("tag", "reply.resource"), REPLY_RESOURCE);
        assert_eq!(effect_binding("tag", "reply.metadata"), REPLY_METADATA);
        assert_eq!(effect_binding("tag", "reply.resource_error"), REPLY_RESOURCE_ERROR);
        assert_eq!(effect_binding("error", "resource.Closed"), 0);
        assert_eq!(effect_binding("error", "resource.MalformedResource"), 1);
        assert_eq!(effect_binding("error", "resource.RightNotHeld"), 2);
        assert_eq!(effect_binding("error", "resource.ReleaseFailed"), 3);
        assert_eq!(effect_binding("tag", "resource_kind.FsHandle"), 0);
        assert_eq!(effect_binding("lifetime", "resource_error_reply_schema"), 1);
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

    #[test]
    fn resource_error_reply_decoder_is_canonical_and_fail_closed() {
        let zero = ResourceErrorReplyV1::default();
        assert_eq!(
            decode_resource_error_reply(0, zero),
            Some(crate::ResourceErrorV1::Closed)
        );
        assert_eq!(
            decode_resource_error_reply(1, zero),
            Some(crate::ResourceErrorV1::MalformedResource)
        );
        let rights = ResourceErrorReplyV1 {
            schema_version: 1,
            required: 4,
            held: 1,
            ..zero
        };
        assert_eq!(
            decode_resource_error_reply(2, rights),
            Some(crate::ResourceErrorV1::RightNotHeld {
                required: 4,
                held: 1,
            })
        );
        let release = ResourceErrorReplyV1 {
            schema_version: 1,
            resource_kind: 0,
            identity: u64::MAX,
            io: io_error_tag(crate::IoErrorIdentityV1::Other(-5)),
            required: 0,
            held: 0,
        };
        assert_eq!(
            decode_resource_error_reply(3, release),
            Some(crate::ResourceErrorV1::ReleaseFailed {
                schema_version: 1,
                resource_kind: crate::ResourceKindV1::FsHandle,
                identity: crate::ResourceTraceIdentityV1(u64::MAX),
                io: crate::IoErrorIdentityV1::Other(-5),
            })
        );

        for invalid in [
            (4, zero),
            (0, ResourceErrorReplyV1 { schema_version: 1, ..zero }),
            (1, ResourceErrorReplyV1 { identity: 1, ..zero }),
            (2, ResourceErrorReplyV1 { schema_version: 2, ..rights }),
            (2, ResourceErrorReplyV1 { resource_kind: 1, ..rights }),
            (2, ResourceErrorReplyV1 { required: 256, ..rights }),
            (2, ResourceErrorReplyV1 { held: 256, ..rights }),
            (2, ResourceErrorReplyV1 { io: 1, ..rights }),
            (3, ResourceErrorReplyV1 { schema_version: 2, ..release }),
            (3, ResourceErrorReplyV1 { resource_kind: 1, ..release }),
            (3, ResourceErrorReplyV1 { io: 2, ..release }),
            (3, ResourceErrorReplyV1 { io: 0x100, ..release }),
            (3, ResourceErrorReplyV1 { required: 1, ..release }),
            (3, ResourceErrorReplyV1 { held: 1, ..release }),
        ] {
            assert_eq!(decode_resource_error_reply(invalid.0, invalid.1), None);
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn resource_errors_use_the_distinct_fully_initialized_reply_projection() {
        let directory =
            std::env::temp_dir().join(format!("ken-px7f-resource-reply-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let initialized = context(&directory);
        let context = unsafe { &mut *initialized.context.cast::<ProcessContext>() };
        let mut reply = HostReplyV1 {
            tag: u64::MAX,
            detail: u64::MAX,
            bytes: SliceV1 {
                data: std::ptr::dangling(),
                len: usize::MAX,
            },
            resource_error: ResourceErrorReplyV1 {
                schema_version: u64::MAX,
                resource_kind: u64::MAX,
                identity: u64::MAX,
                io: u64::MAX,
                required: u64::MAX,
                held: u64::MAX,
            },
        };

        let mut project = |error| {
            set_reply(
                &mut reply,
                CanonicalOutcomeV1::Error(crate::SemanticErrorV1::Resource(error)),
                context,
            );
            (reply.tag, reply.detail, reply.resource_error)
        };
        assert_eq!(
            project(crate::ResourceErrorV1::Closed),
            (REPLY_RESOURCE_ERROR, 0, ResourceErrorReplyV1::default())
        );
        assert_eq!(
            project(crate::ResourceErrorV1::MalformedResource),
            (REPLY_RESOURCE_ERROR, 1, ResourceErrorReplyV1::default())
        );
        assert_eq!(
            project(crate::ResourceErrorV1::RightNotHeld {
                required: 0x80,
                held: 0x04,
            }),
            (
                REPLY_RESOURCE_ERROR,
                2,
                ResourceErrorReplyV1 {
                    schema_version: 1,
                    required: 0x80,
                    held: 0x04,
                    ..ResourceErrorReplyV1::default()
                },
            )
        );
        assert_eq!(
            project(crate::ResourceErrorV1::ReleaseFailed {
                schema_version: 1,
                resource_kind: crate::ResourceKindV1::FsHandle,
                identity: crate::ResourceTraceIdentityV1(0xfedc_ba98_7654_3210),
                io: crate::IoErrorIdentityV1::Other(-9),
            }),
            (
                REPLY_RESOURCE_ERROR,
                3,
                ResourceErrorReplyV1 {
                    schema_version: 1,
                    resource_kind: 0,
                    identity: 0xfedc_ba98_7654_3210,
                    io: io_error_tag(crate::IoErrorIdentityV1::Other(-9)),
                    required: 0,
                    held: 0,
                },
            )
        );

        set_reply(
            &mut reply,
            CanonicalOutcomeV1::Error(crate::SemanticErrorV1::Io(
                crate::IoErrorIdentityV1::InvalidInput,
            )),
            context,
        );
        assert_eq!(reply.tag, REPLY_ERROR);
        assert_eq!(reply.detail, 6);
        assert_eq!(reply.resource_error, ResourceErrorReplyV1::default());

        unsafe { ken_host_invocation_v1_destroy(initialized.context) };
        std::fs::remove_dir_all(directory).unwrap();
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
                1,
                std::ptr::null(),
                0,
                2,
                1,
                0,
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
                1,
                std::ptr::null(),
                0,
                2,
                1,
                0,
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
                1,
                std::ptr::null(),
                0,
                2,
                0,
                0,
                1,
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
        assert!(initialize_process_context(
            root,
            FsRootSpec::default(),
            AUTH_FULL,
            1,
            &[],
            Err(PostureErrorV1::HostPostureUnavailable),
            Some(EffectiveUidSnapshotV1::scripted(1000)),
        )
        .is_err());
    }

    #[test]
    fn scripted_effective_uid_discriminates_the_native_startup_posture() {
        assert_eq!(
            establish_process_posture_v1(Ok(EffectiveUidSnapshotV1::scripted(0)), false)
                .unwrap_err(),
            PostureErrorV1::RootExecutionDenied
        );
        assert!(
            establish_process_posture_v1(Ok(EffectiveUidSnapshotV1::scripted(0)), true).is_ok()
        );
        assert!(
            establish_process_posture_v1(Ok(EffectiveUidSnapshotV1::scripted(1000)), false).is_ok()
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn root_denial_writer_needs_no_live_process_context() {
        use std::os::unix::ffi::OsStrExt;

        let path =
            std::env::temp_dir().join(format!("ken-px14-root-denied-{}", std::process::id()));
        write_startup_terminal_observation_v1(
            path.as_os_str().as_bytes(),
            17,
            1,
            crate::TerminalErrorV1::RootExecutionDenied,
        )
        .unwrap();
        let trace = crate::decode_linked_effect_trace_v1(&std::fs::read(&path).unwrap()).unwrap();
        let _ = std::fs::remove_file(path);
        assert_eq!(trace.plan_hash, 17);
        assert_eq!(trace.terminal_value, 1);
        assert_eq!(
            trace.terminal_error,
            Some(crate::TerminalErrorV1::RootExecutionDenied)
        );
        assert!(trace.effect_trace.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn scripted_home_failures_use_real_init_and_pre_context_terminal_writer() {
        use std::os::unix::ffi::OsStrExt;

        struct FailingLookup(crate::HomeRootResolutionFailureV1);
        impl crate::account_db_v1::AccountHomeLookupV1 for FailingLookup {
            fn resolve_effective_user_home(
                &self,
                uid: EffectiveUidSnapshotV1,
            ) -> Result<Vec<u8>, crate::HomeRootResolutionFailureV1> {
                assert_eq!(uid, EffectiveUidSnapshotV1::scripted(1000));
                Err(self.0.clone())
            }
        }

        let cwd = RootPath::new(std::env::current_dir().unwrap()).unwrap();
        for (index, failure) in [
            crate::HomeRootResolutionFailureV1::NoAccountRecord,
            crate::HomeRootResolutionFailureV1::AccountRecordTooLarge,
            crate::HomeRootResolutionFailureV1::AccountLookup(
                crate::IoErrorIdentityV1::PermissionDenied,
            ),
            crate::HomeRootResolutionFailureV1::InvalidAccountRecord,
            crate::HomeRootResolutionFailureV1::RootOpen(crate::IoErrorIdentityV1::NotFound),
            crate::HomeRootResolutionFailureV1::ScopeEscape,
            crate::HomeRootResolutionFailureV1::SymlinkDenied,
        ]
        .into_iter()
        .enumerate()
        {
            let error = match initialize_process_context_with_lookup(
                cwd.clone(),
                FsRootSpec::EffectiveUserHome(b"data".to_vec()),
                AUTH_FULL,
                31,
                &[],
                establish_process_posture_v1(Ok(EffectiveUidSnapshotV1::scripted(1000)), false),
                Some(EffectiveUidSnapshotV1::scripted(1000)),
                &FailingLookup(failure.clone()),
            ) {
                Err(error) => error,
                Ok(_) => panic!("scripted home failure unexpectedly initialized a context"),
            };
            assert!(matches!(
                error,
                ProcessContextInitError::Home(ref actual) if actual == &failure
            ));
            let path = std::env::temp_dir().join(format!(
                "ken-px16-home-failure-{}-{index}",
                std::process::id()
            ));
            write_startup_terminal_observation_v1(
                path.as_os_str().as_bytes(),
                31,
                1,
                crate::TerminalErrorV1::HomeRootResolutionFailed(failure.clone()),
            )
            .unwrap();
            let trace =
                crate::decode_linked_effect_trace_v1(&std::fs::read(&path).unwrap()).unwrap();
            std::fs::remove_file(path).unwrap();
            assert_eq!(trace.plan_hash, 31);
            assert_eq!(trace.terminal_value, 1);
            assert_eq!(
                trace.terminal_error,
                Some(crate::TerminalErrorV1::HomeRootResolutionFailed(failure))
            );
            assert!(trace.effect_trace.is_empty());
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn scripted_home_success_mints_only_after_one_real_resolution() {
        use std::cell::Cell;
        use std::os::unix::ffi::OsStrExt;
        use std::rc::Rc;

        struct Lookup {
            uid: EffectiveUidSnapshotV1,
            home: Vec<u8>,
            calls: Rc<Cell<usize>>,
        }
        impl crate::account_db_v1::AccountHomeLookupV1 for Lookup {
            fn resolve_effective_user_home(
                &self,
                uid: EffectiveUidSnapshotV1,
            ) -> Result<Vec<u8>, crate::HomeRootResolutionFailureV1> {
                assert_eq!(uid, self.uid);
                self.calls.set(self.calls.get() + 1);
                Ok(self.home.clone())
            }
        }

        let parent =
            std::env::temp_dir().join(format!("ken-px16-init-success-{}", std::process::id()));
        let cwd = parent.join("cwd");
        let home = parent.join("account-home");
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(home.join("data")).unwrap();
        std::fs::write(home.join("data/value"), b"resolved").unwrap();
        let uid = EffectiveUidSnapshotV1::scripted(1000);
        let calls = Rc::new(Cell::new(0));
        let context = initialize_process_context_with_lookup(
            RootPath::new(&cwd).unwrap(),
            FsRootSpec::EffectiveUserHome(b"data".to_vec()),
            AUTH_FULL,
            41,
            &[],
            establish_process_posture_v1(Ok(uid), false),
            Some(uid),
            &Lookup {
                uid,
                home: home.as_os_str().as_bytes().to_vec(),
                calls: calls.clone(),
            },
        )
        .unwrap();
        assert_eq!(calls.get(), 1);
        let grant = context.capabilities.resolve(context.capability).unwrap();
        let crate::FsHandle::Posix(root) = &grant.capability.scope().root else {
            panic!("home capability must retain only a descriptor")
        };
        let value = crate::open_at(
            root,
            &PathComponent::new(b"value").unwrap(),
            OpenRequest::Read,
        )
        .unwrap();
        assert_eq!(crate::read(&value).unwrap(), b"resolved");
        drop(context);
        std::fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn full_native_init_uses_only_the_scripted_observer_seam() {
        use std::os::unix::ffi::OsStrExt;

        let cwd = b".";
        let observation_path = std::env::temp_dir().join(format!(
            "ken-px14-native-init-denied-{}",
            std::process::id()
        ));
        let observation_bytes = observation_path.as_os_str().as_bytes();
        let mut denied = HostInitResultV1 {
            context: usize::MAX as *mut c_void,
            capability: u64::MAX,
            plan_hash: u64::MAX,
        };
        let status = with_scripted_effective_uid_v1(0, || unsafe {
            ken_host_invocation_v1_init(
                cwd.as_ptr(),
                cwd.len(),
                1,
                std::ptr::null(),
                0,
                2,
                19,
                0,
                1,
                crate::TARGET_ABI_MANIFEST_HASH.as_ptr(),
                crate::HOST_EFFECT_ABI_V1_HASH.as_ptr(),
                observation_bytes.as_ptr(),
                observation_bytes.len(),
                &mut denied,
            )
        });
        assert_eq!(status, 1);
        assert!(denied.context.is_null());
        assert_eq!(denied.capability, 0);
        assert_eq!(denied.plan_hash, 0);
        let trace =
            crate::decode_linked_effect_trace_v1(&std::fs::read(&observation_path).unwrap())
                .unwrap();
        let _ = std::fs::remove_file(observation_path);
        assert_eq!(
            trace.terminal_error,
            Some(crate::TerminalErrorV1::RootExecutionDenied)
        );
        assert!(trace.effect_trace.is_empty());

        let mut allowed = HostInitResultV1 {
            context: std::ptr::null_mut(),
            capability: 0,
            plan_hash: 0,
        };
        let status = with_scripted_effective_uid_v1(0, || unsafe {
            ken_host_invocation_v1_init(
                cwd.as_ptr(),
                cwd.len(),
                1,
                std::ptr::null(),
                0,
                2,
                19,
                1,
                1,
                crate::TARGET_ABI_MANIFEST_HASH.as_ptr(),
                crate::HOST_EFFECT_ABI_V1_HASH.as_ptr(),
                std::ptr::null(),
                0,
                &mut allowed,
            )
        });
        assert_eq!(status, 0);
        assert!(!allowed.context.is_null());
        unsafe { ken_host_invocation_v1_destroy(allowed.context) };
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
            resource_error: ResourceErrorReplyV1::default(),
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

    #[test]
    fn change_mode_rejects_file_type_bits_before_dispatch() {
        let directory =
            std::env::temp_dir().join(format!("ken-px13-invalid-mode-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("mode.bin"), b"retained").unwrap();
        let initialized = context(&directory);
        assert!(!initialized.context.is_null());
        let invocation = NativeInvocationV1 {
            process_input: std::ptr::null(),
            host_context: initialized.context.cast(),
            capability: initialized.capability,
        };
        let path = b"mode.bin";
        let request = FsChangeModeRequestV1 {
            capability: initialized.capability,
            path: SliceV1 {
                data: path.as_ptr(),
                len: path.len(),
            },
            mode: 0o10000,
        };
        let mut reply = HostReplyV1 {
            tag: u64::MAX,
            detail: u64::MAX,
            bytes: SliceV1 {
                data: std::ptr::null(),
                len: 0,
            },
            resource_error: ResourceErrorReplyV1::default(),
        };
        let status = unsafe {
            ken_host_dispatch_v1(
                (&invocation as *const NativeInvocationV1).cast(),
                HostOpV1::FsChangeMode as u64,
                (&request as *const FsChangeModeRequestV1).cast(),
                std::mem::size_of::<FsChangeModeRequestV1>(),
                (&mut reply as *mut HostReplyV1).cast(),
            )
        };
        assert_eq!(status, 0);
        assert_eq!(reply.tag, REPLY_ERROR);
        assert_eq!(reply.detail, io_error_tag(IoErrorIdentityV1::InvalidInput));
        let context = unsafe { &*initialized.context.cast::<ProcessContext>() };
        assert!(context.effect_trace.is_empty());
        assert_eq!(
            std::fs::read(directory.join("mode.bin")).unwrap(),
            b"retained"
        );
        unsafe { ken_host_invocation_v1_destroy(initialized.context) };
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn native_resource_open_metadata_release_and_stale_use_share_one_real_context() {
        let directory =
            std::env::temp_dir().join(format!("ken-px7r-native-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("held.bin"), b"held-resource").unwrap();
        let initialized = context(&directory);
        let invocation = NativeInvocationV1 {
            process_input: std::ptr::null(),
            host_context: initialized.context.cast(),
            capability: initialized.capability,
        };
        let path = b"held.bin";
        let open = FsOpenRequestV1 {
            capability: initialized.capability,
            path: SliceV1 {
                data: path.as_ptr(),
                len: path.len(),
            },
            mode: 1,
        };
        let mut reply = HostReplyV1 {
            tag: u64::MAX,
            detail: u64::MAX,
            bytes: SliceV1 {
                data: std::ptr::null(),
                len: 0,
            },
            resource_error: ResourceErrorReplyV1::default(),
        };
        let status = unsafe {
            ken_host_dispatch_v1(
                (&invocation as *const NativeInvocationV1).cast(),
                HostOpV1::FsOpen as u64,
                (&open as *const FsOpenRequestV1).cast(),
                std::mem::size_of::<FsOpenRequestV1>(),
                (&mut reply as *mut HostReplyV1).cast(),
            )
        };
        assert_eq!(status, 0);
        assert_eq!(reply.tag, REPLY_RESOURCE);
        let resource = reply.detail;

        let request = ResourceRequestV1 { resource };
        let dispatch_resource = |operation, reply: &mut HostReplyV1| unsafe {
            ken_host_dispatch_v1(
                (&invocation as *const NativeInvocationV1).cast(),
                operation as u64,
                (&request as *const ResourceRequestV1).cast(),
                std::mem::size_of::<ResourceRequestV1>(),
                (reply as *mut HostReplyV1).cast(),
            )
        };
        assert_eq!(dispatch_resource(HostOpV1::FsHandleMetadata, &mut reply), 0);
        assert_eq!(reply.tag, REPLY_METADATA);
        assert_eq!(reply.detail, 13);
        assert_eq!(dispatch_resource(HostOpV1::ResourceRelease, &mut reply), 0);
        assert_eq!(reply.tag, REPLY_UNIT);
        assert_eq!(dispatch_resource(HostOpV1::FsHandleMetadata, &mut reply), 0);
        assert_eq!(reply.tag, REPLY_RESOURCE_ERROR);
        assert_eq!(reply.detail, 0);
        assert_eq!(reply.resource_error, ResourceErrorReplyV1::default());

        let context = unsafe { &*initialized.context.cast::<ProcessContext>() };
        assert_eq!(context.effect_trace.len(), 4);
        let identities = context
            .effect_trace
            .iter()
            .filter_map(|event| event.resource)
            .collect::<Vec<_>>();
        assert_eq!(identities, vec![crate::ResourceTraceIdentityV1(1); 4]);
        assert!(matches!(
            context.effect_trace[3].outcome,
            CanonicalOutcomeV1::Error(crate::SemanticErrorV1::Resource(
                crate::ResourceErrorV1::Closed
            ))
        ));
        unsafe { ken_host_invocation_v1_destroy(initialized.context) };
        std::fs::remove_dir_all(directory).unwrap();
    }
}
