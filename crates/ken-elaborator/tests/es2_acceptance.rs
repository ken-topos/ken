//! ES2 acceptance tests: prelude hygiene — the `trusted_base()` shrink.
//!
//! Pins `docs/program/wp/ES2-prelude-hygiene.md`'s AC1/AC3/AC4 against the
//! **real** `prelude.rs`/`trusted_base()` (producer-grep, not a hand-fed
//! test): after ES2, `Equal`/`And`/`Bool`/`IO`/`print_line` must not remain
//! `declare_postulate`d assumed axioms, and `Map`/`Set` must be re-classed
//! `declare_primitive` (still trusted, but audited, item-2) rather than
//! removed. `isSorted`/`Perm` are NOT included here — see the WP thread for
//! the escalated Ord/DecEq-class design gap blocking their demotion.
//!
//! Spec: `spec/30-surface/37-strings-collections.md`;
//! `conformance/surface/taxonomy/minimality.md` (the derivation table).

use ken_elaborator::ElabEnv;
use ken_kernel::env::Decl;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — the demoted entries are no longer `declare_postulate`d assumed axioms
// ─────────────────────────────────────────────────────────────────────────────

/// `Equal`/`And`/`IO`/`print_line` must be `Decl::Transparent` (real,
/// re-checked definitions) — never `Decl::Opaque` (the demoted-postulate
/// bloat AC1 forbids) and never absent from `trusted_base()`'s underlying
/// `Decl` classification by accident (they must resolve to a real decl).
#[test]
fn demoted_predicates_are_transparent_not_opaque() {
    let env = mk_env();
    for name in ["Equal", "And", "IO", "print_line"] {
        let id = env.globals[name];
        match env.env.lookup(id) {
            Some(Decl::Transparent { .. }) => {}
            other => panic!(
                "AC1: '{}' must be Decl::Transparent (demoted, re-checked def), \
                 got {:?}",
                name, other
            ),
        }
    }
}

/// `Bool` must be a real inductive (`data Bool = True | False`) — matchable
/// data, not an opaque primitive type.
#[test]
fn bool_is_a_real_inductive() {
    let env = mk_env();
    let bool_id = env.globals["Bool"];
    assert!(
        env.env.inductive(bool_id).is_some(),
        "AC1/AC3: Bool must be a real inductive, not an opaque primitive"
    );
    assert!(env.globals.contains_key("True"), "True ctor registered");
    assert!(env.globals.contains_key("False"), "False ctor registered");
}

/// Discriminating: none of the demoted names appear in `trusted_base()` —
/// the real accounting, not a hand-fed assertion. `Bool` is excluded via its
/// `Decl::Inductive` kind (neither `Opaque` nor `Primitive`); the others via
/// `Decl::Transparent`.
#[test]
fn demoted_predicates_absent_from_trusted_base() {
    let env = mk_env();
    let tb = env.env.trusted_base();
    for name in ["Equal", "And", "Bool", "IO", "print_line"] {
        let id = env.globals[name];
        assert!(
            !tb.contains(&id),
            "AC1: '{}' must not remain in trusted_base() after ES2",
            name
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — Bool is matchable data
// ─────────────────────────────────────────────────────────────────────────────

/// Discriminating: a `match` on a comparison-primitive result elaborates —
/// impossible against the former opaque `Bool` primitive.
#[test]
fn match_on_comparison_result_elaborates() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl(
            "view isZero (a : Int) (b : Int) : Int = \
             match eq_int a b { True => 1 ; False => 0 }",
        )
        .expect("AC3: match on eq_int's Bool result must elaborate");
    assert!(env.env.const_type(id).is_some(), "isZero registered with a type");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — Map/Set re-class preserves trust (still audited, not regressed)
// ─────────────────────────────────────────────────────────────────────────────

/// `Map`/`Set` stay in `trusted_base()` (no trust regression) but as
/// `Decl::Primitive` (audited, item-2) — never `Decl::Opaque` (assumed
/// axiom, item-3).
#[test]
fn map_set_reclassed_primitive_stay_in_trusted_base() {
    let env = mk_env();
    let tb = env.env.trusted_base();
    for name in ["Map", "Set"] {
        let id = env.globals[name];
        assert!(
            tb.contains(&id),
            "AC4: '{}' must remain in trusted_base() (audited primitive, not removed)",
            name
        );
        match env.env.lookup(id) {
            Some(Decl::Primitive { .. }) => {}
            other => panic!(
                "AC4: '{}' must be Decl::Primitive (re-classed, not Decl::Opaque); got {:?}",
                name, other
            ),
        }
    }
}

/// Discriminating pair: a demotion that leaves the postulate in place fails
/// AC1's `Opaque` check above; a re-class that instead REMOVES `Map`/`Set`
/// from `trusted_base()` entirely (rather than correctly re-classing) would
/// fail here — the verdict must flip on the specific fate (stays-audited vs
/// removed vs stays-assumed), not just "some change happened."
#[test]
fn map_set_are_not_opaque_postulates() {
    let env = mk_env();
    for name in ["Map", "Set"] {
        let id = env.globals[name];
        assert!(
            !matches!(env.env.lookup(id), Some(Decl::Opaque { .. })),
            "AC4: '{}' must not remain Decl::Opaque (assumed axiom)",
            name
        );
    }
}
