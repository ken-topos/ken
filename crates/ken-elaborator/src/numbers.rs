//! Numeric tower registration and obligation infrastructure (`35 §2`, `35 §3`).
//!
//! Registers all numeric types and ops in the kernel global env, emits
//! no-overflow obligations for fixed-width arithmetic, and provides the
//! type-directed dispatch table used by the elaborator.
//!
//! Types: `Int` (arbitrary-precision), `Int8`…`Int64`, `UInt8`…`UInt64`,
//! `Decimal`, `Float`, `Float32`, `Bool`, `Char`.
//! All are primitive opaque types (`PrimReduction::OpaqueType`).
//!
//! Ops: registered as `PrimReduction::Op { symbol }` with matching entries
//! in `ken-interp`'s `prim_reduce`. The symbol names are the stable interface.

use std::collections::HashMap;

use ken_kernel::{declare_postulate, declare_primitive, GlobalEnv, GlobalId, Level, Term};
use ken_kernel::env::PrimReduction;

use crate::error::ElabError;

// ── numeric literal value (for side-table; independent of ken-interp) ─────────

/// The concrete value of a numeric literal (`35 §4.1`).
///
/// Stored in `ElabEnv.num_values` keyed by the opaque-postulate `GlobalId`
/// that represents the literal in the kernel. Tests convert to `EvalVal`
/// via the bridge in `ken-interp/tests/`.
#[derive(Clone, Debug)]
pub enum NumericLitVal {
    Int(i128),
    Float(f64),
    Float32(f32),
    Decimal { coeff: i64, exp: i32 },
    /// NFC-normalized UTF-8 string literal (`37 §2.1`, VAL1-surface).
    Str(String),
}

// ── dispatch entries ────────────────────────────────────────────────────────

/// Dispatch record for a type-directed `+` / `*` operation.
#[derive(Clone, Debug)]
pub struct AddEntry {
    /// GlobalId of the obligation-generating (bare `+`) op.
    pub op_id: GlobalId,
    /// GlobalId of the explicit wrapping op (`+%`); `None` for total ops.
    pub wrapping_id: Option<GlobalId>,
    /// GlobalId of the no-overflow proposition; `None` for total ops.
    pub no_ovf_id: Option<GlobalId>,
    /// GlobalId of the result type.
    pub result_id: GlobalId,
}

/// Dispatch record for a type-directed `==` operation.
#[derive(Clone, Debug)]
pub struct EqEntry {
    /// GlobalId of the equality op.
    pub op_id: GlobalId,
}

// ── NumericEnv ─────────────────────────────────────────────────────────────

/// All GlobalIds and dispatch tables for the numeric tower (`35 §2`).
pub struct NumericEnv {
    // --- type ids (for infer/check dispatch) ---
    pub int_id:     GlobalId,
    pub int8_id:    GlobalId,
    pub int16_id:   GlobalId,
    pub int32_id:   GlobalId,
    pub int64_id:   GlobalId,
    pub uint8_id:   GlobalId,
    pub uint16_id:  GlobalId,
    pub uint32_id:  GlobalId,
    pub uint64_id:  GlobalId,
    pub decimal_id: GlobalId,
    pub float_id:   GlobalId,
    pub float32_id: GlobalId,
    pub bool_id:    GlobalId,
    pub char_id:    GlobalId,

    // --- `+` dispatch table (keyed by the type's GlobalId) ---
    add_table: HashMap<GlobalId, AddEntry>,

    // --- `==` dispatch table (keyed by the type's GlobalId) ---
    eq_table: HashMap<GlobalId, EqEntry>,
}

impl NumericEnv {
    /// Look up the dispatch entry for a type-directed `+` on the given type.
    pub fn classify_add(&self, ty: &Term) -> Option<&AddEntry> {
        match ty {
            Term::Const { id, .. } => self.add_table.get(id),
            _ => None,
        }
    }

    /// Look up the dispatch entry for a type-directed `==` on the given type.
    pub fn classify_eq(&self, ty: &Term) -> Option<&EqEntry> {
        match ty {
            Term::Const { id, .. } => self.eq_table.get(id),
            _ => None,
        }
    }

    /// Return the kernel `Term::Const` for a type by name.
    pub fn ty_const(&self, name: &str) -> Option<Term> {
        let id = self.id_for(name)?;
        Some(Term::const_(id, vec![]))
    }

