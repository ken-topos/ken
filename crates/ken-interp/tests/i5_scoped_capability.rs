//! I-5 confinement net. These are trusted-runtime discriminators, not claims
//! that the kernel proves path confinement.

use ken_elaborator::capabilities::{
    attenuate, discharge_attenuation, AttenuationWindow, Cap, RightSet, SymlinkPolicy,
    AUTH_FULL, AUTH_PARTIAL,
};
use ken_elaborator::prover::Verdict;
use ken_interp::{
    CapabilityDenied, CaptureHost, EvalVal, FsOpKind, HostHandler, Resolution, ResolveError,
    VirtualFsNode,
};
use ken_kernel::GlobalEnv;

struct FsEnv {
    elab: ken_elaborator::ElabEnv,
    read_bytes: ken_kernel::GlobalId,
    write_file: ken_kernel::GlobalId,
    afull: ken_kernel::GlobalId,
    create_or_truncate: ken_kernel::GlobalId,
    ok: ken_kernel::GlobalId,
    err: ken_kernel::GlobalId,
    console: ken_interp::ConsoleIds,
    fs: ken_interp::FSIds,
}

fn fs_env() -> FsEnv {
    let elab = ken_elaborator::ElabEnv::new().expect("prelude");
    let get = |name: &str| *elab.globals.get(name).unwrap_or_else(|| panic!("{name}"));
    let console = ken_interp::ConsoleIds::from_elab(&elab).expect("Console ABI");
    let fs = ken_interp::FSIds::from_elab(&elab).expect("FS ABI");
    FsEnv {
        read_bytes: get("read_bytes"),
        write_file: get("write_file"),
        afull: get("AFull"),
        create_or_truncate: get("CreateOrTruncate"),
        ok: console.ok_id,
        err: console.err_id,
        console,
        fs,
        elab,
    }
}

enum DriverOp<'a> {
    Read,
    Write(&'a [u8]),
}

fn drive(
    env: &FsEnv,
    host: &mut CaptureHost,
    cap: &Cap,
    path: &[u8],
    op: DriverOp<'_>,
) -> EvalVal {
    let mut store = ken_interp::EvalStore::new();
    let id = match op {
        DriverOp::Read => env.read_bytes,
        DriverOp::Write(_) => env.write_file,
    };
    let mut tree = ken_interp::eval(
        &[],
        &ken_kernel::Term::const_(id, vec![]),
        &env.elab.env,
        &mut store,
    );
    let auth = ken_interp::eval(
        &[],
        &ken_kernel::Term::Constructor {
            id: env.afull,
            level_args: vec![],
        },
        &env.elab.env,
        &mut store,
    );
    for argument in [
        auth,
        EvalVal::Cap(cap.clone()),
        EvalVal::Bytes(path.to_vec()),
    ] {
        tree = ken_interp::apply(tree, argument, &env.elab.env, &mut store);
    }
    if let DriverOp::Write(bytes) = op {
        let policy = ken_interp::eval(
            &[],
            &ken_kernel::Term::Constructor {
                id: env.create_or_truncate,
                level_args: vec![],
            },
            &env.elab.env,
            &mut store,
        );
        tree = ken_interp::apply(tree, policy, &env.elab.env, &mut store);
        tree = ken_interp::apply(
            tree,
            EvalVal::Bytes(bytes.to_vec()),
            &env.elab.env,
            &mut store,
        );
    }
    ken_interp::run_io(
        tree,
        host,
        &env.console,
        Some(&env.fs),
        None,
        None,
        &env.elab.env,
        &mut store,
    )
    .expect("total FS result")
}

fn assert_ok(env: &FsEnv, value: &EvalVal) {
    assert!(matches!(value, EvalVal::Ctor { id, .. } if *id == env.ok));
}

fn assert_err(env: &FsEnv, value: &EvalVal) {
    assert!(matches!(value, EvalVal::Ctor { id, .. } if *id == env.err));
}

#[test]
fn traversal_denies_while_sibling_accepts_through_real_dispatch() {
    let env = fs_env();
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1/sub".to_vec());
    host.insert_file(b"dir1/sub/ok".to_vec(), b"ok".to_vec());
    host.insert_file(b"dir1/secret".to_vec(), b"secret".to_vec());
    let cap = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1/sub", RightSet::READ, SymlinkPolicy::NoFollow)
        .unwrap();

    let denied = drive(&env, &mut host, &cap, b"../secret", DriverOp::Read);
    assert_err(&env, &denied);
    assert_eq!(host.fs_denials(), &[CapabilityDenied::ScopeEscape]);
    assert!(host.fs_trace().is_empty());
    let accepted = drive(&env, &mut host, &cap, b"ok", DriverOp::Read);
    assert_ok(&env, &accepted);
    assert_eq!(host.fs_trace().len(), 1);
}

