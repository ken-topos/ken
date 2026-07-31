//! `SURF-SPACE-CELLS-P1` — behavioral pins for `36 §4` cell surface.

use ken_elaborator::{
    effects::RowType, parser::parse_decls, Decl as SurfaceDecl, ElabEnv, ElabError, NumericLitVal,
};
use ken_interp::eval::{eval, EvalStore, EvalVal};
use ken_kernel::{Decl, GlobalId, Term};

const COUNTER_EXAMPLE: &str = r#"
space Counter {
  mut n : Int = 0
  proc inc () : Unit  visits [Counter] = n becomes n + 1
  proc get () : Int   visits [Counter] = n
}
"#;

const THREE_CELLS: &str = r#"
space Registers {
  mut left : Int = 11
  mut middle : Int = 22
  mut right : Int = 33
  proc read_left () : Int visits [Registers] = left
  proc read_middle () : Int visits [Registers] = middle
  proc read_right () : Int visits [Registers] = right
  proc write_middle () : Unit visits [Registers] = middle becomes 99
  proc write_middle_wide () : Unit visits [Registers, Console] = middle becomes 99
}
"#;

const INITIAL_COLLISION: &str = r#"
space Collision {
  mut n : Int = 7
  proc initial () : Int visits [Collision] = n
}
"#;

fn literal_value(value: &NumericLitVal, decimal_pair: GlobalId) -> EvalVal {
    match value {
        NumericLitVal::Int(value) => EvalVal::from(*value),
        NumericLitVal::Float(value) => EvalVal::Float(*value),
        NumericLitVal::Float32(value) => EvalVal::Float32(*value),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(decimal_pair, *coeff, *exp)
        }
        NumericLitVal::Str(value) => EvalVal::Str(value.clone()),
    }
}

fn eval_store(env: &ElabEnv) -> EvalStore {
    let mut store = EvalStore::new();
    for (id, value) in &env.num_values {
        store
            .num_values
            .insert(*id, literal_value(value, env.prelude_env.mkdecimalpair_id));
    }
    store
}

fn apply_all(head: Term, args: &[Term]) -> Term {
    args.iter().fold(head, |function, argument| {
        Term::app(function, argument.clone())
    })
}

fn run_operation(
    env: &ElabEnv,
    space: &str,
    operation: &str,
    result_type: Term,
) -> (EvalVal, EvalVal) {
    let p = &env.prelude_env;
    let state_id = env.globals[space];
    let initial_id = env
        .space_initial_state(space)
        .expect("space elaboration records its private initial state");
    let empty_id = env.globals["Empty"];
    let state_type = Term::const_(state_id, vec![]);
    let empty_type = Term::indformer(empty_id, vec![]);
    let resp_empty = Term::lam(empty_type.clone(), Term::indformer(p.unit_id, vec![]));
    let run = apply_all(
        Term::const_(p.run_state_id, vec![]),
        &[
            state_type,
            empty_type,
            resp_empty,
            result_type,
            Term::const_(initial_id, vec![]),
            Term::const_(env.globals[operation], vec![]),
        ],
    );
    let mut store = eval_store(env);
    let evaluated = eval(&[], &run, &env.env, &mut store);
    let payload = match evaluated {
        EvalVal::Ctor { id, args, .. } if id == p.ret_id => {
            args.last().cloned().expect("Ret carries its payload")
        }
        other => panic!("run_state must reduce to Ret, got {other:?}"),
    };
    match payload {
        EvalVal::Pair { fst, snd, .. } => (fst.as_ref().clone(), snd.as_ref().clone()),
        other => panic!("run_state payload must be (result, final-state), got {other:?}"),
    }
}

fn int(value: &EvalVal) -> i64 {
    match value {
        EvalVal::Int(value) => *value,
        EvalVal::BigInt(value) => value.clone().try_into().expect("fixture integer fits i64"),
        other => panic!("expected integer, got {other:?}"),
    }
}

