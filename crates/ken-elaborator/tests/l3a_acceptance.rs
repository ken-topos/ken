//! L3a acceptance tests — String + List combinators + `unfoldUpTo` + `sort`.
//!
//! Pins the L3a slice of `conformance/surface/collections/seed-collections.md`:
//! AC1 (String UTF-8 primitive), AC2 (List pattern-match via real `elim_List`),
//! AC3 (functor law emits a real obligation; cross-decl ref resolves), AC4 (no
//! coinduction + fuel-bounded `unfoldUpTo`), AC5-List (structural slot-id
//! equality), AC6 (verified `sort` emits the conjoined `is_sorted ∧ Perm`
//! obligation). Spec: `spec/30-surface/37-strings-collections.md`.
//!
//! The combinator / `unfoldUpTo` / `sort` views are declared here (driving the
//! recursive-view-through-SCT wiring in `elab.rs`); the prelude (`prelude.rs`)
//! supplies the types + Ω constants. `filter` is deferred — it needs Boolean
//! branching, but `Bool` is an opaque primitive (not `data Bool = True | False`
//! ), so it is not pattern-matchable, and a CBV `if` primitive would
//! double-evaluate a recursive branch — a separate change (tracked follow-on).

use ken_elaborator::{ElabEnv, NumericLitVal, ObligationKind};
use ken_interp::eval::{eval, prim_reduce, EvalStore, EvalVal};
use ken_kernel::{whnf, Context, GlobalId, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env")
}

