//! Store index + arena + intern algorithm — per
//! `docs/design/content-addressing.md §3` and
//! `spec/40-runtime/44-capacity.md §1a,§1b`.
//!
//! This is a small-scale simulation for F4 design validation, NOT the
//! production store (K3/X2). It uses a simple Vec-based arena and a
//! Vec-based open-addressing index — no concurrency, no huge pages,
//! no mmap — sufficient to validate the algorithm's correctness and
//! scaling behavior at 10⁴–10⁶ values.

use crate::canonical::Canonical;
use crate::hash::fnv1a_64;
use crate::values::Value;

/// A slot id — the permanent identity of a distinct value.
pub type SlotId = u64;

/// Null/invalid sentinel (slot 0 is never a valid slot).
pub const NULL_SLOT: SlotId = 0;

/// Maximum load factor before resize.
const MAX_LOAD: f64 = 0.70;

/// Initial index capacity (power of two).
const INITIAL_CAPACITY: usize = 65536; // 2^16

/// An index bucket.
#[derive(Debug, Clone)]
struct Bucket {
    hash: u64,
    slot_id: SlotId,
    /// Index into `arena` where the canonical bytes live.
    canon_offset: usize,
    canon_len: u32,
    /// Whether this bucket is a tombstone (deleted).
    tombstone: bool,
}

impl Bucket {
    fn empty() -> Self {
        Self {
            hash: 0,
            slot_id: NULL_SLOT,
            canon_offset: 0,
            canon_len: 0,
            tombstone: false,
        }
    }

    fn is_occupied(&self) -> bool {
        self.slot_id != NULL_SLOT && !self.tombstone
    }
}

/// The store — index + arena + intern logic.
pub struct Store {
    buckets: Vec<Bucket>,
    occupied: usize,
    /// Arena backing store — canonical bytes are appended here.
    arena: Vec<u8>,
    /// Next slot id (monotonic, starts at 1).
    next_slot_id: SlotId,
    /// Total intern() calls.
    total_interns: u64,
    /// intern() calls that returned an existing slot.
    dedup_hits: u64,
    /// Capacity limit (0 = none).
    capacity_limit: u64,
}

/// Result of an intern operation.
#[derive(Debug, PartialEq, Eq)]
pub enum InternResult {
    /// A new slot was created.
    New(SlotId),
    /// The value already existed at this slot.
    Hit(SlotId),
    /// The store is at capacity.
    CapacityExhausted {
        limit: u64,
        current: u64,
    },
}

/// Store statistics (the witness surface).
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_slots: u64,
    pub total_interns: u64,
    pub dedup_hits: u64,
    pub arena_bytes: u64,
    pub index_buckets: u64,
    pub index_load: f64,
}

impl Store {
    /// Create a new store.
    pub fn new() -> Self {
        Self {
            buckets: vec![Bucket::empty(); INITIAL_CAPACITY],
            occupied: 0,
            arena: Vec::with_capacity(INITIAL_CAPACITY * 256), // start with ~16 MiB
            next_slot_id: 1,
            total_interns: 0,
            dedup_hits: 0,
            capacity_limit: 0,
        }
    }

    /// Create a store with an artificial capacity limit (for loud-at-limit test).
    pub fn with_capacity_limit(limit: u64) -> Self {
        let mut store = Self::new();
        store.capacity_limit = limit;
        store
    }

