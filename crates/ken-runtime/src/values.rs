//! Value types — `spec/40-runtime/41-values.md §1–2,§5,§6`.
//!
//! Scalars are immediate (never interned). Compounds are content-addressed.
//! `Unknown` is the third truth value for partially-verified programs.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// A Ken value.  Scalars are immediate; compounds are content-addressed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    // --- immediate scalars (§1, §5 table) ---
    Bool(bool),
    Char(char),
    Float(u64),   // f64 bits; -0.0 ≠ +0.0 by bit pattern (design doc §1.1)
    Float32(u32), // f32 bits
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    SmallInt(i64), // Int within i64 range (§1 fast path)
    SmallDecimal {
        coefficient: i64,
        exponent: i32,
    },

    // --- interned compounds (§2, §5 table) ---
    /// Arbitrary-precision integer beyond i64 (§1 overflow path).
    BigInt {
        sign: Sign,
        limbs: Vec<u64>, // minimal-limb, LE (design doc §1.10)
    },
    /// Big Decimal (coefficient beyond i64 fast path, design doc §1.10.1).
    BigDecimal {
        sign: Sign,
        coefficient: Vec<u64>,
        exponent: i32,
    },
    /// Constructor application — `data` kind (design doc §1.2).
    Constructor {
        constructor_id: u32,
        args: Vec<Value>,
    },
    /// Named-field record — Σ-type (design doc §1.3).
    Record {
        type_id: u32,
        fields: Vec<Value>, // declaration order (normative)
    },
    /// NFC-normalized Unicode string (design doc §1.4 — K3 must normalize).
    String(String),
    /// Opaque byte sequence (design doc §1.5).
    Bytes(Vec<u8>),
    /// Indexed sequence (design doc §1.6).
    Array {
        elem_type_id: u32,
        elements: Vec<Value>,
    },
    /// Key-value mapping; keys stored as canonical bytes for lexicographic order
    /// (design doc §1.7).
    Map {
        key_type_id: u32,
        value_type_id: u32,
        entries: BTreeMap<Vec<u8>, Value>,
    },
    /// Unordered set; elements stored as canonical bytes (design doc §1.8).
    Set {
        elem_type_id: u32,
        elements: BTreeSet<Vec<u8>>,
    },
    /// Closure — code pointer + full canonical captured environment (design doc §1.9).
    Closure {
        code_id: u64,
        captured: Vec<Value>, // in capture order; encoded inline (memcmp-exact)
    },

    // --- special (§6) ---
    /// Third truth value: the result of an open verification hole.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sign {
    NonNegative = 0,
    Negative = 1,
}

impl Value {
    /// Returns `true` iff this value is an immediate scalar (never interned).
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

    /// Returns `true` iff this value is a compound (must be interned).
    pub fn is_compound(&self) -> bool {
        !self.is_immediate() && !matches!(self, Value::Unknown)
    }
}
