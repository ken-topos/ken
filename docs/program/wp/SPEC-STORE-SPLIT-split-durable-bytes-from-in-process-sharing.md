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

## `AC-4` — every coupled conformance row retargeted or retired, each with its reason

> ### ⛔ AMENDED 2026-07-26 — THIS AC SAID "ALL EIGHT" AND THAT WAS A CLOSED
> ### POPULATION CLAIM OVER A HAND LIST. The eight are a FLOOR, not the set.
>
> `spec-leader` verified at `bce75fec` that further live conformance producers
> assert the same mechanics — in the values, evaluation, capacity, and surface
> areas — and that capacity and surface **cross-case prose** names them too. So
> the eight below are **not** the population asserting global interning /
> same-slot / FNV-1a / probing / load factor / page size / reset.
>
> ⭐ **The tell was inside the table the whole time: four of its eight "lives in"
> cells read `(locate)`.** A table that cannot say where half its rows live was
> never a census, and I labelled it "all eight" anyway.
>
> ⇒ **Required, and it is a DELIVERABLE, not a check:**
> 1. **Retain the eight-row table below** — verified as the consistent narrow
>    treatment — and **ground every `(locate)` cell** to a real producer.
> 2. **Add a producer-derived expanded census** of every row asserting any demoted
>    mechanism, and give each one a retarget-or-retire disposition **plus its
>    cross-case prose**, in the **SAME candidate**.
> 3. ⛔ **Absence from my table is NOT a reason to leave a mechanism assertion
>    standing.** That is the specific failure this amendment exists to prevent.
>
> ⚠ **Derive the census from the PRODUCERS, not from a keyword grep.** A grep for
> `slot|page|arena|intern` over `conformance/` returns matches in unrelated senses
> — "page" and "slot" are both homonyms here — so a grep-derived list would trade
> my under-count for an over-count. ⛔ **Do not replace one false population claim
> with another.**

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

> ### ⛔ CORRECTED 2026-07-26 — MY `lives in` CELLS CONFLATED A **PRODUCER** WITH
> ### A **REFERENCE**, and the "each carry more than one row" note was FALSE.
>
> This section previously said *"`seed-runtime.md` and `seed-evaluation.md` each
> carry more than one of these rows."* `conformance/README.md §Case format`
> defines a row as a structured `##`/`###` `<case-id>` heading, and under that
> definition:
>
> - `evaluation/seed-evaluation.md:189` only says its own row **extends**
>   `dedup-shares-slot`; it does **not** produce a second `dedup-shares-slot` row.
> - `runtime/seed-runtime.md:44–50` only **points at** the canonical capacity
>   producer and names the old subsumed `runtime/addressing/no-lattice-on-hot-path`;
>   it does **not** produce `runtime/capacity/no-lattice-on-hot-path`.
>
> ⇒ **Eight unique IDs, nine heading occurrences** — the one duplicate is
> `dedup-shares-slot`. ⭐ **Sweep per-ROW still stands, but count producers by
> HEADING, not by mention**, or you will "retarget" a cross-reference and believe a
> row is done.
>
> **CV's producer-derived locations — these replace my four `(locate)` cells:**
>
> | row | exact producer(s) |
> |---|---|
> | `equality-is-slot-id` | `runtime/values/README.md:28–37` |
> | `dedup-shares-slot` | `runtime/values/README.md:18–26`; `runtime/seed-runtime.md:7–15` |
> | `structurally-equal-collections-o1-comparable` | `surface/collections/seed-collections.md:389–405` |
> | `no-lattice-on-hot-path` | `runtime/capacity/seed-capacity.md:158–169` |
> | `index-resize-preserves-slot-ids` | `runtime/capacity/seed-capacity.md:171–181` |
> | `arena-spans-pages-oversized-safe` | `runtime/capacity/seed-capacity.md:183–195` |
> | `reset-retires-ids-never-resurrected` | `runtime/capacity/seed-capacity.md:145–156` |
> | `det-sharing-dedups-by-slot` | `runtime/evaluation/seed-evaluation.md:187–200` |