    /// Intern a compound value. Returns the slot id (new or existing).
    ///
    /// This is the algorithm from `docs/design/content-addressing.md §3.4`.
    pub fn intern(&mut self, value: &Value) -> InternResult {
        assert!(value.is_compound(), "only compounds are interned");

        // 1. Canonical encode.
        let mut canon_bytes = Vec::new();
        value.encode_canonical(&mut canon_bytes);

        // 2. Hash.
        let hash = fnv1a_64(&canon_bytes);

        // 3. Probe.
        let cap = self.buckets.len();
        let mut idx = (hash as usize) & (cap - 1);

        loop {
            let bucket = &self.buckets[idx];

            if bucket.slot_id == NULL_SLOT && !bucket.tombstone {
                // Empty — new slot.
                if self.capacity_limit > 0
                    && (self.next_slot_id - 1) >= self.capacity_limit
                {
                    self.total_interns += 1;
                    return InternResult::CapacityExhausted {
                        limit: self.capacity_limit,
                        current: self.next_slot_id - 1,
                    };
                }

                // Maybe resize before inserting.
                if (self.occupied + 1) as f64 >= self.buckets.len() as f64 * MAX_LOAD
                {
                    self.resize();
                    // Recompute probe after resize — hash is the same, capacity doubled.
                    return self.intern(value);
                }

                let slot_id = self.next_slot_id;
                self.next_slot_id += 1;
                let canon_offset = self.arena.len();
                self.arena.extend_from_slice(&canon_bytes);

                self.buckets[idx] = Bucket {
                    hash,
                    slot_id,
                    canon_offset,
                    canon_len: canon_bytes.len() as u32,
                    tombstone: false,
                };
                self.occupied += 1;
                self.total_interns += 1;

                return InternResult::New(slot_id);
            }

            if bucket.hash == hash
                && !bucket.tombstone
                && bucket.canon_len as usize == canon_bytes.len()
            {
                // Potential hit — memcmp.
                let stored = &self.arena
                    [bucket.canon_offset..bucket.canon_offset + bucket.canon_len as usize];
                if stored == canon_bytes.as_slice() {
                    self.total_interns += 1;
                    self.dedup_hits += 1;
                    return InternResult::Hit(bucket.slot_id);
                }
                // Collision — continue probing.
            }

            // Linear probe.
            idx = (idx + 1) & (cap - 1);
        }
    }

    /// Get the canonical bytes for a slot.
    pub fn get_canonical(&self, slot_id: SlotId) -> Option<&[u8]> {
        if slot_id == NULL_SLOT {
            return None;
        }
        for bucket in &self.buckets {
            if bucket.slot_id == slot_id && !bucket.tombstone {
                return Some(
                    &self.arena
                        [bucket.canon_offset..bucket.canon_offset + bucket.canon_len as usize],
                );
            }
        }
        None
    }

    /// Reset the store — release all slots, reclaim arena memory.
    pub fn reset(&mut self) {
        self.buckets.fill(Bucket::empty());
        self.occupied = 0;
        self.arena.clear();
        // next_slot_id is NOT reset — slot ids are permanently retired.
        // total_interns and dedup_hits persist for the witness.
    }

    /// Return current store statistics.
    pub fn stats(&self) -> StoreStats {
        let occupied = self.buckets.iter().filter(|b| b.is_occupied()).count();
        StoreStats {
            total_slots: self.next_slot_id - 1,
            total_interns: self.total_interns,
            dedup_hits: self.dedup_hits,
            arena_bytes: self.arena.len() as u64,
            index_buckets: self.buckets.len() as u64,
            index_load: occupied as f64 / self.buckets.len() as f64,
        }
    }

    /// Number of distinct values currently stored.
    pub fn distinct_count(&self) -> usize {
        self.buckets.iter().filter(|b| b.is_occupied()).count()
    }

