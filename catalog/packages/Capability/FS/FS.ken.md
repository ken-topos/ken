# `FS` — file-error rendering

The built-in FS algebra keeps paths as raw `Bytes` and returns structured
`FileError` values. Rendering is an ordinary package policy, never part of the
host driver, and adds zero entries to `trusted_base()`.

Security boundary: the current authority check is coarse and is **not
path-confined**. An `AFull` capability permits writes and deletes anywhere the
host process can access. Scoped rights, symlink policy, and TOCTOU-resistant
resolution are deferred to CA4/I-5.

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
