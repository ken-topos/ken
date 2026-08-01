//! F4 acceptance tests — content-addressing design validation.
//!
//! Exercises the design at test time (not bench time) to ensure the
//! deduplication, loud-at-limit, and O(1) equality properties hold.
//! Uses the shared `testing` module for synthetic-value generation.

use ken_foundation::store::{InternResult, Store};
use ken_foundation::testing::generate_values;
use ken_foundation::values::Value;

// ---------------------------------------------------------------------------
// 1. Dedup rate validation
// ---------------------------------------------------------------------------

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

    // The measured dedup rate should be within 10% of expected.
    // (Some divergence because duplicates are drawn from a fixed pool,
    // not exactly 50% of total.)
    assert!(
        (measured_dedup - dup_ratio).abs() < 0.1,
        "dedup rate within 10% of expected (got {:.4}, expected {:.4})",
        measured_dedup,
        dup_ratio
    );

    // Memory per distinct value should be reasonable (< 2 KiB for these
    // small synthetic values).
    let mem_per_distinct = stats.arena_bytes as f64 / stats.total_slots as f64;
    assert!(
        mem_per_distinct < 2048.0,
        "memory per distinct value under 2 KiB (got {:.1})",
        mem_per_distinct
    );

    eprintln!(
        "Dedup rate: measured={:.4}, expected≈{:.4}",
        measured_dedup, dup_ratio
    );
    eprintln!(
        "  total_interns={}, dedup_hits={}, total_slots={}",
        stats.total_interns, stats.dedup_hits, stats.total_slots
    );
    eprintln!(
        "  arena_bytes={}, index_load={:.4}",
        stats.arena_bytes, stats.index_load
    );
    eprintln!(
        "  mem/distinct_value={:.1} bytes",
        mem_per_distinct
    );
}

// ---------------------------------------------------------------------------
// 2. Loud at-limit validation
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 3. Equality is O(1) — slot-id compare regardless of value depth
// ---------------------------------------------------------------------------

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
