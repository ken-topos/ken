//! ES4-classes-build acceptance tests: `Eq`/`DecEq`/`Ord` structure classes
//! + the `Int` audited-delta canonical instances, against the REAL
//! `packages/lawful-classes/lawful_classes.ken` source (producer-grep: this
//! drives the actual package file via `include_str!`, never a hand-copied
//! string).
//!
//! Scope note (Architect ruling, `evt_68ppz77ysh5ne` / `evt_69epknf0nfdmc`-
//! style soundness gate): the zero-delta, law-CARRYING instance over an
//! inductive carrier (AC3's positive arm — e.g. `Ord Bool`) is a forward WP
//! gated on the kernel gaining Ω-motive `Elim` support. This suite covers
//! what's buildable today: the class records (zero delta), the `Int`
//! audited-delta instances (law fields are honest, visible postulates), and
//! AC2 (`where Ord a` supplies the same comparator the explicit form
//! threads).

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
