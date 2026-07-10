//! ES2 acceptance tests: prelude hygiene — the `trusted_base()` shrink.
//!
//! Pins `docs/program/wp/ES2-prelude-hygiene.md`'s AC1/AC3/AC4 (+ the
//! ES2-remainder AC1/AC2 for `is_sorted`/`Perm`) against the **real**
//! `prelude.rs`/`trusted_base()` (producer-grep, not a hand-fed test): after
//! ES2, `Equal`/`And`/`Bool`/`IO`/`print_line`/`is_sorted`/`Perm` must not
//! remain `declare_postulate`d assumed axioms. `Map`/`Set` were originally
//! re-classed `declare_primitive` here (still trusted, audited, item-2) —
//! **superseded by Map-build** (`spec/50-stdlib/52-map.md`, VAL2 #8/OQ-A):
//! the audited primitive is now **retired outright**, replaced by a proved,
//! pure `Tree k v` (`catalog/packages/Data/Collections/Map.ken`) that is derived Ken —
//! `declare_inductive`/`declare_def`, never `declare_primitive`/
//! `declare_postulate` — a **net-negative** `trusted_base()` delta (AC4
//! below now pins the retirement, not the re-class).
//!
//! Spec: `spec/30-surface/37-strings-collections.md` §6 (`is_sorted`/`Perm`
//! defining shapes); `conformance/surface/taxonomy/minimality.md` (the
//! derivation table); `spec/50-stdlib/52-map.md` §1.1/§9 (Map-build's
//! supersession + AC1 net-negative TCB).

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv};
use ken_kernel::env::Decl;
use ken_kernel::{whnf, Term};

const COLLECTIONS_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const MAP_KEN: &str = include_str!("../../../catalog/packages/Data/Collections/Map.ken");

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

