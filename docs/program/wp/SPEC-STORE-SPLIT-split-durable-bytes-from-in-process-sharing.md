# WP `SPEC-STORE-SPLIT` — split durable canonical bytes from in-process sharing

**Node:** `docs/program/issues/SPEC-STORE-SPLIT.md` · **Owner:** spec-enclave ·
**Size:** L · **Gate:** none ·
**Blocks:** `RT-NATIVE-FNSPLIT`, `RT-FNSPLIT-B2F`, `RT-VALUE-TOTALITY`,
`PX8-F-CAP-41`

> ## ▶ READ THE NODE FIRST. This is not a design task.
>
> Research supplied the split and the prior art; `runtime-leader` corroborated it
> from five days inside the wall. ⭐ **Your job is to write it into `spec/` +
> `conformance/` and to say what stopped being required** — not to re-derive
> whether the relaxation is right.
>
> ⛔ **Do not re-open `SPEC-CLOSURE-BOUNDARY`'s six ruled clauses** (merged, PR
> #982). It is **upstream** of this campaign. Node §3a is the one seam.

## Objective

Ken's spec conflates two contracts that research shows are separable:

> **"Canonical durable bytes and maximal in-process sharing are separate
> contracts. The former can be required without the latter."**

Write that separation into the normative text, demote the runtime mechanism to
private, retarget the conformance rows that assert the mechanism, and rule fork
C7. Node §3 states the four-part split; ⛔ adopt it verbatim rather than
re-deciding it.

## ⛔ ONE CANDIDATE, NOT A SERIES — and this is a correctness constraint

Node §4: *"A spec edit that leaves them asserting the old mechanism has relaxed
nothing — it has created a contradiction."*

⇒ **The `spec/` relaxation and the `conformance/` retarget must ride ONE branch
and ONE Decision.** Splitting them opens a window in which `main` carries a spec
that says one thing and a locked corpus that asserts another.

⭐ This is also what makes the review correct: a combined diff touches `spec/`
**and** `conformance/`, so the diff-scope check pulls the **Spec vote**
(`COORDINATION §14`, point 4) — the conformance-validator, not the spec-author
who wrote it. A `spec/`-only branch would merge with the wrong reviewer set.

## `AC-1` — the durable-boundary clause EXCLUDES ordinary closures by construction

Node §3 item 1 retains extensional equality and deterministic canonical encoding
**for values that cross a durable boundary**. `SPEC-CLOSURE-BOUNDARY` says a
closure reaching that boundary is **refused before bytes exist** — ⛔ never
substituted by a pointer, ordinal, digest, or handle.

⛔ **Write the clause so that "values that cross a durable boundary" excludes
ordinary closures BY CONSTRUCTION — not as an exception appended afterwards.**
A durable-bytes clause written without the exclusion in front of it re-admits
the exact arm `RT-VALUE-TOTALITY` P2 exists to delete (`canonical.rs:182`,
measured still live).

**Control — state which form you wrote.** Quote your clause and say whether a
reader who knows only *this* clause (not the closure boundary) could conclude a
closure has canonical bytes. If yes, the clause is wrong regardless of what
another chapter says elsewhere.

## `AC-2` — the demotion names each mechanism, and DEMOTED ≠ FORBIDDEN

Demote to private, per node §3 item 2, each of: **global interning, same-slot
conformance, FNV-1a, probing policy, load factor, page size, slot retirement.**

Report a table: mechanism → the section that used to mandate it → its status
after your edit. ⛔ **A mechanism you did not locate is not demoted** — say so
rather than omitting the row.

> ### ⛔ THE TRAP: "need not" is not "must not", and it cuts both ways here
>
> Demoting **same-slot conformance** to private must **not** read as making slot
> identity newly *available* to closures. The closure boundary forbids closures
> having slot identity **at all** — which is strictly stronger than "the
> mechanism is private."
>
> ⇒ **Two different claims. State both.** *"The runtime may choose its slot
> policy"* and *"a closure has no slot identity"* are simultaneously true, and a
> reader who has only the first will get the second wrong.

## `AC-3` — `O(1)` equality is re-expressed, or explicitly dropped

