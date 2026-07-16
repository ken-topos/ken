//! RTP1 pinned perf regression (`docs/program/wp/RT-perf-sharing.md`, AC5).
//!
//! Root cause (D1, instrumented `elim_reduce` call-counts, Architect-
//! confirmed): the ι-reduction's "apply IH values for recursive positions"
//! loop unconditionally computed a full recursive `elim_reduce` walk for
//! EVERY recursive constructor position, even when the selected method's
//! body never references that IH binder (`build_ctor_buckets` always wraps
//! one `Lam` per recursive position — `ColKind::Ih` columns are per-
//! constructor, not per-use). An ordinary self-recursive `view` compiles its
//! own recursion via `Term::Const` δ-unfold, not the IH binder, so the eager
//! IH was a redundant walk whose result `apply`'s catch-all silently
//! discarded — a 2x-per-level baseline multiplier that compounded with any
//! additional explicit self-reference (confirmed 2.00x/+depth for zero-
//! reference-multiplicity cases, ruling out "no substitution sharing" as the
//! cause: nothing was being shared, the walk was simply always redone).
//!
//! Fix (B'): a static free-variable check on the UNEVALUATED method term
//! decides, per recursive position, whether the IH binder is actually
//! referenced; an unreferenced IH is skipped entirely (dead-code
//! elimination, not laziness/memoisation — a referenced IH is still computed
//! exactly once, as before).
//!
//! This test pins the previously-exponential shapes at depths that were
//! measured multi-second/timeout pre-fix and must now complete in
//! well-under a second, guarding against the blowup silently returning.

use ken_elaborator::ElabEnv;
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::Term;
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_secs(2);

fn nat_lit(n: u32) -> String {
    let mut s = "Zero".to_string();
    for _ in 0..n {
        s = format!("Suc ({s})");
    }
    s
}

fn nat_depth(v: &EvalVal, suc_id: ken_kernel::GlobalId) -> u32 {
    let mut d = 0;
    let mut cur = v.clone();
    loop {
        match cur {
            EvalVal::Ctor { id, ref args, .. } if id == suc_id => {
                d += 1;
                cur = args[0].clone();
            }
            _ => break,
        }
    }
    d
}

