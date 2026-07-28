---
id: RT-FNSPLIT-C1
title: "operational carrier + three executable eliminators — a runtime-general carrier at the Lowered/lowering boundary with a real producer -> validator -> eliminator edge, grounded on artifact-static semantic identity"
status: merged
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B2O, RT-FNSPLIT-B2R, RT-FNSPLIT-B2V, RT-VALUE-TOTALITY]
blocks: [RT-FNSPLIT-B2F]
github: https://github.com/swe-toolkit/ken/pull/1156
origin: Architect ruling evt_7ay6s5s79awz8 on the Steward's re-put of hard-stop #11 against the relaxed store contract (evt_70jp2sk4by7t8), Decision dec_45aa2gngjc79z resolved — verified from the object. Required by SPEC-STORE-SPLIT §7 item 2. Replaces the retired RT-FNSPLIT-B2E, whose binding contract (a closed disposition LEDGER, inert) and name authority (store-local interning) were both superseded by that ruling. Steward-filed; Steward owns the frame and AC/control placement.
---

> ## ✅ MERGED 2026-07-28 — PR #1156, blob-verified. ⛔ READ WHAT DID *NOT* CLOSE.
>
> **`origin/main = feab3cb56efc9fe7b31c7f8891fe1dc9a10ac98f`**, landed tree
> `c3d54139`, candidate `2029f915adf5bbf880d13ebdb968c8d2f5400fdd` (tree
> `d9549e15`). **All nine `crates/ken-runtime/` paths verified blob-identical.**
> ⛔ Verified by blob, ⛔ **not** by ancestry — the publisher squashes, so
> `merge-base --is-ancestor` reads false for merged work.
>
> Approved on Decision `dec_35z7g494s9ba4`, resolved by the Architect for this
> SHA only; fresh exact-SHA QA approved the same object.
>
> ### ⭐ THE `B2F` RELEASE GATE IS NOW SATISFIED
>
> `B2F` gates on **the closed `C1` carrier artifact**, not on `C1` merely being
> merged. That artifact is closed: a real producer → validator → eliminator edge
> is executing on `main`, with the residuals recorded in the frame's `§5`.
>
> ### ⛔ WHAT IS STILL OPEN — do not read "`AC-C4` merged" as "`AC-C4` done"
>
> **`AC-C4` SPLIT, and only the REPRESENTATION half closed here.** A carried
> recursive position **builds** its induction hypothesis and the case
> eliminates. ⛔ **The runtime INVOCATION of that hypothesis is `B2F`'s** — it
> needs one closed, recursively callable Cranelift target per static
> computational-eliminator origin, which is target-function population, forbidden
> here by `AC-C10`. Control 1 is **PARTIAL by design**; see frame `§5` residual 1.
>
> ### ⚠ THE LIVE RESIDUAL A READER MUST NOT LOSE
>
> ⛔ **The carried case-binder projection is pinned at ARITY 2 ONLY.** Any
> projection defect **conditional on arity ≥ 3** is unpinned — `min(position, 1)`,
> an off-by-one that only trips past index 1, and any arity-conditional branch are
> all unseen. ⚠ **Field permutation is one instance, not the boundary**, and a
> reader checking against the word *"permutation"* would wrongly rule the others
> covered. Recorded as frame `§5` residual 5 (PR #1155), an Architect publication
> condition on the Decision.
>
> ⚠ Control 7's `THE GAP` comment in `constructors.rs` is **narrower** than the
> frame and the two do not agree; the comment is non-authoritative test narrative
> and **`§5` governs**.
>
> ### ⭐ The mechanism worth carrying forward
>
> The blocking defect was a **compile-preserving evasion**
> (`children[position]` → `children[0]`) that left a control green because every
> existing control observed the residual's **metadata** edge and none read its
> **content**. What closed it was not a stronger assertion but an expected value
> sourced from **the fixture's own declaration** rather than from the production
> variable the mutation perturbs. ⭐ **A control whose expectation moves with
> production cannot detect production moving** — `sibling_position: 1` surviving
> M7 is the proof.
>
> ⛔ The node is **not** the whole of `RT-NATIVE-FNSPLIT`. `B2F` remains, and it
> is what `PX8-ERRID-ALLOC` is actually waiting on.

> ## ▶ THE NODE THE `#11` RULING PRODUCED
>
> **Sequence:** `B2O` → `B2R` → `B2V` → **`C1`** → `B2F`.
>
> Frame: [`RT-FNSPLIT-C1-operational-carrier.md`][f], under `docs/program/wp/`.
> The frame is the executable artifact; this node carries the ruling and the
> program bookkeeping.

## ⛔ WHY THIS NODE REPLACES `RT-FNSPLIT-B2E` RATHER THAN AMENDING IT

`B2E` was authored under two premises the `#11` re-put ruling removed. **Both
were structural, not wording**, which is why the node is retired rather than
edited:

| `B2E`'s premise | what replaced it |
|---|---|
| **"a closed LEDGER, not three eliminators"** — the node's own binding-contract heading. It was to land an opaque inhabitant plus a classification table, **inert**, with elimination deferred to `B2F`. | ⛔ Killed by the **inertness rule**: *"a representation-only artifact with the semantic consumers deferred does not discharge `#11`."* The three eliminations are now **the node**, and the ledger is one deliverable (`D5`) inside it. |
| **name authority = "one artifact-static name reference resolved through the producer's store-local interning authority"** (`B2E` ruling `R1`) | ⛔ Killed by lever requirement 3: identity comes from **artifact/module semantic authority shared by producer and consumer**, ⛔ **not** persistent-store identity. |

⭐ **`R1`'s measurement survives; only its conclusion died.** `R1` was right that
there is no artifact-static `u64` name ID sitting ready to be used. It was wrong
to reach for the store to supply one — `SemanticPlane`'s `CaseConstructor` /
`ProjectField` / `ConstructorSymbol` / `RecordFieldName` atoms are the authority,
and they are not store-derived. The frame's §2c measures them.

## ⛔ AND WHY `RT-FNSPLIT-B2F` IS **AMENDED, NOT RETIRED**

`SPEC-STORE-SPLIT` §7 item 1 directed that **both** `B2E` and `B2F` be retired
and rewritten, on the ground that they are *"built around the constraint being
removed."*

⭐ **That direction is followed for `B2E` and departed from for `B2F`, and the
reason is the ruling itself.** §7's instruction rests on `SPEC-STORE-SPLIT` §1's
causal claim — that the store/sharing conflation is *"why every eliminator
needed a compile-time template."* **The re-put put that claim under test and the
Architect ruled it over-broad:** the conflation *enlarged* the old prerequisite;
it did **not cause** the template requirement.

⇒ `B2E`'s contract genuinely descended from the removed substrate, so it dies.
`B2F`'s purpose — per-static-origin Cranelift target functions, atomic
switch-over, equivalence evidence, old-path removal — **never depended on the
store contract at all**, and the Architect's earlier `#11` ruling that *"`B2F`
resumes unchanged in purpose and atomicity"* is undisturbed. Retiring it would
discard a correct frame to satisfy a premise that did not survive.

⚠ **What `B2F` does need is a correction, not a rewrite**, and it is applied in
that node: its dependency moves `B2E` → `C1`, and its `R1`-derived residual
(*"loads the resolved store-local ID from the binding/table"*) is **false** under
the new identity authority.

⛔ **Do not read this as licence to skip §7's other items.** Items 3, 4 and 5 of
`SPEC-STORE-SPLIT` §7 are unaffected by this reasoning and remain owed.

## The ruling, in the form that binds this node

Full transcription is at the head of
[`RT-NATIVE-FNSPLIT.md`](RT-NATIVE-FNSPLIT.md). The four lever requirements,
which are the node's deliverables `D2`–`D5`:

1. `Match` and `ComputationalMatch` **discriminate runtime constructor identity
   against the artifact-static case set**, then project children back into **the
   same operational carrier**.
2. `Project` selects a runtime record field using **artifact-static field
   identity** and returns **that same carrier**.
3. Constructor and field identity come from **artifact/module semantic authority
   shared by producer and consumer** — ⛔ **not** persistent-store identity.
4. Every reachable consumer outcome is **structurally closed**; unsupported forms
   **fail closed at the typed boundary**.

Plus the rule that makes this node different from the three before it:

> *"A prerequisite may be inert **only** in the sense that production function
> routing has not switched to it yet. Its **producer → validator → eliminator
> edge must nevertheless be real and executable.**"*

## ⛔ Sequencing — this node is THIRD in the Runtime queue

`ABI-S3` (active) → `RT-VALUE-TOTALITY` P2 (framed, queued) → **`C1`**.

**Contention is measured and real:** P2 and `C1` both edit
`crates/ken-runtime/src/boundary_value.rs` at **different lines**, which is the
shape git merges into a silent union. The frame's §7 carries the site table.
⇒ ⛔ **Do not run them concurrently**, and **re-derive the frame's §2 substrate
after P2 lands** — the class/tag rows `D5` closes over are the ones P2 edits.

`ABI-S3` is contention-free with this node (disjoint crates).

## What this unblocks

`B2F` → then, in order, `RT-FNSPLIT-B2B`, `RT-FNSPLIT-B2O-CHECK`, `RT-SCALE-B`,
the parent `RT-NATIVE-FNSPLIT`, `NATIVE-HANDLE-CARRIER`, `PX8-F-CAP-41`
(Foundation), `PX8`, `PX9`, and the ABI/PX chain behind them.

⭐ **That chain is the reason this node is the highest-leverage frame in the
tracker:** as of `a31bb7b6` every releasable node in the DAG is Runtime-owned or
enclave-owned, and Foundation's only path to tracked work runs through here.

> ### ⭐ SYMPTOM INVENTORY + RESEARCH COUNT — armed 2026-07-27 at release
>
> **This is a FRESH chain.** `RT-NATIVE-FNSPLIT`'s pre-recut chain is **frozen
> at 33 hard-stops** and its post-recut chain carries its own counts on that
> node. ⛔ Do **not** resume either count here, and do not read an anchor for
> this node out of the parent's prose.
>
> **Why both lines exist:** the parent ran to **33** hard-stops with the
> research-escalation trigger and the symptom inventory *both* silently
> unarmed — the Architect's self-trigger lapsed across its compactions and the
> Steward's backstop lapsed because the count lived only as prose, never as a
> line anyone re-read. ⭐ **An unarmed count is not a trigger.** Re-read the two
> lines below on **every** hard-stop.
>
> The Architect appends one entry per hard-stop **before it rules**, and at
> each predicate check answers whether the entries **share a predicate**
> (architect §1b, steward §5a-ii). A shared predicate is not another ruling —
> it says the *representation* is the defect, and it comes to the Steward as a
> recut. ⛔ *"The architecture is still viable"* is **not** an answer to the
> predicate question, and *"every ruling so far was correct"* is the symptom
> rather than the refutation.
>
> ```text
> SYMPTOM INVENTORY (append only; never rewritten)
> NEXT PREDICATE CHECK = 3rd entry, then 6th, 9th, …
> NEXT RESEARCH PULL   = 3rd hard-stop, then 6th, 9th, …
> ENTRIES = 0
> (empty)
> ```
>
> ⚠ A deep chain carrying **zero** research advisories is itself the tell that
> both the self-trigger and the backstop have lapsed. If that is ever true
> here, apply the catch-up rule — fire at the **very next** hard-stop and
> re-anchor the cadence from there; do not wait for the next clean multiple.

[f]: ../wp/RT-FNSPLIT-C1-operational-carrier.md
