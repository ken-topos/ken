//! I-5 confinement net. These are trusted-runtime discriminators, not claims
//! that the kernel proves path confinement.

use ken_elaborator::capabilities::{
    attenuate, discharge_attenuation, AttenuationWindow, RightSet, SymlinkPolicy, AUTH_FULL,
    AUTH_PARTIAL,
};
use ken_elaborator::prover::Verdict;
use ken_interp::{
    check_fs_capability, fs_target_components, CapabilityDenied, CaptureHost, EvalVal, FsOpKind,
    HostCreatePolicy, HostHandler, Resolution, ResolveError, VirtualFsNode,
};
use ken_kernel::GlobalEnv;

fn components(path: &[u8]) -> Vec<Vec<u8>> {
    fs_target_components(path).expect("relative test target")
}

#[test]
fn traversal_denies_while_sibling_accepts_before_any_denied_operation() {
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1/sub".to_vec());
    host.insert_file(b"dir1/sub/ok".to_vec(), b"ok".to_vec());
    host.insert_file(b"dir1/secret".to_vec(), b"secret".to_vec());
    let cap = host
        .mint_scoped_fs_cap(
            AUTH_FULL,
            b"dir1/sub",
            RightSet::READ,
            SymlinkPolicy::NoFollow,
        )
        .unwrap();

    let denied = host.fs_resolve(
        &cap.scope().root,
        &[b"..".to_vec(), b"secret".to_vec()],
        FsOpKind::Read,
        cap.scope().symlink,
    );
    assert!(matches!(
        denied,
        Err(ResolveError::Denied(CapabilityDenied::ScopeEscape))
    ));
    assert!(
        host.fs_trace().is_empty(),
        "denial must precede the operation"
    );

    let accepted = host
        .fs_resolve(
            &cap.scope().root,
            &components(b"ok"),
            FsOpKind::Read,
            cap.scope().symlink,
        )
        .unwrap();
    let Resolution::Existing(handle) = accepted else {
        panic!("existing file")
    };
    assert_eq!(host.fs_read_at(&handle).unwrap(), b"ok");
}

#[test]
fn symlink_escape_denies_while_real_target_and_within_scope_link_accept() {
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1/real".to_vec());
    host.insert_file(b"dir1/real/x".to_vec(), b"x".to_vec());
    host.insert_symlink(b"dir1/escape".to_vec(), b"/etc".to_vec());
    host.insert_symlink(b"dir1/alias".to_vec(), b"real".to_vec());

    let no_follow = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::READ, SymlinkPolicy::NoFollow)
        .unwrap();
    let denied = host.fs_resolve(
        &no_follow.scope().root,
        &components(b"escape/passwd"),
        FsOpKind::Read,
        no_follow.scope().symlink,
    );
    assert!(matches!(
        denied,
        Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied))
    ));
    assert!(host.fs_trace().is_empty());

    let follow = host
        .mint_scoped_fs_cap(
            AUTH_FULL,
            b"dir1",
            RightSet::READ,
            SymlinkPolicy::FollowWithinScope,
        )
        .unwrap();
    let escape = host.fs_resolve(
        &follow.scope().root,
        &components(b"escape/passwd"),
        FsOpKind::Read,
        follow.scope().symlink,
    );
    assert!(matches!(
        escape,
        Err(ResolveError::Denied(CapabilityDenied::ScopeEscape))
    ));
    let accepted = host
        .fs_resolve(
            &follow.scope().root,
            &components(b"alias/x"),
            FsOpKind::Read,
            follow.scope().symlink,
        )
        .unwrap();
    let Resolution::Existing(handle) = accepted else {
        panic!("linked file")
    };
    assert_eq!(host.fs_read_at(&handle).unwrap(), b"x");
}

#[test]
fn absolute_target_and_missing_right_have_exact_fail_closed_variants() {
    assert_eq!(
        fs_target_components(b"/etc/passwd"),
        Err(CapabilityDenied::ScopeEscape)
    );

    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1".to_vec());
    let cap = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::READ, SymlinkPolicy::NoFollow)
        .unwrap();
    let value = EvalVal::Cap(cap);
    assert!(matches!(
        check_fs_capability(&value, FsOpKind::Write, AUTH_FULL, "write"),
        Err(CapabilityDenied::RightNotHeld {
            op: FsOpKind::Write,
            ..
        })
    ));
    assert!(check_fs_capability(&value, FsOpKind::Read, AUTH_PARTIAL, "read").is_ok());
    assert!(host.fs_trace().is_empty());
}

#[test]
fn coarse_authority_remains_a_named_defense_in_depth_gate() {
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1".to_vec());
    let cap = host
        .mint_scoped_fs_cap(
            AUTH_PARTIAL,
            b"dir1",
            RightSet::WRITE.union(RightSet::CREATE),
            SymlinkPolicy::NoFollow,
        )
        .unwrap();
    assert_eq!(
        check_fs_capability(&EvalVal::Cap(cap), FsOpKind::Write, AUTH_FULL, "write"),
        Err(CapabilityDenied::AuthorityInsufficient)
    );
    assert!(host.fs_trace().is_empty());
}

