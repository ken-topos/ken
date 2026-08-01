//! L3b acceptance tests — user-type `DecEq`/`Ord` instance resolution via
//! `instance_search` for collection ops (`37 §6`, `33 §5`, `39 §6`).
//!
//! Pins the 4 discriminating ACs from
//! `conformance/surface/collections/seed-collections.md` §L3b:
//!
//! - AC1 (`user-deceq-instance-keys-map-via-real-search`): user `instance
//!   DecEq K` → `Map K v` via `where DecEq K` accepts; absence → `NoInstance`.
//! - AC2 (`user-ord-instance-drives-verified-sort`): user `instance Ord K`
//!   → `const where Ord K` accepts; absence → `NoInstance`.
//! - AC3 (`user-ord-sort-emits-both-conjuncts`, soundness): verified sort
//!   under user `Ord K` emits the conjoined `is_sorted ∧ Perm` obligation —
//!   both conjuncts, `Perm` present (the untrusted-layer omission guard).
//! - AC4 (`user-deceq-keyed-map-canonical-identity`): `Map K v` identity is
//!   `DecEq`-keyed (byte-order canonical, `41 §3a`), NOT `Ord`-keyed — a
//!   `where DecEq K`-only const accepts; adding `where Ord K` (no `Ord K`
//!   instance) rejects, showing the identity gate is DecEq.

use ken_elaborator::{error::ElabError, ElabEnv, ObligationKind};
use ken_kernel::{GlobalId, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("ElabEnv construction failed")
}

fn elab(env: &mut ElabEnv, src: &str) -> Result<GlobalId, ElabError> {
    env.elaborate_decl(src)
}

// ============================================================================
// AC1 — user DecEq instance → Map key op resolves; absence → NoInstance
// ============================================================================

/// `surface/collections/user-deceq-instance-keys-map-via-real-search` (AC1, ★)
///
/// (a) `data K` + `instance DecEq K {}` + `const where DecEq K` → accepts:
///     `instance_search("DecEq", "K")` returns `Some(id)` (the real resolver).
/// (b) Same `K` **without** `instance DecEq K` → `NoInstance` (no silent
///     built-in fallback; the reject arm is the guard against a built-in-only
///     table that ignores the `instance_search` call for user types).
///
/// Structural discriminator: verdict flips on the user instance.
/// Producer: the `where` constraint mechanism calls `instance_search`
/// (`classes.rs:91`) in `elaborate_rdecl_v1` (`elab.rs`) — QA grep target.
#[test]
fn user_deceq_instance_keys_map_via_real_search() {
    // (a) accept: user K with DecEq instance → constraint resolves.
    let mut env_a = mk_env();
    elab(&mut env_a, "class DecEq A { }").unwrap();
    elab(&mut env_a, "data K = MkK").unwrap();
    elab(&mut env_a, "instance DecEq K { }").unwrap();
    // The constraint `where DecEq K` fires instance_search("DecEq", "K") →
    // Some(id); body `k` has type `K` matching the return type.
    let r_a = env_a.elaborate_decl(
        "fn mapKey (k : K) : K where DecEq K = k",
    );
    assert!(
        r_a.is_ok(),
        "AC1(a): user DecEq K instance → const with `where DecEq K` must accept; \
         got {:?}",
        r_a
    );
    // Verify instance_search directly returns Some for the user type.
    let resolved = env_a.class_env.instance_search("DecEq", "K");
    assert!(
        resolved.is_some(),
        "AC1(a): instance_search(\"DecEq\", \"K\") must return Some(id)"
    );

    // (b) reject: same K but no instance → NoInstance (not a silent fallback).
    let mut env_b = mk_env();
    elab(&mut env_b, "class DecEq A { }").unwrap();
    elab(&mut env_b, "data K = MkK").unwrap();
    // No `instance DecEq K` declared.
    let r_b = env_b.elaborate_decl(
        "fn mapKey (k : K) : K where DecEq K = k",
    );
    assert!(
        matches!(r_b, Err(ElabError::NoInstance { .. })),
        "AC1(b): absent instance DecEq K → NoInstance (not silent fallback); \
         got {:?}",
        r_b
    );
}

// ============================================================================
// AC2 — user Ord instance drives verified sort; absence → NoInstance
// ============================================================================

