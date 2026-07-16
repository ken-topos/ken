//! Audited host-ABI boundary for Ken's Linux runtime.
//!
//! This crate is a tested and target-validated TCB extension around `rustix`;
//! its host guarantees are never Ken proofs. `rustix` and raw descriptors stay
//! private. The kernel is unaffected and retains its own unsafe-code ban.
//! PX14 also snapshots `rustix::process::geteuid` once at startup; that root
//! posture is discriminator-tested runtime evidence, never a confinement proof.
//! The generated target manifest dual-sources the numeric filesystem ABI from
//! `linux-raw-sys` and a target-qualified system-header observer. A mismatch
//! fails the build closed. This is tested/validated host evidence, never a Ken
//! proof.
//!
//! Ken's sole supported entrypoint is the standard-Rust `ken` binary. Rust's
//! standard runtime ignores SIGPIPE before `main`, including in Rust test
//! binaries, so console writes surface a broken pipe as an I/O error. Ken does
//! not support `cdylib`, `staticlib`, C embedding, or a `#[unix_sigpipe]`
//! opt-out. The supported produced linked artifact is C-started, so its private
//! `abi_v1` host context re-establishes the same process-lifetime posture before
//! calling Ken; no general C embedding API is exposed.

#![deny(unsafe_code)]

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod abi_v1;
pub mod capability;
mod effect_v1;
mod effect_wire_v1;

