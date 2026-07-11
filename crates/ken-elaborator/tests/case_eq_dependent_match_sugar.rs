//! Acceptance tests for the `match e eqn: h { ... }` dependent-match modifier.

use ken_elaborator::{error::ElabError, ElabEnv};
use ken_kernel::{GlobalId, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base environment")
}

fn mentions_elim(t: &Term) -> bool {
    match t {
        Term::Elim { .. } => true,
        Term::App(f, a) => mentions_elim(f) || mentions_elim(a),
        Term::Pi(a, b) | Term::Lam(a, b) | Term::Sigma(a, b) => {
            mentions_elim(a) || mentions_elim(b)
        }
        Term::Ascript(a, b) => mentions_elim(a) || mentions_elim(b),
        Term::Let { ty, val, body } => {
            mentions_elim(ty) || mentions_elim(val) || mentions_elim(body)
        }
        _ => false,
    }
}

#[test]
fn stuck_bool_scrutinee_binds_a_usable_equation_and_kernel_checks() {
    let mut env = mk_env();
    let ids = env
        .elaborate_file(
            "fn da_eq (x : Bool) (y : Bool) : Bool = x\n\
             fn stuck_eq (x : Bool) (y : Bool) : Equal Bool (da_eq x y) (da_eq x y) = match (da_eq x y) eqn: h {
               True => J (\\b' _. Equal Bool b' b') Refl h ;
               False => J (\\b' _. Equal Bool b' b') Refl h }",
        )
        .expect("the equation binder must transport a stuck Bool application");
    let (_, body) = env
        .env
        .transparent_body(ids[1])
        .expect("stuck_eq is transparent");
    assert!(
        mentions_elim(&body),
        "modifier must elaborate through an eliminator: {body:?}"
    );
}

#[test]
fn three_constructor_enum_is_not_bool_hardcoded() {
    let mut env = mk_env();
    env.elaborate_file(
        "data OrdResult = Lt | Eq | Gt\n\
         fn ord_self (o : OrdResult) : Equal OrdResult o o = match o eqn: h {
           Lt => J (\\b' _. Equal OrdResult b' b') Refl h ;
           Eq => J (\\b' _. Equal OrdResult b' b') Refl h ;
           Gt => J (\\b' _. Equal OrdResult b' b') Refl h }",
    )
    .expect("the modifier must support all three OrdResult constructors");
}

#[test]
fn reverted_hypothesis_shape_elaborates_with_a_real_byte_reduction() {
    let mut env = mk_env();
    // This is the `deceq Cons/Cons` proof shape in miniature: the comparison
    // appears in the domain of a returned function, so the modifier transports
    // it rather than requiring an author-written dispatch helper and J motive.
    const SUGAR: &str = "fn cons_eq (x : Bool) (y : Bool) : Bool = x\n\
        fn IsTrueLike (b : Bool) : Prop = Equal Bool b True\n\
        fn cons_sound (x : Bool) (y : Bool) : IsTrueLike (cons_eq x y) -> Equal Bool x x = \
        match (cons_eq x y) eqn: h {
          True => \\p. Refl ;
          False => \\p. absurd p }";
    // The previous required idiom contains a dichotomy, a named dispatch
    // helper, and an explicit J motive. It is deliberately kept as source
    // text, so this is a byte-count assertion over author-facing programs.
    const HAND_IDIOM: &str = "fn cons_sound (x : Bool) (y : Bool) : IsTrueLike (cons_eq x y) -> Equal Bool x x = \
        bool_dichotomy (cons_eq x y) |> dispatch_cons_eq x y where \
        fn dispatch_cons_eq (x : Bool) (y : Bool) (d : Or (Equal Bool (cons_eq x y) True) (Equal Bool (cons_eq x y) False)) \
          : IsTrueLike (cons_eq x y) -> Equal Bool x x = \
          match d { Inl q => J (\\b' _. IsTrueLike b' -> Equal Bool x x) (\\p. Refl) q ; \
                    Inr q => J (\\b' _. IsTrueLike b' -> Equal Bool x x) (\\p. absurd p) q }";
    env.elaborate_file(SUGAR)
        .expect("reverted hypothesis occurrence must be generalized and transported");
    assert!(
        SUGAR.len() < HAND_IDIOM.len(),
        "the modifier must be byte-smaller than the real hand-written idiom"
    );
}

#[test]
fn non_nullary_constructors_are_rejected_at_the_modifier_boundary() {
    let mut env = mk_env();
    let result = env.elaborate_file(
        "data Parcel = Wrap Bool\n\
         fn no_fields (b : Parcel) : Equal Parcel b b = match b eqn: h { Wrap x => Refl }",
    );
    assert!(
        matches!(result, Err(ElabError::TypeMismatch { ref reason, .. })
            if reason.contains("finite enums with nullary constructors")),
        "field-bearing constructors must be rejected by this bounded modifier: {result:?}"
    );
}

#[test]
fn mismatched_equation_branch_is_kernel_rejected() {
    let mut env = mk_env();
    let result = env.elaborate_file(
        "fn wrong (b : Bool) : Equal Bool b b = match b eqn: h { True => h ; False => h }",
    );
    assert!(
        matches!(result, Err(ElabError::KernelRejected { .. })),
        "the whole assembled transport must fail closed through KernelRejected: {result:?}"
    );
}

#[test]
fn modifier_has_zero_trusted_base_delta() {
    let base: std::collections::HashSet<GlobalId> = ElabEnv::new()
        .unwrap()
        .env
        .trusted_base()
        .into_iter()
        .collect();
    let mut env = mk_env();
    env.elaborate_file(
        "fn eqn_tb (b : Bool) : Equal Bool b b = match b eqn: h {
           True => J (\\b' _. Equal Bool b' b') Refl h ;
           False => J (\\b' _. Equal Bool b' b') Refl h }",
    )
    .expect("bounded dependent match elaborates");
    let after: std::collections::HashSet<GlobalId> = env.env.trusted_base().into_iter().collect();
    assert_eq!(base, after, "the modifier may not add trusted declarations");
}
