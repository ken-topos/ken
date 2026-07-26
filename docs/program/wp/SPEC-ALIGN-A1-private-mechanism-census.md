# `SPEC-ALIGN-A1` — private-mechanism census, and the authority-convention repair

> **Node:** [`docs/program/issues/SPEC-ALIGN-A1.md`](../issues/SPEC-ALIGN-A1.md) ·
> **Campaign:** [`14-spec-mission-alignment-campaign.md`](../14-spec-mission-alignment-campaign.md) ·
> **Advisory (captured, verbatim):**
> [`spec-mission-overspecification-advisory.md`](../spec-mission-overspecification-advisory.md)
>
> **Owner:** spec enclave (`spec-leader` + `spec-author` + `conformance-validator`)
> · **Size:** M · **Base:** the `origin/main` in the kickoff, bound exactly.

## 0. Read this first — what this WP is NOT

⛔ **It is not "delete the over-specified constants."** The operator's dispatch
called category 1 *"spec edits, low risk, mostly mechanical."* That framing was
built on my summary table, and **my table was wrong about the risk** — see §2.
Every nominated constant has a conformance consumer. The dispatch stands; its
*shape* changed.

⛔ **It is not a relaxation quota.** The deliverable is a **census plus an
honest stop list**. If the census finds that every candidate has a live
consumer and the edit set is empty, **A1 is complete and successful** — the
authority repair alone justifies it. ⚠ The named failure mode here is a ring
that manufactures relaxations because a census feels like a non-deliverable.

⛔ **It is not a licence to move a conformance row.** A row that asserts a
private mechanism is *evidence for a stop*, not a thing to retarget. Retargeting
is a conformance-granularity decision and it belongs to the Architect.

## 1. The obstruction — stated as a property, not a to-do

The mission is outcome-oriented: human-readable contracts, independently
rechecked certificates through a small kernel, explicit
tested/delegated/unknown boundaries, totality by default, explicit effects and
authority, honest separation of proof from testing from monitoring. It does not
by itself require particular hashes, heap layouts, slot identities, probing
policies, or page sizes.

But **absence from the mission is not proof of over-specification.** The
discriminator is:

> Could two implementations provide identical source meaning, proof results,
> trust boundaries, security guarantees, durable artifacts, and observable
> behavior, yet one fail conformance solely because it uses different internal
> machinery?

A *yes* is a **signal**. Research's four-class test turns a signal into a
verdict, and **only class 4 (private mechanism) is in this WP's scope.** Classes
1/2/3 need a semantic, compatibility, or threat argument respectively, and each
has a track that is not this one.

## 2. ⭐⭐ THE MEASUREMENT — every candidate has a conformance consumer

Measured on `origin/main=9410d7b8`, `spec/` tree `7fce4373`:

| class-4 candidate | conformance rows asserting it |
|---|---|
| FNV-1a + `memcmp` + monotonic `u64` | `conformance/runtime/capacity/seed-capacity.md:156` |
| **0.70** load factor + resize behavior | `seed-capacity.md:167`, `:170` |
| same-slot dedup, `==` is O(1) | `conformance/runtime/seed-runtime.md:11`; `runtime/values/README.md:23`; `runtime/evaluation/seed-evaluation.md:159`, `:169`, `:193` |
| same-slot as the *structural-sharing* observable | `surface/collections/seed-collections.md:214`, `:256`, `:593`, `:605` |
| **4 MiB** arena page size | `seed-capacity.md:179` |
| bignum tag `0x01`, inline-`i64` fast path | `surface/numbers/seed-numbers.md:49`, `:68`; `runtime/values/README.md:76` |
| minimal-limb sign-magnitude encoding | `runtime/values/README.md:154`–`:161`; `conformance/README.md:124` |
| canonical two-space indentation, byte-identity | `surface/elaboration/seed-multi-binding-let.md:363`, `:393`, `:585` |
| formatter line width (96 columns, `31-lexical.md:123`) | `surface/formatting/seed-canonical-format.md:10` — **`RED-UNTIL-BUILT`** |