#[test]
fn symlink_policy_pairs_reach_the_real_dispatch_and_resolver() {
    let env = fs_env();
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1/real".to_vec());
    host.insert_file(b"dir1/real/x".to_vec(), b"x".to_vec());
    host.insert_symlink(b"dir1/link".to_vec(), b"/etc".to_vec());
    host.insert_symlink(b"dir1/inside".to_vec(), b"real/x".to_vec());
    host.insert_symlink(b"dir1/loop".to_vec(), b"loop".to_vec());

    let no_follow = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::READ, SymlinkPolicy::NoFollow)
        .unwrap();
    let denied = drive(&env, &mut host, &no_follow, b"link/passwd", DriverOp::Read);
    assert_err(&env, &denied);
    assert_eq!(host.fs_denials(), &[CapabilityDenied::SymlinkDenied]);
    assert!(host.fs_trace().is_empty());
    assert_ok(&env, &drive(&env, &mut host, &no_follow, b"real/x", DriverOp::Read));

    let follow = host
        .mint_scoped_fs_cap(
            AUTH_FULL,
            b"dir1",
            RightSet::READ,
            SymlinkPolicy::FollowWithinScope,
        )
        .unwrap();
    let denied = drive(&env, &mut host, &follow, b"link/passwd", DriverOp::Read);
    assert_err(&env, &denied);
    assert_eq!(host.fs_denials().last(), Some(&CapabilityDenied::ScopeEscape));
    assert_ok(&env, &drive(&env, &mut host, &follow, b"inside", DriverOp::Read));
    let before = host.fs_trace().len();
    let denied = drive(&env, &mut host, &follow, b"loop", DriverOp::Read);
    assert_err(&env, &denied);
    assert_eq!(host.fs_denials().last(), Some(&CapabilityDenied::SymlinkDenied));
    assert_eq!(host.fs_trace().len(), before);
}

#[test]
fn absolute_and_right_pairs_have_exact_pre_operation_denials() {
    let env = fs_env();
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1".to_vec());
    host.insert_file(b"dir1/x".to_vec(), b"x".to_vec());
    let cap = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::READ, SymlinkPolicy::NoFollow)
        .unwrap();

    let denied = drive(&env, &mut host, &cap, b"/dir1/x", DriverOp::Read);
    assert_err(&env, &denied);
    assert_eq!(host.fs_denials(), &[CapabilityDenied::ScopeEscape]);
    assert!(host.fs_trace().is_empty());
    assert_ok(&env, &drive(&env, &mut host, &cap, b"x", DriverOp::Read));
    let before = host.fs_trace().len();
    let denied = drive(&env, &mut host, &cap, b"x", DriverOp::Write(b"denied"));
    assert_err(&env, &denied);
    assert!(matches!(
        host.fs_denials().last(),
        Some(CapabilityDenied::RightNotHeld { op: FsOpKind::Write, .. })
    ));
    assert_eq!(host.fs_trace().len(), before);
}

#[test]
fn coarse_authority_is_a_named_real_dispatch_backstop() {
    let env = fs_env();
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1".to_vec());
    host.insert_file(b"dir1/x".to_vec(), b"x".to_vec());
    let cap = host
        .mint_scoped_fs_cap(
            AUTH_PARTIAL,
            b"dir1",
            RightSet::WRITE.union(RightSet::CREATE),
            SymlinkPolicy::NoFollow,
        )
        .unwrap();
    let denied = drive(&env, &mut host, &cap, b"x", DriverOp::Write(b"denied"));
    assert_err(&env, &denied);
    assert_eq!(host.fs_denials(), &[CapabilityDenied::AuthorityInsufficient]);
    assert!(host.fs_trace().is_empty());
}

#[test]
fn resolved_virtual_handle_stays_pinned_across_parent_replacement() {
    let env = fs_env();
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"dir1/sub".to_vec());
    host.insert_file(b"dir1/sub/file".to_vec(), b"old".to_vec());
    let cap = host
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::ALL, SymlinkPolicy::NoFollow)
        .unwrap();
    host.replace_subtree_after_next_resolve(
        b"dir1/sub".to_vec(),
        b"dir1/other".to_vec(),
        b"dir1/sub/file".to_vec(),
        b"replacement".to_vec(),
    );
    assert_ok(
        &env,
        &drive(&env, &mut host, &cap, b"sub/file", DriverOp::Write(b"pinned")),
    );
    assert_eq!(host.fs_resolve_count(), 1, "operation must not re-resolve");
    let nodes = host.fs_nodes();
    assert_eq!(
        nodes.get(b"dir1/other/file".as_slice()),
        Some(&VirtualFsNode::File(b"pinned".to_vec()))
    );
    assert_eq!(
        nodes.get(b"dir1/sub/file".as_slice()),
        Some(&VirtualFsNode::File(b"replacement".to_vec()))
    );

    let mut control = CaptureHost::new(Vec::new());
    control.insert_directory(b"dir1/sub".to_vec());
    control.insert_file(b"dir1/sub/file".to_vec(), b"old".to_vec());
    let control_cap = control
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1", RightSet::ALL, SymlinkPolicy::NoFollow)
        .unwrap();
    assert_ok(
        &env,
        &drive(
            &env,
            &mut control,
            &control_cap,
            b"sub/file",
            DriverOp::Write(b"pinned"),
        ),
    );
    assert_eq!(control.fs_resolve_count(), 1);
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
        .mint_scoped_fs_cap(AUTH_FULL, b"dir1/sub", RightSet::READ, SymlinkPolicy::NoFollow)
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
    assert!(disjoint.bound_scope.empty);
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
    use ken_interp::fs_target_components;
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    let root = std::env::temp_dir().join(format!(
        "ken-i5-openat-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
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
        &fs_target_components(b"escape/passwd").unwrap(),
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
            &fs_target_components(b"x").unwrap(),
            FsOpKind::Read,
            SymlinkPolicy::NoFollow,
        )
        .unwrap();
    let Resolution::Existing(handle) = accepted else { panic!("existing descriptor") };
    assert_eq!(host.fs_read_at(&handle).unwrap(), b"x");
    std::fs::remove_dir_all(root).unwrap();
}
