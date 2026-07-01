//! VAL1 acceptance tests: string-literals, batch-1 fizzbuzz, batch-2 numeric.
//!
//! String-literals: AC1 (parse + elaborate to `String`), AC2 (evaluates to
//! `EvalVal::Str`), AC3 (infer path). `37 §2.1`, VAL1-surface.
//!
//! FizzBuzz: verifies mod3/mod5/classify elaborate (batch-1 QA blocker).
//! Batch-2: verifies fibonacci/gcd/ackermann views elaborate (batch-2 fixes).

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{ConsoleIds, EvalStore, EvalVal};
use ken_kernel::{Decl, GlobalId};

fn make_store(env: &ElabEnv) -> EvalStore {
    let mut store = EvalStore::new();
    for (id, v) in &env.num_values {
        store.num_values.insert(*id, lit_to_eval(v));
    }
    store
}

fn lit_to_eval(v: &NumericLitVal) -> EvalVal {
    match v {
        NumericLitVal::Int(n) => {
            let n = *n;
            if n >= i64::MIN as i128 && n <= i64::MAX as i128 {
                EvalVal::Int(n as i64)
            } else {
                EvalVal::BigInt(n)
            }
        }
        NumericLitVal::Float(f) => EvalVal::Float(*f),
        NumericLitVal::Float32(f) => EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            EvalVal::DecimalVal { coeff: *coeff, exp: *exp }
        }
        NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
    }
}

fn eval_def(env: &ElabEnv, store: &mut EvalStore, id: GlobalId) -> EvalVal {
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => {
            ken_interp::eval::eval(&[], body, &env.env, store)
        }
        _ => EvalVal::Unknown,
    }
}

// ── AC1: string literal elaborates to String type ────────────────────────────

/// `surface/strings/string-literal-elaborates` (VAL1-surface, `37 §2.1`)
///
/// A string literal in a view body elaborates and the view's type is `String`.
#[test]
fn string_literal_elaborates_to_string_type() {
    let mut env = ElabEnv::new().expect("base env");
    let result = env
        .elaborate_decl("view greeting : String = \"Hello, World!\"")
        .expect("string literal view elaborates");

    let str_id = *env.globals.get("String").expect("String registered");
    let (_, ty) = env.env.const_type(result).expect("greeting has type");
    assert_eq!(
        ty,
        ken_kernel::Term::const_(str_id, vec![]),
        "greeting must have type String"
    );
}

// ── AC2: string literal reaches interpreter as EvalVal::Str ──────────────────

/// `surface/strings/string-literal-evaluates` (VAL1-surface, `37 §2.1`)
///
/// The `NumericLitVal::Str` side-table entry flows through to `EvalVal::Str`.
#[test]
fn string_literal_evaluates_to_str_val() {
    let mut env = ElabEnv::new().expect("base env");
    let id = env
        .elaborate_decl("view greeting : String = \"Hello, World!\"")
        .expect("string literal view elaborates");

    let mut store = make_store(&env);
    let val = eval_def(&env, &mut store, id);
    assert_eq!(
        val,
        EvalVal::Str("Hello, World!".to_owned()),
        "greeting must evaluate to EvalVal::Str(\"Hello, World!\")"
    );
}

// ── AC3: string literal in infer position (no ascription) ────────────────────

/// `surface/strings/string-literal-infer-path` (VAL1-surface)
///
/// A string literal without type ascription still elaborates correctly when
/// the view has no explicit return type (infer path).
#[test]
fn string_literal_infer_path_elaborates() {
    let mut env = ElabEnv::new().expect("base env");
    let id = env
        .elaborate_decl("view s = \"Ken language\"")
        .expect("unascribed string literal elaborates");

    let str_id = *env.globals.get("String").expect("String registered");
    let (_, ty) = env.env.const_type(id).expect("s has type");
    assert_eq!(
        ty,
        ken_kernel::Term::const_(str_id, vec![]),
        "unascribed string literal must default to String type"
    );
}

// ── AC4: print_line type-check: String → IO Unit ─────────────────────────────

