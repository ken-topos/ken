//! SUB-1 acceptance: the bounded `Bytes ↔ List UInt8` structural view.

use std::collections::BTreeSet;
use std::rc::Rc;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{apply, eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId, PrimReduction, Term};

const COLLECTIONS: &str = include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const TRANSPORT: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");

fn make_store(env: &ElabEnv) -> EvalStore {
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, value) in &env.num_values {
        store
            .num_values
            .insert(*id, literal_value(value, mkdecimalpair_id));
    }
    // The historical name is retained by the interpreter, but these are the
    // polymorphic `List` constructor ids used for `List UInt8` too.
    store.list_char_ids = Some(ListCharIds {
        nil_id: env.prelude_env.nil_id,
        cons_id: env.prelude_env.cons_id,
    });
    store
}

fn literal_value(value: &NumericLitVal, mkdecimalpair_id: GlobalId) -> EvalVal {
    match value {
        NumericLitVal::Int(n) => EvalVal::from(*n),
        NumericLitVal::Float(f) => EvalVal::Float(*f),
        NumericLitVal::Float32(f) => EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
        }
        NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
    }
}

fn resync_literals(env: &ElabEnv, store: &mut EvalStore) {
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, value) in &env.num_values {
        store
            .num_values
            .entry(*id)
            .or_insert_with(|| literal_value(value, mkdecimalpair_id));
    }
}

fn eval_const(env: &ElabEnv, store: &mut EvalStore, id: GlobalId) -> EvalVal {
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        other => panic!("expected transparent constant, got {other:?}"),
    }
}

fn eval_view(
    env: &mut ElabEnv,
    store: &mut EvalStore,
    name: &str,
    ty: &str,
    body: &str,
) -> EvalVal {
    let id = env
        .elaborate_decl(&format!("const {name} : {ty} = {body}"))
        .unwrap_or_else(|error| panic!("{name} must elaborate: {error}"));
    resync_literals(env, store);
    eval_const(env, store, id)
}

fn nat_count(env: &ElabEnv, value: &EvalVal) -> usize {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_count(env, &args[0])
        }
        other => panic!("expected Nat constructor chain, got {other:?}"),
    }
}

fn list_uint8(env: &ElabEnv, bytes: &[u8]) -> EvalVal {
    let mut value = EvalVal::Ctor {
        id: env.prelude_env.nil_id,
        args: Rc::new(vec![EvalVal::Unknown]),
        slot: 0,
    };
    for byte in bytes.iter().rev() {
        value = EvalVal::Ctor {
            id: env.prelude_env.cons_id,
            args: Rc::new(vec![
                EvalVal::Unknown,
                EvalVal::Int(i64::from(*byte)),
                value,
            ]),
            slot: 0,
        };
    }
    value
}

fn list_uint8_values(env: &ElabEnv, value: &EvalVal) -> Vec<u8> {
    let mut out = Vec::new();
    let mut current = value;
    loop {
        match current {
            EvalVal::Ctor { id, .. } if *id == env.prelude_env.nil_id => return out,
            EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.cons_id => {
                match args.get(1) {
                    Some(EvalVal::Int(byte)) => out.push(u8::try_from(*byte).expect("UInt8")),
                    other => panic!("expected UInt8 list head, got {other:?}"),
                }
                current = args.get(2).expect("Cons tail");
            }
            other => panic!("expected List UInt8 constructor chain, got {other:?}"),
        }
    }
}

fn contains_const(term: &Term, needle: GlobalId) -> bool {
    match term {
        Term::Const { id, .. } => *id == needle,
        Term::Pi(a, b)
        | Term::Lam(a, b)
        | Term::Sigma(a, b)
        | Term::App(a, b)
        | Term::Pair(a, b)
        | Term::Quot(a, b)
        | Term::Absurd(a, b) => contains_const(a, needle) || contains_const(b, needle),
        Term::Eq(a, b, c) | Term::J(a, b, c) => {
            contains_const(a, needle) || contains_const(b, needle) || contains_const(c, needle)
        }
        Term::Let { ty, val, body } => {
            contains_const(ty, needle)
                || contains_const(val, needle)
                || contains_const(body, needle)
        }
        Term::Ascript(term, ty) => contains_const(term, needle) || contains_const(ty, needle),
        Term::Cast(a, b, c, d) => {
            contains_const(a, needle)
                || contains_const(b, needle)
                || contains_const(c, needle)
                || contains_const(d, needle)
        }
        Term::Proj1(term)
        | Term::Proj2(term)
        | Term::QuotClass(term)
        | Term::Trunc(term)
        | Term::TruncProj(term)
        | Term::Refl(term) => contains_const(term, needle),
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => {
            contains_const(motive, needle)
                || contains_const(method, needle)
                || contains_const(respect, needle)
                || contains_const(scrut, needle)
        }
        Term::Elim {
            params,
            motive,
            methods,
            indices,
            scrut,
            ..
        } => {
            params.iter().any(|term| contains_const(term, needle))
                || contains_const(motive, needle)
                || methods.iter().any(|term| contains_const(term, needle))
                || indices.iter().any(|term| contains_const(term, needle))
                || contains_const(scrut, needle)
        }
        _ => false,
    }
}

