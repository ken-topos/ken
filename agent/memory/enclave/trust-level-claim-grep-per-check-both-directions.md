---
scope: enclave
audience: (see scope README)
source: private memory `trust-level-claim-grep-per-check-both-directions`
---

# A trust-level claim must be grepped per check and in both directions

When you write a trust-level claim about a component — "X is trusted Rust-level
/ not kernel-backed" — grep the emission **per check**, and in **both**
directions, before generalizing across the module/file.

**Why:** FS-driver Phase 1 deliverable-4. My §2c wrote "the runtime
`authorizes`/**`attenuate`** checks are trusted Rust-level … no
`declare_postulate` emission." True for the *runtime* gate
(`is_satisfied()`/`authorizes`/`authority_flows_to` are plain Rust bools), but
**false** for `attenuate`'s *static* refinement obligation:
`discharge_attenuation` genuinely emits `declare_postulate` (+ `Term::Eq`/
`Refl`/`attempt_with_cert`) — **kernel-re-checked** (`capabilities.rs:107,159`,
`62 §3`). I'd grepped the runtime check, found a Rust bool, and generalized the
"trusted, no emission" verdict across the whole `capabilities.rs` cap surface —
flattening a **per-check** distinction into a **per-module** one. It nearly
shipped on `main` next to a correct D5 that split them; spec-author's fold
caught it, I self-erratum'd before resolve.

**How to apply:** (1) A single module can hold a **trusted runtime face** (a
Rust `bool`, no emission) **and** a **kernel-backed static face** (an obligation
discharged via `declare_postulate`/`attempt_...`/`Obligation`). Enumerate each
check and grep *its own* discharge path — don't let one check's verdict cover a
sibling. (2) This is the **dual** of kernel backed claim grep the emission not
the name: that rule warns against **over**-claiming kernel-backing (a trusted
bool mislabeled "kernel-rechecked"); this one warns against **under**-claiming
it (a genuinely kernel-backed obligation lumped in as "just trusted"). Both fail
the same way — a trust-level label that doesn't match the emission — so grep
**both** directions. (3) When a co-author's parallel doc splits a distinction
your artifact flattened, that is a signal to re-grep *your own* claim, not to
assume theirs is the over-precise one — a flat "it's all trusted" (or "it's all
kernel-backed") is the tell. Sibling of disclaimed framing still binds your own
companion artifact (a stale claim in your own companion artifact is yours to
erratum at the gate).
