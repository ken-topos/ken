//! Canonical byte encoding — `docs/design/content-addressing.md §1`,
//! `spec/40-runtime/41-values.md §3a`.
//!
//! Correctness invariant: two structurally-equal values MUST encode to
//! identical bytes regardless of construction history.

use crate::values::Value;
use unicode_normalization::UnicodeNormalization;

/// Values that can produce a canonical byte encoding.
pub trait Canonical {
    fn encode_canonical(&self, out: &mut Vec<u8>);
}

/// Kind tags (design doc §1.1).
mod tag {
    pub const BIG_INT: u8 = 0x01;
    pub const DATA: u8 = 0x02;
    pub const RECORD: u8 = 0x03;
    pub const STRING: u8 = 0x04;
    pub const BYTES: u8 = 0x05;
    pub const ARRAY: u8 = 0x06;
    pub const MAP: u8 = 0x07;
    pub const SET: u8 = 0x08;
    pub const CLOSURE: u8 = 0x09;
    pub const BIG_DECIMAL: u8 = 0x0A;
    // Immediate scalars appear in sub-value position within compounds.
    pub const BOOL: u8 = 0x10;
    pub const CHAR: u8 = 0x11;
    pub const FLOAT: u8 = 0x12;
    pub const FLOAT32: u8 = 0x13;
    pub const INT8: u8 = 0x14;
    pub const INT16: u8 = 0x15;
    pub const INT32: u8 = 0x16;
    pub const INT64: u8 = 0x17;
    pub const UINT8: u8 = 0x18;
    pub const UINT16: u8 = 0x19;
    pub const UINT32: u8 = 0x1A;
    pub const UINT64: u8 = 0x1B;
    pub const SMALL_INT: u8 = 0x1C;
    pub const SMALL_DECIMAL: u8 = 0x1D;
    pub const UNKNOWN: u8 = 0xFE;
}

