//! PX8-P checked Buffer acquisition and canonical settlement in the interpreter.

use std::path::PathBuf;

use ken_interp::{apply, eval, ConsoleIds, CoproductIds, EvalStore, EvalVal, FSIds, PosixHost};
use ken_kernel::{Decl, Term};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn env() -> ken_elaborator::ElabEnv {
    let mut env = ken_elaborator::ElabEnv::empty().expect("PX8-P prelude");
    // The escape discriminator reaches the same private checked release node
    // used by `withBuffer`'s finalizer without publishing a raw release name.
    env.globals.insert(
        "Px8pPrivateResourceRelease".to_string(),
        env.prelude_env.private_resource_release_id,
    );
    env.elaborate_file(
        r#"
fn px8p_ok_body (_resource : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

fn px8p_error_body (_resource : Resource Buffer)
  : HostIO AFull (ResourceBodyResult ResourceError Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult ResourceError Unit)
    (ResourceBodyErr ResourceError Unit InvalidBounds)

fn px8p_escape_body (resource : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit (Resource Buffer)) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit (Resource Buffer))
    (ResourceBodyOk Unit (Resource Buffer) resource)

proc px8p_private_release (resource : Resource Buffer)
  : HostIO AFull (Result ResourceError Unit) visits [FS] =
  Vis (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError Unit)
    (InL (FSOp AFull) AmbientOp
      (Px8pPrivateResourceRelease AFull Buffer resource))
    (\settled. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result ResourceError Unit) settled)

proc px8p_after_escape_bracket
  (bracket : ResourceBracketResult Unit (Resource Buffer))
  : HostIO AFull (Result ResourceError Unit) visits [FS] =
  match bracket {
    ResourceBracketOk resource |-> px8p_private_release resource;
    ResourceBracketBodyError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result ResourceError Unit) (Err ResourceError Unit MalformedResource);
    ResourceBracketReleaseError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result ResourceError Unit) (Err ResourceError Unit error);
    ResourceBracketBodyAndReleaseError body_error release_error |->
      Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result ResourceError Unit) (Err ResourceError Unit release_error)
  }

proc px8p_after_escape_outer
  (outcome : Result ResourceError
    (ResourceBracketResult Unit (Resource Buffer)))
  : HostIO AFull (Result ResourceError Unit) visits [FS] =
  match outcome {
    Err acquire_error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result ResourceError Unit) (Err ResourceError Unit acquire_error);
    Ok bracket |-> px8p_after_escape_bracket bracket
  }

proc px8p_escape_then_release (capacity : Int)
  : HostIO AFull (Result ResourceError Unit)
    visits [FS, BufferAllocate, ResourceRelease] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit (Resource Buffer)))
    (Result ResourceError Unit)
    (withBuffer AFull Unit (Resource Buffer) capacity px8p_escape_body)
    (\outcome. px8p_after_escape_outer outcome)
"#,
    )
    .expect("checked PX8-P interpreter fixtures elaborate");
    env.globals.remove("Px8pPrivateResourceRelease");
    env
}

fn eval_global(env: &ken_elaborator::ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    let id = env.globals[name];
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        other => panic!("`{name}` must be transparent, got {other:?}"),
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

fn with_buffer(
    env: &ken_elaborator::ElabEnv,
    store: &mut EvalStore,
    error_type: EvalVal,
    result_type: EvalVal,
    capacity: i64,
    body: &str,
) -> EvalVal {
    let mut function = eval_global(env, store, "withBuffer");
    function = apply(
        function,
        eval(
            &[],
            &Term::constructor(env.globals["AFull"], vec![]),
            &env.env,
            store,
        ),
        &env.env,
        store,
    );
    function = apply(function, error_type, &env.env, store);
    function = apply(function, result_type, &env.env, store);
    function = apply(function, EvalVal::Int(capacity), &env.env, store);
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
    .expect("checked buffer action drives")
}

fn assert_outer_error(env: &ken_elaborator::ElabEnv, value: &EvalVal, error: &str) {
    let EvalVal::Ctor { id, args, .. } = value else {
        panic!("expected Result error, got {value:?}");
    };
    assert_eq!(*id, env.globals["Err"]);
    assert!(matches!(
        &args[2],
        EvalVal::Ctor { id, .. } if *id == env.globals[error]
    ));
}

#[test]
fn positive_capacity_settles_normally_and_body_error_still_settles() {
    let env = env();

    let mut ok_store = EvalStore::new();
    let unit = type_value(&env, &mut ok_store, "Unit");
    let ok_tree = with_buffer(&env, &mut ok_store, unit.clone(), unit, 8, "px8p_ok_body");
    let ok = drive(&env, &mut ok_store, ok_tree);
    let EvalVal::Ctor { id, args, .. } = ok else {
        panic!("expected outer Result, got {ok:?}");
    };
    assert_eq!(id, env.globals["Ok"]);
    assert!(matches!(
        &args[2],
        EvalVal::Ctor { id, .. } if *id == env.globals["ResourceBracketOk"]
    ));

    let mut error_store = EvalStore::new();
    let error_type = type_value(&env, &mut error_store, "ResourceError");
    let unit = type_value(&env, &mut error_store, "Unit");
    let error_tree = with_buffer(
        &env,
        &mut error_store,
        error_type,
        unit,
        8,
        "px8p_error_body",
    );
    let error = drive(&env, &mut error_store, error_tree);
    let EvalVal::Ctor { id, args, .. } = error else {
        panic!("expected outer Result, got {error:?}");
    };
    assert_eq!(id, env.globals["Ok"]);
    assert!(matches!(
        &args[2],
        EvalVal::Ctor { id, args, .. }
            if *id == env.globals["ResourceBracketBodyError"]
                && matches!(&args[2], EvalVal::Ctor { id, .. }
                    if *id == env.globals["InvalidBounds"])
    ));
}

#[test]
fn malformed_and_policy_invalid_capacities_have_exact_distinct_errors() {
    let env = env();

    let mut malformed_store = EvalStore::new();
    let unit = type_value(&env, &mut malformed_store, "Unit");
    let tree = with_buffer(
        &env,
        &mut malformed_store,
        unit.clone(),
        unit,
        -1,
        "px8p_ok_body",
    );
    let result = drive(&env, &mut malformed_store, tree);
    assert_outer_error(&env, &result, "InvalidBounds");

    for capacity in [0, 1_048_577] {
        let mut store = EvalStore::new();
        let unit = type_value(&env, &mut store, "Unit");
        let tree = with_buffer(
            &env,
            &mut store,
            unit.clone(),
            unit,
            capacity,
            "px8p_ok_body",
        );
        let result = drive(&env, &mut store, tree);
        assert_outer_error(&env, &result, "BufferLimit");
    }
}

#[test]
fn escaped_copy_reaches_exact_closed_after_bracket_settlement() {
    let env = env();
    let mut store = EvalStore::new();
    let mut action = eval_global(&env, &mut store, "px8p_escape_then_release");
    action = apply(action, EvalVal::Int(8), &env.env, &mut store);
    let result = drive(&env, &mut store, action);
    assert_outer_error(&env, &result, "Closed");

    // The production namespace remains locked even though the test reached the
    // private checked identity by immutable GlobalId.
    assert!(!env.globals.contains_key("PrivateResourceRelease"));
    assert!(!env.globals.contains_key("Px8pPrivateResourceRelease"));
}
