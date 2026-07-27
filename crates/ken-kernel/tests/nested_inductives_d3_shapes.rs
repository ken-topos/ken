use ken_kernel::inductive::{
    recursive_args, recursive_shapes, RecursiveArgumentShape, RecursiveShape,
};
use ken_kernel::{
    convert_type, declare_def, declare_inductive, declare_postulate, ConstructorDecl, Context,
    CtorSpec, GlobalEnv, GlobalId, InductiveSpec, KernelError, Level, LevelVar, Term,
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

fn optional_topology_equivalent(
    left: &Option<Box<RecursiveShape>>,
    right: &Option<Box<RecursiveShape>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => topology_equivalent(left, right),
        _ => false,
    }
}

fn topology_equivalent(left: &RecursiveShape, right: &RecursiveShape) -> bool {
    match (left, right) {
        (
            RecursiveShape::Direct { index_exprs: left },
            RecursiveShape::Direct { index_exprs: right },
        ) => left.len() == right.len(),
        (
            RecursiveShape::Pi {
                domains: left_domains,
                body: left_body,
            },
            RecursiveShape::Pi {
                domains: right_domains,
                body: right_body,
            },
        ) => {
            left_domains.len() == right_domains.len() && topology_equivalent(left_body, right_body)
        }
        (
            RecursiveShape::Sigma {
                domain: left_domain,
                codomain: left_codomain,
            },
            RecursiveShape::Sigma {
                domain: right_domain,
                codomain: right_codomain,
            },
        ) => {
            optional_topology_equivalent(left_domain, right_domain)
                && optional_topology_equivalent(left_codomain, right_codomain)
        }
        (
            RecursiveShape::Former {
                former: left_former,
                level_args: left_levels,
                arguments: left_arguments,
            },
            RecursiveShape::Former {
                former: right_former,
                level_args: right_levels,
                arguments: right_arguments,
            },
        ) => {
            left_former == right_former
                && left_levels.len() == right_levels.len()
                && left_levels
                    .iter()
                    .zip(right_levels)
                    .all(|(left, right)| left.equiv(right))
                && left_arguments.len() == right_arguments.len()
                && left_arguments.iter().zip(right_arguments).all(
                    |(left_argument, right_argument)| {
                        optional_topology_equivalent(&left_argument.shape, &right_argument.shape)
                    },
                )
        }
        _ => false,
    }
}

fn descriptors_equivalent(
    env: &GlobalEnv,
    base_ctx: &Context,
    left_constructor: &ConstructorDecl,
    right_constructor: &ConstructorDecl,
    left: &[RecursiveArgumentShape],
    right: &[RecursiveArgumentShape],
) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            if left.position != right.position
                || left.position >= left_constructor.args.len()
                || right.position >= right_constructor.args.len()
                || !topology_equivalent(&left.shape, &right.shape)
            {
                return false;
            }
            let mut field_ctx = base_ctx.clone();
            for (left_prefix, right_prefix) in left_constructor.args[..left.position]
                .iter()
                .zip(&right_constructor.args[..right.position])
            {
                if !convert_type(env, &field_ctx, left_prefix, right_prefix) {
                    return false;
                }
                field_ctx.push(left_prefix.clone());
            }
            convert_type(
                env,
                &field_ctx,
                &left_constructor.args[left.position],
                &right_constructor.args[right.position],
            )
        })
}

fn constructor_shapes_equivalent(
    env: &GlobalEnv,
    ctx: &Context,
    left: &ConstructorDecl,
    right: &ConstructorDecl,
    d: GlobalId,
    parameter_count: usize,
) -> bool {
    let Ok(left_shapes) = recursive_shapes(env, left, d, parameter_count) else {
        return false;
    };
    let Ok(right_shapes) = recursive_shapes(env, right, d, parameter_count) else {
        return false;
    };
    descriptors_equivalent(env, ctx, left, right, &left_shapes, &right_shapes)
}

fn assert_equivalent_shapes(
    env: &GlobalEnv,
    left: &ConstructorDecl,
    right: &ConstructorDecl,
    d: GlobalId,
    parameter_count: usize,
    message: &str,
) {
    assert_eq!(left.args.len(), 1, "property helper expects one field");
    assert_eq!(right.args.len(), 1, "property helper expects one field");
    let ctx = Context::new();
    assert!(
        convert_type(env, &ctx, &left.args[0], &right.args[0]),
        "property precondition failed: field types are not definitionally equal"
    );
    assert!(
        constructor_shapes_equivalent(env, &ctx, left, right, d, parameter_count),
        "{message}"
    );
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
    assert_eq!(shapes.len(), 1);
    assert_eq!(shapes[0].position, 0);
    let RecursiveShape::Sigma { domain, codomain } = &shapes[0].shape else {
        panic!("primitive Sigma topology was flattened");
    };
    assert!(matches!(
        domain.as_deref(),
        Some(RecursiveShape::Direct { index_exprs }) if index_exprs.is_empty()
    ));
    assert!(matches!(
        codomain.as_deref(),
        Some(RecursiveShape::Direct { index_exprs }) if index_exprs.is_empty()
    ));
    assert_eq!(shapes[0].shape.leaf_count(), 2);
    assert!(
        shapes[0].shape.as_legacy().is_none(),
        "structured Sigma cannot collapse into a legacy flat IH"
    );
}