### ⭐ The expanded population, as measured — start the census HERE, not from zero

`conformance-validator` produced the closure below. ⛔ **It is the floor for the
expanded census, not a substitute for it** — each row still needs its own
retain / retarget / retire disposition **and its cross-case prose**.

- **`runtime/values/README.md`** — `canonical-encoding-map-ordering`,
  `canonical-encoding-set-ordering`, `canonical-encoding-record-field-order`,
  `int-small-to-bignum`, `immediate-vs-interned-boundary`,
  `bignum-minimal-limb-encoding`, `dedup-across-kinds`: all turn canonical
  bytes / kind distinctions into same-or-different **slots**, or into mandatory
  interning.
- **`runtime/evaluation/seed-evaluation.md`** — the CAN2 preamble plus
  `det-same-term-same-value` and `det-canonical-order-independent` require
  same-slot / `O(1)` outcomes independently of the named sharing row.
- **`runtime/capacity/seed-capacity.md`** —
  `dedup-accounting-distinct-not-occurrences`,
  `loud-at-limit-raises-not-silent`, `at-limit-repeat-does-not-trip`,
  `reclamation-releases-pages`, `space-reset-is-isolated`,
  `escape-survives-sender-reset`, `no-automatic-gc`, plus the coverage and
  cross-case prose: these pin `Hit`-before-limit probing, page-buffer reclamation,
  per-space arena/index reset, recipient re-interning, stable ids, and no
  background reclamation.
- **`surface/collections/seed-collections.md`** —
  `string-nfc-canonically-equal-shares-slot`,
  `array-update-shares-unchanged-structure`, and the already-superseded
  `user-deceq-keyed-map-canonical-identity`, plus the cross-case invariant at
  `:897–915`. ⭐ **Keep NFC equality and persistence/immutability; do not retain
  mandatory same-slot structural sharing.**
- **`surface/bytes-io/seed-bytes-io.md`** — `bytes-immutable-concat-allocates-fresh`
  pins fresh/distinct slot ids. **Immutability survives; slot allocation does not.**
- **`surface/numbers/seed-f1-bignum-int.md`** —
  `f1-dedup-content-address-stable-across-paths`
  correctly retains identical canonical bytes / content address, **but its
  additional "one store slot — dedup holds" clause needs separating** from the
  surviving part.

⚠ **This is not a widening of the design.** It is the closure of the demotion
already ruled: canonical bytes/hash, equality, no-false-merge, immutability, loud
refusal, lifetime/isolation and non-corruption are all retainable **without**
asserting slots, pages, probing, or interning. ⛔ **A candidate that changes only
the eight named IDs would land normatively self-contradictory** — which is the
whole reason this is one atomic candidate.

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
3. **`OQ-Space` is RECONCILED IN PLACE — ⛔ it is already `DECIDED` and you must
   NOT fabricate a closure transition.**

   > ### ⛔ AMENDED 2026-07-26 — THIS ITEM PREVIOUSLY INSTRUCTED YOU TO CLOSE
   > ### `OQ-Space`, AND THAT INSTRUCTION WAS FALSE AGAINST THIS FRAME'S OWN BASE.
   >
   > `bce75fec:spec/90-open-decisions.md:502` reads **`OQ-Space — State,
   > concurrency & isolation — DECIDED`**, operator 2026-06-27, and the summary
   > row (≈`:825`) records the same. It has been decided for five weeks. The
   > *campaign doc* §6.7 is what says "remains open" — and **§6.7 is the stale
   > text**, not `spec/90`.
   >
   > ⭐ **The decision already contains this WP's authorization.** It settles the
   > **logical** contract — encapsulated, non-aliased cells; bounded per-space
   > Hoare, no separation logic; shared-nothing message-passing; closure-free
   > content-addressed transport — and then states that the **runtime realization
   > (process/thread/green/distributed) is deferred to `40-runtime`.** C7 is not a
   > reopening of `OQ-Space`; **C7 IS the realization question `OQ-Space` handed
   > to `40-runtime`,** and this WP is where it is answered.

   Required, therefore:

   - **PRESERVE, verbatim and unweakened:** the encapsulated / non-aliased cell
     state, bounded per-space Hoare, and **no shared mutable authority**. ⛔ These
     are the operator's 2026-06-27 decision — out of scope for this WP to touch.
   - **RECORD, as the discharge of that decision's own `40-runtime` deferral:**
     durable canonical bytes, copy/share, and per-`space` index / arena / reset
     realization become **private**.
   - ⛔ **Do not write an `OPEN → DECIDED` history, do not add a "closed by" line,
     and do not re-litigate the 2026-06-27 decision.** Reconcile the entry where
     it stands.
