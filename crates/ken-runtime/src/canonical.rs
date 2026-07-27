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
    // ⛔ `0x09` was `CLOSURE` and is **retired, not reused**. `41 §2.1` assigns
    // ordinary closures no canonical kind tag at all, so there is no encoding to
    // give one. The ordinal stays burned so a decoder meeting a legacy `0x09`
    // byte refuses it rather than silently reading it as whatever takes the slot
    // next.
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

/// One unit of pending emission work for the iterative encoder (`D1`).
///
/// Encoding is a **streaming pre-order append**: every arm writes its own header
/// (tag, ids, arity/length) and children append after it, so a parent's bytes
/// never depend on a child's. That is precisely what lets one explicit work
/// stack replace host recursion here without any postorder fold.
enum Step<'a> {
    /// Emit this value's own header bytes, then push its children.
    Val(&'a Value),
    /// Emit a `u32` length prefix followed by these already-canonical bytes.
    /// Needed only for the `Map` key that must precede each entry value.
    Raw(&'a [u8]),
}

/// The closed **allow-list** of child-position shapes — `D2` clauses 1 and 2.
///
/// This is deliberately not a deny-list of spellings (`Rc<`, `RefCell`, …): a
/// spelling list is not a proof of the property, and `type Handle = Rc<Value>`
/// walks straight past one. Instead the permitted owning shapes are enumerated
/// positively and the compiler rejects everything else.
mod child_positions {
    use super::{Step, Value};
    use std::collections::BTreeMap;

    /// Sealed, so no downstream crate can widen the allow-list from outside.
    mod sealed {
        pub trait Sealed {}
        impl Sealed for Vec<super::Value> {}
        impl Sealed for std::collections::BTreeMap<Vec<u8>, super::Value> {}
    }

    /// Implemented **only** for the permitted *owning* child-collection shapes.
    ///
    /// A recursive child position that acquires reference / handle / arena /
    /// slot / index indirection, or interior mutation (`Rc<Value>`,
    /// `&'a Value`, `SlotId`, `RefCell<Value>`, …) has no impl here, so
    /// [`push`] below fails to compile for it and the encoder cannot be built.
    /// That is what makes the unrepresentability of cycles unable to silently
    /// lapse: the property is enforced by the bound, not by review.
    pub(super) trait OwnedChildren: sealed::Sealed {
        fn push_steps<'a>(&'a self, stack: &mut Vec<Step<'a>>);
    }

    impl OwnedChildren for Vec<Value> {
        fn push_steps<'a>(&'a self, stack: &mut Vec<Step<'a>>) {
            // Reversed: LIFO pops then restore declaration order.
            for child in self.iter().rev() {
                stack.push(Step::Val(child));
            }
        }
    }

    impl OwnedChildren for BTreeMap<Vec<u8>, Value> {
        fn push_steps<'a>(&'a self, stack: &mut Vec<Step<'a>>) {
            // Each entry emits its len-prefixed canonical key, then its value.
            // Reversing the entry order makes the pops interleave
            // key, value, key, value — matching the recursive encoder exactly.
            for (key_bytes, val) in self.iter().rev() {
                stack.push(Step::Val(val));
                stack.push(Step::Raw(key_bytes));
            }
        }
    }

    /// Hand a child collection to the allow-list.
    ///
    /// Generic and **bounded** — the bound is the half of `D2` that rejects an
    /// indirection-bearing child position.
    pub(super) fn push<'a, C: OwnedChildren>(children: &'a C, stack: &mut Vec<Step<'a>>) {
        children.push_steps(stack);
    }
}

impl Canonical for Value {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        // The one iterative driver. Host-stack usage is O(1) in value depth;
        // the work stack lives on the heap, so depth is bounded by allocation
        // (an ordinary resource boundary) and never by the host stack.
        let mut stack: Vec<Step<'_>> = vec![Step::Val(self)];
        while let Some(step) = stack.pop() {
            match step {
                Step::Raw(bytes) => {
                    write_u32_le(bytes.len() as u32, out);
                    out.extend_from_slice(bytes);
                }
                Step::Val(value) => encode_header(value, out, &mut stack),
            }
        }
    }
}