pub use abi_v1::{
    EffectiveUidSnapshotV1, RootExecutionDeniedV1, admit_root_execution, observe_effective_uid_v1,
};
pub use capability::*;
pub use effect_v1::*;
pub use effect_wire_v1::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DependencyIdentity {
    pub name: &'static str,
    pub version: &'static str,
    pub checksum: &'static str,
    pub features: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbiFact {
    pub name: &'static str,
    pub value: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetAbi {
    pub schema_version: u32,
    pub target: &'static str,
    pub target_os: &'static str,
    pub backend: &'static str,
    pub dependencies: &'static [DependencyIdentity],
    pub fact_count: usize,
    pub facts: &'static [AbiFact],
    pub manifest_hash: [u8; 32],
}

include!(concat!(env!("OUT_DIR"), "/target_abi.rs"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetAbiIdentityError {
    BackendUnavailable,
    HashMismatch,
}

impl std::fmt::Display for TargetAbiIdentityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendUnavailable => formatter.write_str("target ABI backend is unavailable"),
            Self::HashMismatch => formatter.write_str("target ABI manifest hash mismatch"),
        }
    }
}

impl std::error::Error for TargetAbiIdentityError {}

/// Validates an artifact's compiled-in ABI identity before it enters the host
/// boundary. An unavailable or mismatched identity always fails closed.
pub fn assert_target_abi_identity(artifact_hash: [u8; 32]) -> Result<(), TargetAbiIdentityError> {
    if TARGET_ABI.backend != "linux_raw" {
        return Err(TargetAbiIdentityError::BackendUnavailable);
    }
    if artifact_hash != TARGET_ABI_MANIFEST_HASH {
        return Err(TargetAbiIdentityError::HashMismatch);
    }
    Ok(())
}

fn assert_current_target_abi() -> HostResult<()> {
    assert_target_abi_identity(TARGET_ABI_MANIFEST_HASH)
        .map_err(|error| io::Error::new(io::ErrorKind::Unsupported, error.to_string()).into())
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use rustix::fd::OwnedFd;
    use rustix::fs::{self, AtFlags, Mode, OFlags};

    #[derive(Debug)]
    pub(super) struct Handle(pub(super) Arc<OwnedFd>);

    impl Clone for Handle {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    fn file(handle: &Handle) -> io::Result<File> {
        Ok(File::from(handle.0.as_ref().try_clone()?))
    }

    pub(super) fn open_root(path: &RootPath) -> io::Result<Handle> {
        Ok(Handle(Arc::new(File::open(path.as_path())?.into())))
    }

    pub(super) fn open_at(
        parent: &Handle,
        leaf: &PathComponent,
        request: OpenRequest,
    ) -> io::Result<Handle> {
        let flags = match request {
            OpenRequest::Read => OFlags::RDONLY | OFlags::NOFOLLOW,
            OpenRequest::ReadDirectory => OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW,
            OpenRequest::ReadWrite => OFlags::RDWR | OFlags::NOFOLLOW,
            OpenRequest::CreateNew => {
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW
            }
            OpenRequest::CreateOrTruncate => {
                OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC | OFlags::NOFOLLOW
            }
            OpenRequest::CreateOrKeep => {
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW
            }
            OpenRequest::AppendOrCreate => {
                OFlags::WRONLY | OFlags::CREATE | OFlags::APPEND | OFlags::NOFOLLOW
            }
        } | OFlags::CLOEXEC;
        fs::openat(
            &*parent.0,
            leaf.as_bytes(),
            flags,
            Mode::from_raw_mode(0o666),
        )
        .map(|fd| Handle(Arc::new(fd)))
        .map_err(io::Error::from)
    }

    pub(super) fn readlink_at(parent: &Handle, leaf: &PathComponent) -> io::Result<Vec<u8>> {
        fs::readlinkat(&*parent.0, leaf.as_bytes(), Vec::new())
            .map(|path| path.into_bytes())
            .map_err(io::Error::from)
    }

    pub(super) fn metadata(handle: &Handle) -> io::Result<Metadata> {
        use std::os::unix::fs::{FileTypeExt, MetadataExt};
        let metadata = file(handle)?.metadata()?;
        let file_type = metadata.file_type();
        let kind = if file_type.is_file() {
            FileKind::File
        } else if file_type.is_dir() {
            FileKind::Directory
        } else if file_type.is_symlink() {
            FileKind::Symlink
        } else if file_type.is_socket()
            || file_type.is_fifo()
            || file_type.is_block_device()
            || file_type.is_char_device()
        {
            FileKind::Other
        } else {
            FileKind::Other
        };
        Ok(Metadata {
            size: metadata.len(),
            kind,
            mode: matches!(kind, FileKind::File | FileKind::Directory)
                .then_some((metadata.mode() & 0o7777) as u16),
            identity: FileIdentity {
                device: metadata.dev(),
                inode: metadata.ino(),
            },
        })
    }

    pub(super) fn read(handle: &Handle) -> io::Result<Vec<u8>> {
        let mut file = file(handle)?;
        file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub(super) fn replace(handle: &Handle, bytes: &[u8]) -> io::Result<()> {
        let mut file = file(handle)?;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(bytes)?;
        file.sync_all()
    }

    pub(super) fn append(handle: &Handle, bytes: &[u8]) -> io::Result<()> {
        let mut file = file(handle)?;
        file.seek(SeekFrom::End(0))?;
        file.write_all(bytes)
    }

    pub(super) fn write_new(handle: &Handle, bytes: &[u8]) -> io::Result<()> {
        let mut file = file(handle)?;
        file.write_all(bytes)?;
        file.sync_all()
    }

    pub(super) fn change_mode(handle: &Handle, mode: u16) -> io::Result<()> {
        fs::fchmod(&*handle.0, Mode::from_bits_retain(u32::from(mode))).map_err(io::Error::from)
    }

    pub(super) fn read_directory(handle: &Handle) -> io::Result<Vec<DirectoryEntry>> {
        use std::os::fd::AsRawFd;
        use std::os::unix::ffi::OsStringExt;
        let path = PathBuf::from(format!("/proc/self/fd/{}", handle.0.as_raw_fd()));
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let kind = if entry.file_type()?.is_file() {
                FileKind::File
            } else if entry.file_type()?.is_dir() {
                FileKind::Directory
            } else if entry.file_type()?.is_symlink() {
                FileKind::Symlink
            } else {
                FileKind::Other
            };
            entries.push(DirectoryEntry {
                name: entry.file_name().into_vec(),
                kind,
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }

    pub(super) fn create_directory(parent: &Handle, leaf: &PathComponent) -> io::Result<()> {
        fs::mkdirat(&*parent.0, leaf.as_bytes(), Mode::from_raw_mode(0o777))
            .map_err(io::Error::from)
    }

    pub(super) fn remove(
        parent: &Handle,
        leaf: &PathComponent,
        kind: RemoveKind,
    ) -> io::Result<()> {
        let flags = match kind {
            RemoveKind::File => AtFlags::empty(),
            RemoveKind::Directory => AtFlags::REMOVEDIR,
        };
        fs::unlinkat(&*parent.0, leaf.as_bytes(), flags).map_err(io::Error::from)
    }

    pub(super) fn remove_directory_tree(parent: &Handle, leaf: &PathComponent) -> io::Result<()> {
        use std::os::fd::AsRawFd;
        use std::os::unix::ffi::OsStrExt;
        let mut target = PathBuf::from(format!("/proc/self/fd/{}", parent.0.as_raw_fd()));
        target.push(std::ffi::OsStr::from_bytes(leaf.as_bytes()));
        std::fs::remove_dir_all(target)
    }

    pub(super) fn rename(
        from_parent: &Handle,
        from_leaf: &PathComponent,
        to_parent: &Handle,
        to_leaf: &PathComponent,
    ) -> io::Result<()> {
        fs::renameat(
            &*from_parent.0,
            from_leaf.as_bytes(),
            &*to_parent.0,
            to_leaf.as_bytes(),
        )
        .map_err(io::Error::from)
    }
}

/// An opaque host-owned descriptor rooted at an authorized filesystem node.
#[derive(Clone)]
pub struct RootedHandle {
    #[cfg(target_os = "linux")]
    inner: linux::Handle,
}

impl std::fmt::Debug for RootedHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("RootedHandle(..)")
    }
}

impl PartialEq for RootedHandle {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(target_os = "linux")]
        {
            Arc::ptr_eq(&self.inner.0, &other.inner.0)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = other;
            true
        }
    }
}

impl Eq for RootedHandle {}

/// A validated process-provided path used only to choose a capability root.
#[derive(Clone, Debug)]
pub struct RootPath(PathBuf);

impl RootPath {
    pub fn new(path: impl AsRef<Path>) -> HostResult<Self> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
        }
        Ok(Self(path.to_path_buf()))
    }

    fn as_path(&self) -> &Path {
        &self.0
    }
}

