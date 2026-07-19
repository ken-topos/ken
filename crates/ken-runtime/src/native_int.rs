//! Compiler-private exact `Int` carrier and invocation arena.
//!
//! `RuntimeIntV1` is the semantic image used by Runtime IR. `NativeIntV1` is
//! its two-word native transport: Small values remain unboxed and Big values
//! name a canonical `Value::BigInt` in one invocation-scoped arena.

use crate::{Sign, Value};
use std::cmp::Ordering;
use std::fmt;
use std::ptr;

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

#[repr(C)]
struct NativeBigEntryV1 {
    next: *mut NativeBigEntryV1,
    slot: u64,
    sign: u64,
    len: u64,
    limbs: [u64; 0],
}

/// C-compatible invocation header shared by every generated execution lane.
#[repr(C)]
pub struct NativeIntArenaV1 {
    head: *mut NativeBigEntryV1,
    next_slot: u64,
    final_tag: u64,
    final_payload: u64,
    final_sign: u64,
    final_len: u64,
    final_limbs: *const u64,
    final_small: u64,
}

impl Default for NativeIntArenaV1 {
    fn default() -> Self {
        Self {
            head: ptr::null_mut(),
            next_slot: 0,
            final_tag: u64::MAX,
            final_payload: 0,
            final_sign: 0,
            final_len: 0,
            final_limbs: ptr::null(),
            final_small: 0,
        }
    }
}

impl NativeIntArenaV1 {
    pub fn resolve(&self, value: NativeIntV1) -> Option<RuntimeIntV1> {
        match value.tag {
            NATIVE_INT_SMALL_TAG_V1 => Some(RuntimeIntV1::Small(value.payload as i64)),
            NATIVE_INT_BIG_TAG_V1 => {
                if value.payload == 0 {
                    return None;
                }
                let mut entry = self.head;
                while !entry.is_null() {
                    // SAFETY: entries are allocated and linked only by the
                    // generated local helper graph and remain owned by this
                    // arena until `drop`.
                    let current = unsafe { &*entry };
                    if current.slot == value.payload {
                        let len = usize::try_from(current.len).ok()?;
                        let limbs =
                            unsafe { std::slice::from_raw_parts(current.limbs.as_ptr(), len) }
                                .to_vec();
                        let sign = match current.sign {
                            0 => Sign::NonNegative,
                            1 => Sign::Negative,
                            _ => return None,
                        };
                        return Some(RuntimeIntV1::from_canonical_parts(sign, limbs));
                    }
                    entry = current.next;
                }
                None
            }
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        usize::try_from(self.next_slot).unwrap_or(usize::MAX)
    }

    pub fn final_result(&self) -> Option<NativeIntV1> {
        (self.final_tag != u64::MAX).then_some(NativeIntV1 {
            tag: self.final_tag,
            payload: self.final_payload,
        })
    }
}

unsafe extern "C" {
    fn free(pointer: *mut std::ffi::c_void);
}

impl Drop for NativeIntArenaV1 {
    fn drop(&mut self) {
        let mut entry = self.head;
        while !entry.is_null() {
            let next = unsafe { (*entry).next };
            unsafe { free(entry.cast()) };
            entry = next;
        }
        self.head = ptr::null_mut();
    }
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
    fn arena_header_matches_the_generated_local_helper_layout() {
        assert_eq!(
            std::mem::size_of::<NativeIntArenaV1>(),
            crate::native_int_clif::ARENA_BYTES
        );
    }

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
}
