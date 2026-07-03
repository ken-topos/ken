//! `sct-completeness` investigation repros (VAL2 #12 + Ackermann).
//!
//! Investigation-only per `docs/program/wp/sct-completeness.md` — these pin
//! the CURRENT (pre-fix) false-rejection behavior against the landed
//! `crates/ken-kernel/src/sct.rs`. No kernel edits accompany this file; see
//! the kickoff-thread proposal for the grounded root-cause + candidate fix
//! per shape.

use ken_elaborator::ElabEnv;

fn fresh_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

/// Control — flat (non-nested) match, ONE sibling field recurses. Per the
/// L-match-ih-fix (#5) acceptance suite this shape already elaborates;
/// confirms the baseline "field siblings get `Down` provenance" path is
/// intact before probing the nested case.
#[test]
fn control_flat_single_sibling_recurses() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Nat Tree")
        .expect("Tree should declare");
    env.elaborate_decl(
        "view countR (t : Tree) : Nat = \
         match t { Leaf => Zero ; Node l c r => countR r }",
    )
    .expect("flat single-sibling recursion must be accepted (control)");
}

/// Control — flat (non-nested) match, BOTH sibling fields recurse (the
/// literal shape of `tree-traversal`'s intended `inorder`, modulo `List`
/// `Char` machinery). Also already accepted.
#[test]
fn control_flat_both_siblings_recurse() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Nat Tree")
        .expect("Tree should declare");
    env.elaborate_decl(
        "view natAdd (a : Nat) (b : Nat) : Nat = \
         match a { Zero => b ; Suc m => Suc (natAdd m b) }",
    )
    .expect("natAdd should declare");
    env.elaborate_decl(
        "view countBoth (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node l c r => Suc (natAdd (countBoth l) (countBoth r)) \
         }",
    )
    .expect("flat both-siblings recursion must be accepted (control)");
}

/// **(a) VAL2 #12 — reproduced.** A nested sub-pattern split on the FIRST
/// field of `Node`, with a recursive call on the FLAT SIBLING field `r`
/// (never descending into the nested part). This is otherwise a completely
/// ordinary structural recursion (`r` is a genuine, strictly-smaller
/// sub-part of `t`) and must terminate — but SCT currently rejects it.
///
/// Root cause (grounded against `sct.rs @ 39b253d`): `enter_method`
/// (`sct.rs:139-164`) assumes a compiled method body has exactly
/// `n_fields + n_ihs` leading `Term::Lam` binders before its "recurse" logic
/// starts (`collect_calls`); with a nested split (produced by the #5
/// match-compiler fix, `07d167f`), the OUTER `Node` method only presents
/// ONE leading `Lam` (for `l`) before hitting a nested `Term::Elim` — the
/// remaining fields/IHs (`c`, `r`, `IH_l`, `IH_r`) are bound LATER, inside
/// EACH of the nested split's own branches. `enter_method`'s peel loop hits
/// the `_ => break` arm (`sct.rs:160`) after only 1 of 5 expected binders,
/// so `collect_calls` falls through to its generic `Term::Lam` arm
/// (`sct.rs:210-214`) for the deferred fields — which pushes `None`
/// provenance (not the correct `field_prov`) for `r`. `size_rel` therefore
/// reports `Unknown` instead of `Down` for the `countR r` self-call, and the
/// resulting self-loop matrix has no strict diagonal entry ⇒ rejected.
#[test]
fn shape_a_val2_12_nested_split_flat_sibling_recursion() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Nat Tree")
        .expect("Tree should declare");
    let res = env.elaborate_decl(
        "view countR (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node (Node ll lc lr) c r => countR r ; \
           Node Leaf c r => countR r \
         }",
    );
    assert!(
        res.is_err(),
        "pins the CURRENT false-rejection (VAL2 #12); this must flip to Ok \
         once the completeness fix lands, without weakening the acceptance \
         criterion"
    );
}