fn state3(value: EvalVal) -> [i64; 3] {
    let EvalVal::Pair { fst: left, snd, .. } = value else {
        panic!("three-cell state must be a right-nested pair");
    };
    let EvalVal::Pair {
        fst: middle,
        snd: right,
        ..
    } = snd.as_ref()
    else {
        panic!("three-cell state tail must be a pair");
    };
    [int(&left), int(&middle), int(&right)]
}

fn peel_apps(term: &Term) -> (&Term, Vec<&Term>) {
    let mut head = term;
    let mut args = Vec::new();
    while let Term::App(function, argument) = head {
        args.push(argument.as_ref());
        head = function.as_ref();
    }
    args.reverse();
    (head, args)
}

#[test]
fn ac_s1_spec_counter_example_parses_verbatim() {
    let declarations = parse_decls(COUNTER_EXAMPLE).expect("the §4 Counter example must parse");
    assert!(matches!(
        declarations.as_slice(),
        [SurfaceDecl::SpaceDecl {
            name,
            cells,
            operations,
            ..
        }] if name == "Counter"
            && cells.len() == 1
            && operations.len() == 2
            && matches!(operations[0].body, ken_elaborator::Expr::EBecomes(..))
    ));
}

#[test]
fn ac_s2_middle_write_preserves_both_neighbors() {
    let mut env = ElabEnv::new().expect("prelude");
    env.elaborate_file(THREE_CELLS)
        .expect("three-cell space must elaborate");
    let unit_type = Term::indformer(env.prelude_env.unit_id, vec![]);

    let (_, written_state) = run_operation(&env, "Registers", "Registers.write_middle", unit_type);
    let actual = state3(written_state);
    println!("AC-S2 actual post-write state: {actual:?}");
    assert_eq!(
        actual,
        [11, 99, 33],
        "writing the middle cell must preserve both neighbors"
    );
}

#[test]
fn ac_s3_reads_each_of_three_pairwise_distinct_components() {
    let mut env = ElabEnv::new().expect("prelude");
    env.elaborate_file(THREE_CELLS)
        .expect("three-cell space must elaborate");
    let int_type = Term::const_(env.globals["Int"], vec![]);

    let (left, left_state) =
        run_operation(&env, "Registers", "Registers.read_left", int_type.clone());
    let (middle, middle_state) =
        run_operation(&env, "Registers", "Registers.read_middle", int_type.clone());
    let (right, right_state) = run_operation(&env, "Registers", "Registers.read_right", int_type);

    assert_eq!(
        (
            int(&left),
            int(&middle),
            int(&right),
            state3(left_state),
            state3(middle_state),
            state3(right_state),
        ),
        (11, 22, 33, [11, 22, 33], [11, 22, 33], [11, 22, 33],),
        "each read must select its own component and preserve the initial state"
    );
}

#[test]
fn ac_s4_write_core_is_bind_get_then_put() {
    let mut env = ElabEnv::new().expect("prelude");
    env.elaborate_file(THREE_CELLS)
        .expect("three-cell space must elaborate");
    let operation_id = env.globals["Registers.write_middle"];
    let body = match env.env.lookup(operation_id) {
        Some(Decl::Transparent { body, .. }) => body,
        other => panic!("space operation must be transparent, got {other:?}"),
    };
    let (bind_head, bind_args) = peel_apps(body);
    assert!(matches!(
        bind_head,
        Term::Const { id, .. } if *id == env.prelude_env.bind_id
    ));
    assert_eq!(
        bind_args.len(),
        6,
        "bind must receive its six explicit arguments"
    );
    let (get_head, _) = peel_apps(bind_args[4]);
    assert!(matches!(
        get_head,
        Term::Const { id, .. } if *id == env.prelude_env.get_fn_id
    ));
    let Term::Lam(_, continuation) = bind_args[5] else {
        panic!("bind continuation must name the fetched state");
    };
    let (put_head, _) = peel_apps(continuation);
    assert!(matches!(
        put_head,
        Term::Const { id, .. } if *id == env.prelude_env.put_fn_id
    ));
}

