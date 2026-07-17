//! PX7-F public checked-Ken resource lifecycle through the real interpreter.

use std::path::PathBuf;

use ken_elaborator::capabilities::AUTH_FULL;
use ken_interp::{apply, eval, ConsoleIds, CoproductIds, EvalStore, EvalVal, FSIds, PosixHost};
use ken_kernel::{Decl, Term};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn env() -> ken_elaborator::ElabEnv {
    let mut env = ken_elaborator::ElabEnv::empty().expect("PX7-F prelude");
    env.elaborate_file(
        r#"
        fn px7f_escape_body (resource : Resource FsHandle)
          : HostIO AFull (ResourceBodyResult Unit (Resource FsHandle)) =
          Ret (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (ResourceBodyResult Unit (Resource FsHandle))
            (ResourceBodyOk Unit (Resource FsHandle) resource)

        fn px7f_metadata_after (outcome : Result ResourceError FileMetadata)
          : HostIO AFull (ResourceBodyResult ResourceError Unit) =
          match outcome {
            Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (ResourceBodyResult ResourceError Unit)
              (ResourceBodyErr ResourceError Unit error);
            Ok metadata |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (ResourceBodyResult ResourceError Unit)
              (ResourceBodyOk ResourceError Unit MkUnit)
          }

        proc px7f_metadata_body (resource : Resource FsHandle)
          : HostIO AFull (ResourceBodyResult ResourceError Unit) visits [FS] =
          bind (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (Result ResourceError FileMetadata)
            (ResourceBodyResult ResourceError Unit)
            (resourceMetadata AFull resource)
            (\outcome. px7f_metadata_after outcome)

        fn px7f_release_after (outcome : Result ResourceError Unit)
          : HostIO AFull (ResourceBodyResult ResourceError Unit) =
          match outcome {
            Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (ResourceBodyResult ResourceError Unit)
              (ResourceBodyErr ResourceError Unit error);
            Ok unit |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (ResourceBodyResult ResourceError Unit)
              (ResourceBodyOk ResourceError Unit MkUnit)
          }

        proc px7f_early_release_body (resource : Resource FsHandle)
          : HostIO AFull (ResourceBodyResult ResourceError Unit) visits [FS] =
          bind (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (Result ResourceError Unit)
            (ResourceBodyResult ResourceError Unit)
            (release AFull resource)
            (\outcome. px7f_release_after outcome)

        proc px7f_after_escaped_bracket
          (bracket : ResourceBracketResult Unit (Resource FsHandle))
          : HostIO AFull (Result ResourceError FileMetadata) visits [FS] =
          match bracket {
            ResourceBracketOk resource |-> resourceMetadata AFull resource;
            ResourceBracketBodyError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (Result ResourceError FileMetadata)
              (Err ResourceError FileMetadata MalformedResource);
            ResourceBracketReleaseError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (Result ResourceError FileMetadata)
              (Err ResourceError FileMetadata error);
            ResourceBracketBodyAndReleaseError body_error release_error |->
              Ret (Coproduct (FSOp AFull) AmbientOp)
                (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                (Result ResourceError FileMetadata)
                (Err ResourceError FileMetadata release_error)
          }

        proc px7f_after_escaped_outer
          (outcome : Result FileError (ResourceBracketResult Unit (Resource FsHandle)))
          : HostIO AFull (Result ResourceError FileMetadata) visits [FS] =
          match outcome {
            Err open_error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (Result ResourceError FileMetadata)
              (Err ResourceError FileMetadata MalformedResource);
            Ok bracket |-> px7f_after_escaped_bracket bracket
          }

        proc px7f_escape_then_use (cap : Cap AFull) (path : Bytes)
          : HostIO AFull (Result ResourceError FileMetadata) visits [FS] =
          bind (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (Result FileError (ResourceBracketResult Unit (Resource FsHandle)))
            (Result ResourceError FileMetadata)
            (withResource AFull Unit (Resource FsHandle)
              cap path ResourceMetadata px7f_escape_body)
            (\outcome. px7f_after_escaped_outer outcome)
        "#,
    )
    .expect("public PX7-F body fixtures elaborate");
    env
}

fn eval_global(env: &ken_elaborator::ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    let id = env.globals[name];
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        other => panic!("`{name}` must be transparent, got {other:?}"),
    }
}

fn ctor(env: &ken_elaborator::ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    eval(
        &[],
        &Term::constructor(env.globals[name], vec![]),
        &env.env,
        store,
    )
}

fn with_resource(
    env: &ken_elaborator::ElabEnv,
    store: &mut EvalStore,
    mode: &str,
    body: &str,
) -> EvalVal {
    let mut function = eval_global(env, store, "withResource");
    function = apply(function, ctor(env, store, "AFull"), &env.env, store);
    let unit_type = eval(
        &[],
        &Term::indformer(env.globals["Unit"], vec![]),
        &env.env,
        store,
    );
    let resource_error_type = eval(
        &[],
        &Term::indformer(env.globals["ResourceError"], vec![]),
        &env.env,
        store,
    );
    let (error_type, result_type) = if body == "px7f_escape_body" {
        (unit_type, EvalVal::Neutral)
    } else {
        (resource_error_type, unit_type)
    };
    function = apply(function, error_type, &env.env, store);
    function = apply(function, result_type, &env.env, store);
    let cap = PosixHost::new_at(repo_root()).mint_fs_cap(AUTH_FULL);
    function = apply(function, EvalVal::Cap(cap), &env.env, store);
    function = apply(
        function,
        EvalVal::Bytes(b"conformance/fs/fixtures/three-lines.txt".to_vec()),
        &env.env,
        store,
    );
    function = apply(function, ctor(env, store, mode), &env.env, store);
    let body = eval_global(env, store, body);
    apply(function, body, &env.env, store)
}

fn drive(env: &ken_elaborator::ElabEnv, store: &mut EvalStore, tree: EvalVal) -> EvalVal {
    let ids = ConsoleIds::from_elab(env).expect("Console ABI");
    let fs = FSIds::from_elab(env).expect("FS ABI");
    let coproduct = CoproductIds {
        inl_id: env.globals["InL"],
        inr_id: env.globals["InR"],
    };
    let mut host = PosixHost::new_at(repo_root());
    ken_interp::run_io(
        tree,
        &mut host,
        &ids,
        Some(&fs),
        None,
        Some(&coproduct),
        &env.env,
        store,
    )
    .expect("public resource action drives")
}

fn result_payload<'a>(env: &ken_elaborator::ElabEnv, value: &'a EvalVal) -> &'a EvalVal {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.globals["Ok"] => &args[2],
        other => panic!("expected outer Ok, got {other:?}"),
    }
}

fn type_value(env: &ken_elaborator::ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    eval(
        &[],
        &Term::indformer(env.globals[name], vec![]),
        &env.env,
        store,
    )
}

fn apply_many(
    env: &ken_elaborator::ElabEnv,
    store: &mut EvalStore,
    name: &str,
    values: impl IntoIterator<Item = EvalVal>,
) -> EvalVal {
    values
        .into_iter()
        .fold(eval_global(env, store, name), |f, value| {
            apply(f, value, &env.env, store)
        })
}

fn apply_constructor_many(
    env: &ken_elaborator::ElabEnv,
    store: &mut EvalStore,
    name: &str,
    values: impl IntoIterator<Item = EvalVal>,
) -> EvalVal {
    values.into_iter().fold(ctor(env, store, name), |f, value| {
        apply(f, value, &env.env, store)
    })
}

fn injected_release_failure(env: &ken_elaborator::ElabEnv, store: &mut EvalStore) -> EvalVal {
    let identity = {
        let constructor = eval(
            &[],
            &Term::constructor(env.prelude_env.private_resource_trace_identity_id, vec![]),
            &env.env,
            store,
        );
        let constructor = apply(constructor, EvalVal::Int(0), &env.env, store);
        apply(constructor, EvalVal::Int(7), &env.env, store)
    };
    let io = apply_constructor_many(env, store, "Other", [EvalVal::Int(5)]);
    let kind = ctor(env, store, "FsHandle");
    apply_constructor_many(env, store, "ReleaseFailed", [kind, identity, io])
}

fn injected_release_result(env: &ken_elaborator::ElabEnv, store: &mut EvalStore) -> EvalVal {
    let error_type = type_value(env, store, "ResourceError");
    let unit_type = type_value(env, store, "Unit");
    let failure = injected_release_failure(env, store);
    apply_constructor_many(env, store, "Err", [error_type, unit_type, failure])
}

fn assert_injected_release_failure(env: &ken_elaborator::ElabEnv, value: &EvalVal) {
    let EvalVal::Ctor { id, args, .. } = value else {
        panic!("expected ReleaseFailed, got {value:?}");
    };
    assert_eq!(*id, env.globals["ReleaseFailed"]);
    assert!(matches!(
        &args[0],
        EvalVal::Ctor { id, .. } if *id == env.globals["FsHandle"]
    ));
    assert!(matches!(
        &args[1],
        EvalVal::Ctor { id, args, .. }
            if *id == env.prelude_env.private_resource_trace_identity_id
                && args.as_ref() == &[EvalVal::Int(0), EvalVal::Int(7)]
    ));
    assert!(matches!(
        &args[2],
        EvalVal::Ctor { id, args, .. }
            if *id == env.globals["Other"] && args.as_ref() == &[EvalVal::Int(5)]
    ));
}

#[test]
fn public_bracket_success_and_early_release_settle_once() {
    let env = env();
    for body in ["px7f_escape_body", "px7f_early_release_body"] {
        let mut store = EvalStore::new();
        let tree = with_resource(&env, &mut store, "ResourceMetadata", body);
        let value = drive(&env, &mut store, tree);
        let bracket = result_payload(&env, &value);
        assert!(matches!(
            bracket,
            EvalVal::Ctor { id, .. } if *id == env.globals["ResourceBracketOk"]
        ));
    }
}

#[test]
fn read_only_handle_metadata_surfaces_exact_right_not_held_masks() {
    let env = env();
    let mut store = EvalStore::new();
    let tree = with_resource(&env, &mut store, "ResourceRead", "px7f_metadata_body");
    let value = drive(&env, &mut store, tree);
    let bracket = result_payload(&env, &value);
    let EvalVal::Ctor { id, args, .. } = bracket else {
        panic!("expected bracket constructor, got {bracket:?}");
    };
    assert_eq!(*id, env.globals["ResourceBracketBodyError"]);
    let EvalVal::Ctor {
        id: error_id,
        args: error_args,
        ..
    } = &args[2]
    else {
        panic!("expected ResourceError payload, got {:?}", args[2]);
    };
    assert_eq!(*error_id, env.globals["RightNotHeld"]);
    assert_eq!(error_args.as_ref(), &[EvalVal::Int(32), EvalVal::Int(1)]);
}

#[test]
fn escaped_copy_is_legal_but_every_later_use_is_closed() {
    let env = env();
    let mut store = EvalStore::new();
    let mut action = eval_global(&env, &mut store, "px7f_escape_then_use");
    let cap = PosixHost::new_at(repo_root()).mint_fs_cap(AUTH_FULL);
    action = apply(action, EvalVal::Cap(cap), &env.env, &mut store);
    action = apply(
        action,
        EvalVal::Bytes(b"conformance/fs/fixtures/three-lines.txt".to_vec()),
        &env.env,
        &mut store,
    );
    let result = drive(&env, &mut store, action);
    let EvalVal::Ctor { id, args, .. } = result else {
        panic!("expected Result constructor, got {result:?}");
    };
    assert_eq!(id, env.globals["Err"]);
    assert!(matches!(
        &args[2],
        EvalVal::Ctor { id, .. } if *id == env.globals["Closed"]
    ));
}

#[test]
/// Caller-control only: these exact injected results exercise the bracket's
/// checked settlement functions; they do not claim an observed OS failure.
fn caller_control_release_failure_preserves_success_and_body_error_ordering() {
    let env = env();

    let mut success_store = EvalStore::new();
    let success_error_type = type_value(&env, &mut success_store, "ResourceError");
    let success_unit_type = type_value(&env, &mut success_store, "Unit");
    let success_unit = ctor(&env, &mut success_store, "MkUnit");
    let success_release = injected_release_result(&env, &mut success_store);
    let success = apply_many(
        &env,
        &mut success_store,
        "resource_settle_ok",
        [
            success_error_type,
            success_unit_type,
            success_unit,
            success_release,
        ],
    );
    let success_bracket = result_payload(&env, &success);
    let EvalVal::Ctor { id, args, .. } = success_bracket else {
        panic!("expected bracket constructor, got {success_bracket:?}");
    };
    assert_eq!(*id, env.globals["ResourceBracketReleaseError"]);
    assert_injected_release_failure(&env, &args[2]);

    let mut error_store = EvalStore::new();
    let body_error = ctor(&env, &mut error_store, "MalformedResource");
    let error_type = type_value(&env, &mut error_store, "ResourceError");
    let error_unit_type = type_value(&env, &mut error_store, "Unit");
    let error_release = injected_release_result(&env, &mut error_store);
    let combined = apply_many(
        &env,
        &mut error_store,
        "resource_settle_err",
        [error_type, error_unit_type, body_error, error_release],
    );
    let combined_bracket = result_payload(&env, &combined);
    let EvalVal::Ctor { id, args, .. } = combined_bracket else {
        panic!("expected bracket constructor, got {combined_bracket:?}");
    };
    assert_eq!(*id, env.globals["ResourceBracketBodyAndReleaseError"]);
    assert!(matches!(
        &args[2],
        EvalVal::Ctor { id, .. } if *id == env.globals["MalformedResource"]
    ));
    assert_injected_release_failure(&env, &args[3]);
}