⚠ **THIS TABLE IS A SEARCH BUDGET, NOT A POPULATION.** It is a Steward keyword
grep over `conformance/` for the mechanisms the advisory happened to name. **An
inventory is bounded by an unwritten notion of its surface**, so a constraint
asserted in a spelling I did not grep is absent here **and still live**.
⇒ You derive your own census (§3), state the reading you used, and **if yours
disagrees with mine, yours wins and you say so.**

## 3. Deliverable 1 — the census, and how to derive it

For **every** constraint you propose to relax, produce a row:

| constraint | spec site | class | conformance rows | `crates/` dependence | external consumer | verdict |
|---|---|---|---|---|---|---|

**The derivation must be by what a row ASSERTS, not by keyword co-occurrence.**
⛔ A keyword grep over `conformance/` is a starting budget and it produces false
hits in both directions. Two measured examples, given as your controls:

### ⭐ Control A (positive — a keyword hit that is a FALSE consumer)

`grep -rn probing conformance/` returns
`conformance/stdlib/collections/seed-cat4-maps-sets-relations.md:327`. That row
is about **probing a map key after a delete** — the stdlib `Map` API, nothing to
do with the content store's collision policy. ⇒ The store's probing/collision
policy has **zero** conformance consumers, and a phrase-count census would
record one. **Your census must classify by what the row asserts.**

### ⭐ Control B (negative — a constraint that IS consumed, answer known in advance)

