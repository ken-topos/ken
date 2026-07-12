//! `mutual-recursion-surface` acceptance (VAL2 #3).
//!
//! `modules.rs::expand_scope` now auto-groups a maximal run of non-`pub`
//! `view`/`let` decls by call-graph SCC and routes any real (size > 1)
//! cycle through `elab::elaborate_mutual_group` — one `sct_check` over the
//! whole cycle, no member escapes the termination check.
//!
//! - AC1 (kernel untouched) is verified out-of-band (`git diff -- crates/
//!   ken-kernel/` empty) — `elaborate_mutual_group` reuses the existing
//!   `Decl::Opaque` pre-admit / `kernel_check` / `sct_check` /
//!   `upgrade_to_transparent` sequence, no new kernel API.
//! - AC2 — `isEven`/`isOdd` elaborates, SCT-accepts, and computes correct
//!   values (checked via `Refl`-discharged `Equal Bool` goals — the kernel
//!   itself must see the two sides as definitionally equal for these to
//!   elaborate); a divergent mutual group is rejected by SCT (discriminating
//!   pair).
//! - AC3 (no regression / single-recursion unaffected) is covered by the
//!   full pre-existing `ken-elaborator`/workspace suite, not here.

use ken_elaborator::ElabEnv;

fn fresh_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

const IS_EVEN_ODD: &str = "\
    fn isEven (n : Nat) : Bool = match n { Zero |-> True ; Suc m |-> isOdd m }\n\
    fn isOdd (n : Nat) : Bool = match n { Zero |-> False ; Suc m |-> isEven m }";

#[test]
fn is_even_is_odd_mutual_group_elaborates_as_one_group() {
    let mut env = fresh_env();
    let ids = env
        .elaborate_file(IS_EVEN_ODD)
        .expect("mutual isEven/isOdd must elaborate and pass SCT as one group");
    assert_eq!(ids.len(), 2, "both members of the cycle must produce a definition");
}

#[test]
fn is_even_is_odd_compute_correct_values() {
    let mut env = fresh_env();
    env.elaborate_file(IS_EVEN_ODD).expect("mutual group elaborates");
    env.elaborate_decl("const three : Nat = Suc (Suc (Suc Zero))").expect("three declares");
    env.elaborate_decl("const four : Nat = Suc three").expect("four declares");

    // `Equal Bool (op ...) True/False`, once the operand reduces to a
    // concrete `Bool` constructor, observationally COLLAPSES to `Top`
    // (K7) — the goal is no longer `Eq`-shaped, so the right closing form
    // is `Proved` (Top-introduction), not `Refl` (the same idiom documented in
    // `catalog/packages/Core/LawfulClasses.ken`). Elaborating only
    // succeeds if the operand genuinely reduced to the SAME concrete
    // constructor as the right-hand side — a real correctness check.
    env.elaborate_decl("lemma checkOdd3 : Equal Bool (isEven three) False = Proved")
        .expect("isEven 3 must reduce to False (3 is odd)");
    env.elaborate_decl("lemma checkEven4 : Equal Bool (isEven four) True = Proved")
        .expect("isEven 4 must reduce to True (4 is even)");
    env.elaborate_decl("lemma checkOdd3b : Equal Bool (isOdd three) True = Proved")
        .expect("isOdd 3 must reduce to True (3 is odd)");
}

#[test]
fn non_terminating_mutual_group_is_rejected_by_sct() {
    let mut env = fresh_env();
    let res = env.elaborate_file(
        "fn loopA (n : Nat) : Bool = loopB n\n\
         fn loopB (n : Nat) : Bool = loopA n",
    );
    assert!(
        res.is_err(),
        "a non-terminating mutual group must be rejected by SCT (discriminating pair), got {res:?}"
    );
}

#[test]
fn single_recursion_still_works_unaffected_by_the_grouping_pass() {
    // A plain, non-mutual self-recursive view (a size-1 SCC, no cycle with
    // any sibling) must keep working exactly as it did before this WP.
    let mut env = fresh_env();
    env.elaborate_decl(
        "fn natAdd (a : Nat) (b : Nat) : Nat = \
         match a { Zero |-> b ; Suc m |-> Suc (natAdd m b) }",
    )
    .expect("ordinary single self-recursion must still elaborate");
}