#[test]
fn ac_s5_space_label_is_emitted_and_required() {
    let mut env = ElabEnv::new().expect("prelude");
    let wide_result = env
        .elaborate_decl_v1(THREE_CELLS)
        .expect("three-cell space must elaborate");
    assert_eq!(
        env.effect_rows.get("Registers.write_middle"),
        Some(&RowType::singleton("Registers")),
        "the emitted operation row must carry its space label"
    );
    let wide_row = RowType::concrete(ken_elaborator::effects::EffectRow::from_effects([
        "Registers".to_string(),
        "Console".to_string(),
    ]));
    assert_eq!(
        env.effect_rows.get("Registers.write_middle_wide"),
        Some(&wide_row),
        "the declared operation row must retain stable-interface headroom"
    );
    assert_eq!(
        wide_result.effect_row_type.as_ref(),
        Some(&wide_row),
        "the elaboration result must retain the same declared row"
    );

    let mut env = ElabEnv::new().expect("prelude");
    let error = env
        .elaborate_file("space MissingRow { mut n : Int = 0 proc read () : Int = n }")
        .expect_err("cell access without visits [MissingRow] must fail");
    assert!(matches!(error, ElabError::TypeMismatch { ref reason, .. }
        if reason.contains("effect escape") && reason.contains("MissingRow")));

    let mut env = ElabEnv::new().expect("prelude");
    let error = env
        .elaborate_file(
            "space MissingLabel { \
               mut n : Int = 0 \
               proc read () : Int visits [Console] = n \
             }",
        )
        .expect_err("a declared row without the space label must fail");
    assert!(matches!(error, ElabError::TypeMismatch { ref reason, .. }
        if reason.contains("must include visits [MissingLabel]")));

    let mut env = ElabEnv::new().expect("prelude");
    let error = env
        .elaborate_file(
            "space UnsupportedTail { \
               mut n : Int = 0 \
               proc read () : Int visits [UnsupportedTail | e] = n \
             }",
        )
        .expect_err("an unbound space-row tail must fail closed");
    assert!(matches!(error, ElabError::TypeMismatch { ref reason, .. }
        if reason.contains("unknown row variable `e`")));
}

#[test]
// Promise class: durable invariant. Every space operation's inferred body
// effects remain a subset of its full retained declaration row.
fn ac_s5_space_body_effects_use_the_existing_escape_judgment() {
    let mut env = ElabEnv::new().expect("prelude");
    env.elaborate_decl("proc fs_source (x : Int) : Int visits [FS] = x")
        .expect("FS source");
    let error = env
        .elaborate_file(
            "space Escaping { \
               mut n : Int = 0 \
               proc read () : Int visits [Escaping] = fs_source n \
             }",
        )
        .expect_err("an undeclared body effect must escape");
    assert!(matches!(error, ElabError::TypeMismatch { ref reason, .. }
        if reason.contains("effect escape") && reason.contains("FS")));

    let mut env = ElabEnv::new().expect("prelude");
    env.elaborate_decl("proc fs_source (x : Int) : Int visits [FS] = x")
        .expect("FS source");
    env.elaborate_file(
        "space Covered { \
           mut n : Int = 0 \
           proc read () : Int visits [Covered, FS] = fs_source n \
         }",
    )
    .expect("the full declared row must cover both inferred effects");
    assert_eq!(
        env.effect_rows.get("Covered.read"),
        Some(&RowType::concrete(
            ken_elaborator::effects::EffectRow::from_effects([
                "Covered".to_string(),
                "FS".to_string(),
            ])
        )),
        "the accepted operation must retain its full declared row"
    );
}