The **4 MiB page size** is asserted at `seed-capacity.md:179` (*"intern enough
values to overflow one **4 MiB** page"*). ⇒ **A census that returns an empty
consumer set for the page size is under-derived, and you have measured your own
method rather than the suite.** Run Control B first: a census that cannot find
the answer you already know cannot be trusted on the answers you don't.

⚠ **Non-vacuity requirement.** Your census must return **at least one** empty
consumer set **and at least one** non-empty. If every candidate comes back the
same way, the census is not discriminating — report that as a method finding
rather than as a result.

## 4. Deliverable 2 — the authority-convention repair (do this one first)

Two sites, one shape:

| site | text |
|---|---|
| `spec/40-runtime/44-capacity.md:20` | *"Where the F4 design and the landed code diverge, the **landed code is normative** and the divergence is flagged inline."* |
| `conformance/runtime/capacity/seed-capacity.md:44` | *"conformance follows the landed code."* |

⛔ **Repair the SCOPE, not a principle.** The `44` sentence lives in an X2
grounding block explicitly labelled *"perishable-frame, K2c-s2 rule"*: it
arbitrates **F4 design prose vs the landed K3 store**, two internal drafts. It
is not a global claim that implementation outranks specification. ⇒ The repair
names the two drafts it arbitrates and its expiry, so that an independent
implementer reading the chapter's status block cannot take it as **the**
authority rule. A repair written as though the spec had asserted the global
principle will overshoot and change a claim the spec never made.

⛔ **Preserve the divergence record.** That block documents three real
divergences worth keeping: per-`space` bare-hash index rather than a
process-wide `(root, hash)` index; reclamation drops page buffers rather than
`madvise`; single-writer resize rather than lock-free. The defect is the
authority claim wrapped around them.

⚠ **Sweep for siblings.** The same convention may appear in other chapters'
status blocks under other spellings (*"grounded on the landed", "conformance
follows", "impl is normative"*). Report what you searched for, so the next
reader can tell your surface from mine.

## 5. Deliverable 3 — relax only the empty-consumer constraints

For each constraint the census clears, the edit lands with the **five-item
record** (campaign §9), in the commit and in the terminal handoff:

1. the mission outcome that remains protected;
2. the observable or security invariant retained;
3. the implementation choices newly permitted;
4. any external consumer requiring exact compatibility;
5. ⭐ **a conformance pair showing the relaxed contract still rejects an actual
   mission-breaking implementation.**

⭐ **Item 5 is what makes this a relaxation rather than a subtraction.** A
relaxed constraint that can no longer reject a bad implementation did not free a
mechanism — it deleted a guarantee. If you cannot produce the pair, the
constraint is **not** cleared, whatever the census said.

## 6. Deliverable 4 — the stop list

Every candidate with a live consumer, named with: its spec site, the rows that
assert it, and **which class (1/2/3) it turned out to be**. A stop is a
first-class deliverable here, not a failure to relax.

**Expected stops, so you can tell a surprise from a confirmation:** the whole
content-store family (FNV-1a, load factor, same-slot, page size), because it is
also entangled with fork **C7** (logical `space` vs physical realization) and
with live `crates/ken-runtime` work (`RT-FNSPLIT-B2E`). ⚠ If one of those comes
back *clear*, that is the interesting result of the WP and it needs its own
paragraph, not a table cell.

⭐ **The numeric family splits and the split is the deliverable, not a stop.**
Ranges, rounding, overflow, normalization, and equality are **source
semantics — not relaxable.** Tags, limb width, coefficient layout, and fast
paths are class 4. ⚠ But *minimality* of the limb encoding is what buys **unique
encoding**, an observable durable contract — so "minimal-limb" and "64-bit limb"
are not the same kind of thing, and if bignum bytes cross a durable boundary
(package hash), limb width is part of that encoding. Derive it; do not inherit
my split.

## 7. Acceptance criteria

- **`AC-A1`** — Both authority sites repaired, scoped as §4 requires. **Control:**
  the repaired text, read cold by someone who has not seen this WP, cannot be
  read as *implementation outranks specification*. **Second control:** the three
  divergences in §4 are still recorded somewhere after the edit — quote them
  from the post-edit file.
- **`AC-A2`** — Census delivered per §3, with the reading stated. **Control A**
  (the `probing` false hit) and **Control B** (the 4 MiB known answer) both run
  and reported. ⛔ Control B failing = the census is under-derived and `AC-A2`
  is not met.
- **`AC-A3`** — Non-vacuity: at least one empty and one non-empty consumer set,
  or an explicit method finding explaining why not.
- **`AC-A4`** — Every relaxation carries the five-item record **including the
  conformance pair**. A relaxation without item 5 is reverted, not defended.
- **`AC-A5`** — The stop list is complete against the census: every non-cleared
  candidate appears with its rows and its class. ⛔ **Every candidate is in
  exactly one of the cleared set or the stop list** — a candidate in neither is
  an unreported result.
- **`AC-A6`** — Inertness, with a positive control: `git diff --stat` shows
  **zero** `crates/` change and **zero** moved conformance rows. ⚠ Use
  `git diff --quiet` for the emptiness test — `--stat` always exits 0.
- **`AC-A7`** — The `⛔ DO NOT RELAX` list (node) is intact. **Control:** name
  each of the nine guarantees and the diff hunk (or absence of one) that shows
  its text is not weakened.

## 8. Reporting discipline — the axis list is a required field

⛔ **Any partial clearance names the AXES it covers.** Not *"the numeric family
is clear"* — *"the numeric family is clear on the tag and fast-path axes; limb
width is UNMEASURED."*

⭐ This exists because of what it cost on `RT-FNSPLIT-B2F`: a truthful *"no hard
stop"* was scoped to three measured axes, read by two readers as a verdict on
the node, and both of them built on it. **Nothing decayed and nothing was
mis-worded — the evidence base under the clearance grew.** ⇒ The reader's
question is always *is this item OK?* and yours is only ever *did axis N hold?*
Those questions have **different arities**, and prose silently collapses the
second into the first. The axis list is the repair, and it is a reported field,
not a stylistic preference.

**Terminal handoff states:** exact candidate SHA and bound base; the census
table; Control A and Control B results; the cleared set with five-item records;
the stop list with classes; the `AC-A6` inertness control output; and the axes
each clearance covers.

## 9. A1's own residual — named here so the next node does not discover it

A1 closes a **census** and an **authority repair**. It **cannot** establish that
the relaxed constraints are the *right* relaxations at the level of Ken's
design, because that judgement belongs to the Track C forks — most sharply
**C7**, which decides whether the logical `space` guarantee is separable from
per-space arenas and re-interning at all. ⇒ **A1's cleared set is provisional
against C7**, and the terminal handoff says so.

⚠ This paragraph exists because three nodes in a row on the `RT-FNSPLIT` chain
each shipped an inert artifact whose residual was **exactly the half its own
inertness made unverifiable** — and each was found by the node downstream. When
a node ships a partial result, its frame names who completes it.
