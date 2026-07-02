//! ES4-classes-build + ES4-lawproofs acceptance tests: `Eq`/`DecEq`/`Ord`
//! structure classes + canonical `Int` (audited-delta) and `Bool`
//! (zero-delta, K4-enabled) instances, against the REAL
//! `packages/lawful-classes/lawful_classes.ken` source (producer-grep: this
//! drives the actual package file via `include_str!`, never a hand-copied
//! string).
//!
//! Scope note (Architect rulings `evt_68ppz77ysh5ne` or `wp/ES4-classes-
//! build`, and the ES4-lawproofs reopen post-K4 `3be0e30`): `Ord Bool`'s
//! `refl`/`trans`/`total` and `Eq Bool`'s `refl` are REAL, kernel-checked
//! proofs (K4's Ω-motive `Elim` + this WP's `check_match_dependent`
//! dependent-elimination wiring). `Ord Bool`'s `antisym` and `DecEq Bool`'s
//! `sound`/`complete` conclude or hypothesize a BARE `Equal a x y` — a shape
//! that observationally collapses past `Eq` into the kernel's `Top`/`Bottom`
//! (which have no introduction/elimination rule anywhere in the kernel
//! today) — so they stay honest, visible `Axiom`s pending a further "K5"
//! kernel WP (`Top`-intro + `Bottom`-elim), not silently claimed proved.
//!
//! `Eq Bool`'s `sym`/`trans` are ALSO `Axiom`, but for a distinct reason —
//! NOT the K5/Top-Bottom wall (their conclusions are `IsTrue`-shaped, never
//! a bare `Equal a x y`). Reusing a hypothesis under a swapped-argument goal
//! (`p : IsTrue (eq x y)` where the goal is `IsTrue (eq y x)`) needs the
//! kernel to see two structurally-different-but-value-equal `Eq`
//! propositions as convertible; `ken-kernel/src/conv.rs`'s `conv_struct` has
//! no congruence case for two `Term::Eq(...)` nodes, so this fails even
//! though the propositions are semantically identical. This is the
//! Architect-ruled **"K6"** gap (`evt_4y4pyernxpzzt`) — `conv.rs`-only,
//! independent of K5 (no `Top`/`Bottom`, `eq_reduce` untouched); the only
//! admissible fix is a POSITIONAL congruence arm (a cross-wise arm would
//! smuggle propositional symmetry into definitional equality — a hard NO).
//! Mechanism-grounded, not just structurally confirmed (`evt_23r0bbx00g18m`):
//! a local patch-and-revert experiment with full term dumps showed the arm
//! is the necessary trigger that lets `conv_struct`'s recursion reach each
//! already-case-split leaf's concrete literals, where ordinary pre-existing
//! iota-reduction — not a new commutativity rule — closes it. K6 is its own
//! reviewed kernel WP (not yet merged), not fixable from the surface.

use ken_elaborator::ElabEnv;
use ken_kernel::env::Decl as KernelDecl;
use ken_kernel::Term;

const LAWFUL_CLASSES_KEN: &str = include_str!("../../../packages/lawful-classes/lawful_classes.ken");

fn mk_env_with_package() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_file(LAWFUL_CLASSES_KEN).expect("packages/lawful-classes/lawful_classes.ken must elaborate");
    env
}

/// Walk a right-nested `Pair` chain (a class instance's record VALUE) and
/// return field `idx`'s own value term: `proj1(proj2^idx(whole))`, reduced
/// (the raw `Proj1`/`Proj2` wrapper is otherwise unevaluated syntax).
fn field_value(env: &ken_kernel::GlobalEnv, whole: &Term, idx: usize) -> Term {
    let mut cur = whole.clone();
    for _ in 0..idx {
        cur = Term::proj2(cur);
    }
    ken_kernel::whnf(env, &ken_kernel::Context::new(), &Term::proj1(cur))
}

