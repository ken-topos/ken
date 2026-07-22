---
id: ABI-REVOKE
title: "runtime revocation membrane — the deferred runtime face of 62 §4"
status: draft
owner: runtime
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

> ### ⛔ STILL NOT SHOVEL-READY — DO NOT RELEASE TO A BUILD TEAM
>
> **The Architect has ruled** (`dec_p1dv4gw6bsc2`, §4), so the *design* question
> is settled and `owner: runtime` is now correct. **`size: TBD` is not a
> placeholder I forgot to fill:** the ruling requires **an ADR, and a
> Spec-owned behavioral slice, both BEFORE sizing.** Until those exist there is
> no mechanism to brief. **Do not let a leader pull this.**
>
> **Two prerequisites, in order:** (1) route the narrow observable contract to
> the **Spec enclave** — revoke/attenuate operation shape, distinct `revoked`
> identity, settlement observation; (2) the **ADR**, whose isolation argument
> must be structural and closed-world (§4). Then size it.
>
> ### ⛔ ABI-R3 GATES THIS, and the dependency is bound HERE, not in `depends_on:`
>
> **`PX8 -> ABI-R3 -> ABI-REVOKE`** (Architect, `dec_p1dv4gw6bsc2`).
> **ABI-R3 must land first**, and the reason is load-bearing rather than
> sequencing hygiene: the membrane **adds and guards operation identities**,
> and ABI-R3's generated inventory is what makes a new operation a **build
> break**. Landing the membrane first would put un-inventoried operations in
> the dispatcher — the precise failure ABI-R3 exists to prevent.
>
> **ABI-REVOKE then gates `ABI-A1`-`ABI-A3` and `PX9`**, so PX9 absorbs revoked
> identity *before* the synchronous/Track-T expansion rather than retrofitting
> it. `ABI-M1`/`ABI-M2` stay on their existing inventory/probe path unless the
> revised manifest explicitly carries revocation evidence.
>
> **Why `depends_on:` is empty.** `ABI-R3` is an unframed item in
> `10-linux-abi-completion.md`, **not a tracked issue**, so there is no id to
> reference — the schema gate rejects one. **Minting a stub purely to satisfy a
> schema field would be inventing scope.** The gate has now caught me doing
> exactly that three times, and it has been right each time. When `ABI-R3` is
> framed it takes `blocks: [ABI-REVOKE]`, this issue takes
> `depends_on: [ABI-R3]`, and this note comes out.

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

> ### ⛔ CORRECTED — my mechanism argument was WRONG
>
> Adversary `evt_3fqyadvebxja0`, 2026-07-22.
>
> I originally wrote: *"a generation check cannot express revocation — after
> `revoke()` the OS resource is still open and the generation has not
> changed."* **That is false about the mechanism.** There are **two**
> independent generation-checked tables in `ken-host/src/effect_v1.rs`:
>
> | table | keyed on | tracks |
> |---|---|---|
> | `CapabilityTableV1` (`:484-506`) | `CapabilityTokenV1 { slot, generation }` | **authority** |
> | the resource table (`:700+`) | `ResourceTokenV1 { slot, generation }` | **OS resource liveness** |
>
> `CapabilityTableV1::resolve` (`:496-505`) denies on
> `slot.generation == token.generation`. **That generation has nothing to do
> with whether the fd is open.** Bumping it denies every outstanding token for
> that slot **while the resource stays open and its slot is untouched** —
> exactly the cell where I claimed a generation check cannot go. **A
> capability-slot generation IS a validity cell; it is simply never flipped.**
>
> **My sentence was a true statement about the CURRENT CODE presented as a
> property of the MECHANISM.** Those are different claims and the second is
> false. Watch for that conversion — it is how a scoping document acquires a
> false blocker.

**The honest gap, which is sharper and more actionable:**

1. **`CapabilityTableV1` is append-only.** Its whole surface is `insert`
   (`:489`) and `resolve` (`:496`) — **no bump, no remove, no revoke.** The
   substrate exists; the operation does not. **That is one method away.**
