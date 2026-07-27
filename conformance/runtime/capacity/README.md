# Capacity conformance — moved to `seed-capacity.md`

> **The capacity cases moved.** The resource-profile commitments (loud refusal,
> profile-declared accounting, semantics-invisible reclamation, logical-space
> lifetime, and the lattice non-dependency) are now pinned at **X2 rigor** in
> **`seed-capacity.md`**,
> grounded on the elaborated `spec/40-runtime/44-capacity.md §1–§6` and the
> landed per-`space` store (`crates/ken-runtime/src/store.rs`). See that file
> for AC1–AC5 + private-growth safety.
>
> The F4 corpus that lived here is **subsumed** there (one home per property).
> Its prior fixed error payload, store statistics, page-release observation,
> and reset mechanics are private after `SPEC-STORE-SPLIT`. The replacement
> cases retain loud typed failure and semantics-invisible lifetime behavior
> without prescribing those shapes.