/// **(a) discriminating near-miss (for the eventual AC1 net).** Shares the
/// EXACT syntactic shape as the shape-(a) repro (nested split, recursion
/// dispatched from inside it) but is genuinely non-terminating: one arm
/// recurses on the UNCHANGED original scrutinee `t` instead of a real
/// sub-part. A fix that makes shape (a) accept must still reject this.
#[test]
fn shape_a_near_miss_recurses_on_unchanged_scrutinee() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Nat Tree")
        .expect("Tree should declare");
    let res = env.elaborate_decl(
        "view bad (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node (Node ll lc lr) c r => bad t ; \
           Node Leaf c r => bad r \
         }",
    );
    assert!(
        res.is_err(),
        "genuinely non-terminating (recurses on `t` itself, no descent) — \
         must stay rejected under any candidate fix"
    );
}

/// **(b) Ackermann/lexicographic — reproduced.** `A(0,n)=n+1`,
/// `A(Suc m,0)=A(m,1)`, `A(Suc m,Suc n)=A(m, A(Suc m,n))`. Terminates under
/// the lexicographic order on `(m,n)`, but SCT currently rejects it.
///
/// Root cause (grounded): `size_rel` (`sct.rs:116-125`) returns `Unknown`
/// for any argument that is not a bare `Term::Var` — including a
/// constructor-application that exactly RECONSTRUCTS the matched scrutinee
/// (`Suc m2` in the `A(m, A(Suc m2, n2))`-shaped inner call, where `m2` was
/// bound by matching the SAME `m` against `Suc m2`, so `Suc m2 ≡ m`
/// definitionally). Tracing the compiled term: the inner self-call
/// `ack (Suc m2) n2` gets self-loop matrix `[[Unknown,Unknown],[Unknown,
/// Down]]` (missing the true `DownEq` at `[0][0]`); composed with the outer
/// call's matrix `[[Down,Unknown],[Unknown,Unknown]]`, the composition
/// `A∘B` collapses to the all-`Unknown` matrix (an `Unknown` step anywhere
/// breaks the whole composed thread, `compose_ord`, `sct.rs:31-38`) — an
/// idempotent self-loop with no strict diagonal ⇒ rejected. With the true
/// `DownEq` at `B[0][0]`, `compose_ord(Down, DownEq) = Down` survives
/// composition and the diagonal has its strict entry.
#[test]
fn shape_b_ackermann_lexicographic() {
    let mut env = fresh_env();
    let res = env.elaborate_decl(
        "view ack (m : Nat) (n : Nat) : Nat = \
         match m { \
           Zero => Suc n ; \
           Suc m2 => match n { \
             Zero => ack m2 (Suc Zero) ; \
             Suc n2 => ack m2 (ack (Suc m2) n2) \
           } \
         }",
    );
    assert!(
        res.is_err(),
        "pins the CURRENT false-rejection (Ackermann/lexicographic); must \
         flip to Ok once the completeness fix lands"
    );
}

/// **(b) discriminating near-miss.** Shares the exact "reconstruct the
/// matched param and pass it positionally" shape SCT must newly recognize
/// as `DownEq` — but is genuinely non-terminating: `n` is passed UNCHANGED
/// (not decremented), so `badAck (Suc m2) n` is definitionally identical to
/// the original call `badAck m n` (since `m = Suc m2` by the match). Both
/// relations are `DownEq`, never `Down` — a fix that (over-broadly) records
/// `Down` instead of `DownEq` for a constructor-reconstruction would wrongly
/// accept this; `has_strict_diagonal` requires an actual `Down`, so a
/// correctly-scoped fix keeps this rejected.
#[test]
fn shape_b_near_miss_reconstruction_with_unchanged_second_arg() {
    let mut env = fresh_env();
    let res = env.elaborate_decl(
        "view badAck (m : Nat) (n : Nat) : Nat = \
         match m { Zero => n ; Suc m2 => badAck (Suc m2) n }",
    );
    assert!(
        res.is_err(),
        "genuinely non-terminating (m reconstructed unchanged, n never \
         decreases) — must stay rejected under any candidate fix"
    );
}
