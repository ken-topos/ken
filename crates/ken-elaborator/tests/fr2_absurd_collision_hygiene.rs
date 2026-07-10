//! FR-2 (`docs/program/wp/ds-1-findings-remediation.md`) acceptance —
//! `absurd`/`Refl`/`Axiom` silent-shadow hygiene, Architect-corrected design.
//!
//! `Refl`/`Axiom`/`absurd` are a bare/arity-1 total intercept: declaring a
//! real global under one of these names used to elaborate successfully and
//! then be permanently unreachable via ordinary call syntax (DS-1 finding);
//! this is now a resolve-time hard error, fail-closed.
//!
//! `J`/`Eq` intercept ONLY a 3-argument application, by design, so a
//! lower-arity type-former/class of the same name (the landed `class Eq a`)
//! coexists correctly — these two are deliberately NOT in the hard-error set
//! (the Architect's corrected FR-2 ruling; the original pin's "every
//! syntactic Eq" framing was wrong and would have broken `class Eq a`).

use ken_elaborator::{ElabEnv, ElabError};

fn expect_collision_error(env: &mut ElabEnv, src: &str, colliding_name: &str) {
    let err = env
        .elaborate_decl(src)
        .expect_err(&format!("'{colliding_name}' must be rejected as a reserved-sugar collision"));
    match err {
        ElabError::ParseError { msg, .. } => {
            assert!(
                msg.contains(colliding_name) && msg.contains("reserved surface sugar"),
                "error must name the collision, not a generic message: {msg}"
            );
        }
        other => panic!("expected a ParseError (a real hard error, not a warning), got {other:?}"),
    }
}

// A real hard error, not a warning: elaborate_decl must return Err, and the
// error must be the specific ParseError collision variant naming the
// colliding identifier — not merely "elaboration produced a diagnostic
// while still succeeding."
#[test]
fn declaring_absurd_is_a_hard_error_not_a_silent_shadow() {
    let mut env = ElabEnv::new().expect("base env");
    expect_collision_error(
        &mut env,
        "fn absurd (C : Type) (e : Bool) : C = match e { True => e ; False => e }",
        "absurd",
    );
}

#[test]
fn declaring_refl_or_axiom_are_hard_errors() {
    for (src, name) in [
        ("const Refl : Bool = True", "Refl"),
        ("const Axiom : Bool = True", "Axiom"),
    ] {
        let mut env = ElabEnv::new().expect("base env");
        expect_collision_error(&mut env, src, name);
    }
}

// Architect ruling: J/Eq intercept ONLY a 3-argument application, by design,
// so a lower-arity declaration of the same name is NOT a collision and must
// elaborate normally — a declaration-time reject here would be the
// categorically wrong tool and would break real, landed code (see below).
#[test]
fn declaring_lower_arity_j_or_eq_elaborates_fine_not_a_collision() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("fn J (x : Bool) : Bool = x")
        .expect("a 1-arg 'J' is not the arity-3 J sugar; must elaborate");
    env.elaborate_decl("fn Eq (x : Bool) : Bool = x")
        .expect("a 1-arg 'Eq' is not the arity-3 Eq sugar; must elaborate");
}

// THE regression-guard test: the actual landed `class Eq a` shape (arity-1,
// `catalog/packages/Core/LawfulClasses.ken`) must elaborate cleanly with the
// collision guard active — this is the exact case the original (wrong) pin
// would have broken, and the corrected ruling exists specifically to keep
// working. Mirrors the real landed shape, not just any arity-1 stand-in.
#[test]
fn landed_class_eq_shape_elaborates_cleanly_with_the_guard_active() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl(
        "class Eq a { eq : a -> a -> Bool ; \
                      refl : (x : a) -> IsTrue (eq x x) ; \
                      sym : (x : a) -> (y : a) -> IsTrue (eq x y) -> IsTrue (eq y x) ; \
                      trans : (x : a) -> (y : a) -> (z : a) -> IsTrue (eq x y) -> IsTrue (eq y z) -> IsTrue (eq x z) }",
    )
    .expect("the landed class Eq a shape must elaborate cleanly with the collision guard active");
}