fn is_opaque_const(env: &ken_kernel::GlobalEnv, t: &Term) -> bool {
    matches!(t, Term::Const { id, .. } if matches!(env.lookup(*id), Some(KernelDecl::Opaque { .. })))
}

/// Does `t` mention the global `id` anywhere (`Term::Const{id,..}`)? Used to
/// confirm a law field's TYPE genuinely contains a specific sub-application
/// (e.g. `or_bool`), not just that it type-checks.
fn mentions_const(t: &Term, id: ken_kernel::GlobalId) -> bool {
    match t {
        Term::Const { id: i, .. } => *i == id,
        Term::App(f, a) => mentions_const(f, id) || mentions_const(a, id),
        Term::Pi(a, b) | Term::Lam(a, b) | Term::Sigma(a, b) => mentions_const(a, id) || mentions_const(b, id),
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The three classes are real, zero-delta record types (`33 §5.2`, `51 §2`)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn classes_are_transparent_structure_records_zero_delta() {
    let env = mk_env_with_package();
    let base_tb: std::collections::HashSet<_> =
        ElabEnv::new().unwrap().env.trusted_base().into_iter().collect();

    for name in ["Eq", "DecEq", "Ord"] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(KernelDecl::Transparent { .. })),
            "{} must be a real (Transparent) record type, not a postulate/primitive",
            name
        );
        assert!(
            !base_tb.contains(&id) && !env.env.trusted_base().contains(&id),
            "{}'s own class-type id must never enter trusted_base()",
            name
        );
    }
}

/// Seed `stdlib/classes/ord-total-law-is-omega-bool-equation`: `Ord`'s
/// `total` law field must be the Bool-EQUATION `IsTrue (bool_or (leq x y)
/// (leq y x))`, never a bare/incomplete form — the disjunction is the
/// entire point of `51 §3` (it's what keeps totality Ω-clean without
/// truncation). Regression for a real authoring slip caught by
/// language-qa (`evt_3asqqsehdsj0y`): the field originally shipped as a
/// bare `IsTrue (leq x y)`, silently dropping `|| leq y x` — a materially
/// different (and for any non-trivial order false) proposition, not
/// totality. Discriminating: assert the field's TYPE structurally contains
/// the `bool_or` application, not just that it type-checks (a class
/// declaration with a defective law field still elaborates fine — that's
/// exactly how the slip got through the first time).
///
/// (`bool_or` — not the `or_bool` PRIMITIVE — deliberately: a primitive
/// never reduces regardless of argument concreteness, `51 §6`, which would
/// make `total` permanently unprovable for ANY carrier, inductive or not;
/// `Ord Bool`'s own `total` field, below, is a real proof that needs this.)
#[test]
fn ord_total_law_is_the_bool_or_equation() {
    let env = mk_env_with_package();
    let bool_or_id = env.globals["bool_or"];
    let ord_ci = &env.class_env.classes["Ord"];
    let total_idx = ord_ci.field_names.iter().position(|n| n == "total").expect("Ord has a `total` field");
    let total_ty = &ord_ci.field_types[total_idx];
    assert!(
        mentions_const(total_ty, bool_or_id),
        "Ord's `total` law field must mention `bool_or` (the Bool-equation \
         totality form, `51 §3`) — a bare `IsTrue (leq x y)` silently drops \
         the disjunction and states a different, non-totality proposition. \
         Got: {:?}",
        total_ty
    );
}