If Ken deliberately promises `O(1)` equality, re-express it as a **performance
profile / complexity contract** — an NFR per
`15-requirements-and-acceptance-criteria.md` — ⛔ **not as a mandated hash
table**. If Ken does *not* mean to promise it, say so and drop it.

⛔ **Pick one and record which.** A clause that neither promises nor disclaims a
complexity bound leaves the implementer to infer it from the retired mechanism,
which is how the conflation got in.

## `AC-4` — all eight conformance rows retargeted or retired, each with its reason

| row | asserts | lives in |
|---|---|---|
| `runtime/values/equality-is-slot-id` | equality **is** slot identity | `conformance/runtime/values/README.md` |
| `runtime/values/dedup-shares-slot` | same-slot dedup is observable | `values/README.md`, `seed-runtime.md`, `evaluation/seed-evaluation.md` |
| `surface/collections/structurally-equal-collections-o1-comparable` | `O(1)` comparison | (locate) |
| `runtime/capacity/no-lattice-on-hot-path` | a **negative** mechanism constraint | `capacity/seed-capacity.md`, `seed-runtime.md` |
| `runtime/capacity/index-resize-preserves-slot-ids` | `2¹⁶`, `0.70`, double-and-rehash | `capacity/seed-capacity.md` |
| `runtime/capacity/arena-spans-pages-oversized-safe` | 4 MiB pages | (locate) |
| `runtime/capacity/reset-retires-ids-never-resurrected` | slot-id retirement | (locate) |
| `runtime/evaluation/det-sharing-dedups-by-slot` | determinism **via slots** | (locate) |

⭐ **Read the split before touching any row.** Several are **retained in
substance and relaxed only in mechanism**: *no false merge* and *no slot-id
resurrection* are real properties that survive; *FNV-1a* and *`0.70`* do not.
⛔ **Do not retire a row whose property is real just because its mechanism
moved** — restate the property mechanism-independently and keep the row.

⛔ **A row asserting the demoted mechanism after your edit is a contradiction on
`main`, not an oversight.** Every row gets an explicit disposition; a row you
leave untouched is a decision and needs its one-clause reason too.

⚠ `conformance/runtime/seed-runtime.md` and
`conformance/runtime/evaluation/seed-evaluation.md` each carry more than one of
these rows. Sweep per-**row**, not per-file — a file you have "done" can still
hold an unretargeted row.

## `AC-5` — fork C7 is RULED here, in its durable home

C7 (logical `space` vs physical structure) is defined at
`docs/program/14-spec-mission-alignment-campaign.md` **§6.7** (≈`:757`), and the
fork table row sits at **≈`:221`**, currently `✅ DEFERRED — operator concurred
2026-07-26`.

> ### ⭐ YOU ARE NOT OVERRIDING AN OPERATOR RULING — you are discharging its
> ### stated precondition. Read this before you hesitate.
>
> The concurrence reads: *"concur, defer for now"* — **"hold C7 until the
> content-store question lands."** ⇒ This node **is** the content-store question
> landing. Ruling C7 here is what the operator concurred to, not a reversal of
> it. ⛔ Do not route this back as a fork; do not stall waiting for a fresh
> operator confirmation.

Required:

1. **Rule it:** the logical `space` contract is **retained**; per-`space` index
   shape, arena organization, and reset mechanics become **private**.
2. **EDIT §6.7's operative text and flip the `:221` table row.** ⛔ **Do not
   append a correction below a stale ruling** — the superseded text stays
   operative and is the one a later reader finds first.
3. **`OQ-Space` in `spec/90-open-decisions.md`** was explicitly kept open, with
   *"⛔ do not close it on the strength of this deferral."* The deferral is now
   discharged, so **this WP closes `OQ-Space`** with the C7 ruling as its answer.
   ⛔ A ruling that leaves its own open-decision carrier open has not landed.

## `AC-6` — the four "must not" boundaries hold, with a positive control

Per node §6, this WP must **not**:

1. relax anything on the campaign's §8 *do not relax* list;
2. touch `41 §3`'s separation of cryptographic/Merkle serialization from
   in-process addressing — ⭐ **that separation is what makes the split
   possible**;
3. weaken **no-shared-mutable-authority** while relaxing shared-nothing
   *storage* — the campaign flags these as easy to confuse and they are
   different;
