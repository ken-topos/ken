use ken_kernel::inductive::{iota_reduct, method_type, peel_app, peel_pi};
use ken_kernel::{
    check, convert, convert_type, declare_inductive, infer, ConstructorDecl, Context, CtorSpec,
    Decl, GlobalEnv, GlobalId, InductiveDecl, InductiveSpec, Level, LevelVar, Term,
};

const U: LevelVar = LevelVar(0);

fn level_u() -> Level {
    Level::Var(U)
}

fn ty0() -> Term {
    Term::Type(Level::zero())
}

fn former(id: GlobalId) -> Term {
    Term::indformer(id, vec![])
}

fn constructor(id: GlobalId) -> Term {
    Term::constructor(id, vec![])
}

fn sigma_pair_of_family(d: GlobalId) -> Term {
    Term::sigma(former(d), former(d))
}

fn beta_sigma_pair_of_family(d: GlobalId) -> Term {
    Term::app(
        Term::lam(ty0(), Term::sigma(Term::var(0), Term::var(1))),
        former(d),
    )
}

fn declare_sigma_tree(env: &mut GlobalEnv) -> GlobalId {
    declare_inductive(env, |tree| InductiveSpec {
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
                args: vec![sigma_pair_of_family(tree)],
                target_indices: vec![],
            },
        ],
    })
    .expect("primitive Sigma-recursive family is already admitted")
}

fn sigma_tree_terms(env: &GlobalEnv, tree: GlobalId) -> (Term, Vec<Term>, Term, Term, Term) {
    let ind = env.inductive(tree).expect("Sigma tree declaration");
    let leaf = ind.constructors[0].id;
    let fork = ind.constructors[1].id;
    let tree_type = former(tree);
    let field_type = sigma_pair_of_family(tree);
    let lifted_type = Term::sigma(tree_type.clone(), tree_type.clone());
    let motive = Term::Ascript(
        Box::new(Term::lam(tree_type.clone(), tree_type.clone())),
        Box::new(Term::pi(tree_type.clone(), ty0())),
    );
    let leaf_method = constructor(leaf);
    let fork_method = Term::lam(
        field_type,
        Term::lam(lifted_type, Term::app(constructor(fork), Term::var(0))),
    );
    let leaf_value = constructor(leaf);
    let field_value = Term::pair(leaf_value.clone(), leaf_value);
    let scrutinee = Term::app(constructor(fork), field_value.clone());
    (
        motive,
        vec![leaf_method, fork_method],
        field_value,
        scrutinee,
        tree_type,
    )
}

fn constructor_with_field(template: &ConstructorDecl, field_type: Term) -> ConstructorDecl {
    let mut constructor = template.clone();
    constructor.args = vec![field_type];
    constructor
}

fn apply_terms(mut head: Term, arguments: &[Term]) -> Term {
    for argument in arguments {
        head = Term::app(head, argument.clone());
    }
    head
}

fn declare_list(env: &mut GlobalEnv) -> GlobalId {
    declare_inductive(env, |list| InductiveSpec {
        level_params: vec![],
        params: vec![ty0()],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![
                    Term::var(0),
                    Term::app(Term::indformer(list, vec![]), Term::var(1)),
                ],
                target_indices: vec![],
            },
        ],
    })
    .expect("ordinary positive List")
}

fn declare_poly_box(env: &mut GlobalEnv) -> GlobalId {
    declare_inductive(env, |_| InductiveSpec {
        level_params: vec![U],
        params: vec![Term::Type(level_u())],
        indices: vec![],
        level: level_u(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] },
            CtorSpec { args: vec![Term::var(0)], target_indices: vec![] },
        ],
    })
    .expect("polymorphic unary PolyBox declaration")
}

fn install_test_only_polymorphic_nested_family(
    env: &mut GlobalEnv,
    poly_box: GlobalId,
) -> GlobalId {
    let family = env.fresh_id();
    let leaf = env.fresh_id();
    let wrap = env.fresh_id();
    let mut declaration = InductiveDecl {
        id: family,
        level_params: vec![U],
        params: vec![],
        parameter_polarities: vec![],
        indices: vec![],
        level: level_u(),
        constructors: vec![
            ConstructorDecl {
                id: leaf,
                args: vec![],
                target_indices: vec![],
                type_: ty0(),
                recursive_positions: vec![],
            },
            ConstructorDecl {
                id: wrap,
                args: vec![Term::app(
                    Term::indformer(poly_box, vec![level_u()]),
                    Term::indformer(family, vec![level_u()]),
                )],
                target_indices: vec![],
                type_: ty0(),
                recursive_positions: vec![],
            },
        ],
        former_type: ty0(),
    };
    declaration.build_types();
    env.add_decl(Decl::Inductive(declaration));
    family
}