/// Declare the L3a combinator / `unfoldUpTo` / `insert` views (no refinement
/// obligations). `sort` is declared per-test (AC6 asserts its obligation).
fn setup_combinators(env: &mut ElabEnv) {
    // `map : (a → b) → List a → List b` — functor.
    env.elaborate_decl(
        "fn map (a b : Type) (f : a → b) (xs : List a) : List b = \
         match xs { Nil |-> Nil b ; Cons h t |-> Cons b (f h) (map a b f t) }",
    )
    .expect("map elaborates");
    // `fold : (a → b → b) → b → List a → b` — foldr.
    env.elaborate_decl(
        "fn fold (a b : Type) (f : a → b → b) (z : b) (xs : List a) : b = \
         match xs { Nil |-> z ; Cons h t |-> f h (fold a b f z t) }",
    )
    .expect("fold elaborates");
    // `zip : List a → List b → List (Prod a b)`.
    env.elaborate_decl(
        "fn zip (a b : Type) (xs : List a) (ys : List b) : List (Prod a b) = \
         match xs { Nil |-> Nil (Prod a b) ; Cons h t |-> match ys { Nil |-> Nil (Prod a b) ; Cons k u |-> Cons (Prod a b) (MkProd a b h k) (zip a b t u) } }",
    )
    .expect("zip elaborates");
    // `unfoldUpTo : (s → Option (Prod a s)) → Nat → s → List a` — fuel-bounded
    // inductive unfold (the no-coinduction infinitude demo, `37 §5`).
    env.elaborate_decl(
        "fn unfoldUpTo (a s : Type) (step : s → Option (Prod a s)) (n : Nat) (seed : s) : List a = \
         match n { Zero |-> Nil a ; Suc m |-> match step seed { None |-> Nil a ; Some p |-> match p { MkProd x y |-> Cons a x (unfoldUpTo a s step m y) } } }",
    )
    .expect("unfoldUpTo elaborates");
    // `insert : (a → a → Bool) → a → List a → List a` — insertion helper.
    // `leq x h` decides whether `x` goes before `h` (ES2: `Bool` is real
    // matchable data, `data Bool = True | False`; the former `OrdResult`
    // 3-way branch workaround is retired, `30 §6`/ES2).
    env.elaborate_decl(
        "fn insert (a : Type) (leq : a → a → Bool) (x : a) (xs : List a) : List a = \
         match xs { Nil |-> Cons a x (Nil a) ; Cons h t |-> match leq x h { True |-> Cons a x (Cons a h t) ; False |-> Cons a h (insert a leq x t) } }",
    )
    .expect("insert elaborates");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — List pattern-matches via the real `elim_List` (`34 §3`, `37 §3.1`)
// ─────────────────────────────────────────────────────────────────────────────

/// `surface/collections/list-pattern-matches-via-real-elim`
#[test]
fn list_pattern_matches_via_real_elim() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl(
            "fn head (a : Type) (xs : List a) : Option a = \
             match xs { Nil |-> None a ; Cons h t |-> Some a h }",
        )
        .expect("head elaborates");
    let body = env.env.transparent_body(id).expect("head is transparent").1;

    // Peel the param lambdas (`a`, `xs`) to the match body. The match lowers
    // through the REAL `elim_List` (a `Term::Elim` on `List`), not a bespoke
    // collection protocol.
    let mut inner = &body;
    while let Term::Lam(_, b) = inner {
        inner = b;
    }
    match inner {
        Term::Elim { fam, .. } => {
            assert_eq!(
                *fam, env.globals["List"],
                "head's match must lower to elim_List"
            );
        }
        other => panic!(
            "head's match must be a Term::Elim (real elim_List); got {:?}",
            other
        ),
    }

    // ι-reduces on a concrete `Cons`: head Int (Cons Int 42 (Nil Int)) ⇝ Some Int 42.
    let (cons_id, nil_id, some_id) = (env.globals["Cons"], env.globals["Nil"], env.globals["Some"]);
    let int_id = env.globals["Int"];
    let int_t = Term::const_(int_id, vec![]);
    let v42 = mk_int_const(&mut env, 42);
    let nil_int = Term::app(
        Term::Constructor {
            id: nil_id,
            level_args: vec![],
        },
        int_t.clone(),
    );
    let cons_int_42_nil = Term::app(
        Term::app(
            Term::app(
                Term::Constructor {
                    id: cons_id,
                    level_args: vec![],
                },
                int_t.clone(),
            ),
            v42,
        ),
        nil_int,
    );
    // `head` body = Lam(a, Lam(xs, Elim)). Apply to Int + (Cons Int 42 (Nil Int)).
    let applied = whnf(
        &env.env,
        &Context::new(),
        &Term::app(Term::app(body, int_t), cons_int_42_nil),
    );
    // Should reduce to `Some Int 42` — a `Some` constructor application.
    assert_eq!(
        peel_app_head(&applied),
        Some(some_id),
        "head (Cons 42 Nil) should ι-reduce to Some; got {:?}",
        applied
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — fuel-bounded `unfoldUpTo` (no-coinduction presence half, `37 §5`)
// ─────────────────────────────────────────────────────────────────────────────

/// `surface/collections/fuel-bounded-unfold-produces-finite-prefix`
#[test]
fn fuel_bounded_unfold_produces_finite_prefix() {
    let mut env = mk_env();
    setup_combinators(&mut env);

    let unfold_id = env.globals["unfoldUpTo"];
    // SCT accepted: the recursive const upgraded to transparent (out of
    // trusted_base) — the recursion terminates by structural descent on the
    // `Nat` fuel (SCT's ↓), not by coinduction.
    assert!(
        !env.env.trusted_base().contains(&unfold_id),
        "unfoldUpTo must upgrade to transparent after SCT (not an open hole)"
    );
    assert!(
        env.env.transparent_body(unfold_id).is_some(),
        "unfoldUpTo must have a δ-unfoldable body after SCT"
    );

    // The body matches on the `Nat` fuel (a real `Term::Elim` on `Nat`), not a
    // coinductive former.
    let body = env.env.transparent_body(unfold_id).unwrap().1;
    // Peel the leading param lambdas to find the match (Term::Elim on Nat).
    let mut inner = &body;
    while let Term::Lam(_, b) = inner {
        inner = b;
    }
    assert!(
        matches!(inner, Term::Elim { .. }),
        "unfoldUpTo body must match on the fuel via Term::Elim; got {:?}",
        inner
    );
    if let Term::Elim { fam, .. } = inner {
        assert_eq!(
            *fam, env.globals["Nat"],
            "the fuel match is on Nat (the fuel)"
        );
    }

    // Produces a CONCRETE finite prefix: unfoldUpTo Nat Nat step (Suc Zero) Zero
    // with step = \s. Some (MkProd Nat Nat s (Suc s)) ⇝ [Zero] (a 1-element List).
    let (term, _ty) = env
        .elaborate_expr(
            "fuel_bounded_unfold_produces_finite_prefix",
            "unfoldUpTo Nat Nat (\\s. Some (Prod Nat Nat) (MkProd Nat Nat s (Suc s))) (Suc Zero) Zero",
        )
        .expect("unfoldUpTo application elaborates");
    let reduced = whnf(&env.env, &Context::new(), &term);
    // The result must be a `Cons` (a finite, non-empty List prefix), not stuck.
    let head = peel_app_head(&reduced);
    assert_eq!(
        head,
        Some(env.globals["Cons"]),
        "unfoldUpTo (fuel 1) must produce a Cons (finite prefix), not stay stuck; got {:?}",
        reduced
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Register an `Int` literal `n` as an opaque postulate + record its value, and
/// return the `Term::Const` that refers to it. (Mirrors the elaborator's literal
/// elaboration, for building concrete scrutinees in tests.)
fn mk_int_const(env: &mut ElabEnv, n: i128) -> Term {
    let int_id = env.globals["Int"];
    let ty = Term::const_(int_id, vec![]);
    let id = env
        .declare_postulate_raw(&format!("__lit_{}", n), ty)
        .expect("declare lit");
    env.num_values
        .insert(id, ken_elaborator::NumericLitVal::Int(n));
    Term::const_(id, vec![])
}

/// Peel `App` chains to the head: `App(App(h, _), _)` → `h`'s underlying
/// constructor/global id, if the head is a `Constructor`/`Const`.
fn peel_app_head(term: &Term) -> Option<GlobalId> {
    let mut cur = term;
    loop {
        match cur {
            Term::App(f, _) => cur = f,
            Term::Constructor { id, .. } | Term::Const { id, .. } => return Some(*id),
            _ => return None,
        }
    }
}

// keep `eval`/`EvalStore`/`EvalVal` imports used (AC1/AC5/AC6 below use them).
// ─────────────────────────────────────────────────────────────────────────────
// AC1 — `String` is a content-addressed UTF-8 primitive (not `List Char`)
// ─────────────────────────────────────────────────────────────────────────────

/// `surface/collections/string-byte-length-differs-from-char-length`
#[test]
fn string_byte_length_differs_from_char_length() {
    // ASCII: byteLength == charLength == 3.
    assert_eq!(
        prim_reduce("byte_length", &[EvalVal::Str("abc".into())]),
        EvalVal::Int(3)
    );
    assert_eq!(
        prim_reduce("char_length", &[EvalVal::Str("abc".into())]),
        EvalVal::Int(3)
    );
    // CJK U+4E16 (世): 3 UTF-8 bytes, 1 code point — NFC-independent (real now).
    assert_eq!(
        prim_reduce("byte_length", &[EvalVal::Str("世".into())]),
        EvalVal::Int(3)
    );
    assert_eq!(
        prim_reduce("char_length", &[EvalVal::Str("世".into())]),
        EvalVal::Int(1)
    );
    // They DIFFER on the multi-byte witness — AC1's headline (String ≠ List Char).
    assert_ne!(
        prim_reduce("byte_length", &[EvalVal::Str("世".into())]),
        prim_reduce("char_length", &[EvalVal::Str("世".into())]),
    );
    // Producer-grep: the ops are real prelude primitives (not hand-fed literals).
    let env = mk_env();
    assert!(env.globals.contains_key("byte_length") && env.globals.contains_key("char_length"));
    // Verdict flips: neutral (stuck) on a non-literal arg, not a silent compute.
    assert!(matches!(
        prim_reduce("byte_length", &[EvalVal::Neutral]),
        EvalVal::Neutral
    ));
}

/// `surface/collections/string-is-not-list-char-but-convertible`
#[test]
fn string_is_not_list_char_but_convertible() {
    let env = mk_env();
    let (string_id, list_id, char_id) = (
        env.globals["String"],
        env.globals["List"],
        env.globals["Char"],
    );
    // `String` (opaque primitive) ≠ `List Char` (inductive former applied to
    // `Char`) — distinct types; a `String` is not accepted where `List Char` is
    // required without an explicit conversion.
    let string_ty = Term::const_(string_id, vec![]);
    let list_char_ty = Term::app(
        Term::indformer(list_id, vec![]),
        Term::const_(char_id, vec![]),
    );
    assert_ne!(
        string_ty, list_char_ty,
        "String and List Char are distinct types"
    );

    // The two convertible views are registered (total-typed): `String → List
    // Char` and `List Char → String` (`37 §2.3`).
    let s2lc = env.globals["string_to_list_char"];
    let lc2s = env.globals["list_char_to_string"];
    let s2lc_ty = env.env.const_type(s2lc).unwrap().1;
    assert_eq!(
        s2lc_ty,
        Term::pi(string_ty.clone(), list_char_ty.clone()),
        "string_to_list_char : String → List Char (total)"
    );
    let lc2s_ty = env.env.const_type(lc2s).unwrap().1;
    assert_eq!(
        lc2s_ty,
        Term::pi(list_char_ty, string_ty),
        "list_char_to_string : List Char → String (total)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — functor law emits a real obligation; cross-decl ref resolves
// ─────────────────────────────────────────────────────────────────────────────

/// `surface/collections/functor-law-emits-obligation-cross-decl-resolves`
#[test]
fn functor_law_emits_obligation_cross_decl_resolves() {
    let mut env = mk_env();
    setup_combinators(&mut env); // declares `map` in a separate declaration

    // `map_id : map id xs ≡ xs`, stated in a declaration SEPARATE from `map`.
    // The cross-declaration lowercase reference `map` resolves via the landed
    // `L-resolver-globals` fallback (`c3a3f1d`).
    let res = env
        .elaborate_decl_v1(
            "fn map_id (a : Type) (xs : List a) : Int \
             ensures Equal (List a) (map a a (\\x. x) xs) xs = 0",
        )
        .expect("map_id elaborates (cross-decl ref `map` resolves)");
    // (b) A real `≡`-obligation is emitted (observe emission, not "it type-checks").
    assert!(
        !res.obligations.is_empty(),
        "map_id must emit a law obligation"
    );
    let obl = &res.obligations[0];
    assert!(
        matches!(obl.kind, ObligationKind::Ensures),
        "obligation kind is Ensures"
    );
    // (c) The obligation references the REAL `map` (cross-decl resolved) + the
    // `Equal` (≡) constant — not a synthetic/hand-fed obligation.
    let map_id = env.globals["map"];
    let equal_id = env.globals["Equal"];
    assert!(
        term_mentions_const(&obl.goal_closed, map_id),
        "obligation must reference the real `map` combinator"
    );
    assert!(
        term_mentions_const(&obl.goal_closed, equal_id),
        "obligation must be an ≡ (Equal) proposition"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 (List half) — structurally-equal lists share a slot (O(1) equality)
// ─────────────────────────────────────────────────────────────────────────────

/// `surface/collections/structurally-equal-collections-o1-comparable` (List half)
///
/// Asserts the content-addressed slot-id equality mechanism (the landed K3
/// heap, `41 §4`): structurally-equal compound values share one slot ⇒ O(1)
/// equality is a slot-id comparison. Uses a monomorphic `IntList` because the
/// interp does not type-erase a polymorphic constructor's type argument (`Cons
/// Int 1 …` stores `Int` as a `Neutral`, which is not `RtValue`-representable,
/// so it does not intern) — a landed interp limitation; the slot-id mechanism
/// itself is identical, and the polymorphic-`List` slot-id is a tracked
/// follow-on (interp type-erasure).
#[test]
fn structurally_equal_lists_share_slot() {
    let mut env = mk_env();
    env.elaborate_decl("data IntList = INil | ICons Int IntList")
        .expect("IntList elaborates");
    // `[1, 2]` and `[2, 1]` as elaborated core terms (monomorphic — no type arg).
    let (t12, _) = env
        .elaborate_expr(
            "structurally_equal_lists_share_slot",
            "ICons 1 (ICons 2 INil)",
        )
        .expect("[1,2] elaborates");
    let (t21, _) = env
        .elaborate_expr(
            "structurally_equal_lists_share_slot",
            "ICons 2 (ICons 1 INil)",
        )
        .expect("[2,1] elaborates");

    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, lit) in &env.num_values {
        store.num_values.insert(*id, lit_to_evalval(lit, mkdecimalpair_id));
    }
    // Two structurally-equal lists share one slot (content-addressed dedup).
    let v12a = eval(&[], &t12, &env.env, &mut store);
    let v12b = eval(&[], &t12, &env.env, &mut store);
    let (sa, sb) = (slot_of(&v12a), slot_of(&v12b));
    assert!(
        sa != 0 && sa == sb,
        "structurally-equal lists share a slot (O(1) equality): {:?} vs {:?}",
        sa,
        sb
    );
    // Discriminating: a DIFFERENT list has a DIFFERENT slot.
    let v21 = eval(&[], &t21, &env.env, &mut store);
    assert_ne!(sa, slot_of(&v21), "different lists have different slots");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — verified `sort` emits the conjoined `is_sorted ∧ Perm` obligation
// ─────────────────────────────────────────────────────────────────────────────

/// `surface/collections/sort-emits-issorted-and-perm` (soundness)
#[test]
fn sort_emits_issorted_and_perm() {
    let mut env = mk_env();
    setup_combinators(&mut env); // declares `insert` (the sort helper)
                                 // `sort : (a → a → Bool) → List a → { ys | is_sorted leq ys ∧ Perm ys xs }`.
                                 // `leq` is the buildable-now spelling of `Ord a` (the `where Ord a`
                                 // constraint + constraint resolution is L3b-gated, `37 §6`); ES2-remainder
                                 // threads it through `is_sorted` too (`is_sorted a leq ys`, real def, `§6`).
    let res = env
        .elaborate_decl_v1(
            "fn sort (a : Type) (leq : a → a → Bool) (xs : List a) : \
             { ys : List a | And (is_sorted a leq ys) (Perm a ys xs) } = \
             match xs { Nil |-> Nil a ; Cons h t |-> insert a leq h (sort a leq t) }",
        )
        .expect("sort elaborates (recursive + refinement)");
    // SCT accepted: `sort` upgraded to transparent (recursion on `t`, a sub-term
    // of `xs`), not left as an open hole.
    let sort_id = env.globals["sort"];
    assert!(
        !env.env.trusted_base().contains(&sort_id),
        "sort must upgrade to transparent after SCT"
    );

    // The conjoined refinement obligation is EMITTED (`34 §5`, `22 §2.1`).
    let obl = res
        .obligations
        .iter()
        .find(|o| matches!(o.kind, ObligationKind::Ensures))
        .expect("sort emits an Ensures (refinement) obligation");

    // AC6 (soundness): the obligation carries BOTH conjuncts — `is_sorted` AND
    // the load-bearing `Perm` (sortedness-alone is `const Nil`-vacuous; a
    // dropped `Perm` reads `proved`-by-default, the untrusted-layer omission).
    let (issorted_id, perm_id, and_id) = (
        env.globals["is_sorted"],
        env.globals["Perm"],
        env.globals["And"],
    );
    assert!(
        term_mentions_const(&obl.goal_closed, and_id),
        "obligation must be the CONJOIN (And)"
    );
    assert!(
        term_mentions_const(&obl.goal_closed, issorted_id),
        "obligation must carry the is_sorted conjunct"
    );
    assert!(
        term_mentions_const(&obl.goal_closed, perm_id),
        "obligation must carry the load-bearing Perm conjunct (not is_sorted-alone)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// helpers (Term walking, literal bridge, slot extraction)
// ─────────────────────────────────────────────────────────────────────────────

/// Does `term` mention the global constant `id` (as a `Term::Const`)? Walks the
/// sub-term tree (Pi/Lam/App/Let/Ascript/Elim/Const/Constructor/IndFormer/...).
fn term_mentions_const(term: &Term, id: GlobalId) -> bool {
    match term {
        Term::Const { id: i, .. } => *i == id,
        Term::Var(_) | Term::Type(_) | Term::Omega(_) => false,
        Term::Pi(a, b) | Term::Lam(a, b) | Term::App(a, b) => {
            term_mentions_const(a, id) || term_mentions_const(b, id)
        }
        Term::Let { ty, val, body, .. } => {
            term_mentions_const(ty, id)
                || term_mentions_const(val, id)
                || term_mentions_const(body, id)
        }
        Term::Ascript(e, t) => term_mentions_const(e, id) || term_mentions_type(t, id),
        Term::Elim {
            motive,
            methods,
            scrut,
            params,
            ..
        } => {
            term_mentions_const(motive, id)
                || methods.iter().any(|m| term_mentions_const(m, id))
                || term_mentions_const(scrut, id)
                || params.iter().any(|p| term_mentions_const(p, id))
        }
        Term::IndFormer { .. } | Term::Constructor { .. } => false,
        // Pair/Cast/Eq/Refl/Proj/QuotElim/QuotClass — recurse into their Term
        // sub-fields conservatively (the obligation tree only uses the above).
        _ => false,
    }
}

/// `term_mentions_type` — same walk for a type sub-term (Ascription annotation).
fn term_mentions_type(term: &Term, id: GlobalId) -> bool {
    term_mentions_const(term, id)
}

/// Bridge an elaborator `NumericLitVal` to an interp `EvalVal` (the literal
/// side-table the eval store consumes).
fn lit_to_evalval(lit: &NumericLitVal, mkdecimalpair_id: GlobalId) -> EvalVal {
    match lit {
        NumericLitVal::Int(n) => EvalVal::from(*n),
        NumericLitVal::Float(f) => EvalVal::Float(*f),
        NumericLitVal::Float32(f) => EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
        }
        NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
    }
}

/// Extract the content-addressed slot id from a compound `EvalVal` (Ctor/Pair/
/// Closure), or 0 (NULL_SLOT) if it has none.
fn slot_of(v: &EvalVal) -> u64 {
    match v {
        EvalVal::Ctor { slot, .. } | EvalVal::Pair { slot, .. } | EvalVal::Closure { slot, .. } => {
            *slot
        }
        _ => 0,
    }
}