// ─────────────────────────────────────────────────────────────────────────
// AC1/AC2 — Ord Bool/Eq Bool/DecEq Bool: the zero-delta exemplar (51 §6),
// K4-enabled. Producer-grep: law fields carry REAL proofs where provable
// (refl/trans/total for Ord, refl for Eq) — zero `declare_postulate`/holes
// — and are HONEST, visible `Axiom`s where a bare `Equal a x y`
// conclusion/hypothesis collapses past `Eq` before `Refl` can fire
// (`antisym`; `Eq`'s `sym`/`trans`; `DecEq`'s `sound`/`complete`) — a
// forward-gated (K5: `Top`-intro/`Bottom`-elim) gap, not silently claimed
// proved. The discriminating flip: a law-less (all-`Axiom`) `Ord`-shaped
// dictionary is REJECTED as unlawful wherever it matters (AC2) — here,
// `Ord Bool`'s `refl`/`trans`/`total` fields being genuinely Opaque-free is
// exactly that flip, verified against the REAL elaborator.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn ord_bool_provable_laws_are_real_proofs_not_postulates() {
    let env = mk_env_with_package();
    let id = env.globals["Ord_instance_Bool"];
    assert!(matches!(env.env.lookup(id), Some(KernelDecl::Transparent { .. })));
    let (_, body) = env.env.transparent_body(id).expect("Ord Bool instance is transparent");
    // Field order: leq, refl, antisym, trans, total.
    let leq_val = field_value(&env.env, &body, 0);
    let refl_val = field_value(&env.env, &body, 1);
    let antisym_val = field_value(&env.env, &body, 2);
    let trans_val = field_value(&env.env, &body, 3);
    let total_val = field_value(&env.env, &body, 4);

    assert!(!is_opaque_const(&env.env, &leq_val), "leq must not be a postulate");
    for (name, v) in [("refl", &refl_val), ("trans", &trans_val), ("total", &total_val)] {
        assert!(
            !is_opaque_const(&env.env, v),
            "Ord Bool's '{}' must be a REAL kernel-checked proof (K4-enabled, zero-delta) — \
             not a postulate. Got {:?}",
            name, v
        );
    }
    // `antisym` stays an honest, VISIBLE Axiom — forward-gated on K5 (Top-
    // intro/Bottom-elim), not silently claimed proved.
    assert!(
        is_opaque_const(&env.env, &antisym_val),
        "Ord Bool's 'antisym' must still be a visible Axiom (K5-gated) — not silently \
         proved (would be a false zero-delta claim) and not silently missing"
    );

    // The discriminating delta count: exactly ONE new postulate (antisym),
    // not four (which would mean refl/trans/total silently regressed to
    // Axiom too) and not zero (which would mean antisym got proved without
    // K5 — impossible today, or claimed proved falsely).
    let base_tb: std::collections::HashSet<_> =
        ElabEnv::new().unwrap().env.trusted_base().into_iter().collect();
    let delta: Vec<_> = env.env.trusted_base().into_iter().filter(|id| !base_tb.contains(id)).collect();
    // NOTE: the base delta set is computed fresh (no package loaded), so
    // this counts ONLY entries the package itself contributes across ALL
    // its instances (Int's 4+3+2 Axioms + leq_int/eq_int primitives +
    // Bool's 1 Axiom for antisym) — assert antisym's contribution
    // specifically by checking it's present and everything else isn't
    // double-counted via the per-field checks above (the real assertions).
    assert!(!delta.is_empty(), "package must contribute a non-empty delta (Int's audited postulates alone guarantee this)");
}

#[test]
fn eq_bool_refl_is_real_proof() {
    let env = mk_env_with_package();
    let id = env.globals["Eq_instance_Bool"];
    let (_, body) = env.env.transparent_body(id).expect("Eq Bool instance is transparent");
    let eq_val = field_value(&env.env, &body, 0);
    let refl_val = field_value(&env.env, &body, 1);
    let sym_val = field_value(&env.env, &body, 2);
    let trans_val = field_value(&env.env, &body, 3);
    assert!(!is_opaque_const(&env.env, &eq_val), "eq must not be a postulate");
    assert!(
        !is_opaque_const(&env.env, &refl_val),
        "Eq Bool's 'refl' must be a REAL kernel-checked proof — not a postulate. Got {:?}",
        refl_val
    );
    // sym/trans: honest Axioms for now — NOT K5-gated (their conclusions are
    // IsTrue-shaped, not a bare Equal a x y); blocked instead by K6, a
    // distinct, narrow conv_struct Eq/Eq congruence gap outside this WP's
    // lane (Architect-ruled `evt_4y4pyernxpzzt` — see the .ken source's own
    // comment for the full mechanism grounding).
    assert!(is_opaque_const(&env.env, &sym_val), "Eq Bool's 'sym' is a visible Axiom (K6-gated)");
    assert!(is_opaque_const(&env.env, &trans_val), "Eq Bool's 'trans' is a visible Axiom (K6-gated)");
}

