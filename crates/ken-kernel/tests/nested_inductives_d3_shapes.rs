use ken_kernel::inductive::{
    recursive_args, recursive_shapes, RecursiveArgumentShape, RecursiveFormerArgument,
    RecursiveShape,
};
use ken_kernel::{
    declare_inductive, ConstructorDecl, CtorSpec, GlobalEnv, GlobalId, InductiveSpec, KernelError,
    Level, LevelVar, Term,
};

const U: LevelVar = LevelVar(0);

fn ty0() -> Term {
    Term::Type(Level::zero())
}

fn former(id: GlobalId) -> Term {
    Term::indformer(id, vec![])
}

fn constructor(args: Vec<Term>) -> ConstructorDecl {
    ConstructorDecl {
        id: GlobalId(u32::MAX - 1),
        args,
        target_indices: vec![],
        type_: ty0(),
        recursive_positions: vec![],
    }
}

fn declare_positive_carrier(env: &mut GlobalEnv) -> GlobalId {
    declare_inductive(env, |carrier| InductiveSpec {
        level_params: vec![U],
        params: vec![Term::Type(Level::Var(U))],
        indices: vec![],
        level: Level::Var(U),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![Term::var(0)],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![
                    Term::app(Term::indformer(carrier, vec![Level::Var(U)]), Term::var(0)),
                    Term::app(Term::indformer(carrier, vec![Level::Var(U)]), Term::var(1)),
                ],
                target_indices: vec![],
            },
        ],
    })
    .expect("fresh carrier is strictly positive")
}

#[test]
fn direct_and_wstyle_shapes_project_to_the_legacy_api() {
    // Durable invariant: the preparatory descriptor is observationally inert
    // for every class consumed by the landed eliminator.
    let env = GlobalEnv::new();
    let d = GlobalId(u32::MAX);
    let bool_ty = ty0();
    let indexed_d = Term::app(Term::app(former(d), Term::var(0)), Term::var(1));
    let c = constructor(vec![
        bool_ty.clone(),
        indexed_d.clone(),
        Term::pi(bool_ty.clone(), Term::pi(Term::var(0), indexed_d)),
    ]);

    let shapes = recursive_shapes(&env, &c, d, 1).expect("direct/W shapes");
    let projected = shapes
        .iter()
        .map(|argument| {
            let (domains, indices) = argument
                .shape
                .as_legacy()
                .expect("direct and W-style shapes have a legacy projection");
            (argument.position, domains, indices)
        })
        .collect::<Vec<_>>();

    assert_eq!(projected, recursive_args(&c, d, 1));
    assert_eq!(shapes[0].shape.leaf_count(), 1);
    assert_eq!(shapes[1].shape.leaf_count(), 1);

    let d_free = constructor(vec![Term::pi(bool_ty.clone(), bool_ty)]);
    assert!(
        recursive_shapes(&env, &d_free, d, 0)
            .expect("D-free field")
            .is_empty(),
        "a D-free constructor contributes no recursive descriptor"
    );
}

#[test]
fn primitive_sigma_preserves_topology_and_both_motive_leaves() {
    // Durable invariant: primitive dependent Sigma is a native positive
    // container. Its two components remain distinct; neither may be flattened
    // or dropped by the descriptor producer.
    let env = GlobalEnv::new();
    let d = GlobalId(u32::MAX);
    let c = constructor(vec![Term::sigma(former(d), former(d))]);

    let shapes = recursive_shapes(&env, &c, d, 0).expect("Sigma shape");
    let direct = || {
        Some(Box::new(RecursiveShape::Direct {
            index_exprs: vec![],
        }))
    };
    assert_eq!(
        shapes,
        vec![RecursiveArgumentShape {
            position: 0,
            shape: RecursiveShape::Sigma {
                domain: direct(),
                codomain: direct(),
            },
        }]
    );
    assert_eq!(shapes[0].shape.leaf_count(), 2);
    assert!(
        shapes[0].shape.as_legacy().is_none(),
        "structured Sigma cannot collapse into a legacy flat IH"
    );
}

#[test]
fn checked_positive_former_paths_compose_without_opening_admission() {
    // Transition sentinel: the descriptor is ready for the later atomic
    // D3-semantic+D4 landing, while foreign-former admission remains rejected
    // until D1b.
    let mut env = GlobalEnv::new();
    let outer = declare_positive_carrier(&mut env);
    let inner = declare_positive_carrier(&mut env);
    let d = GlobalId(u32::MAX);
    let nested = Term::app(
        Term::indformer(outer, vec![Level::zero()]),
        Term::app(Term::indformer(inner, vec![Level::zero()]), former(d)),
    );
    let c = constructor(vec![nested]);

    let shapes = recursive_shapes(&env, &c, d, 0).expect("composed positive path");
    assert_eq!(shapes.len(), 1);
    assert_eq!(shapes[0].shape.leaf_count(), 1);
    let RecursiveShape::Former {
        former: outer_id,
        arguments: outer_arguments,
        ..
    } = &shapes[0].shape
    else {
        panic!("outer positive former topology was flattened");
    };
    assert_eq!(*outer_id, outer);
    let Some(outer_body) = outer_arguments[0].shape.as_deref() else {
        panic!("outer positive parameter lost its recursive shape");
    };
    let RecursiveShape::Former {
        former: inner_id,
        arguments: inner_arguments,
        ..
    } = outer_body
    else {
        panic!("inner positive former topology was flattened");
    };
    assert_eq!(*inner_id, inner);
    assert_eq!(
        inner_arguments,
        &[RecursiveFormerArgument {
            term: former(d),
            shape: Some(Box::new(RecursiveShape::Direct {
                index_exprs: vec![],
            })),
        }]
    );
    assert!(shapes[0].shape.as_legacy().is_none());

    let rejected = declare_inductive(&mut env, |rose| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![Term::app(
                Term::indformer(outer, vec![Level::zero()]),
                former(rose),
            )],
            target_indices: vec![],
        }],
    });
    assert!(
        matches!(rejected, Err(KernelError::PositivityViolation(_))),
        "preparatory D3 must not open nested admission"
    );
}
