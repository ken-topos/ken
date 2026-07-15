//! PX3 acceptance: target-manifest-bound machine/ABI scalars.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, ElabError, NumericLitVal};
use ken_interp::eval::{eval, prim_reduce, EvalStore, EvalVal};
use ken_kernel::env::PrimReduction;
use ken_kernel::{check, Context, Decl, Term};
use num_bigint::BigInt;

fn width(name: &str) -> u32 {
    u32::try_from(
        ken_host::TARGET_ABI
            .facts
            .iter()
            .find(|fact| fact.name == name)
            .unwrap_or_else(|| panic!("missing manifested {name}"))
            .value,
    )
    .expect("manifested width fits u32")
}

fn max_for(width: u32, signed: bool) -> BigInt {
    if signed {
        (BigInt::from(1u8) << (width - 1)) - 1u8
    } else {
        (BigInt::from(1u8) << width) - 1u8
    }
}

fn eval_const(env: &ElabEnv, id: ken_kernel::GlobalId) -> EvalVal {
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (literal_id, value) in &env.num_values {
        let value = match value {
            NumericLitVal::Int(value) => EvalVal::from(*value),
            NumericLitVal::Float(value) => EvalVal::Float(*value),
            NumericLitVal::Float32(value) => EvalVal::Float32(*value),
            NumericLitVal::Decimal { coeff, exp } => {
                ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
            }
            NumericLitVal::Str(value) => EvalVal::Str(value.clone()),
        };
        store.num_values.insert(*literal_id, value);
    }
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, &mut store),
        other => panic!("expected transparent constant, got {other:?}"),
    }
}

#[test]
fn ac2_types_are_distinct_nominal_opaque_primitives() {
    let mut env = ElabEnv::new().expect("PX3 prelude");
    let ids = [
        env.numeric_env.usize_id,
        env.numeric_env.isize_id,
        env.numeric_env.cint_id,
    ];
    assert_eq!(ids.into_iter().collect::<BTreeSet<_>>().len(), 3);
    for (name, id) in [("USize", ids[0]), ("ISize", ids[1]), ("CInt", ids[2])] {
        assert_eq!(env.globals[name], id);
        assert!(matches!(
            env.env.lookup(id),
            Some(Decl::Primitive {
                reduction: PrimReduction::OpaqueType,
                ..
            })
        ));
    }
    for (abi_id, fixed_id) in [
        (env.numeric_env.usize_id, env.numeric_env.uint64_id),
        (env.numeric_env.isize_id, env.numeric_env.int64_id),
        (env.numeric_env.cint_id, env.numeric_env.int32_id),
    ] {
        assert_ne!(abi_id, fixed_id, "ABI scalars must not alias fixed widths");
    }

    let result = env.elaborate_decl("fn reject_abi_mix (x : USize) : ISize = x");
    assert!(
        matches!(
            result,
            Err(ElabError::TypeMismatch { .. }) | Err(ElabError::KernelRejected { .. })
        ),
        "nominally distinct ABI scalars must reject cross-use, got {result:?}"
    );
    let fixed = env.elaborate_decl("fn reject_fixed_mix (x : USize) : UInt64 = x");
    assert!(
        matches!(
            fixed,
            Err(ElabError::TypeMismatch { .. }) | Err(ElabError::KernelRejected { .. })
        ),
        "ABI scalars must reject fixed-width cross-use, got {fixed:?}"
    );
}

#[test]
fn ac3_manifest_max_is_ok_and_max_plus_one_is_err_for_every_scalar() {
    let seed_env = ElabEnv::new().expect("PX3 prelude");
    for target in ["USize", "ISize", "CInt"] {
        assert!(
            seed_env
                .num_values
                .values()
                .any(|value| matches!(value, NumericLitVal::Str(name) if name == target)),
            "the actionable RangeError payload must retain its {target} string seed"
        );
    }

    for (name, width_fact, signed) in [
        ("USize", "POINTER_WIDTH", false),
        ("ISize", "POINTER_WIDTH", true),
        ("CInt", "C_INT_WIDTH", true),
    ] {
        let max = max_for(width(width_fact), signed);

        let mut env = ElabEnv::new().expect("PX3 prelude");
        let ok = env
            .elaborate_decl_v1(&format!(
                "const px3_ok : Result RangeError {name} = intTo{name} {max}"
            ))
            .expect("manifest-derived max elaborates");
        let ok_value = eval_const(&env, ok.def_id);
        assert!(
            matches!(&ok_value, EvalVal::Ctor { id, .. } if *id == env.globals["Ok"]),
            "{name} manifested max must be Ok, got {ok_value:?}"
        );

        let err = env
            .elaborate_decl_v1(&format!(
                "const px3_err : Result RangeError {name} = intTo{name} (add_int {max} 1)"
            ))
            .expect("manifest-derived max+1 elaborates");
        let err_value = eval_const(&env, err.def_id);
        let payload = match &err_value {
            EvalVal::Ctor { id, args, .. } if *id == env.globals["Err"] => {
                args.last().expect("Err carries RangeError")
            }
            other => panic!("{name} manifested max+1 must be Err, got {other:?}"),
        };
        match payload {
            EvalVal::Ctor { id, args, .. } if *id == env.globals["MkRangeError"] => {
                assert!(
                    matches!(args.first(), Some(EvalVal::Str(target)) if target == name),
                    "RangeError must name {name}, got {args:?}"
                );
            }
            other => panic!("expected actionable MkRangeError, got {other:?}"),
        }
    }
}