/// Emit `value`'s own bytes, then push its children onto `stack`.
///
/// ⛔ **Exhaustive over every `Value` variant, with no `_` arm** — `D2`
/// clause 1. A new variant fails to compile until it declares its child
/// position here, so coverage is designed in rather than discovered by review.
fn encode_header<'a>(value: &'a Value, out: &mut Vec<u8>, stack: &mut Vec<Step<'a>>) {
    match value {
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
            child_positions::push(args, stack);
        }

        Value::Record { type_id, fields } => {
            out.push(tag::RECORD);
            write_u32_le(*type_id, out);
            let arity = fields.len().min(65535) as u16;
            write_u16_le(arity, out);
            child_positions::push(fields, stack);
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
            child_positions::push(elements, stack);
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
            // BTreeMap iterates in key-canonical-bytes lexicographic order;
            // the allow-list impl preserves that, emitting each entry as a
            // len-prefixed `Step::Raw` key followed by its value.
            child_positions::push(entries, stack);
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

        // ⛔ **No closure arm.** An ordinary closure has no canonical encoding —
        // not an inline one, and equally not a digest, pointer, ordinal or
        // handle standing in for one. It cannot appear here because the carrier
        // this encoder walks has no closure variant to match, which is the
        // property `41 §2.1` requires rather than a check this function
        // performs.

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

/// `AC-V1b` — a frozen replica of the **pre-change recursive** encoder, kept as
/// the differential reference for the iterative one.
///
/// It deliberately uses the production `tag` constants and [`minimal_limbs`] so
/// it is a faithful *pre-change* replica: its job is to pin that restructuring
/// the **traversal** changed no bytes. ⚠ It is therefore **not** an independent
/// oracle for the byte *values* — a mutation to a `tag` constant moves both
/// sides and this differential stays green. The independent oracle for the
/// actual bytes is the integration test `value_depth_totality`, which carries
/// its own copy of the format, plus the landed conformance tests below.
///
/// ⚠ This replica **cannot** run at the integration test's depth `D` — it
/// overflows by construction, which is exactly what `AC-V1` step 2 asserts. The
/// corpus it is applied to is therefore **shallow-to-moderate on purpose**;
/// depth coverage lives in `value_depth_totality`, not here.
#[cfg(test)]
fn encode_canonical_recursive_reference(value: &Value, out: &mut Vec<u8>) {
    match value {
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
            write_u16_le(args.len().min(65535) as u16, out);
            for arg in args {
                encode_canonical_recursive_reference(arg, out);
            }
        }
        Value::Record { type_id, fields } => {
            out.push(tag::RECORD);
            write_u32_le(*type_id, out);
            write_u16_le(fields.len().min(65535) as u16, out);
            for field in fields {
                encode_canonical_recursive_reference(field, out);
            }
        }
        Value::String(s) => {
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
                encode_canonical_recursive_reference(elem, out);
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
            for (key_bytes, val) in entries {
                write_u32_le(key_bytes.len() as u32, out);
                out.extend_from_slice(key_bytes);
                encode_canonical_recursive_reference(val, out);
            }
        }
        Value::Set {
            elem_type_id,
            elements,
        } => {
            out.push(tag::SET);
            write_u32_le(*elem_type_id, out);
            write_u32_le(elements.len() as u32, out);
            for elem_bytes in elements {
                write_u32_le(elem_bytes.len() as u32, out);
                out.extend_from_slice(elem_bytes);
            }
        }
        // ⛔ No closure arm here either. `AC-V1b` compares this reference
        // encoder against the production one, so a closure arm surviving on one
        // side of that differential would be a silent asymmetry rather than a
        // leftover.
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

// ---------------------------------------------------------------------------
// `RT-FNSPLIT-B2V` `D2` — the decode inverse
// ---------------------------------------------------------------------------

/// Decode one canonical value, returning it and the bytes consumed.
///
/// ⭐ **Why this exists.** `Canonical` had an `encode_canonical` and no inverse,
/// so a `SlotId` was an identity you could mint and never redeem — which is
/// what made a handle-shaped boundary word unprojectable and what the frame's
/// `D2` amendment corrected. `Space::canonical_bytes` finds a slot's bytes;
/// this turns them back into a value.
///
/// ⛔ **Fail-closed, three outcomes not two.** `None` covers *"this decoder
/// does not cover that tag"* exactly as it covers *"those bytes are
/// malformed"*: an unhandled tag must never fall through to a plausible
/// default, because a silent wrong value is worse than a refusal. The covered
/// set is the boundary ABI's persistent family — `BigInt`, `Constructor`,
/// `Record`, `String`, `Bytes`, and the `Bool`/`SmallInt` scalars that appear in
/// sub-value position — and every other tag is refused rather than guessed.
///
/// ⛔ **`Closure` is NOT covered, and the retired `0x09` tag is refused like any
/// other unknown byte.** An ordinary closure is not persistable at all
/// (`41 §2.1`), so there is nothing for store adoption or independent recovery
/// to decode: a closure never becomes bytes in the first place. `Array`, `Map`,
/// `Set` and the remaining scalars are encodable and still **refused** here.
/// Widening the decoder stays a deliberate edit at this `match`, never a side
/// effect at a call site.
pub fn decode_canonical(bytes: &[u8]) -> Option<(Value, usize)> {
    let (&kind, rest) = bytes.split_first()?;
    let mut at = 1usize;
    let value = match kind {
        tag::BIG_INT => {
            let sign = decode_sign(*rest.first()?)?;
            at += 1;
            let count = read_u32(bytes, &mut at)? as usize;
            let mut limbs = Vec::with_capacity(count);
            for _ in 0..count {
                limbs.push(read_u64(bytes, &mut at)?);
            }
            Value::BigInt { sign, limbs }
        }
        tag::DATA => {
            let constructor_id = read_u32(bytes, &mut at)?;
            let arity = read_u16(bytes, &mut at)? as usize;
            let args = decode_children(bytes, &mut at, arity)?;
            Value::Constructor {
                constructor_id,
                args,
            }
        }
        tag::RECORD => {
            let type_id = read_u32(bytes, &mut at)?;
            let arity = read_u16(bytes, &mut at)? as usize;
            let fields = decode_children(bytes, &mut at, arity)?;
            Value::Record { type_id, fields }
        }
        tag::STRING => {
            let len = read_u32(bytes, &mut at)? as usize;
            let utf8 = bytes.get(at..at.checked_add(len)?)?;
            at += len;
            Value::String(std::str::from_utf8(utf8).ok()?.to_string())
        }
        tag::BYTES => {
            let len = read_u32(bytes, &mut at)? as usize;
            let data = bytes.get(at..at.checked_add(len)?)?;
            at += len;
            Value::Bytes(data.to_vec())
        }
        tag::BOOL => {
            let bit = *bytes.get(at)?;
            at += 1;
            match bit {
                0 => Value::Bool(false),
                1 => Value::Bool(true),
                // Not "truthy": a byte outside {0,1} is a corrupt encoding.
                _ => return None,
            }
        }
        tag::SMALL_INT => Value::SmallInt(read_u64(bytes, &mut at)? as i64),
        // ⛔ No `0x09` arm: the retired closure tag falls to the refusal below,
        // which is the point. A legacy byte stream carrying `0x09` is refused
        // rather than reconstructed — there is no closure value to reconstruct
        // it into, and inventing one would be exactly the "substitute a pointer,
        // ordinal, digest, or handle" that `41 §2.1` forbids.
        //
        // ⛔ Every other tag — including the ones this crate encodes — is
        // REFUSED, not approximated. Widening the decoder is a deliberate edit
        // here, never an accident at a call site.
        _ => return None,
    };
    Some((value, at))
}

fn decode_children(bytes: &[u8], at: &mut usize, arity: usize) -> Option<Vec<Value>> {
    let mut children = Vec::with_capacity(arity);
    for _ in 0..arity {
        let (child, used) = decode_canonical(bytes.get(*at..)?)?;
        *at += used;
        children.push(child);
    }
    Some(children)
}

fn decode_sign(byte: u8) -> Option<crate::values::Sign> {
    match byte {
        0 => Some(crate::values::Sign::NonNegative),
        1 => Some(crate::values::Sign::Negative),
        _ => None,
    }
}

fn read_u16(bytes: &[u8], at: &mut usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    let raw: [u8; 2] = bytes.get(*at..end)?.try_into().ok()?;
    *at = end;
    Some(u16::from_le_bytes(raw))
}

fn read_u32(bytes: &[u8], at: &mut usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    let raw: [u8; 4] = bytes.get(*at..end)?.try_into().ok()?;
    *at = end;
    Some(u32::from_le_bytes(raw))
}

fn read_u64(bytes: &[u8], at: &mut usize) -> Option<u64> {
    let end = at.checked_add(8)?;
    let raw: [u8; 8] = bytes.get(*at..end)?.try_into().ok()?;
    *at = end;
    Some(u64::from_le_bytes(raw))
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

    fn encode_reference(v: &Value) -> Vec<u8> {
        let mut out = Vec::new();
        encode_canonical_recursive_reference(v, &mut out);
        out
    }

    /// The `AC-V1b` differential corpus.
    ///
    /// Covers every variant, both compound orderings, empty *and* non-empty
    /// collections, nested `Map` values, and `Closure` captures (still present
    /// in Phase 1). ⚠ Shallow-to-moderate depth on purpose — see the note on
    /// [`encode_canonical_recursive_reference`].
    fn differential_corpus() -> Vec<Value> {
        let key_a = encode(&Value::String("a".into()));
        let key_b = encode(&Value::String("b".into()));

        let mut map_nested = BTreeMap::new();
        map_nested.insert(
            key_a.clone(),
            Value::Record {
                type_id: 9,
                fields: vec![Value::SmallInt(1), Value::Bytes(vec![0xAA])],
            },
        );
        map_nested.insert(
            key_b.clone(),
            Value::Array {
                elem_type_id: 3,
                elements: vec![Value::Bool(true), Value::Unknown],
            },
        );

        let mut set_ne = BTreeSet::new();
        set_ne.insert(key_a.clone());
        set_ne.insert(key_b.clone());

        vec![
            // --- scalars, every one ---
            Value::Bool(false),
            Value::Bool(true),
            Value::Char('ß'),
            Value::Float(0f64.to_bits()),
            Value::Float((-0f64).to_bits()),
            Value::Float32(1.5f32.to_bits()),
            Value::Int8(-8),
            Value::Int16(-16),
            Value::Int32(-32),
            Value::Int64(-64),
            Value::UInt8(8),
            Value::UInt16(16),
            Value::UInt32(32),
            Value::UInt64(64),
            Value::SmallInt(-1),
            Value::SmallDecimal {
                coefficient: -12345,
                exponent: -3,
            },
            Value::Unknown,
            // --- bignums, incl. the minimal-limb path ---
            Value::BigInt {
                sign: Sign::NonNegative,
                limbs: vec![0, 0, 0],
            },
            Value::BigInt {
                sign: Sign::Negative,
                limbs: vec![7, 1],
            },
            Value::BigDecimal {
                sign: Sign::Negative,
                coefficient: vec![5, 0],
                exponent: 4,
            },
            // --- flat data, empty and non-empty ---
            Value::String(String::new()),
            Value::String("e\u{0301}".into()), // decomposed: exercises NFC
            Value::Bytes(vec![]),
            Value::Bytes(vec![0, 1, 2, 255]),
            // --- compounds, empty and non-empty, both orderings ---
            Value::Constructor {
                constructor_id: 1,
                args: vec![],
            },
            Value::Constructor {
                constructor_id: 1,
                args: vec![Value::SmallInt(1), Value::String("x".into())],
            },
            Value::Record {
                type_id: 2,
                fields: vec![],
            },
            Value::Record {
                type_id: 2,
                fields: vec![Value::SmallInt(1), Value::String("x".into())],
            },
            // the reversed ordering — a distinct value, must encode distinctly
            Value::Record {
                type_id: 2,
                fields: vec![Value::String("x".into()), Value::SmallInt(1)],
            },
            Value::Array {
                elem_type_id: 3,
                elements: vec![],
            },
            Value::Array {
                elem_type_id: 3,
                elements: vec![Value::UInt8(1), Value::UInt8(2)],
            },
            Value::Map {
                key_type_id: 4,
                value_type_id: 5,
                entries: BTreeMap::new(),
            },
            Value::Map {
                key_type_id: 4,
                value_type_id: 5,
                entries: map_nested,
            },
            Value::Set {
                elem_type_id: 6,
                elements: BTreeSet::new(),
            },
            Value::Set {
                elem_type_id: 6,
                elements: set_ne,
            },
            // ⛔ The two closure entries this corpus used to carry are gone —
            // `RT-VALUE-TOTALITY-P2` is the phase their own comment named. The
            // canonical carrier has no closure variant, so there is no closure
            // for a differential over canonical encodings to compare.
            // --- moderate nesting through several child-position kinds ---
            Value::Constructor {
                constructor_id: 8,
                args: vec![Value::Array {
                    elem_type_id: 3,
                    // The nesting depth and the mix of child-position kinds are
                    // what this entry exercises, so the closure leaf is replaced
                    // rather than the entry dropped.
                    elements: vec![Value::Record {
                        type_id: 2,
                        fields: vec![Value::Unknown],
                    }],
                }],
            },
        ]
    }

    /// `AC-V1b` — the iterative emitter is byte-identical to the frozen
    /// recursive one across the whole corpus.
    ///
    /// **Operand: the SUBJECT** (the new emitter), not the detector. Perturbing
    /// the new emitter — dropping an arity prefix, reordering two children,
    /// dropping a `Map` key — reddens this.
    #[test]
    fn ac_v1b_iterative_encoding_is_byte_identical_to_the_recursive_reference() {
        let corpus = differential_corpus();
        // Non-vacuity: the corpus must actually cover every variant, or a
        // missing arm would make this differential silently narrow.
        //
        // ⛔ This asserted the literal corpus size `38`. A cardinality is a
        // **proxy** for coverage and it fails both ways: it reddened when
        // `RT-VALUE-TOTALITY-P2` legitimately removed a variant, and it would
        // have stayed green if two corpus members were swapped for two others
        // covering less. Assert the reached inventory directly instead — it is
        // the property the comment above already claims.
        let reached: std::collections::BTreeSet<u8> =
            corpus.iter().map(|value| encode(value)[0]).collect();
        assert_eq!(
            reached,
            permitted_kind_tags(),
            "corpus coverage changed — re-check variant coverage before editing"
        );
        for value in &corpus {
            assert_eq!(
                encode(value),
                encode_reference(value),
                "iterative and recursive encodings diverged for {value:?}"
            );
        }
    }

    /// Non-vacuity for the differential: the corpus must contain values whose
    /// encodings actually **differ** from one another. A corpus that collapsed
    /// to one byte string would make the equality above pass trivially.
    #[test]
    fn ac_v1b_corpus_is_non_vacuous_and_discriminating() {
        let corpus = differential_corpus();
        let mut seen = std::collections::BTreeSet::new();
        for value in &corpus {
            let bytes = encode(value);
            assert!(!bytes.is_empty(), "no value may encode to zero bytes");
            seen.insert(bytes);
        }
        // Every corpus member is a distinct value, so every encoding must be
        // distinct: the encoder is injective on this corpus.
        assert_eq!(
            seen.len(),
            corpus.len(),
            "two distinct corpus values collided — the differential would then \
             pass while covering less than it claims"
        );
    }

    /// `AC-V1b` reaches every variant.
    ///
    /// ⛔ **Asserted against the ALLOWED INVENTORY, not against a count.** This
    /// pinned the literal `25` and went red when `RT-VALUE-TOTALITY-P2` removed
    /// the closure variant — a legitimate removal reported as a regression,
    /// which is the failure mode a frozen derived count always has. It is also
    /// the *weaker* claim: a corpus that dropped `Record` and gained a second
    /// scalar keeps the count at 25 and still covers less than it says.
    ///
    /// The set below is the exact permitted inventory, so **adding** a variant
    /// without extending the corpus reddens, **removing** one without retiring
    /// its tag reddens, and **swapping** one for another reddens — none of which
    /// a cardinality can distinguish.
    #[test]
    fn ac_v1b_corpus_covers_every_value_variant() {
        let corpus = differential_corpus();
        let mut kinds = std::collections::BTreeSet::new();
        for value in &corpus {
            // The leading tag byte is the variant discriminator.
            kinds.insert(encode(value)[0]);
        }

        assert_eq!(
            kinds,
            permitted_kind_tags(),
            "the corpus's reached kind tags are not exactly the permitted \
             inventory"
        );
        assert!(
            !kinds.contains(&0x09),
            "0x09 is the RETIRED closure tag: no value on the canonical carrier \
             may encode to it"
        );
    }

    /// The exact set of kind tags a value on the canonical carrier may encode
    /// to — the **allowed inventory** both `AC-V1b` controls assert against.
    ///
    /// ⛔ `tag::CLOSURE` (`0x09`) is absent **by construction, not by
    /// omission**: the carrier has no closure variant, so no value can emit it.
    /// Re-adding a closure arm without re-adding the tag here reddens, which is
    /// the direction that matters.
    fn permitted_kind_tags() -> std::collections::BTreeSet<u8> {
        [
            tag::BIG_INT,
            tag::DATA,
            tag::RECORD,
            tag::STRING,
            tag::BYTES,
            tag::ARRAY,
            tag::MAP,
            tag::SET,
            tag::BIG_DECIMAL,
            tag::BOOL,
            tag::CHAR,
            tag::FLOAT,
            tag::FLOAT32,
            tag::INT8,
            tag::INT16,
            tag::INT32,
            tag::INT64,
            tag::UINT8,
            tag::UINT16,
            tag::UINT32,
            tag::UINT64,
            tag::SMALL_INT,
            tag::SMALL_DECIMAL,
            tag::UNKNOWN,
        ]
        .into_iter()
        .collect()
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