#[test]
fn ac2_trusted_base_delta_is_exactly_the_named_pair_and_propositions() {
    let env = ElabEnv::new().expect("base env");
    let view = &env.bytes_env;
    let expected = BTreeSet::from([
        view.bytes_to_list_id,
        view.list_to_bytes_id,
        view.bytes_list_roundtrip_id,
        view.list_bytes_roundtrip_id,
    ]);
    let actual: BTreeSet<_> = view.structural_view_trusted_delta.iter().copied().collect();
    assert_eq!(
        actual, expected,
        "SUB-1 must add exactly four named entries"
    );

    for (name, id, symbol) in [
        ("bytes_to_list", view.bytes_to_list_id, "bytes_to_list"),
        ("list_to_bytes", view.list_to_bytes_id, "list_to_bytes"),
    ] {
        assert_eq!(env.globals[name], id);
        assert!(matches!(
            env.env.lookup(id),
            Some(Decl::Primitive {
                reduction: PrimReduction::Op { symbol: actual },
                ..
            }) if *actual == symbol
        ));
    }
    for (name, id) in [
        ("bytes_list_roundtrip", view.bytes_list_roundtrip_id),
        ("list_bytes_roundtrip", view.list_bytes_roundtrip_id),
    ] {
        assert_eq!(env.globals[name], id);
        assert!(matches!(env.env.lookup(id), Some(Decl::Opaque { .. })));
    }
}

#[test]
fn ac1_ac3_structural_fold_terminates_runs_and_adds_no_axiom() {
    let mut env = ElabEnv::new().expect("base env");
    let trust_before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(TRANSPORT)
        .expect("Transport package");
    env.elaborate_ken_md_file(COLLECTIONS)
        .expect("Collections package with derived Bytes fold");
    let trust_after_package: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        trust_after_package, trust_before,
        "the derived byte surface must add zero trusted declarations"
    );

    let length_id = env.globals["length"];
    let bytes_nat_length_id = env.globals["bytes_nat_length"];
    let (_, length_body) = env
        .env
        .transparent_body(length_id)
        .expect("length must pass SCT and become transparent");
    assert!(matches!(
        length_body,
        Term::Lam(_, body) if matches!(body.as_ref(), Term::Lam(_, inner) if matches!(inner.as_ref(), Term::Elim { .. }))
    ));
    let (_, derived_body) = env
        .env
        .transparent_body(bytes_nat_length_id)
        .expect("bytes_nat_length must be an ordinary transparent definition");
    assert!(contains_const(&derived_body, length_id));
    assert!(contains_const(
        &derived_body,
        env.bytes_env.bytes_to_list_id
    ));

    let mut store = make_store(&env);
    let value = eval_view(
        &mut env,
        &mut store,
        "sub1_count",
        "Nat",
        "bytes_nat_length (bytes_encode \"a/b\")",
    );
    assert_eq!(nat_count(&env, &value), 3);
    let trust_after_call: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        trust_after_call, trust_before,
        "the structural fold call site must need no cached Nat or Axiom"
    );
}

#[test]
fn ac4_both_runtime_roundtrip_directions_are_real_and_total() {
    let env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    let bytes_to_list = eval(
        &[],
        &Term::const_(env.bytes_env.bytes_to_list_id, vec![]),
        &env.env,
        &mut store,
    );
    let list_to_bytes = eval(
        &[],
        &Term::const_(env.bytes_env.list_to_bytes_id, vec![]),
        &env.env,
        &mut store,
    );

    let bytes = vec![0, 47, 128, 255];
    let as_list = apply(
        bytes_to_list.clone(),
        EvalVal::Bytes(bytes.clone()),
        &env.env,
        &mut store,
    );
    assert_eq!(list_uint8_values(&env, &as_list), bytes);
    let back_to_bytes = apply(list_to_bytes.clone(), as_list, &env.env, &mut store);
    assert_eq!(back_to_bytes, EvalVal::Bytes(bytes));

    let independent_list = list_uint8(&env, &[1, 2, 200, 255]);
    let as_bytes = apply(
        list_to_bytes,
        independent_list.clone(),
        &env.env,
        &mut store,
    );
    assert_eq!(as_bytes, EvalVal::Bytes(vec![1, 2, 200, 255]));
    let back_to_list = apply(bytes_to_list, as_bytes, &env.env, &mut store);
    assert_eq!(
        list_uint8_values(&env, &back_to_list),
        list_uint8_values(&env, &independent_list)
    );
}

#[test]
fn ac4_roundtrip_propositions_are_usable_but_not_refl_reductions() {
    let mut env = ElabEnv::new().expect("base env");
    let trust_before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();

    env.elaborate_decl(
        "lemma use_bytes_roundtrip (bs : Bytes) : \
         Equal Bytes (list_to_bytes (bytes_to_list bs)) bs = \
         bytes_list_roundtrip bs",
    )
    .expect("registered Bytes roundtrip proposition must be usable");
    env.elaborate_decl(
        "lemma use_list_roundtrip (xs : List UInt8) : \
         Equal (List UInt8) (bytes_to_list (list_to_bytes xs)) xs = \
         list_bytes_roundtrip xs",
    )
    .expect("registered List UInt8 roundtrip proposition must be usable");

    let refl = env.elaborate_decl(
        "lemma false_refl_roundtrip (bs : Bytes) : \
         Equal Bytes (list_to_bytes (bytes_to_list bs)) bs = Refl",
    );
    assert!(
        refl.is_err(),
        "primitive operations are conversion-opaque; Refl must not close the law"
    );
    let trust_after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        trust_after, trust_before,
        "using the registered propositions must not mint consumer Axioms"
    );
}
