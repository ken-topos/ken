//! DS-6a (ADR 0013 Layer 1) acceptance at the elaborator layer — `Int`'s
//! REAL certificate registration (`crates/ken-elaborator/src/numbers.rs`)
//! and the catalog rewiring's conformance bar (`docs/program/wp/
//! DS-6a-int-deceq-certificate.md`), against the REAL
//! `catalog/packages/Core/Classes/LawfulClasses.ken.md` source. The general
//! mechanism + its registration-time hardening (negative arms) are covered
//! kernel-side in `ken-kernel/tests/ds6a_int_deceq_certificate.rs`; the
//! zero-Axiom-delta / real-proof discriminators for `Eq Int`/`DecEq Int`
//! themselves live in `es4_classes_acceptance.rs` (the existing home for
//! this file's law-field producer-grep gates).

use ken_elaborator::ElabEnv;
use ken_kernel::env::{Context, Decl as KernelDecl};
use ken_kernel::term::Term;
use ken_kernel::{check, whnf, KernelError};

const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");

fn mk_env_with_package() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("catalog/packages/Core/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("catalog/packages/Data/Collections/Collections.ken must elaborate");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("catalog/packages/Core/LawfulClasses.ken must elaborate");
    env
}

/// Walk a right-nested `Pair` chain (a class instance's record VALUE) and
/// return field `idx`'s own SOURCE term exactly as elaborated, no `whnf`
/// (mirrors `es4_classes_acceptance.rs::field_raw` — duplicated locally
/// since that helper is private to its own test binary).
fn field_raw(whole: &Term, idx: usize) -> Term {
    let mut cur = whole.clone();
    for _ in 0..idx {
        cur = match cur {
            Term::Pair(_, b) => *b,
            other => panic!("expected a Pair chain at depth {}, got {:?}", idx, other),
        };
    }
    match cur {
        Term::Pair(a, _) => *a,
        other => panic!("expected a Pair at depth {}, got {:?}", idx, other),
    }
}

/// Builds `proj1(proj2^idx(Const(id)))` — the exact shape a `.field`
/// projection produces (mirrors `es4_classes_acceptance.rs::expected_field_proj`).
fn expected_field_proj(id: ken_kernel::GlobalId, idx: usize) -> Term {
    let mut cur = Term::const_(id, vec![]);
    for _ in 0..idx {
        cur = Term::proj2(cur);
    }
    Term::proj1(cur)
}

// ─────────────────────────────────────────────────────────────────────────
// `DecEq Char` — rides free by transport from `DecEq Int` (ADR 0013's
// stated Layer-1 consequence), the same shape as the pre-existing
// `Ord Char` transport.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn deceq_char_transports_from_deceq_int_not_a_fresh_postulate() {
    let env = mk_env_with_package();
    let id = env.globals["DecEq_instance_Char"];
    let dec_eq_int_id = env.globals["DecEq_instance_Int"];
    assert!(matches!(env.env.lookup(id), Some(KernelDecl::Transparent { .. })));
    let (_, body) = env
        .env
        .transparent_body(id)
        .expect("DecEq Char instance is transparent");

    for (name, idx) in [("eq", 0), ("sound", 1), ("complete", 2)] {
        let raw = field_raw(&body, idx);
        let expected = expected_field_proj(dec_eq_int_id, idx);
        assert_eq!(
            raw, expected,
            "DecEq Char's '{}' must be a direct `.`-projection off DecEq_instance_Int's \
             own field {} (honest transport), not a fresh construction",
            name, idx
        );
    }

    // Zero-NEW-delta by transport: the raw (unreduced) projection syntax
    // doesn't itself mention any trusted_base Const (it only reaches
    // `Const{dec_eq_int_id}`, which is Transparent, not trusted) — same
    // shape as `Ord_instance_Char`'s own zero-NEW-delta.
    let mut delta = ken_elaborator::trusted_base_delta(&env.env, id);
    delta.remove(&env.class_env.record_nil_val_id);
    assert!(
        delta.is_empty(),
        "DecEq Char must be zero-NEW-delta by transport — got {:?}",
        delta
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Neutral preserved (ADR 0013's discriminating arm): DS-6a registers a
// certificate but adds NO reduction rule — `obs.rs::eq_reduce` is
// untouched, so `Eq`/`Equal` at an abstract operand pair stays exactly as
// unreachable as before, for both the registered `Int` and an unregistered
// primitive.
// ─────────────────────────────────────────────────────────────────────────

fn assert_abstract_eq_stays_neutral(env: &ElabEnv, prim_id: ken_kernel::GlobalId) {
    let prim_const = Term::const_(prim_id, vec![]);
    let mut ctx = Context::new();
    ctx.push(prim_const.clone());
    ctx.push(prim_const.clone());
    // x = Var(1), y = Var(0): both bound, abstract, unconstrained.
    let eq_ty = Term::Eq(
        Box::new(prim_const),
        Box::new(Term::var(1)),
        Box::new(Term::var(0)),
    );
    let reduced = whnf(&env.env, &ctx, &eq_ty);
    assert!(
        matches!(reduced, Term::Eq(..)),
        "Eq at abstract operands must stay neutral (Term::Eq), got {:?}",
        reduced
    );
}

#[test]
fn abstract_int_equality_stays_neutral() {
    let env = mk_env_with_package();
    assert_abstract_eq_stays_neutral(&env, env.numeric_env.int_id);
}

#[test]
fn unregistered_primitive_equality_stays_neutral() {
    let env = mk_env_with_package();
    // `Float` never registers a certificate (DS-6a scopes `Int` only) —
    // confirms the mechanism is genuinely opt-in, not a blanket change to
    // primitive-`Eq` behavior.
    assert!(env.env.deceq_cert(env.numeric_env.float_id).is_none());
    assert_abstract_eq_stays_neutral(&env, env.numeric_env.float_id);
}

// ─────────────────────────────────────────────────────────────────────────
// Over-equate regression pin (soundness). Per the Architect's conformance
// clarification: under DS-6a ALONE (no native `IntLit` reduction — that is
// DS-6b, separate and later), a proof of a CONCRETE false equality like
// `Equal Int 5 6` cannot even be phrased as a reducible goal (`eq_int`
// never fires on literals in K1 either way) — so the discriminating,
// DS-6a-scoped regression pin is: the certificate opens no new path to
// equating two genuinely DISTINCT `Int`s at all, concrete or abstract.
// Exercised at the most basic level `Refl` already covers, confirming
// DS-6a's registration didn't loosen `check`'s `Refl` rule or `conv`.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn distinct_int_variables_refl_still_rejected() {
    let env = mk_env_with_package();
    let int_const = Term::const_(env.numeric_env.int_id, vec![]);
    let mut ctx = Context::new();
    ctx.push(int_const.clone());
    ctx.push(int_const.clone());
    let eq_ty = Term::Eq(
        Box::new(int_const),
        Box::new(Term::var(1)),
        Box::new(Term::var(0)),
    );
    // `Refl (Var(1))` witnesses `x = x`, not the goal `x = y`.
    let err = check(&env.env, &ctx, &Term::Refl(Box::new(Term::var(1))), &eq_ty)
        .expect_err("Refl must not prove two distinct Int variables equal");
    assert!(
        matches!(err, KernelError::BadEliminator(_)),
        "expected BadEliminator, got {:?}",
        err
    );
}