    /// Numeric type GlobalId by surface name.
    pub fn id_for(&self, name: &str) -> Option<GlobalId> {
        match name {
            "Int"     => Some(self.int_id),
            "Int8"    => Some(self.int8_id),
            "Int16"   => Some(self.int16_id),
            "Int32"   => Some(self.int32_id),
            "Int64"   => Some(self.int64_id),
            "UInt8"   => Some(self.uint8_id),
            "UInt16"  => Some(self.uint16_id),
            "UInt32"  => Some(self.uint32_id),
            "UInt64"  => Some(self.uint64_id),
            "Decimal" => Some(self.decimal_id),
            "Float"   => Some(self.float_id),
            "Float32" => Some(self.float32_id),
            "Bool"    => Some(self.bool_id),
            "Char"    => Some(self.char_id),
            _         => None,
        }
    }
}

// ── registration ───────────────────────────────────────────────────────────

/// Register the full numeric tower in `env` and `globals` (`35 §2`, `14 §5`).
///
/// Returns the `NumericEnv` with all type and op GlobalIds populated.
/// On error returns `ElabError::Internal`.
pub fn register_numeric_env(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
) -> Result<NumericEnv, ElabError> {
    let type0 = Term::ty(Level::Zero);
    let omega0 = Term::omega(Level::Zero);

    // ── register an opaque primitive type : Type 0 ─────────────────────────
    macro_rules! reg_ty {
        ($name:expr) => {{
            // Reuse the GlobalId if already registered (e.g. Bool from ElabEnv::new).
            if let Some(&existing_id) = globals.get($name) {
                existing_id
            } else {
                let id = declare_primitive(env, vec![], type0.clone(), PrimReduction::OpaqueType)
                    .map_err(|e| ElabError::Internal(format!("prim {} failed: {}", $name, e)))?;
                globals.insert($name.to_string(), id);
                id
            }
        }};
    }

    // ── register a binary op `A → A → A` ───────────────────────────────────
    macro_rules! reg_binop {
        ($name:expr, $ty_id:expr) => {{
            let ty_const = Term::const_($ty_id, vec![]);
            let op_ty = Term::pi(
                ty_const.clone(),
                Term::pi(ty_const.clone(), ty_const.clone()),
            );
            let id = declare_primitive(env, vec![], op_ty, PrimReduction::Op { symbol: $name })
                .map_err(|e| ElabError::Internal(format!("prim {} failed: {}", $name, e)))?;
            globals.insert($name.to_string(), id);
            id
        }};
    }

    // ── register a binary op `A → A → Bool` ────────────────────────────────
    macro_rules! reg_cmpop {
        ($name:expr, $ty_id:expr, $bool_id:expr) => {{
            let ty_const = Term::const_($ty_id, vec![]);
            let bool_const = Term::indformer($bool_id, vec![]);
            let op_ty = Term::pi(
                ty_const.clone(),
                Term::pi(ty_const.clone(), bool_const.clone()),
            );
            let id = declare_primitive(env, vec![], op_ty, PrimReduction::Op { symbol: $name })
                .map_err(|e| ElabError::Internal(format!("prim {} failed: {}", $name, e)))?;
            globals.insert($name.to_string(), id);
            id
        }};
    }

    // ── register a no-overflow proposition `A → A → Ω₀` ───────────────────
    macro_rules! reg_novf {
        ($name:expr, $ty_id:expr) => {{
            let ty_const = Term::const_($ty_id, vec![]);
            let novf_ty = Term::pi(
                ty_const.clone(),
                Term::pi(ty_const.clone(), omega0.clone()),
            );
            // Opaque proposition — the prover (V3) discharges it statically.
            let id = declare_postulate(env, vec![], novf_ty)
                .map_err(|e| ElabError::Internal(format!("no-ovf {} failed: {}", $name, e)))?;
            globals.insert($name.to_string(), id);
            id
        }};
    }

    // ---- types ----
    let int_id     = reg_ty!("Int");
    let int8_id    = reg_ty!("Int8");
    let int16_id   = reg_ty!("Int16");
    let int32_id   = reg_ty!("Int32");
    let int64_id   = reg_ty!("Int64");
    let uint8_id   = reg_ty!("UInt8");
    let uint16_id  = reg_ty!("UInt16");
    let uint32_id  = reg_ty!("UInt32");
    let uint64_id  = reg_ty!("UInt64");
    let decimal_id = reg_ty!("Decimal");
    let float_id   = reg_ty!("Float");
    let float32_id = reg_ty!("Float32");
    let bool_id    = reg_ty!("Bool");
    let char_id    = reg_ty!("Char");

    // ---- Int ops (total, no obligation) ----
    let add_int_id = reg_binop!("add_int", int_id);
    let sub_int_id = reg_binop!("sub_int", int_id);
    let mul_int_id = reg_binop!("mul_int", int_id);
    let eq_int_id  = reg_cmpop!("eq_int", int_id, bool_id);
    // `Int`'s ordering comparison (`30-taxonomy.md §4`'s "comparison
    // primitives `Int → Int → Bool`" — plural, already assumed to justify
    // `Bool`'s prelude membership — but only `eq_int` had actually been
    // wired; `leq_int` completes it). ES4-classes needs it to wrap `Ord
    // Int`'s `leq` operation field (`51-lawful-classes.md §6`).
    let leq_int_id = reg_cmpop!("leq_int", int_id, bool_id);
    let _ = (sub_int_id, mul_int_id, leq_int_id);

    // ---- Int8 ops ----
    let add_int8_id = reg_binop!("add_int8", int8_id);
    let wrap_add_int8_id = reg_binop!("wrapping_add_int8", int8_id);
    let novf_add_int8_id = reg_novf!("NoOvfAddInt8", int8_id);

    // ---- Int16 ops ----
    let add_int16_id = reg_binop!("add_int16", int16_id);
    let wrap_add_int16_id = reg_binop!("wrapping_add_int16", int16_id);
    let novf_add_int16_id = reg_novf!("NoOvfAddInt16", int16_id);

    // ---- Int32 ops ----
    let add_int32_id = reg_binop!("add_int32", int32_id);
    let wrap_add_int32_id = reg_binop!("wrapping_add_int32", int32_id);
    let novf_add_int32_id = reg_novf!("NoOvfAddInt32", int32_id);

    // ---- Int64 ops ----
    let add_int64_id = reg_binop!("add_int64", int64_id);
    let wrap_add_int64_id = reg_binop!("wrapping_add_int64", int64_id);
    let novf_add_int64_id = reg_novf!("NoOvfAddInt64", int64_id);

    // ---- UInt8 ops ----
    let add_uint8_id = reg_binop!("add_uint8", uint8_id);
    let wrap_add_uint8_id = reg_binop!("wrapping_add_uint8", uint8_id);
    let novf_add_uint8_id = reg_novf!("NoOvfAddUInt8", uint8_id);

    // ---- UInt16 ops ----
    let add_uint16_id = reg_binop!("add_uint16", uint16_id);
    let wrap_add_uint16_id = reg_binop!("wrapping_add_uint16", uint16_id);
    let novf_add_uint16_id = reg_novf!("NoOvfAddUInt16", uint16_id);

    // ---- UInt32 ops ----
    let add_uint32_id = reg_binop!("add_uint32", uint32_id);
    let wrap_add_uint32_id = reg_binop!("wrapping_add_uint32", uint32_id);
    let novf_add_uint32_id = reg_novf!("NoOvfAddUInt32", uint32_id);

    // ---- UInt64 ops ----
    let add_uint64_id = reg_binop!("add_uint64", uint64_id);
    let wrap_add_uint64_id = reg_binop!("wrapping_add_uint64", uint64_id);
    let novf_add_uint64_id = reg_novf!("NoOvfAddUInt64", uint64_id);

    // ---- Decimal ops (exact) ----
    let add_decimal_id = reg_binop!("add_decimal", decimal_id);
    let _ = reg_binop!("sub_decimal", decimal_id);
    let _ = reg_binop!("mul_decimal", decimal_id);
    let eq_decimal_id = reg_cmpop!("eq_decimal", decimal_id, bool_id);

    // ---- Float ops (IEEE 754 f64) ----
    let add_float_id = reg_binop!("add_float", float_id);
    let _ = reg_binop!("sub_float", float_id);
    let _ = reg_binop!("mul_float", float_id);
    let _ = reg_binop!("div_float", float_id);
    let eq_float_id = reg_cmpop!("eq_float", float_id, bool_id);

    // ---- Float32 ops (IEEE 754 f32) ----
    let add_float32_id = reg_binop!("add_float32", float32_id);
    let eq_float32_id = reg_cmpop!("eq_float32", float32_id, bool_id);

    // ---- Bool ops ----
    {
        let bool_ty = Term::indformer(bool_id, vec![]);
        let not_ty  = Term::pi(bool_ty.clone(), bool_ty.clone());
        let _not_id = declare_primitive(env, vec![], not_ty, PrimReduction::Op { symbol: "not_bool" })
            .map_err(|e| ElabError::Internal(format!("prim not_bool failed: {}", e)))?;
        globals.insert("not_bool".to_string(), _not_id);
        let and_ty = Term::pi(bool_ty.clone(), Term::pi(bool_ty.clone(), bool_ty.clone()));
        let _and_id = declare_primitive(env, vec![], and_ty.clone(), PrimReduction::Op { symbol: "and_bool" })
            .map_err(|e| ElabError::Internal(format!("prim and_bool failed: {}", e)))?;
        globals.insert("and_bool".to_string(), _and_id);
        let _or_id = declare_primitive(env, vec![], and_ty, PrimReduction::Op { symbol: "or_bool" })
            .map_err(|e| ElabError::Internal(format!("prim or_bool failed: {}", e)))?;
        globals.insert("or_bool".to_string(), _or_id);
    }

    // ── build dispatch tables ───────────────────────────────────────────────

    let mut add_table = HashMap::new();
    let mut eq_table  = HashMap::new();

    // Int: total
    add_table.insert(int_id, AddEntry {
        op_id: add_int_id,
        wrapping_id: None,
        no_ovf_id: None,
        result_id: int_id,
    });

    // Fixed-width signed: obligation-generating
    for (ty_id, op_id, wrap_id, novf_id) in &[
        (int8_id,  add_int8_id,  wrap_add_int8_id,  novf_add_int8_id),
        (int16_id, add_int16_id, wrap_add_int16_id, novf_add_int16_id),
        (int32_id, add_int32_id, wrap_add_int32_id, novf_add_int32_id),
        (int64_id, add_int64_id, wrap_add_int64_id, novf_add_int64_id),
    ] {
        add_table.insert(*ty_id, AddEntry {
            op_id: *op_id,
            wrapping_id: Some(*wrap_id),
            no_ovf_id: Some(*novf_id),
            result_id: *ty_id,
        });
    }

    // Fixed-width unsigned: obligation-generating
    for (ty_id, op_id, wrap_id, novf_id) in &[
        (uint8_id,  add_uint8_id,  wrap_add_uint8_id,  novf_add_uint8_id),
        (uint16_id, add_uint16_id, wrap_add_uint16_id, novf_add_uint16_id),
        (uint32_id, add_uint32_id, wrap_add_uint32_id, novf_add_uint32_id),
        (uint64_id, add_uint64_id, wrap_add_uint64_id, novf_add_uint64_id),
    ] {
        add_table.insert(*ty_id, AddEntry {
            op_id: *op_id,
            wrapping_id: Some(*wrap_id),
            no_ovf_id: Some(*novf_id),
            result_id: *ty_id,
        });
    }

    // Decimal: total (exact base-10)
    add_table.insert(decimal_id, AddEntry {
        op_id: add_decimal_id,
        wrapping_id: None,
        no_ovf_id: None,
        result_id: decimal_id,
    });

    // Float: total (IEEE, may lose precision)
    add_table.insert(float_id, AddEntry {
        op_id: add_float_id,
        wrapping_id: None,
        no_ovf_id: None,
        result_id: float_id,
    });

    // Float32: total
    add_table.insert(float32_id, AddEntry {
        op_id: add_float32_id,
        wrapping_id: None,
        no_ovf_id: None,
        result_id: float32_id,
    });

    // == dispatch
    eq_table.insert(int_id,     EqEntry { op_id: eq_int_id });
    eq_table.insert(decimal_id, EqEntry { op_id: eq_decimal_id });
    eq_table.insert(float_id,   EqEntry { op_id: eq_float_id });
    eq_table.insert(float32_id, EqEntry { op_id: eq_float32_id });

    Ok(NumericEnv {
        int_id, int8_id, int16_id, int32_id, int64_id,
        uint8_id, uint16_id, uint32_id, uint64_id,
        decimal_id, float_id, float32_id, bool_id, char_id,
        add_table,
        eq_table,
    })
}

