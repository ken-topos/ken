---
id: ABI-REVOKE
title: "runtime revocation membrane — the deferred runtime face of 62 §4"
status: draft
owner: TBD
size: TBD
gate: none
depends_on: []
blocks: []
github: null
origin: docs/program/09-posix-linux-abi-campaign.md §5 (charter gap); split out per operator directive 2026-07-22; absent from docs/program/10-linux-abi-completion.md
---

**Split out on the operator's instruction (2026-07-22).** The charter said
*"fold into PX7 (it is the same handle-lifetime machinery) or split out if PX7
grows."* **It is not the same machinery** (§2), and PX7 has landed, so the
fold-in option is gone regardless.

> ### ⛔ THIS IS NOT SHOVEL-READY AND MUST NOT BE RELEASED TO A BUILD TEAM
>
> `owner: TBD` / `size: TBD` are honest, not placeholders I forgot to fill.
> **Grounding it turned up a blocking prerequisite the charter does not
> mention** (§3), and until that is resolved by the Architect there is no
> mechanism to brief. **Route to the Architect for a design pass and an ADR
> before any sizing.** Do not let a leader pull this.

## 1. What exists today, measured

`crates/ken-elaborator/src/capabilities.rs:248-282`:

```rust
pub struct RevocationHandle { pub revoked: bool }
impl RevocationHandle { fn new() -> Self; fn revoke(&mut self); }
pub fn check_revocation_transitive(handle: &RevocationHandle) -> bool {
    !handle.revoked
}
```

Its own doc comment (`:250-255`) states the position exactly:

> **Static contract:** revoking the parent revokes the parent AND every
> capability attenuated from it (transitivity). The runtime membrane
> (forwarder / validity-cell flip) is **DEFERRED** to `40-runtime`/`OQ-Space`.

**Transitivity is by construction** — all attenuated caps share the parent's
handle, so one `revoke()` closes the sub-delegation. That is a real property
and it is genuinely delivered. **It is also the whole of what is delivered.**

**Revocation appears in exactly two files in `crates/`:**
`ken-elaborator/src/capabilities.rs` and
`ken-elaborator/tests/sec2_acceptance.rs`. There is **zero** revocation code in
`ken-host`, `ken-runtime`, `ken-interp`, or `catalog/`.

> ⚠ **The charter's citation is STALE.** `09:495-496` cites
> `capabilities.rs:468-471` for the struct and `:464-467` for the doc comment.
> The actual locations are **`:256-258`** and **`:250-255`**. Re-verify before
> quoting; the file has moved since the charter was written. Every anchor in a
> program document is perishable.

## 2. ⛔ PX7 is NOT this, and the charter's fold-in advice was wrong

The charter proposed folding this into PX7 as *"the same handle-lifetime
machinery."* **It is a different property:**

| | PX7's generation-checked handle table | the revocation membrane |
|---|---|---|
| guards against | **use-after-close** — a stale handle id reused for a new resource | **withdrawal of delegated authority** — a still-valid handle whose grantor revoked it |
| the handle is | invalid because the resource is gone | **valid**, and must nonetheless be denied |
| detection | generation counter mismatch | validity of a controlling cell, checked transitively |

A generation check cannot express revocation: after `revoke()`, the OS resource
is still open and the handle still names it. Nothing about the generation has
changed. **Conflating them would have produced a membrane that fails exactly
where it is needed.**

## 3. ★ THE BLOCKING PREREQUISITE THE CHARTER DOES NOT NAME

`spec/60-security/62-authority.md:218-221` specifies the mechanism:

> A **revocable capability** is tied to a controlling `space` cell (`36 §4`)
> whose validity gates the capability; **revoking flips the cell**, after which
> the capability and every capability `⊑`-derived from it fails closed.

**There is no `36 §4` space in any runtime crate.**

And the trap is that a `Space` type *does* exist and greps positively:

| | `spec/30-surface/36 §4` — what revocation needs | `spec/40-runtime/44 §3` — what is landed |
|---|---|---|
| **is** | a unit of encapsulated mutable state and isolation: cells, `becomes`, `visits`, shared-nothing message passing | an **arena + index partition**, a reclamation unit |
| **landed as** | **nothing** | `crates/ken-runtime/src/store.rs:198` — `arena`, `index`, `capacity_limit`, `total_slots`, `total_interns`, `dedup_hits` |
| **has a validity cell** | yes, by definition — it is the membrane | **no** — it has no cells at all |

**Two different concepts share one word, and one of them is landed.** A search
for "space in the runtime" returns a confident hit that is the wrong space.
This is precisely the *same-literal-spelling* failure that has bitten this
project repeatedly, and it is why the charter's *"fold into PX7"* read as
plausible.

`spec/62 §4:234-240` is explicit that the runtime face is **`(oracle)`-tagged,
named not omitted**, deferred to `40-runtime`/`OQ-Space` — and that **ADR-0004
requires the mechanism to carry a stated, proven isolation property** resting
on the shared-nothing guarantee (`36 §4.4`).

**`OQ-Space` is DECIDED at the spec level** (operator, 2026-06-27:
shared-nothing message-passing, per-space Hoare, content-addressed immutable
transport) — **but its runtime realization (process / thread / green /
distributed) is explicitly deferred to `40-runtime`, and `40-runtime` has not
settled it.**

## 4. What this issue is actually asking

**Not** "build the membrane." The prior question, for the Architect:

1. **Is the `36 §4` space runtime realization a prerequisite, or is there a
   narrower membrane that gates OS-resource capabilities without it?** The spec
   ties revocation to a space cell; whether that is *the* mechanism or *a*
   mechanism is a design call, not a reading of the text. **State which clause
   of `62 §4` any narrower proposal discharges.**
2. **If it is a prerequisite,** this issue is blocked on a `40-runtime` space
   realization WP that does not exist, and that fact belongs in
   `10-linux-abi-completion.md` §5's graph — where **this item is currently
   absent entirely.**
3. **What are the in-flight semantics?** The charter names them
   (`09:497-498`: shared delegation identity, transitive child invalidation,
   close-on-revoke policy, defined in-flight semantics) without settling any.
   An operation that has already crossed the FFI boundary when `revoke()` lands
   is the hard case, and it is not addressed anywhere in the spec.
4. **Does ADR-0004's "stated, proven isolation property" requirement mean this
   needs its own ADR?** My reading is yes, and that it should precede sizing.

## 5. What is NOT in scope

- **IFC at the OS boundary is `Sec1`, an existing workstream** (`09:501-503`:
  *"Note the dependency; do NOT duplicate it here."*). Authority and
  information-flow are independent axes — holding permission to *write* a
  socket must not imply permission to *send secrets through it* — but that is
  Sec1's, not this issue's.
- **The static contract is delivered and is not to be re-litigated.** `62 §4`'s
  static face is done and correct. This issue is the deferred runtime face
  only.

## 6. Why it matters now

`docs/program/10-linux-abi-completion.md` **§8.1 gate 9 assumes L2 has it.**
The completion program is built on an assumption that no code satisfies, and
the item is not in that document's graph at all — the hole I found when the
operator asked for the coverage answer. **Whatever the Architect rules,
`10-…` §5 and §9 must be updated to reflect it**, so that the next coverage
question gets a true answer.