#[test]
fn resolved_virtual_handle_stays_pinned_across_parent_rename() {
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1/sub".to_vec());
    host.insert_file(b"dir1/sub/file".to_vec(), b"old".to_vec());
    let cap = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::ALL, SymlinkPolicy::NoFollow)
        .unwrap();

    let resolved = host
        .fs_resolve(
            &cap.scope().root,
            &components(b"sub/file"),
            FsOpKind::Write,
            cap.scope().symlink,
        )
        .unwrap();
    let Resolution::Existing(file_handle) = resolved else {
        panic!("file handle")
    };
    let from = host
        .fs_resolve(
            &cap.scope().root,
            &components(b"sub"),
            FsOpKind::RenameSource,
            cap.scope().symlink,
        )
        .unwrap();
    let to = host
        .fs_resolve(
            &cap.scope().root,
            &components(b"other"),
            FsOpKind::RenameDestination,
            cap.scope().symlink,
        )
        .unwrap();
    let Resolution::Parent(from_parent, from_leaf) = from else {
        panic!("source parent")
    };
    let Resolution::Parent(to_parent, to_leaf) = to else {
        panic!("destination parent")
    };
    host.fs_rename_at(&from_parent, &from_leaf, &to_parent, &to_leaf)
        .unwrap();
    host.fs_write_at(&file_handle, HostCreatePolicy::CreateOrTruncate, b"pinned")
        .unwrap();

    assert_eq!(
        host.fs_nodes().get(b"dir1/other/file".as_slice()),
        Some(&VirtualFsNode::File(b"pinned".to_vec()))
    );
    assert!(!host.fs_nodes().contains_key(b"dir1/sub/file".as_slice()));
}

#[test]
fn product_attenuation_narrows_and_deviant_widenings_are_unknown() {
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1/sub".to_vec());
    host.insert_directory(b"dir2".to_vec());
    let parent = host
        .mint_scoped_fs_cap(
            AUTH_FULL,
            b"dir1",
            RightSet::ALL,
            SymlinkPolicy::FollowWithinScope,
        )
        .unwrap();
    let child_scope = host
        .mint_scoped_fs_cap(
            AUTH_FULL,
            b"dir1/sub",
            RightSet::READ,
            SymlinkPolicy::NoFollow,
        )
        .unwrap()
        .scope()
        .clone();
    let (child, obligation) = attenuate(
        &parent,
        AttenuationWindow {
            authority: AUTH_FULL,
            rights: RightSet::READ,
            scope: Some(child_scope),
            symlink: SymlinkPolicy::NoFollow,
        },
    );
    assert_eq!(child.scope().rights, RightSet::READ);
    assert!(obligation.is_satisfied());
    let mut env = GlobalEnv::new();
    assert!(matches!(
        discharge_attenuation(&mut env, &obligation, "i5_narrow").verdict,
        Verdict::Proved { .. }
    ));

    let mut widened = obligation.clone();
    widened.child_rights = RightSet::READ.union(RightSet::WRITE);
    let mut env = GlobalEnv::new();
    assert!(matches!(
        discharge_attenuation(&mut env, &widened, "i5_right_widen").verdict,
        Verdict::Unknown { .. }
    ));

    let outside = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir2", RightSet::READ, SymlinkPolicy::NoFollow)
        .unwrap()
        .scope()
        .clone();
    let (_, disjoint) = attenuate(
        &parent,
        AttenuationWindow {
            authority: AUTH_FULL,
            rights: RightSet::READ,
            scope: Some(outside.clone()),
            symlink: SymlinkPolicy::NoFollow,
        },
    );
    assert!(
        disjoint.bound_scope.empty,
        "disjoint scope meet must be empty"
    );
    let mut escaped = disjoint.clone();
    escaped.child_scope = outside;
    let mut env = GlobalEnv::new();
    assert!(matches!(
        discharge_attenuation(&mut env, &escaped, "i5_scope_widen").verdict,
        Verdict::Unknown { .. }
    ));
}

#[cfg(unix)]
#[test]
fn posix_resolution_is_descriptor_relative_and_nofollow() {
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    let root = std::env::temp_dir().join(format!(
        "ken-i5-openat-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("dir1")).unwrap();
    std::fs::write(root.join("dir1/x"), b"x").unwrap();
    symlink("/etc", root.join("dir1/escape")).unwrap();

    let mut host = ken_interp::PosixHost::new_at(&root);
    let cap = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::READ, SymlinkPolicy::NoFollow)
        .unwrap();
    let denied = host.fs_resolve(
        &cap.scope().root,
        &components(b"escape/passwd"),
        FsOpKind::Read,
        SymlinkPolicy::NoFollow,
    );
    assert!(matches!(
        denied,
        Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied))
    ));
    let accepted = host
        .fs_resolve(
            &cap.scope().root,
            &components(b"x"),
            FsOpKind::Read,
            SymlinkPolicy::NoFollow,
        )
        .unwrap();
    let Resolution::Existing(handle) = accepted else {
        panic!("existing descriptor")
    };
    assert_eq!(host.fs_read_at(&handle).unwrap(), b"x");

    std::fs::remove_dir_all(root).unwrap();
}