4. **CORRECT the campaign's stale wording — it is the false text.** In §6.7,
   *"Open decision `OQ-Space` already exists"* and *"`OQ-Space` remains open in
   `spec/90-open-decisions.md` and is the durable carrier — ⛔ do not close it on
   the strength of this deferral"* are both **wrong as written**: the entry was
   already `DECIDED` when that ruling was recorded. EDIT both to say that
   `OQ-Space` is decided and that C7 is the realization half it deferred to
   `40-runtime`.

⚠ **Why this defect existed, so you can catch the next one:** I took "`OQ-Space`
is the open carrier" from §6.7 and never checked `spec/90` — an **inherited**
premise I presented as a **derived** one. `spec-leader` reproduced the
contradiction independently at the released base and held rather than building on
it. **That hold was correct and is exactly what the perishability clause is for.**

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
clause was **still stronger than the mission needs**.

> ### ✅ ANSWERED 2026-07-26 — IT **WAS** EXERCISED. This AC previously said
> ### "unverified — establish it"; that question is now closed, by measurement.
>
> `conformance-validator` established it from the landed record at `bce75fec`:
>
> - CV's early challenge `evt_4jyaf3f36d854` rejected mandatory handle/trampoline
>   representation, a literal `StaticCallableRef` spelling/layout, a required
>   `FrozenClosure` feature, exhaustive/extensional application comparison, and
>   mandatory cross-artifact exchange.
> - The author handoff `evt_x5c47amhqdfb` states the `AC-S7` challenge was folded.
> - The final approved contract retained the minimum constraints, rechecked in
>   `evt_5znneqyam1fdj`.
>
> ⇒ ⛔ **Do NOT report the prior invitation as unexercised or silently skipped.**
> Probing it **anew against the store split** is still required — that is a
> different question from whether it was ever exercised — but the historical
> answer is **YES**, and stating otherwise would misdescribe a landed record.

⛔ **Silence is not "nothing to say."** If, while writing `AC-1`, you find a
closure clause over-strong against the relaxed store contract, **route it to the
Steward as a fork.** ⛔ Do not fold a closure relaxation into this candidate.

## `AC-8` — ⛔ WITHDRAWN. The ledger is NOT this candidate's concern.

> ### ⛔ WITHDRAWN 2026-07-26 BY OPERATOR RULING — AND THE WITHDRAWAL IS THE
> ### POINT, SO IT IS RECORDED RATHER THAN DELETED.
>
> **Operator, verbatim:** *"The librarian's responsibilities are a distraction to
> the spec enclave and the implementation teams. For them, the librarian is not a
> concern, downstream, and unobserved."*
>
> This AC previously required the `library/SOURCE-ATTESTATIONS` fold to ride the
> same SHA, and `spec-leader` had already tasked the Librarian inside this WP's
> thread on the strength of it. **That coordination is cancelled.** It is recorded
> here because the instruction went out; a reader who finds only silence cannot
> tell a withdrawn obligation from one nobody thought of.

