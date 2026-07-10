use ken_elaborator::parser::parse_decls;
use ken_elaborator::{BinOp, ConstructorSignatureArg, Decl, ExplicitDataCtor, Expr, Type};

fn single_decl(src: &str) -> Decl {
    let mut decls = parse_decls(src).expect("source must parse");
    assert_eq!(decls.len(), 1);
    decls.remove(0)
}

fn app_head(expr: &Expr) -> &str {
    match expr {
        Expr::ECon(name, _) | Expr::EVar(name, _) => name,
        Expr::EApp(f, _, _) => app_head(f),
        other => panic!("expected application or name, got {other:?}"),
    }
}

fn app_arg_count(expr: &Expr) -> usize {
    match expr {
        Expr::EApp(f, _, _) => 1 + app_arg_count(f),
        _ => 0,
    }
}

#[test]
fn explicit_family_vec_preserves_constructor_signature_shape() {
    let decl = single_decl(
        r#"
        data Vec (A : Type) : Nat -> Type where {
          VNil  : Vec A 0;
          VCons : (n : Nat) -> A -> Vec A n -> Vec A (n+1)
        }
        "#,
    );

    let Decl::ExplicitDataDecl {
        name,
        params,
        family,
        ctors,
        ..
    } = decl
    else {
        panic!("expected explicit family declaration");
    };

    assert_eq!(name, "Vec");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].names, vec!["A"]);
    assert!(matches!(params[0].ty, Type::TUniv(None, _)));
    assert!(matches!(family, Type::TArr(_, _, _)));
    assert_eq!(ctors.len(), 2);

    let ExplicitDataCtor::Signature {
        name: nil_name,
        signature: nil_sig,
        ..
    } = &ctors[0]
    else {
        panic!("expected VNil signature");
    };
    assert_eq!(nil_name, "VNil");
    assert!(nil_sig.args.is_empty());
    assert_eq!(app_head(&nil_sig.result), "Vec");
    assert_eq!(app_arg_count(&nil_sig.result), 2);

    let ExplicitDataCtor::Signature {
        name: cons_name,
        signature: cons_sig,
        ..
    } = &ctors[1]
    else {
        panic!("expected VCons signature");
    };
    assert_eq!(cons_name, "VCons");
    assert_eq!(cons_sig.args.len(), 3);
    match &cons_sig.args[0] {
        ConstructorSignatureArg::Explicit(binder) => assert_eq!(binder.names, vec!["n"]),
        other => panic!("expected named constructor binder, got {other:?}"),
    }
    assert!(matches!(
        cons_sig.args[1],
        ConstructorSignatureArg::Anonymous(_)
    ));
    assert!(matches!(
        cons_sig.args[2],
        ConstructorSignatureArg::Anonymous(_)
    ));
    assert_eq!(app_head(&cons_sig.result), "Vec");
    assert_eq!(app_arg_count(&cons_sig.result), 2);
}

#[test]
fn proof_carrying_constructor_signature_parses_as_telescope() {
    let decl = single_decl(
        r#"
        data CheckedSource : Type where {
          MkCheckedSource :
            (sid : SourceId) ->
            (bs : Bytes) ->
            (len : Nat) ->
            UnitByteLength bs ->
            IsUtf8 bs ->
            SourceLength bs len ->
            CheckedSource
        }
        "#,
    );

    let Decl::ExplicitDataDecl { ctors, .. } = decl else {
        panic!("expected explicit family declaration");
    };
    let ExplicitDataCtor::Signature {
        name, signature, ..
    } = &ctors[0]
    else {
        panic!("expected explicit constructor signature");
    };

    assert_eq!(name, "MkCheckedSource");
    assert_eq!(signature.args.len(), 6);
    for (idx, expected) in ["sid", "bs", "len"].into_iter().enumerate() {
        match &signature.args[idx] {
            ConstructorSignatureArg::Explicit(binder) => {
                assert_eq!(binder.names, vec![expected]);
            }
            other => panic!("expected explicit binder {expected}, got {other:?}"),
        }
    }
    assert!(signature.args[3..]
        .iter()
        .all(|arg| matches!(arg, ConstructorSignatureArg::Anonymous(_))));
    assert_eq!(app_head(&signature.result), "CheckedSource");
}

#[test]
fn implicit_constructor_binder_is_preserved() {
    let decl = single_decl(
        r#"
        data Box (A : Type) : Type where {
          Mk : {x : A} -> Box A
        }
        "#,
    );

    let Decl::ExplicitDataDecl { ctors, .. } = decl else {
        panic!("expected explicit family declaration");
    };
    let ExplicitDataCtor::Signature { signature, .. } = &ctors[0] else {
        panic!("expected explicit constructor signature");
    };

    assert_eq!(signature.args.len(), 1);
    match &signature.args[0] {
        ConstructorSignatureArg::Implicit(binder) => assert_eq!(binder.names, vec!["x"]),
        other => panic!("expected implicit constructor binder, got {other:?}"),
    }
    assert_eq!(app_head(&signature.result), "Box");
}