fn run(src: &str) -> (Duration, EvalVal, ElabEnv) {
    use ken_elaborator::NumericLitVal;
    let mut env = ElabEnv::new().expect("init");
    let ids = env.elaborate_file(src).expect("elaborate");
    let main_id = *ids.last().unwrap();
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, lit) in &env.num_values {
        let val = match lit {
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
    store.list_char_ids = Some(ListCharIds {
        nil_id: env.prelude_env.nil_id,
        cons_id: env.prelude_env.cons_id,
    });
    let t0 = Instant::now();
    let v = eval(&[], &Term::const_(main_id, vec![]), &env.env, &mut store);
    (t0.elapsed(), v, env)
}

/// AC5 (value-depth shape) — a single self-recursive `view` with ZERO
/// reference-multiplicity was already 2.00x/+depth exponential pre-fix
/// (`single(20)` measured 19.5s). Pin depth 40, well past the pre-fix
/// timeout point, and assert both the value AND the time budget.
#[test]
fn single_self_reference_stays_linear_at_depth_40() {
    let src = format!(
        "fn single (n : Nat) : Nat = match n {{ Zero |-> Zero ; Suc m |-> Suc (single m) }}\nconst main : Nat = single ({})\n",
        nat_lit(40)
    );
    let (dt, v, env) = run(&src);
    let suc_id = *env.globals.get("Suc").unwrap();
    assert_eq!(nat_depth(&v, suc_id), 40, "single(n) must equal n exactly — value-preservation");
    assert!(
        dt < BUDGET,
        "single(40) took {dt:?}, budget {BUDGET:?} — the eager-IH exponential blowup may have returned"
    );
}

/// AC5 (env-size shape) — the exact VAL2 finding #6 repro (`OrdResult`/
/// `list_append`/`concat` prelude + `gcd.ken`'s own helpers + the
/// `natToDecimal` digit machinery, `natGcd(12,8)` as the argument). Pre-fix:
/// >60s, never completed. Pinned here at the same generous 2s budget.
#[test]
fn natgcd_plus_nattodecimal_stays_fast() {
    let src = r#"
data OrdResult = Lt | Eq | Gt

fn list_append (a : Type) (xs : List a) (ys : List a) : List a =
  match xs { Nil |-> ys ; Cons x xs2 |-> Cons a x (list_append a xs2 ys) }

fn concat (a : String) (b : String) : String =
  list_char_to_string (list_append Char (string_to_list_char a) (string_to_list_char b))

fn natAdd (a : Nat) (b : Nat) : Nat =
  match a { Zero |-> b ; Suc m |-> Suc (natAdd m b) }

fn natSub (a : Nat) (b : Nat) : Nat =
  match b {
    Zero  |-> a ;
    Suc n |-> match a {
               Zero  |-> Zero ;
               Suc m |-> natSub m n
             }
  }

fn natCmpZero (b : Nat) : OrdResult =
  match b { Zero |-> Eq ; Suc n |-> Lt }

fn natCmp (a : Nat) (b : Nat) : OrdResult =
  match a {
    Zero  |-> natCmpZero b ;
    Suc m |-> match b { Zero |-> Gt ; Suc n |-> natCmp m n }
  }

fn natToInt (n : Nat) : Int =
  match n { Zero |-> (0 : Int) ; Suc m |-> (1 : Int) + natToInt m }

fn natGcdFueled (fuel : Nat) (a : Nat) (b : Nat) : Nat =
  match fuel {
    Zero  |-> a ;
    Suc f |-> match natCmp a b {
               Eq |-> a ;
               Gt |-> natGcdFueled f (natSub a b) b ;
               Lt |-> natGcdFueled f a (natSub b a)
             }
  }

fn natGcd (a : Nat) (b : Nat) : Nat =
  let fuel : Nat = natAdd a b in
  natGcdFueled fuel a b

const one    : Nat = Suc Zero
const two    : Nat = Suc one
const three  : Nat = Suc two
const four   : Nat = Suc three
const five   : Nat = Suc four
const six    : Nat = Suc five
const seven  : Nat = Suc six
const eight  : Nat = Suc seven
const nine   : Nat = Suc eight
const ten    : Nat = Suc nine
const eleven : Nat = Suc ten
const twelve : Nat = Suc eleven

fn sub10 (n : Nat) : Option Nat =
  match n { Zero |-> None Nat ; Suc n1 |->
  match n1 { Zero |-> None Nat ; Suc n2 |->
  match n2 { Zero |-> None Nat ; Suc n3 |->
  match n3 { Zero |-> None Nat ; Suc n4 |->
  match n4 { Zero |-> None Nat ; Suc n5 |->
  match n5 { Zero |-> None Nat ; Suc n6 |->
  match n6 { Zero |-> None Nat ; Suc n7 |->
  match n7 { Zero |-> None Nat ; Suc n8 |->
  match n8 { Zero |-> None Nat ; Suc n9 |->
  match n9 { Zero |-> None Nat ; Suc n10 |-> Some Nat n10 } } } } } } } } } }

fn mod10Fueled (fuel : Nat) (n : Nat) : Nat =
  match fuel {
    Zero |-> n ;
    Suc f |-> match sub10 n { None |-> n ; Some n2 |-> mod10Fueled f n2 }
  }

fn mod10 (n : Nat) : Nat = mod10Fueled n n

fn div10Fueled (fuel : Nat) (n : Nat) (q : Nat) : Nat =
  match fuel {
    Zero |-> q ;
    Suc f |-> match sub10 n { None |-> q ; Some n2 |-> div10Fueled f n2 (Suc q) }
  }

fn div10 (n : Nat) : Nat = div10Fueled n n Zero

fn digitChar (d : Nat) : String =
  list_char_to_string (Cons Char ((48 : Int) + natToInt d) (Nil Char))

fn natToDecimalFueled (fuel : Nat) (n : Nat) : String =
  match fuel {
    Zero |-> digitChar n ;
    Suc f |-> match div10 n {
      Zero  |-> digitChar (mod10 n) ;
      Suc q |-> concat (natToDecimalFueled f (Suc q)) (digitChar (mod10 n))
    }
  }

fn natToDecimal (n : Nat) : String =
  match n { Zero |-> "0" ; Suc m |-> natToDecimalFueled (Suc m) (Suc m) }

const main : String = natToDecimal (natGcd twelve eight)
"#;
    let (dt, v, _env) = run(src);
    assert_eq!(v, EvalVal::Str("4".to_string()), "natToDecimal(natGcd 12 8) must be \"4\"");
    assert!(
        dt < BUDGET,
        "natGcd+natToDecimal repro took {dt:?}, budget {BUDGET:?} — the eager-IH blowup may have returned"
    );
}