fn mk_env_with_map() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD).expect("transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD).expect("collections.ken.md must elaborate");
    env.elaborate_file(MAP_KEN).expect("map.ken must elaborate");
    env
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — the demoted entries are no longer `declare_postulate`d assumed axioms
// ─────────────────────────────────────────────────────────────────────────────

/// `Equal`/`And`/`IO`/`print_line` must be `Decl::Transparent` (real,
/// re-checked definitions) — never `Decl::Opaque` (the demoted-postulate
/// bloat AC1 forbids) and never absent from `trusted_base()`'s underlying
/// `Decl` classification by accident (they must resolve to a real decl).
#[test]
fn demoted_predicates_are_transparent_not_opaque() {
    let env = mk_env();
    for name in ["Equal", "And", "IO", "print_line", "is_sorted", "Perm"] {
        let id = env.globals[name];
        match env.env.lookup(id) {
            Some(Decl::Transparent { .. }) => {}
            other => panic!(
                "AC1: '{}' must be Decl::Transparent (demoted, re-checked def), \
                 got {:?}",
                name, other
            ),
        }
    }
}

/// `Bool` must be a real inductive (`data Bool = True | False`) — matchable
/// data, not an opaque primitive type.
#[test]
fn bool_is_a_real_inductive() {
    let env = mk_env();
    let bool_id = env.globals["Bool"];
    assert!(
        env.env.inductive(bool_id).is_some(),
        "AC1/AC3: Bool must be a real inductive, not an opaque primitive"
    );
    assert!(env.globals.contains_key("True"), "True ctor registered");
    assert!(env.globals.contains_key("False"), "False ctor registered");
}

/// Discriminating: none of the demoted names appear in `trusted_base()` —
/// the real accounting, not a hand-fed assertion. `Bool` is excluded via its
/// `Decl::Inductive` kind (neither `Opaque` nor `Primitive`); the others via
/// `Decl::Transparent`.
#[test]
fn demoted_predicates_absent_from_trusted_base() {
    let env = mk_env();
    let tb = env.env.trusted_base();
    for name in ["Equal", "And", "Bool", "IO", "print_line", "is_sorted", "Perm"] {
        let id = env.globals[name];
        assert!(
            !tb.contains(&id),
            "AC1: '{}' must not remain in trusted_base() after ES2",
            name
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ES2-remainder AC1/AC2 — is_sorted/Perm are real, unfoldable definitions
// ─────────────────────────────────────────────────────────────────────────────

/// **Necessary but NOT sufficient** on its own (QA's 3rd-occurrence
/// finding, `dec_3xzdpjm4ecwps`): `elaborate_decl_v1` succeeding here does
/// NOT by itself prove `is_sorted`/`Perm` actually unfold — checked out
/// against `e5ffbf2` (is_sorted/Perm still postulates), this exact
/// assertion passes identically, because emitting the `Ensures` obligation
/// never required the predicates to be *defined*, only *declared* (arity +
/// sort). The real discriminant is
/// `issorted_and_perm_applications_reduce_past_their_own_head` below —
/// this test stays only as the "the surface still parses/type-checks at
/// all" smoke check.
#[test]
fn sort_refinement_unfolds_issorted_and_perm() {
    let mut env = mk_env();
    env.elaborate_decl(
        "fn insert (a : Type) (leq : a -> a -> Bool) (x : a) (xs : List a) : List a = \
         match xs { Nil => Cons a x (Nil a) ; \
           Cons h t => match leq x h { True => Cons a x (Cons a h t) ; \
                                        False => Cons a h (insert a leq x t) } }",
    )
    .expect("insert elaborates");
    env.elaborate_decl_v1(
        "fn sort (a : Type) (leq : a -> a -> Bool) (xs : List a) : \
         { ys : List a | And (is_sorted a leq ys) (Perm a ys xs) } = \
         match xs { Nil => Nil a ; Cons h t => insert a leq h (sort a leq t) }",
    )
    .expect("smoke check: sort refinement must at least type-check");
}

/// **The real AC2 discriminant.** Build the exact `is_sorted`/`Perm`
/// applications `sort`'s own obligation contains (same registered
/// `GlobalId`s, a concrete closed witness: `a = Bool`, `xs = ys = Nil`,
/// `leq = \_ _. True`) and `whnf` them. A genuine definition unfolds PAST
/// its own head (δ on the `Const`, then ι on the concrete `Nil` scrutinee);
/// a postulate is permanently stuck at `Const(isSorted_id)`/`Const(perm_id)`
/// applied to its arguments — `whnf` cannot make progress on an opaque
/// constant. Verified to actually discriminate: checked out against
/// `e5ffbf2` (pre-ES2-remainder, is_sorted/Perm still postulates) this
/// assertion FAILS there (stuck at `Const`, per QA's now-3×-validated
/// checkout-and-rerun technique) and PASSES on this branch.
#[test]
fn issorted_and_perm_applications_reduce_past_their_own_head() {
    use ken_kernel::{Context, GlobalId};

    fn peel_app_head(t: &Term) -> &Term {
        let mut cur = t;
        while let Term::App(f, _) = cur {
            cur = f;
        }
        cur
    }
    fn is_stuck_at(t: &Term, id: GlobalId) -> bool {
        matches!(peel_app_head(t), Term::Const { id: i, .. } if *i == id)
    }

    let env = mk_env();
    let issorted_id = env.globals["is_sorted"];
    let perm_id = env.globals["Perm"];
    let bool_id = env.globals["Bool"];
    let true_id = env.globals["True"];
    let nil_id = env.globals["Nil"];

    let bool_t = Term::indformer(bool_id, vec![]);
    let nil_bool =
        Term::app(Term::Constructor { id: nil_id, level_args: vec![] }, bool_t.clone());
    let true_ctor = Term::Constructor { id: true_id, level_args: vec![] };
    // `leq := \_ _. True` — a trivial but well-typed `Bool -> Bool -> Bool`.
    let leq = Term::lam(bool_t.clone(), Term::lam(bool_t.clone(), true_ctor));

    let issorted_app = Term::app(
        Term::app(Term::app(Term::const_(issorted_id, vec![]), bool_t.clone()), leq),
        nil_bool.clone(),
    );
    let perm_app = Term::app(
        Term::app(Term::app(Term::const_(perm_id, vec![]), bool_t.clone()), nil_bool.clone()),
        nil_bool,
    );

    let ctx = Context::new();
    let issorted_reduced = whnf(&env.env, &ctx, &issorted_app);
    let perm_reduced = whnf(&env.env, &ctx, &perm_app);

    assert!(
        !is_stuck_at(&issorted_reduced, issorted_id),
        "AC2: `is_sorted ...` must whnf past its own Const head (a real, \
         unfoldable definition); stuck at Const(is_sorted) means it's still \
         an opaque postulate. Reduced to: {:?}",
        issorted_reduced
    );
    assert!(
        matches!(perm_reduced, Term::Trunc(_)),
        "AC2: `Perm ...` must whnf to Term::Trunc(Perm_rel ...) (a real \
         definition unfolding to the truncation), not stay stuck at \
         Const(perm_id). Reduced to: {:?}",
        perm_reduced
    );
}

/// `Perm`'s underlying relation (`Perm_rel`) must be `Type`-level and
/// truncated into `Ω` — never a bare proof-relevant inductive declared
/// directly at `Ω` (the `16 §1.3` relevance leak `evt_3cn9v6em54yej` rules
/// out). Structural check: `Perm`'s body is `Term::Trunc(...)`, not a raw
/// `Term::IndFormer`/`Term::Const` application.
#[test]
fn perm_body_is_a_truncation() {
    let env = mk_env();
    let perm_id = env.globals["Perm"];
    let (_, body) = env.env.transparent_body(perm_id).expect("Perm is transparent");
    // Peel the 3 lambdas (a, xs, ys) to the truncation.
    let mut inner = &body;
    while let Term::Lam(_, b) = inner {
        inner = b;
    }
    assert!(
        matches!(inner, Term::Trunc(_)),
        "Perm's body must be a Term::Trunc(Perm_rel a xs ys), got {:?}",
        inner
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — Bool is matchable data
// ─────────────────────────────────────────────────────────────────────────────

/// Discriminating: a `match` on a comparison-primitive result elaborates —
/// impossible against the former opaque `Bool` primitive.
#[test]
fn match_on_comparison_result_elaborates() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl(
            "fn isZero (a : Int) (b : Int) : Int = \
             match eq_int a b { True => 1 ; False => 0 }",
        )
        .expect("AC3: match on eq_int's Bool result must elaborate");
    assert!(env.env.const_type(id).is_some(), "isZero registered with a type");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — Map/Set are RETIRED (net-negative TCB), superseded by the proved,
// derived `Tree k v` (`52-map.md` AC1/AC5) — flipped from the original
// "stays audited" assertion this WP superseded.
// ─────────────────────────────────────────────────────────────────────────────

/// `Map`/`Set` no longer exist as prelude-primitive globals at all — the
/// opaque `declare_primitive` entries are GONE (`52 §9` AC1(b)/AC5), not
/// merely re-classed. This is the net-negative half: two `trusted_base()`
/// entries removed, verified structurally (absence from `env.globals`), not
/// asserted from the spec.
#[test]
fn map_set_retired_not_prelude_primitives() {
    let env = mk_env();
    assert!(
        !env.globals.contains_key("Map"),
        "AC1/AC5: 'Map' must no longer exist as a prelude-primitive global — retired by Map-build"
    );
    assert!(
        !env.globals.contains_key("Set"),
        "AC1/AC5: 'Set' must no longer exist as a prelude-primitive global — retired by Map-build"
    );
}

/// The replacement — `Tree k v` + its ops (`catalog/packages/Data/Collections/Map.ken`)
/// — is derived, kernel-rechecked Ken: `Tree` is `Decl::Inductive`
/// (`declare_inductive`), every op is `Decl::Transparent` (`declare_def`).
/// Discriminating pair with the retirement test above: a build that adds the
/// proved map WITHOUT retiring the primitive (two `Map`s) passes this test
/// but fails the retirement test; a build that retires the primitive but
/// ships the replacement as a NEW `declare_primitive`/`declare_postulate`
/// (re-growing `trusted_base()`) fails THIS test. Both halves of AC1(b)'s
/// "net delta is a shrink by exactly two, zero new" must hold together.
#[test]
fn map_replacement_is_derived_not_primitive() {
    let env = mk_env_with_map();
    let tree_id = env.globals["Tree"];
    assert!(
        matches!(env.env.lookup(tree_id), Some(Decl::Inductive { .. })),
        "AC1(b): the replacement carrier 'Tree' must be Decl::Inductive (declare_inductive), never a primitive"
    );
    for name in ["insert", "lookup", "member", "to_list", "from_list", "set_insert", "set_member", "set_to_list"] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "AC1(b): '{name}' must be Decl::Transparent (declare_def), never declare_primitive/declare_postulate"
        );
        let delta = trusted_base_delta(&env.env, id);
        assert!(delta.is_empty(), "AC1(b): '{name}' must add ZERO new trusted_base() entries, got {delta:?}");
    }
}
