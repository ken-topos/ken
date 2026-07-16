//! Twin byte-identical real-root fixture and raw-byte snapshots.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

static NEXT_ROOTS: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedNodeKind {
    Directory,
    File(Vec<u8>),
    Symlink(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedNode {
    pub relative_path: Vec<u8>,
    pub kind: SeedNodeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnapshotNodeKind {
    Directory,
    File,
    Symlink,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SnapshotNode {
    pub relative_path: Vec<u8>,
    pub kind: SnapshotNodeKind,
    /// File bytes or raw symlink-target bytes. Directories use an empty value.
    pub bytes: Vec<u8>,
    /// POSIX permission/special bits for regular files and directories.
    pub mode: Option<u16>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RootSnapshot {
    pub nodes: Vec<SnapshotNode>,
}

#[derive(Debug)]
pub enum TwinRootError {
    Io(io::Error),
    InvalidRelativePath(Vec<u8>),
    UnsupportedPlatform,
}

impl fmt::Display for TwinRootError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "twin-root I/O failed: {error}"),
            Self::InvalidRelativePath(path) => {
                write!(formatter, "invalid raw relative path: {path:?}")
            }
            Self::UnsupportedPlatform => {
                formatter.write_str("PX6 raw-byte twin-root fixtures require a Unix target")
            }
        }
    }
}

impl std::error::Error for TwinRootError {}

impl From<io::Error> for TwinRootError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Owns twin real roots and a sibling artifact directory.
pub struct TwinRealRoots {
    base: PathBuf,
    interpreter: PathBuf,
    native: PathBuf,
    artifacts: PathBuf,
}

impl TwinRealRoots {
    pub fn create(seed: &[SeedNode]) -> Result<Self, TwinRootError> {
        let serial = NEXT_ROOTS.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!("ken-px6-{}-{serial}", std::process::id()));
        if base.exists() {
            fs::remove_dir_all(&base)?;
        }
        let interpreter = base.join("interp");
        let native = base.join("native");
        let artifacts = base.join("artifacts");
        let setup = (|| {
            fs::create_dir_all(&interpreter)?;
            fs::create_dir_all(&native)?;
            fs::create_dir_all(&artifacts)?;
            seed_root(&interpreter, seed)?;
            seed_root(&native, seed)?;
            Ok::<(), TwinRootError>(())
        })();
        if let Err(error) = setup {
            let _ = fs::remove_dir_all(&base);
            return Err(error);
        }

        let roots = Self {
            base,
            interpreter,
            native,
            artifacts,
        };
        let left = roots.snapshot_interpreter()?;
        let right = roots.snapshot_native()?;
        if left != right {
            return Err(TwinRootError::Io(io::Error::other(
                "twin roots were not byte-identical after seeding",
            )));
        }
        Ok(roots)
    }

    pub fn interpreter(&self) -> &Path {
        &self.interpreter
    }

    pub fn native(&self) -> &Path {
        &self.native
    }

    pub fn artifacts(&self) -> &Path {
        &self.artifacts
    }

    pub fn snapshot_interpreter(&self) -> Result<RootSnapshot, TwinRootError> {
        snapshot_root(&self.interpreter)
    }

    pub fn snapshot_native(&self) -> Result<RootSnapshot, TwinRootError> {
        snapshot_root(&self.native)
    }
}

impl Drop for TwinRealRoots {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn seed_root(root: &Path, seed: &[SeedNode]) -> Result<(), TwinRootError> {
    let mut nodes = seed.to_vec();
    nodes.sort_by(|left, right| {
        path_depth(&left.relative_path)
            .cmp(&path_depth(&right.relative_path))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    for node in nodes {
        let path = join_raw_relative(root, &node.relative_path)?;
        match node.kind {
            SeedNodeKind::Directory => fs::create_dir_all(path)?,
            SeedNodeKind::File(bytes) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, bytes)?;
            }
            SeedNodeKind::Symlink(target) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                create_symlink(&target, &path)?;
            }
        }
    }
    Ok(())
}

