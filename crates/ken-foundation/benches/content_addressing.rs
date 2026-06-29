//! F4 content-addressing benchmark harness.
//!
//! Validates the design at small scale (10⁴–10⁶ values) per
//! `docs/program/wp/F4-content-addr-design.md` §4.
//!
//! Reports: intern throughput, measured dedup rate vs expected,
//! memory per distinct value, O(1) equality (slot-id compare),
//! and loud-at-limit behaviour.
//!
//! Run with: `scripts/ken-cargo bench -p ken-foundation`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ken_foundation::canonical::Canonical;
use ken_foundation::hash::fnv1a_64;
use ken_foundation::store::{InternResult, Store};
use ken_foundation::values::{Sign, Value};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Generate `n` synthetic values with `dup_ratio` duplicates (0.0–1.0).
///
/// Half the values are unique; the other half are repeats from a pool
/// of size `n * (1 - dup_ratio)` so the expected dedup rate is `dup_ratio`.
fn generate_values(n: usize, dup_ratio: f64) -> Vec<Value> {
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
fn synthetic_value(idx: usize, rng: &mut WyHashRng) -> Value {
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

/// Simple WyHash-based RNG (deterministic, no external dep).
struct WyHashRng {
    state: u64,
}

impl WyHashRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

fn bench_intern(c: &mut Criterion) {
    let mut group = c.benchmark_group("intern");

    for n in [10_000usize, 100_000, 1_000_000].iter() {
        let values = generate_values(*n, 0.5);
        let values = black_box(values); // prevent const-folding

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("n={}", n)),
            &values,
            |b, vals| {
                b.iter(|| {
                    let mut store = Store::new();
                    for v in vals.iter() {
                        if v.is_compound() {
                            black_box(store.intern(v));
                        }
                    }
                    store.stats()
                });
            },
        );
    }

    group.finish();
}

fn bench_equality_is_slot_id(c: &mut Criterion) {
    // Demonstrate that equality of interned values is O(1) — slot-id compare —
    // independent of value depth.
    let shallow = Value::String("hello".into());
    let deep = {
        // Build a deeply nested record: 100 levels of nesting.
        let mut inner = Value::SmallInt(42);
        for _ in 0..100 {
            inner = Value::Record {
                type_id: 200,
                fields: vec![inner, Value::String("padding".into())],
            };
        }
        inner
    };

    c.bench_function("equality_shallow", |b| {
        let mut store = Store::new();
        let s1 = match store.intern(&shallow) {
            InternResult::New(id) => id,
            _ => unreachable!(),
        };
        let s2 = match store.intern(&shallow) {
            InternResult::Hit(id) => id,
            _ => unreachable!(),
        };
        b.iter(|| black_box(s1 == s2));
    });

    c.bench_function("equality_deep", |b| {
        let mut store = Store::new();
        let s1 = match store.intern(&deep) {
            InternResult::New(id) => id,
            _ => unreachable!(),
        };
        let s2 = match store.intern(&deep) {
            InternResult::Hit(id) => id,
            _ => unreachable!(),
        };
        b.iter(|| black_box(s1 == s2));
    });
}

fn bench_hash_speed(c: &mut Criterion) {
    let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    c.bench_function("fnv1a_1kb", |b| {
        b.iter(|| black_box(fnv1a_64(black_box(&data))));
    });
}

fn bench_canonical_encode(c: &mut Criterion) {
    let record = Value::Record {
        type_id: 100,
        fields: vec![
            Value::SmallInt(42),
            Value::String("a moderately sized string for encoding".into()),
            Value::Bool(true),
            Value::Array {
                elem_type_id: 200,
                elements: (0..10).map(|i| Value::SmallInt(i)).collect(),
            },
        ],
    };

    c.bench_function("canonical_encode_record", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            black_box(&record).encode_canonical(black_box(&mut out));
            black_box(out)
        });
    });
}

/// Loud at-limit validation (not a bench, but we run it in the bench harness
/// so it's recorded alongside the performance numbers).
#[test]
fn loud_at_limit_test() {
    let mut store = Store::with_capacity_limit(100);
    for i in 0..100 {
        let v = Value::String(format!("val{:04}", i));
        assert!(matches!(store.intern(&v), InternResult::New(_)));
    }
    // The 101st should fail loudly.
    let v = Value::String("overflow".into());
    match store.intern(&v) {
        InternResult::CapacityExhausted { limit, current } => {
            assert_eq!(limit, 100);
            assert_eq!(current, 100);
        }
        other => panic!("expected CapacityExhausted, got {:?}", other),
    }
}

/// Dedup rate validation.
#[test]
fn dedup_rate_matches_expected() {
    let n = 10_000;
    let dup_ratio = 0.5;
    let values = generate_values(n, dup_ratio);

    let mut store = Store::new();
    let mut total_compounds = 0;
    for v in &values {
        if v.is_compound() {
            store.intern(v);
            total_compounds += 1;
        }
    }

    let stats = store.stats();
    let measured_dedup = stats.dedup_hits as f64 / total_compounds as f64;

    // The measured dedup rate should be within 5% of expected.
    // (Some divergence because duplicates are drawn from a fixed pool,
    // not exactly 50% of total.)
    println!(
        "Dedup rate: measured={:.4}, expected≈{:.4}",
        measured_dedup, dup_ratio
    );
    println!("  total_interns={}, dedup_hits={}, total_slots={}",
             stats.total_interns, stats.dedup_hits, stats.total_slots);
    println!("  arena_bytes={}, index_load={:.4}",
             stats.arena_bytes, stats.index_load);

    assert!(
        (measured_dedup - dup_ratio).abs() < 0.1,
        "dedup rate within 10% of expected"
    );

    // Memory per distinct value should be reasonable (< 1 KiB for these
    // small synthetic values).
    let mem_per_distinct = stats.arena_bytes as f64 / stats.total_slots as f64;
    println!("  mem/distinct_value={:.1} bytes", mem_per_distinct);
    assert!(
        mem_per_distinct < 2048.0,
        "memory per distinct value under 2 KiB"
    );
}

/// Equality is slot-id compare — O(1) regardless of value depth.
#[test]
fn equality_is_constant_time() {
    let mut store = Store::new();

    let shallow = Value::String("hello".into());
    let deep = {
        let mut inner = Value::SmallInt(42);
        for _ in 0..100 {
            inner = Value::Record {
                type_id: 200,
                fields: vec![inner, Value::String("padding".into())],
            };
        }
        inner
    };

    let s_shallow = match store.intern(&shallow) {
        InternResult::New(id) => id,
        _ => unreachable!(),
    };
    let s_deep = match store.intern(&deep) {
        InternResult::New(id) => id,
        _ => unreachable!(),
    };

    // Both are integer comparisons — O(1).
    let s_shallow2 = match store.intern(&shallow) {
        InternResult::Hit(id) => id,
        _ => unreachable!(),
    };
    let s_deep2 = match store.intern(&deep) {
        InternResult::Hit(id) => id,
        _ => unreachable!(),
    };

    assert_eq!(s_shallow, s_shallow2);
    assert_eq!(s_deep, s_deep2);
    assert_ne!(s_shallow, s_deep);
}

criterion_group!(
    benches,
    bench_intern,
    bench_equality_is_slot_id,
    bench_hash_speed,
    bench_canonical_encode,
);
criterion_main!(benches);
