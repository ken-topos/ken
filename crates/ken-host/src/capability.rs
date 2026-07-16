//! Shared runtime capability representation and fail-closed FS gate.
//!
//! This is trusted Rust, exercised by both executors. It is not a Ken proof.

use std::fmt;

use crate::RootedHandle;

/// Checked filesystem-root spelling retained until executor initialization.
///
/// Resolution consumes the execution-start cwd handle once and produces the
/// ordinary handle-backed `FsScope`; no operation retains this specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FsRootSpec {
    Absolute(Vec<u8>),
    ExecutionStartCwd(Vec<u8>),
    EffectiveUserHome(Vec<u8>),
}

impl Default for FsRootSpec {
    fn default() -> Self {
        Self::ExecutionStartCwd(Vec::new())
    }
}

impl FsRootSpec {
    pub fn parse_declared(bytes: &[u8]) -> Option<Self> {
        if let Some(suffix) = bytes.strip_prefix(b"~/") {
            Some(Self::EffectiveUserHome(suffix.to_vec()))
        } else if let Some(suffix) = bytes.strip_prefix(b"./") {
            Some(Self::ExecutionStartCwd(suffix.to_vec()))
        } else if bytes == b"." {
            Some(Self::ExecutionStartCwd(Vec::new()))
        } else if bytes.starts_with(b"/") {
            Some(Self::Absolute(bytes.to_vec()))
        } else {
            None
        }
    }

    pub fn tag_v1(&self) -> u64 {
        match self {
            Self::Absolute(_) => 0,
            Self::ExecutionStartCwd(_) => 1,
            Self::EffectiveUserHome(_) => 2,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Absolute(bytes)
            | Self::ExecutionStartCwd(bytes)
            | Self::EffectiveUserHome(bytes) => bytes,
        }
    }
}

/// Exact startup failures while binding an effective-user home root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HomeRootResolutionFailureV1 {
    BufferCapacityExceeded,
    NoAccountRecord,
    InvalidHomeDirectory,
    NssError(i32),
    RootOpen,
    ScopeEscape,
    SymlinkDenied,
}

impl fmt::Display for HomeRootResolutionFailureV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "effective-user home resolution failed: {self:?}")
    }
}

impl std::error::Error for HomeRootResolutionFailureV1 {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Authority(pub u8);

pub const AUTH_NONE: Authority = Authority(0);
pub const AUTH_PARTIAL: Authority = Authority(1);
pub const AUTH_FULL: Authority = Authority(2);

pub const fn authority_meet(left: Authority, right: Authority) -> Authority {
    Authority(if left.0 < right.0 { left.0 } else { right.0 })
}

pub const fn authority_flows_to(left: Authority, right: Authority) -> bool {
    left.0 <= right.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RightSet(u8);

impl RightSet {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const CREATE: Self = Self(1 << 2);
    pub const DELETE: Self = Self(1 << 3);
    pub const ENUMERATE: Self = Self(1 << 4);
    pub const METADATA: Self = Self(1 << 5);
    pub const CHANGE_MODE: Self = Self(1 << 6);
    pub const ALL: Self = Self((1 << 7) - 1);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
    pub const fn contains(self, right: Self) -> bool {
        self.0 & right.0 == right.0
    }
    pub const fn is_subset_of(self, other: Self) -> bool {
        self.0 & !other.0 == 0
    }
    pub const fn bits(self) -> u8 {
        self.0
    }
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & Self::ALL.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymlinkPolicy {
    NoFollow,
    FollowWithinScope,
}

#[derive(Clone)]
pub enum FsHandle {
    Posix(RootedHandle),
    Virtual(u64),
}

impl fmt::Debug for FsHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Posix(handle) => formatter.debug_tuple("Posix").field(handle).finish(),
            Self::Virtual(id) => formatter.debug_tuple("Virtual").field(id).finish(),
        }
    }
}

