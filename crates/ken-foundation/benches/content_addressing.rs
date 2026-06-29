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
//!
//! Note: the three validation tests previously in this file (`loud_at_limit_test`,
//! `dedup_rate_matches_expected`, `equality_is_constant_time`) were dead code
//! (this bench has `harness = false`). They have been moved to
//! `tests/acceptance.rs` and use the shared `testing` module.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ken_foundation::canonical::Canonical;
use ken_foundation::hash::fnv1a_64;
use ken_foundation::store::{InternResult, Store};
use ken_foundation::testing::generate_values;
use ken_foundation::values::Value;

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

// ── The three validation tests below were dead code (this bench has
//    `harness = false` in Cargo.toml). They now live in `tests/acceptance.rs`
//    and use the shared `ken_foundation::testing` module. ──
//
// #[test]
// fn loud_at_limit_test() { ... }
// #[test]
// fn dedup_rate_matches_expected() { ... }
// #[test]
// fn equality_is_constant_time() { ... }

criterion_group!(
    benches,
    bench_intern,
    bench_equality_is_slot_id,
    bench_hash_speed,
    bench_canonical_encode,
);
criterion_main!(benches);
