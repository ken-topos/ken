//! Numeric tower registration and obligation infrastructure (`35 §2`, `35 §3`).
//!
//! Registers all numeric types and ops in the kernel global env, emits
//! no-overflow obligations for fixed-width arithmetic, and provides the
//! type-directed dispatch table used by the elaborator.
//!
//! Types: `Int` (arbitrary-precision), `Int8`…`Int64`, `UInt8`…`UInt64`,
//! `Float`, `Float32`, `Bool`.
//! All are primitive opaque types (`PrimReduction::OpaqueType`).
//!
//! `Decimal`/`Char` are **derived** (`18a §5.6`/`§5.9`, Phase-2 tranche #2
//! DEMOTE) — registered by `crate::decimal_char::register_decimal_char`
//! after this module, reusing `Prod`/`Int` rather than a primitive type.
//! `NumericEnv::decimal_id`/`char_id` are filled in by the caller once that
//! registration runs (see `lib.rs`).
//!
//! Ops: registered as `PrimReduction::Op { symbol }` with matching entries
//! in `ken-interp`'s `prim_reduce`. The symbol names are the stable interface.

use std::collections::HashMap;

use ken_kernel::{
    declare_deceq_certificate, declare_postulate, declare_primitive, GlobalEnv, GlobalId, Level,
    Term,
};
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

/// Dispatch record for a type-directed `-` / `*` operation (VAL2 #11) — a
/// total op, no overflow obligation (scoped to `Int`/`Float`, the types that
/// already carry a `sub_*`/`mul_*` primitive; unlike `+`, fixed-width `-`/`*`
/// obligation-generation is out of scope here).
#[derive(Clone, Debug)]
pub struct BinOpEntry {
    /// GlobalId of the op.
    pub op_id: GlobalId,
    /// GlobalId of the result type.
    pub result_id: GlobalId,
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
    /// Derived (`18a §5.6`) — filled in by `decimal_char::register_decimal_char`
    /// after `register_numeric_env` returns; not a primitive registration here.
    /// `Term::Const`-shaped (the transparent `Decimal := DecimalPair` alias).
    pub decimal_id: GlobalId,
    /// The alias's underlying inductive (`DecimalPair`) — `whnf` unfolds the
    /// `Decimal` alias all the way through, so any WHNF'd-type dispatch
    /// (`classify_add`/`classify_eq`) sees `Term::IndFormer{id:
    /// decimalpair_id}`, never `Term::Const{id: decimal_id}`. Also filled in
    /// by `decimal_char::register_decimal_char`.
    pub decimalpair_id: GlobalId,
    pub float_id:   GlobalId,
    pub float32_id: GlobalId,
    pub bool_id:    GlobalId,
    /// Derived (`18a §5.9`) — filled in by `decimal_char::register_decimal_char`.
    pub char_id:    GlobalId,
    /// `uint8_int_retract`, SUB-1b's single conversion-layer postulate.
    /// Filled by `conversions::register_conversions` after the two existing
    /// `UInt8`/`Int` primitive operations have been registered.
    pub uint8_int_retract_id: GlobalId,
    /// The actual `trusted_base()` delta observed while installing SUB-1b.
    /// Tests assert that this is exactly `{uint8_int_retract_id}`.
    pub uint8_retract_trusted_delta: Vec<GlobalId>,

    // --- `+` dispatch table (keyed by the type's GlobalId) ---
    add_table: HashMap<GlobalId, AddEntry>,

    // --- `==` dispatch table (keyed by the type's GlobalId) ---
    eq_table: HashMap<GlobalId, EqEntry>,

    // --- `-` dispatch table (keyed by the type's GlobalId, VAL2 #11) ---
    sub_table: HashMap<GlobalId, BinOpEntry>,

    // --- `*` dispatch table (keyed by the type's GlobalId, VAL2 #11) ---
    mul_table: HashMap<GlobalId, BinOpEntry>,
}

