use ken_kernel::inductive::check_positivity;
use ken_kernel::{
    declare_inductive, CtorSpec, GlobalEnv, InductiveSpec, KernelError, Level, LevelVar,
    ParameterPolarity, Term,
};

const U: LevelVar = LevelVar(0);
const V: LevelVar = LevelVar(1);

fn level_u() -> Level {
    Level::Var(U)
}

fn declare_bool(env: &mut GlobalEnv) -> ken_kernel::GlobalId {
    declare_inductive(env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
        ],
    })
    .expect("Bool declaration")
}

fn declare_list(env: &mut GlobalEnv) -> ken_kernel::GlobalId {
    declare_inductive(env, |list| InductiveSpec {
        level_params: vec![U],
        params: vec![Term::Type(level_u())],
        indices: vec![],
        level: level_u(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![
                    Term::var(0),
                    Term::app(Term::indformer(list, vec![level_u()]), Term::var(1)),
                ],
                target_indices: vec![],
            },
        ],
    })
    .expect("List declaration")
}

#[test]
fn admission_records_positive_and_non_positive_parameters() {
    // Durable invariant: polarity is declaration-derived metadata, independent
    // of a particular family name.
    let mut env = GlobalEnv::new();
    let bool_id = declare_bool(&mut env);
    let list_id = declare_list(&mut env);
    let contra_id = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![U],
        params: vec![Term::Type(level_u())],
        indices: vec![],
        level: level_u(),
        constructors: vec![CtorSpec {
            args: vec![Term::pi(Term::var(0), Term::indformer(bool_id, vec![]))],
            target_indices: vec![],
        }],
    })
    .expect("a negative parameter does not make the family itself recursive");
    let mixed_id = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![U, V],
        params: vec![Term::Type(level_u()), Term::Type(Level::Var(V))],
        indices: vec![],
        level: level_u().max(Level::Var(V)),
        constructors: vec![CtorSpec {
            // Under Δp=[A,B], the successive field contexts are:
            // A=Var(1); then B=Var(1); then A=Var(3) in the third field.
            args: vec![
                Term::var(1),
                Term::var(1),
                Term::pi(Term::var(3), Term::indformer(bool_id, vec![])),
            ],
            target_indices: vec![],
        }],
    })
    .expect("mixed two-parameter declaration");
    let unknown_nested_id = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![U],
        params: vec![Term::Type(level_u())],
        indices: vec![],
        level: level_u(),
        constructors: vec![CtorSpec {
            args: vec![Term::app(
                Term::indformer(list_id, vec![level_u()]),
                Term::pi(Term::var(0), Term::indformer(bool_id, vec![])),
            )],
            target_indices: vec![],
        }],
    })
    .expect("foreign carrier use remains admissible while its polarity fails closed");

    assert_eq!(
        env.inductive(list_id)
            .expect("recorded List")
            .parameter_polarities,
        vec![ParameterPolarity::StrictlyPositive]
    );
    assert_eq!(
        env.inductive(contra_id)
            .expect("recorded Contra")
            .parameter_polarities,
        vec![ParameterPolarity::NonPositive]
    );
    assert_eq!(
        env.inductive(mixed_id)
            .expect("recorded Mixed")
            .parameter_polarities,
        vec![
            ParameterPolarity::NonPositive,
            ParameterPolarity::StrictlyPositive,
        ]
    );
    assert_eq!(
        env.inductive(unknown_nested_id)
            .expect("recorded unknown nested use")
            .parameter_polarities,
        vec![ParameterPolarity::NonPositive],
        "unknown carrier positions must absorb nested polarity flips"
    );
}

#[test]
fn recorded_polarity_is_a_causal_admission_input() {
    // Durable invariant for D1a/AC-K11: changing recorded polarity changes the
    // admission clause's verdict rather than leaving inert metadata behind.
    let mut env = GlobalEnv::new();
    let bool_id = declare_bool(&mut env);
    let contra_id = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![U],
        params: vec![Term::Type(level_u())],
        indices: vec![],
        level: level_u(),
        constructors: vec![CtorSpec {
            args: vec![Term::pi(Term::var(0), Term::indformer(bool_id, vec![]))],
            target_indices: vec![],
        }],
    })
    .expect("correctly recorded negative position is admitted");

    let admitted = env.inductive(contra_id).expect("Contra in environment");
    assert!(check_positivity(admitted).is_ok());

    let mut perturbed = admitted.clone();
    perturbed.parameter_polarities[0] = ParameterPolarity::StrictlyPositive;
    assert!(matches!(
        check_positivity(&perturbed),
        Err(KernelError::PositivityViolation(message))
            if message == "recorded parameter polarity does not match the declaration"
    ));

    let mut missing = admitted.clone();
    missing.parameter_polarities.clear();
    assert!(matches!(
        check_positivity(&missing),
        Err(KernelError::PositivityViolation(message))
            if message == "parameter polarity record does not match the parameter telescope"
    ));
}

#[test]
fn d1a_preserves_the_nested_occurrence_rejection() {
    // Transition sentinel for the SPEC-NESTED-IND gate: D1a must keep this
    // exact nested declaration rejected until D1b deliberately retires it.
    let mut env = GlobalEnv::new();
    let list_id = declare_list(&mut env);
    let result = declare_inductive(&mut env, |rose| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![Term::app(
                    Term::indformer(list_id, vec![Level::zero()]),
                    Term::indformer(rose, vec![]),
                )],
                target_indices: vec![],
            },
        ],
    });

    assert!(matches!(
        result,
        Err(KernelError::PositivityViolation(message))
            if message.contains("non-strictly-positive occurrence of D")
                && message.ends_with("arg 0")
    ));
}
