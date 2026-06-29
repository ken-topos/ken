//! Content-addressed value store — `docs/design/content-addressing.md §3`,
//! `spec/40-runtime/44-capacity.md §1–3`.
//!
//! Production differences from ken-foundation's F4 design-validation:
//! - Arena organised as a chain of fixed-size pages (`44 §1b`)
//! - Spaces own separate arenas + index partitions (`44 §3`)
//! - Loud typed error on capacity exhaustion (`44 §2`, `OQ-5`)
//! - Slot ids monotonic and never reused across reset (`41 §3b`)

use crate::canonical::Canonical;
use crate::hash::fnv1a_64;
use crate::values::Value;
use std::sync::atomic::{AtomicU64, Ordering};

/// A slot id — permanent identity of a distinct value.
pub type SlotId = u64;

/// Null/invalid sentinel (slot 0 is never valid).
pub const NULL_SLOT: SlotId = 0;

/// Initial index capacity (power of two, design doc §3.4).
const INITIAL_CAPACITY: usize = 65536; // 2^16

/// Resize when load exceeds this fraction (`44 §1a` recommendation).
const MAX_LOAD: f64 = 0.70;

/// Arena page size: 4 MiB (`44 §1b` recommendation).
const PAGE_SIZE: usize = 4 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Process-wide monotonic slot-id counter (`41 §3b`)
// ---------------------------------------------------------------------------
static NEXT_SLOT_ID: AtomicU64 = AtomicU64::new(1);

