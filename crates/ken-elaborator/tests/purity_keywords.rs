//! SURF-1 D2 acceptance smoke: `const`/`fn`/`proc` are checked against
//! declared purity and explicit value arity on the production elaboration path.

use ken_elaborator::ElabEnv;

fn err_text<T: std::fmt::Debug>(res: Result<T, ken_elaborator::ElabError>) -> String {
    format!("{:?}", res.expect_err("expected elaboration to reject"))
}

#[test]
fn surf1_d2_const_and_fn_accept_by_value_arity() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_decl("const surf1_answer : Int = 40 + 2")
        .expect("zero-value-param pure definition must accept as const");
    env.elaborate_decl("fn surf1_triple (n : Int) : Int = n + n + n")
        .expect("one-value-param pure definition must accept as fn");
}

#[test]
fn surf1_d2_pure_arity_mismatches_reject() {
    let mut env = ElabEnv::new().expect("base env");

    let fn_zero = err_text(env.elaborate_decl("fn surf1_zero : Int = 42"));
    assert!(
        fn_zero.contains("use `const`"),
        "zero-value-param fn must be rejected as should-be-const: {fn_zero}"
    );

    let const_param = err_text(env.elaborate_decl("const surf1_id (n : Int) : Int = n"));
    assert!(
        const_param.contains("use `fn`"),
        "const with a value param must be rejected as should-be-fn: {const_param}"
    );
}

#[test]
fn surf1_d2_proc_empty_row_must_earn_impurity() {
    let mut env = ElabEnv::new().expect("base env");

    let pure_proc = err_text(env.elaborate_decl("proc surf1_pure (n : Int) : Int = n"));
    assert!(
        pure_proc.contains("provably pure") && pure_proc.contains("use `fn`"),
        "empty-row pure proc must be a hard should-be-fn error: {pure_proc}"
    );

    env.elaborate_decl("proc surf1_headroom (n : Int) : Int visits [Console] = n")
        .expect("proc with non-empty declared row is honest headroom even when body is pure");
}

#[test]
fn surf1_d2_fn_calling_proc_reuses_escape_gate() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_decl("proc surf1_read (p : String) : String visits [FS] = p")
        .expect("declared non-empty proc seeds the surface effect row table");

    let bad_fn =
        err_text(env.elaborate_decl("fn surf1_load_bad (p : String) : String = surf1_read p"));
    assert!(
        bad_fn.contains("false purity or effect escape")
            && bad_fn.contains("EffectEscapes")
            && bad_fn.contains("FS"),
        "fn calling a proc must reject via the row escape gate: {bad_fn}"
    );

    env.elaborate_decl("proc surf1_load_ok (p : String) : String visits [FS] = surf1_read p")
        .expect("same body under proc with matching row must accept");
}

#[test]
fn surf1_d2_recursive_proc_group_with_visits_reaches_checker() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_file(
        "proc surf1_even_proc (n : Nat) : Bool visits [Console] = \
             match n { Zero |-> True ; Suc m |-> surf1_odd_proc m }\n\
         proc surf1_odd_proc (n : Nat) : Bool visits [Console] = \
             match n { Zero |-> False ; Suc m |-> surf1_even_proc m }",
    )
    .expect("mutually recursive proc group with visits must not be screened out");

    let bad_fn =
        err_text(env.elaborate_decl("fn surf1_even_bad (n : Nat) : Bool = surf1_even_proc n"));
    assert!(
        bad_fn.contains("false purity or effect escape")
            && bad_fn.contains("EffectEscapes")
            && bad_fn.contains("Console"),
        "effect rows from a recursive proc group must seed later purity checks: {bad_fn}"
    );
}

#[test]
fn surf1_d2_fn_and_const_cannot_declare_effect_rows() {
    let mut env = ElabEnv::new().expect("base env");

    let bad_fn =
        err_text(env.elaborate_decl("fn surf1_fn_row (n : Int) : Int visits [Console] = n"));
    assert!(
        bad_fn.contains("use `proc`"),
        "fn with a declared effect row must be rejected as should-be-proc: {bad_fn}"
    );

    let bad_const =
        err_text(env.elaborate_decl("const surf1_const_row : Int visits [Console] = 1"));
    assert!(
        bad_const.contains("use `proc`"),
        "const with a declared effect row must be rejected as should-be-proc: {bad_const}"
    );
}
