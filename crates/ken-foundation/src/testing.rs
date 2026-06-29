//! Testing utilities for content-addressing benchmarks and acceptance
//! tests — not part of the public API.
//!
//! These are `#[doc(hidden)]` because they exist only to support the
//! bench harness (`benches/content_addressing.rs`) and acceptance tests
//! (`tests/acceptance.rs`). Production code must not depend on them.

use crate::canonical::Canonical;
use crate::values::{Sign, Value};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Simple WyHash-based RNG (deterministic, no external dep).
#[doc(hidden)]
pub struct WyHashRng {
    state: u64,
}

#[doc(hidden)]
impl WyHashRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

/// Generate `n` synthetic values with `dup_ratio` duplicates (0.0–1.0).
///
/// Half the values are unique; the other half are repeats from a pool
/// of size `n * (1 - dup_ratio)` so the expected dedup rate is
/// `dup_ratio`.
#[doc(hidden)]
pub fn generate_values(n: usize, dup_ratio: f64) -> Vec<Value> {
    let unique_count = (n as f64 * (1.0 - dup_ratio)) as usize;
    let mut values = Vec::with_capacity(n);
    let mut rng = WyHashRng::new(42);

    // Generate the unique pool.
    for i in 0..unique_count {
        values.push(synthetic_value(i, &mut rng));
    }

    // Fill the rest with repeats from the pool.
    while values.len() < n {
        let idx = rng.next() as usize % unique_count;
        values.push(values[idx].clone());
    }

    values
}

/// Generate a synthetic value of varying kind based on index.
#[doc(hidden)]
pub fn synthetic_value(idx: usize, rng: &mut WyHashRng) -> Value {
    match idx % 10 {
        0 => {
            // String
            let len = 4 + (rng.next() as usize % 32);
            let s: String = (0..len)
                .map(|_| (b'a' + (rng.next() % 26) as u8) as char)
                .collect();
            Value::String(s)
        }
        1 => {
            // Small record (2 fields)
            Value::Record {
                type_id: 100,
                fields: vec![
                    Value::SmallInt(rng.next() as i64),
                    Value::Bool(rng.next() % 2 == 0),
                ],
            }
        }
        2 => {
            // Bytes
            let len = 8 + (rng.next() as usize % 48);
            let data: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
            Value::Bytes(data)
        }
        3 => {
            // Small Array
            let n = 2 + (rng.next() as usize % 5);
            Value::Array {
                elem_type_id: 200,
                elements: (0..n)
                    .map(|_| Value::SmallInt(rng.next() as i64))
                    .collect(),
            }
        }
        4 => {
            // Bignum (overflowed Int)
            Value::BigInt {
                sign: Sign::NonNegative,
                limbs: vec![rng.next(), rng.next()],
            }
        }
        5 => {
            // Constructor application
            Value::Constructor {
                constructor_id: 300,
                args: vec![
                    Value::SmallInt(rng.next() as i64),
                    Value::String(format!("arg{}", rng.next() % 100)),
                ],
            }
        }
        6 => {
            // Map with 2–4 entries
            let n_entries = 2 + (rng.next() as usize % 3);
            let mut entries = BTreeMap::new();
            for k in 0..n_entries {
                let key_val = Value::String(format!("mapkey{}_{}", idx, k));
                let mut key_bytes = Vec::new();
                key_val.encode_canonical(&mut key_bytes);
                entries.insert(key_bytes, Value::SmallInt(rng.next() as i64));
            }
            Value::Map {
                key_type_id: 1,
                value_type_id: 2,
                entries,
            }
        }
        7 => {
            // Set with 2–4 elements
            let n_elems = 2 + (rng.next() as usize % 3);
            let mut elements = BTreeSet::new();
            for e in 0..n_elems {
                let elem_val = Value::String(format!("setelem{}_{}", idx, e));
                let mut elem_bytes = Vec::new();
                elem_val.encode_canonical(&mut elem_bytes);
                elements.insert(elem_bytes);
            }
            Value::Set {
                elem_type_id: 10,
                elements,
            }
        }
        8 => {
            // Closure with captured environment
            Value::Closure {
                code_id: rng.next(),
                captured: vec![
                    Value::SmallInt(rng.next() as i64),
                    Value::String(format!("cap{}", rng.next() % 1000)),
                ],
            }
        }
        9 => {
            // Record (3+ fields)
            Value::Record {
                type_id: 101,
                fields: vec![
                    Value::SmallInt(rng.next() as i64),
                    Value::String(format!("f{}", rng.next() % 1000)),
                    Value::Bool(rng.next() % 2 == 0),
                ],
            }
        }
        _ => unreachable!(),
    }
}