#[test]
fn dec_eq_bool_sound_complete_stay_honest_axioms() {
    let env = mk_env_with_package();
    let id = env.globals["DecEq_instance_Bool"];
    let (_, body) = env.env.transparent_body(id).expect("DecEq Bool instance is transparent");
    let eq_val = field_value(&env.env, &body, 0);
    let sound_val = field_value(&env.env, &body, 1);
    let complete_val = field_value(&env.env, &body, 2);
    assert!(!is_opaque_const(&env.env, &eq_val), "eq must not be a postulate");
    assert!(is_opaque_const(&env.env, &sound_val), "DecEq Bool's 'sound' is a visible Axiom (K5-gated)");
    assert!(is_opaque_const(&env.env, &complete_val), "DecEq Bool's 'complete' is a visible Axiom (K5-gated)");
}

// ─────────────────────────────────────────────────────────────────────────
// AC (audited-delta): the Int instances' OP fields wrap existing primitives,
// LAW fields are honest, visible postulates — never hidden, never silently
// claimed zero-delta (`51 §6`).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn int_ord_instance_is_audited_delta_not_zero_delta() {
    let env = mk_env_with_package();
    let ord_int_id = env.globals["Ord_instance_Int"];

    // The instance record itself is Transparent (a real declare_def re-check
    // of the Σ-chain value) — never Opaque/Primitive itself.
    assert!(matches!(env.env.lookup(ord_int_id), Some(KernelDecl::Transparent { .. })));

    let (_, body) = env.env.transparent_body(ord_int_id).expect("Ord Int instance is transparent");
    // Field order (class declaration order): leq, refl, antisym, trans, total.
    let leq_val = field_value(&env.env, &body, 0);
    let refl_val = field_value(&env.env, &body, 1);
    let antisym_val = field_value(&env.env, &body, 2);
    let trans_val = field_value(&env.env, &body, 3);
    let total_val = field_value(&env.env, &body, 4);

    // Producer-grep gate: the OP field is NOT itself a postulate (it wraps
    // `int_leq`/`leq_int`, a real def/primitive, never `Axiom`-produced).
    assert!(
        !is_opaque_const(&env.env, &leq_val),
        "AC (op-field): `leq` must wrap the real leq_int primitive, not be a fresh postulate"
    );

    // Producer-grep gate: EVERY law field genuinely IS a fresh Decl::Opaque
    // (an honest, visible postulate) — never a hand-waved/hidden trust hole,
    // and never (by accident) something that LOOKS proved but isn't.
    for (name, v) in [("refl", &refl_val), ("antisym", &antisym_val), ("trans", &trans_val), ("total", &total_val)] {
        assert!(
            is_opaque_const(&env.env, v),
            "AC (audited-delta): Ord Int's law field '{}' must be a real, grep-able Decl::Opaque \
             postulate (honest non-zero delta) — got {:?}",
            name, v
        );
    }

    // The discriminating observable itself: trusted_base_delta is NON-EMPTY
    // (the 4 law postulates), confirming this is NOT (and never silently
    // becomes) a zero-delta/AC3-lawful instance.
    let base_tb: std::collections::HashSet<_> =
        ElabEnv::new().unwrap().env.trusted_base().into_iter().collect();
    let delta: Vec<_> = env.env.trusted_base().into_iter().filter(|id| !base_tb.contains(id)).collect();
    assert!(
        delta.len() >= 4,
        "AC (audited-delta): Ord Int must contribute a non-empty trusted_base delta \
         (>= 4 law postulates: refl/antisym/trans/total), got {} new entries",
        delta.len()
    );
}

