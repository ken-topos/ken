# Capacity conformance — moved to `seed-capacity.md`

> **The capacity cases moved.** The store-capacity commitments (loud refusal,
> dedup-aware accounting, reclamation, region-scoped lifetime, the lattice
> non-dependency) are now pinned at **X2 rigor** in **`seed-capacity.md`**,
> grounded on the elaborated `spec/40-runtime/44-capacity.md §1–§6` and the
> landed per-`space` store (`crates/ken-runtime/src/store.rs`). See that file
> for AC1–AC5 + the supporting §1a/§1b/§3 mechanics.
>
> The F4 corpus that lived here is **subsumed** there (one home per property).
> Two details it carried were **corrected** by the X2 landed-code reconcile and
> are fixed in `seed-capacity.md`:
>
> - the loud error is `CapacityExhausted { limit, current }` — there is **no
>   `arena` id** field (a future addition once spaces are named, `44 §2`);
> - reclamation is observed via **`StoreStats.arena_bytes → 0`, not RSS** (which
>   is allocator-dependent, `44 §3`).