/// A nonempty, non-dot, slash-free, NUL-free path component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathComponent(Vec<u8>);

impl PathComponent {
    pub fn new(bytes: &[u8]) -> HostResult<Self> {
        if bytes.is_empty()
            || bytes == b"."
            || bytes == b".."
            || bytes.contains(&b'/')
            || bytes.contains(&0)
        {
            return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
        }
        Ok(Self(bytes.to_vec()))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenRequest {
    Read,
    ReadDirectory,
    ReadWrite,
    CreateNew,
    CreateOrTruncate,
    CreateOrKeep,
    AppendOrCreate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoveKind {
    File,
    Directory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileIdentity {
    pub device: u64,
    pub inode: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub size: u64,
    pub kind: FileKind,
    pub mode: Option<u16>,
    pub identity: FileIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub name: Vec<u8>,
    pub kind: FileKind,
}

/// A semantic host failure. Raw errno values and backend errors stay private.
#[derive(Debug)]
pub struct HostError(io::Error);

impl HostError {
    pub fn kind(&self) -> io::ErrorKind {
        self.0.kind()
    }

    pub fn into_io_error(self) -> io::Error {
        self.0
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for HostError {}

impl From<io::Error> for HostError {
    fn from(error: io::Error) -> Self {
        Self(error)
    }
}

impl From<HostError> for io::Error {
    fn from(error: HostError) -> Self {
        error.into_io_error()
    }
}

pub type HostResult<T> = Result<T, HostError>;

#[cfg(not(target_os = "linux"))]
fn unsupported<T>() -> HostResult<T> {
    Err(io::Error::from(io::ErrorKind::Unsupported).into())
}

pub fn open_root(path: &RootPath) -> HostResult<RootedHandle> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::open_root(path)
            .map(|inner| RootedHandle { inner })
            .map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        unsupported()
    }
}

#[derive(Debug)]
pub enum FsRootResolveError {
    ScopeEscape,
    SymlinkDenied,
    Io(io::Error),
}

impl From<HostError> for FsRootResolveError {
    fn from(error: HostError) -> Self {
        Self::Io(error.into_io_error())
    }
}

/// Resolve a checked root specification exactly once at capability-table init.
///
/// Both executors call this function. The returned scope owns only resolved
/// handles and identities; neither the cwd spelling nor the root specification
/// survives into the operation path or canonical observations.
pub fn resolve_fs_root_spec_v1(
    spec: &FsRootSpec,
    execution_start_cwd: &RootedHandle,
    rights: RightSet,
    symlink: SymlinkPolicy,
) -> Result<FsScope, FsRootResolveError> {
    #[cfg(target_os = "linux")]
    {
        let (mut handle, suffix) = match spec {
            FsRootSpec::Absolute(bytes) => {
                use std::os::unix::ffi::OsStringExt;
                let path = PathBuf::from(std::ffi::OsString::from_vec(bytes.clone()));
                let path = RootPath::new(path)?;
                (open_root(&path)?, &[][..])
            }
            FsRootSpec::ExecutionStartCwd(suffix) => {
                (execution_start_cwd.clone(), suffix.as_slice())
            }
        };
        let root_metadata = metadata(&handle)?;
        let mut lineage = vec![FsIdentity::Posix {
            device: root_metadata.identity.device,
            inode: root_metadata.identity.inode,
        }];
        for component in suffix.split(|byte| *byte == b'/') {
            if component.is_empty() || component == b"." {
                continue;
            }
            if component == b".." {
                return Err(FsRootResolveError::ScopeEscape);
            }
            let component = PathComponent::new(component)?;
            match open_at(&handle, &component, OpenRequest::ReadDirectory) {
                Ok(next) => handle = next,
                Err(error) if readlink_at(&handle, &component).is_ok() => {
                    let _ = error;
                    return Err(FsRootResolveError::SymlinkDenied);
                }
                Err(error) => return Err(error.into()),
            }
            let metadata = metadata(&handle)?;
            lineage.push(FsIdentity::Posix {
                device: metadata.identity.device,
                inode: metadata.identity.inode,
            });
        }
        Ok(FsScope {
            rights,
            root: FsHandle::Posix(handle),
            lineage,
            symlink,
            empty: false,
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (spec, execution_start_cwd, rights, symlink);
        Err(FsRootResolveError::Io(io::Error::from(
            io::ErrorKind::Unsupported,
        )))
    }
}

pub fn open_at(
    parent: &RootedHandle,
    leaf: &PathComponent,
    request: OpenRequest,
) -> HostResult<RootedHandle> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::open_at(&parent.inner, leaf, request)
            .map(|inner| RootedHandle { inner })
            .map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (parent, leaf, request);
        unsupported()
    }
}

pub fn readlink_at(parent: &RootedHandle, leaf: &PathComponent) -> HostResult<Vec<u8>> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::readlink_at(&parent.inner, leaf).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (parent, leaf);
        unsupported()
    }
}

macro_rules! handle_op {
    ($name:ident, $result:ty) => {
        pub fn $name(handle: &RootedHandle) -> HostResult<$result> {
            assert_current_target_abi()?;
            #[cfg(target_os = "linux")]
            {
                linux::$name(&handle.inner).map_err(Into::into)
            }
            #[cfg(not(target_os = "linux"))]
            {
                let _ = handle;
                unsupported()
            }
        }
    };
}

handle_op!(metadata, Metadata);
handle_op!(read, Vec<u8>);
handle_op!(read_directory, Vec<DirectoryEntry>);

pub fn replace(handle: &RootedHandle, bytes: &[u8]) -> HostResult<()> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::replace(&handle.inner, bytes).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (handle, bytes);
        unsupported()
    }
}