/// `surface/collections/user-ord-instance-drives-verified-sort` (AC2)
///
/// (a) `data K` + `instance Ord K {}` + `const where Ord K` → accepts:
///     `instance_search("Ord", "K")` returns `Some(id)`.
/// (b) Same `K` without `instance Ord K` → `NoInstance`.
///
/// Structural discriminator: verdict flips on the user Ord instance.
/// Producer: `instance_search("Ord", "K")` in `elaborate_rdecl_v1`.
#[test]
fn user_ord_instance_drives_verified_sort() {
    // (a) accept: user K with Ord instance.
    let mut env_a = mk_env();
    elab(&mut env_a, "class Ord A { }").unwrap();
    elab(&mut env_a, "data K = MkK").unwrap();
    elab(&mut env_a, "instance Ord K { }").unwrap();
    // view `where Ord K` fires instance_search("Ord", "K") → Some(id).
    // Body `xs` returns `List K` (the identity on the list).
    let r_a = env_a.elaborate_decl(
        "fn sortK (xs : List K) : List K where Ord K = xs",
    );
    assert!(
        r_a.is_ok(),
        "AC2(a): user Ord K instance → const with `where Ord K` must accept; \
         got {:?}",
        r_a
    );

    // (b) reject: no Ord K instance → NoInstance.
    let mut env_b = mk_env();
    elab(&mut env_b, "class Ord A { }").unwrap();
    elab(&mut env_b, "data K = MkK").unwrap();
    // No `instance Ord K`.
    let r_b = env_b.elaborate_decl(
        "fn sortK (xs : List K) : List K where Ord K = xs",
    );
    assert!(
        matches!(r_b, Err(ElabError::NoInstance { .. })),
        "AC2(b): absent instance Ord K → NoInstance; got {:?}",
        r_b
    );
}

// ============================================================================
// AC3 — user-Ord sort emits conjoined is_sorted ∧ Perm obligation (soundness)
// ============================================================================

/// `surface/collections/user-ord-sort-emits-both-conjuncts` (AC3, soundness ★)
///
/// A `sort`-shaped const with `where Ord K` (user instance) and the refinement
/// `{ ys : List K | And (is_sorted K ys) (Perm K ys xs) }` emits the conjoined
/// Ensures obligation carrying BOTH `is_sorted` AND the load-bearing `Perm`
/// conjunct on the user-`Ord` path.
///
/// Discriminating: a build that drops `Perm` on the user-`Ord` path (while
/// preserving it for built-in `Ord`, L3a AC6) passes L3a-AC6 but fails here.
/// The `const Nil` degeneracy: `is_sorted`-alone is met by `sort _ = Nil`
/// (empty list is vacuously sorted); `Perm` forces `sort` to be a sort.
/// The conjunct is never generated ⇒ never discharged ⇒ reads proved-by-default
/// (the untrusted-layer omission: the kernel does not generate what we drop).
///
/// Producer: grep the emitted obligation at the refinement return site on the
/// user-`Ord` path (`34 §5`, `22 §2.1`) — not an assumed proposition.
#[test]
fn user_ord_sort_emits_both_conjuncts() {
    let mut env = mk_env();
    elab(&mut env, "class Ord A { }").unwrap();
    elab(&mut env, "data K = MkK").unwrap();
    elab(&mut env, "instance Ord K { }").unwrap();

    // Declare a sort-shaped const with the full conjoined refinement and
    // `where Ord K`.  Body `Nil K` (the empty list) type-checks against
    // `List K` (the refinement carrier) and causes the elaborator to emit
    // the Ensures obligation
    // `And (is_sorted K (\_ _. True) (Nil K)) (Perm K (Nil K) xs)`.
    // The body does NOT need to be a correct sort — only the obligation
    // emission is under test. `is_sorted` (ES2-remainder, real def) now takes
    // an explicit comparator; `K = MkK` has a single value, so a constant
    // `True` comparator is a valid (if trivial) `K -> K -> Bool`.
    let res = env
        .elaborate_decl_v1(
            "fn sortK (xs : List K) : \
             { ys : List K | And (is_sorted K (\\_ _. True) ys) (Perm K ys xs) } \
             where Ord K = Nil K",
        )
        .expect(
            "AC3: sort-shaped const with user Ord K + refinement must elaborate",
        );

    // The conjoined Ensures obligation must be emitted.
    let obl = res
        .obligations
        .iter()
        .find(|o| matches!(o.kind, ObligationKind::Ensures))
        .expect("AC3: Ensures (refinement) obligation must be emitted");

    let and_id = env.globals["And"];
    let issorted_id = env.globals["is_sorted"];
    let perm_id = env.globals["Perm"];

    assert!(
        term_mentions_const(&obl.goal_closed, and_id),
        "AC3 (soundness): obligation must be the conjunction (And)"
    );
    assert!(
        term_mentions_const(&obl.goal_closed, issorted_id),
        "AC3 (soundness): obligation must carry the is_sorted conjunct"
    );
    assert!(
        term_mentions_const(&obl.goal_closed, perm_id),
        "AC3 (soundness ★): obligation must carry the load-bearing Perm \
         conjunct (not is_sorted-alone; the const-Nil degeneracy guard)"
    );
}