2. **★ Derivation is not represented, so transitivity has NO SUBSTRATE.**
   `62 §4` requires revoking `c` to fail-close *"every capability `⊑`-derived
   from it, to any depth."* At the host boundary
   `CapabilityTraceIdentity` is **`pub struct CapabilityTraceIdentity(pub
   String)`** (`:1854`) — a bare string, **no parent pointer, no derivation
   edge** — and **`attenuate` does not exist in `ken-host` at all** (the only
   hit is a test *name*, `:3174`). **You cannot walk a delegation tree that was
   never recorded.**

**PX7 remains a different property** — its generation means close/stale/reuse,
never withdrawal — but the reason is the missing derivation edge, not an
inability of generation checks in general.

## 3. ⛔ CORRECTED — one abstraction, one landed projection

> **My original §3 claimed the runtime's `Space` and the spec's `36 §4` space
> were UNRELATED CONCEPTS SHARING A WORD. That is false, and the Architect
> corrected it (`dec_p1dv4gw6bsc2`).** The correction is recorded rather than
> quietly overwritten, because the way I got it wrong is instructive.

`spec/40-runtime/44-capacity.md:293-294` says, **verbatim**:

> **Each surface `space` (`36 §4`) realizes as a store `Space`**; when the
> `space` terminates — or `reset`s a bounded unit of work — its arena is
> reclaimed.

**They are the same intended abstraction.** `crates/ken-runtime/src/store.rs:198`
is the **memory/reclamation projection** of the surface space's runtime
realization — not a different thing that happens to share a name.

| | status |
|---|---|
| **intended abstraction** | one — the `36 §4` surface space, realized at runtime |
| **landed projection** | **memory/reclamation only**: `arena`, `index`, `capacity_limit`, `total_slots`, `total_interns`, `dedup_hits` |
| **missing** | the rest of the runtime aggregate — **state/effect execution, authority/revocation, isolation/transport**, and wiring each surface space to its own aggregate |

**What revocation needs is the authority projection, which is absent.** That
much of the original finding stands: `store::Space` has **no cells and no
validity to flip**. But "the authority projection of a partly-realized
abstraction is missing" is a materially different — and much cheaper — claim
than "the prerequisite concept does not exist."

> ### ★ How I got it wrong, and why the confirmation didn't help
>
> **I concluded "not implemented" by inspecting one struct's field list.** A
> field list tells you what a type *currently holds*; it cannot tell you what
> it is *intended to become*, and the spec said so in a sentence ~100 lines
> from the definition I was reading. **Before concluding a spec concept is
> unrealized, grep the spec for a realization sentence** — `realizes as`, `is
> realized at runtime as`, `maps to`, `projection of`. One grep would have
> settled it.
>
> **And the adversary independently confirmed my framing** — *"the doc comment
> convicts itself: arena + index partition, no cells, nothing to flip"* — which
> is true and does not bear on the question. **We both examined the landed
> struct; neither of us searched for whether the spec linked the two names.**
> Agreement across reviewers is **not independence** when both inherited the
> framing from the same document. The Architect caught it only because it was
> asked a different question — *"what should be built?"* — which forced reading
> `44 §3` forward instead of checking my table.

### ⚠ And `62 §4` is in tension with itself — worth knowing before reading §4

My original framing rested on the `:218-221` bullet, which states the
space-cell mechanism in the indicative. **But the deferral paragraph twelve
lines later enumerates four candidate realizations:**

> the mechanism that realizes fail-closed at evaluation — **forwarder /
> membrane / validity-indexed / region lifetime in the controlling space** — is
> a downstream runtime WP.

**Only the fourth is space-resident**, and *"validity-indexed"* fairly
describes a generation-checked slot table. So the chapter states one
realization as *the* mechanism and then opens three others. **That is the
chapter disagreeing with itself, not a design fork** — and it is why §3's
original "blocking prerequisite" framing was over-strong.

## 4. ✅ ARCHITECT RULING — `dec_p1dv4gw6bsc2`, resolved 2026-07-22

**Full multi-space runtime realization is NOT a prerequisite.** Build a
**bounded authority projection for the current implicit root execution space**,
owned by `ProcessContext`.

> ⛔ **The closure claim is deliberately bounded.** This closes the current
> **OS-operation runtime face** of `62 §4`. **It must NOT be reported as
> closing general `36 §4` runtime realization or cross-space revocation.** It
> supplies no separate runtime spaces, no cross-space forwarders, no message
> transport, and no distributed isolation proof.

**Required component shape:**