⛔ **Do not touch `library/`. Do not generate, diff, or report an attestation
row. Do not coordinate with the Librarian.** If a `spec/` edit changes an
attested source's blob OID — it will — **that is downstream and not yours.**

⇒ **This candidate is `spec/` + `conformance/` only.** The one-candidate
constraint is unchanged and unrelated: it exists so a spec relaxation never lands
while conformance rows still assert the demoted mechanism.

> ⭐ **Why the original AC was wrong, not merely noisy.** The operator's own
> 2026-07-26 gate ruling made the currency ledger a **release-point artifact,
> explicitly not enforced per merge** — *"remove the CI coupling."* Writing a
> same-candidate fold into this frame **re-introduced per-merge ledger
> enforcement in a WP frame hours after it was removed from CI.** A gate deleted
> in one carrier and re-created in another has not been removed.
>
> ⚠ **And the alarm it raised was false.** The withdrawn text argued the AC was
> *"the only thing standing where a red test used to be"* and that a miss would
> surface as the next PR's red. **Measured at `ad1b9a01`, neither holds:** no CI
> step invokes `gen-doc-status.sh` or `gen-source-attestations.sh`; the live-corpus
> gates `check_source_currency` / `check_source_anchors` /
> `check_generated_current` survive as functions wired into `VALIDATION_GATES`
> (`crates/ken-cli/tests/library_documentation_gates.rs:577`) whose **only other
> mention in the workspace is a comment** — zero consumers. ⇒ **There is no red at
> either gate, so there was never a failure for this ring to prevent.**
>
> ⛔ **The trap for the next reader, since it nearly caught me:** eleven live
> `#[test]` functions in that file are named `content_currency_gate_rejects_…` and
> `ledger_set_mismatch_when_a_citation_is_added_without_a_ledger_row`, and they
> run green in CI. **Every one builds a fixture repo in a temp dir.** They test the
> detector, not the corpus. **A name that describes the corpus is not a check on
> it.**

## `AC-9` — the residual is stated

⛔ **Do not claim this makes `B2F` achievable.** Node §8: it removes the
contract-level conflation research identifies as the root cause and is the
strongest available lever; **whether the compiled-once call boundary then closes
is an open question for the Architect.** ⭐ Say so plainly — the previous five
days ran on the assumption that one more layer would close it.

## ⚠ On RQ links — none, and this is the stated reason

`15-requirements-and-acceptance-criteria.md` rule 1 asks that every AC name its
requirement **or that none do, with a reason.** None do here.

`SPEC-STORE-SPLIT` carries no `## Requirements` block because the requirements it
would restate are **the ones this WP changes.** The relaxation's whole content is
that a set of currently-normative obligations stops being required; deriving RQs
from the pre-relaxation contract would pin the ACs to the text they exist to
move.

⇒ **The RQ tier is owed by the `RT-NATIVE-FNSPLIT` re-cut** (node §7 item 1),
where the relaxed contract is the input rather than the output. ⛔ Do not add an
RQ block to this node to satisfy the checker — that would manufacture
traceability to a superseded contract.

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
reasons; the C7 ruling text and the **`OQ-Space` in-place reconciliation**; the
`AC-6` positive controls; and the `AC-9` residual. ⛔ **No `AC-8` ledger report —
that AC is withdrawn.**

> ⛔ **CORRECTED 2026-07-26 — this list said *"the `OQ-Space` closure"*, which
> `AC-5` forbids.** The amendment that rewrote `AC-5` to *reconcile in place* left
> this restatement of it standing, so the frame instructed both at once and the
> **Handoff is the section an author reads last.** ⭐ The general shape:
> **amending an AC does not sweep the sections that RESTATE it** — and a handoff
> checklist is exactly such a section, which is why it is the most dangerous place
> for a stale duplicate.

⛔ **No Decision is opened by the enclave** — that is the Steward's. Diff-scope
will pull the **Spec vote** (conformance-validator) and the **Architect**.
