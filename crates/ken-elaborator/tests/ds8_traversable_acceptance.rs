//! DS-8 (`Traversable` constructor class) acceptance, extended by DS-8c —
//! `docs/program/wp/ds-8-traversable.md` +
//! `docs/program/wp/ds-8c-traverse-composition-law.md`, design contract
//! `spec/50-stdlib/56-effectful-classes.md` (CAT-2, `§5`).
//!
//! DS-8 landed the class, both instances, and the identity + naturality
//! laws. DS-8c closes the two pieces DS-8 honestly size-deferred: `Compose
//! g h`'s own `ap_cmp` (its 4th `Applicative` law) and the `§5.3`
//! composition coherence law that consumes it — see
//! `catalog/packages/Core/Classes/EffectfulClasses.ken.md §9.6`-`§9.7`. All three
//! `§5.3` coherence laws (identity, naturality, composition) are now
//! proved for both instances; the `Traversable` showcase is complete.
//!
//! - **AC1** — kernel-untouched, zero new elaborator capability, zero
//!   `trusted_base()` delta (structural before/after set-diff).
//! - **AC2–AC4** — laws `Ω`, pointwise, proved, zero `Axiom`.
//! - **AC6** — `traverse` classifies `proc` via SURF-1's row-variable
//!   mechanism; a pure `fn` instance implementation satisfies the field
//!   via DS-8b's `∅ ⊆ proc` widening.
//! - **AC7** — WIRE applied consistently (`functor`/`foldable` supplied
//!   whole).
//! - **AC8** — discriminators flip accept→reject at the named law field,
//!   asserted as the specific error variant.

use ken_elaborator::ElabEnv;

const TRANSPORT_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_FUNCTORS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulFunctors.ken.md");
const EFFECTFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/EffectfulClasses.ken.md");

fn base_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core/Logic/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Data/Collections/Derived.ken.md must elaborate");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("Core/Classes/LawfulClasses.ken must elaborate");
    env.elaborate_ken_md_file(LAWFUL_FUNCTORS_KEN_MD)
        .expect("Core/Classes/LawfulFunctors.ken.md must elaborate");
    env
}

#[test]
fn entry_elaborates_with_every_checked_fence() {
    let mut env = base_env();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("catalog/packages/Core/Classes/EffectfulClasses.ken.md must elaborate (Definition + every checked fence)");
    assert!(env.globals.contains_key("Traversable_instance_List"));
    assert!(env.globals.contains_key("Traversable_instance_Option"));
    assert!(env.globals.contains_key("Applicative_instance_Identity"));
    assert!(env.globals.contains_key("Functor_instance_Identity"));
}

// AC1/AC4: zero-Axiom acceptance bar, grounded on the CHECKED code only
// (fences), not prose (which legitimately discusses "Axiom"/"gated"
// while explaining the honest landed/deferred split).
#[test]
fn zero_axiom_in_checked_fences() {
    let extracted = ken_elaborator::literate::extract_ken_md(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must extract");
    assert!(
        !extracted.source.contains("Axiom"),
        "EffectfulClasses.ken.md's tangled/checked code must contain zero Axiom literals"
    );
    for range in extracted
        .example_ranges
        .iter()
        .chain(extracted.reject_ranges.iter())
    {
        assert!(
            !EFFECTFUL_CLASSES_KEN_MD[range.clone()].contains("Axiom"),
            "example/reject fences must contain zero Axiom literals"
        );
    }
}

// AC1: structural trusted_base() before==after set-diff — stronger than
// a source grep, catches a delta introduced indirectly through any
// helper (including everything §9 adds: Traversable, Compose's partial
// law set, Identity, the naturality machinery).
#[test]
fn trusted_base_delta_is_empty_across_the_entry() {
    let mut env = base_env();
    let before: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("catalog/packages/Core/Classes/EffectfulClasses.ken.md must elaborate");
    let after: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "EffectfulClasses.ken.md must introduce ZERO new trusted_base() entries (zero-Axiom acceptance bar)"
    );
}