#[test]
fn polymorphic_former_transport_preserves_guest_levels() {
    // A general-host fixture: the guest family and its unary former both carry
    // a universe parameter.  The motive and source levels are intentionally
    // different so the transport path cannot silently reuse a retained head.
    let mut env = GlobalEnv::new();
    let poly_box = declare_poly_box(&mut env);
    let family = install_test_only_polymorphic_nested_family(&mut env, poly_box);
    let declaration = env.inductive(family).expect("polymorphic D declaration");
    let leaf = declaration.constructors[0].id;
    let wrap = declaration.constructors[1].id;
    let poly = env.inductive(poly_box).expect("PolyBox declaration");
    let box_ctor = poly.constructors[1].id;
    let d0 = Term::indformer(family, vec![Level::zero()]);
    let boxed_leaf = Term::app(
        Term::app(
            Term::constructor(box_ctor, vec![Level::zero()]),
            d0.clone(),
        ),
        Term::constructor(leaf, vec![Level::zero()]),
    );
    let motive = Term::Ascript(
        Box::new(Term::lam(d0.clone(), Term::Type(Level::zero().suc()))),
        Box::new(Term::pi(d0.clone(), Term::Type(Level::zero().suc().suc()))),
    );
    let leaf_method = Term::Type(Level::zero());
    let wrap_method_type = method_type(&env, declaration, 1, &motive, &[], &[Level::zero()])
        .expect("PolyBox Former host method type");
    let (wrap_domains, _) = peel_pi(&wrap_method_type);
    assert_eq!(wrap_domains.len(), 2, "field and Former lift binders");
    let wrap_method = Term::lam(
        wrap_domains[0].clone(),
        Term::lam(wrap_domains[1].clone(), Term::Type(Level::zero())),
    );
    check(&env, &Context::new(), &leaf_method,
          &method_type(&env, declaration, 0, &motive, &[], &[Level::zero()]).unwrap())
        .expect("PolyBox leaf method");
    check(&env, &Context::new(), &wrap_method, &wrap_method_type)
        .expect("PolyBox Former method");
    let methods = vec![leaf_method, wrap_method];
    let eliminator = Term::Elim {
        fam: family,
        level_args: vec![Level::zero()],
        params: vec![],
        motive: Box::new(motive.clone()),
        methods: methods.clone(),
        indices: vec![],
        scrut: Box::new(Term::app(
            Term::constructor(wrap, vec![Level::zero()]),
            boxed_leaf.clone(),
        )),
    };
    let elim_ty = infer(&env, &Context::new(), &eliminator).expect("guest eliminator");
    check(&env, &Context::new(), &eliminator, &elim_ty).expect("checked guest eliminator");
    let reduct = iota_reduct(
        &env, declaration, 1, &[Level::zero()], &[], &motive, &methods,
        std::slice::from_ref(&boxed_leaf),
    ).expect("PolyBox Former iota");
    let (_, args) = peel_app(&reduct);
    let supplied = args.last().expect("supplied Former lift");
    check(&env, &Context::new(), supplied, &wrap_domains[1])
        .expect("checked type-level Former lift");
    if let Term::Elim { fam, level_args, .. } = supplied {
        assert_eq!(*fam, poly_box);
        assert_eq!(level_args, &vec![Level::zero()]);
    } else {
        panic!("Former lift is not a host elimination");
    }
}

fn install_test_only_nested_family(env: &mut GlobalEnv, list: GlobalId) -> GlobalId {
    // D1b deliberately remains closed in this slice.  This declaration is
    // installed directly in a test environment so the already-produced Former
    // skeleton can be exercised semantically before admission is opened.
    let family = env.fresh_id();
    let leaf = env.fresh_id();
    let wrap = env.fresh_id();
    let mut declaration = InductiveDecl {
        id: family,
        level_params: vec![],
        params: vec![],
        parameter_polarities: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            ConstructorDecl {
                id: leaf,
                args: vec![],
                target_indices: vec![],
                type_: ty0(),
                recursive_positions: vec![],
            },
            ConstructorDecl {
                id: wrap,
                args: vec![Term::app(former(list), former(family))],
                target_indices: vec![],
                type_: ty0(),
                recursive_positions: vec![],
            },
        ],
        former_type: ty0(),
    };
    declaration.build_types();
    env.add_decl(Decl::Inductive(declaration));
    family
}