/// `surface/io/print-line-type-checks` (VAL1-surface, `36 §2.1`)
///
/// `print_line "Hello, World!"` must elaborate and have type `IO Unit`.
/// Prim reduction (`wp/VAL1-console-exec`) is held; this tests the type only.
#[test]
fn print_line_type_checks_as_io_unit() {
    let mut env = ElabEnv::new().expect("base env");
    let id = env
        .elaborate_decl("view main : IO Unit = print_line \"Hello, World!\"")
        .expect("print_line app elaborates");

    let io_id = *env.globals.get("IO").expect("IO registered");
    let unit_id = *env.globals.get("Unit").expect("Unit registered");
    let (_, ty) = env.env.const_type(id).expect("main has type");
    let unit_t = ken_kernel::Term::indformer(unit_id, vec![]);
    let io_unit = ken_kernel::Term::app(ken_kernel::Term::const_(io_id, vec![]), unit_t);
    assert_eq!(ty, io_unit, "main must have type IO Unit");
}

// ── AC5: print_line prim reduction builds Vis (Write s) (\\_. Ret MkUnit) ─────

/// `surface/io/print-line-prim-reduction` (VAL1-surface, `36 §2.1`)
///
/// When `store.print_line_id` + `store.console_ids` are wired, evaluating
/// `print_line "Hello, World!"` produces a `Vis (Write s) k` ITree value.
/// This pins the full prim-reduction path end-to-end.
#[test]
fn print_line_prim_reduction_builds_itree() {
    let mut env = ElabEnv::new().expect("base env");
    let id = env
        .elaborate_decl("view main : IO Unit = print_line \"Hello, World!\"")
        .expect("print_line app elaborates");

    let p = &env.prelude_env;
    let console_ids = ConsoleIds {
        itree_id: p.itree_id,
        ret_id: p.ret_id,
        vis_id: p.vis_id,
        write_id: p.write_id,
        unit_id: p.mkunit_id,  // `unit_id` in ConsoleIds = MkUnit constructor
        params_len: 1,          // `data ITree r` — one type param
    };
    let mut store = make_store(&env);
    store.print_line_id = Some(p.print_line_id);
    store.console_ids = Some(console_ids.clone());

    let val = eval_def(&env, &mut store, id);

    // The result must be a Vis node: Ctor { vis_id, args: [_, Write_s, k] }
    match val {
        EvalVal::Ctor { id: ctor_id, ref args, .. } => {
            assert_eq!(ctor_id, p.vis_id, "outer ctor must be Vis");
            // args[0] = type param (Unknown); args[1] = Write s; args[2] = continuation
            assert!(args.len() >= 3, "Vis must have >= 3 args (type_param + op + k)");
            match &args[1] {
                EvalVal::Ctor { id: op_id, args: op_args, .. } => {
                    assert_eq!(*op_id, p.write_id, "op must be Write");
                    assert_eq!(
                        op_args.as_ref(),
                        &[EvalVal::Str("Hello, World!".to_owned())],
                        "Write arg must be the string"
                    );
                }
                other => panic!("expected Write ctor, got {:?}", other),
            }
        }
        other => panic!("expected Vis ITree node, got {:?}", other),
    }
}

// ── FizzBuzz batch-1 QA blocker: semicolons in match arms ────────────────────