#[test]
fn generated_initial_state_does_not_claim_the_initial_member() {
    let mut env = ElabEnv::new().expect("prelude");
    let emitted = env
        .elaborate_file(THREE_CELLS)
        .expect("three-cell space must elaborate");
    let initial_state_id = env
        .space_initial_state("Registers")
        .expect("the private initial state remains addressable");
    assert!(
        !env.globals.contains_key("Registers.initial"),
        "the generated initial state must not claim a source global"
    );
    assert!(
        !emitted.contains(&initial_state_id),
        "the generated initial state must not claim a public elaboration result"
    );

    let mut env = ElabEnv::new().expect("prelude");
    env.elaborate_file(INITIAL_COLLISION)
        .expect("an operation named initial must elaborate");

    let operation_id = env.globals["Collision.initial"];
    let initial_state_id = env
        .space_initial_state("Collision")
        .expect("the private initial state remains addressable");
    assert_ne!(
        operation_id, initial_state_id,
        "the source operation and generated initial state must stay distinct"
    );

    let int_type = Term::const_(env.globals["Int"], vec![]);
    let (result, final_state) = run_operation(&env, "Collision", "Collision.initial", int_type);
    assert_eq!(
        (int(&result), int(&final_state)),
        (7, 7),
        "the operation remains callable and uses the separate private initial state"
    );
}

#[test]
// Promise class: transition sentinel. Retire only when an authorized
// public-space export and effect-label contract replaces P1's refusal.
fn public_block_space_is_a_specific_surface_refusal() {
    let mut env = ElabEnv::new().expect("prelude");
    let error = env
        .elaborate_file(
            "pub space Public { \
               mut n : Int = 0 \
               proc read () : Int visits [Public] = n \
             }",
        )
        .expect_err("P1 does not define public block spaces");
    assert!(matches!(
        error,
        ElabError::UnsupportedSpacePlacement {
            ref placement,
            ref span
        } if placement == "public" && span.end > span.start
    ));
}

#[test]
// Promise class: transition sentinel. Retire only when an authorized nested
// qualification and effect-label contract replaces P1's refusal.
fn nested_block_space_is_a_specific_surface_refusal() {
    let mut env = ElabEnv::new().expect("prelude");
    let error = env
        .elaborate_file(
            "module Outer { \
               space Nested { \
                 mut n : Int = 0 \
                 proc read () : Int visits [Nested] = n \
               } \
             }",
        )
        .expect_err("P1 does not define nested block spaces");
    assert!(matches!(
        error,
        ElabError::UnsupportedSpacePlacement {
            ref placement,
            ref span
        } if placement == "nested" && span.end > span.start
    ));
}

#[test]
fn ac_s6_mut_outside_space_is_a_specific_error_with_a_span() {
    let mut env = ElabEnv::new().expect("prelude");
    let mut_error = env
        .elaborate_file("mut n : Int = 0")
        .expect_err("top-level mut must fail");
    assert!(matches!(
        mut_error,
        ElabError::MutationOutsideSpace {
            ref construct,
            ref span
        } if construct == "mut" && span.end > span.start
    ));
}

#[test]
fn ac_s6_becomes_outside_space_is_a_specific_error_with_a_span() {
    let mut env = ElabEnv::new().expect("prelude");
    let becomes_error = env
        .elaborate_file("fn bad (n : Int) : Unit = n becomes 1")
        .expect_err("becomes on an ordinary value must fail");
    assert!(matches!(
        becomes_error,
        ElabError::MutationOutsideSpace {
            ref construct,
            ref span
        } if construct == "becomes" && span.end > span.start
    ));
}

#[test]
fn ac_s7_old_in_space_preserves_the_specific_fence() {
    let mut env = ElabEnv::new().expect("prelude");
    let space_error = env
        .elaborate_file(
            "space OldFence { \
               mut n : Int = 0 \
               proc read () : Int ensures Equal Int result (old n) = n \
             }",
        )
        .expect_err("old in a space operation still fails closed");
    assert!(
        matches!(
            space_error,
            ElabError::OldPreStateUnsupported { ref span } if span.end > span.start
        ),
        "space `old` fence changed: {space_error:?}"
    );
}

#[test]
fn ac_s7_old_in_pure_code_remains_unbound() {
    let mut env = ElabEnv::new().expect("prelude");
    let pure_error = env
        .elaborate_decl("fn pure_old (n : Int) : Int ensures Equal Int result (old n) = n")
        .expect_err("old in a pure fn remains unbound");
    assert!(matches!(
        pure_error,
        ElabError::UnboundName { ref name, .. } if name == "old"
    ));
}
