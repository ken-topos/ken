//! I-3 CA3 FS floor: non-constant surface, total values, and capture seam.

use ken_elaborator::capabilities::{Cap, AUTH_FULL, AUTH_PARTIAL};
use ken_interp::{CaptureHost, ConsoleIds, EvalStore, EvalVal, FSIds, FsTrace};
use ken_kernel::{GlobalId, Term};

const FS_PACKAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../catalog/packages/Capability/FS/FS.ken.md"
));

struct FsEnv {
    elab: ken_elaborator::ElabEnv,
    console: ConsoleIds,
    fs: FSIds,
}

fn setup() -> FsEnv {
    let elab = ken_elaborator::ElabEnv::new().expect("prelude registers");
    let console = ConsoleIds::from_elab(&elab).expect("Console ABI");
    let fs = FSIds::from_elab(&elab).expect("FS ABI");
    FsEnv { elab, console, fs }
}

fn get(env: &FsEnv, name: &str) -> GlobalId {
    *env.elab
        .globals
        .get(name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn ctor(env: &FsEnv, name: &str, store: &mut EvalStore) -> EvalVal {
    ken_interp::eval(
        &[],
        &Term::Constructor {
            id: get(env, name),
            level_args: vec![],
        },
        &env.elab.env,
        store,
    )
}

fn drive(
    env: &FsEnv,
    host: &mut CaptureHost,
    name: &str,
    authority: ken_elaborator::capabilities::Authority,
    mut args: Vec<EvalVal>,
) -> EvalVal {
    let mut store = EvalStore::new();
    let mut function = ken_interp::eval(
        &[],
        &Term::const_(get(env, name), vec![]),
        &env.elab.env,
        &mut store,
    );
    let auth_name = if authority == AUTH_FULL {
        "AFull"
    } else {
        "APartial"
    };
    function = ken_interp::apply(
        function,
        ctor(env, auth_name, &mut store),
        &env.elab.env,
        &mut store,
    );
    function = ken_interp::apply(
        function,
        EvalVal::Cap(Cap::mint(authority, "FS")),
        &env.elab.env,
        &mut store,
    );
    for argument in args.drain(..) {
        function = ken_interp::apply(function, argument, &env.elab.env, &mut store);
    }
    ken_interp::run_io(
        function,
        host,
        &env.console,
        Some(&env.fs),
        None,
        &env.elab.env,
        &mut store,
    )
    .expect("FS tree drives")
}

fn bytes(value: &[u8]) -> EvalVal {
    EvalVal::Bytes(value.to_vec())
}

fn result_payload(value: &EvalVal, expected: GlobalId) -> &EvalVal {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == expected => args.get(2).expect("Result payload"),
        other => panic!("expected Result ctor {expected:?}, got {other:?}"),
    }
}

fn file_error_kind(value: &EvalVal, env: &FsEnv) -> GlobalId {
    let error = result_payload(value, env.console.err_id);
    match error {
        EvalVal::Ctor { id, args, .. } if *id == env.fs.mk_file_error_id => match args.get(2) {
            Some(EvalVal::Ctor { id, .. }) => *id,
            other => panic!("expected named IOError, got {other:?}"),
        },
        other => panic!("expected structured FileError, got {other:?}"),
    }
}

#[test]
fn surface_and_render_package_add_zero_trusted_base_entries() {
    let mut elab = ken_elaborator::ElabEnv::new().expect("prelude registers");
    for name in [
        "FSOp",
        "ReadFile",
        "WriteFile",
        "AppendFile",
        "Metadata",
        "ReadDirectory",
        "CreateDirectory",
        "RemoveFile",
        "RemoveDirectory",
        "Rename",
        "fs_resp",
    ] {
        let id = *elab
            .globals
            .get(name)
            .unwrap_or_else(|| panic!("missing {name}"));
        assert!(
            !elab.env.trusted_base().contains(&id),
            "{name} must be checked Ken"
        );
    }
    let before = elab.env.trusted_base();
    elab.elaborate_ken_md_file(FS_PACKAGE)
        .expect("ordinary FS render package elaborates");
    assert_eq!(before, elab.env.trusted_base());
    let extracted =
        ken_elaborator::literate::extract_ken_md(FS_PACKAGE).expect("FS.ken.md must extract");
    assert!(
        !extracted.source.contains("Axiom"),
        "FS.ken code must declare no Axiom"
    );
}

#[test]
fn full_authority_drives_every_fs_arm_with_exact_state_and_trace() {
    let env = setup();
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"root".to_vec());

    let ok = |value: &EvalVal| {
        result_payload(value, env.console.ok_id);
    };
    ok(&drive(
        &env,
        &mut host,
        "create_directory",
        AUTH_FULL,
        vec![
            EvalVal::Ctor {
                id: env.console.false_id,
                args: Default::default(),
                slot: 0,
            },
            bytes(b"root/sub"),
        ],
    ));
    ok(&drive(
        &env,
        &mut host,
        "write_file",
        AUTH_FULL,
        vec![
            bytes(b"root/sub/a\xff"),
            EvalVal::Ctor {
                id: env.fs.create_new_id,
                args: Default::default(),
                slot: 0,
            },
            bytes(b"one"),
        ],
    ));
    ok(&drive(
        &env,
        &mut host,
        "append_file",
        AUTH_FULL,
        vec![bytes(b"root/sub/a\xff"), bytes(b"-two")],
    ));
    ok(&drive(
        &env,
        &mut host,
        "read_bytes",
        AUTH_FULL,
        vec![bytes(b"root/sub/a\xff")],
    ));
    ok(&drive(
        &env,
        &mut host,
        "file_metadata",
        AUTH_FULL,
        vec![bytes(b"root/sub/a\xff")],
    ));
    ok(&drive(
        &env,
        &mut host,
        "read_directory",
        AUTH_FULL,
        vec![bytes(b"root/sub")],
    ));
    ok(&drive(
        &env,
        &mut host,
        "rename_file",
        AUTH_FULL,
        vec![bytes(b"root/sub/a\xff"), bytes(b"root/sub/b\xff")],
    ));
    ok(&drive(
        &env,
        &mut host,
        "remove_file",
        AUTH_FULL,
        vec![bytes(b"root/sub/b\xff")],
    ));
    ok(&drive(
        &env,
        &mut host,
        "remove_directory",
        AUTH_FULL,
        vec![
            EvalVal::Ctor {
                id: env.console.false_id,
                args: Default::default(),
                slot: 0,
            },
            bytes(b"root/sub"),
        ],
    ));

    assert_eq!(
        host.fs_nodes(),
        [(b"root".to_vec(), ken_interp::VirtualFsNode::Directory)]
            .into_iter()
            .collect()
    );
    assert_eq!(
        host.fs_trace(),
        &[
            FsTrace::CreateDirectory {
                path: b"root/sub".to_vec(),
                recursive: false
            },
            FsTrace::WriteFile {
                path: b"root/sub/a\xff".to_vec(),
                policy: ken_interp::HostCreatePolicy::CreateNew,
                bytes: b"one".to_vec()
            },
            FsTrace::AppendFile {
                path: b"root/sub/a\xff".to_vec(),
                bytes: b"-two".to_vec()
            },
            FsTrace::ReadFile {
                path: b"root/sub/a\xff".to_vec()
            },
            FsTrace::Metadata {
                path: b"root/sub/a\xff".to_vec()
            },
            FsTrace::ReadDirectory {
                path: b"root/sub".to_vec()
            },
            FsTrace::Rename {
                from: b"root/sub/a\xff".to_vec(),
                to: b"root/sub/b\xff".to_vec()
            },
            FsTrace::RemoveFile {
                path: b"root/sub/b\xff".to_vec()
            },
            FsTrace::RemoveDirectory {
                path: b"root/sub".to_vec(),
                recursive: false
            },
        ]
    );
}

