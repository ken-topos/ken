use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::{GlobalId, KernelError, Level, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

fn elab_ok(env: &mut ElabEnv, src: &str) -> GlobalId {
    env.elaborate_decl(src)
        .unwrap_or_else(|e| panic!("elaboration failed: {}", e))
}

fn elab_file_ok(env: &mut ElabEnv, src: &str) {
    env.elaborate_file(src)
        .unwrap_or_else(|e| panic!("file elaboration failed: {}", e));
}

fn expect_err(env: &mut ElabEnv, src: &str) -> String {
    env.elaborate_decl(src)
        .expect_err("declaration unexpectedly elaborated")
        .to_string()
}

#[test]
fn non_indexed_explicit_family_elaborates_and_constructor_is_usable() {
    let mut env = mk_env();
    let id = elab_ok(
        &mut env,
        "data Box (A : Type) : Type where { Boxed : A -> Box A }",
    );

    let ind = env
        .env
        .inductive(id)
        .expect("Box should be an inductive family");
    assert_eq!(ind.params.len(), 1);
    assert!(ind.indices.is_empty());
    assert_eq!(ind.constructors.len(), 1);
    assert_eq!(ind.constructors[0].args.len(), 1);
    assert!(ind.constructors[0].target_indices.is_empty());

    elab_ok(&mut env, "let boxed : Box Int = Boxed Int 3");
}

#[test]
fn legacy_named_constructor_field_sugar_lowers_to_positional_constructor() {
    let mut env = mk_env();
    let id = elab_ok(&mut env, "data Point = MkPoint { x : Int, y : Int }");

    let ind = env.env.inductive(id).expect("Point should elaborate");
    assert!(ind.indices.is_empty());
    assert_eq!(ind.constructors.len(), 1);
    assert_eq!(ind.constructors[0].args.len(), 2);

    elab_ok(&mut env, "let point : Point = MkPoint 1 2");
}

#[test]
fn explicit_where_named_constructor_field_sugar_lowers_to_positional_constructor() {
    let mut env = mk_env();
    let id = elab_ok(
        &mut env,
        r#"
        data PairBox (A : Type) : Type where {
          PairBoxed { first : A, second : A }
        }
        "#,
    );

    let ind = env.env.inductive(id).expect("PairBox should elaborate");
    assert_eq!(ind.params.len(), 1);
    assert!(ind.indices.is_empty());
    assert_eq!(ind.constructors.len(), 1);
    assert_eq!(ind.constructors[0].args.len(), 2);

    elab_ok(&mut env, "let pairBox : PairBox Int = PairBoxed Int 1 2");
}

#[test]
fn indexed_vector_family_records_indices_and_constructor_targets() {
    let mut env = mk_env();
    let id = elab_ok(
        &mut env,
        r#"
        data Vector (A : Type) : Nat -> Type where {
          EmptyVector : Vector A 0;
          ConsVector : (n : Nat) -> A -> Vector A n -> Vector A (n+1)
        }
        "#,
    );

    let ind = env
        .env
        .inductive(id)
        .expect("Vector should be an inductive family");
    assert_eq!(ind.params.len(), 1);
    assert_eq!(ind.indices.len(), 1);
    assert_eq!(ind.constructors.len(), 2);
    assert_eq!(ind.constructors[0].target_indices.len(), 1);
    assert_eq!(ind.constructors[1].args.len(), 3);
    assert_eq!(ind.constructors[1].target_indices.len(), 1);

    let (head, args) = peel_app(&ind.constructors[1].target_indices[0]);
    assert_eq!(args.len(), 1, "Suc target should carry n");
    assert_eq!(
        *args[0],
        Term::var(2),
        "n should be in scope for the result index"
    );
    assert!(matches!(head, Term::Constructor { .. }));
}

#[test]
fn proof_carrying_constructor_telescope_elaborates_with_prior_binders_in_scope() {
    let mut env = mk_env();
    elab_file_ok(
        &mut env,
        r#"
        data UnitByteLength (bs : Bytes) : Type where {
          UnitByteLengthOk : UnitByteLength bs
        }

        data IsUtf8 (bs : Bytes) : Type where {
          IsUtf8Ok : IsUtf8 bs
        }

        data SourceLength (bs : Bytes) (len : Nat) : Type where {
          SourceLengthOk : SourceLength bs len
        }

        data CheckedSource : Type where {
          CheckedSourceMk :
            (bs : Bytes) ->
            (len : Nat) ->
            UnitByteLength bs ->
            IsUtf8 bs ->
            SourceLength bs len ->
            CheckedSource
        }
        "#,
    );

    let id = env.globals["CheckedSource"];
    let ind = env
        .env
        .inductive(id)
        .expect("CheckedSource should elaborate");
    assert_eq!(ind.constructors[0].args.len(), 5);
}

#[test]
fn bad_constructor_result_targets_are_surface_errors() {
    let cases = [
        (
            "wrong family head",
            "data WrongHead (A : Type) : Nat -> Type where { BadTarget : List A }",
        ),
        (
            "changed parameter",
            "data ChangedParam (A : Type) : Nat -> Type where { BadTarget : ChangedParam Bool Zero }",
        ),
        (
            "too few indices",
            "data TooFew (A : Type) : Nat -> Type where { BadTarget : TooFew A }",
        ),
        (
            "too many indices",
            "data TooMany (A : Type) : Nat -> Type where { BadTarget : TooMany A Zero Zero }",
        ),
        (
            "non family result",
            "data NonFamily (A : Type) : Type where { BadTarget : A }",
        ),
    ];

    for (label, src) in cases {
        let mut env = mk_env();
        let err = expect_err(&mut env, src);
        assert!(
            err.contains("bad constructor result target")
                && err.contains("BadTarget")
                && (err.contains("WrongHead")
                    || err.contains("ChangedParam")
                    || err.contains("TooFew")
                    || err.contains("TooMany")
                    || err.contains("NonFamily")),
            "{label}: unexpected diagnostic: {err}"
        );
    }
}

#[test]
fn same_family_occurrence_in_result_index_rejects_before_install() {
    let mut env = mk_env();
    let result = env.elaborate_decl("data Bad : Type -> Type where { BadMk : Bad (Bad Int) }");
    let err = result.expect_err("same-family target index should reject");
    let rendered = err.to_string();
    assert!(
        !rendered.contains("bad constructor result target"),
        "same-family target index should no longer reject through surface target validation: {rendered}"
    );
    assert!(
        matches!(
            &err,
            ElabError::KernelRejected {
                error: KernelError::PositivityViolation(_),
                ..
            }
        ),
        "same-family target index should reach kernel positivity, got {err:?}"
    );
    assert!(
        rendered.contains("kernel rejected")
            && rendered.contains("D occurs in constructor")
            && rendered.contains("target index"),
        "unexpected diagnostic: {rendered}"
    );
    assert!(
        !env.globals.contains_key("Bad"),
        "rejected family should not be installed"
    );
    assert!(
        !env.globals.contains_key("BadMk"),
        "rejected constructor should not be installed"
    );
}

#[test]
fn negative_recursive_occurrence_rejects_through_kernel_gate() {
    let mut env = mk_env();
    let err = expect_err(
        &mut env,
        "data Bad : Type where { BadMk : (Bad -> Bool) -> Bad }",
    );
    assert!(
        err.contains("kernel rejected") && err.contains("non-strictly-positive occurrence"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn same_level_universe_constructor_rejects_before_decoder_can_form() {
    let mut env = mk_env();
    let error = env
        .elaborate_file(
            r#"
            data D : Type where {
              C : (s : Type) -> D
            }

            fn decode (d : D) : Type =
              match d { C s ↦ s }
            "#,
        )
        .expect_err("same-level universe family must be rejected");

    assert!(
        matches!(
            &error,
            ElabError::KernelRejected {
                error: KernelError::ConstructorUniverseViolation {
                    argument,
                    family,
                },
                ..
            } if *argument == Level::zero().suc() && *family == Level::zero()
        ),
        "expected the constructor-universe gate, got {error:?}"
    );
    assert_eq!(
        error.to_string(),
        "kernel rejected at 13-82: constructor argument universe suc 0 exceeds family universe 0"
    );
}

#[test]
fn legacy_simple_data_still_elaborates() {
    let mut env = mk_env();
    let id = elab_ok(&mut env, "data MaybeNumber = NoNumber | SomeNumber Int");
    let ind = env
        .env
        .inductive(id)
        .expect("legacy data should still elaborate");
    assert!(ind.indices.is_empty());
    assert_eq!(ind.constructors.len(), 2);

    elab_ok(
        &mut env,
        "let answer : Int = match SomeNumber 5 { SomeNumber x |-> x ; NoNumber |-> 0 }",
    );
}

const VECTOR_DECL: &str = r#"
data Vector (A : Type) : Nat -> Type where {
  EmptyVector : Vector A Zero;
  ConsVector : (n : Nat) -> A -> Vector A n -> Vector A (Suc n)
}
"#;

#[test]
fn indexed_impossible_constructor_may_be_omitted_from_non_empty_vector_match() {
    let mut env = mk_env();
    elab_ok(&mut env, VECTOR_DECL);

    let head_id = elab_ok(
        &mut env,
        r#"
        fn vectorHead (A : Type) (n : Nat) (v : Vector A (Suc n)) : A =
          match v { ConsVector m x xs |-> x }
        "#,
    );

    let body = env.env.transparent_body(head_id).expect("transparent").1;
    let body = peel_lams(&body, 3);
    let elim = match body {
        Term::App(f, proof) => {
            assert!(
                matches!(proof.as_ref(), Term::Refl(_)),
                "indexed elim result must be applied to generated equality evidence"
            );
            f.as_ref()
        }
        other => panic!("expected indexed elim applied to equality evidence, got {other:?}"),
    };
    let Term::Elim {
        motive,
        methods,
        indices,
        ..
    } = elim
    else {
        panic!("expected dependent vector elim, got {elim:?}");
    };
    assert_eq!(
        indices.len(),
        1,
        "indexed match must pass the scrutinee index"
    );
    assert_eq!(methods.len(), 2);
    assert!(
        contains_absurd(&methods[0]),
        "omitted EmptyVector method must discharge through absurdity"
    );
    assert!(
        motive_has_index_and_scrutinee_lambdas(motive),
        "motive must abstract the index before the scrutinee"
    );
}

#[test]
fn concrete_non_empty_vector_index_omits_empty_constructor() {
    let mut env = mk_env();
    elab_ok(&mut env, VECTOR_DECL);

    elab_ok(
        &mut env,
        r#"
        fn vectorHeadZero (A : Type) (v : Vector A (Suc Zero)) : A =
          match v { ConsVector m x xs |-> x }
        "#,
    );
}

#[test]
fn dependent_index_telescope_lifts_prior_index_in_motive_premise() {
    let mut env = mk_env();
    elab_file_ok(
        &mut env,
        r#"
        data IsZero : Nat -> Type where {
          IsZeroZero : IsZero Zero
        }

        data DepIndex : (n : Nat) -> IsZero n -> Type where {
          DepMk : DepIndex Zero IsZeroZero
        }

        const depValue : DepIndex Zero IsZeroZero = DepMk

        fn depHead (p : IsZero Zero) (x : DepIndex Zero p) : Nat =
          match x { DepMk |-> Zero }
        "#,
    );
}

#[test]
fn indexed_head_rejects_empty_vector_application() {
    let mut env = mk_env();
    elab_ok(&mut env, VECTOR_DECL);
    elab_ok(
        &mut env,
        r#"
        fn vectorHead (A : Type) (n : Nat) (v : Vector A (Suc n)) : A =
          match v { ConsVector m x xs |-> x }
        "#,
    );

    let err = expect_err(
        &mut env,
        "const badHead : Nat = vectorHead Nat Zero (EmptyVector Nat)",
    );
    assert!(
        err.contains("type mismatch") || err.contains("kernel rejected"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn type_possible_indexed_constructor_is_still_required() {
    let mut env = mk_env();
    elab_ok(&mut env, VECTOR_DECL);

    let err = expect_err(
        &mut env,
        r#"
        fn badVectorHead (A : Type) (n : Nat) (v : Vector A n) : A =
          match v { ConsVector m x xs |-> x }
        "#,
    );
    assert!(
        err.contains("non-exhaustive match") && err.contains("EmptyVector"),
        "unexpected diagnostic: {err}"
    );
}

fn peel_app(term: &Term) -> (&Term, Vec<&Term>) {
    let mut head = term;
    let mut args = Vec::new();
    while let Term::App(f, a) = head {
        args.push(a.as_ref());
        head = f.as_ref();
    }
    args.reverse();
    (head, args)
}

fn peel_lams(mut term: &Term, count: usize) -> &Term {
    for _ in 0..count {
        match term {
            Term::Lam(_, body) => term = body,
            other => panic!("expected lambda, got {other:?}"),
        }
    }
    term
}

fn contains_absurd(term: &Term) -> bool {
    match term {
        Term::Absurd(_, _) => true,
        _ => term.children().iter().any(|child| contains_absurd(child)),
    }
}

fn motive_has_index_and_scrutinee_lambdas(motive: &Term) -> bool {
    match motive {
        Term::Ascript(term, _) => motive_has_index_and_scrutinee_lambdas(term),
        Term::Lam(_, body) => matches!(body.as_ref(), Term::Lam(_, _)),
        _ => false,
    }
}