#[test]
fn sigma_lift_method_and_iota_are_one_kernel_checked_pair() {
    // Durable invariant (AC-K14): primitive dependent Sigma contributes one
    // structured IH whose two leaves are both supplied by iota.  This test
    // discriminates both halves: deleting the method binder changes the Pi
    // spine; deleting or perturbing either iota leaf makes the explicit
    // kernel check below fail.
    let mut env = GlobalEnv::new();
    let tree = declare_sigma_tree(&mut env);
    let ind = env.inductive(tree).expect("Sigma tree declaration");
    let (motive, methods, field_value, scrutinee, tree_type) = sigma_tree_terms(&env, tree);
    for (index, method) in methods.iter().enumerate() {
        let ty = method_type(&env, ind, index, &motive, &[], &[]).unwrap();
        check(&env, &Context::new(), method, &ty)
            .unwrap_or_else(|error| panic!("method {index} failed: {error}; type={ty:?}"));
    }

    let fork_method_type =
        method_type(&env, ind, 1, &motive, &[], &[]).expect("structured method type");
    let (method_domains, _) = peel_pi(&fork_method_type);
    assert_eq!(
        method_domains.len(),
        2,
        "fork method has its field binder and one structured lift binder"
    );
    let expected_lift_type = Term::sigma(tree_type.clone(), tree_type.clone());
    assert!(
        convert_type(
            &env,
            &Context::new(),
            &method_domains[1],
            &expected_lift_type,
        ),
        "the structured Sigma lift preserves both motive leaves"
    );

    let reduct = iota_reduct(
        &env,
        ind,
        1,
        &[],
        &[],
        &motive,
        &methods,
        std::slice::from_ref(&field_value),
    )
    .expect("matching structured iota");
    let (_, reduct_args) = peel_app(&reduct);
    assert_eq!(
        reduct_args.len(),
        2,
        "iota supplies the field and exactly one structured lift"
    );
    let Term::Pair(first_leaf, second_leaf) = &reduct_args[1] else {
        panic!("structured iota lift must preserve Sigma topology");
    };
    assert!(matches!(first_leaf.as_ref(), Term::Elim { .. }));
    assert!(matches!(second_leaf.as_ref(), Term::Elim { .. }));
    let instantiated_lift_type = ken_kernel::subst::subst0(&method_domains[1], &field_value);
    check(
        &env,
        &Context::new(),
        &reduct_args[1],
        &instantiated_lift_type,
    )
    .expect("the iota-supplied lift inhabits the generated method binder");

    let eliminator = Term::Elim {
        fam: tree,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods,
        indices: vec![],
        scrut: Box::new(scrutinee),
    };
    let inferred = infer(&env, &Context::new(), &eliminator)
        .expect("the kernel checks the complete generated method/iota pair");
    assert!(convert_type(&env, &Context::new(), &inferred, &tree_type,));
}

#[test]
fn convertible_field_spellings_generate_convertible_lifts_and_iota_terms() {
    // Durable invariant (amended AC-K15): conversion, not descriptor Rust
    // identity, is the consumer boundary.  Beta-equivalent constructor field
    // spellings yield convertible generated methods, and each matching iota
    // lift checks at that method's binder type.
    let mut env = GlobalEnv::new();
    let tree = declare_sigma_tree(&mut env);
    let admitted = env.inductive(tree).expect("Sigma tree declaration");
    let direct = admitted.clone();
    let mut beta = admitted.clone();
    beta.constructors[1] =
        constructor_with_field(&admitted.constructors[1], beta_sigma_pair_of_family(tree));
    let (motive, methods, field_value, _, _) = sigma_tree_terms(&env, tree);

    let direct_method = method_type(&env, &direct, 1, &motive, &[], &[]).expect("direct method");
    let beta_method = method_type(&env, &beta, 1, &motive, &[], &[]).expect("beta method");
    assert!(
        convert_type(&env, &Context::new(), &direct_method, &beta_method,),
        "definitionally equal fields must generate definitionally equal method/lift types"
    );

    let direct_reduct = iota_reduct(
        &env,
        &direct,
        1,
        &[],
        &[],
        &motive,
        &methods,
        std::slice::from_ref(&field_value),
    )
    .expect("direct iota");
    let beta_reduct = iota_reduct(
        &env,
        &beta,
        1,
        &[],
        &[],
        &motive,
        &methods,
        std::slice::from_ref(&field_value),
    )
    .expect("beta iota");
    let (_, direct_args) = peel_app(&direct_reduct);
    let (_, beta_args) = peel_app(&beta_reduct);
    let direct_lift = direct_args.last().expect("direct structured lift");
    let beta_lift = beta_args.last().expect("beta structured lift");
    let direct_lift_type = peel_pi(&direct_method).0[1].clone();
    let beta_lift_type = peel_pi(&beta_method).0[1].clone();
    check(&env, &Context::new(), direct_lift, &direct_lift_type)
        .expect("direct matching iota term inhabits its lift");
    check(&env, &Context::new(), beta_lift, &beta_lift_type)
        .expect("beta matching iota term inhabits its lift");
    assert!(
        convert(
            &env,
            &Context::new(),
            &direct_lift_type,
            direct_lift,
            beta_lift,
        ),
        "matching iota terms are definitionally equal at the quotient boundary"
    );
}

