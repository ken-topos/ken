//! PX8-I interpreter/native arbitrary-precision `Int` differential.

use ken_interp::eval::{prim_reduce, EvalVal};
use ken_runtime::{
    run_example_with_seed_observation, NativeSeedEnvironment, RuntimeExample, RuntimeExpr,
    RuntimeGroundValue, RuntimeIntV1, RuntimeObservation, RuntimePartiality, RuntimePrimitive,
    RuntimeValue, Sign,
};
use num_bigint::{BigInt, Sign as NumSign};

fn runtime_int(value: &BigInt) -> RuntimeIntV1 {
    if let Ok(value) = i64::try_from(value) {
        return RuntimeIntV1::Small(value);
    }
    let (sign, limbs) = value.to_u64_digits();
    RuntimeIntV1::Big {
        sign: match sign {
            NumSign::Minus => Sign::Negative,
            NumSign::NoSign | NumSign::Plus => Sign::NonNegative,
        },
        limbs,
    }
}

#[test]
fn interpreter_and_native_agree_beyond_i128_and_on_canonical_image() {
    let operand = BigInt::from(1_u8) << 64_u32;
    let expected = &operand * &operand;
    let interpreter = prim_reduce(
        "mul_int",
        &[
            EvalVal::BigInt(operand.clone()),
            EvalVal::BigInt(operand.clone()),
        ],
    );
    assert_eq!(interpreter, EvalVal::BigInt(expected.clone()));

    let operand = runtime_int(&operand);
    let expected = runtime_int(&expected);
    let example = RuntimeExample {
        name: "px8i-interpreter-native-big-mul".to_string(),
        checked_core_shape: "exact Int multiplication beyond i128".to_string(),
        ir: RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "mul_int".to_string(),
                partiality: RuntimePartiality::Total,
            },
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Int(operand.clone())),
                RuntimeExpr::Value(RuntimeValue::Int(operand)),
            ],
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(expected)),
    };
    let native = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("native exact-Int execution succeeds");
    assert_eq!(native.observation, example.observation);
}
