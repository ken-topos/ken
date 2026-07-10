//! `Either` / `Coproduct` (L5, COEXIST rescope) — value-level acceptance.
//!
//! `docs/program/wp/either-neutral-coproduct.md`: three distinct-role
//! coproducts now exist — `Result` (fallible, untouched), `Either`
//! (user-facing value disjunction, ADDED this WP), `Coproduct` (the
//! internal effect coproduct, RENAMED from `Sum`, `InL`/`InR` kept). This
//! file is the WP's own AC: (1) construct + match a concrete `Either Int
//! String` value on each tag, discriminating (not just "it compiles"); (2)
//! confirm `Coproduct`/`InL`/`InR` still drives effect composition
//! identically post-rename (covered in depth by
//! `effect_composition_resp_coproduct_acceptance.rs` and the interpreter's
//! D3 peel tests — this file adds a light elaborator-level cross-check
//! that the renamed globals are live and distinct from `Either`).

use ken_elaborator::{ElabEnv, NumericLitVal};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

/// Elaborates `decl_src` (a `const <name> : Int = <expr>`-shaped
/// declaration), normalizes its body, and returns the `Int` value the
/// normal form's `Const` head resolves to in `env.num_values` — panics if
/// the normal form isn't a ground integer literal (e.g. still a stuck
/// application), which is exactly the failure mode this test guards.
fn normalize_int_const(env: &mut ElabEnv, name: &str, decl_src: &str) -> i128 {
    let id = env
        .elaborate_decl(decl_src)
        .unwrap_or_else(|e| panic!("{name} failed to elaborate: {e}"));
    let ken_kernel::env::Decl::Transparent { body, .. } =
        env.env.lookup(id).expect("decl must be a real decl")
    else {
        panic!("{name} must be a transparent def");
    };
    let ctx = ken_kernel::env::Context::new();
    let normal = ken_kernel::normalize(&env.env, &ctx, &body);
    let ken_kernel::Term::Const { id: lit_id, .. } = normal else {
        panic!("{name} normalized to a non-Const term (stuck?): {normal:?}");
    };
    match env.num_values.get(&lit_id) {
        Some(NumericLitVal::Int(n)) => *n,
        other => panic!("{name}'s normal form isn't a registered Int literal: {other:?}"),
    }
}

/// `Left Int String 42 : Either Int String`, matched to project the payload
/// back out — must normalize to the SAME ground value as the literal `42`
/// itself, not remain a stuck `match`.
#[test]
fn either_left_int_string_matches_and_normalizes() {
    let mut env = mk_env();
    let got = normalize_int_const(
        &mut env,
        "probe",
        "const probe : Int = \
         match (Left Int String 42) { Left x => x ; Right y => 0 }",
    );
    assert_eq!(
        got, 42,
        "Left's match arm must reduce to the concrete payload `42`, not a \
         stuck match — got {got}"
    );
}

/// Symmetric case: `Right Int String "hi" : Either Int String` must select
/// the OTHER branch — discriminating against the Left case above on the
/// same scrutinee shape (a flipped tag must flip the result).
#[test]
fn either_right_int_string_matches_and_normalizes() {
    let mut env = mk_env();
    let got = normalize_int_const(
        &mut env,
        "probe",
        "const probe : Int = \
         match (Right Int String \"hi\") { Left x => x ; Right y => 7 }",
    );
    assert_eq!(
        got, 7,
        "Right's match arm must reduce to `7`, discriminating from the Left \
         case — got {got}"
    );
}

/// `Left`/`Right` are distinct constructors of ONE declared `Either` — not
/// two unrelated globals — confirmed via the inductive's own metadata
/// (`params.len() == 2`, both `Left`/`Right` are its constructors, each
/// with exactly one ctor argument). Also confirms `Either` is genuinely
/// independent from `Coproduct` (distinct `GlobalId`s, not an alias).
#[test]
fn either_is_one_declared_two_param_coproduct_distinct_from_effect_coproduct() {
    let env = mk_env();
    let either_id = env
        .globals
        .get("Either")
        .copied()
        .expect("Either registered");
    let left_id = env.globals.get("Left").copied().expect("Left registered");
    let right_id = env
        .globals
        .get("Right")
        .copied()
        .expect("Right registered");

    let ind = env
        .env
        .inductive(either_id)
        .expect("Either must be a real inductive");
    assert_eq!(ind.params.len(), 2, "Either a b takes exactly two params");
    assert_eq!(ind.constructors.len(), 2, "Left | Right — exactly two ctors");
    assert_eq!(ind.constructors[0].id, left_id);
    assert_eq!(ind.constructors[1].id, right_id);
    assert_eq!(ind.constructors[0].args.len(), 1, "Left : a -> Either a b");
    assert_eq!(ind.constructors[1].args.len(), 1, "Right : b -> Either a b");

    // `Coproduct`/`InL`/`InR` (ex-`Sum`) are still registered, live, and a
    // DIFFERENT inductive from `Either` — the rename didn't fold the two
    // roles into one type (the whole point of COEXIST).
    let coproduct_id = env
        .globals
        .get("Coproduct")
        .copied()
        .expect("Coproduct registered (renamed from Sum)");
    let inl_id = env.globals.get("InL").copied().expect("InL registered");
    let inr_id = env.globals.get("InR").copied().expect("InR registered");
    assert_ne!(coproduct_id, either_id, "Coproduct and Either are distinct types");
    assert_ne!(inl_id, left_id, "InL (Coproduct) and Left (Either) are distinct ctors");
    assert_ne!(inr_id, right_id, "InR (Coproduct) and Right (Either) are distinct ctors");

    let coproduct_ind = env
        .env
        .inductive(coproduct_id)
        .expect("Coproduct must be a real inductive");
    assert_eq!(coproduct_ind.params.len(), 2, "Coproduct a b takes exactly two params");
    assert_eq!(coproduct_ind.constructors.len(), 2, "InL | InR — exactly two ctors");
    assert_eq!(coproduct_ind.constructors[0].id, inl_id);
    assert_eq!(coproduct_ind.constructors[1].id, inr_id);

    // `Sum` itself is fully vacated as a type name — freed for the future
    // `Data.Functor.Sum` (the WP's stated AC).
    assert!(
        env.globals.get("Sum").is_none(),
        "`Sum` must not be registered as a type — it's freed for Data.Functor.Sum"
    );
}
