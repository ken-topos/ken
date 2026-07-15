//! The `IntN↔Int` conversion floor + `checked_*`/`saturating_*` DEMOTE
//! (`18a §5.7`, Phase-2 tranche #4).
//!
//! Sixteen NATIVE ops (8 widths × 2 directions) complete the conversion
//! floor `18a §5.7` marks GAP: `IntN.toInt` (widening, total) and the
//! unchecked `Int→IntN` raw cast that backs narrowing. Both reduce as
//! **identity** at the value level — every fixed-width type already shares
//! `Int`'s own `EvalVal` representation (`eval.rs`'s `fixed_binop_*`
//! helpers cast `EvalVal::Int` to the narrower Rust type and back); only the
//! KERNEL type changes across the cast, never the value. Registered as
//! `declare_primitive`/`PrimReduction::Op`, same posture as `add_int8` (F1)
//! and `leq_int` (F5) — tested-not-trusted, netted by the round-trip law
//! (AC1) and the boundary sweep (AC2), never a K3 trusted-base promotion.
//!
//! The public narrowing API (`intToIntN : Int → Option IntN`) and the
//! `checkedAdd/Sub/MulIntN`/`saturatingAdd/Sub/MulIntN` families are
//! **derived** Ken `view`s over the floor (`N/A` oracle-ref, `18a §5`
//! ~L439-440) — no new native surface, reusing `leq_int`/`add_int`/
//! `sub_int`/`mul_int`/`and_bool`/`Option`/`Some`/`None`, matching
//! `decimal_char.rs`'s native-primitive + Ken-fn split (`intToChar`).
//!
//! `neg_intN` (signed widths only) stays NATIVE and checked — it does NOT
//! demote to `sub_int 0 x` like bignum `neg_int`, since `neg(MIN_intN)`
//! overflows the asymmetric two's-complement range (`18a §5`, ~L256). Its
//! `prim_reduce` arms live in `eval.rs` alongside F2's checked fixed-width
//! arithmetic; this module only registers the primitive's kernel signature.
//!
//! Called from `prelude::register_prelude`, after `register_numeric_env`
//! (needs the 8 `IntN`/`UIntN` type ids) and after the prelude's `Option`
//! inductive is declared.

use std::collections::{BTreeSet, HashMap};

use ken_kernel::env::PrimReduction;
use ken_kernel::{declare_postulate, declare_primitive, GlobalEnv, GlobalId, Term};
use num_bigint::BigInt;

use crate::error::ElabError;
use crate::ElabEnv;

/// One `IntN`/`UIntN` width's registration inputs: its Ken type name, the
/// snake_case stem used in native primitive symbols, and Ken-source
/// expressions for its exact `[T_MIN, T_MAX]` bounds. Bounds are built via
/// `sub_int 0 <lit>` rather than a negative literal — this grammar has no
/// unary negation (the `Decimal`/`Char` demote's established workaround).
struct WidthSpec {
    name: &'static str,
    snake: &'static str,
    min_expr: &'static str,
    max_expr: &'static str,
    signed: bool,
}

fn width_specs() -> [WidthSpec; 8] {
    [
        WidthSpec { name: "Int8",   snake: "int8",   min_expr: "(sub_int 0 128)",                  max_expr: "127",                  signed: true },
        WidthSpec { name: "Int16",  snake: "int16",  min_expr: "(sub_int 0 32768)",                 max_expr: "32767",                signed: true },
        WidthSpec { name: "Int32",  snake: "int32",  min_expr: "(sub_int 0 2147483648)",            max_expr: "2147483647",           signed: true },
        WidthSpec { name: "Int64",  snake: "int64",  min_expr: "(sub_int 0 9223372036854775808)",   max_expr: "9223372036854775807",  signed: true },
        WidthSpec { name: "UInt8",  snake: "uint8",  min_expr: "0",                                 max_expr: "255",                  signed: false },
        WidthSpec { name: "UInt16", snake: "uint16", min_expr: "0",                                 max_expr: "65535",                signed: false },
        WidthSpec { name: "UInt32", snake: "uint32", min_expr: "0",                                 max_expr: "4294967295",           signed: false },
        WidthSpec { name: "UInt64", snake: "uint64", min_expr: "0",                                 max_expr: "18446744073709551615", signed: false },
    ]
}

struct AbiScalarSpec {
    name: &'static str,
    snake: &'static str,
    min_expr: String,
    max_expr: String,
}

