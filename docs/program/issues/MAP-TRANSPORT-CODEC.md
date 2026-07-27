---
id: MAP-TRANSPORT-CODEC
title: "If Map/Set need a portable canonical serialization, it is ordinary package Ken — not a runtime primitive: settle whether a codec is required at all, and if so place it out of trusted_base()"
status: draft
owner: unassigned
size: TBD
gate: none
depends_on: [SPEC-STORE-SPLIT]
blocks: []
github: null
origin: Operator ruling 2026-07-26 (`RULING R2` on SPEC-STORE-SPLIT) — "Map/Set's internal bytes should not be observable. Only it's external behavior should be observable. If a codec is required for map/set to be transportable, then the codec should be in ken, as it would also be generally useful not just inter-thread, but inter-process or over the network." Steward-filed per COORDINATION §2.
---

> ## ✅ GATE SATISFIED — `SPEC-STORE-SPLIT` merged 2026-07-27 at `c631841d`.
>
> The gate was that `SPEC-STORE-SPLIT` **removes** Map/Set byte canonicity from
> the normative contract, so until it landed, `spec/40-runtime/41-values.md`
> still promised the very property a codec would be built to supply and any work
> here would have been designing a replacement for something the corpus said
> still existed. That is no longer the case: `41 §2/§3a/§5`, `30-taxonomy §6`, and
> every generic closure-free byte clause now dispatch by value domain, and Map/Set
> expose only extensional equality, ordered `to_list`, and durable round-trip.
>
> ⛔ **This node is still not released.** §2 below is the reason — the first
> deliverable is a decision about whether a codec is required at all, and that
> has not been taken. It is releasable, not ready.

## 1. The operator ruling, verbatim

> *"Map/Set's internal bytes should not be observable. Only it's external
> behavior should be observable. If a codec is required for map/set to be
> transportable, then the codec should be in ken, as it would also be generally
> useful not just inter-thread, but inter-process or over the network."*

Two clauses, and **the second is conditional.** The ruling settles **where** a
codec lives if one is wanted. ⛔ It does **not** assert one is needed.

## 2. ⭐ THE FIRST QUESTION IS WHETHER THIS WP EXISTS AT ALL

⛔ **Do not open this as "design the codec."** The premise *"if a codec is
required"* is itself unresolved, and it is the cheapest thing on the node to
settle. `SPEC-STORE-SPLIT` lands three observables — extensional equality,
ordered `to_list`, and **durable round-trip** — and the third one is the
interesting one: a Map/Set already round-trips through ordinary `data` tree
bytes. So the honest question is narrow:

> **What does a caller want that ordered `to_list` plus ordinary `data`
> encoding does not already give it?**

Candidate answers, each of which would justify the WP, and none of which is
established today:

- **Cross-space dedup that HITS.** `OQ-Space` passes immutable closure-free
  values with dedup by hash. Under `RULING R2` that dedup simply **misses** for
  extensionally-equal maps built in different orders. ⚠ That is not a defect —
  it is an optimization over non-observable bytes — but if a workload needs the
  hit rate, a canonical form is how you get it.
- **A stable name for a map.** Content-addressed identity for caching,
  memoization keys, or a durable index — anything that wants *"the same map has
  the same name"* across processes.
- **A wire format for a counterparty that is not Ken.** Two Ken spaces can agree
  on tree bytes; an external peer cannot be assumed to.

⇒ **The first deliverable is a ruling on whether any of these is a real
requirement.** If none is, this node closes as *not needed* and that is a
complete outcome, not a failure.

## 3. Placement — settled, and this is the whole of what the ruling fixes

**Ordinary package Ken, exactly like `spec/50-stdlib/52-map.md` itself.**

| | |
|---|---|
| ✅ where it goes | a `catalog/packages/` module, proved, pure |
| ⛔ where it does NOT go | `trusted_base()`, a `declare_primitive` opaque, `spec/40-runtime/`, `spec/30-surface/30-taxonomy.md §6` |

⭐ **This is the same shape as `OQ-A`, and that is the argument for it.** `OQ-A`
(operator, 2026-07-03) took a capability that looked like it wanted to be a
runtime primitive — an O(1) content-addressed canonical map — and chose
**proved + pure + zero-TCB** over the runtime heap form, explicitly accepting
O(log n) and loss of insertion-order canonicity as the price.

A runtime codec would invert that trade twice over: it would serve **only** the
in-process case, and it would **grow the TCB** to do it. The operator's own
reason is the wider one — a canonical serialization is *"generally useful not
just inter-thread, but inter-process or over the network"*, and none of those
three consumers is reachable from a runtime primitive.

## 4. ⛔⛔ THE TRAP — A CODEC MUST NOT RE-CREATE THE OBSERVABLE `R2` REMOVED

This is the one place this WP can go wrong in a way that quietly undoes
`SPEC-STORE-SPLIT`, so state it before any design starts.

**A codec's output is observable. Map/Set's internal bytes are not. These are
different propositions and the WP depends on keeping them apart:**

- ✅ `encode : Map k v → Bytes` is an **ordinary pure function**. Its result is
  observable the way any function's result is, and a conformance row may assert
  exact bytes for a given input — that is a property of **`encode`**.
- ⛔ It remains true that **Map/Set's own durable representation is not an
  observable**. No row may assert byte equality *or* byte inequality of a
  map's internal encoding across insertion histories.

⇒ The discriminator: a row that names `encode` and pins its output is fine. A row
that observes *a map* and concludes something about *bytes* is the thing `R2`
forbids, whether or not a codec exists. ⚠ **Do not let the existence of `encode`
be read as evidence that the map has canonical bytes** — it is evidence that a
*function* is total and deterministic.

## 5. Known coupling — the key interface

A canonical encoding needs a **key ordering**, and Ken's Map/Set key interface is
itself an open item (`C2`, Architect-ruled as a localized key-interface split).
⚠ **Do not treat this as a dependency edge until §2 returns "yes, a codec is
required"** — an unneeded codec has no key-ordering problem. If §2 returns yes,
re-derive the coupling against the landed `C2` state rather than against this
line, which is a current-state claim and perishable.

## 6. Do-not-reopen

- ⛔ **`OQ-A` is settled.** The O(1) content-addressed heap map is retired and
  parked as a possible later fast-map. This node is not a route back to it.
- ⛔ **`RULING R2` is settled.** Map/Set internal bytes are not observable. A
  codec does not change that (§4).
- ⛔ **Placement is settled.** Package Ken, out of `trusted_base()`. A proposal
  that needs a new trusted primitive is a **TCB delta** and goes to the operator,
  never into this WP's scope.