// ── literal value construction ─────────────────────────────────────────────

/// Construct a `NumericLitVal` from an integer literal `n` at type `ty`.
///
/// If `ty` is `Int`, produce `NumericLitVal::Int(n)`.
/// If `ty` is a fixed-width signed/unsigned type, clamp/truncate to that width
/// so the interpreter evaluates the correct bit-pattern.
pub fn int_lit_val(n: i128, ty: &Term, nenv: &NumericEnv) -> NumericLitVal {
    if let Term::Const { id, .. } = ty {
        if *id == nenv.int_id {
            return NumericLitVal::Int(n);
        }
        // Fixed-width: store the truncated value as i64 (EvalVal::Int)
        let truncated = if *id == nenv.int8_id  { (n as i8) as i128 }
            else if *id == nenv.int16_id { (n as i16) as i128 }
            else if *id == nenv.int32_id { (n as i32) as i128 }
            else if *id == nenv.int64_id { (n as i64) as i128 }
            else if *id == nenv.uint8_id  { (n as u8) as i128 }
            else if *id == nenv.uint16_id { (n as u16) as i128 }
            else if *id == nenv.uint32_id { (n as u32) as i128 }
            else if *id == nenv.uint64_id { (n as u64) as i128 }
            else { n };
        return NumericLitVal::Int(truncated);
    }
    NumericLitVal::Int(n)
}