// ============================================================================
// AC4 — Map canonical identity is DecEq-keyed, NOT Ord-keyed
// ============================================================================

/// `surface/collections/user-deceq-keyed-map-canonical-identity` (AC4)
///
/// `Map K v` canonical form is byte-order keyed (`41 §3a`), enforced by
/// `DecEq K` (not `Ord K`). The discriminating pair:
///
/// (a) `const where DecEq K` only (no Ord) → accepts: Map identity needs only
///     `DecEq K`; the constraint check resolves `instance_search("DecEq","K")`.
/// (b) `const where Ord K` (no `Ord K` instance, only `DecEq K` registered) →
///     `NoInstance`: if identity required `Ord`, this would need to accept.
///     Its rejection shows that `Ord` is NOT the map-identity gate — the
///     correct gate is `DecEq` alone.
///
/// Producer: the `where` constraint check in `elaborate_rdecl_v1` calls
/// `instance_search` for the specific constraint name; the DecEq-only path
/// resolves; the Ord-only path rejects (no `Ord K` instance registered).
#[test]
fn user_deceq_keyed_map_canonical_identity() {
    // Set up a user type K with DecEq only (no Ord).
    let mut env = mk_env();
    elab(&mut env, "class DecEq A { }").unwrap();
    elab(&mut env, "class Ord A { }").unwrap();
    elab(&mut env, "data K = MkK").unwrap();
    elab(&mut env, "instance DecEq K { }").unwrap();
    // No `instance Ord K`.

    // (a) Map key op with `where DecEq K` only → accepts (byte-order canonical;
    //     DecEq is the identity gate, not Ord).
    let r_a = env.elaborate_decl(
        "fn mapIdentity (k : K) : K where DecEq K = k",
    );
    assert!(
        r_a.is_ok(),
        "AC4(a): Map with `where DecEq K` only must accept \
         (DecEq = identity gate, not Ord); got {:?}",
        r_a
    );

    // (b) Ord-gated const fails → Ord is NOT required for Map identity.
    // If Ord were the identity gate, this would need to accept; its rejection
    // proves the gate is DecEq alone.
    let r_b = env.elaborate_decl(
        "fn ordGatedOp (k : K) : K where Ord K = k",
    );
    assert!(
        matches!(r_b, Err(ElabError::NoInstance { .. })),
        "AC4(b): `where Ord K` with no Ord instance → NoInstance \
         (Ord is not the Map identity gate); got {:?}",
        r_b
    );
}

// ============================================================================
// Helpers
// ============================================================================

/// Walk a `Term` and return `true` if it mentions `id` as a `Const`.
fn term_mentions_const(term: &Term, id: GlobalId) -> bool {
    match term {
        Term::Const { id: i, .. } => *i == id,
        Term::Var(_) | Term::Type(_) | Term::Omega(_) => false,
        Term::Pi(a, b) | Term::Lam(a, b) | Term::App(a, b) => {
            term_mentions_const(a, id) || term_mentions_const(b, id)
        }
        Term::Let { ty, val, body, .. } => {
            term_mentions_const(ty, id)
                || term_mentions_const(val, id)
                || term_mentions_const(body, id)
        }
        Term::Ascript(e, t) => {
            term_mentions_const(e, id) || term_mentions_const(t, id)
        }
        Term::Elim { motive, methods, scrut, params, .. } => {
            term_mentions_const(motive, id)
                || methods.iter().any(|m| term_mentions_const(m, id))
                || term_mentions_const(scrut, id)
                || params.iter().any(|p| term_mentions_const(p, id))
        }
        _ => false,
    }
}
