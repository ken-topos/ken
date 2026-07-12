//! `surface-transport` (Map Gap A) acceptance tests — the `J` surface
//! former (`spec/30-surface/34-data-match.md §3.4`) + the `Eq`-at-any-level
//! type-position spelling it needs to state its own combinators
//! (`spec/50-stdlib/53-transport.md`), against the REAL
//! `catalog/packages/Core/Transport.ken` source (producer-grep: drives the
//! actual package file via `include_str!`).
//!
//! AC1 (soundness, load-bearing): the elaboration emits a real `Term::J`
//! the kernel type-checks, plus a discriminating negative (an ill-typed
//! transport is kernel-rejected). AC2 (trust surface): zero
//! `trusted_base()` delta. AC3 (capability): a genuine stuck-match
//! transport over an ABSTRACT hypothesis (the Map shape) discharges,
//! including via the package's own `sym`/`cast` combinators. AC4 (no
//! regression) is `cargo test --workspace`, run separately.

use ken_elaborator::ElabEnv;
use ken_kernel::{GlobalId, Term};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

fn mk_env_with_package() -> ElabEnv {
    let mut env = mk_env();
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("catalog/packages/Core/Transport.ken must elaborate");
    env
}

/// Does `t` contain a `Term::J` node anywhere in its structure? (AC1: grep
/// the actual elaborated term, not just "it type-checked".)
fn mentions_j(t: &Term) -> bool {
    match t {
        Term::J(..) => true,
        Term::App(f, a) => mentions_j(f) || mentions_j(a),
        Term::Pi(a, b) | Term::Lam(a, b) | Term::Sigma(a, b) => mentions_j(a) || mentions_j(b),
        Term::Ascript(a, b) => mentions_j(a) || mentions_j(b),
        Term::Let { ty, val, body } => mentions_j(ty) || mentions_j(val) || mentions_j(body),
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// AC1 — soundness: real `Term::J` emission, kernel-checked, + discriminating
// negative.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn j_elaborates_to_a_real_term_j_node() {
    let mut env = mk_env();
    let ids = env
        .elaborate_file(
            "lemma refl_test (ty : Type) (a : ty) (q : Equal ty a a) : Equal ty a a = \
               J (\\b' _. Equal ty a b') Refl q",
        )
        .expect("J on refl should elaborate and kernel-check");
    let id = ids[0];
    let (_, body) = env.env.transparent_body(id).expect("lemma must be transparent");
    assert!(
        mentions_j(&body),
        "elaborated body must contain a real Term::J node, got {:?}",
        body
    );
}

#[test]
fn ill_typed_transport_wrong_equation_is_kernel_rejected() {
    let mut env = mk_env();
    // `base` proves `Equal ty bogus bogus` (an unrelated free variable) but
    // the motive demands `base : Equal ty a a` — not convertible.
    let res = env.elaborate_file(
        "lemma bad_transport (ty : Type) (a : ty) (b : ty) (q : Equal ty a b) \
           (bogus : ty) (r : Equal ty bogus bogus) : Equal ty a b = \
           J (\\b' _. Equal ty a b') r q",
    );
    assert!(
        res.is_err(),
        "ill-typed base (proving the wrong equation) must be kernel-rejected, got {:?}",
        res
    );
}

#[test]
fn ill_typed_transport_wrong_witness_type_is_kernel_rejected() {
    let mut env = mk_env();
    // `eq` isn't even an `Eq`-typed value (it's a plain `ty`), so `J` must
    // reject at the point it tries to recover `A`/`a`/`b` from its type.
    let res = env.elaborate_file(
        "fn not_an_eq (ty : Type) (a : ty) (not_a_proof : ty) : ty = \
           J (\\b' _. ty) a not_a_proof",
    );
    assert!(
        res.is_err(),
        "J's `eq` argument must be `Eq`-typed; a plain value must be rejected, got {:?}",
        res
    );
}

// ─────────────────────────────────────────────────────────────────────────
// AC2 — trust surface: zero `trusted_base()` delta.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn transport_package_adds_zero_trusted_base_delta() {
    let base_tb: std::collections::HashSet<GlobalId> =
        ElabEnv::new().unwrap().env.trusted_base().into_iter().collect();
    let env = mk_env_with_package();
    let with_pkg_tb: std::collections::HashSet<GlobalId> =
        env.env.trusted_base().into_iter().collect();
    assert_eq!(
        base_tb, with_pkg_tb,
        "loading catalog/packages/Core/Transport.ken must add ZERO trusted_base() entries \
         (every combinator reduces through the already-trusted J/Cast)"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// AC3 — capability: a real transport-blocked proof discharges (the Map
// shape: a `match` stuck on an ABSTRACT scrutinee, fired via a propositional
// order hypothesis).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn stuck_match_over_abstract_key_transports_via_hand_written_motive() {
    let mut env = mk_env();
    // `stuck_of k = match k {...}` stands in for `if leq k k' then … else
    // …` (the Map shape) — a `match` on an ABSTRACT `k` can never fire on
    // its own; `q : Equal Bool True k` is the (flipped) order hypothesis
    // (mirrors the real Map proof's own `sym q` first step). The base case
    // is `Proved` (Top-introduction), not `Refl`: `Equal Bool (stuck_of True)
    // True` observationally COLLAPSES to `Top` once the operand reduces
    // (K7), the same `Refl`/`Proved`/`absurd` idiom documented in
    // `catalog/packages/Core/LawfulClasses.ken`.
    let ids = env
        .elaborate_file(
            "fn stuck_of (k : Bool) : Bool = match k { True => True ; False => False }\n\
             lemma stuck_transport (k : Bool) (q : Equal Bool True k) \
               : Equal Bool (stuck_of k) True = \
               J (\\b' _. Equal Bool (stuck_of b') True) Proved q",
        )
        .expect("J must transport a stuck match over an abstract Bool hypothesis");
    let stuck_transport_id = ids[1];
    let (_, body) = env
        .env
        .transparent_body(stuck_transport_id)
        .expect("lemma must be transparent");
    assert!(mentions_j(&body), "the proof must be a real Term::J, got {:?}", body);
}

#[test]
fn stuck_match_transports_via_package_sym() {
    let mut env = mk_env_with_package();
    // Same shape as above, but stated the way a real proof would use it:
    // the hypothesis is in its NATURAL orientation (`q : Equal Bool k
    // True`), flipped via the package's own `sym` (note: `subst` is
    // `Type`-valued-family-only per `53-transport.md §3` — the Ω-valued
    // Map-law goal shape here must still go through `J` directly, exactly
    // as the spec states; `sym`'s hypothesis-flip is the part this test
    // exercises from the package rather than hand-inlining it).
    let ids = env
        .elaborate_file(
            "fn stuck_of2 (k : Bool) : Bool = match k { True => True ; False => False }\n\
             lemma stuck_transport2 (k : Bool) (q : Equal Bool k True) \
               : Equal Bool (stuck_of2 k) True = \
               J (\\b' _. Equal Bool (stuck_of2 b') True) Proved (sym Bool k True q)",
        )
        .expect("J + package sym must transport a stuck match too");
    let id = ids[1];
    let (_, body) = env.env.transparent_body(id).expect("lemma must be transparent");
    assert!(mentions_j(&body), "must bottom out in a real Term::J, got {:?}", body);
}