#[test]
fn transparent_aliases_preserve_former_topology_and_fail_closed_otherwise() {
    // Durable invariant: transparent delta reduction is spelling-invariant for
    // recursive topology, including universe instantiation. Opaque and missing
    // heads remain the fail-closed boundary.
    let mut env = GlobalEnv::new();
    let carrier = declare_positive_carrier(&mut env);
    let alias_ty = Term::pi(Term::Type(Level::Var(U)), Term::Type(Level::Var(U)));
    let alias = declare_def(
        &mut env,
        vec![U],
        alias_ty.clone(),
        Term::indformer(carrier, vec![Level::Var(U)]),
    )
    .expect("transparent alias of the positive carrier");
    let chained_alias = declare_def(
        &mut env,
        vec![U],
        alias_ty.clone(),
        Term::const_(alias, vec![Level::Var(U)]),
    )
    .expect("transparent alias chain");
    let lambda_alias = declare_def(
        &mut env,
        vec![U],
        alias_ty.clone(),
        Term::lam(
            Term::Type(Level::Var(U)),
            Term::app(Term::indformer(carrier, vec![Level::Var(U)]), Term::var(0)),
        ),
    )
    .expect("lambda-bodied transparent alias");
    let opaque_alias = declare_postulate(
        &mut env,
        "opaque carrier alias".to_string(),
        vec![U],
        alias_ty,
    )
    .expect("opaque alias declaration");
    let d = GlobalId(u32::MAX);
    let direct = constructor(vec![Term::app(
        Term::indformer(carrier, vec![Level::zero()]),
        former(d),
    )]);
    let through_alias = constructor(vec![Term::app(
        Term::const_(alias, vec![Level::zero()]),
        former(d),
    )]);
    let through_chain = constructor(vec![Term::app(
        Term::const_(chained_alias, vec![Level::zero()]),
        former(d),
    )]);
    let through_lambda = constructor(vec![Term::app(
        Term::const_(lambda_alias, vec![Level::zero()]),
        former(d),
    )]);
    let semilattice_level = constructor(vec![Term::app(
        Term::indformer(carrier, vec![Level::zero().max(Level::zero())]),
        former(d),
    )]);

    assert_equivalent_shapes(
        &env,
        &through_alias,
        &direct,
        d,
        0,
        "transparent aliases must preserve the instantiated Former topology",
    );
    assert_equivalent_shapes(
        &env,
        &through_chain,
        &direct,
        d,
        0,
        "finite transparent alias chains must preserve the same topology",
    );
    assert_equivalent_shapes(
        &env,
        &through_lambda,
        &direct,
        d,
        0,
        "beta-equivalent transparent aliases must preserve the same topology",
    );
    assert_equivalent_shapes(
        &env,
        &semilattice_level,
        &direct,
        d,
        0,
        "semilattice-equivalent universe spellings must preserve topology",
    );

    let proposition = declare_postulate(
        &mut env,
        "descriptor equality proposition".to_string(),
        vec![],
        Term::Omega(Level::zero()),
    )
    .expect("proposition declaration");
    let proof_ty = Term::const_(proposition, vec![]);
    let left_proof = declare_postulate(
        &mut env,
        "left descriptor proof".to_string(),
        vec![],
        proof_ty.clone(),
    )
    .expect("left proof declaration");
    let right_proof = declare_postulate(
        &mut env,
        "right descriptor proof".to_string(),
        vec![],
        proof_ty.clone(),
    )
    .expect("right proof declaration");
    let proof_carrier = declare_inductive(&mut env, |_proof_carrier| InductiveSpec {
        level_params: vec![],
        params: vec![proof_ty, ty0()],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![],
            target_indices: vec![],
        }],
    })
    .expect("carrier with a proof parameter");
    let through_left_proof = constructor(vec![Term::app(
        Term::app(
            Term::indformer(proof_carrier, vec![]),
            Term::const_(left_proof, vec![]),
        ),
        former(d),
    )]);
    let through_right_proof = constructor(vec![Term::app(
        Term::app(
            Term::indformer(proof_carrier, vec![]),
            Term::const_(right_proof, vec![]),
        ),
        former(d),
    )]);
    assert_equivalent_shapes(
        &env,
        &through_left_proof,
        &through_right_proof,
        d,
        0,
        "proof-irrelevant retained arguments must preserve topology",
    );

    let beta_type = Term::app(
        Term::lam(Term::Type(Level::zero().suc()), Term::var(0)),
        ty0(),
    );
    let direct_index = constructor(vec![Term::app(former(d), ty0())]);
    let beta_index = constructor(vec![Term::app(former(d), beta_type.clone())]);
    assert_equivalent_shapes(
        &env,
        &beta_index,
        &direct_index,
        d,
        0,
        "definitionally equal recursive indices must have identical topology",
    );
    assert_eq!(
        recursive_shapes(&env, &beta_index, d, 0).expect("beta-normalized direct index")[0]
            .shape
            .as_legacy(),
        recursive_shapes(&env, &direct_index, d, 0).expect("normal direct index")[0]
            .shape
            .as_legacy(),
        "the descriptor stores recursive indices in full normal form"
    );

    let direct_pi = constructor(vec![Term::pi(ty0(), former(d))]);
    let beta_pi = constructor(vec![Term::pi(beta_type, former(d))]);
    assert_equivalent_shapes(
        &env,
        &beta_pi,
        &direct_pi,
        d,
        0,
        "definitionally equal branching domains must have identical topology",
    );
    assert_eq!(
        recursive_shapes(&env, &beta_pi, d, 0).expect("beta-normalized Pi domain")[0]
            .shape
            .as_legacy(),
        recursive_shapes(&env, &direct_pi, d, 0).expect("normal Pi domain")[0]
            .shape
            .as_legacy(),
        "the descriptor stores branching domains in full normal form"
    );

    let opaque = constructor(vec![Term::app(
        Term::const_(opaque_alias, vec![Level::zero()]),
        former(d),
    )]);
    assert!(matches!(
        recursive_shapes(&env, &opaque, d, 0),
        Err(KernelError::PositivityViolation(message))
            if message.contains("opaque or unresolved application head")
    ));

    let unresolved = constructor(vec![Term::app(
        Term::const_(GlobalId(u32::MAX - 2), vec![Level::zero()]),
        former(d),
    )]);
    assert!(matches!(
        recursive_shapes(&env, &unresolved, d, 0),
        Err(KernelError::PositivityViolation(message))
            if message.contains("opaque or unresolved application head")
    ));
}

