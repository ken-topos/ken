//! `Coproduct` (L5, `Sum`→`Coproduct` rename) — value-level acceptance.
//!
//! `docs/program/wp/either-neutral-coproduct.md`: the internal effect
//! coproduct is renamed `Sum`→`Coproduct` (type name only, `InL`/`InR`
//! kept). This file confirms `Coproduct`/`InL`/`InR` are still live and
//! drive effect composition post-rename (covered in depth by
//! `effect_composition_resp_coproduct_acceptance.rs` and the interpreter's
//! D3 peel tests — this file adds a light elaborator-level cross-check),
//! and that `Sum` is fully vacated as a type name (freed for a future
//! `Data.Functor.Sum`). `Either` is a separate, later catalog-package WP
//! (`docs/program/wp/either-catalog-package.md`) — out of scope here.

use ken_elaborator::ElabEnv;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

/// `Coproduct`/`InL`/`InR` (ex-`Sum`) are registered, live, and structurally
/// unchanged by the rename — confirmed via the inductive's own metadata
/// (`params.len() == 2`, `InL`/`InR` are its constructors, each with exactly
/// one ctor argument). Also confirms `Sum` is fully vacated as a type name.
#[test]
fn coproduct_is_the_live_two_param_effect_signature_type() {
    let env = mk_env();
    let coproduct_id = env
        .globals
        .get("Coproduct")
        .copied()
        .expect("Coproduct registered (renamed from Sum)");
    let inl_id = env.globals.get("InL").copied().expect("InL registered");
    let inr_id = env.globals.get("InR").copied().expect("InR registered");

    let ind = env
        .env
        .inductive(coproduct_id)
        .expect("Coproduct must be a real inductive");
    assert_eq!(
        ind.params.len(),
        2,
        "Coproduct a b takes exactly two params"
    );
    assert_eq!(ind.constructors.len(), 2, "InL | InR — exactly two ctors");
    assert_eq!(ind.constructors[0].id, inl_id);
    assert_eq!(ind.constructors[1].id, inr_id);
    assert_eq!(
        ind.constructors[0].args.len(),
        1,
        "InL : a -> Coproduct a b"
    );
    assert_eq!(
        ind.constructors[1].args.len(),
        1,
        "InR : b -> Coproduct a b"
    );

    // `Sum` itself is fully vacated as a type name — freed for the future
    // `Data.Functor.Sum` (the WP's stated AC).
    assert!(
        env.globals.get("Sum").is_none(),
        "`Sum` must not be registered as a type — it's freed for Data.Functor.Sum"
    );
}