#[test]
fn named_errors_and_write_gate_are_total_and_discriminating() {
    let env = setup();
    let mut host = CaptureHost::new(Vec::new());
    host.insert_directory(b"root".to_vec());
    host.insert_directory(b"root/nonempty".to_vec());
    host.insert_file(b"root/nonempty/file".to_vec(), b"x".to_vec());
    host.insert_file(b"root/existing".to_vec(), b"old".to_vec());

    let exists = drive(
        &env,
        &mut host,
        "write_file",
        AUTH_FULL,
        vec![
            bytes(b"root/existing"),
            EvalVal::Ctor {
                id: env.fs.create_new_id,
                args: Default::default(),
                slot: 0,
            },
            bytes(b"new"),
        ],
    );
    assert_eq!(file_error_kind(&exists, &env), env.console.alreadyexists_id);

    let not_empty = drive(
        &env,
        &mut host,
        "remove_directory",
        AUTH_FULL,
        vec![
            EvalVal::Ctor {
                id: env.console.false_id,
                args: Default::default(),
                slot: 0,
            },
            bytes(b"root/nonempty"),
        ],
    );
    assert_eq!(file_error_kind(&not_empty, &env), env.console.notempty_id);

    let trace_len = host.fs_trace().len();
    let denied = drive(
        &env,
        &mut host,
        "write_file",
        AUTH_PARTIAL,
        vec![
            bytes(b"root/denied"),
            EvalVal::Ctor {
                id: env.fs.create_new_id,
                args: Default::default(),
                slot: 0,
            },
            bytes(b"no"),
        ],
    );
    assert_eq!(
        file_error_kind(&denied, &env),
        env.console.capabilitydenied_id
    );
    assert_eq!(
        host.fs_trace().len(),
        trace_len,
        "gate must run before host call"
    );
    assert!(!host.fs_nodes().contains_key(b"root/denied".as_slice()));
}
