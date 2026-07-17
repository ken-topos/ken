use std::collections::BTreeSet;

use ken_elaborator::compiler_driver::{
    compile_checked_target_denotation, CheckedTargetDenotationError, CompilerSource,
};
use ken_elaborator::{emit_checked_target_export, ExportError};

const RESOURCE_PRODUCER: &str = r#"
fn after_metadata (outcome : Result ResourceError FileMetadata)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
    Err _ |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit);
    Ok _ |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit)
  }

proc body (resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit)
    visits [FS, FsHandleMetadata] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError FileMetadata) (ResourceBodyResult Unit Unit)
    (resourceMetadata AFull resource) (\outcome. after_metadata outcome)

proc target (cap : Cap AFull) (path : Bytes)
  : HostIO AFull (Result FileError (ResourceBracketResult Unit Unit))
    visits [FS, FsOpen, FsHandleMetadata, ResourceRelease] =
  withResource AFull Unit Unit cap path ResourceMetadata body
"#;

const DECLARED_HEADROOM_WITHOUT_METADATA: &str = r#"
fn body (resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit)

proc target (cap : Cap AFull) (path : Bytes)
  : HostIO AFull (Result FileError (ResourceBracketResult Unit Unit))
    visits [FS, FsOpen, FsHandleMetadata, ResourceRelease] =
  withResource AFull Unit Unit cap path ResourceMetadata body
"#;

fn export(source: &str) -> Result<ken_elaborator::BehavioralExport, ExportError> {
    export_target("b1_exact", source, "target")
}

fn export_target(
    package: &str,
    source: &str,
    target: &str,
) -> Result<ken_elaborator::BehavioralExport, ExportError> {
    let denotation = compile_checked_target_denotation(
        package,
        CompilerSource::new("fixture.ken", source),
        target,
    )
    .expect("checked target denotation");
    emit_checked_target_export(&denotation, &[], &BTreeSet::new(), vec![], vec![])
}

#[test]
fn real_resource_performs_derive_the_exact_lifecycle_alphabet() {
    let export = export(RESOURCE_PRODUCER).expect("exact resource export");
    for operation in ["FsOpen", "FsHandleMetadata", "ResourceRelease"] {
        assert!(export.alphabet.contains(operation), "missing {operation}");
    }
    assert!(!export.alphabet.contains("FS"));
}

#[test]
fn declared_headroom_cannot_manufacture_a_metadata_perform_node() {
    assert!(matches!(
        export(DECLARED_HEADROOM_WITHOUT_METADATA),
        Err(ExportError::TemporalSymbolOutsideAlphabet { symbol })
            if symbol == "FsHandleMetadata"
    ));
}

#[test]
fn ordinary_declared_headroom_is_not_an_alphabet_source() {
    let headroom = export(
        r#"
proc target (value : Unit) : Unit visits [Console] = value
"#,
    )
    .expect("legal ordinary declared headroom");
    assert!(headroom.alphabet.is_empty());

    let performed = export(
        r#"
proc target (_value : Unit)
  : HostIO AFull (Result IOError Unit) visits [Console] =
  host_console AFull (Result IOError Unit) (flush Stdout)
"#,
    )
    .expect("real ordinary perform");
    assert_eq!(
        performed.alphabet,
        BTreeSet::from(["ConsoleFlush".to_string()])
    );
}

#[test]
fn non_host_l5_perform_uses_its_typed_inductive_identity() {
    let ping_source = r#"
data LocalOp = Ping | Pong

fn local_resp (_op : LocalOp) : Type = Unit

proc target (_value : Unit)
  : ITree LocalOp local_resp Unit visits [Local] =
  Vis LocalOp local_resp Unit Ping
    (\_response. Ret LocalOp local_resp Unit MkUnit)
"#;
    let ping = export(ping_source).expect("closed non-host L5 perform");
    let pong_source = ping_source.replacen("Unit Ping", "Unit Pong", 1);
    let pong = export(&pong_source).expect("same-family L5 sibling perform");

    assert_eq!(ping.alphabet.len(), 1);
    assert_eq!(pong.alphabet.len(), 1);
    let ping_signature = ping.alphabet.first().expect("one Ping signature");
    let pong_signature = pong.alphabet.first().expect("one Pong signature");
    assert!(ping_signature.starts_with("L5:"));
    assert!(ping_signature.contains("LocalOp"));
    assert!(ping_signature.contains("Ping"));
    assert!(pong_signature.contains("Pong"));
    assert_ne!(ping_signature, pong_signature);
    assert_ne!(ping.hash, pong.hash);
    assert!(!ping.alphabet.contains(pong_signature));
}