// Non-regression: the arity-3 Eq/J sugar itself must still work exactly as
// before — FR-2's correction only narrows the DECLARATION-time guard, it
// never touches the sugar's own interception logic.
#[test]
fn arity_three_eq_and_j_sugar_still_intercept_normally() {
    let mut env = ElabEnv::new().expect("base env");
    // `Eq A a b` at type position (the kernel equality-type sugar).
    env.elaborate_decl("fn eqRefl (a : Type) (x : a) : Eq a x x = Refl")
        .expect("arity-3 Eq sugar must still elaborate at type position");
}

// The sweep must cover DATA CONSTRUCTOR names too, not just the decl head —
// a `data ... = Refl | ...` ctor resolves as an RCon exactly like a bare
// declaration name. (`Eq`/`J` ctors are unaffected by the guard now, per the
// ruling — covered by the coexistence test below.)
#[test]
fn legacy_data_ctor_named_refl_is_a_hard_error() {
    let mut env = ElabEnv::new().expect("base env");
    expect_collision_error(&mut env, "data Foo = Refl | Bar", "Refl");
}

#[test]
fn explicit_data_ctor_named_axiom_is_a_hard_error() {
    let mut env = ElabEnv::new().expect("base env");
    expect_collision_error(&mut env, "data Foo : Type where { Axiom : Foo }", "Axiom");
}

// A ctor literally named Eq/J is NOT a collision (same coexistence-by-design
// rule as the decl-head case) — confirms the ctor sweep reads the SAME
// narrowed RESERVED_SUGAR set, not a stale independent list.
#[test]
fn data_ctor_named_eq_or_j_elaborates_fine_not_a_collision() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("data Foo = Eq | J")
        .expect("Eq/J ctors are arity-0 here, not the arity-3 sugar; must elaborate");
}

// A NON-colliding constructor alongside a colliding one still only reports
// the real collision — confirms the sweep checks each ctor's own name, not
// the whole decl as one unit.
#[test]
fn non_colliding_ctor_alongside_a_colliding_one_still_reports_only_the_real_collision() {
    let mut env = ElabEnv::new().expect("base env");
    let err = env
        .elaborate_decl("data Foo = Bar | Refl | Baz")
        .expect_err("Refl ctor must be rejected");
    match err {
        ElabError::ParseError { msg, .. } => assert!(msg.contains("Refl")),
        other => panic!("expected ParseError, got {other:?}"),
    }
}

// Cast/Ascript are Term-level kernel constructors only, never resolver-
// intercepted as bare surface identifiers -- excluded from the reserved set
// by design, not by omission. A user global under either name must elaborate
// fine, proving the exclusion is a real, checked design decision.
#[test]
fn cast_and_ascript_are_not_reserved_and_elaborate_fine() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("data Cast = MkCast")
        .expect("'Cast' is not resolver-intercepted sugar; must elaborate");
    env.elaborate_decl("data Ascript = MkAscript")
        .expect("'Ascript' is not resolver-intercepted sugar; must elaborate");
}

// Non-regression: the total-intercept sugar itself must still work when
// there is no colliding user declaration -- FR-2 must not have broken the
// sugar it guards.
#[test]
fn reserved_sugar_still_works_when_undeclared() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("fn reflBoolId (x : Bool) : Equal Bool x x = Refl")
        .expect("Refl sugar must still elaborate");
    // A real, syntactically valid absurd use (mirroring DS-1's own
    // `absurdEmpty`): the sugar elaborates when the goal is Ω-classified.
    env.elaborate_decl("fn absurdFromBottom (C : Type) (h : Bottom) : C = absurd h")
        .expect("absurd sugar must still elaborate over an Ω-classified goal");
}