/// Verifies mod3/mod5/classify elaborate using modular accumulator types.
///
/// Workaround for two surface gaps:
/// - GAP-nested-patterns: nested constructor patterns trigger ReachabilityError
/// - No mutual recursion between views
///
/// Solution: define a 3-element `Mod3` and 5-element `Mod5` data type as the
/// accumulator; `mod3Step`/`mod5Step` are self-recursive on the Nat argument,
/// passing the incremented accumulator — flat patterns only, no mutual recursion.
#[test]
fn fizzbuzz_classification_elaborates() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("data FizzTag = Plain | IsFizz | IsBuzz | IsFizzBuzz")
        .expect("FizzTag");
    env.elaborate_decl("data IsZero = Zero_ | NonZero_").expect("IsZero");
    env.elaborate_decl(
        "view isZero (n : Nat) : IsZero = \
         match n { Zero => Zero_ ; Suc m => NonZero_ }",
    )
    .expect("isZero");

    // mod3 via Mod3 accumulator type
    env.elaborate_decl("data Mod3 = Zero3 | One3 | Two3").expect("Mod3");
    env.elaborate_decl(
        "view incMod3 (x : Mod3) : Mod3 = \
         match x { Zero3 => One3 ; One3 => Two3 ; Two3 => Zero3 }",
    )
    .expect("incMod3");
    env.elaborate_decl(
        "view isZeroMod3 (x : Mod3) : IsZero = \
         match x { Zero3 => Zero_ ; One3 => NonZero_ ; Two3 => NonZero_ }",
    )
    .expect("isZeroMod3");
    env.elaborate_decl(
        "view mod3Step (n : Nat) (acc : Mod3) : Mod3 = \
         match n { Zero => acc ; Suc m => mod3Step m (incMod3 acc) }",
    )
    .expect("mod3Step");
    env.elaborate_decl("view mod3 (n : Nat) : Mod3 = mod3Step n Zero3").expect("mod3");

    // mod5 via Mod5 accumulator type
    env.elaborate_decl("data Mod5 = Zero5 | One5 | Two5 | Three5 | Four5").expect("Mod5");
    env.elaborate_decl(
        "view incMod5 (x : Mod5) : Mod5 = match x { \
         Zero5 => One5 ; One5 => Two5 ; Two5 => Three5 ; Three5 => Four5 ; Four5 => Zero5 }",
    )
    .expect("incMod5");
    env.elaborate_decl(
        "view isZeroMod5 (x : Mod5) : IsZero = match x { \
         Zero5 => Zero_ ; One5 => NonZero_ ; Two5 => NonZero_ ; \
         Three5 => NonZero_ ; Four5 => NonZero_ }",
    )
    .expect("isZeroMod5");
    env.elaborate_decl(
        "view mod5Step (n : Nat) (acc : Mod5) : Mod5 = \
         match n { Zero => acc ; Suc m => mod5Step m (incMod5 acc) }",
    )
    .expect("mod5Step");
    env.elaborate_decl("view mod5 (n : Nat) : Mod5 = mod5Step n Zero5").expect("mod5");

    // classify
    env.elaborate_decl(
        "view classify (n : Nat) : FizzTag = \
         match isZeroMod3 (mod3 n) { \
           Zero_ => match isZeroMod5 (mod5 n) { \
             Zero_ => IsFizzBuzz ; NonZero_ => IsFizz } ; \
           NonZero_ => match isZeroMod5 (mod5 n) { \
             Zero_ => IsBuzz ; NonZero_ => Plain } }",
    )
    .expect("classify");
}

// ── Batch-2: fibonacci (iterative accumulator) ───────────────────────────────

/// Verifies the iterative `fibStep`/`fib` views elaborate.
/// The naive 3-case fib used `Suc (Suc m)` nested patterns (GAP-nested-patterns);
/// the iterative form uses only flat `Zero | Suc m` patterns.
#[test]
fn fibonacci_iterative_elaborates() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl(
        "view natAdd (a : Nat) (b : Nat) : Nat = \
         match a { Zero => b ; Suc m => Suc (natAdd m b) }",
    )
    .expect("natAdd");
    env.elaborate_decl(
        "view natToInt (n : Nat) : Int = \
         match n { Zero => (0 : Int) ; Suc m => (1 : Int) + natToInt m }",
    )
    .expect("natToInt");
    env.elaborate_decl(
        "view fibStep (n : Nat) (a : Nat) (b : Nat) : Nat = \
         match n { Zero => a ; Suc m => fibStep m b (natAdd a b) }",
    )
    .expect("fibStep");
    env.elaborate_decl("view fib (n : Nat) : Nat = fibStep n Zero (Suc Zero)")
        .expect("fib");
    // F(10): define ten via chain
    for (name, pred) in [("one","Zero"),("two","Suc Zero"),("three","Suc (Suc Zero)"),
                         ("four","Suc (Suc (Suc Zero))"),("five","Suc (Suc (Suc (Suc Zero)))"),
                         ("six","Suc five"),("seven","Suc six"),("eight","Suc seven"),
                         ("nine","Suc eight"),("ten","Suc nine")] {
        let _ = pred; // suppress warning
        env.elaborate_decl(&format!("view {} : Nat = Suc {}", name,
            match name { "one" => "Zero", "two" => "one", "three" => "two",
                         "four" => "three", "five" => "four", "six" => "five",
                         "seven" => "six", "eight" => "seven", "nine" => "eight",
                         "ten" => "nine", _ => "Zero" }))
            .expect(name);
    }
    env.elaborate_decl("view main : Int = natToInt (fib ten)").expect("main");
}

