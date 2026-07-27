# MAP-TRANSPORT-CODEC R1 — settle whether a codec is required at all

**Node:** [`MAP-TRANSPORT-CODEC`](../issues/MAP-TRANSPORT-CODEC.md) · **Owner:**
Ergo · **Size:** S–M · **Gate:** none (`SPEC-STORE-SPLIT` merged `c631841d`)

**Fixed inputs, measured at `origin/main = 957250ef`. ⛔ Re-derive at point of
use — these are current-state claims and they perish.**

| input | pin |
|---|---|
| the ruling | `MAP-TRANSPORT-CODEC.md §1` — operator 2026-07-26. **Two clauses, and the second is conditional.** It settles *where* a codec lives if one is wanted; it does **not** assert one is needed |
| the question | `§2` — *"What does a caller want that ordered `to_list` plus ordinary `data` encoding does not already give it?"* |
| the three candidates | `§2` — cross-space dedup that hits · a stable name for a map · a wire format for a non-Ken peer |
| ⛔ the trap | `§4` — a codec's **output** is observable; a map's **internal bytes** are not. Different propositions |
| settled, do not reopen | `§6` — `OQ-A`, `RULING R2`, and placement (package Ken, out of `trusted_base()`) |

## 1. What this WP is

**A written, evidence-backed determination of whether any of `§2`'s three
candidates is a real requirement today.** That is the whole deliverable.

⛔ **This is NOT "design the codec" and NOT "build the codec."** If the answer is
*no*, the node closes as **not needed** — ⭐ **that is a complete outcome, not a
failure, and it is the result I consider most likely.** Do not treat a negative
finding as an under-delivery and go looking for something to build.

## 2. Deliverable

For **each** of the three candidates, a section stating:

1. **The method** — what you searched, in which trees, with what spelling.
2. **What you found** — the actual consumers, cited by path and line, or none.
3. **The verdict** — is this a requirement *today*, a plausible *future* want, or
   not a thing anyone needs.

Then one **recommendation** paragraph: does a codec have a requirement, and if so
which candidate carries it.

⚠ **Candidate 3 (a non-Ken wire peer) is a roadmap question, not a corpus
measurement.** Say so plainly and mark it *operator input needed* rather than
answering it from the tree. Candidates 1 and 2 are measurable.

## 3. Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-R1a` | ⭐⭐ **The search can actually find things.** | ⛔ **A "no consumers found" result passes for any reason — including a broken search.** Before reporting any negative, run your exact method against a **known-present** consumer and show it is located. Suggested positive control: the cross-space dedup path itself (`OQ-Space` value passing) — your method must find *that* before its silence about anything else means something. **Report the positive control's output, not just that you ran one** |
| `AC-R1b` | Candidate 1 (**dedup that hits**) is measured. | Name the dedup mechanism and where it lives. ⚠ `§2` states dedup **misses** for extensionally-equal maps built in different orders and that this *"is not a defect — it is an optimization over non-observable bytes."* The question is **whether any workload depends on the hit rate**, not whether misses occur. Do not report the miss as if it were the finding |
| `AC-R1c` | Candidate 2 (**a stable name for a map**) is measured. | Search for consumers wanting content-addressed map identity: caching, memoization keys, durable indexes. ⭐ Search for the *want*, not the word — a consumer that hand-rolls `to_list`-then-hash is exactly this requirement showing up unnamed, and it will not match a grep for "canonical" |
| `AC-R1d` | Candidate 3 is **marked as operator input**, not answered. | ⛔ Do not infer a roadmap from the corpus |
| `AC-R1e` | ⭐ **The `§4` distinction survives the writeup.** | Your document must not contain a sentence that treats a map as having canonical bytes. If you recommend a codec, say *"`encode` is a total deterministic function"* — ⛔ never *"a map has a canonical encoding"* |
| `AC-R1f` | **Zero code, zero spec, zero conformance changes.** | This WP's whole output is one document under `docs/program/`. If you find yourself editing `crates/`, `spec/`, or `conformance/`, the premise has failed — stop and report |

## 4. Scope

**IN:** reading the tree, the measurement, and one document.

⛔ **OUT:**
- ⛔ **Designing or building a codec.** Even if the answer is yes — that is a
  successor WP with its own frame.
- ⛔ **The `C2` key-interface coupling.** `§5` is explicit: not a dependency edge
  until this WP returns *yes*. ⛔ Do not re-derive it speculatively.
- ⛔ **Anything in `§6`'s do-not-reopen list.**
- ⛔ **A new trusted primitive.** That is a TCB delta and goes to the operator,
  never into this WP.

## 5. Contention check

**Measured at `957250ef`.** This WP writes one new file under `docs/program/` and
touches no crate, spec, or conformance path. **Empty intersection with every open
WP branch by construction.**

## 6. Validation

**No build.** ⛔ Do not take the machine-wide `ken-cargo` slot — Runtime, Kernel
and Language are all live on it and this WP has nothing to compile.

## 7. Reporting

Return the document's path and: **the positive control's actual output**
(`AC-R1a`); the per-candidate verdicts; the recommendation; and — if your answer
is *no codec is required* — say so directly and without hedging. ⭐ A clean
negative, well-evidenced, is the most useful thing this WP can produce.