impl PartialEq for FsHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Posix(left), Self::Posix(right)) => left == right,
            (Self::Virtual(left), Self::Virtual(right)) => left == right,
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}
impl Eq for FsHandle {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsIdentity {
    Posix { device: u64, inode: u64 },
    Virtual(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsScope {
    pub rights: RightSet,
    pub root: FsHandle,
    pub lineage: Vec<FsIdentity>,
    pub symlink: SymlinkPolicy,
    pub empty: bool,
}

impl FsScope {
    pub fn root(
        rights: RightSet,
        root: FsHandle,
        identity: FsIdentity,
        symlink: SymlinkPolicy,
    ) -> Self {
        Self {
            rights,
            root,
            lineage: vec![identity],
            symlink,
            empty: false,
        }
    }
    pub fn child(
        &self,
        rights: RightSet,
        root: FsHandle,
        identity: FsIdentity,
        symlink: SymlinkPolicy,
    ) -> Self {
        let mut lineage = self.lineage.clone();
        lineage.push(identity);
        Self {
            rights,
            root,
            lineage,
            symlink,
            empty: self.empty,
        }
    }
}

pub const fn rights_for_authority(authority: Authority) -> RightSet {
    if authority.0 == AUTH_FULL.0 {
        RightSet::ALL
    } else if authority.0 == AUTH_PARTIAL.0 {
        RightSet::READ
            .union(RightSet::ENUMERATE)
            .union(RightSet::METADATA)
    } else {
        RightSet::NONE
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cap {
    authority: Authority,
    effect: String,
    scope: FsScope,
}

impl Cap {
    pub fn mint(authority: Authority, effect: impl Into<String>) -> Self {
        Self::mint_scoped(
            authority,
            effect,
            FsScope::root(
                rights_for_authority(authority),
                FsHandle::Virtual(0),
                FsIdentity::Virtual(0),
                SymlinkPolicy::NoFollow,
            ),
        )
    }
    pub fn mint_scoped(authority: Authority, effect: impl Into<String>, scope: FsScope) -> Self {
        Self {
            authority,
            effect: effect.into(),
            scope,
        }
    }
    pub fn authority(&self) -> Authority {
        self.authority
    }
    pub fn effect(&self) -> &str {
        &self.effect
    }
    pub fn scope(&self) -> &FsScope {
        &self.scope
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsCapabilityOperation {
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

impl FsCapabilityOperation {
    pub const fn required_right(self) -> RightSet {
        match self {
            Self::Read => RightSet::READ,
            Self::Write | Self::Append => RightSet::WRITE.union(RightSet::CREATE),
            Self::Metadata => RightSet::METADATA,
            Self::Enumerate => RightSet::ENUMERATE,
            Self::CreateDirectory => RightSet::CREATE,
            Self::RemoveFile | Self::RemoveDirectory => RightSet::DELETE,
            Self::RenameSource | Self::RenameDestination => RightSet::WRITE.union(RightSet::DELETE),
            Self::ChangeMode => RightSet::CHANGE_MODE,
        }
    }
    pub const fn resolves_parent(self) -> bool {
        matches!(
            self,
            Self::CreateDirectory
                | Self::RemoveFile
                | Self::RemoveDirectory
                | Self::RenameSource
                | Self::RenameDestination
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityDenied {
    RightNotHeld {
        op: FsCapabilityOperation,
        held_rights: u8,
    },
    ScopeEscape,
    SymlinkDenied,
    AuthorityInsufficient,
    MalformedCapability,
}

/// The sole runtime FS capability gate used by interpreter and native lanes.
pub fn check_fs_capability(
    cap: &Cap,
    operation: FsCapabilityOperation,
    required_authority: Authority,
) -> Result<&FsScope, CapabilityDenied> {
    let scope = cap.scope();
    let right = operation.required_right();
    if !scope.rights.contains(right) {
        return Err(CapabilityDenied::RightNotHeld {
            op: operation,
            held_rights: scope.rights.bits(),
        });
    }
    if !authority_flows_to(required_authority, cap.authority()) {
        return Err(CapabilityDenied::AuthorityInsufficient);
    }
    if scope.empty {
        return Err(CapabilityDenied::ScopeEscape);
    }
    Ok(scope)
}
