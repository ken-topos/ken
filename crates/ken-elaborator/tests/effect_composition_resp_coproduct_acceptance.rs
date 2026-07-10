//! `effect-composition` D1 (BV2, the hinge) — `resp_coproduct` must be a REDUCING
//! `declare_def`, never a postulate: `resp_coproduct g h rg rh (InL x)` fires the
//! per-tag ι and reduces to `rg x` (not stuck), and symmetrically for `InR`.
//! A postulate here would silently break `inject_l`/`inject_r`'s coercion-free
//! typing (doc §D1.1/§D2.2) — this is the discriminating test the doc
//! demands ("not just 'it compiles'").

use ken_elaborator::ElabEnv;

/// `resp_coproduct Nat Nat idNat idNat (InL Nat Nat Zero)` must REDUCE to `Zero`
/// (the `InL` summand's own response, `rg` applied to the payload) — a
/// concrete ground normal form, not merely a well-typed stuck term.
#[test]
fn resp_coproduct_inl_reduces_to_rg_applied_to_payload() {
    let mut env = ElabEnv::new().expect("env");
    env.elaborate_decl("fn idNat (n : Nat) : Type = Nat")
        .expect("idNat elaborates");
    let main_id = env
        .elaborate_decl(
            "const probe : Type = resp_coproduct Nat Nat idNat idNat (InL Nat Nat Zero)",
        )
        .expect("probe elaborates");
    // `probe`'s BODY (a `Type`-classified term) must normalize to `Nat`
    // itself (idNat's constant body) — not remain a stuck `resp_coproduct`
    // application (which would mean the ι never fired, i.e. resp_coproduct is
    // opaque/postulate-like).
    let ken_kernel::env::Decl::Transparent { body, .. } =
        env.env.lookup(main_id).expect("probe is a real decl")
    else {
        panic!("probe must be a transparent def");
    };
    let ctx = ken_kernel::env::Context::new();
    let normal = ken_kernel::normalize(&env.env, &ctx, &body);
    let nat_id = env.globals.get("Nat").copied().expect("Nat registered");
    assert_eq!(
        normal,
        ken_kernel::Term::indformer(nat_id, vec![]),
        "resp_coproduct g h rg rh (InL x) must ι-reduce to `rg x` (here: idNat Zero ≡ Nat) — \
         got {normal:?}, meaning the ι never fired (resp_coproduct would be acting like a postulate)"
    );
}

/// Symmetric case: `resp_coproduct g h rg rh (InR y)` must reduce to `rh y` — the
/// same hinge, other tag (no wrap/reorder between summands).
#[test]
fn resp_coproduct_inr_reduces_to_rh_applied_to_payload() {
    let mut env = ElabEnv::new().expect("env");
    env.elaborate_decl("fn idNat (n : Nat) : Type = Nat")
        .expect("idNat elaborates");
    let main_id = env
        .elaborate_decl(
            "const probe : Type = resp_coproduct Nat Nat idNat idNat (InR Nat Nat Zero)",
        )
        .expect("probe elaborates");
    let ken_kernel::env::Decl::Transparent { body, .. } =
        env.env.lookup(main_id).expect("probe is a real decl")
    else {
        panic!("probe must be a transparent def");
    };
    let ctx = ken_kernel::env::Context::new();
    let normal = ken_kernel::normalize(&env.env, &ctx, &body);
    let nat_id = env.globals.get("Nat").copied().expect("Nat registered");
    assert_eq!(
        normal,
        ken_kernel::Term::indformer(nat_id, vec![]),
        "resp_coproduct g h rg rh (InR y) must ι-reduce to `rh y` — got {normal:?}"
    );
}
