# `FS` — file-error rendering

The built-in FS algebra keeps paths as raw `Bytes` and returns structured
`FileError` values. Rendering is an ordinary package policy, never part of the
host driver, and adds zero entries to `trusted_base()`.

Security boundary: filesystem operations are authorized per operation by the
seven named rights in `RightSet`. `rights_for_authority` maps `Full`, `Partial`,
and `None` authority to those rights; `Full` retains all rights, including write
and delete, but exercises them only within its `FsScope`. Each `FsScope` is
rooted in an `FsHandle` and records its `FsIdentity` lineage; the `FsRootSpec`
spelling is resolved once during executor initialization and is not retained by
operations. `SymlinkPolicy` is a carried, expressible per-scope two-state
choice, `NoFollow` or `FollowWithinScope`, rather than a global policy. The
runtime `check_fs_capability` gate checks required rights and authority;
downstream filesystem resolution enforces confinement.

```ken
fn renderIOError (error : IOError) : String =
  match error {
    NotFound ↦ "NotFound";
    PermissionDenied ↦ "PermissionDenied";
    CapabilityDenied ↦ "CapabilityDenied";
    BrokenPipe ↦ "BrokenPipe";
    Interrupted ↦ "Interrupted";
    AlreadyExists ↦ "AlreadyExists";
    InvalidInput ↦ "InvalidInput";
    IsDirectory ↦ "IsDirectory";
    NotDirectory ↦ "NotDirectory";
    NotEmpty ↦ "NotEmpty";
    Unsupported ↦ "Unsupported";
    Other errno ↦ "Other"
  }

fn renderFileError (error : FileError) : String =
  match error {
    MkFileError operation path kind ↦ renderIOError kind
  }
```

The `Other Int` payload remains available for structured inspection even though
this minimal renderer intentionally chooses a stable label.
