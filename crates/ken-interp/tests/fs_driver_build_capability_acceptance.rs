//! FS-driver-build D3 — the runtime capability thread's AC3 discriminating
//! pair (`docs/program/wp/FS-driver.md` D3, `FS-driver-conformance.md` §2b:
//! R1/R2, Phase-1 authority-level form).
//!
//! Drives the REAL `read_bytes` reduction + REAL `run_io` driver arm (same
//! path as `fs_driver_build_acceptance.rs`) against a minted, then attenuated,
//! `Cap` — proving the runtime `authorizes(cap, path)` gate (`eval.rs`) is
//! load-bearing: the SAME op, on the SAME existing fixture path, flips
//! outcome purely on the carried authority. R2's cap is present and clears
//! the *static* `using cap` gate (so it actually reaches the runtime check,
//! not a `MissingCapability` bounce) but is runtime-insufficient.

use std::path::PathBuf;

use ken_elaborator::capabilities::{attenuate, Cap, AUTH_FULL, AUTH_NONE, AUTH_PARTIAL};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_path(name: &str) -> PathBuf {
    repo_root().join("conformance/fs/fixtures").join(name)
}

struct FsEnv {
    elab_env: ken_elaborator::ElabEnv,
    read_bytes_id: ken_kernel::GlobalId,
    anone_id: ken_kernel::GlobalId,
    apartial_id: ken_kernel::GlobalId,
    afull_id: ken_kernel::GlobalId,
    ok_id: ken_kernel::GlobalId,
    err_id: ken_kernel::GlobalId,
    capabilitydenied_id: ken_kernel::GlobalId,
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
        anone_id: get("ANone"),
        apartial_id: get("APartial"),
        afull_id: get("AFull"),
        ok_id: fs_ids.ok_id,
        err_id: fs_ids.err_id,
        capabilitydenied_id: fs_ids.capabilitydenied_id,
        console_ids,
        fs_ids,
        elab_env,
    }
}

/// Encode a `Cap`'s carried authority the way `eval.rs`'s `authorizes` decodes
/// it: a real opaque `EvalVal::Cap` (fs-read-file-lines-flip D3, Architect
/// ruling — NOT the earlier `EvalVal::Int(level)` positional-scalar).
fn cap_evalval(cap: &Cap) -> ken_interp::EvalVal {
    ken_interp::EvalVal::Cap(cap.clone())
}

/// The `Auth` ctor `GlobalId` matching a `capabilities::Authority` level
/// (the D2↔D3 shared contract, `capabilities.rs:33-35`). Erased at runtime
/// (`read_bytes`'s `a` parameter is never inspected past elaboration — the
/// REAL enforcement reads `Authority` off the carried `EvalVal::Cap`), but
/// `read_bytes` is authority-polymorphic in `a`, so SOME `Auth` value must be
/// supplied to reach the `Cap`/`Bytes` arguments.
fn auth_ctor_id(env: &FsEnv, authority: ken_elaborator::capabilities::Authority) -> ken_kernel::GlobalId {
    use ken_elaborator::capabilities::{AUTH_FULL, AUTH_NONE, AUTH_PARTIAL};
    match authority {
        AUTH_NONE => env.anone_id,
        AUTH_PARTIAL => env.apartial_id,
        AUTH_FULL => env.afull_id,
        other => panic!("no Auth ctor for authority level {:?}", other),
    }
}

fn read_via_real_driver(
    env: &mut FsEnv,
    cap: &Cap,
    path: &str,
) -> Result<ken_interp::EvalVal, ken_interp::RunIoError> {
    let mut store = ken_interp::EvalStore::new();
    let term = ken_kernel::Term::const_(env.read_bytes_id, vec![]);
    let f = ken_interp::eval(&[], &term, &env.elab_env.env, &mut store);
    let a_id = auth_ctor_id(env, ken_elaborator::capabilities::authority(cap));
    let a_term = ken_kernel::Term::Constructor { id: a_id, level_args: vec![] };
    let a_val = ken_interp::eval(&[], &a_term, &env.elab_env.env, &mut store);
    let step0 = ken_interp::apply(f, a_val, &env.elab_env.env, &mut store);
    let step1 = ken_interp::apply(step0, cap_evalval(cap), &env.elab_env.env, &mut store);
    let path_val = ken_interp::EvalVal::Bytes(path.as_bytes().to_vec());
    let tree = ken_interp::apply(step1, path_val, &env.elab_env.env, &mut store);
    ken_interp::run_io(
        tree,
        &env.console_ids,
        Some(&env.fs_ids),
        &env.elab_env.env,
        &mut store,
    )
}

/// R1 — a cap attenuated to a SUFFICIENT authority (`AUTH_PARTIAL`, "read-only,
/// single dir") reaches the driver and reads the fixture.
#[test]
fn r1_sufficient_cap_reads_fixture() {
    let mut env = mk_env();
    let path = fixture_path("three-lines.txt");
    let expected = std::fs::read(&path).expect("fixture file exists");

    let parent = Cap::mint(AUTH_FULL, "FS");
    let (cap, _obl) = attenuate(&parent, AUTH_PARTIAL);

    let result = read_via_real_driver(&mut env, &cap, path.to_str().unwrap());

    match result {
        Ok(ken_interp::EvalVal::Ctor { id, args, .. }) if id == env.ok_id => match args.get(2) {
            Some(ken_interp::EvalVal::Bytes(b)) => assert_eq!(*b, expected),
            other => panic!("expected Ok(Bytes(..)), got payload {:?}", other),
        },
        other => panic!("expected Ok(<fixture bytes>) for a sufficient cap, got {:?}", other),
    }
}

/// R2 — the SAME op, on the SAME existing fixture path (so a denial can only
/// be the capability gate, never `NotFound`), with the cap attenuated to
/// `AUTH_NONE` ⇒ `CapabilityDenied` at the driver, no read. Outcome flips
/// purely on the carried authority — proves `authorizes` is load-bearing, not
/// a no-op (a no-op always-true gate would return `Ok` here too).
#[test]
fn r2_insufficient_cap_denied_before_read() {
    let mut env = mk_env();
    let path = fixture_path("three-lines.txt");
    assert!(path.exists(), "fixture harness precondition: path must exist (isolates capability denial from NotFound)");

    let parent = Cap::mint(AUTH_FULL, "FS");
    let (cap, _obl) = attenuate(&parent, AUTH_NONE);

    let result = read_via_real_driver(&mut env, &cap, path.to_str().unwrap());

    match result {
        Ok(ken_interp::EvalVal::Ctor { id, args, .. }) if id == env.err_id => match args.get(2) {
            Some(ken_interp::EvalVal::Ctor { id: err_id, .. }) => {
                assert_eq!(*err_id, env.capabilitydenied_id, "insufficient cap must surface CapabilityDenied, not e.g. NotFound");
            }
            other => panic!("expected Err(CapabilityDenied), got payload {:?}", other),
        },
        other => panic!("expected a total Err(CapabilityDenied) Result, got {:?}", other),
    }
}