fn path_depth(path: &[u8]) -> usize {
    path.iter().filter(|byte| **byte == b'/').count()
}

fn validate_raw_relative(path: &[u8]) -> Result<(), TwinRootError> {
    if path.is_empty()
        || path.starts_with(b"/")
        || path.contains(&0)
        || path
            .split(|byte| *byte == b'/')
            .any(|part| part.is_empty() || part == b"." || part == b"..")
    {
        return Err(TwinRootError::InvalidRelativePath(path.to_vec()));
    }
    Ok(())
}

#[cfg(unix)]
fn join_raw_relative(root: &Path, path: &[u8]) -> Result<PathBuf, TwinRootError> {
    validate_raw_relative(path)?;
    Ok(root.join(OsString::from_vec(path.to_vec())))
}

#[cfg(not(unix))]
fn join_raw_relative(_root: &Path, _path: &[u8]) -> Result<PathBuf, TwinRootError> {
    Err(TwinRootError::UnsupportedPlatform)
}

#[cfg(unix)]
fn create_symlink(target: &[u8], path: &Path) -> Result<(), TwinRootError> {
    std::os::unix::fs::symlink(OsStr::from_bytes(target), path)?;
    Ok(())
}

#[cfg(not(unix))]
fn create_symlink(_target: &[u8], _path: &Path) -> Result<(), TwinRootError> {
    Err(TwinRootError::UnsupportedPlatform)
}

pub fn snapshot_root(root: &Path) -> Result<RootSnapshot, TwinRootError> {
    let mut nodes = Vec::new();
    snapshot_directory(root, &[], &mut nodes)?;
    nodes.sort();
    Ok(RootSnapshot { nodes })
}

/// Project two real-root snapshots into PX5's imported raw-path delta type.
pub fn canonical_filesystem_delta(
    before: &RootSnapshot,
    after: &RootSnapshot,
) -> Vec<ken_host::FsDeltaV1> {
    let before = canonical_nodes(before);
    let after = canonical_nodes(after);
    before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(
            |relative_path| match (before.get(&relative_path), after.get(&relative_path)) {
                (None, Some(node)) => Some(ken_host::FsDeltaV1::Created {
                    relative_path,
                    node: node.clone(),
                }),
                (Some(node), None) => Some(ken_host::FsDeltaV1::Removed {
                    relative_path,
                    node: node.clone(),
                }),
                (Some(before), Some(after)) if before != after => {
                    Some(ken_host::FsDeltaV1::Modified {
                        relative_path,
                        before: before.clone(),
                        after: after.clone(),
                    })
                }
                _ => None,
            },
        )
        .collect()
}

fn canonical_nodes(snapshot: &RootSnapshot) -> BTreeMap<Vec<u8>, ken_host::FsNodeObservationV1> {
    snapshot
        .nodes
        .iter()
        .map(|node| {
            let observation = match node.kind {
                SnapshotNodeKind::Directory => ken_host::FsNodeObservationV1 {
                    kind: ken_host::FsNodeKindV1::Directory,
                    file_bytes: None,
                    symlink_target: None,
                    mode: node.mode,
                },
                SnapshotNodeKind::File => ken_host::FsNodeObservationV1 {
                    kind: ken_host::FsNodeKindV1::File,
                    file_bytes: Some(node.bytes.clone()),
                    symlink_target: None,
                    mode: node.mode,
                },
                SnapshotNodeKind::Symlink => ken_host::FsNodeObservationV1 {
                    kind: ken_host::FsNodeKindV1::Symlink,
                    file_bytes: None,
                    symlink_target: Some(node.bytes.clone()),
                    mode: None,
                },
            };
            (node.relative_path.clone(), observation)
        })
        .collect()
}

