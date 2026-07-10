//! Source-level regressions for `KM-sigma-eq-pair-refl`.

use ken_elaborator::ElabEnv;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

const PAIR_LENS_SOURCE: &str = r#"
fn setFstPairBoolBool (s : Pair Bool Bool) (b : Bool) : Pair Bool Bool =
  mk_pair Bool Bool b (pair_snd Bool Bool s)

fn pair_refl_fn (s : Pair Bool Bool) : Equal (Pair Bool Bool) s s =
  Refl

fn fstLensSetGet (s : Pair Bool Bool)
  : Equal (Pair Bool Bool)
      (setFstPairBoolBool s (pair_fst Bool Bool s))
      s =
  Refl

fn fstLensSetSet (s : Pair Bool Bool) (b : Bool) (c : Bool)
  : Equal (Pair Bool Bool)
      (setFstPairBoolBool (setFstPairBoolBool s b) c)
      (setFstPairBoolBool s c) =
  Refl
"#;

#[test]
fn pair_refl_and_full_pair_lens_laws_elaborate() {
    let mut env = mk_env();
    env.elaborate_file(PAIR_LENS_SOURCE)
        .expect("full Pair Bool Bool equality laws must elaborate");
}

#[test]
fn wrong_endpoint_pair_lens_law_still_rejects() {
    let mut env = mk_env();
    env.elaborate_file(PAIR_LENS_SOURCE)
        .expect("base pair lens source elaborates");

    let bad = env.elaborate_decl(
        "fn badSetGet (s : Pair Bool Bool) \
           : Equal (Pair Bool Bool) (setFstPairBoolBool s True) s = \
           Refl",
    );
    assert!(
        bad.is_err(),
        "Refl must not prove set-get with an arbitrary True endpoint"
    );
}

#[test]
fn componentwise_proof_is_not_unrelated_full_pair_equality() {
    let mut env = mk_env();
    env.elaborate_file(PAIR_LENS_SOURCE)
        .expect("base pair lens source elaborates");

    let bad = env.elaborate_decl(
        "fn badComponentAsFull \
           (s : Pair Bool Bool) \
           (t : Pair Bool Bool) \
           (p : And \
             (Equal Bool (pair_fst Bool Bool s) (pair_fst Bool Bool s)) \
             (Equal Bool (pair_snd Bool Bool s) (pair_snd Bool Bool s))) \
           : Equal (Pair Bool Bool) s t = \
           p",
    );
    assert!(
        bad.is_err(),
        "an arbitrary component proof must not be accepted as unrelated full equality"
    );
}
