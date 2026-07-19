//! Compiler-private exact `Int` carrier and invocation arena.
//!
//! `RuntimeIntV1` is the semantic image used by Runtime IR. `NativeIntV1` is
//! its two-word native transport: Small values remain unboxed and Big values
//! name a canonical `Value::BigInt` in one invocation-scoped arena.

use crate::{Canonical, Sign, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

pub const NATIVE_INT_SMALL_TAG_V1: u64 = 0;
pub const NATIVE_INT_BIG_TAG_V1: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIntV1 {
    Small(i64),
    Big { sign: Sign, limbs: Vec<u64> },
}

impl From<i64> for RuntimeIntV1 {
    fn from(value: i64) -> Self {
        Self::Small(value)
    }
}

impl RuntimeIntV1 {
    pub fn from_canonical_parts(sign: Sign, limbs: Vec<u64>) -> Self {
        let (sign, limbs) = normalize(sign, limbs);
        match signed_magnitude_to_i64(sign, &limbs) {
            Some(value) => Self::Small(value),
            None => Self::Big { sign, limbs },
        }
    }

    pub fn canonical_big_image(&self) -> Value {
        let (sign, limbs) = self.sign_magnitude();
        Value::BigInt { sign, limbs }
    }

    pub fn add(&self, rhs: &Self) -> Self {
        exact_binop(self, rhs, add_signed)
    }

    pub fn sub(&self, rhs: &Self) -> Self {
        let (sign, limbs) = rhs.sign_magnitude();
        let negated = if is_zero(&limbs) {
            Sign::NonNegative
        } else {
            opposite(sign)
        };
        let rhs = Self::Big {
            sign: negated,
            limbs,
        };
        exact_binop(self, &rhs, add_signed)
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        let (lhs_sign, lhs) = self.sign_magnitude();
        let (rhs_sign, rhs) = rhs.sign_magnitude();
        let limbs = mul_magnitude(&lhs, &rhs);
        let sign = if is_zero(&limbs) || lhs_sign == rhs_sign {
            Sign::NonNegative
        } else {
            Sign::Negative
        };
        Self::from_canonical_parts(sign, limbs)
    }

    pub fn exact_cmp(&self, rhs: &Self) -> Ordering {
        let (lhs_sign, lhs) = self.sign_magnitude();
        let (rhs_sign, rhs) = rhs.sign_magnitude();
        match (lhs_sign, rhs_sign) {
            (Sign::Negative, Sign::NonNegative) => Ordering::Less,
            (Sign::NonNegative, Sign::Negative) => Ordering::Greater,
            (Sign::NonNegative, Sign::NonNegative) => cmp_magnitude(&lhs, &rhs),
            (Sign::Negative, Sign::Negative) => cmp_magnitude(&rhs, &lhs),
        }
    }

    pub fn checked_to_u64(&self) -> Option<u64> {
        let (sign, limbs) = self.sign_magnitude();
        if sign == Sign::Negative || limbs.len() != 1 {
            return None;
        }
        Some(limbs[0])
    }

    pub fn as_small(&self) -> Option<i64> {
        match self {
            Self::Small(value) => Some(*value),
            Self::Big { .. } => None,
        }
    }

    fn sign_magnitude(&self) -> (Sign, Vec<u64>) {
        match self {
            Self::Small(value) if *value < 0 => (Sign::Negative, vec![value.unsigned_abs()]),
            Self::Small(value) => (Sign::NonNegative, vec![*value as u64]),
            Self::Big { sign, limbs } => normalize(*sign, limbs.clone()),
        }
    }
}

impl fmt::Display for RuntimeIntV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Small(value) => write!(formatter, "{value}"),
            Self::Big { sign, limbs } => {
                if *sign == Sign::Negative {
                    formatter.write_str("-")?;
                }
                formatter.write_str("0x")?;
                for (index, limb) in limbs.iter().rev().enumerate() {
                    if index == 0 {
                        write!(formatter, "{limb:x}")?;
                    } else {
                        write!(formatter, "{limb:016x}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NativeBigSlotV1(u64);

impl NativeBigSlotV1 {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct NativeIntV1 {
    pub tag: u64,
    pub payload: u64,
}

#[derive(Default)]
pub struct NativeIntArenaV1 {
    entries: Vec<Value>,
    canonical_slots: BTreeMap<Vec<u8>, NativeBigSlotV1>,
}

impl NativeIntArenaV1 {
    pub fn intern(&mut self, value: &RuntimeIntV1) -> NativeIntV1 {
        if let RuntimeIntV1::Small(value) = value {
            return NativeIntV1 {
                tag: NATIVE_INT_SMALL_TAG_V1,
                payload: *value as u64,
            };
        }
        let image = value.canonical_big_image();
        let mut canonical = Vec::new();
        image.encode_canonical(&mut canonical);
        let slot = if let Some(slot) = self.canonical_slots.get(&canonical) {
            *slot
        } else {
            self.entries.push(image);
            let slot = NativeBigSlotV1(self.entries.len() as u64);
            self.canonical_slots.insert(canonical, slot);
            slot
        };
        NativeIntV1 {
            tag: NATIVE_INT_BIG_TAG_V1,
            payload: slot.get(),
        }
    }

    pub fn resolve(&self, value: NativeIntV1) -> Option<RuntimeIntV1> {
        match value.tag {
            NATIVE_INT_SMALL_TAG_V1 => Some(RuntimeIntV1::Small(value.payload as i64)),
            NATIVE_INT_BIG_TAG_V1 => {
                let index = usize::try_from(value.payload).ok()?.checked_sub(1)?;
                match self.entries.get(index)? {
                    Value::BigInt { sign, limbs } => Some(RuntimeIntV1::Big {
                        sign: *sign,
                        limbs: limbs.clone(),
                    }),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// JIT-side implementation of the compiler-private exact-Int support call.
/// Linked artifacts provide the same private ABI from their generated runtime
/// support object; neither form is a Ken host operation.
#[doc(hidden)]
pub unsafe extern "C" fn native_int_binop_v1(
    arena: *mut NativeIntArenaV1,
    operation: u64,
    lhs_tag: u64,
    lhs_payload: u64,
    rhs_tag: u64,
    rhs_payload: u64,
    output: *mut NativeIntV1,
) -> i64 {
    let (Some(arena), Some(output)) = (arena.as_mut(), output.as_mut()) else {
        return -1;
    };
    let Some(lhs) = arena.resolve(NativeIntV1 {
        tag: lhs_tag,
        payload: lhs_payload,
    }) else {
        return -1;
    };
    let Some(rhs) = arena.resolve(NativeIntV1 {
        tag: rhs_tag,
        payload: rhs_payload,
    }) else {
        return -1;
    };
    let result = match operation {
        0 => lhs.add(&rhs),
        1 => lhs.sub(&rhs),
        2 => lhs.mul(&rhs),
        _ => return -1,
    };
    *output = arena.intern(&result);
    0
}

#[doc(hidden)]
pub unsafe extern "C" fn native_int_compare_v1(
    arena: *mut NativeIntArenaV1,
    operation: u64,
    lhs_tag: u64,
    lhs_payload: u64,
    rhs_tag: u64,
    rhs_payload: u64,
) -> i64 {
    let Some(arena) = arena.as_ref() else {
        return -1;
    };
    let Some(lhs) = arena.resolve(NativeIntV1 {
        tag: lhs_tag,
        payload: lhs_payload,
    }) else {
        return -1;
    };
    let Some(rhs) = arena.resolve(NativeIntV1 {
        tag: rhs_tag,
        payload: rhs_payload,
    }) else {
        return -1;
    };
    match operation {
        0 => i64::from(lhs == rhs),
        1 => i64::from(lhs.exact_cmp(&rhs).is_le()),
        _ => -1,
    }
}

#[doc(hidden)]
pub unsafe extern "C" fn native_int_narrow_u64_v1(
    arena: *mut NativeIntArenaV1,
    tag: u64,
    payload: u64,
    output: *mut u64,
) -> i64 {
    let (Some(arena), Some(output)) = (arena.as_ref(), output.as_mut()) else {
        return -1;
    };
    let Some(value) = arena.resolve(NativeIntV1 { tag, payload }) else {
        return -1;
    };
    let Some(value) = value.checked_to_u64() else {
        return 1;
    };
    *output = value;
    0
}

fn exact_binop(
    lhs: &RuntimeIntV1,
    rhs: &RuntimeIntV1,
    op: fn(Sign, &[u64], Sign, &[u64]) -> (Sign, Vec<u64>),
) -> RuntimeIntV1 {
    let (lhs_sign, lhs) = lhs.sign_magnitude();
    let (rhs_sign, rhs) = rhs.sign_magnitude();
    let (sign, limbs) = op(lhs_sign, &lhs, rhs_sign, &rhs);
    RuntimeIntV1::from_canonical_parts(sign, limbs)
}

fn add_signed(lhs_sign: Sign, lhs: &[u64], rhs_sign: Sign, rhs: &[u64]) -> (Sign, Vec<u64>) {
    if lhs_sign == rhs_sign {
        return normalize(lhs_sign, add_magnitude(lhs, rhs));
    }
    match cmp_magnitude(lhs, rhs) {
        Ordering::Greater => normalize(lhs_sign, sub_magnitude(lhs, rhs)),
        Ordering::Less => normalize(rhs_sign, sub_magnitude(rhs, lhs)),
        Ordering::Equal => (Sign::NonNegative, vec![0]),
    }
}

fn add_magnitude(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    let mut out = Vec::with_capacity(lhs.len().max(rhs.len()) + 1);
    let mut carry = 0u128;
    for index in 0..lhs.len().max(rhs.len()) {
        let sum = u128::from(*lhs.get(index).unwrap_or(&0))
            + u128::from(*rhs.get(index).unwrap_or(&0))
            + carry;
        out.push(sum as u64);
        carry = sum >> 64;
    }
    if carry != 0 {
        out.push(carry as u64);
    }
    out
}

fn sub_magnitude(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    let mut out = Vec::with_capacity(lhs.len());
    let mut borrow = 0u128;
    for index in 0..lhs.len() {
        let left = u128::from(lhs[index]);
        let right = u128::from(*rhs.get(index).unwrap_or(&0)) + borrow;
        if left >= right {
            out.push((left - right) as u64);
            borrow = 0;
        } else {
            out.push(((1u128 << 64) + left - right) as u64);
            borrow = 1;
        }
    }
    debug_assert_eq!(borrow, 0);
    out
}

fn mul_magnitude(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    let mut out = vec![0; lhs.len() + rhs.len()];
    for (left_index, &left) in lhs.iter().enumerate() {
        let mut carry = 0u128;
        for (right_index, &right) in rhs.iter().enumerate() {
            let index = left_index + right_index;
            let product = u128::from(left) * u128::from(right) + u128::from(out[index]) + carry;
            out[index] = product as u64;
            carry = product >> 64;
        }
        let mut index = left_index + rhs.len();
        while carry != 0 {
            let sum = u128::from(out[index]) + carry;
            out[index] = sum as u64;
            carry = sum >> 64;
            index += 1;
        }
    }
    normalize(Sign::NonNegative, out).1
}

fn cmp_magnitude(lhs: &[u64], rhs: &[u64]) -> Ordering {
    let lhs = minimal_limbs(lhs);
    let rhs = minimal_limbs(rhs);
    lhs.len()
        .cmp(&rhs.len())
        .then_with(|| lhs.iter().rev().cmp(rhs.iter().rev()))
}

fn signed_magnitude_to_i64(sign: Sign, limbs: &[u64]) -> Option<i64> {
    let limbs = minimal_limbs(limbs);
    if limbs.len() != 1 {
        return None;
    }
    match sign {
        Sign::NonNegative => i64::try_from(limbs[0]).ok(),
        Sign::Negative if limbs[0] == 1u64 << 63 => Some(i64::MIN),
        Sign::Negative => i64::try_from(limbs[0]).ok().map(|value| -value),
    }
}

fn normalize(sign: Sign, mut limbs: Vec<u64>) -> (Sign, Vec<u64>) {
    while limbs.len() > 1 && limbs.last() == Some(&0) {
        limbs.pop();
    }
    if limbs.is_empty() {
        limbs.push(0);
    }
    let sign = if is_zero(&limbs) {
        Sign::NonNegative
    } else {
        sign
    };
    (sign, limbs)
}

fn minimal_limbs(limbs: &[u64]) -> &[u64] {
    let end = limbs
        .iter()
        .rposition(|limb| *limb != 0)
        .map_or(1, |index| index + 1);
    &limbs[..end.min(limbs.len().max(1))]
}

fn is_zero(limbs: &[u64]) -> bool {
    limbs.iter().all(|limb| *limb == 0)
}

fn opposite(sign: Sign) -> Sign {
    match sign {
        Sign::NonNegative => Sign::Negative,
        Sign::Negative => Sign::NonNegative,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_arithmetic_promotes_and_canonicalizes() {
        let promoted = RuntimeIntV1::Small(i64::MAX).add(&RuntimeIntV1::Small(1));
        assert_eq!(
            promoted,
            RuntimeIntV1::Big {
                sign: Sign::NonNegative,
                limbs: vec![1u64 << 63]
            }
        );
        assert_eq!(
            promoted.sub(&RuntimeIntV1::Small(1)),
            RuntimeIntV1::Small(i64::MAX)
        );
    }

    #[test]
    fn multiplication_has_no_i128_ceiling() {
        let huge = RuntimeIntV1::from_canonical_parts(Sign::NonNegative, vec![0, 0, 1]);
        let product = huge.mul(&huge);
        assert_eq!(
            product,
            RuntimeIntV1::Big {
                sign: Sign::NonNegative,
                limbs: vec![0, 0, 0, 0, 1]
            }
        );
    }

    #[test]
    fn comparisons_use_every_limb() {
        let low_equal = RuntimeIntV1::from_canonical_parts(Sign::NonNegative, vec![7, 1]);
        let different = RuntimeIntV1::from_canonical_parts(Sign::NonNegative, vec![7, 2]);
        assert_ne!(low_equal, different);
        assert_eq!(low_equal.exact_cmp(&different), Ordering::Less);
    }

    #[test]
    fn arena_slots_are_nonzero_content_identities() {
        let value = RuntimeIntV1::from_canonical_parts(Sign::NonNegative, vec![0, 1]);
        let mut arena = NativeIntArenaV1::default();
        let first = arena.intern(&value);
        let second = arena.intern(&value);
        assert_eq!(first, second);
        assert_eq!(first.tag, NATIVE_INT_BIG_TAG_V1);
        assert_ne!(first.payload, 0);
        assert_eq!(arena.resolve(first), Some(value));
        assert_eq!(arena.len(), 1);
    }
}
