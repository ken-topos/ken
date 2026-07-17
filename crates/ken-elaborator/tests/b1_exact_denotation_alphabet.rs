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
    assert_eq!(performed.alphabet, BTreeSet::from(["Console".to_string()]));
}

#[test]
fn non_host_l5_perform_uses_its_typed_inductive_identity() {
    let export = export(
        r#"
data LocalOp = Ping

fn local_resp (_op : LocalOp) : Type = Unit

proc target (_value : Unit)
  : ITree LocalOp local_resp Unit visits [Local] =
  Vis LocalOp local_resp Unit Ping
    (\_response. Ret LocalOp local_resp Unit MkUnit)
"#,
    )
    .expect("closed non-host L5 perform");

    assert_eq!(export.alphabet, BTreeSet::from(["Local".to_string()]));
}

#[test]
fn dynamic_operation_input_fails_closed_without_family_widening() {
    let result = compile_checked_target_denotation(
        "b1_dynamic_reject",
        CompilerSource::new(
            "dynamic.ken",
            r#"
data LocalOp = Ping

fn local_resp (_op : LocalOp) : Type = Unit

proc target (operation : LocalOp)
  : ITree LocalOp local_resp Unit visits [Local] =
  Vis LocalOp local_resp Unit operation
    (\_response. Ret LocalOp local_resp Unit MkUnit)
"#,
        ),
        "target",
    );
    assert!(matches!(
        result,
        Err(CheckedTargetDenotationError::NonClosedPerformGraph { .. })
    ));
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
    assert_eq!(with.alphabet, BTreeSet::from(["Console".to_string()]));
    assert!(!with.alphabet.contains("Clock"));
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
        BTreeSet::from(["Clock".to_string(), "Console".to_string()]),
        "all retained cases are traversed without family widening or duplicates"
    );
}