#[test]
fn ac3_native_widening_and_private_raw_floors_preserve_boundary_values() {
    for (snake, width_fact, signed) in [
        ("usize", "POINTER_WIDTH", false),
        ("isize", "POINTER_WIDTH", true),
        ("cint", "C_INT_WIDTH", true),
    ] {
        let boundary = EvalVal::BigInt(max_for(width(width_fact), signed));
        assert_eq!(
            prim_reduce(&format!("{snake}_to_int"), std::slice::from_ref(&boundary)),
            boundary,
            "{snake} widening must be total at the manifested max"
        );
        assert_eq!(
            prim_reduce(
                &format!("int_to_{snake}_raw"),
                std::slice::from_ref(&boundary),
            ),
            boundary,
            "{snake} raw floor must preserve the checked wrapper's value"
        );
    }
}

#[test]
fn ac4_trusted_delta_is_the_exact_named_twelve_entries() {
    let env = ElabEnv::new().expect("PX3 prelude");
    let type_ids = BTreeSet::from([
        env.numeric_env.usize_id,
        env.numeric_env.isize_id,
        env.numeric_env.cint_id,
    ]);
    let actual_type_ids = env
        .numeric_env
        .abi_scalar_type_trusted_delta
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_type_ids, type_ids);

    let conversion_ids = env
        .numeric_env
        .abi_scalar_conversion_trusted_delta
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(conversion_ids.len(), 9);
    let expected_symbols = BTreeSet::from([
        "usize_to_int",
        "int_to_usize_raw",
        "isize_to_int",
        "int_to_isize_raw",
        "cint_to_int",
        "int_to_cint_raw",
    ]);
    let actual_symbols = conversion_ids
        .iter()
        .filter_map(|id| match env.env.lookup(*id) {
            Some(Decl::Primitive {
                reduction: PrimReduction::Op { symbol },
                ..
            }) => Some(*symbol),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_symbols, expected_symbols);

    let retract_names = env
        .numeric_env
        .abi_scalar_retract_ids
        .iter()
        .map(|id| match env.env.lookup(*id) {
            Some(Decl::Opaque { name, .. }) => name.as_str(),
            other => panic!("retract must be a named opaque declaration, got {other:?}"),
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        retract_names,
        BTreeSet::from(["usize_int_retract", "isize_int_retract", "cint_int_retract",])
    );
    assert_eq!(
        conversion_ids
            .union(&type_ids)
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        12
    );
}

#[test]
fn ac3_raw_narrowing_is_private_and_ac5_refl_stays_rejected() {
    let mut env = ElabEnv::new().expect("PX3 prelude");
    for (name, snake) in [("USize", "usize"), ("ISize", "isize"), ("CInt", "cint")] {
        assert!(!env.globals.contains_key(&format!("int_to_{snake}_raw")));
        let raw = env.elaborate_decl(&format!(
            "fn px3_bad_raw (n : Int) : {name} = int_to_{snake}_raw n"
        ));
        assert!(
            matches!(raw, Err(ElabError::UnresolvedCon { .. })),
            "unchecked {name} narrowing must be unavailable, got {raw:?}"
        );

        let retract_id = env.globals[&format!("{snake}_int_retract")];
        assert!(env.env.trusted_base().contains(&retract_id));
        let retract_ty = match env.env.lookup(retract_id) {
            Some(Decl::Opaque { ty, .. }) => ty.clone(),
            other => panic!("expected named retract postulate, got {other:?}"),
        };
        let domain = match &retract_ty {
            Term::Pi(domain, _) => *domain.clone(),
            other => panic!("expected retract Pi type, got {other:?}"),
        };
        let refl_candidate = Term::lam(domain, Term::Refl(Box::new(Term::var(0))));
        assert!(
            check(&env.env, &Context::new(), &refl_candidate, &retract_ty).is_err(),
            "{name} primitive conversions must remain opaque to kernel Refl"
        );

        env.elaborate_decl(&format!(
            "lemma px3_{snake}_ordinary_refl (x : {name}) : Equal {name} x x = Refl"
        ))
        .expect("ordinary Refl control must remain valid");
    }
}
