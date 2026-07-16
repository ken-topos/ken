//! Direct discriminating test for `elim_reduce`'s `EvalVal::Bool` dispatch
//! arm (`eval.rs`), added while wiring `decimalAdd`'s alignment (WP
//! decimal-char-demote-build).
//!
//! `eq_int`/`leq_int`/`not_bool`/`and_bool`/`or_bool`/`eq_float`/
//! `eq_float32` return the interpreter's native `EvalVal::Bool(bool)`
//! immediate from `prim_reduce`, but `Bool` is a real inductive (`data Bool
//! = True | False`) whose bare-literal values reduce to `EvalVal::Ctor{id:
//! true_id/false_id}` via the constructor path — a DIFFERENT representation
//! of the same logical type. `Term::Elim`'s ι-reduction (`elim_reduce`)
//! previously only dispatched on `EvalVal::Ctor`, so a `match` scrutinizing
//! a COMPUTED `Bool` silently got stuck at `Neutral` instead of picking a
//! branch. The fix added an `EvalVal::Bool(b)` arm selecting
//! `methods[0]`/`methods[1]` directly.
//!
//! Architect's hard gate (`evt_536pz1sn0nr9m`): the old failure was safely
//! STUCK; a flipped branch-index mapping in the fix would be a WRONG VALUE
//! — strictly worse than stuck. This test pins the mapping directly: it
//! must select the SAME branch whether the scrutinee is a computed `Bool`
//! (via `eq_int`) or a bare `True`/`False` literal (the pre-existing,
//! already-correct `Ctor` dispatch path) — confirming `methods[0] ≡ True` /
//! `methods[1] ≡ False` matches `data Bool = True | False`'s declared order
//! on BOTH representations, not just one.

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal};
use ken_kernel::Decl;

fn eval_view(src: &str) -> EvalVal {
    let mut env = ElabEnv::new().expect("prelude init");
    let r = env.elaborate_decl_v1(src).expect("elaborates");
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, v) in &env.num_values {
        let val = match v {
            NumericLitVal::Int(n) => EvalVal::from(*n),
            NumericLitVal::Float(f) => EvalVal::Float(*f),
            NumericLitVal::Float32(f) => EvalVal::Float32(*f),
            NumericLitVal::Decimal { coeff, exp } => {
                ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
            }
            NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
        };
        store.num_values.insert(*id, val);
    }
    match env.env.lookup(r.def_id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, &mut store),
        other => panic!("expected a checked Transparent def, got {:?}", other.map(|_| ())),
    }
}

/// surface/numbers/elim-reduce-computed-bool-true-branch (soundness) —
/// `match (eq_int a a) { True => 1 ; False => 2 }` on a COMPUTED `Bool`
/// (genuinely `eq_int 5 5`, never a bare literal) must pick the `True`
/// branch, matching `data Bool = True | False`'s declared index 0.
#[test]
fn computed_bool_true_dispatches_to_first_method() {
    let result = eval_view(
        "const t = match (eq_int 5 5) { True |-> 1 ; False |-> 2 }",
    );
    assert_eq!(result, EvalVal::Int(1), "eq_int 5 5 is True — must select methods[0]");
}

/// surface/numbers/elim-reduce-computed-bool-false-branch (soundness) —
/// the discriminating PAIR: a genuinely FALSE computed `Bool` must pick the
/// OTHER branch (methods[1]), not the same one as the True case (which
/// would net a flipped/collapsed mapping,
/// [[taint-axis-orientation-needs-distinguishing-pair]]).
#[test]
fn computed_bool_false_dispatches_to_second_method() {
    let result = eval_view(
        "const t = match (eq_int 5 6) { True |-> 1 ; False |-> 2 }",
    );
    assert_eq!(result, EvalVal::Int(2), "eq_int 5 6 is False — must select methods[1]");
}

/// surface/numbers/elim-reduce-computed-vs-literal-bool-agree (soundness) —
/// a computed `Bool` and a bare `True`/`False` literal scrutinee must
/// dispatch to the IDENTICAL branch for the "same" logical value — pins
/// that the fix's new `EvalVal::Bool` arm agrees with the pre-existing,
/// already-correct `EvalVal::Ctor{true_id/false_id}` literal path, not just
/// that it happens to produce SOME value.
#[test]
fn computed_bool_agrees_with_literal_bool_dispatch() {
    let computed_true = eval_view("const t = match (eq_int 5 5) { True |-> 1 ; False |-> 2 }");
    let literal_true = eval_view("const t = match True { True |-> 1 ; False |-> 2 }");
    assert_eq!(computed_true, literal_true, "computed True and literal True must agree");

    let computed_false = eval_view("const t = match (eq_int 5 6) { True |-> 1 ; False |-> 2 }");
    let literal_false = eval_view("const t = match False { True |-> 1 ; False |-> 2 }");
    assert_eq!(computed_false, literal_false, "computed False and literal False must agree");
}

/// surface/numbers/elim-reduce-computed-bool-via-leq-int (soundness) — the
/// exact shape `decimalAdd`'s alignment needs: `leq_int`-computed `Bool` as
/// a match scrutinee (not `eq_int`), confirming the fix isn't narrowly
/// specific to one prim symbol's `EvalVal::Bool` output.
#[test]
fn computed_bool_via_leq_int_dispatches_correctly() {
    let le = eval_view("const t = match (leq_int 3 5) { True |-> 1 ; False |-> 2 }");
    assert_eq!(le, EvalVal::Int(1), "leq_int 3 5 is True — must select methods[0]");
    let gt = eval_view("const t = match (leq_int 5 3) { True |-> 1 ; False |-> 2 }");
    assert_eq!(gt, EvalVal::Int(2), "leq_int 5 3 is False — must select methods[1]");
}