#[test]
fn declared_positive_former_lift_builds_dependent_host_evidence() {
    // Durable invariant: the D3a Former skeleton is a semantic consumer, not a
    // test-only ornament. List's original eliminator builds an All-style
    // evidence family without re-instantiating List at a larger universe.
    // Admission of this outer family remains a later D1b concern.
    let mut env = GlobalEnv::new();
    let list = declare_list(&mut env);
    let family = install_test_only_nested_family(&mut env, list);
    let declaration = env.inductive(family).expect("test-only nested family");
    let leaf = declaration.constructors[0].id;
    let list_declaration = env.inductive(list).expect("List declaration");
    let nil = list_declaration.constructors[0].id;
    let cons = list_declaration.constructors[1].id;
    let family_type = former(family);
    let motive = Term::Ascript(
        Box::new(Term::lam(
            family_type.clone(),
            Term::sigma(
                Term::Eq(
                    Box::new(family_type.clone()),
                    Box::new(Term::var(0)),
                    Box::new(Term::var(0)),
                ),
                family_type.clone(),
            ),
        )),
        Box::new(Term::pi(family_type.clone(), ty0())),
    );
    let wrap_method_type =
        method_type(&env, declaration, 1, &motive, &[], &[])
            .expect("Former method lift");
    let (wrap_domains, _) = peel_pi(&wrap_method_type);
    assert_eq!(wrap_domains.len(), 2);
    let methods = vec![
        Term::pair(ken_kernel::obs::tt_term(&env), constructor(leaf)),
        Term::lam(
            wrap_domains[0].clone(),
            Term::lam(
                wrap_domains[1].clone(),
                Term::pair(
                    Term::Refl(Box::new(Term::var(1))),
                    constructor(leaf),
                ),
            ),
        ),
    ];
    let leaf_value = constructor(leaf);
    let nil_value = Term::app(constructor(nil), family_type.clone());
    let list_value = apply_terms(
        constructor(cons),
        &[family_type.clone(), leaf_value, nil_value],
    );

    let method = wrap_method_type;
    let (domains, _) = peel_pi(&method);
    assert_eq!(domains.len(), 2, "field plus one structured Former lift");
    let instantiated_lift_type = ken_kernel::subst::subst0(&domains[1], &list_value);
    assert!(
        matches!(instantiated_lift_type, Term::Elim { fam, .. } if fam == list),
        "FormerLift is a type-level elimination over the original host"
    );

    let reduct = iota_reduct(
        &env,
        declaration,
        1,
        &[],
        &[],
        &motive,
        &methods,
        std::slice::from_ref(&list_value),
    )
    .expect("Former iota lift");
    let (_, arguments) = peel_app(&reduct);
    let lifted = arguments.last().expect("iota supplies the Former lift");
    assert!(matches!(lifted, Term::Elim { fam, .. } if *fam == list));
    if let Term::Elim {
        motive: host_motive,
        methods: host_methods,
        ..
    } = lifted
    {
        for (index, host_method) in host_methods.iter().enumerate() {
            let expected = method_type(
                &env,
                list_declaration,
                index,
                host_motive,
                std::slice::from_ref(&family_type),
                &[],
            )
            .expect("host method type");
            check(&env, &Context::new(), host_method, &expected)
                .unwrap_or_else(|error| panic!("host method {index}: {error}"));
        }
    }
    check(&env, &Context::new(), lifted, &instantiated_lift_type)
        .expect("the second host elimination inhabits the generated FormerLift");

    let attempted = declare_inductive(&mut env, |nested| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![Term::app(former(list), former(nested))],
            target_indices: vec![],
        }],
    });
    assert!(
        attempted.is_err(),
        "atomic method/iota semantics must not open D1b admission"
    );
}
