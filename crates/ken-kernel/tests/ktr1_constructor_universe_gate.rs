use ken_kernel::env::Context;
use ken_kernel::{
    declare_inductive, infer, CtorSpec, GlobalEnv, InductiveSpec, KernelError, Level, LevelVar,
    Term,
};

const U: LevelVar = LevelVar(0);
const V: LevelVar = LevelVar(1);

fn var(level: LevelVar) -> Level {
    Level::Var(level)
}

fn empty_ctor(args: Vec<Term>) -> Vec<CtorSpec> {
    vec![CtorSpec {
        args,
        target_indices: vec![],
    }]
}

#[test]
fn same_level_universe_argument_is_rejected_with_specific_error() {
    let mut env = GlobalEnv::new();
    let before = env.clone();
    let error = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: empty_ctor(vec![Term::Type(Level::zero())]),
    })
    .expect_err("Type 0 cannot be a constructor field of a Type 0 family");

    assert_eq!(
        error,
        KernelError::ConstructorUniverseViolation {
            argument: Level::zero().suc(),
            family: Level::zero(),
        }
    );
    assert_eq!(
        error.to_string(),
        "constructor argument universe suc 0 exceeds family universe 0"
    );
    assert_eq!(
        env, before,
        "failed admission must restore GlobalEnv exactly"
    );
}

#[test]
fn lifted_universe_argument_is_accepted() {
    let mut env = GlobalEnv::new();
    declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero().suc(),
        constructors: empty_ctor(vec![Term::Type(Level::zero())]),
    })
    .expect("Type 0 is a legal field of a Type 1 family");
}

#[test]
fn parameters_are_context_not_constructor_fields() {
    let mut env = GlobalEnv::new();
    declare_inductive(&mut env, |list| InductiveSpec {
        level_params: vec![U],
        params: vec![Term::Type(var(U))],
        indices: vec![],
        level: var(U),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![
                    Term::var(0),
                    Term::app(Term::indformer(list, vec![var(U)]), Term::var(1)),
                ],
                target_indices: vec![],
            },
        ],
    })
    .expect("List parameter A : Type u must not be checked as a local field");
}

#[test]
fn w_style_recursive_argument_at_family_level_is_accepted() {
    let mut env = GlobalEnv::new();
    let bool_id = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![],
    })
    .expect("Bool-shaped domain");

    declare_inductive(&mut env, |tree| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: empty_ctor(vec![Term::pi(
            Term::indformer(bool_id, vec![]),
            Term::indformer(tree, vec![]),
        )]),
    })
    .expect("W-style field remains at Type 0");
}

#[test]
fn symbolic_sublevel_accepts_when_join_proves_it() {
    let mut env = GlobalEnv::new();
    declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![U, V],
        params: vec![Term::Type(var(U))],
        indices: vec![],
        level: var(U).max(var(V)),
        constructors: empty_ctor(vec![Term::var(0)]),
    })
    .expect("u is below max u v");
}

#[test]
fn incomparable_symbolic_level_is_rejected_closed() {
    let mut env = GlobalEnv::new();
    let error = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![U, V],
        params: vec![Term::Type(var(V))],
        indices: vec![],
        level: var(U),
        constructors: empty_ctor(vec![Term::var(0)]),
    })
    .expect_err("the kernel cannot prove v <= u");

    assert_eq!(
        error,
        KernelError::ConstructorUniverseViolation {
            argument: var(V),
            family: var(U),
        }
    );
}

#[test]
fn proposition_payload_at_family_level_is_accepted() {
    let mut env = GlobalEnv::new();
    let family = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![Term::Omega(Level::zero())],
        indices: vec![],
        level: Level::zero(),
        constructors: empty_ctor(vec![Term::var(0)]),
    })
    .expect("proof payloads in Ω0 remain valid fields of a Type 0 family");

    let former = Term::app(
        Term::indformer(family, vec![]),
        Term::Const {
            id: env.top_id(),
            level_args: vec![],
        },
    );
    assert!(infer(&env, &Context::new(), &former).is_ok());
}
