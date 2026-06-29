//! Canonical byte encoding — per `docs/design/content-addressing.md §1`
//! and `spec/40-runtime/41-values.md §3a`.
//!
//! The correctness bar: two structurally-equal values MUST encode to
//! identical bytes regardless of construction history.

use crate::values::Value;

/// Trait for values that can produce a canonical byte encoding.
pub trait Canonical {
    /// Write the canonical byte encoding into `out`.
    fn encode_canonical(&self, out: &mut Vec<u8>);
}

/// Kind tags — 1-byte prefix disambiguating value kinds.
/// See `docs/design/content-addressing.md §1.1`.
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
    // Immediates (0x10–0x1F) — encoded when appearing as sub-values within
    // compounds; never hashed directly since they are never interned.
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

fn write_u32_le(v: u32, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u64_le(v: u64, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_i32_le(v: i32, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
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
                out.extend_from_slice(&arity.to_le_bytes());
                for arg in args {
                    arg.encode_canonical(out);
                }
            }
            Value::Record { type_id, fields } => {
                out.push(tag::RECORD);
                write_u32_le(*type_id, out);
                let arity = fields.len().min(65535) as u16;
                out.extend_from_slice(&arity.to_le_bytes());
                for field in fields {
                    field.encode_canonical(out);
                }
            }
            Value::String(s) => {
                out.push(tag::STRING);
                // NFC normalization would go here in production.
                // For F4 bench, we assume input strings are already normalized.
                let utf8 = s.as_bytes();
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
                // BTreeMap iterates in key order — keys are canonical bytes,
                // sorted lexicographically by construction (BTreeMap's Ord).
                for (key_bytes, val) in entries {
                    // key is already canonical bytes — write length-prefixed
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
                // BTreeSet iterates in element order — elements are canonical
                // bytes, sorted lexicographically.
                for elem_bytes in elements {
                    write_u32_le(elem_bytes.len() as u32, out);
                    out.extend_from_slice(elem_bytes);
                }
            }
            Value::Closure { code_id, captured } => {
                out.push(tag::CLOSURE);
                write_u64_le(*code_id, out);
                let arity = captured.len().min(65535) as u16;
                out.extend_from_slice(&arity.to_le_bytes());
                for val in captured {
                    val.encode_canonical(out);
                }
            }

            // --- immediate scalars — encoded when sub-values of compounds ---
            Value::Bool(b) => {
                out.push(tag::BOOL);
                out.push(*b as u8);
            }
            Value::Char(c) => {
                out.push(tag::CHAR);
                out.extend_from_slice(&(*c as u32).to_le_bytes());
            }
            Value::Float(f) => {
                out.push(tag::FLOAT);
                out.extend_from_slice(&f.to_le_bytes());
            }
            Value::Float32(f) => {
                out.push(tag::FLOAT32);
                out.extend_from_slice(&f.to_le_bytes());
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
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::UInt16(v) => {
                out.push(tag::UINT16);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::UInt32(v) => {
                out.push(tag::UINT32);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::UInt64(v) => {
                out.push(tag::UINT64);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::SmallInt(v) => {
                out.push(tag::SMALL_INT);
                out.extend_from_slice(&v.to_le_bytes());
            }
            Value::SmallDecimal { coefficient, exponent } => {
                out.push(tag::SMALL_DECIMAL);
                out.extend_from_slice(&coefficient.to_le_bytes());
                out.extend_from_slice(&exponent.to_le_bytes());
            }
            Value::Unknown => {
                out.push(tag::UNKNOWN);
            }
        }
    }
}

/// Strip trailing zero limbs to produce the minimal representation.
fn minimal_limbs(limbs: &[u64]) -> &[u64] {
    let end = limbs
        .iter()
        .rposition(|&l| l != 0)
        .map(|i| i + 1)
        .unwrap_or(1); // zero → one zero limb
    &limbs[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::Sign;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    fn encode(v: &Value) -> Vec<u8> {
        let mut out = Vec::new();
        v.encode_canonical(&mut out);
        out
    }

    #[test]
    fn map_ordering_deterministic() {
        // Two maps with same entries inserted in different orders
        // must encode identically.
        let key_bytes_1 = encode(&Value::String("key1".into()));
        let key_bytes_2 = encode(&Value::String("key2".into()));
        let key_bytes_3 = encode(&Value::String("key3".into()));

        // Insert order 3,1,2
        let mut entries_a = BTreeMap::new();
        entries_a.insert(key_bytes_3.clone(), Value::SmallInt(3));
        entries_a.insert(key_bytes_1.clone(), Value::SmallInt(1));
        entries_a.insert(key_bytes_2.clone(), Value::SmallInt(2));

        let map_a = Value::Map {
            key_type_id: 1,
            value_type_id: 2,
            entries: entries_a,
        };

        // Insert order 1,2,3
        let mut entries_b = BTreeMap::new();
        entries_b.insert(key_bytes_1.clone(), Value::SmallInt(1));
        entries_b.insert(key_bytes_2.clone(), Value::SmallInt(2));
        entries_b.insert(key_bytes_3.clone(), Value::SmallInt(3));

        let map_b = Value::Map {
            key_type_id: 1,
            value_type_id: 2,
            entries: entries_b,
        };

        assert_eq!(encode(&map_a), encode(&map_b));
    }

    #[test]
    fn set_ordering_deterministic() {
        let elem_a = encode(&Value::String("c".into()));
        let elem_b = encode(&Value::String("a".into()));
        let elem_c = encode(&Value::String("b".into()));

        let mut set_a = BTreeSet::new();
        set_a.insert(elem_a.clone());
        set_a.insert(elem_b.clone());
        set_a.insert(elem_c.clone());

        let mut set_b = BTreeSet::new();
        set_b.insert(elem_b.clone());
        set_b.insert(elem_c.clone());
        set_b.insert(elem_a.clone());

        assert_eq!(encode(&Value::Set { elem_type_id: 1, elements: set_a }),
                   encode(&Value::Set { elem_type_id: 1, elements: set_b }));
    }

    #[test]
    fn record_field_order_is_declaration() {
        // Record fields encode in the order they appear in `fields` —
        // the caller (construction site) must use declaration order.
        let rec_a = Value::Record {
            type_id: 1,
            fields: vec![Value::SmallInt(1), Value::String("hello".into())],
        };
        let rec_b = Value::Record {
            type_id: 1,
            fields: vec![Value::SmallInt(1), Value::String("hello".into())],
        };
        assert_eq!(encode(&rec_a), encode(&rec_b));

        // Different field order → different encoding (correct — the fields
        // vector order IS the declaration order; a different order is a
        // different record type)
        let rec_c = Value::Record {
            type_id: 1,
            fields: vec![Value::String("hello".into()), Value::SmallInt(1)],
        };
        assert_ne!(encode(&rec_a), encode(&rec_c));
    }

    #[test]
    fn bignum_minimal_limb() {
        let zero = Value::BigInt {
            sign: Sign::NonNegative,
            limbs: vec![0, 0, 0],
        };
        let zero_expected = Value::BigInt {
            sign: Sign::NonNegative,
            limbs: vec![0],
        };
        assert_eq!(encode(&zero), encode(&zero_expected));
    }

    #[test]
    fn kind_tags_disambiguate() {
        // String "42" vs Bytes [0x34, 0x32] — different tags
        let s = Value::String("42".into());
        let b = Value::Bytes(vec![0x34, 0x32]);
        assert_ne!(encode(&s), encode(&b));
    }
}
