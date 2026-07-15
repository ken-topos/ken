//! `sct-reconstruction-descent` acceptance tests (`sct-completeness` shape
//! (b) — lexicographic / Ackermann-style nested recursion via exact
//! reconstruction of a matched parameter).
//!
//! Fix (grounded, Architect-approved `evt_2r390qa0jbknf`): `size_rel`
//! (`crates/ken-kernel/src/sct.rs`) now recognizes an argument that is an
//! **exact reconstruction** of a matched parameter's destructured value
//! (`Suc m2` where `m2` was bound by matching that parameter against
//! `Suc m2`) and assigns it `SizeOrd::DownEq` (never `Down` — a
//! reconstruction is never strictly smaller). The predicate
//! (`is_exact_reconstruction`, `sct.rs`) requires the reconstruction to use
//! the SAME constructor, the SAME field count, and each field to be the
//! exact, positionally-raw matched-field variable — any reorder,
//! substitution, or added structure stays `Unknown`. Grounded via
//! `SCT_DEBUG` tracing against landed `sct.rs@b34d4aa`: the composed
//! self-loop for Ackermann's inner-then-outer call collapses to
//! all-`Unknown` (no strict diagonal) without this edge, and gains a strict
//! diagonal at the genuinely-decreasing parameter once it's present.
//!
//! **REORDER / WRONG-FIELD / WRONG-CTOR near-misses (AC1) are pinned as
//! direct unit tests of `is_exact_reconstruction` in `sct.rs`'s own test
//! module, not as surface `.ken` programs here** — see that module's doc
//! comment for why (a naive "swap two fields on the recursive call" surface
//! program is easy to accidentally construct as a genuinely-different,
//! validly-terminating potential-function descent, which is not a
//! discriminator at all; the predicate-level test isolates the mechanism
//! precisely instead).
//!
//! **Classic non-terminators** (`c := c`, `loop := id loop`,
//! recursion-through-a-combinator) are pinned at the kernel level in
//! `crates/ken-kernel/tests/k2c_conversion.rs`
//! (`sct_reject_bare_self_reference`, `sct_reject_combinator_laundered`) —
//! unaffected by this WP (their self-loops never touch a reconstruction
//! edge) and re-verified by the full `cargo test --workspace` run (AC4),
//! not duplicated here.

use ken_elaborator::ElabEnv;

fn fresh_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

fn nat_count(env: &ken_elaborator::ElabEnv, v: &ken_interp::eval::EvalVal) -> u64 {
    use ken_interp::eval::EvalVal;
    match v {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_count(env, &args[0])
        }
        other => panic!("expected a Nat Ctor chain, got {other:?}"),
    }
}

fn eval_nat_decl(env: &ElabEnv, name: &str) -> u64 {
    let id = *env.globals.get(name).unwrap_or_else(|| panic!("{name} should be a global"));
    let body = match env.env.lookup(id) {
        Some(ken_kernel::Decl::Transparent { body, .. }) => body.clone(),
        other => panic!("{name} should be Transparent, got {other:?}"),
    };
    let mut store = ken_interp::eval::EvalStore::new();
    for (nid, v) in &env.num_values {
        if let ken_elaborator::NumericLitVal::Int(n) = v {
            store
                .num_values
                .entry(*nid)
                .or_insert_with(|| ken_interp::eval::EvalVal::from(*n));
        }
    }
    let v = ken_interp::eval::eval(&[], &body, &env.env, &mut store);
    nat_count(env, &v)
}

/// **AC2 — the Ackermann repro accepts.** The grounding case for this WP:
/// `ack (Suc m) (Suc n) = ack m (ack (Suc m) n)` reconstructs param 0 exactly
/// (`Suc m` where `m` is bound by matching param 0 against `Suc m`) while
/// param 1 genuinely descends via the inner call.
#[test]
fn shape_b_ackermann_accepts() {
    let mut env = fresh_env();
    env.elaborate_decl(
        "fn ack (m : Nat) (n : Nat) : Nat = \
         match m { \
           Zero |-> Suc n ; \
           Suc m2 |-> match n { \
             Zero |-> ack m2 (Suc Zero) ; \
             Suc n2 |-> ack m2 (ack (Suc m2) n2) \
           } \
         }",
    )
    .expect("Ackermann must now be accepted (AC2)");
}

/// **AC2 — accepts AND evaluates to the correct value**, not just
/// type-checks. `ack(2,2) = 7` (A(2,n) = 2n+3).
#[test]
fn shape_b_ackermann_accepts_and_evaluates_correctly() {
    let mut env = fresh_env();
    env.elaborate_decl(
        "fn ack (m : Nat) (n : Nat) : Nat = \
         match m { \
           Zero |-> Suc n ; \
           Suc m2 |-> match n { \
             Zero |-> ack m2 (Suc Zero) ; \
             Suc n2 |-> ack m2 (ack (Suc m2) n2) \
           } \
         }",
    )
    .expect("must accept (AC2)");
    env.elaborate_decl("const two : Nat = Suc (Suc Zero)")
        .expect("two should declare");
    env.elaborate_decl("const result : Nat = ack two two")
        .expect("ack two two should elaborate");
    assert_eq!(
        eval_nat_decl(&env, "result"),
        7,
        "ack(2,2) must evaluate to 7 (A(2,n) = 2n+3), not just accept"
    );
}