#[test]
fn descriptor_equivalence_validates_dependent_telescope_prefixes_in_lockstep() {
    // Durable invariant: a dependent recursive field may share a comparison
    // context only after every preceding telescope entry converts there.
    let mut env = GlobalEnv::new();
    let d = declare_inductive(&mut env, |family| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero().suc().suc(),
        constructors: vec![
            CtorSpec {
                args: vec![
                    ty0(),
                    Term::pi(Term::var(0), Term::indformer(family, vec![])),
                ],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![
                    Term::Type(Level::zero().max(Level::zero())),
                    Term::pi(Term::var(0), Term::indformer(family, vec![])),
                ],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![
                    Term::Omega(Level::zero()),
                    Term::pi(Term::var(0), Term::indformer(family, vec![])),
                ],
                target_indices: vec![],
            },
        ],
    })
    .expect("dependent-prefix family is admitted");
    let constructors = env
        .inductive(d)
        .expect("admitted family metadata")
        .constructors
        .clone();
    let ctx = Context::new();

    assert!(
        constructor_shapes_equivalent(&env, &ctx, &constructors[0], &constructors[1], d, 0,),
        "convertible dependent prefixes must establish a shared field context"
    );
    assert!(
        !constructor_shapes_equivalent(&env, &ctx, &constructors[0], &constructors[2], d, 0,),
        "non-convertible dependent prefixes must reject before field comparison"
    );
    assert!(
        constructor_shapes_equivalent(&env, &ctx, &constructors[1], &constructors[0], d, 0,),
        "the shared dependent context must be valid in either comparison direction"
    );
    assert!(
        !constructor_shapes_equivalent(&env, &ctx, &constructors[2], &constructors[0], d, 0,),
        "prefix rejection must not depend on which telescope is left"
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
    assert_eq!(inner_arguments.len(), 1);
    assert_eq!(inner_arguments[0].term, former(d));
    assert!(matches!(
        inner_arguments[0].shape.as_deref(),
        Some(RecursiveShape::Direct { index_exprs }) if index_exprs.is_empty()
    ));
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