fn manifested_width(name: &str) -> Result<u32, ElabError> {
    if ken_host::TARGET_ABI.backend != "linux_raw" {
        return Err(ElabError::Internal(format!(
            "PX3 ABI scalar conversions are unavailable for target ABI backend {:?}",
            ken_host::TARGET_ABI.backend
        )));
    }
    let width = ken_host::TARGET_ABI
        .facts
        .iter()
        .find(|fact| fact.name == name)
        .ok_or_else(|| ElabError::Internal(format!("target ABI manifest lacks {name}")))?
        .value;
    let width = u32::try_from(width)
        .map_err(|_| ElabError::Internal(format!("target ABI width {name} is too large")))?;
    if width == 0 {
        return Err(ElabError::Internal(format!(
            "target ABI width {name} must be nonzero"
        )));
    }
    Ok(width)
}

fn bound_exprs(width: u32, signed: bool) -> (String, String) {
    if signed {
        let magnitude = BigInt::from(1u8) << (width - 1);
        let max = &magnitude - 1u8;
        (format!("(sub_int 0 {magnitude})"), max.to_string())
    } else {
        let max = (BigInt::from(1u8) << width) - 1u8;
        ("0".to_owned(), max.to_string())
    }
}

fn abi_scalar_specs() -> Result<Vec<AbiScalarSpec>, ElabError> {
    [
        ("USize", "usize", "POINTER_WIDTH", false),
        ("ISize", "isize", "POINTER_WIDTH", true),
        ("CInt", "cint", "C_INT_WIDTH", true),
    ]
    .into_iter()
    .map(|(name, snake, width_fact, signed)| {
        let (min_expr, max_expr) = bound_exprs(manifested_width(width_fact)?, signed);
        Ok(AbiScalarSpec { name, snake, min_expr, max_expr })
    })
    .collect()
}

/// Register a native unary op `A → B` (mirrors `numbers.rs`'s inline
/// `not_bool` registration — no existing shared unary-op macro).
fn reg_unop(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    name: &str,
    from_id: GlobalId,
    to_id: GlobalId,
) -> Result<GlobalId, ElabError> {
    let op_ty = Term::pi(Term::const_(from_id, vec![]), Term::const_(to_id, vec![]));
    let id = declare_primitive(env, vec![], op_ty, PrimReduction::Op { symbol: name.to_string().leak() })
        .map_err(|e| ElabError::Internal(format!("prim {} failed: {}", name, e)))?;
    globals.insert(name.to_string(), id);
    Ok(id)
}