/// **AC2 — a second, non-Ackermann-identical lexicographic-descent shape.**
/// Param 0 is preserved via an exact reconstruction (`DownEq`) on every
/// recursive step while param 1 genuinely descends (`Down`, a real matched
/// field) — the general reconstruction-`DownEq` mechanism, not an
/// Ackermann-specific special case (no *inner* nested call at all here).
#[test]
fn shape_b_second_lexicographic_shape_accepts() {
    let mut env = fresh_env();
    env.elaborate_decl(
        "fn walkDown (m : Nat) (n : Nat) : Nat = \
         match m { \
           Zero |-> n ; \
           Suc m2 |-> match n { \
             Zero |-> Zero ; \
             Suc n2 |-> walkDown (Suc m2) n2 \
           } \
         }",
    )
    .expect("outer-preserved-via-reconstruction + inner-genuinely-decreasing must accept (AC2)");
}

/// AC2 companion — evaluate `walkDown` to confirm accept-with-correct-value,
/// not accept-with-wrong-wiring. `walkDown(3,2)` walks `n` down to `Zero`
/// (`m` stays nonzero throughout) => `0`; `walkDown(0,5)` hits the `m = Zero`
/// base case immediately => `5`.
#[test]
fn shape_b_second_lexicographic_shape_evaluates_correctly() {
    let mut env = fresh_env();
    env.elaborate_decl(
        "fn walkDown (m : Nat) (n : Nat) : Nat = \
         match m { \
           Zero |-> n ; \
           Suc m2 |-> match n { \
             Zero |-> Zero ; \
             Suc n2 |-> walkDown (Suc m2) n2 \
           } \
         }",
    )
    .expect("must accept (AC2)");
    env.elaborate_decl("const three : Nat = Suc (Suc (Suc Zero))")
        .expect("three should declare");
    env.elaborate_decl("const two : Nat = Suc (Suc Zero)")
        .expect("two should declare");
    env.elaborate_decl("const five : Nat = Suc (Suc (Suc (Suc (Suc Zero))))")
        .expect("five should declare");
    env.elaborate_decl("const r1 : Nat = walkDown three two")
        .expect("walkDown three two should elaborate");
    env.elaborate_decl("const r2 : Nat = walkDown Zero five")
        .expect("walkDown Zero five should elaborate");
    assert_eq!(eval_nat_decl(&env, "r1"), 0, "walkDown(3,2) must be 0");
    assert_eq!(eval_nat_decl(&env, "r2"), 5, "walkDown(0,5) must be 5");
}

/// **AC1 — `badAck`, exact reconstruction, all-`DownEq`, no strict
/// diagonal.** Proves the fix's edge is `DownEq` and never `Down`: if it
/// were `Down`, this would be wrongly accepted (param 0 never shrinks —
/// `Suc m2` is size-preserving, not size-decreasing — and param 1 is passed
/// through completely unchanged every call).
#[test]
fn shape_b_bad_ack_still_rejected() {
    let mut env = fresh_env();
    let res = env.elaborate_decl(
        "fn badAck (m : Nat) (n : Nat) : Nat = \
         match m { Zero |-> n ; Suc m2 |-> badAck (Suc m2) n }",
    );
    assert!(
        res.is_err(),
        "badAck diverges (param 0 reconstructed size-preserving, param 1 \
         never touched) — must stay rejected; a `Down` edge for \
         reconstruction would wrongly accept this"
    );
}

/// **AC1 — `badAck2`, size-INCREASING reconstruction, must stay
/// `Unknown`/rejected.** `Suc (Suc m2)` is not the exact matched-field shape
/// (`args[0]` is `App(Ctor, Var)`, not a raw `Var`) — proves the predicate
/// requires exact depth, not just "same constructor somewhere in there".
#[test]
fn shape_b_bad_ack2_still_rejected() {
    let mut env = fresh_env();
    let res = env.elaborate_decl(
        "fn badAck2 (m : Nat) (n : Nat) : Nat = \
         match m { Zero |-> n ; Suc m2 |-> badAck2 (Suc (Suc m2)) n }",
    );
    assert!(
        res.is_err(),
        "badAck2 reconstructs param 0 as a STRICTLY LARGER value \
         (Suc (Suc m2), not the exact matched Suc m2) and diverges — must \
         stay rejected"
    );
}

/// **AC4 — monotone, no regression.** All of shape (a)'s `sct-completeness`
/// nested-split cases are still accepted under this WP's threading changes
/// (the reconstruction case is purely additive to `size_rel`, never removes
/// an existing prov-based edge).
#[test]
fn control_shape_a_nested_split_still_accepts() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Int Tree")
        .expect("Tree should declare");
    env.elaborate_decl(
        "fn countR (t : Tree) : Nat = \
         match t { \
           Leaf |-> Zero ; \
           Node (Node ll lc lr) c r |-> countR r ; \
           Node Leaf c r |-> countR r \
         }",
    )
    .expect("shape (a)'s nested-split repro must still be accepted (AC4 monotonicity)");
}

/// **AC4 companion — the near-miss that must STILL be rejected** (shape (a)'s
/// own discriminating negative, unaffected by this WP).
#[test]
fn control_shape_a_near_miss_still_rejected() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Int Tree")
        .expect("Tree should declare");
    let res = env.elaborate_decl(
        "fn bad (t : Tree) : Nat = \
         match t { \
           Leaf |-> Zero ; \
           Node (Node ll lc lr) c r |-> bad t ; \
           Node Leaf c r |-> bad r \
         }",
    );
    assert!(res.is_err(), "shape (a)'s near-miss must stay rejected");
}