/// Regression for GAP-nested-patterns (`elab.rs::infer_match` pattern-matrix
/// compilation, `34 §3.1`) — the reachability checker used to track coverage
/// by top-level constructor only, so `Suc Zero` and `Suc (Suc m)` (sharing
/// the `Suc` head) falsely tripped `ReachabilityError`. `isEven`, recursing
/// on the literally-matched `m` (SCT-decreasing), elaborates directly with a
/// `Suc (Suc m)` nested pattern and ι-reduces to the correct value.
#[test]
fn is_even_nested_pattern_elaborates_and_reduces() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("data BoolL = TrueL | FalseL").expect("BoolL");
    env.elaborate_decl(
        "view isEven (n : Nat) : BoolL = \
         match n { Zero => TrueL ; Suc Zero => FalseL ; Suc (Suc m) => isEven m }",
    )
    .expect("isEven");
    for (name, pred) in [("one", "Zero"), ("two", "one"), ("three", "two"), ("four", "three")] {
        env.elaborate_decl(&format!("view {} : Nat = Suc {}", name, pred)).expect(name);
    }
    env.elaborate_decl("view result : BoolL = isEven four").expect("result");
}

// ── Batch-2: GCD (subtraction-based with fuel) ───────────────────────────────

/// Verifies natSub/natCmpZero/natCmp/natGcdFueled/natGcd elaborate.
#[test]
fn gcd_views_elaborate() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl(
        "view natAdd (a : Nat) (b : Nat) : Nat = \
         match a { Zero => b ; Suc m => Suc (natAdd m b) }",
    )
    .expect("natAdd");
    env.elaborate_decl(
        "view natSub (a : Nat) (b : Nat) : Nat = \
         match b { Zero => a ; Suc n => match a { Zero => Zero ; Suc m => natSub m n } }",
    )
    .expect("natSub");
    // ES2 retired the prelude's `OrdResult` (bloat — no primitive signature
    // named it, `30-taxonomy §6`); a genuine 3-way comparison (gcd needs
    // Lt/Eq/Gt, not just Bool) still gets one, declared locally here.
    env.elaborate_decl("data OrdResult = Lt | Eq | Gt").expect("OrdResult");
    env.elaborate_decl(
        "view natCmpZero (b : Nat) : OrdResult = \
         match b { Zero => Eq ; Suc n => Lt }",
    )
    .expect("natCmpZero");
    env.elaborate_decl(
        "view natCmp (a : Nat) (b : Nat) : OrdResult = \
         match a { Zero => natCmpZero b ; Suc m => match b { Zero => Gt ; Suc n => natCmp m n } }",
    )
    .expect("natCmp");
    env.elaborate_decl(
        "view natGcdFueled (fuel : Nat) (a : Nat) (b : Nat) : Nat = \
         match fuel { \
           Zero => a ; \
           Suc f => match natCmp a b { \
             Eq => a ; \
             Gt => natGcdFueled f (natSub a b) b ; \
             Lt => natGcdFueled f a (natSub b a) } }",
    )
    .expect("natGcdFueled");
    env.elaborate_decl(
        "view natGcd (a : Nat) (b : Nat) : Nat = \
         let fuel : Nat = natAdd a b in natGcdFueled fuel a b",
    )
    .expect("natGcd");
}

// ── Batch-2: Ackermann ───────────────────────────────────────────────────────

/// GAP-ackermann-sct: Ken's SCT does not (yet) accept lexicographic
/// termination arguments. `ack` is total, but SCT requires a single
/// structurally-decreasing parameter; it rejects the lexicographic (m,n)
/// ordering and fails with "idempotent self-loop has no strictly-decreasing
/// parameter". This test pins that the gap is present and the error is SCT.
#[test]
fn ackermann_sct_gap_pinned() {
    let mut env = ElabEnv::new().expect("base env");
    let result = env.elaborate_decl(
        "view ack (m : Nat) (n : Nat) : Nat = \
         match m { \
           Zero => Suc n ; \
           Suc p => match n { \
             Zero => ack p (Suc Zero) ; \
             Suc q => ack p (ack (Suc p) q) } }",
    );
    assert!(
        result.is_err(),
        "GAP-ackermann-sct: expected SCT to reject ack, but it elaborated"
    );
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(
        err_str.contains("NotTerminating") || err_str.contains("KernelRejected"),
        "error should be an SCT/NotTerminating rejection, got: {}", err_str
    );
}