pub fn append(handle: &RootedHandle, bytes: &[u8]) -> HostResult<()> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::append(&handle.inner, bytes).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (handle, bytes);
        unsupported()
    }
}

pub fn write_new(handle: &RootedHandle, bytes: &[u8]) -> HostResult<()> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::write_new(&handle.inner, bytes).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (handle, bytes);
        unsupported()
    }
}

/// Changes only permission/set-id/sticky bits on an already-authorized handle.
pub fn change_mode(handle: &RootedHandle, mode: u16) -> HostResult<()> {
    assert_current_target_abi()?;
    if mode & !0o7777 != 0 {
        return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
    }
    #[cfg(target_os = "linux")]
    {
        linux::change_mode(&handle.inner, mode).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (handle, mode);
        unsupported()
    }
}

pub fn create_directory(parent: &RootedHandle, leaf: &PathComponent) -> HostResult<()> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::create_directory(&parent.inner, leaf).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (parent, leaf);
        unsupported()
    }
}

pub fn remove(parent: &RootedHandle, leaf: &PathComponent, kind: RemoveKind) -> HostResult<()> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::remove(&parent.inner, leaf, kind).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (parent, leaf, kind);
        unsupported()
    }
}

pub fn remove_directory_tree(parent: &RootedHandle, leaf: &PathComponent) -> HostResult<()> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::remove_directory_tree(&parent.inner, leaf).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (parent, leaf);
        unsupported()
    }
}

