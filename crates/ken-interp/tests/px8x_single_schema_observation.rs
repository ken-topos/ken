//! PX8-X sole public interpreter observation route.

use ken_interp::{apply, eval, ConsoleIds, CoproductIds, EvalStore, EvalVal, FSIds, PosixHost};
use ken_kernel::Decl;

fn fixture() -> ken_elaborator::ElabEnv {
    let mut env = ken_elaborator::ElabEnv::empty().expect("PX8-X prelude");
    env.elaborate_file(
        r#"
fn px8x_body (_resource : BufferHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

fn px8x_after
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
    Err _ |-> host_exit AFull (Failure 51);
    Ok bracket |-> match bracket {
      ResourceBracketOk _ |-> host_exit AFull Success;
      ResourceBracketBodyError _ |-> host_exit AFull (Failure 52);
      ResourceBracketReleaseError _ |-> host_exit AFull (Failure 53);
      ResourceBracketBodyAndReleaseError _ _ |-> host_exit AFull (Failure 54)
    }
  }

proc px8x_observed (capacity : Int)
  : HostIO AFull ExitCode visits [FS, BufferAllocate, ResourceRelease] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) ExitCode
    (withBuffer AFull Unit Unit capacity px8x_body)
    (\outcome. px8x_after outcome)
"#,
    )
    .expect("checked Buffer observation fixture elaborates");
    env
}

fn action(env: &ken_elaborator::ElabEnv, store: &mut EvalStore) -> EvalVal {
    let body = match env.env.lookup(env.globals["px8x_observed"]) {
        Some(Decl::Transparent { body, .. }) => body,
        other => panic!("PX8-X fixture must be transparent, got {other:?}"),
    };
    let function = eval(&[], body, &env.env, store);
    apply(function, EvalVal::Int(8), &env.env, store)
}

#[test]
fn real_buffer_bracket_exposes_ordered_target_bindings() {
    let env = fixture();
    let ids = ConsoleIds::from_elab(&env).expect("Console ABI");
    let fs = FSIds::from_elab(&env).expect("FS ABI");
    let coproduct = CoproductIds {
        inl_id: env.globals["InL"],
        inr_id: env.globals["InR"],
    };
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut store = EvalStore::new();
    let tree = action(&env, &mut store);
    let mut host = PosixHost::new_at(root);
    let observation = ken_interp::run_io_effect_observation(
        tree,
        &mut host,
        &ids,
        Some(&fs),
        None,
        Some(&coproduct),
        &env.env,
        &mut store,
        env.globals["Success"],
        env.globals["Failure"],
    );

    assert_eq!(observation.exit_status, 0);
    assert_eq!(
        observation.terminal_exit,
        ken_host::TerminalExitClass::NormalReturn
    );
    assert_eq!(
        observation
            .effect_trace
            .iter()
            .map(|event| (event.sequence, event.operation))
            .collect::<Vec<_>>(),
        vec![
            (0, ken_host::HostOpV1::BufferAllocate),
            (1, ken_host::HostOpV1::ResourceRelease),
        ]
    );
    let allocated = observation.effect_trace[0].resource_bindings.as_slice();
    let released = observation.effect_trace[1].resource_bindings.as_slice();
    assert_eq!(allocated.len(), 1);
    assert_eq!(released.len(), 1);
    assert_eq!(allocated[0].0, ken_host::ResourceBindingRole::Target);
    assert_eq!(released[0].0, ken_host::ResourceBindingRole::Target);
    assert_eq!(allocated[0].1, released[0].1);
}
