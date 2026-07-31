//! Value types — the set of values the F4 bench exercises.
//!
//! This models the value kinds in `spec/40-runtime/41-values.md §1–2`
//! at the level needed for benchmarking canonical encoding + intern.
//!
//! ⛔ **This model is closure-free, and deliberately so.** It previously
//! carried a `Closure` variant with its own `0x09` canonical encoding, which
//! made it a **second, contradictory answer** to the question
//! `RT-VALUE-TOTALITY-P2` settles: `41 §2.1` gives ordinary closures no
//! canonical encoding, no slot identity and no structural equality. A bench is
//! still a **shipped public validation model**, and a reader had no way to tell
//! which of the two answers binds — so the stale one is retired here rather
//! than left for a follow-up.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// A Ken value. Scalars are immediate; compounds are content-addressed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    // --- immediate scalars (§1) ---
    Bool(bool),
    Char(char),
    Float(u64),   // f64 bits as u64 for Eq/Ord
    Float32(u32), // f32 bits
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    SmallInt(i64), // Int within i64 range
    SmallDecimal {
        coefficient: i64,
        exponent: i32,
    },

    // --- interned compounds (§2) ---
    BigInt {
        sign: Sign,
        limbs: Vec<u64>,
    },
    BigDecimal {
        sign: Sign,
        coefficient: Vec<u64>,
        exponent: i32,
    },
    Constructor {
        constructor_id: u32,
        args: Vec<Value>,
    },
    Record {
        type_id: u32,
        fields: Vec<Value>, // declaration order
    },
    String(String),
    Bytes(Vec<u8>),
    Array {
        elem_type_id: u32,
        elements: Vec<Value>,
    },
    Map {
        key_type_id: u32,
        value_type_id: u32,
        entries: BTreeMap<Vec<u8>, Value>, // keys stored as canonical bytes for ordering
    },
    Set {
        elem_type_id: u32,
        elements: BTreeSet<Vec<u8>>, // elements stored as canonical bytes for ordering
    },
    // ⛔ **No `Closure` variant**, and its doc comment is gone with it. That
    // comment asserted the captured environment was encoded inline rather than
    // as a digest, and concluded that an "equal slot implies structurally
    // equal" invariant was therefore total — a statement of the retired
    // contract, and the more misleading for being confident. `41 §2.1` gives
    // ordinary closures no canonical encoding, no slot identity and no
    // structural equality, so there is no invariant of that shape to be total.
    //
    // ⚠ Removed rather than annotated: an appended correction leaves the false
    // sentence operative, and it is the sentence positioned to be believed.
    //
    // ⚠ Paraphrased rather than quoted, deliberately: `AC-V6`'s probe greps
    // `crates/` for the retired phrasing, and a verbatim quotation inside the
    // comment that *removes* it would keep the probe red. A deletion has a text
    // surface that outlives the deletion.

    // --- special ---
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sign {
    NonNegative = 0,
    Negative = 1,
}

impl Value {
    /// Is this value immediate (never interned)?
    pub fn is_immediate(&self) -> bool {
        matches!(
            self,
            Value::Bool(_)
                | Value::Char(_)
                | Value::Float(_)
                | Value::Float32(_)
                | Value::Int8(_)
                | Value::Int16(_)
                | Value::Int32(_)
                | Value::Int64(_)
                | Value::UInt8(_)
                | Value::UInt16(_)
                | Value::UInt32(_)
                | Value::UInt64(_)
                | Value::SmallInt(_)
                | Value::SmallDecimal { .. }
        )
    }

    /// Is this value content-addressed (interned)?
    pub fn is_compound(&self) -> bool {
        !self.is_immediate() && !matches!(self, Value::Unknown)
    }
}