#[test]
fn same_family_host_operations_have_distinct_signatures_and_hashes() {
    let flush = export(
        r#"
proc target (_value : Unit)
  : HostIO AFull (Result IOError Unit) visits [Console] =
  host_console AFull (Result IOError Unit) (flush Stdout)
"#,
    )
    .expect("ConsoleFlush export");
    let terminal = export(
        r#"
proc target (_value : Unit)
  : HostIO AFull Bool visits [Console] =
  host_console AFull Bool (is_terminal Stdout)
"#,
    )
    .expect("ConsoleIsTerminal export");

    assert_eq!(flush.alphabet, BTreeSet::from(["ConsoleFlush".to_string()]));
    assert_eq!(
        terminal.alphabet,
        BTreeSet::from(["ConsoleIsTerminal".to_string()])
    );
    assert!(!flush.alphabet.contains("ConsoleIsTerminal"));
    assert_ne!(flush.hash, terminal.hash);
}

#[test]
fn dynamic_operation_input_fails_closed_without_family_widening() {
    assert_dynamic_reject(
        r#"
data LocalOp = Ping

fn local_resp (_op : LocalOp) : Type = Unit

proc target (operation : LocalOp)
  : ITree LocalOp local_resp Unit visits [Local] =
  Vis LocalOp local_resp Unit operation
    (\_response. Ret LocalOp local_resp Unit MkUnit)
"#,
    );
}

fn assert_dynamic_reject(source: &str) {
    let result = compile_checked_target_denotation(
        "b1_dynamic_reject",
        CompilerSource::new("dynamic.ken", source),
        "target",
    );
    assert!(matches!(
        result,
        Err(CheckedTargetDenotationError::NonClosedPerformGraph { .. })
    ));
}

#[test]
fn wrapped_operation_tree_and_callback_inputs_fail_closed_transitively() {
    assert_dynamic_reject(
        r#"
data LocalOp = Ping
data OperationBox = Wrap LocalOp
fn local_resp (_op : LocalOp) : Type = Unit
proc target (boxed : OperationBox)
  : ITree LocalOp local_resp Unit visits [Local] =
  match boxed {
    Wrap operation |-> Vis LocalOp local_resp Unit operation
      (\_response. Ret LocalOp local_resp Unit MkUnit)
  }
"#,
    );

    assert_dynamic_reject(
        r#"
data LocalOp = Ping
fn local_resp (_op : LocalOp) : Type = Unit
data TreeBox = WrapTree (ITree LocalOp local_resp Unit)
proc target (boxed : TreeBox)
  : ITree LocalOp local_resp Unit visits [Local] =
  match boxed { WrapTree tree |-> tree }
"#,
    );

    assert_dynamic_reject(
        r#"
data LocalOp = Ping
fn local_resp (_op : LocalOp) : Type = Unit
data CallbackBox = WrapCallback (Unit -> ITree LocalOp local_resp Unit)
proc target (boxed : CallbackBox)
  : ITree LocalOp local_resp Unit visits [Local] =
  match boxed { WrapCallback callback |-> callback MkUnit }
"#,
    );
}

#[test]
fn target_closure_follows_callback_and_excludes_unused_sibling() {
    let closure = r#"
proc after_flush (_outcome : Result IOError Unit)
  : HostIO AFull Bool visits [Console] =
  host_console AFull Bool (is_terminal Stdout)

proc unused_sibling (_value : Unit)
  : HostIO AFull Instant visits [Clock] =
  host_clock AFull Instant wall_now

proc target (_value : Unit)
  : HostIO AFull Bool visits [Console] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result IOError Unit) Bool
    (host_console AFull (Result IOError Unit) (flush Stdout))
    (\outcome. after_flush outcome)
"#;
    let without_unused = closure.replace(
        r#"proc unused_sibling (_value : Unit)
  : HostIO AFull Instant visits [Clock] =
  host_clock AFull Instant wall_now

"#,
        "",
    );

    let with = export_target("b1_closure", closure, "target").expect("closed callback graph");
    let without = export_target("b1_closure", &without_unused, "target")
        .expect("same graph without unused sibling");
    assert_eq!(
        with.alphabet,
        BTreeSet::from(["ConsoleFlush".to_string(), "ConsoleIsTerminal".to_string(),])
    );
    assert!(!with.alphabet.contains("ClockWallNow"));
    assert_eq!(
        with.hash, without.hash,
        "unused admitted siblings are not population"
    );
}

#[test]
fn recursive_cases_contribute_each_static_perform_identity_as_a_set() {
    let export = export(
        r#"
proc walk (fuel : Nat)
  : HostIO AFull Instant visits [Console, Clock] =
  match fuel {
    Zero |-> host_clock AFull Instant wall_now;
    Suc smaller |-> bind (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result IOError Unit) Instant
      (host_console AFull (Result IOError Unit) (flush Stdout))
      (\_outcome. walk smaller)
  }

proc target (fuel : Nat)
  : HostIO AFull Instant visits [Console, Clock] = walk fuel
"#,
    )
    .expect("finite recursive checked graph");

    assert_eq!(
        export.alphabet,
        BTreeSet::from(["ClockWallNow".to_string(), "ConsoleFlush".to_string()]),
        "all retained cases are traversed without family widening or duplicates"
    );
}
