//! FS-driver-build D4 — fixture harness for the real `[FS]` driver
//! (`docs/program/wp/FS-driver.md` D1/D2, `FS-driver-conformance.md` §1/§3).
//!
//! Drives `read_bytes` through the REAL `ken-interp` reduction (D1, a genuine
//! kernel-rechecked `view`) and the REAL `run_io` driver arm (D2, the sole new
//! `std::fs::read` syscall) against a checked-in, version-controlled fixture —
//! no mock/virtual FS anywhere in this file (AC4, grep-verifiable).
//!
//! `authorizes` (D3, `eval.rs`) is now the real runtime capability gate:
//! it decodes the carried `Cap`'s `Authority` and calls
//! `capabilities::check_authority_sufficient`. These fixtures thread a real
//! minted `Cap` end to end (mint -> `read_bytes cap path` -> `ReadFile cap
//! path` `Vis` node -> driver decode) to exercise the reduction + I/O
//! failure-surfacing shape; `fs_driver_build_capability_acceptance.rs` pins
//! AC3's runtime arm (R1/R2).

use std::path::PathBuf;

use ken_elaborator::capabilities::{Cap, AUTH_FULL};

/// Encode a minted `Cap`'s authority as the runtime carries it — a real
/// opaque `EvalVal::Cap` (fs-read-file-lines-flip D3, Architect ruling: NOT
/// the earlier `EvalVal::Int(level)` positional-scalar).
fn cap_evalval(cap: &Cap) -> ken_interp::EvalVal {
    ken_interp::EvalVal::Cap(cap.clone())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_path(name: &str) -> PathBuf {
    repo_root().join("conformance/fs/fixtures").join(name)
}

struct FsEnv {
    elab_env: ken_elaborator::ElabEnv,
    read_bytes_id: ken_kernel::GlobalId,
    afull_id: ken_kernel::GlobalId,
    vis_id: ken_kernel::GlobalId,
    ret_id: ken_kernel::GlobalId,
    ok_id: ken_kernel::GlobalId,
    err_id: ken_kernel::GlobalId,
    notfound_id: ken_kernel::GlobalId,
    console_ids: ken_interp::ConsoleIds,
    fs_ids: ken_interp::FSIds,
}

fn mk_env() -> FsEnv {
    let elab_env = ken_elaborator::ElabEnv::new().expect("prelude registers");
    let g = &elab_env.globals;
    let get = |name: &str| -> ken_kernel::GlobalId {
        *g.get(name).unwrap_or_else(|| panic!("prelude: '{}' not registered", name))
    };
    let console_ids = ken_interp::ConsoleIds {
        itree_id: get("ITree"),
        ret_id: get("Ret"),
        vis_id: get("Vis"),
        write_id: get("Write"),
        unit_id: get("Unit"),
        params_len: 3,
    };
    let fs_ids = ken_interp::FSIds {
        readfile_id: get("ReadFile"),
        ok_id: get("Ok"),
        err_id: get("Err"),
        notfound_id: get("NotFound"),
        permissiondenied_id: get("PermissionDenied"),
        capabilitydenied_id: get("CapabilityDenied"),
        other_id: get("Other"),
    };
    FsEnv {
        read_bytes_id: get("read_bytes"),
        afull_id: get("AFull"),
        vis_id: console_ids.vis_id,
        ret_id: console_ids.ret_id,
        ok_id: fs_ids.ok_id,
        err_id: fs_ids.err_id,
        notfound_id: fs_ids.notfound_id,
        console_ids,
        fs_ids,
        elab_env,
    }
}

/// Evaluate `read_bytes a cap path` (real D1 reduction; `a` is `read_bytes`'s
/// fs-read-file-lines-flip D2 authority-level parameter, erased at runtime —
/// any well-formed `Auth` `EvalVal` here is equivalent, `AFull` chosen to
/// match the minted `Cap`'s own level) and drive the result through the
/// real D2 `run_io` FS arm.
fn read_via_real_driver(
    env: &mut FsEnv,
    path: &str,
) -> Result<ken_interp::EvalVal, ken_interp::RunIoError> {
    let mut store = ken_interp::EvalStore::new();
    let term = ken_kernel::Term::const_(env.read_bytes_id, vec![]);
    let f = ken_interp::eval(&[], &term, &env.elab_env.env, &mut store);
    let a_term = ken_kernel::Term::Constructor { id: env.afull_id, level_args: vec![] };
    let a_val = ken_interp::eval(&[], &a_term, &env.elab_env.env, &mut store);
    let step0 = ken_interp::apply(f, a_val, &env.elab_env.env, &mut store);
    let cap = Cap::mint(AUTH_FULL, "FS");
    let step1 = ken_interp::apply(step0, cap_evalval(&cap), &env.elab_env.env, &mut store);
    let path_val = ken_interp::EvalVal::Bytes(path.as_bytes().to_vec());
    let tree = ken_interp::apply(step1, path_val, &env.elab_env.env, &mut store);
    ken_interp::run_io(
        tree,
        &env.console_ids,
        Some(&env.fs_ids),
        None,
        &env.elab_env.env,
        &mut store,
    )
}

/// AC2/AC4 — a real `read_bytes` call through the real driver against the
/// checked-in hermetic fixture returns `Ok(<fixture bytes exactly>)`.
#[test]
fn positive_read_returns_exact_fixture_bytes() {
    let mut env = mk_env();
    let path = fixture_path("three-lines.txt");
    let expected = std::fs::read(&path).expect("fixture file exists");
    assert_eq!(expected, b"alpha\nbeta\ngamma\n");

    let result = read_via_real_driver(&mut env, path.to_str().unwrap());

    match result {
        Ok(ken_interp::EvalVal::Ctor { id, args, .. }) if id == env.ok_id => {
            match args.get(2) {
                Some(ken_interp::EvalVal::Bytes(b)) => {
                    assert_eq!(*b, expected, "read_bytes must return the fixture's exact bytes");
                }
                other => panic!("expected Ok(Bytes(..)), got payload {:?}", other),
            }
        }
        other => panic!("expected Ok(<fixture bytes>) via real driver, got {:?}", other),
    }
}

/// AC5 — a sufficient (stubbed-authorized) cap on a NONEXISTENT fixture path
/// reaches the syscall, gets file-not-found, and surfaces a total
/// `Result Err(NotFound)` — never a panic. Distinct from a capability-denial
/// (refused BEFORE the syscall); this fixture proves the driver attempts the
/// read and surfaces I/O failure honestly.
#[test]
fn absent_path_surfaces_total_not_found_result() {
    let mut env = mk_env();
    let path = fixture_path("does-not-exist.txt");
    assert!(!path.exists(), "fixture harness precondition: path must be absent");

    let result = read_via_real_driver(&mut env, path.to_str().unwrap());

    match result {
        Ok(ken_interp::EvalVal::Ctor { id, args, .. }) if id == env.err_id => {
            match args.get(2) {
                Some(ken_interp::EvalVal::Ctor { id: err_id, .. }) => {
                    assert_eq!(*err_id, env.notfound_id, "absent path must surface IOError::NotFound");
                }
                other => panic!("expected Err(NotFound), got payload {:?}", other),
            }
        }
        other => panic!("expected a total Err(NotFound) Result, got {:?}", other),
    }
}

/// The pure `read_bytes` reduction never touches a syscall — it builds a
/// `Vis` node BEFORE any driver runs (D5/AC5 totality: the pure core stays
/// total, all I/O is confined to `run_io`).
#[test]
fn pure_reduction_builds_vis_before_any_syscall() {
    let env = mk_env();
    let mut store = ken_interp::EvalStore::new();
    let term = ken_kernel::Term::const_(env.read_bytes_id, vec![]);
    let f = ken_interp::eval(&[], &term, &env.elab_env.env, &mut store);
    let a_term = ken_kernel::Term::Constructor { id: env.afull_id, level_args: vec![] };
    let a_val = ken_interp::eval(&[], &a_term, &env.elab_env.env, &mut store);
    let step0 = ken_interp::apply(f, a_val, &env.elab_env.env, &mut store);
    let cap_val = ken_interp::EvalVal::Cap(Cap::mint(AUTH_FULL, "FS"));
    let step1 = ken_interp::apply(step0, cap_val, &env.elab_env.env, &mut store);
    // A path that would fail loudly if the pure reduction ever touched
    // `std::fs` (an invalid path with embedded NUL, rejected by every OS
    // filesystem call) — the pure `eval`/`apply` steps below must still
    // succeed, proving zero syscall happens before `run_io` runs.
    let path_val = ken_interp::EvalVal::Bytes(b"/nul\0-embedded-path".to_vec());
    let tree = ken_interp::apply(step1, path_val, &env.elab_env.env, &mut store);

    match tree {
        ken_interp::EvalVal::Ctor { id, .. } if id == env.vis_id => {}
        other => panic!("expected a pure Vis node with zero syscall, got {:?}", other),
    }
    let _ = env.ret_id;
}