4. re-cut the runtime WPs (node §7, Steward's, after this lands).

⛔ **A green "I did not touch it" is not evidence** — it passes identically when
you never looked. **Positive control:** for items 2 and 3, quote the sentence
that carries the property and show it is byte-identical at your candidate, and
name the section you *would* have had to edit to break it. For item 1, list the
§8 entries you read.

## `AC-7` — probe `AC-S7`, and route what it finds as a FORK

`SPEC-CLOSURE-BOUNDARY`'s `AC-S7` invited the enclave to say whether a ruled
clause was **still stronger than the mission needs**. ⚠ **Whether that invitation
was ever exercised is unverified** — establish it, do not assume.

⛔ **Silence is not "nothing to say."** If, while writing `AC-1`, you find a
closure clause over-strong against the relaxed store contract, **route it to the
Steward as a fork.** ⛔ Do not fold a closure relaxation into this candidate.

## `AC-8` — ⚠ THE LEDGER RIDER, and after `LIB-GATE-DECOUPLE` NOTHING REPORTS IT

**13 `spec/` files are attested cited sources in `library/SOURCE-ATTESTATIONS`**,
including files this WP edits — `40-runtime/44-capacity.md`,
`40-runtime/42-evaluation.md`, `30-surface/36-effects.md`,
`40-runtime/45-native-backend.md`, and `90-open-decisions.md`.

⛔ **Editing one moves its blob OID — a locator-only edit does it too.** So this
candidate **owes the `library/SOURCE-ATTESTATIONS` fold in the same SHA.**

> ⭐ **Why this AC is written louder than it looks.** Until 2026-07-26 a CI gate
> caught a missing fold. `LIB-GATE-DECOUPLE` removes that gate by operator
> ruling — the generator is kept and runs at **release points**, so between
> releases **a missing fold is silent.** This AC is now the only thing standing
> where a red test used to be. ⛔ Treat a skipped fold as landing a known-stale
> ledger, not as a deferred chore.

⛔ **Never bump a row without diffing the CITED ANCHORS.** `library/manifest.toml`
and `library/learn/reading-ken/06-execution.md` cite `44-capacity.md` and
`42-evaluation.md`; `library/learn/exercises/solutions.md` and
`06-execution.md` cite `90-open-decisions.md`. A demotion that deletes or renames
an anchored section **breaks the citing page**, and the OID bump hides it. ⭐ The
recurring case is benign, which is exactly what trains the rubber stamp.

⇒ Report: rows bumped, and for each, whether the cited anchor still resolves.

## `AC-9` — the residual is stated

⛔ **Do not claim this makes `B2F` achievable.** Node §8: it removes the
contract-level conflation research identifies as the root cause and is the
strongest available lever; **whether the compiled-once call boundary then closes
is an open question for the Architect.** ⭐ Say so plainly — the previous five
days ran on the assumption that one more layer would close it.

## Evidence bar

- ⛔ **`git diff --stat` always exits 0** — not an emptiness test. Use `--quiet`
  or read `--name-only`.
- ⛔ **Verify every object you cite exists at the base you name** —
  `git cat-file -e <base>:<path>`, and quote the blob so review can bind it.
- ⭐ **A spelling enumeration is not a proof of the property.** For each demoted
  mechanism, the check is that no normative sentence still *requires* it — not
  that a token disappeared. A grep for `FNV` going quiet while a neighbouring
  sentence still mandates "the" hash function is a green that means nothing.
- ⛔ **State the DIRECTION of any weakness you find** in your own edit —
  over-strict or unsound. "Ambiguous" is not a finding a reader can act on.
- ⛔ Targeted validation only; workspace-green means **green in CI**
  (`COORDINATION §12`).

## Handoff

Return **one exact candidate SHA** with the branch freed, plus: the `--name-only`
diff; the `AC-2` demotion table; the `AC-4` per-row disposition table with
reasons; the C7 ruling text and the `OQ-Space` closure; the `AC-6` positive
controls; the `AC-8` ledger report with anchor resolution; and the `AC-9`
residual.

⛔ **No Decision is opened by the enclave** — that is the Steward's. Diff-scope
will pull the **Spec vote** (conformance-validator) and the **Architect**.