#[test]
fn explicit_where_block_accepts_simple_default_result_constructors() {
    let decl = single_decl(
        r#"
        data Box (A : Type) : Type where {
          Mk A;
          Empty
        }
        "#,
    );

    let Decl::ExplicitDataDecl { ctors, .. } = decl else {
        panic!("expected explicit family declaration");
    };
    assert_eq!(ctors.len(), 2);
    match &ctors[0] {
        ExplicitDataCtor::Simple(ctor) => {
            assert_eq!(ctor.name, "Mk");
            assert_eq!(ctor.args.len(), 1);
        }
        other => panic!("expected simple constructor sugar, got {other:?}"),
    }
    match &ctors[1] {
        ExplicitDataCtor::Simple(ctor) => assert_eq!(ctor.name, "Empty"),
        other => panic!("expected simple constructor sugar, got {other:?}"),
    }
}

#[test]
fn legacy_data_accepts_named_constructor_field_sugar() {
    let decl = single_decl("data Point = MkPoint { x : Int, y : Int }");
    let Decl::DataDecl { ctors, .. } = decl else {
        panic!("expected legacy data declaration");
    };
    assert_eq!(ctors.len(), 1);
    assert_eq!(ctors[0].name, "MkPoint");
    assert_eq!(ctors[0].args.len(), 2);
    assert_eq!(
        ctors[0].field_labels.as_ref().expect("field labels"),
        &vec!["x".to_string(), "y".to_string()]
    );
}

#[test]
fn explicit_where_simple_constructor_accepts_named_field_sugar() {
    let decl = single_decl(
        r#"
        data PairBox (A : Type) : Type where {
          PairBoxed { first : A, second : A }
        }
        "#,
    );
    let Decl::ExplicitDataDecl { ctors, .. } = decl else {
        panic!("expected explicit family declaration");
    };
    match &ctors[0] {
        ExplicitDataCtor::Simple(ctor) => {
            assert_eq!(ctor.name, "PairBoxed");
            assert_eq!(ctor.args.len(), 2);
            assert_eq!(
                ctor.field_labels.as_ref().expect("field labels"),
                &vec!["first".to_string(), "second".to_string()]
            );
        }
        other => panic!("expected simple constructor sugar, got {other:?}"),
    }
}

#[test]
fn named_constructor_field_sugar_rejects_duplicate_labels() {
    let err = parse_decls("data Point = MkPoint { x : Int, x : Int }")
        .expect_err("duplicate constructor field labels must reject");
    let msg = format!("{err}");
    assert!(
        msg.contains("duplicate field `x`") && msg.contains("MkPoint"),
        "{msg}"
    );
}

#[test]
fn explicit_signature_constructor_rejects_record_style_field_list() {
    let err =
        parse_decls("data Box (A : Type) : Type where { Mk : { value : A, other : A } -> Box A }")
            .expect_err("record-style field lists inside explicit signatures are out of scope");
    let msg = format!("{err}");
    assert!(msg.contains("found Comma"), "{msg}");
}

#[test]
fn legacy_data_form_stays_simple_and_rejects_explicit_signatures() {
    let legacy = single_decl("data Box A = Mk A");
    let Decl::DataDecl {
        name,
        type_params,
        ctors,
        ..
    } = legacy
    else {
        panic!("expected legacy data declaration");
    };
    assert_eq!(name, "Box");
    assert_eq!(type_params, vec!["A"]);
    assert_eq!(ctors.len(), 1);
    assert_eq!(ctors[0].name, "Mk");
    assert_eq!(ctors[0].args.len(), 1);
    assert!(ctors[0].field_labels.is_none());

    parse_decls("data Box (A : Type) : Type where { Mk : A -> Box A }")
        .expect("explicit where form must parse");

    let err = parse_decls("data Box A = Mk : A -> Box A").expect_err(
        "legacy data form must reject explicit constructor signatures at syntax boundary",
    );
    let msg = format!("{err}");
    assert!(msg.contains("found Colon"), "{msg}");
}

#[test]
fn explicit_family_rejects_bare_head_parameters() {
    let err = parse_decls("data Box A : Type where { Mk : A -> Box A }")
        .expect_err("explicit family parameters must be written as binders");
    let msg = format!("{err}");
    assert!(msg.contains("parameters must use binder syntax"), "{msg}");
}

#[test]
fn explicit_family_accepts_empty_constructor_block() {
    // FR-1 (`docs/program/wp/ds-1-findings-remediation.md`): a zero-
    // constructor `data` declares an empty type at the surface — the
    // kernel already admits zero-constructor inductives (that's how
    // `Empty` bootstrapped in DS-1); this is a pure parser/surface gap.
    let decl = single_decl("data Empty : Type where {}");
    let Decl::ExplicitDataDecl { name, ctors, .. } = decl else {
        panic!("expected ExplicitDataDecl, got {decl:?}");
    };
    assert_eq!(name, "Empty");
    assert!(ctors.is_empty());
}

#[test]
fn constructor_result_indices_use_expression_surface() {
    let decl = single_decl(
        r#"
        data Vec (A : Type) : Nat -> Type where {
          VCons : (n : Nat) -> Vec A (n+1)
        }
        "#,
    );

    let Decl::ExplicitDataDecl { ctors, .. } = decl else {
        panic!("expected explicit family declaration");
    };
    let ExplicitDataCtor::Signature { signature, .. } = &ctors[0] else {
        panic!("expected explicit constructor signature");
    };
    let Expr::EApp(_, index, _) = &signature.result else {
        panic!("expected indexed Vec result");
    };
    let Expr::EBinOp(BinOp::Add, _, rhs, _) = &**index else {
        panic!("expected n+1 as the final result index, got {index:?}");
    };
    assert!(matches!(**rhs, Expr::ENumLit(_, _)));
}