- `ProcessContext` owns **one** host-trusted `RevocationDomain` — explicitly
  the authority projection of Ken's current implicit root space.
- The domain allocates **opaque, monotonic, non-Ken-visible**
  `RevocationNodeId` values. **No raw pointer or reference to the validity cell
  crosses the host boundary**, and IDs are never reused.
- **Copying** a capability preserves node identity. **Attenuation** creates a
  child node with a **parent link**. **Revoking** closes that node and every
  descendant — never its parent or siblings.
- **Admission checks the addressed node AND every ancestor.** A cached
  leaf-live bit is insufficient unless its invalidation proof is part of the
  design.
- Every capability grant slot retains its node ID, and **every resource
  acquired under that authority retains the same provenance in its resource
  slot** — so a later `FsReadAt`/`FsWriteAt`/metadata op **cannot bypass
  revocation merely by consuming only a resource token.**
- **Keep `CapabilityTableV1` and `ResourceTableV1` separate** (ADR-0021): grant
  authority and object lifetime are different axes. They consult the same
  revocation domain; **generation continues to mean close/stale/reuse, never
  withdrawal.**
- A duplicated resource inherits the same node unless a future explicit
  reauthorization establishes a different sponsor. **Do not invent
  multi-sponsor "any live grant wins" semantics in this WP.**
- The denial must have a **distinct `revoked` semantic identity** — it must not
  collapse into malformed, closed, stale-generation, or right-not-held. **Spec
  owns the exact public error placement and observation shape.**

**In-flight semantics — admission is the linearization point**, with a lease
acquired atomically with the live ancestry check:

- **revoke before admission** → distinct revoked denial, **no backend call**;
- **admission before revoke** → the operation may finish and report its real
  result. **Revocation promises neither rollback nor cancellation, and a side
  effect may already commit.** This follows the spec's *"subsequent perform"*
  language and states the boundary honestly.
- revocation closes **new** admission immediately and logically invalidates
  resources in the subtree;
- **owned OS resources close only after all already-admitted leases drain.**
  Never close or reuse an fd while an admitted operation may borrow it;
- close/settlement failure is recorded once under ADR-0021 settlement
  discipline; **it does not reopen authority**;
- today's unique synchronous `ProcessContext` drains immediately in ordinary
  dispatch. **PX12/future concurrency must preserve this same admission/lease
  contract**, and **cancellation is a separate operation that cannot be
  inferred from revocation.**

**A dedicated ADR IS required, before sizing.** ADR-0004 states the property
but chooses none of: ownership, lineage, admission linearization, resource
provenance, close-after-drain, error identity, or the bounded root-space claim.
The ADR's isolation argument must be **structural and closed-world**: (1) one
mutable owner in the unique `ProcessContext`; (2) tokens carry opaque IDs, not
shared mutable host references; (3) only the domain mutates validity/lineage;
(4) the dispatcher's generated operation inventory closes the admission
boundary; (5) backend borrowing is reachable only through a live admission
lease. **Call this runtime-trusted component isolation, not a kernel theorem.**
A future concurrent realization must **replace** unique borrowing with a proved
atomic/locked linearization — **it cannot inherit today's proof by assertion.**

**Program order and ownership:**

- **Primary implementation owner: Runtime.** Keep `size: TBD` until the ADR and
  the behavioral contract land.
- **First, route the narrow observable contract to the Spec enclave** —
  revoke/attenuate operation shape, distinct revoked identity, settlement
  observation. **This is behavioral, not an implementer choice.**
- **Correct the Linux graph to `PX8 -> ABI-R3 -> ABI-REVOKE`.** ABI-R3 lands
  first because the membrane adds and guards operation identities and must be
  caught by the generated inventory.
- **ABI-REVOKE then gates `ABI-A1`–`ABI-A3` and `PX9`**, so PX9 absorbs revoked
  identity *before* the synchronous/Track-T expansion rather than retrofitting
  it. `ABI-M1/M2` stay on their existing inventory/probe path unless the
  revised manifest explicitly carries revocation evidence.
- Full surface-space runtime realization remains a separate future Runtime
  dependency for the **general** claim — **not a blocker on this bounded item.**

**⇒ The issue and the program graph are now corrected, but this item remains
NOT SHOVEL-READY until the ADR plus the Spec-owned behavioral slice exist.**

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