/// Register the `IntN↔Int` conversion floor + its derived `checked_*`/
/// `saturating_*` family (`18a §5.7`).
pub fn register_conversions(elab: &mut ElabEnv) -> Result<(), ElabError> {
    let int_id = elab.numeric_env.int_id;

    for spec in width_specs() {
        let ty_id = elab
            .numeric_env
            .id_for(spec.name)
            .ok_or_else(|| ElabError::Internal(format!("conversions: {} not registered", spec.name)))?;

        // ── native floor: widening (total) + narrowing raw cast (unchecked) ──
        let widen_name = format!("{}_to_int", spec.snake);
        let widen_id = reg_unop(
            &mut elab.env,
            &mut elab.globals,
            &widen_name,
            ty_id,
            int_id,
        )?;

        let narrow_raw_name = format!("int_to_{}_raw", spec.snake);
        let narrow_raw_id = reg_unop(
            &mut elab.env,
            &mut elab.globals,
            &narrow_raw_name,
            int_id,
            ty_id,
        )?;

        // SUB-1b: `UInt8` is opaque, so its conversion retraction cannot be
        // proved by structural elimination. Register exactly this one
        // conversion-layer proposition; every lawful equality consumer is
        // derived in ordinary Ken from it.
        if spec.name == "UInt8" {
            let trusted_before: BTreeSet<_> =
                elab.env.trusted_base().into_iter().collect();
            let uint8_t = Term::const_(ty_id, vec![]);
            let widen = Term::const_(widen_id, vec![]);
            let narrow = Term::const_(narrow_raw_id, vec![]);
            let retract_ty = Term::pi(
                uint8_t.clone(),
                Term::Eq(
                    Box::new(uint8_t),
                    Box::new(Term::app(
                        narrow,
                        Term::app(widen, Term::var(0)),
                    )),
                    Box::new(Term::var(0)),
                ),
            );
            let retract_id = declare_postulate(
                &mut elab.env,
                "uint8_int_retract".to_string(),
                vec![],
                retract_ty,
            )
                .map_err(|e| {
                    ElabError::Internal(format!("uint8_int_retract failed: {e}"))
                })?;
            elab.globals
                .insert("uint8_int_retract".to_string(), retract_id);

            let trusted_after: BTreeSet<_> =
                elab.env.trusted_base().into_iter().collect();
            let trusted_delta: Vec<_> = trusted_after
                .difference(&trusted_before)
                .copied()
                .collect();
            let actual_delta: BTreeSet<_> = trusted_delta.iter().copied().collect();
            let expected_delta = BTreeSet::from([retract_id]);
            if actual_delta != expected_delta {
                return Err(ElabError::Internal(format!(
                    "SUB-1b trusted-base delta must be exactly uint8_int_retract: expected {expected_delta:?}, got {actual_delta:?}"
                )));
            }
            elab.numeric_env.uint8_int_retract_id = retract_id;
            elab.numeric_env.uint8_retract_trusted_delta = trusted_delta;
        }

        // ── neg_intN (signed only): NATIVE, checked, does not demote ────────
        if spec.signed {
            let neg_name = format!("neg_{}", spec.snake);
            reg_unop(&mut elab.env, &mut elab.globals, &neg_name, ty_id, ty_id)?;
        }

        // ── derived: public narrowing `Int → Option IntN` ───────────────────
        // Native Bool-check (`leq_int`, already-audited F5) + a Ken view
        // constructing the `Option` — the `intToChar` pattern (`decimal_char.rs`).
        let int_to_n_src = format!(
            "fn intTo{name} (n : Int) : Option {name} = \
             match (and_bool (leq_int {min} n) (leq_int n {max})) {{ \
               True |-> Some {name} (int_to_{snake}_raw n) ; \
               False |-> None {name} \
             }}",
            name = spec.name,
            snake = spec.snake,
            min = spec.min_expr,
            max = spec.max_expr,
        );
        elab.elaborate_decl(&int_to_n_src)
            .map_err(|e| ElabError::Internal(format!("intTo{} failed: {}", spec.name, e)))?;

        // ── derived: checked_add/sub/mul (`T → T → Option T`) ───────────────
        // `checked_add_intN a b := Int.toIntN (add_int (IntN.toInt a) (IntN.toInt b))`
        // (`18a §5`, ~L439) — the narrowing `None` IS the overflow semantics.
        for (op_label, int_op) in [("Add", "add_int"), ("Sub", "sub_int"), ("Mul", "mul_int")] {
            let src = format!(
                "fn checked{op}{name} (a : {name}) (b : {name}) : Option {name} = \
                 intTo{name} ({int_op} ({snake}_to_int a) ({snake}_to_int b))",
                op = op_label,
                name = spec.name,
                int_op = int_op,
                snake = spec.snake,
            );
            elab.elaborate_decl(&src)
                .map_err(|e| ElabError::Internal(format!("checked{}{} failed: {}", op_label, spec.name, e)))?;
        }

        // ── derived: saturating_add/sub/mul (`T → T → T`, clamps) ───────────
        // Widen → clamp-compare (`leq_int` against `T_MIN`/`T_MAX`) → narrow
        // via the raw cast (safe: the clamp already guarantees range
        // membership, so the raw cast is never applied out-of-bounds). The
        // widened sum is INLINED (repeated 3x) rather than `let`-bound: a
        // `let`-bound match whose arms return via a function application
        // (here `int_to_{snake}_raw <expr>`) mis-infers its type when the
        // enclosing context has an extra `let` binder — reproduced with a
        // minimal case (`smoke_let_match_no_s_reference`, scratch, not
        // committed) that fails identically WITHOUT ever referencing the
        // bound variable, so the extra binder depth itself is the trigger,
        // not this WP's specific expressions. A pre-existing elaborator gap
        // (`infer_match`'s constant-motive path under nonzero context
        // depth via `RLet`'s `check`-fallback path), not a soundness hole —
        // inlining is a correct, verified-equivalent avoidance, not a
        // guess. Flagged forward to Architect/runtime-leader as a named
        // elaborator finding; not fixed here (out of this WP's scope, no
        // kernel/trust-level impact either way).
        for (op_label, int_op) in [("Add", "add_int"), ("Sub", "sub_int"), ("Mul", "mul_int")] {
            let sum = format!("({int_op} ({snake}_to_int a) ({snake}_to_int b))", int_op = int_op, snake = spec.snake);
            let src = format!(
                "fn saturating{op}{name} (a : {name}) (b : {name}) : {name} = \
                 match (leq_int {sum} {max}) {{ \
                   True |-> match (leq_int {min} {sum}) {{ \
                     True |-> int_to_{snake}_raw {sum} ; \
                     False |-> int_to_{snake}_raw {min} \
                   }} ; \
                   False |-> int_to_{snake}_raw {max} \
                 }}",
                op = op_label,
                name = spec.name,
                sum = sum,
                snake = spec.snake,
                min = spec.min_expr,
                max = spec.max_expr,
            );
            elab.elaborate_decl(&src)
                .map_err(|e| ElabError::Internal(format!("saturating{}{} failed: {}", op_label, spec.name, e)))?;
        }
    }

    // PX3: actionable failure payload for every ABI-scalar narrowing. The
    // target name and manifested bounds travel with the offending value.
    elab.elaborate_decl("data RangeError = MkRangeError String Int Int Int")
        .map_err(|e| ElabError::Internal(format!("RangeError failed: {e}")))?;

    let trusted_before: BTreeSet<_> = elab.env.trusted_base().into_iter().collect();
    let mut retract_ids = Vec::new();
    let mut conversion_ids = Vec::new();

    for spec in abi_scalar_specs()? {
        let ty_id = elab.numeric_env.id_for(spec.name).ok_or_else(|| {
            ElabError::Internal(format!("conversions: {} not registered", spec.name))
        })?;

        let widen_name = format!("{}_to_int", spec.snake);
        let widen_id = reg_unop(
            &mut elab.env, &mut elab.globals, &widen_name, ty_id, int_id,
        )?;
        conversion_ids.push(widen_id);
        let narrow_raw_name = format!("int_to_{}_raw", spec.snake);
        let narrow_raw_id = reg_unop(
            &mut elab.env, &mut elab.globals, &narrow_raw_name, int_id, ty_id,
        )?;
        conversion_ids.push(narrow_raw_id);

        let scalar_t = Term::const_(ty_id, vec![]);
        let retract_ty = Term::pi(
            scalar_t.clone(),
            Term::Eq(
                Box::new(scalar_t),
                Box::new(Term::app(
                    Term::const_(narrow_raw_id, vec![]),
                    Term::app(Term::const_(widen_id, vec![]), Term::var(0)),
                )),
                Box::new(Term::var(0)),
            ),
        );
        let retract_name = format!("{}_int_retract", spec.snake);
        let retract_id = declare_postulate(
            &mut elab.env, retract_name.clone(), vec![], retract_ty,
        )
        .map_err(|e| ElabError::Internal(format!("{retract_name} failed: {e}")))?;
        elab.globals.insert(retract_name, retract_id);
        retract_ids.push(retract_id);
        conversion_ids.push(retract_id);

        let src = format!(
            "fn intTo{name} (n : Int) : Result RangeError {name} = \
             match (and_bool (leq_int {min} n) (leq_int n {max})) {{ \
               True |-> Ok RangeError {name} (int_to_{snake}_raw n) ; \
               False |-> Err RangeError {name} (MkRangeError \"{name}\" n {min} {max}) \
             }}",
            name = spec.name,
            snake = spec.snake,
            min = spec.min_expr,
            max = spec.max_expr,
        );
        elab.elaborate_decl(&src)
            .map_err(|e| ElabError::Internal(format!("intTo{} failed: {e}", spec.name)))?;

        // The primitive remains in Σ for the already-elaborated wrapper and
        // retract, but has no surface spelling: unchecked narrowing is not a
        // public API.
        elab.globals.remove(&narrow_raw_name);
    }

    let trusted_after: BTreeSet<_> = elab.env.trusted_base().into_iter().collect();
    let trusted_delta: Vec<_> = trusted_after
        .difference(&trusted_before)
        .copied()
        .collect();
    let actual_delta: BTreeSet<_> = trusted_delta.iter().copied().collect();
    let expected_delta: BTreeSet<_> = conversion_ids.iter().copied().collect();
    if actual_delta != expected_delta {
        return Err(ElabError::Internal(format!(
            "PX3 ABI scalar conversion delta must be exactly six native floors plus three retracts: expected {expected_delta:?}, got {actual_delta:?}"
        )));
    }
    elab.numeric_env.abi_scalar_retract_ids = retract_ids;
    elab.numeric_env.abi_scalar_conversion_trusted_delta = trusted_delta;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::bound_exprs;

    #[test]
    fn manifested_width_changes_the_generated_bounds() {
        assert_ne!(bound_exprs(32, false), bound_exprs(64, false));
        assert_ne!(bound_exprs(32, true), bound_exprs(64, true));
        assert_eq!(bound_exprs(8, false), ("0".into(), "255".into()));
        assert_eq!(
            bound_exprs(8, true),
            ("(sub_int 0 128)".into(), "127".into())
        );
    }
}