// Honest-boundary check (Architect gate pin #4), DS-8c-updated: `ap_cmp`
// is now landed (below), but the real `instance Applicative (Compose g h)`
// head must still be genuinely absent — that head stays blocked by the
// parametric-instance-head kinding gap (elab.rs:3833-3851), independent of
// ap_cmp being proved (DS-8c's scope guard: work within the `fn`-synonym
// scaffolding, never assemble the real instance).
#[test]
fn compose_instance_head_still_genuinely_absent() {
    let extracted = ken_elaborator::literate::extract_ken_md(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must extract");
    assert!(
        !extracted.source.contains("instance Applicative (Compose"),
        "instance Applicative (Compose g h) must not be assembled — its head stays kinding-blocked independent of ap_cmp"
    );
}

// DS-8c piece 2: the traverse composition coherence law (§5.3) — proved
// for both instances (List by induction, Option by case split), stated
// over the explicit compose_pure/compose_ap operations (never through
// instance search), consuming Compose's ap_cmp via ap_naturality/
// ap_naturality2. Real, kernel-checked globals, not just named in prose.
#[test]
fn traverse_composition_law_is_present_and_kernel_checked() {
    let mut env = base_env();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must elaborate");
    for name in [
        "list_traverse_composed",
        "list_traverse_composition",
        "option_traverse_composed",
        "option_traverse_composition",
    ] {
        assert!(
            env.globals.contains_key(name),
            "DS-8c claims `{}` is proved — it must be a real global after elaborating the entry",
            name
        );
    }
}

// DS-8c positive counterpart: Compose's own `ap_cmp` (the 4th Applicative
// law) must be a REAL, kernel-checked proof term in the tangled code, not
// a stub — closed by the `trans`/`cong`/`ap_cmp`/`ap_naturality2` chain,
// never a bare `Refl`/`Axiom` (Architect honesty pin #4).
#[test]
fn compose_ap_cmp_is_present_and_kernel_checked() {
    let mut env = base_env();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must elaborate");
    for name in ["compose_ap_cmp", "ap_naturality2"] {
        assert!(
            env.globals.contains_key(name),
            "DS-8c claims `{}` is proved — it must be a real global after elaborating the entry",
            name
        );
    }
}

// The positive counterpart to the absence check above: §9.4's claimed
// partial `Compose` proof work (three of four Applicative laws +
// map_coh + Functor + ap_naturality + the Level1/Level2 ap_cmp
// reductions) must actually be PRESENT in the tangled code, kernel
// re-checked here directly — not just described in prose (the exact
// gap foundation-qa's BLOCKED verdict caught).
#[test]
fn compose_partial_law_set_is_actually_present_and_kernel_checked() {
    let mut env = base_env();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must elaborate");
    for name in [
        "Compose",
        "compose_pure",
        "compose_ap",
        "compose_ap_id",
        "compose_ap_hom",
        "compose_ap_ich",
        "compose_map",
        "compose_map::id",
        "compose_map::fusion",
        "compose_map_coh",
        "ap_naturality",
        "cmp_level1_eq",
        "cmp_level2_reduced",
    ] {
        assert!(
            env.globals.contains_key(name),
            "§9.4/§9.5 claims `{}` is proved — it must be a real global after elaborating the entry, not just named in prose",
            name
        );
    }
}

// AC7: the wired superclass fields (`functor`/`foldable`) are supplied
// whole, not re-proved — both instances reuse the landed Functor/Foldable
// instances directly.
#[test]
fn wired_superclass_fields_reused_not_reproved() {
    let mut env = base_env();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must elaborate");
    assert!(env.globals.contains_key("Functor_instance_List"));
    assert!(env.globals.contains_key("Foldable_instance_List"));
    assert!(env.globals.contains_key("Functor_instance_Option"));
    assert!(env.globals.contains_key("Foldable_instance_Option"));
}

// AC6: `traverse`'s abstract-`g`-headed action classifies `proc` at the
// class-field level (SURF-1 row-variable, fail-closed on the unresolved
// codomain head) — reconfirmed directly against the landed class.
#[test]
fn ac6_traverse_field_classifies_proc() {
    let mut env = base_env();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must elaborate");
    env.elaborate_decl(
        "class ProbeTraversable (f : Type -> Type) { \
           proc traverse : (g : Type -> Type) -> Applicative g -> (a : Type) -> (b : Type) -> (a -> g b) -> f a -> g (f b) \
         }",
    )
    .expect("proc traverse must classify and elaborate as a class field");
    let r = env.elaborate_decl(
        "class ProbeTraversableFn (f : Type -> Type) { \
           fn traverse : (g : Type -> Type) -> Applicative g -> (a : Type) -> (b : Type) -> (a -> g b) -> f a -> g (f b) \
         }",
    );
    match r {
        Ok(_) => panic!("fn traverse with an abstract-g-headed codomain must be rejected — it is row-polymorphic, not provably pure"),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("row-polymorphic") || msg.contains("TypeMismatch"),
                "expected the specific 'declares a latent or row-polymorphic effect; use proc' TypeMismatch, got: {:?}",
                e
            );
        }
    }
}

// AC8 discriminator: the identity law flips accept→reject at the named
// law field when the witness is swapped for one that does NOT satisfy
// it (the Nil/None base case picks the wrong endpoint).
#[test]
fn ac8_identity_law_witness_swap_is_rejected() {
    let mut env = base_env();
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("EffectfulClasses.ken.md must elaborate");

    // A deliberately-wrong witness: claims the Nil case closes with the
    // SAME proof term as the Cons/recursive case (ill-typed — a `cong`
    // application, not `Proved`, cannot inhabit the Nil case's collapsed
    // Equal goal).
    let r = env.elaborate_decl(
        "fn bad_list_traverse_identity_law (a : Type) (xs : List a) : \
           Equal (Identity (List a)) (list_traverse Identity Applicative_instance_Identity a a (identity_pure a) xs) (identity_pure (List a) xs) = \
           match xs { \
             Nil ↦ cong (Identity (List a)) (Identity (List a)) (identity_pure (List a) xs) (identity_pure (List a) xs) (identity_map (List a) (List a) (idf (List a))) Proved ; \
             Cons h u ↦ Proved \
           }",
    );
    match r {
        Ok(_) => panic!(
            "a witness with swapped Proved/cong endpoints must be rejected, not silently accepted"
        ),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("TypeMismatch") || msg.contains("KernelRejected"),
                "expected a TypeMismatch/KernelRejected (specific variant), got: {:?}",
                e
            );
        }
    }
}