    /// Resize the index (double capacity, rehash).
    fn resize(&mut self) {
        let old_buckets = std::mem::take(&mut self.buckets);
        let new_cap = old_buckets.len() * 2;
        self.buckets = vec![Bucket::empty(); new_cap];
        self.occupied = 0;

        for bucket in &old_buckets {
            if !bucket.is_occupied() {
                continue;
            }
            let mut idx = (bucket.hash as usize) & (new_cap - 1);
            loop {
                let b = &self.buckets[idx];
                if b.slot_id == NULL_SLOT && !b.tombstone {
                    self.buckets[idx] = bucket.clone();
                    self.occupied += 1;
                    break;
                }
                idx = (idx + 1) & (new_cap - 1);
            }
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Sign imported implicitly via Value

    #[test]
    fn intern_dedup_same_value_same_slot() {
        let mut store = Store::new();
        let v = Value::String("hello".into());
        let r1 = store.intern(&v);
        let r2 = store.intern(&v);
        // The intern results are New(1) and Hit(1) — different variants,
        // but the slot id within them must be the same.
        let id1 = match r1 {
            InternResult::New(id) => id,
            _ => unreachable!(),
        };
        let id2 = match r2 {
            InternResult::Hit(id) => id,
            _ => unreachable!(),
        };
        assert_eq!(id1, id2);
        let stats = store.stats();
        assert!(stats.dedup_hits >= 1);
    }

    #[test]
    fn intern_distinct_values_different_slots() {
        let mut store = Store::new();
        let v1 = Value::String("hello".into());
        let v2 = Value::String("world".into());
        let r1 = store.intern(&v1);
        let r2 = store.intern(&v2);
        assert!(matches!(r1, InternResult::New(_)));
        assert!(matches!(r2, InternResult::New(_)));
        match (r1, r2) {
            (InternResult::New(s1), InternResult::New(s2)) => {
                assert_ne!(s1, s2);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn loud_at_limit() {
        let mut store = Store::with_capacity_limit(5);
        for i in 0..5 {
            let v = Value::String(format!("val{}", i));
            assert!(matches!(store.intern(&v), InternResult::New(_)));
        }
        let v = Value::String("val5".into());
        match store.intern(&v) {
            InternResult::CapacityExhausted { limit, current: _ } => {
                assert_eq!(limit, 5);
            }
            _ => panic!("expected CapacityExhausted"),
        }
    }

    #[test]
    fn reset_releases_memory() {
        let mut store = Store::new();
        let v = Value::String("hello".into());
        store.intern(&v);
        let arena_before = store.stats().arena_bytes;
        assert!(arena_before > 0);
        store.reset();
        let stats = store.stats();
        assert_eq!(stats.arena_bytes, 0);
    }

    #[test]
    fn reset_retires_slot_ids() {
        let mut store = Store::new();
        let v1 = Value::String("a".into());
        let r1 = store.intern(&v1);
        let old_id = match r1 {
            InternResult::New(id) => id,
            _ => unreachable!(),
        };
        store.reset();
        let v2 = Value::String("b".into());
        let r2 = store.intern(&v2);
        match r2 {
            InternResult::New(id) => assert!(id > old_id),
            _ => unreachable!(),
        }
    }

    #[test]
    fn equality_is_slot_id_compare() {
        let mut store = Store::new();
        let v = Value::String("test".into());
        let s1 = match store.intern(&v) {
            InternResult::New(id) => id,
            _ => unreachable!(),
        };
        let s2 = match store.intern(&v) {
            InternResult::Hit(id) => id,
            _ => unreachable!(),
        };
        assert_eq!(s1, s2); // O(1) integer compare
    }

    #[test]
    fn map_ordering_shares_slot() {
        use std::collections::BTreeMap;
        let mut store = Store::new();

        // Build map with "keys" as distinct interned strings, then use their
        // canonical bytes as Map keys. In the real implementation, a Map's keys
        // would be interned values whose canonical bytes become the BTreeMap key.
        // For this bench, we pre-encode the key values.

        let key1_val = Value::String("k1".into());
        let key2_val = Value::String("k2".into());

        let mut kb1 = Vec::new();
        key1_val.encode_canonical(&mut kb1);
        let mut kb2 = Vec::new();
        key2_val.encode_canonical(&mut kb2);

        // Map A: insert k2 then k1
        let mut entries_a = BTreeMap::new();
        entries_a.insert(kb2.clone(), Value::SmallInt(2));
        entries_a.insert(kb1.clone(), Value::SmallInt(1));

        // Map B: insert k1 then k2
        let mut entries_b = BTreeMap::new();
        entries_b.insert(kb1.clone(), Value::SmallInt(1));
        entries_b.insert(kb2.clone(), Value::SmallInt(2));

        let map_a = Value::Map {
            key_type_id: 1,
            value_type_id: 2,
            entries: entries_a,
        };
        let map_b = Value::Map {
            key_type_id: 1,
            value_type_id: 2,
            entries: entries_b,
        };

        let r1 = store.intern(&map_a);
        let r2 = store.intern(&map_b);

        // Both should produce the same slot — Map ordering is canonical.
        let id1 = match r1 {
            InternResult::New(id) => id,
            InternResult::Hit(id) => id,  // may already exist from prior test
            _ => unreachable!(),
        };
        let id2 = match r2 {
            InternResult::Hit(id) => id,
            _ => unreachable!(),
        };
        assert_eq!(id1, id2);
    }
}