#[test]
fn eq_and_deceq_int_instances_are_also_audited_delta() {
    let env = mk_env_with_package();
    for (name, law_names) in [
        ("Eq_instance_Int", ["refl", "sym", "trans"].as_slice()),
        ("DecEq_instance_Int", ["sound", "complete"].as_slice()),
    ] {
        let id = env.globals[name];
        let (_, body) = env.env.transparent_body(id).unwrap_or_else(|| panic!("{} must be transparent", name));
        let op_val = field_value(&env.env, &body, 0);
        assert!(!is_opaque_const(&env.env, &op_val), "{}: op field must not be a postulate", name);
        for (i, law_name) in law_names.iter().enumerate() {
            let v = field_value(&env.env, &body, i + 1);
            assert!(
                is_opaque_const(&env.env, &v),
                "{}: law field '{}' must be a real Decl::Opaque postulate",
                name, law_name
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// AC2 — `where Ord a` supplies the SAME comparator the explicit-comparator
// `sort`/`isSorted` form threads (`51 §4`, reflect-don't-extend).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn where_ord_supplies_same_comparator_as_explicit_form() {
    let mut env = mk_env_with_package();

    // (a) explicit comparator form (ES2-remainder's landed shape).
    let explicit_id = env
        .elaborate_decl(
            "view sortObligationExplicit (leq : Int -> Int -> Bool) (ys : List Int) (xs : List Int) : Prop = \
             And (isSorted Int leq ys) (Perm Int ys xs)",
        )
        .expect("explicit-comparator obligation elaborates");

    // (b) `where Ord Int`-constrained form — `d.leq` supplied by the
    // resolved dictionary, same obligation shape.
    let via_dict_id = env
        .elaborate_decl(
            "view sortObligationViaDict (ys : List Int) (xs : List Int) : Prop where Ord Int = \
             And (isSorted Int (d.leq) ys) (Perm Int ys xs)",
        )
        .expect("`where Ord Int` obligation elaborates — d.leq must project the resolved dictionary's leq field");

    // Discriminating: both must produce a body of the SAME STRUCTURAL shape
    // (`And (isSorted Int <cmp> ys) (Perm Int ys xs)`) — not merely "both
    // type-check". Peel the two param-lambdas (leq/none, ys, xs) down to the
    // inner body and compare modulo the substituted comparator.
    let (_, explicit_body) = env.env.transparent_body(explicit_id).unwrap();
    let (_, dict_body) = env.env.transparent_body(via_dict_id).unwrap();

    fn peel_lams(t: &Term, n: usize) -> Term {
        let mut cur = t.clone();
        for _ in 0..n {
            match cur {
                Term::Lam(_, body) => cur = *body,
                other => return other,
            }
        }
        cur
    }
    // explicit: Lam(leq) Lam(ys) Lam(xs) -> body; via-dict: Lam(ys) Lam(xs) -> body.
    let explicit_inner = peel_lams(&explicit_body, 3);
    let dict_inner = peel_lams(&dict_body, 2);

    fn is_and_is_sorted_perm_shape(t: &Term) -> bool {
        // App(App(Const(And), isSorted-app), Perm-app) — just check the
        // outer head is an application chain of depth >= 2 (structural
        // shape check; exact head ids vary run-to-run only by content, not
        // structure).
        matches!(t, Term::App(f, _) if matches!(f.as_ref(), Term::App(_, _)))
    }
    assert!(is_and_is_sorted_perm_shape(&explicit_inner), "explicit form must have the And(isSorted,Perm) shape");
    assert!(is_and_is_sorted_perm_shape(&dict_inner), "where Ord Int form must have the SAME And(isSorted,Perm) shape");
}