pub fn rename(
    from_parent: &RootedHandle,
    from_leaf: &PathComponent,
    to_parent: &RootedHandle,
    to_leaf: &PathComponent,
) -> HostResult<()> {
    assert_current_target_abi()?;
    #[cfg(target_os = "linux")]
    {
        linux::rename(&from_parent.inner, from_leaf, &to_parent.inner, to_leaf).map_err(Into::into)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (from_parent, from_leaf, to_parent, to_leaf);
        unsupported()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    mod build_support {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/build_support.rs"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn generated_manifest_is_closed_and_probe_comparison_discriminates() {
        assert_eq!(TARGET_ABI.fact_count, 23);
        assert_eq!(TARGET_ABI.fact_count, TARGET_ABI.facts.len());
        assert_eq!(TARGET_ABI.dependencies.len(), 3);
        assert_eq!(
            TARGET_ABI
                .dependencies
                .iter()
                .map(|dependency| (dependency.name, dependency.version, dependency.features))
                .collect::<Vec<_>>(),
            vec![
                ("rustix", "1.1.4", &["std", "fs", "process"][..]),
                ("bitflags", "2.13.0", &[][..]),
                ("linux-raw-sys", "0.12.1", &["std", "general", "errno"][..]),
            ]
        );
        assert!(
            TARGET_ABI
                .dependencies
                .iter()
                .all(|dependency| dependency.checksum.len() == 64)
        );
        assert_eq!(TARGET_ABI.backend, "linux_raw");
        assert!(!TARGET_ABI_CANONICAL.contains("SIG"));

        let expected = TARGET_ABI
            .facts
            .iter()
            .map(|fact| (fact.name, fact.value))
            .collect::<Vec<_>>();
        let protocol = TARGET_ABI
            .facts
            .iter()
            .map(|fact| format!("{}={}\n", fact.name, fact.value))
            .collect::<String>();
        let observed = build_support::parse_probe(&protocol).expect("parse true probe output");
        build_support::verify_probe(&expected, &observed).expect("true values agree");

        let mut tampered = expected.clone();
        tampered
            .iter_mut()
            .find(|(name, _)| *name == "O_RDONLY")
            .expect("O_RDONLY is manifested")
            .1 ^= 1;
        let mismatch = build_support::verify_probe(&tampered, &observed)
            .expect_err("tampered linux-raw-sys value must fail closed");
        assert!(mismatch.contains("O_RDONLY"));

        for width in ["POINTER_WIDTH", "C_INT_WIDTH"] {
            let mut tampered = expected.clone();
            let (_, value) = tampered
                .iter_mut()
                .find(|(name, _)| *name == width)
                .expect("width fact is manifested");
            *value ^= 1;
            let mismatch = build_support::verify_probe(&tampered, &observed)
                .expect_err("tampered width producer must fail closed");
            assert!(mismatch.contains(width));
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn producer_inventory_is_bidirectional_and_sync_drift_is_discriminating() {
        let build = include_str!("../build.rs");
        let host = include_str!("lib.rs");
        let consumer = include_str!("../../ken-interp/src/eval.rs");
        let probe = include_str!("../abi_probe.c");
        let facts = TARGET_ABI
            .facts
            .iter()
            .map(|fact| (fact.name, fact.value))
            .collect::<Vec<_>>();

        build_support::verify_inventory_closure(build, host, consumer, probe, &facts)
            .expect("current 22-member producer is exactly manifested");

        let injected_host = host.replacen(
            "} | OFlags::CLOEXEC;",
            "} | OFlags::CLOEXEC | OFlags::SYNC;",
            1,
        );
        let error =
            build_support::verify_inventory_closure(build, &injected_host, consumer, probe, &facts)
                .expect_err("an unregistered production OFlags variant must fail closed");
        assert_eq!(error, "unmanifested producer ABI fact: OFlags::SYNC");

        let mut restored_facts = facts.clone();
        restored_facts.push(("O_SYNC", linux_raw_sys::general::O_SYNC.into()));
        let restored_probe = probe.replacen(
            "    return 0;",
            "    printf(\"O_SYNC=%lld\\n\", (long long)O_SYNC);\n    return 0;",
            1,
        );
        build_support::verify_inventory_closure(
            build,
            &injected_host,
            consumer,
            &restored_probe,
            &restored_facts,
        )
        .expect("linux-raw-sys registration plus matching observer restores closure");

        let injected_build = build.replacen(
            "        width_fact(\"POINTER_WIDTH\", bit_width::<usize>()),",
            "        width_fact(\"C_LONG_WIDTH\", bit_width::<core::ffi::c_long>()),\n        width_fact(\"POINTER_WIDTH\", bit_width::<usize>()),",
            1,
        );
        let producer_only =
            build_support::verify_inventory_closure(&injected_build, host, consumer, probe, &facts)
                .expect_err("a producer-only width fact must fail closed");
        assert_eq!(
            producer_only,
            "unmanifested producer ABI fact: ABI width::C_LONG_WIDTH"
        );

        let mut registry_only_facts = facts;
        registry_only_facts.push(("C_LONG_WIDTH", 64));
        let registry_only = build_support::verify_inventory_closure(
            build,
            host,
            consumer,
            probe,
            &registry_only_facts,
        )
        .expect_err("a registry-only width fact must fail closed");
        assert_eq!(
            registry_only,
            "manifested ABI fact lacks producer: ABI width::C_LONG_WIDTH"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn manifest_identity_accepts_match_and_rejects_mismatch() {
        assert_target_abi_identity(TARGET_ABI_MANIFEST_HASH).expect("matching manifest");
        let mut mismatch = TARGET_ABI_MANIFEST_HASH;
        mismatch[31] ^= 1;
        assert_eq!(
            assert_target_abi_identity(mismatch),
            Err(TargetAbiIdentityError::HashMismatch)
        );
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn unavailable_target_manifest_fails_closed() {
        assert!(TARGET_ABI.backend.starts_with("unavailable-"));
        assert_eq!(TARGET_ABI.fact_count, 0);
        assert_eq!(
            assert_target_abi_identity(TARGET_ABI_MANIFEST_HASH),
            Err(TargetAbiIdentityError::BackendUnavailable)
        );
    }

    #[test]
    fn public_components_reject_unrooted_or_ambiguous_inputs() {
        for invalid in [b"".as_slice(), b".", b"..", b"a/b", b"a\0b"] {
            assert_eq!(
                PathComponent::new(invalid).unwrap_err().kind(),
                io::ErrorKind::InvalidInput
            );
        }
        assert_eq!(PathComponent::new(&[0xff]).unwrap().as_bytes(), &[0xff]);
    }

    #[test]
    fn public_surface_contains_only_ken_owned_semantic_types() {
        let source = include_str!("lib.rs");
        let public_surface = source
            .split_once("/// An opaque host-owned descriptor")
            .expect("public boundary marker")
            .1
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        for leaked in ["rustix::", "OwnedFd", "RawFd", "OFlags", "AtFlags", "Errno"] {
            assert!(
                !public_surface.contains(leaked),
                "private backend type leaked into public surface: {leaked}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rooted_operations_preserve_bytes_and_nofollow_policy() {
        use std::os::unix::fs::symlink;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("ken-host-px1-{}-{unique}", std::process::id()));
        std::fs::create_dir(&directory).expect("temp root");

        let root_path = RootPath::new(&directory).expect("root path");
        let root = open_root(&root_path).expect("root handle");
        let file = PathComponent::new(&[b'f', 0xff]).expect("byte file");
        let created = open_at(&root, &file, OpenRequest::CreateNew).expect("create file");
        write_new(&created, b"one").expect("write");
        let file_handle = open_at(&root, &file, OpenRequest::ReadWrite).expect("reopen file");
        append(&file_handle, b"-two").expect("append");
        assert_eq!(read(&file_handle).expect("read"), b"one-two");
        assert_eq!(metadata(&file_handle).expect("metadata").size, 7);

        let renamed = PathComponent::new(b"renamed").expect("renamed");
        rename(&root, &file, &root, &renamed).expect("rename");
        let link = PathComponent::new(b"link").expect("link");
        symlink("renamed", directory.join("link")).expect("symlink");
        assert_eq!(readlink_at(&root, &link).expect("readlink"), b"renamed");
        assert!(open_at(&root, &link, OpenRequest::Read).is_err());

        let subdir = PathComponent::new(b"subdir").expect("subdir");
        create_directory(&root, &subdir).expect("mkdir");
        let entries = read_directory(&root).expect("readdir");
        assert!(entries.iter().any(|entry| entry.name == b"renamed"));
        assert!(entries.iter().any(|entry| entry.name == b"link"));
        assert!(entries.iter().any(|entry| entry.name == b"subdir"));

        remove(&root, &link, RemoveKind::File).expect("unlink link");
        remove(&root, &renamed, RemoveKind::File).expect("unlink file");
        remove(&root, &subdir, RemoveKind::Directory).expect("rmdir");
        std::fs::remove_dir(&directory).expect("remove temp root");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn cwd_root_is_resolved_once_and_preserves_scope_and_symlink_denials() {
        use std::os::unix::fs::symlink;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let parent =
            std::env::temp_dir().join(format!("ken-host-px15-{}-{unique}", std::process::id()));
        let start = parent.join("start");
        std::fs::create_dir_all(start.join("data")).expect("start tree");
        std::fs::write(start.join("data/value"), b"original").expect("original file");
        let cwd = open_root(&RootPath::new(&start).unwrap()).expect("startup cwd handle");

        let scope = resolve_fs_root_spec_v1(
            &FsRootSpec::ExecutionStartCwd(b"data".to_vec()),
            &cwd,
            RightSet::READ,
            SymlinkPolicy::NoFollow,
        )
        .expect("resolve root at init");
        let FsHandle::Posix(stored_root) = scope.root else {
            panic!("linux root must be a descriptor")
        };

        std::fs::rename(&start, parent.join("moved")).expect("move startup cwd");
        std::fs::create_dir_all(start.join("data")).expect("replacement tree");
        std::fs::write(start.join("data/value"), b"replacement").expect("replacement file");
        let value = open_at(
            &stored_root,
            &PathComponent::new(b"value").unwrap(),
            OpenRequest::Read,
        )
        .expect("stored handle remains live");
        assert_eq!(read(&value).unwrap(), b"original");

        let fresh_cwd = open_root(&RootPath::new(&start).unwrap()).expect("fresh moved cwd");
        let fresh = resolve_fs_root_spec_v1(
            &FsRootSpec::ExecutionStartCwd(b"data".to_vec()),
            &fresh_cwd,
            RightSet::READ,
            SymlinkPolicy::NoFollow,
        )
        .expect("fresh resolver reaches replacement");
        let FsHandle::Posix(fresh_root) = fresh.root else {
            panic!("linux root must be a descriptor")
        };
        let replacement = open_at(
            &fresh_root,
            &PathComponent::new(b"value").unwrap(),
            OpenRequest::Read,
        )
        .unwrap();
        assert_eq!(read(&replacement).unwrap(), b"replacement");

        assert!(matches!(
            resolve_fs_root_spec_v1(
                &FsRootSpec::ExecutionStartCwd(b"../escape".to_vec()),
                &cwd,
                RightSet::READ,
                SymlinkPolicy::NoFollow,
            ),
            Err(FsRootResolveError::ScopeEscape)
        ));
        symlink(parent.join("moved/data"), start.join("link")).expect("outgoing symlink");
        assert!(matches!(
            resolve_fs_root_spec_v1(
                &FsRootSpec::ExecutionStartCwd(b"link".to_vec()),
                &fresh_cwd,
                RightSet::READ,
                SymlinkPolicy::NoFollow,
            ),
            Err(FsRootResolveError::SymlinkDenied)
        ));

        std::fs::remove_dir_all(parent).expect("remove PX15 tree");
    }
}