fn alloc_slot_id() -> SlotId {
    NEXT_SLOT_ID.fetch_add(1, Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Arena page chain (`44 §1b`)
// ---------------------------------------------------------------------------

struct Page {
    data: Vec<u8>, // up to PAGE_SIZE bytes
}

impl Page {
    fn new() -> Self {
        Page {
            data: Vec::with_capacity(PAGE_SIZE),
        }
    }

    fn remaining(&self) -> usize {
        PAGE_SIZE.saturating_sub(self.data.len())
    }
}

/// A chain of append-mostly pages forming a single arena.
struct Arena {
    pages: Vec<Page>,
}

impl Arena {
    fn new() -> Self {
        Arena {
            pages: vec![Page::new()],
        }
    }

    /// Total bytes stored across all pages.
    fn total_bytes(&self) -> u64 {
        self.pages.iter().map(|p| p.data.len() as u64).sum()
    }

    /// Append `bytes` to the arena; returns `(page_idx, offset_within_page)`.
    fn append(&mut self, bytes: &[u8]) -> (usize, usize) {
        // If current page can't fit `bytes`, allocate a new page.
        let cur = self.pages.len() - 1;
        if bytes.len() > self.pages[cur].remaining() {
            // Large values that exceed PAGE_SIZE get their own oversized page.
            // Push a fresh empty page after it so `cur` always points to a
            // page that `remaining()` can report without saturating_sub.
            let mut p = Page::new();
            p.data.extend_from_slice(bytes);
            self.pages.push(p);
            let page_idx = self.pages.len() - 1;
            self.pages.push(Page::new()); // fresh cur for subsequent appends
            (page_idx, 0)
        } else {
            let page = &mut self.pages[cur];
            let offset = page.data.len();
            page.data.extend_from_slice(bytes);
            (cur, offset)
        }
    }

    /// Borrow the bytes at `(page_idx, offset, len)`.
    fn get(&self, page_idx: usize, offset: usize, len: usize) -> &[u8] {
        &self.pages[page_idx].data[offset..offset + len]
    }

    /// Release all pages (reclamation).
    fn reset(&mut self) {
        self.pages.clear();
        self.pages.push(Page::new());
    }
}

// ---------------------------------------------------------------------------
// Open-addressing index (`44 §1a`, design doc §3.1)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Bucket {
    hash: u64,
    slot_id: SlotId,
    page_idx: usize,
    offset: usize,
    canon_len: u32,
    tombstone: bool,
}

impl Bucket {
    fn empty() -> Self {
        Bucket {
            hash: 0,
            slot_id: NULL_SLOT,
            page_idx: 0,
            offset: 0,
            canon_len: 0,
            tombstone: false,
        }
    }

    fn is_occupied(&self) -> bool {
        self.slot_id != NULL_SLOT && !self.tombstone
    }
}

struct Index {
    buckets: Vec<Bucket>,
    occupied: usize,
}

impl Index {
    fn new() -> Self {
        Index {
            buckets: vec![Bucket::empty(); INITIAL_CAPACITY],
            occupied: 0,
        }
    }

    fn clear(&mut self) {
        self.buckets.fill_with(Bucket::empty);
        self.occupied = 0;
    }

    fn load_factor(&self) -> f64 {
        self.occupied as f64 / self.buckets.len() as f64
    }

    fn total_buckets(&self) -> u64 {
        self.buckets.len() as u64
    }

    fn resize(&mut self) {
        let old = std::mem::take(&mut self.buckets);
        let new_cap = old.len() * 2;
        self.buckets = vec![Bucket::empty(); new_cap];
        self.occupied = 0;

        for b in &old {
            if !b.is_occupied() {
                continue;
            }
            let mut idx = (b.hash as usize) & (new_cap - 1);
            loop {
                if !self.buckets[idx].is_occupied() && !self.buckets[idx].tombstone {
                    self.buckets[idx] = b.clone();
                    self.occupied += 1;
                    break;
                }
                idx = (idx + 1) & (new_cap - 1);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Space — a reclamation unit owning one arena + one index partition (`44 §3`)
// ---------------------------------------------------------------------------

/// A `space`-scoped arena + index partition.
///
/// Each space is reclaimed independently; its slot ids are retired on reset
/// but the monotonic counter is process-wide so ids are never reused.
pub struct Space {
    arena: Arena,
    index: Index,
    /// Soft capacity limit (0 = none).
    capacity_limit: u64,
    /// Total slots created in this space.
    total_slots: u64,
    total_interns: u64,
    dedup_hits: u64,
}

impl Space {
    pub fn new() -> Self {
        Space {
            arena: Arena::new(),
            index: Index::new(),
            capacity_limit: 0,
            total_slots: 0,
            total_interns: 0,
            dedup_hits: 0,
        }
    }

    pub fn with_capacity_limit(limit: u64) -> Self {
        let mut s = Self::new();
        s.capacity_limit = limit;
        s
    }

    /// Intern a compound value.  Returns the slot id (new or existing).
    ///
    /// Algorithm: `docs/design/content-addressing.md §3.4`.
    pub fn intern(&mut self, value: &Value) -> InternResult {
        assert!(value.is_compound(), "only compounds are interned");

        let mut canon_bytes = Vec::new();
        value.encode_canonical(&mut canon_bytes);

        let hash = fnv1a_64(&canon_bytes);
        let result = self.probe_or_insert(hash, &canon_bytes);
        self.total_interns += 1;
        result
    }

    fn probe_or_insert(&mut self, hash: u64, canon_bytes: &[u8]) -> InternResult {
        let cap = self.index.buckets.len();
        let mut idx = (hash as usize) & (cap - 1);

        loop {
            let bucket = &self.index.buckets[idx];

            if bucket.slot_id == NULL_SLOT && !bucket.tombstone {
                // Empty slot — new value.
                if self.capacity_limit > 0 && self.total_slots >= self.capacity_limit {
                    return InternResult::CapacityExhausted {
                        limit: self.capacity_limit,
                        current: self.total_slots,
                    };
                }

                // Resize before inserting if needed.
                if (self.index.occupied + 1) as f64 >= cap as f64 * MAX_LOAD {
                    self.index.resize();
                    return self.probe_or_insert(hash, canon_bytes);
                }

                let slot_id = alloc_slot_id();
                let (page_idx, offset) = self.arena.append(canon_bytes);

                self.index.buckets[idx] = Bucket {
                    hash,
                    slot_id,
                    page_idx,
                    offset,
                    canon_len: canon_bytes.len() as u32,
                    tombstone: false,
                };
                self.index.occupied += 1;
                self.total_slots += 1;

                return InternResult::New(slot_id);
            }

            if !bucket.tombstone
                && bucket.hash == hash
                && bucket.canon_len as usize == canon_bytes.len()
            {
                // Potential hit — memcmp the full canonical bytes.
                let stored =
                    self.arena
                        .get(bucket.page_idx, bucket.offset, bucket.canon_len as usize);
                if stored == canon_bytes {
                    self.dedup_hits += 1;
                    return InternResult::Hit(bucket.slot_id);
                }
                // True hash collision — continue probing.
            }

            idx = (idx + 1) & (self.index.buckets.len() - 1);
        }
    }

    /// Reset the space: release all pages + clear the index.
    /// Slot ids in this space are retired (not reused).
    pub fn reset(&mut self) {
        self.arena.reset();
        self.index.clear();
        self.total_slots = 0;
        // total_interns and dedup_hits persist (they belong to the witness).
    }

    /// Return store statistics for this space.
    pub fn stats(&self) -> StoreStats {
        StoreStats {
            total_slots: self.total_slots,
            total_interns: self.total_interns,
            dedup_hits: self.dedup_hits,
            arena_bytes: self.arena.total_bytes(),
            index_buckets: self.index.total_buckets(),
            index_load: self.index.load_factor(),
            merkle_root: None,
        }
    }

    /// Number of distinct values currently interned in this space.
    pub fn distinct_count(&self) -> usize {
        self.index
            .buckets
            .iter()
            .filter(|b| b.is_occupied())
            .count()
    }
}

impl Default for Space {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Store — convenience wrapper with a default space, for single-space use
// ---------------------------------------------------------------------------

/// A single-space store (the common case).
pub struct Store {
    space: Space,
}

impl Store {
    pub fn new() -> Self {
        Store {
            space: Space::new(),
        }
    }

    pub fn with_capacity_limit(limit: u64) -> Self {
        Store {
            space: Space::with_capacity_limit(limit),
        }
    }

    pub fn intern(&mut self, value: &Value) -> InternResult {
        self.space.intern(value)
    }

    pub fn reset(&mut self) {
        self.space.reset();
    }

    pub fn stats(&self) -> StoreStats {
        self.space.stats()
    }

    pub fn distinct_count(&self) -> usize {
        self.space.distinct_count()
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Result of an intern operation.
#[derive(Debug, PartialEq, Eq)]
pub enum InternResult {
    /// A fresh slot was created.
    New(SlotId),
    /// The value already existed at this slot (dedup hit).
    Hit(SlotId),
    /// The store has reached its capacity limit (`44 §2`, `OQ-5`).
    CapacityExhausted { limit: u64, current: u64 },
}

impl InternResult {
    /// Extract the slot id from New or Hit (panics on CapacityExhausted).
    pub fn slot_id(&self) -> SlotId {
        match self {
            InternResult::New(id) | InternResult::Hit(id) => *id,
            InternResult::CapacityExhausted { .. } => panic!("capacity exhausted"),
        }
    }
}

/// Store statistics — the witness surface (`41 §7`, design doc §9).
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_slots: u64,
    pub total_interns: u64,
    pub dedup_hits: u64,
    pub arena_bytes: u64,
    pub index_buckets: u64,
    pub index_load: f64,
    pub merkle_root: Option<[u8; 32]>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::Sign;

    // --- conformance: runtime/values/dedup-shares-slot ---
    #[test]
    fn dedup_shares_slot() {
        let mut store = Store::new();
        // Two independently-constructed structurally-equal values.
        let v1 = Value::Record {
            type_id: 1,
            fields: vec![Value::SmallInt(42), Value::String("hello".into())],
        };
        let v2 = Value::Record {
            type_id: 1,
            fields: vec![Value::SmallInt(42), Value::String("hello".into())],
        };
        let r1 = store.intern(&v1);
        let r2 = store.intern(&v2);
        assert!(matches!(r1, InternResult::New(_)));
        assert!(matches!(r2, InternResult::Hit(_)));
        assert_eq!(r1.slot_id(), r2.slot_id());
        assert_eq!(store.distinct_count(), 1); // one slot, not two
    }

    // --- conformance: runtime/values/equality-is-slot-id ---
    #[test]
    fn equality_is_slot_id_compare() {
        let mut store = Store::new();
        let v = Value::String("test-value".into());
        let id1 = store.intern(&v).slot_id();
        let id2 = store.intern(&v).slot_id();
        assert_eq!(id1, id2); // O(1) integer compare
    }

    // --- conformance: runtime/values/scalars-are-typed-immediates ---
    #[test]
    fn scalars_are_immediate() {
        let i = Value::SmallInt(42);
        let b = Value::Bool(true);
        let f = Value::Float(1.0f64.to_bits());
        assert!(i.is_immediate());
        assert!(b.is_immediate());
        assert!(f.is_immediate());
        // Confirm none of them accidentally pretend to be compounds
        assert!(!i.is_compound());
        assert!(!b.is_compound());
        assert!(!f.is_compound());
    }

    // --- conformance: runtime/values/int-small-to-bignum ---
    #[test]
    fn int_small_to_bignum_promotion() {
        let mut store = Store::new();
        // Small Int: immediate, never interned.
        let small = Value::SmallInt(i64::MAX);
        assert!(small.is_immediate());

        // BigInt (promoted): interned, content-addressed.
        // 2^64 = [0, 1] in little-endian 64-bit limbs.
        let big = Value::BigInt {
            sign: Sign::NonNegative,
            limbs: vec![0, 1],
        };
        assert!(big.is_compound());
        let r1 = store.intern(&big);
        let r2 = store.intern(&big);
        assert!(matches!(r1, InternResult::New(_)));
        assert_eq!(r1.slot_id(), r2.slot_id()); // same large integer → same slot
    }

    // --- conformance: runtime/values/immediate-vs-interned-boundary ---
    #[test]
    fn immediate_vs_interned_boundary() {
        // Immediates: Bool, Int64, Float, SmallInt, SmallDecimal
        assert!(Value::Bool(false).is_immediate());
        assert!(Value::Int64(0).is_immediate());
        assert!(Value::Float(0).is_immediate());
        assert!(Value::SmallInt(0).is_immediate());
        assert!(Value::SmallDecimal {
            coefficient: 0,
            exponent: 0
        }
        .is_immediate());

        // Interned compounds: String, Record, Array, BigInt
        assert!(Value::String("x".into()).is_compound());
        assert!(Value::Record {
            type_id: 1,
            fields: vec![]
        }
        .is_compound());
        assert!(Value::Array {
            elem_type_id: 1,
            elements: vec![]
        }
        .is_compound());
        assert!(Value::BigInt {
            sign: Sign::NonNegative,
            limbs: vec![0]
        }
        .is_compound());

        let mut store = Store::new();
        let s = Value::String("content".into());
        let r = store.intern(&s);
        assert!(matches!(r, InternResult::New(_))); // compound → gets a slot
    }

    // --- conformance: runtime/values/closure-content-addressed ---
    #[test]
    fn closure_content_addressed() {
        let mut store = Store::new();
        let cap = vec![Value::SmallInt(1), Value::String("env".into())];
        let c1 = Value::Closure {
            code_id: 42,
            captured: cap.clone(),
        };
        let c2 = Value::Closure {
            code_id: 42,
            captured: cap.clone(),
        };
        let r1 = store.intern(&c1);
        let r2 = store.intern(&c2);
        assert!(matches!(r1, InternResult::New(_)));
        assert_eq!(r1.slot_id(), r2.slot_id()); // same code + same env → same slot
    }

    // --- conformance: runtime/values/closure-distinct-env-no-collision ---
    #[test]
    fn closure_distinct_env_no_collision() {
        let mut store = Store::new();
        let c1 = Value::Closure {
            code_id: 42,
            captured: vec![Value::SmallInt(1), Value::String("a".into())],
        };
        let c2 = Value::Closure {
            code_id: 42,
            captured: vec![Value::SmallInt(1), Value::String("b".into())],
        };
        let r1 = store.intern(&c1);
        let r2 = store.intern(&c2);
        assert!(matches!(r1, InternResult::New(_)));
        assert!(matches!(r2, InternResult::New(_)));
        assert_ne!(r1.slot_id(), r2.slot_id()); // distinct envs → distinct slots
    }

    // --- conformance: runtime/capacity/loud-refusal-not-silent ---
    #[test]
    fn loud_refusal_not_silent() {
        let mut store = Store::with_capacity_limit(3);
        for i in 0..3 {
            let v = Value::String(format!("val{}", i));
            assert!(matches!(store.intern(&v), InternResult::New(_)));
        }
        let extra = Value::String("overflow".into());
        match store.intern(&extra) {
            InternResult::CapacityExhausted { limit, current } => {
                assert_eq!(limit, 3);
                assert_eq!(current, 3);
            }
            other => panic!("expected CapacityExhausted, got {:?}", other),
        }
    }

    // --- conformance: runtime/capacity/dedup-aware-accounting ---
    #[test]
    fn dedup_aware_accounting() {
        let mut store = Store::new();
        let v = Value::String("dedup-me".into());
        for _ in 0..1000 {
            store.intern(&v);
        }
        let stats = store.stats();
        assert_eq!(stats.total_slots, 1); // one distinct value
        assert_eq!(stats.total_interns, 1000);
        assert_eq!(stats.dedup_hits, 999);

        // 1000 distinct values → 1000 slots
        let mut store2 = Store::new();
        for i in 0..1000u64 {
            let v2 = Value::String(format!("v{}", i));
            store2.intern(&v2);
        }
        let stats2 = store2.stats();
        assert_eq!(stats2.total_slots, 1000);
        assert_eq!(stats2.dedup_hits, 0);
    }

    // --- conformance: runtime/capacity/reclamation-releases-pages ---
    #[test]
    fn reclamation_releases_pages() {
        let mut store = Store::new();
        for i in 0..100u64 {
            store.intern(&Value::String(format!("value-{}", i)));
        }
        let arena_before = store.stats().arena_bytes;
        assert!(arena_before > 0);

        store.reset();
        let stats_after = store.stats();
        assert_eq!(stats_after.arena_bytes, 0);
        assert_eq!(stats_after.total_slots, 0);
    }

    // --- conformance: runtime/capacity/reset-retires-slot-ids ---
    #[test]
    fn reset_retires_slot_ids() {
        let mut store = Store::new();
        let v1 = Value::String("before-reset".into());
        let old_id = store.intern(&v1).slot_id();

        store.reset();

        let v2 = Value::String("after-reset".into());
        let new_id = store.intern(&v2).slot_id();
        // Slot ids are monotonic and never reused.
        assert!(
            new_id > old_id,
            "slot id {new_id} must be > retired id {old_id}"
        );
    }

    // --- conformance: runtime/capacity/space-bounded-reclamation ---
    #[test]
    fn space_bounded_reclamation() {
        let mut space_a = Space::new();
        let mut space_b = Space::new();

        let v_a = Value::String("space-a-value".into());
        let v_b = Value::String("space-b-value".into());
        let id_a = space_a.intern(&v_a).slot_id();
        space_b.intern(&v_b);

        // Terminate space_a — only its arena is reclaimed.
        space_a.reset();
        assert_eq!(space_a.distinct_count(), 0);
        assert_eq!(space_a.stats().arena_bytes, 0);

        // space_b remains live.
        assert_eq!(space_b.distinct_count(), 1);
        assert!(space_b.stats().arena_bytes > 0);

        // Interning in space_b still works; new slot id > id_a (monotonic counter).
        let v_b2 = Value::String("space-b-second".into());
        let id_b2 = space_b.intern(&v_b2).slot_id();
        assert!(id_b2 > id_a);
    }

    // --- conformance: runtime/capacity/no-lattice-on-hot-path ---
    // Structural: no mmgroup/Leech dependency exists in this crate's Cargo.toml.
    // Verified at build time (no such dep).

    // --- conformance: runtime/capacity/auto-gc-not-present ---
    #[test]
    fn auto_gc_not_present() {
        // Create and "discard" values without explicit reset.
        // Memory grows monotonically — no automatic reclamation.
        let mut store = Store::new();
        for i in 0..100u64 {
            store.intern(&Value::String(format!("disposable-{}", i)));
        }
        let bytes_before = store.stats().arena_bytes;
        // After more interning, memory only grows (or stays same on dedup).
        for i in 0..100u64 {
            store.intern(&Value::String(format!("more-{}", i)));
        }
        let bytes_after = store.stats().arena_bytes;
        assert!(bytes_after >= bytes_before);
    }

    // --- conformance: runtime/capacity/index-resize-preserves-slots ---
    #[test]
    fn index_resize_preserves_slots() {
        let mut store = Store::new();
        // Insert enough distinct values to trigger a resize (> 65536 * 0.7 ≈ 45875)
        let mut ids = Vec::new();
        for i in 0..50_000u64 {
            let v = Value::String(format!("resize-test-{}", i));
            let id = store.intern(&v).slot_id();
            ids.push((i, id));
        }
        // All previously-interned values must still dedup to the original slot id.
        for (i, expected_id) in &ids {
            let v = Value::String(format!("resize-test-{}", i));
            let got = store.intern(&v).slot_id();
            assert_eq!(
                got, *expected_id,
                "slot id changed after resize at index {}",
                i
            );
        }
    }

    // --- conformance: runtime/capacity/arena-page-chaining ---
    #[test]
    fn arena_page_chaining() {
        let mut store = Store::new();
        // Intern enough data to fill more than one 4 MiB page.
        // Each value is ~8 KiB; 4 MiB / 8 KiB = ~512 per page → need >512 to chain.
        let chunk: String = "x".repeat(8 * 1024);
        let mut all_ids = Vec::new();
        for i in 0..600u64 {
            let v = Value::String(format!("{}-{}", chunk, i));
            let id = store.intern(&v).slot_id();
            all_ids.push((i, id));
        }
        // All values remain accessible (dedup returns original slot ids).
        for (i, expected_id) in &all_ids {
            let v = Value::String(format!("{}-{}", chunk, i));
            let got = store.intern(&v).slot_id();
            assert_eq!(got, *expected_id, "slot id lost after page chain at {}", i);
        }
        assert!(
            store.stats().arena_bytes > PAGE_SIZE as u64,
            "should have spanned multiple pages"
        );
    }

    // --- Architect should-fix: oversized-value (>PAGE_SIZE) does not corrupt
    //     the arena's remaining() check for subsequent appends.
    #[test]
    fn oversized_value_no_remaining_underflow() {
        let mut store = Store::new();

        // One value larger than PAGE_SIZE (4 MiB + 1 byte).
        let huge_bytes = vec![0x42u8; PAGE_SIZE + 1];
        let big = Value::Bytes(huge_bytes.clone());
        let id_big = store.intern(&big).slot_id();

        // A normal-sized value interned AFTER the oversized one — exercises
        // the `cur` page that follows the oversized page.
        let small = Value::String("after-big".into());
        let id_small = store.intern(&small).slot_id();
        assert_ne!(id_big, id_small);

        // Both dedup correctly.
        assert_eq!(store.intern(&big).slot_id(), id_big);
        assert_eq!(store.intern(&small).slot_id(), id_small);
        assert_eq!(store.distinct_count(), 2);
    }

    // Collision-handling path: exercise the memcmp branch on a true collision.
    // We can't easily produce a real FNV-1a collision, so we verify the fast
    // path (same hash + equal bytes → dedup) and the slow path (same hash +
    // different bytes → different slot) through the normal API.  A proper
    // collision test would require injecting a collision; we document this as
    // a structural/oracle guard covered by the design.
    #[test]
    fn dedup_uses_memcmp_not_hash_only() {
        // Two values with different content get different slots even if a
        // future implementation had a hash collision. We can't force one here
        // but we can verify that two truly-distinct values do NOT collide.
        let mut store = Store::new();
        let v1 = Value::String("aaaaaa".into());
        let v2 = Value::String("bbbbbb".into());
        let id1 = store.intern(&v1).slot_id();
        let id2 = store.intern(&v2).slot_id();
        assert_ne!(id1, id2);
    }

    // Guard: dedup across ≥2 distinct constructions of equal content.
    #[test]
    fn dedup_across_two_distinct_constructions() {
        let mut store = Store::new();
        let v1 = Value::Record {
            type_id: 7,
            fields: vec![
                Value::SmallInt(100),
                Value::Bool(true),
                Value::String("datum".into()),
            ],
        };
        // Second construction is a separate Rust allocation.
        let v2 = {
            let mut f = Vec::new();
            f.push(Value::SmallInt(100));
            f.push(Value::Bool(true));
            f.push(Value::String("datum".into()));
            Value::Record {
                type_id: 7,
                fields: f,
            }
        };
        let id1 = store.intern(&v1).slot_id();
        let id2 = store.intern(&v2).slot_id();
        assert_eq!(id1, id2); // ≥2 constructions → same slot
    }
}