impl NumericEnv {
    /// Register (or replace) the `+` dispatch entry for `ty_id` — used by
    /// `decimal_char::register_decimal_char` to wire `Decimal` to the derived
    /// `decimal_add`, since `add_table` itself is private to this module.
    pub(crate) fn set_add_entry(&mut self, ty_id: GlobalId, entry: AddEntry) {
        self.add_table.insert(ty_id, entry);
    }

    /// Register (or replace) the `==` dispatch entry for `ty_id`.
    pub(crate) fn set_eq_entry(&mut self, ty_id: GlobalId, entry: EqEntry) {
        self.eq_table.insert(ty_id, entry);
    }

    /// Look up the dispatch entry for a type-directed `+` on the given type.
    ///
    /// Checks `Term::IndFormer` as well as `Term::Const`: `whnf` fully
    /// unfolds a transparent alias like `Decimal := DecimalPair` (`18a
    /// §5.6.1`), so a WHNF'd `Decimal`-typed term arrives here as
    /// `IndFormer{id: decimalpair_id}`, never `Const{id: decimal_id}`.
    pub fn classify_add(&self, ty: &Term) -> Option<&AddEntry> {
        match ty {
            Term::Const { id, .. } => self.add_table.get(id),
            Term::IndFormer { id, .. } => self.add_table.get(id),
            _ => None,
        }
    }

    /// Look up the dispatch entry for a type-directed `==` on the given type
    /// (see `classify_add`'s doc comment on the `IndFormer` case).
    pub fn classify_eq(&self, ty: &Term) -> Option<&EqEntry> {
        match ty {
            Term::Const { id, .. } => self.eq_table.get(id),
            Term::IndFormer { id, .. } => self.eq_table.get(id),
            _ => None,
        }
    }

    /// Look up the dispatch entry for a type-directed `-` on the given type
    /// (VAL2 #11; see `classify_add`'s doc comment on the `IndFormer` case).
    pub fn classify_sub(&self, ty: &Term) -> Option<&BinOpEntry> {
        match ty {
            Term::Const { id, .. } => self.sub_table.get(id),
            Term::IndFormer { id, .. } => self.sub_table.get(id),
            _ => None,
        }
    }

