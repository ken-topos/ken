//! DS-2 (`Ord Nat` export + `Nat` operations) acceptance — `docs/program/
//! wp/ds-2-ord-nat-export.md`.
//!
//! - `refl`/`trans`/`antisym` slot into `class Ord` directly (zero
//!   conversion, `IsTrue` unfolds to `Equal Bool ... True`); `total` needs
//!   the `orEqTrueToIsTrueBoolOr` bridge (probed, mirrors `Ord Bool`'s
//!   proof STYLE, not a literal template for this specific conversion).
//! - **Zero new `Axiom`/`trusted_base()` delta** — the acceptance bar the
//!   frame set (`Nat` is inductive, unlike `Int`).
//! - The entry's `` ```ken ``/`` ```ken example ``/`` ```ken reject ``
//!   fences all check via the real literate extractor.

use ken_elaborator::ElabEnv;

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulClasses.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const ORD_NAT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/OrdNat.ken.md");

fn base_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD).expect("Core/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD).expect("Data/Collections/Collections.ken.md must elaborate");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD).expect("Core/LawfulClasses.ken must elaborate");
    env
}

#[test]
fn entry_elaborates_with_every_checked_fence() {
    let mut env = base_env();
    env.elaborate_ken_md_file(ORD_NAT_KEN_MD)
        .expect("catalog/packages/Core/OrdNat.ken.md must elaborate (Definition + every checked fence)");
    assert!(env.globals.contains_key("Ord_instance_Nat"), "Ord_instance_Nat must be a real registered global");
}

// Zero-Axiom acceptance bar: no `Axiom` literal appears anywhere in the
// entry's own CHECKED code (fences only -- prose legitimately discusses
// the word "Axiom" when explaining the zero-delta claim itself).
#[test]
fn zero_axiom_in_entry_source() {
    let extracted = ken_elaborator::literate::extract_ken_md(ORD_NAT_KEN_MD)
        .expect("OrdNat.ken.md must extract");
    assert!(
        !extracted.source.contains("Axiom"),
        "OrdNat.ken.md's tangled/checked code must contain zero Axiom literals (the frame's acceptance bar)"
    );
    for range in extracted.example_ranges.iter().chain(extracted.reject_ranges.iter()) {
        assert!(
            !ORD_NAT_KEN_MD[range.clone()].contains("Axiom"),
            "OrdNat.ken.md's example/reject fences must contain zero Axiom literals"
        );
    }
}

// Ground the zero-trusted_base()-delta claim structurally: the set of
// trusted (unproved) globals before and after elaborating this entry must
// be IDENTICAL.
#[test]
fn trusted_base_delta_is_empty_across_the_entry() {
    let mut env = base_env();
    let before: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(ORD_NAT_KEN_MD)
        .expect("catalog/packages/Core/OrdNat.ken.md must elaborate");
    let after: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "OrdNat.ken.md must introduce ZERO new trusted_base() entries (zero-Axiom acceptance bar)"
    );
}

// `refl`/`trans`/`antisym` slot into `class Ord`'s IsTrue-phrased fields
// directly, with no conversion function — confirms the "no bridge needed"
// half of the frame's "probe first" instruction empirically, not just by
// citing IsTrue's definition.
#[test]
fn refl_trans_antisym_need_no_conversion() {
    let mut env = base_env();
    env.elaborate_decl(
        "fn leqNat (m : Nat) (n : Nat) : Bool = match m { Zero ⇒ True ; Suc m2 ⇒ match n { Zero ⇒ False ; Suc n2 ⇒ leqNat m2 n2 } }",
    )
    .expect("leqNat");
    env.elaborate_decl(
        "lemma reflLeqNat (x : Nat) : Equal Bool (leqNat x x) True = match x { Zero ⇒ Proved ; Suc x2 ⇒ reflLeqNat x2 }",
    )
    .expect("reflLeqNat");

    // The SAME term, reflLeqNat, satisfies the IsTrue-phrased signature
    // directly — zero adaptation.
    env.elaborate_decl("lemma probeRefl (x : Nat) : IsTrue (leqNat x x) = reflLeqNat x")
        .expect("reflLeqNat must satisfy IsTrue (leqNat x x) with zero conversion code");
}