fn write_u16_le(v: u16, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u32_le(v: u32, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u64_le(v: u64, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_i32_le(v: i32, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// Strip trailing zero limbs; a zero value keeps one zero limb (design doc §1.10).
fn minimal_limbs(limbs: &[u64]) -> &[u64] {
    let end = limbs
        .iter()
        .rposition(|&l| l != 0)
        .map(|i| i + 1)
        .unwrap_or(1);
    &limbs[..end]
}

impl Canonical for Value {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        match self {
            // --- interned compounds ---
            Value::BigInt { sign, limbs } => {
                out.push(tag::BIG_INT);
                out.push(*sign as u8);
                let minimal = minimal_limbs(limbs);
                write_u32_le(minimal.len() as u32, out);
                for &limb in minimal {
                    write_u64_le(limb, out);
                }
            }

            Value::BigDecimal {
                sign,
                coefficient,
                exponent,
            } => {
                out.push(tag::BIG_DECIMAL);
                out.push(*sign as u8);
                write_i32_le(*exponent, out);
                let minimal = minimal_limbs(coefficient);
                write_u32_le(minimal.len() as u32, out);
                for &limb in minimal {
                    write_u64_le(limb, out);
                }
            }

            Value::Constructor {
                constructor_id,
                args,
            } => {
                out.push(tag::DATA);
                write_u32_le(*constructor_id, out);
                let arity = args.len().min(65535) as u16;
                write_u16_le(arity, out);
                for arg in args {
                    arg.encode_canonical(out);
                }
            }

            Value::Record { type_id, fields } => {
                out.push(tag::RECORD);
                write_u32_le(*type_id, out);
                let arity = fields.len().min(65535) as u16;
                write_u16_le(arity, out);
                for field in fields {
                    field.encode_canonical(out);
                }
            }

            Value::String(s) => {
                // K3: NFC-normalize at encoding time (design doc §1.4 note).
                // The normalized form is what gets hashed and stored.
                out.push(tag::STRING);
                let nfc: std::string::String = s.chars().nfc().collect();
                let utf8 = nfc.as_bytes();
                write_u32_le(utf8.len() as u32, out);
                out.extend_from_slice(utf8);
            }

            Value::Bytes(data) => {
                out.push(tag::BYTES);
                write_u32_le(data.len() as u32, out);
                out.extend_from_slice(data);
            }

            Value::Array {
                elem_type_id,
                elements,
            } => {
                out.push(tag::ARRAY);
                write_u32_le(*elem_type_id, out);
                write_u32_le(elements.len() as u32, out);
                for elem in elements {
                    elem.encode_canonical(out);
                }
            }

            Value::Map {
                key_type_id,
                value_type_id,
                entries,
            } => {
                out.push(tag::MAP);
                write_u32_le(*key_type_id, out);
                write_u32_le(*value_type_id, out);
                write_u32_le(entries.len() as u32, out);
                // BTreeMap iterates in key-canonical-bytes lexicographic order.
                for (key_bytes, val) in entries {
                    write_u32_le(key_bytes.len() as u32, out);
                    out.extend_from_slice(key_bytes);
                    val.encode_canonical(out);
                }
            }

            Value::Set {
                elem_type_id,
                elements,
            } => {
                out.push(tag::SET);
                write_u32_le(*elem_type_id, out);
                write_u32_le(elements.len() as u32, out);
                // BTreeSet iterates in element-canonical-bytes lexicographic order.
                for elem_bytes in elements {
                    write_u32_le(elem_bytes.len() as u32, out);
                    out.extend_from_slice(elem_bytes);
                }
            }

            Value::Closure { code_id, captured } => {
                out.push(tag::CLOSURE);
                write_u64_le(*code_id, out);
                let arity = captured.len().min(65535) as u16;
                write_u16_le(arity, out);
                // Full canonical encoding of captured values (design doc §1.9):
                // memcmp-exact, NOT a hash digest.
                for val in captured {
                    val.encode_canonical(out);
                }
            }

            // --- immediate scalars (encoded when sub-values of compounds) ---
            Value::Bool(b) => {
                out.push(tag::BOOL);
                out.push(*b as u8);
            }
            Value::Char(c) => {
                out.push(tag::CHAR);
                write_u32_le(*c as u32, out);
            }
            Value::Float(f) => {
                out.push(tag::FLOAT);
                write_u64_le(*f, out);
            }
            Value::Float32(f) => {
                out.push(tag::FLOAT32);
                write_u32_le(*f, out);
            }
            Value::Int8(v) => {
                out.push(tag::INT8);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::Int16(v) => {
                out.push(tag::INT16);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::Int32(v) => {
                out.push(tag::INT32);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::Int64(v) => {
                out.push(tag::INT64);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::UInt8(v) => {
                out.push(tag::UINT8);
                out.push(*v);
            }
            Value::UInt16(v) => {
                out.push(tag::UINT16);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::UInt32(v) => {
                out.push(tag::UINT32);
                write_u32_le(*v, out);
            }
            Value::UInt64(v) => {
                out.push(tag::UINT64);
                write_u64_le(*v, out);
            }
            Value::SmallInt(v) => {
                out.push(tag::SMALL_INT);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::SmallDecimal {
                coefficient,
                exponent,
            } => {
                out.push(tag::SMALL_DECIMAL);
                out.extend_from_slice(&coefficient.to_le_bytes());
                write_i32_le(*exponent, out);
            }
            Value::Unknown => {
                out.push(tag::UNKNOWN);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::Sign;
    use std::collections::{BTreeMap, BTreeSet};

    fn encode(v: &Value) -> Vec<u8> {
        let mut out = Vec::new();
        v.encode_canonical(&mut out);
        out
    }

    // --- conformance: runtime/values/canonical-encoding-map-ordering ---
    #[test]
    fn map_ordering_deterministic() {
        let kb1 = encode(&Value::String("k1".into()));
        let kb2 = encode(&Value::String("k2".into()));
        let kb3 = encode(&Value::String("k3".into()));

        let mut entries_a = BTreeMap::new();
        entries_a.insert(kb3.clone(), Value::SmallInt(3));
        entries_a.insert(kb1.clone(), Value::SmallInt(1));
        entries_a.insert(kb2.clone(), Value::SmallInt(2));

        let mut entries_b = BTreeMap::new();
        entries_b.insert(kb1.clone(), Value::SmallInt(1));
        entries_b.insert(kb2.clone(), Value::SmallInt(2));
        entries_b.insert(kb3.clone(), Value::SmallInt(3));

        assert_eq!(
            encode(&Value::Map {
                key_type_id: 1,
                value_type_id: 2,
                entries: entries_a
            }),
            encode(&Value::Map {
                key_type_id: 1,
                value_type_id: 2,
                entries: entries_b
            }),
        );
    }

    // --- conformance: runtime/values/canonical-encoding-set-ordering ---
    #[test]
    fn set_ordering_deterministic() {
        let ea = encode(&Value::String("c".into()));
        let eb = encode(&Value::String("a".into()));
        let ec = encode(&Value::String("b".into()));

        let mut set_a = BTreeSet::new();
        set_a.insert(ea.clone());
        set_a.insert(eb.clone());
        set_a.insert(ec.clone());

        let mut set_b = BTreeSet::new();
        set_b.insert(eb.clone());
        set_b.insert(ec.clone());
        set_b.insert(ea.clone());

        assert_eq!(
            encode(&Value::Set {
                elem_type_id: 1,
                elements: set_a
            }),
            encode(&Value::Set {
                elem_type_id: 1,
                elements: set_b
            }),
        );
    }

    // --- conformance: runtime/values/canonical-encoding-record-field-order ---
    #[test]
    fn record_field_order_is_declaration_order() {
        // Same fields same order → identical bytes
        let rec_a = Value::Record {
            type_id: 1,
            fields: vec![Value::SmallInt(1), Value::String("hello".into())],
        };
        let rec_b = Value::Record {
            type_id: 1,
            fields: vec![Value::SmallInt(1), Value::String("hello".into())],
        };
        assert_eq!(encode(&rec_a), encode(&rec_b));

        // Different field order → different encoding (each order is a distinct value)
        let rec_c = Value::Record {
            type_id: 1,
            fields: vec![Value::String("hello".into()), Value::SmallInt(1)],
        };
        assert_ne!(encode(&rec_a), encode(&rec_c));
    }

    // --- conformance: runtime/values/bignum-minimal-limb-encoding ---
    #[test]
    fn bignum_minimal_limb() {
        // Trailing zero limbs are stripped: [0,0,0] encodes as [0]
        let a = Value::BigInt {
            sign: Sign::NonNegative,
            limbs: vec![0, 0, 0],
        };
        let b = Value::BigInt {
            sign: Sign::NonNegative,
            limbs: vec![0],
        };
        assert_eq!(encode(&a), encode(&b));

        // 2^64: two limbs [0, 1], no trailing zero
        let big = Value::BigInt {
            sign: Sign::NonNegative,
            limbs: vec![0, 1],
        };
        assert_ne!(encode(&big), encode(&b));
    }

    // --- conformance: runtime/values/dedup-across-kinds ---
    #[test]
    fn kind_tags_disambiguate() {
        // String "42" vs Bytes b"42" — same raw bytes, different kind tag
        let s = Value::String("42".into());
        let b = Value::Bytes(vec![0x34, 0x32]);
        assert_ne!(encode(&s), encode(&b));
    }

    // NFC normalization: precomposed and decomposed form encode identically
    #[test]
    fn string_nfc_normalization() {
        // U+00E9 LATIN SMALL LETTER E WITH ACUTE (precomposed)
        let precomposed = Value::String("\u{00e9}".into());
        // e + U+0301 COMBINING ACUTE ACCENT (decomposed)
        let decomposed = Value::String("e\u{0301}".into());
        // Both should encode to the same NFC bytes
        assert_eq!(encode(&precomposed), encode(&decomposed));
    }

    // Float bit-pattern encoding: -0.0 ≠ +0.0 (design doc §1.1 note)
    #[test]
    fn float_minus_zero_distinct_from_plus_zero() {
        let pos_zero = Value::Float(0f64.to_bits());
        let neg_zero = Value::Float((-0f64).to_bits());
        assert_ne!(encode(&pos_zero), encode(&neg_zero));
    }
}