    /// Look up the dispatch entry for a type-directed `*` on the given type
    /// (VAL2 #11; see `classify_add`'s doc comment on the `IndFormer` case).
    pub fn classify_mul(&self, ty: &Term) -> Option<&BinOpEntry> {
        match ty {
            Term::Const { id, .. } => self.mul_table.get(id),
            Term::IndFormer { id, .. } => self.mul_table.get(id),
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
            let id = declare_postulate(env, $name.to_string(), vec![], novf_ty)
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
    let float_id   = reg_ty!("Float");
    let float32_id = reg_ty!("Float32");
    let bool_id    = reg_ty!("Bool");
    // `decimal_id`/`decimalpair_id`/`char_id` are derived (`18a §5.6`/`§5.9`)
    // — placeholder zeroed here, filled in by
    // `decimal_char::register_decimal_char` once it runs (needs
    // `Int`/`Bool`/`leq_int`/etc. from this function first).
    let decimal_id = GlobalId(0);
    let decimalpair_id = GlobalId(0);
    let char_id    = GlobalId(0);

    // ---- Int ops (total, no obligation) ----
    let add_int_id = reg_binop!("add_int", int_id);
    let sub_int_id = reg_binop!("sub_int", int_id);
    let mul_int_id = reg_binop!("mul_int", int_id);
    let eq_int_id  = reg_cmpop!("eq_int", int_id, bool_id);

    // Register `Int`'s decidable-equality certificate
    // (`docs/adr/0013-int-decidable-equality-kernel-posture.md` Layer 1):
    // the kernel trusts `eq_int` to decide `Int` equality, both directions.
    // General opt-in mechanism; `Int` is its first registrant. `True` is
    // `Bool`'s constructor 0 (`data Bool = True | False`, `ElabEnv::empty`).
    let bool_true_id = env
        .inductive(bool_id)
        .ok_or_else(|| ElabError::Internal("Bool not registered as inductive".into()))?
        .constructors[0]
        .id;
    let int_eq_cert = declare_deceq_certificate(env, int_id, eq_int_id, bool_id, bool_true_id)
        .map_err(|e| ElabError::Internal(format!("Int deceq certificate failed: {}", e)))?;
    globals.insert("int_eq_sound".to_string(), int_eq_cert.sound);
    globals.insert("int_eq_complete".to_string(), int_eq_cert.complete);

    // Register `Int` as the home type of kernel-native `Term::IntLit`
    // values (`docs/adr/0013-int-decidable-equality-kernel-posture.md`
    // Layer 2). Harmless to land ahead of literal emission: no surface
    // `Int` literal constructs an `IntLit` yet (that wiring is a separate,
    // committed follow-up), so this registration is inert until then —
    // `infer`/`eq_reduce`'s new `IntLit` arms simply have nothing to see.
    env.register_int_lit_type(int_id);

    // `Int`'s ordering comparison (`30-taxonomy.md §4`'s "comparison
    // primitives `Int → Int → Bool`" — plural, already assumed to justify
    // `Bool`'s prelude membership — but only `eq_int` had actually been
    // wired; `leq_int` completes it). ES4-classes needs it to wrap `Ord
    // Int`'s `leq` operation field (`51-lawful-classes.md §6`).
    let leq_int_id = reg_cmpop!("leq_int", int_id, bool_id);
    let _ = leq_int_id;

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

    // ---- Decimal ops: DEMOTE→derived (`18a §5.6.1`) — no reg_binop!/reg_cmpop!
    // here; `decimal_char::register_decimal_char` wires `add_table`/`eq_table`
    // entries directly to the derived `decimal_add`/`decimal_eq` GlobalIds.

    // ---- Float ops (IEEE 754 f64) ----
    let add_float_id = reg_binop!("add_float", float_id);
    let sub_float_id = reg_binop!("sub_float", float_id);
    let mul_float_id = reg_binop!("mul_float", float_id);
    let _ = reg_binop!("div_float", float_id); // `div` out of scope (VAL2 #11)
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

    // Decimal: DEMOTE→derived — `decimal_char::register_decimal_char` inserts
    // the `add_table`/`eq_table` entries once `decimal_id` is a real id.

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

    // == dispatch (Decimal's entry is inserted by `register_decimal_char`)
    eq_table.insert(int_id,     EqEntry { op_id: eq_int_id });
    eq_table.insert(float_id,   EqEntry { op_id: eq_float_id });
    eq_table.insert(float32_id, EqEntry { op_id: eq_float32_id });

    // -/* dispatch (VAL2 #11) — scoped to the types that already carry a
    // total `sub_*`/`mul_*` primitive (`Int`, `Float`); fixed-width and
    // `Decimal` are out of scope (no obligation-generating variant here).
    let mut sub_table = HashMap::new();
    let mut mul_table = HashMap::new();
    sub_table.insert(int_id,   BinOpEntry { op_id: sub_int_id,   result_id: int_id });
    mul_table.insert(int_id,   BinOpEntry { op_id: mul_int_id,   result_id: int_id });
    sub_table.insert(float_id, BinOpEntry { op_id: sub_float_id, result_id: float_id });
    mul_table.insert(float_id, BinOpEntry { op_id: mul_float_id, result_id: float_id });

    Ok(NumericEnv {
        int_id, int8_id, int16_id, int32_id, int64_id,
        uint8_id, uint16_id, uint32_id, uint64_id,
        decimal_id, decimalpair_id, float_id, float32_id, bool_id, char_id,
        uint8_int_retract_id: GlobalId(0),
        uint8_retract_trusted_delta: Vec::new(),
        add_table,
        eq_table,
        sub_table,
        mul_table,
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
