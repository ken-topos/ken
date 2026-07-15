//! Audited host-ABI boundary for Ken's Linux runtime.
//!
//! This crate is a tested and target-validated TCB extension around `rustix`;
//! its host guarantees are never Ken proofs. `rustix` and raw descriptors stay
//! private. The kernel is unaffected and retains its own unsafe-code ban.
//!
//! Ken's sole supported entrypoint is the standard-Rust `ken` binary. Rust's
//! standard runtime ignores SIGPIPE before `main`, including in Rust test
//! binaries, so console writes surface a broken pipe as an I/O error. Ken does
//! not support `cdylib`, `staticlib`, C embedding, or a `#[unix_sigpipe]`
//! opt-out. A future non-Rust-standard-runtime embedding must re-establish the
//! SIGPIPE contract at its own entrypoint before calling Ken.

#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

pub fn open_at(
    parent: &RootedHandle,
    leaf: &PathComponent,
    request: OpenRequest,
) -> HostResult<RootedHandle> {
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

pub fn create_directory(parent: &RootedHandle, leaf: &PathComponent) -> HostResult<()> {
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
}