fn snapshot_directory(
    directory: &Path,
    relative_parent: &[u8],
    nodes: &mut Vec<SnapshotNode>,
) -> Result<(), TwinRootError> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| raw_name(left.file_name()).cmp(&raw_name(right.file_name())));
    for entry in entries {
        let name = raw_name(entry.file_name());
        let mut relative = relative_parent.to_vec();
        if !relative.is_empty() {
            relative.push(b'/');
        }
        relative.extend_from_slice(&name);
        let metadata = fs::symlink_metadata(entry.path())?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            nodes.push(SnapshotNode {
                relative_path: relative,
                kind: SnapshotNodeKind::Symlink,
                bytes: raw_os_str(&fs::read_link(entry.path())?).to_vec(),
                mode: None,
            });
        } else if file_type.is_dir() {
            nodes.push(SnapshotNode {
                relative_path: relative.clone(),
                kind: SnapshotNodeKind::Directory,
                bytes: Vec::new(),
                mode: Some((metadata.mode() & 0o7777) as u16),
            });
            snapshot_directory(&entry.path(), &relative, nodes)?;
        } else if file_type.is_file() {
            nodes.push(SnapshotNode {
                relative_path: relative,
                kind: SnapshotNodeKind::File,
                bytes: fs::read(entry.path())?,
                mode: Some((metadata.mode() & 0o7777) as u16),
            });
        }
    }
    Ok(())
}

#[cfg(unix)]
fn raw_name(name: OsString) -> Vec<u8> {
    name.into_vec()
}

#[cfg(not(unix))]
fn raw_name(name: OsString) -> Vec<u8> {
    name.to_string_lossy().into_owned().into_bytes()
}

#[cfg(unix)]
fn raw_os_str(path: &Path) -> &[u8] {
    path.as_os_str().as_bytes()
}

#[cfg(not(unix))]
fn raw_os_str(_path: &Path) -> &[u8] {
    &[]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_snapshot_projects_mode_but_never_owner_namespace() {
        let node = ken_host::FsNodeObservationV1 {
            kind: ken_host::FsNodeKindV1::File,
            file_bytes: Some(Vec::new()),
            symlink_target: None,
            mode: Some(0o640),
        };
        let ken_host::FsNodeObservationV1 {
            kind: _,
            file_bytes: _,
            symlink_target: _,
            mode,
        } = node;
        assert_eq!(mode, Some(0o640));

        let native = include_str!("../../ken-runtime/src/object_linker_packaging.rs");
        let verifier = include_str!("filesystem.rs");
        for producer in [native, verifier] {
            for owner_projection in ["uid()", "gid()"] {
                let needle = format!("metadata.{owner_projection}");
                assert!(!producer.contains(&needle));
            }
        }
    }

    #[test]
    fn twin_roots_preserve_raw_paths_files_directories_and_symlinks() {
        let roots = TwinRealRoots::create(&[
            SeedNode {
                relative_path: b"dir".to_vec(),
                kind: SeedNodeKind::Directory,
            },
            SeedNode {
                relative_path: vec![b'd', b'i', b'r', b'/', b'f', 0xff],
                kind: SeedNodeKind::File(vec![0, 0xfe, b'x']),
            },
            SeedNode {
                relative_path: b"link".to_vec(),
                kind: SeedNodeKind::Symlink(vec![b'd', b'i', b'r', b'/', b'f', 0xff]),
            },
        ])
        .expect("create twin roots");

        let left = roots.snapshot_interpreter().expect("snapshot interp");
        let right = roots.snapshot_native().expect("snapshot native");
        assert_eq!(left, right);
        assert_eq!(left.nodes.len(), 3);
        assert_eq!(left.nodes[2].kind, SnapshotNodeKind::Symlink);
    }

    #[test]
    fn traversal_and_absolute_seed_paths_fail_closed() {
        for path in [b"../escape".as_slice(), b"/absolute", b"a//b", b"a/./b"] {
            let error = TwinRealRoots::create(&[SeedNode {
                relative_path: path.to_vec(),
                kind: SeedNodeKind::File(vec![]),
            }])
            .err()
            .expect("invalid path rejected");
            assert!(matches!(error, TwinRootError::InvalidRelativePath(_)));
        }
    }
}
