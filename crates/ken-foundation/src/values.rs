//! Value types — the set of values the F4 bench exercises.
//!
//! This models the value kinds in `spec/40-runtime/41-values.md §1–2`
//! at the level needed for benchmarking canonical encoding + intern.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// A Ken value. Scalars are immediate; compounds are content-addressed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    // --- immediate scalars (§1) ---
    Bool(bool),
    Char(char),
    Float(u64),      // f64 bits as u64 for Eq/Ord
    Float32(u32),    // f32 bits
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    SmallInt(i64),   // Int within i64 range
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
        fields: Vec<Value>,  // declaration order
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
    Closure {
        code_id: u64,
        /// Captured environment — a record (fields in capture order).
        /// Encoded inline (memcmp-exact), not via a hash digest, so the
        /// "equal slot ⇒ structurally equal" invariant is total.
        captured: Vec<Value>,
    },

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
