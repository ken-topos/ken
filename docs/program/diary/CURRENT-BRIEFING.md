# Current briefing (live — read this first on every Steward resume)

> **This file is LIVE STATE ONLY.** When something here stops being true,
> move it to `diary/YYYY/Mon/DD.md` — do not append a newer block above it.
> Appending is what grew the old tracker to 2.23 MB.
> History: [`INDEX.md`](INDEX.md) · Work items: `docs/program/issues/*.md`

**As of 2026-07-26 ~00:05Z. OPERATOR IS PRESENT.**

> ### ⛔ THREE STALE `RESUME HERE` BLOCKS WERE REMOVED FROM THIS SPOT
>
> They anchored on `bf8036c0`, `5554b33f`, and `0aa9e53f` — **three, four, and
> five `main`-SHAs stale respectively** — and each was titled `RESUME HERE`, so a
> cold resume read the *oldest* one first. Their content (merged SHAs, retro
> status for Boundary A/B1, `DOC-GATE-WIRE-BINDING`, `RT-FNSPLIT-B2A-S`,
> `DOC-GATE-RECORD-AXIS`) is durably recorded in the issue files and git history;
> nothing was lost by compressing them.
>
> ⇒ **This file's own header rule was being violated by ME, one appended block at
> a time.** Each append was individually reasonable — the newest state on top —
> and collectively they turned the resume anchor into a trap. **The only live
> state is the block immediately below. If you are resuming, read that and
> nothing above it.**

## ▶ LIVE — 2026-07-26 ~00:2xZ · `origin/main` = **`4427147d`**
### **✅ THE B2V RECUT IS BOUND AND ON `main`. Production candidate still blocked.**

> ### ✅ PR #978 LANDED — exact `36064740`, tree `a4c8a604`, full CI
>
> **Verified by content, not the publisher's report** (it exits 0 on failure):
> **7/7 files blob-identical**; the frame blob on `main` (`58c34ad9`) is the
> **exact blob the Architect approved** at `e4fa5ec5`; `c72be0b0` still an
> ancestor, so **nothing landed since was reverted**. `steward/work` reset onto
> the squash-merged `main` — ⛔ the `wp/steward-b2v-recut-*` refs are **spent**;
> a squash-merged branch cannot be continued. Announced `evt_3xc5kqjg8bzbx`;
> all three receiving seats confirmed `Working`.
>
> ⭐ **NOW BINDING, and NONE of it has ever been discharged on this node:**
> `AC-10` total classified-domain closure (variant → static policy, input →
> entailed outcome); `D4`/`AC-3` **five** static encoding policies with the
> spill-variant→immediate-only ban; **`AC-1` layout closure** (field inventory is
> the sole authority; a declared extent is derived **and consumed** or does not
> exist; a causal control reddens on independent drift; ⛔ constant-vs-constant
> does not discharge). RETAIN keeps proved properties, **not** byte counts.
>
> ⚠ **`AC-10` was `NO CONTROL — open residual` on every candidate to date and
> `AC-1` layout closure did not exist until tonight. Neither inherits evidence
> from any prior verdict.**

> ### ✅ OPERATOR DIRECTIVE 2026-07-25 — `lemma` retired from the language
> entirely; permitted in comments and documentation. **VERIFIED ALREADY
> SATISFIED on landed `main`, by POSITIVE CONTROL not by absence:**
> `Lexer::lex("theorem lemma")` → `Token::Ident("lemma")` (ordinary identifier,
> **not** a keyword); a `lemma` declaration head is **rejected**
> (`RETIRED_SPELLING_SOURCE`); normative `spec/30-surface/31-lexical.md` and
> `32-grammar.md` have **zero** occurrences; a corpus-wide source oracle permits
> prose and forbids Ken source, probed against
> `LEMMA lemmas lemma's lemma_identifier` so it cannot false-positive.
> ⛔ *"Absent from the lexer"* alone would have been a **negative check that
> passes for any reason** — the positive control is the evidence.
> ⚠ **One deliberate residue:** `provide_lemma` remains a **normative protocol
> enum value** (`spec/20-verification/25-protocol.md`,
> `SuggestedAction::ProvideLemma`) — an **API token**, not a language construct;
> the mathematical concept *lemma* is not retired. Recommended left alone;
> **no WP opened.** ~350 other occurrences are ordinary prose, which the
> directive permits.

> ### ⛔⛔ FIRST, TWO THINGS THAT CHANGE HOW YOU OPERATE THIS SESSION
>
> **1. My convo MCP connection DIED mid-session.** Every `mcp__convo__*` tool is
> gone. ⛔ **`claude mcp list` reports `convo: ✔ Connected` — it LIES**, because
> it health-checks by spawning a *fresh* process; it says nothing about this
> session's stdio. **Fallback transport is live and verified:**
> `python3 <scratchpad>/convo_post.py <body.md> <agt_id>…` posts over the HTTP
> API with my own steward credential (endpoint `/api/spaces/{sid}/response`,
> **singular**). Reads work the same way — `convo_read.py` in the same
> scratchpad lists events newest-first; that is how `dec_74fwejgv6hda0` and
> `dec_7sd3enk81maws` were verified **off the object**. ⭐ **Losing the tool must
> not downgrade the §14 gate** — read the object, never the report.
>
> ⭐ **CORRECTION — mentions DO still arrive.** An earlier revision of this block
> claimed *"I cannot see mentions arrive"*; that was **wrong**, and acting on it
> would have meant polling blind. The `convo-channel` **subscription is a
> separate transport from the `convo` MCP tool set** and it survived
> (`list_subscriptions` → `spc_4q7g0se87rgje`; the Architect's recut-3 block
> arrived on it unprompted). ⛔ **But the notification is TRUNCATED** and
> `get_recent_context` is one of the dead tools — so treat the notification as a
> *doorbell only* and fetch the full text with `convo_read.py --full --since`.
> ⚠ **Two transports, two failure modes: verify which one is actually down
> before designing around the loss.**
>
> **2. NOTHING IS PUBLISHING.** PR **#977** landed (`c72be0b0`, exact
> `305dc6d5`, full CI — ⛔ **not** `--doc-only`, which merges with *no CI*).
> Verified **by content**, because the publisher exits 0 on failure. Keep that
> habit: the next publish inherits it, not the outcome.

> ⛔ **THREE DEAD CANDIDATES ARE NAMED BELOW. None of them will ever be
> published, and each has a *resolved or rejected* Decision attached — which is
> exactly the shape that gets one published by accident.** Read the SHA, not
> the Decision status.

### ▶ LANE 1 — `RT-FNSPLIT-B2V` — ⛔ RECUT. The Architect NAMED the predicate.

| | |
|---|---|
| branch on origin | `wp/RT-FNSPLIT-B2V-executable-value-abi` = **`fd4e7f08`** — ⛔ **BLOCKED** (production block **#4**) |
| Decision | `dec_7sd3enk81maws` **rejected on the object** — Architect, `evt_4bs6scfmt5ax0` |
| state | runtime-leader routed the three repairs (`evt_4ms9arc37p89w`); implementer folding from `fd4e7f08`, ⛔ **no force-push of any rejected checkpoint** |
| recut | ✅ **BOUND AND MERGED** — read the frame on `origin/main`, never a `wp/steward-b2v-recut-*` ref |

⭐⭐ **THE §5a-ii PREDICATE CHECK FIRED AND WAS ANSWERED `YES`**
(`evt_2zxt6m9bg43r2`). The three production blocks are **not** independent: they
are successive exposed faces of one incomplete claim — **the admitted
disposition is not closed under emitted producer → boundary word → separately
compiled consumer round trip.**

⛔ **The recut is authored** — read the `## RECUT` section of
`docs/program/wp/RT-FNSPLIT-B2V-executable-value-abi.md`. RETAIN everything proved (a named predicate is **not** a licence to
restart); REPLACE only the **shape** of the `AC` set with `AC-10`, total over the
admitted **disposition** (not over tags — the predicate is explicitly stronger
than *"all tags are enumerated"*); the three `NO CONTROL — open residual` rows
are **promoted into scope** as the predicate's uncovered faces. ⛔ **It does NOT
stop the ring** (Architect said so explicitly) and **does NOT choose a
mechanism** — that is the Architect's call, not the frame's.

> ### ⛔ ARCHITECT **BLOCKED** the first recut — folded at `cfe05e37`
>
> `wp/steward-b2v-recut` = `d6026a5c` (**blocked, preserved, NOT force-pushed**)
> → `wp/steward-b2v-recut-2` = **`cfe05e37`** (folded, rebased on `c72be0b0`).
> runtime-leader APPROVED; Architect blocked on three clauses, all correct:
>
> 1. ⭐ **`AC-10`'s disjunction was on the WRONG DOMAIN — vacuously
>    satisfiable.** I wrote *"for every admitted value, either round trip or fail
>    closed"*, which puts the failure arm **inside the admitted subset**, makes
>    admission non-semantic, and **goes green on an implementation that rejects
>    every represented value.** Now: classify first into *represented immediate /
>    represented handle / protocol-only / fail-closed forbidden*; behavior is
>    **entailed by the class**; **no represented value may take the failure arm.**
>    ⛔ **The lesson: the day before I OVER-strengthened a predicate; steering
>    away from that, I UNDER-constrained. Both are the same error — the predicate
>    written on the wrong domain. Fix the domain, not the strength.**
> 2. **"One control total over every value" is not an executable oracle** — the
>    domains are infinite, so it would have become *a finite case sweep wearing a
>    universal name*, which reads as total and is worse than an honest sweep.
>    Totality is now proved **structurally**. **One property / one `AC` — not one
>    test function.**
> 3. **RETAIN froze the disposition wholesale while the same recut allowed
>    narrowing** — self-contradictory. Now retains the sealed no-wildcard
>    *mechanism* plus classifications outside `AC-10`'s implicated domain.
>
> ### ⛔ BLOCKED A SECOND TIME — folded again at `8f4f0d06`
>
> `wp/steward-b2v-recut-3` = **`8f4f0d06`**. The whole-frame reconcile found
> `D4`/`AC-3` classify per **variant** (static policy) while my `AC-10` classified
> per **value** (runtime outcome). ⭐ **`Lowered::Int` proves both levels are
> real:** `RepresentedImmediate { spill: Some(Int) }` is **one static
> variant-level policy** — a small runtime `Int` yields an immediate word, a wide
> one a persistent handle. ⛔ **Calling the whole `Int` population *immediate*
> would let a proof attach handle evidence to ONE SAMPLED SPILL and never
> establish that EVERY spill partition carries the handle obligations** — a live
> vacuity route. Forcing the value-level `AC` to say *handle* contradicts `AC-3`.
> **Neither level may absorb the other.** Now: variants get a static policy
> (immediate-only / handle-only / **immediate-with-declared-handle-spill**);
> inputs get an outcome **entailed** by it; and a handle outcome **including a
> spill arm** must discharge class/owner/identity/lifetime.
>
> ### ⛔ BLOCKED A THIRD TIME — folded again at `d2fdee73`
>
> `wp/steward-b2v-recut-4` = **`d2fdee73`** (`evt_26ewm3bj6gqj2` → my
> `evt_5zhtt7zjp0tnd`). ⭐⭐ **BOTH remaining defects had ONE cause, and it is a
> method defect, not a content defect:**
>
> > *"A later note saying the earlier deliverable is false does not replace the
> > deliverable."* — Architect
>
> I had been folding corrections in as **appended clarifications** while the
> contradicted text stayed **operative and unedited**. `D4` still required *four*
> dispositions and defined *represented immediate* as *"payload fits the tagged
> word directly"*; RETAIN still froze the *64/112 layout* that the promoted
> wide-`Int` obligation necessarily changes. ⛔ **`D4` is the construction
> authority an implementer reads FIRST — so both readings lived in the frame and
> the WRONG one was the one positioned to be obeyed.** Appending *feels like*
> faithful transcription of the reviewer's words; it is leaving the defect live.
>
> **Folded in place:** `D4`'s table is **replaced** by five static encoding
> policies (immediate-only · handle-only · **immediate-with-declared-handle-spill**
> · protocol-only · fail-closed forbidden), `AC-3` names that same set and bans
> the spill-variant→immediate-only misassignment, and RETAIN keeps the **proved
> properties** (one derived layout · native exact-`Int` normalization dependency ·
> distinct content table · owner-correct lifetime) and explicitly **not** the byte
> counts — ⛔ *neither* 64/112 *nor* the successor 80/136, since the exact-SHA
> review found the declared 136 was a **144-byte publish with no consumer**.
> **The pin is the retained property, never the number.** Recorded: a reviewed
> layout delta required by an `AC-10` outcome is **predicate delta, not restart**.
>
> ⇒ **STANDING RULE now in the recut preamble:** every fold **edits the operative
> deliverable in place**, and a **whole-frame reconcile is part of the fold, not a
> step after it.**
>
> ⚠ **SIX distinct defects in this one document across four reviews by three
> readers** (vacuous domain · unachievable oracle · RETAIN contradiction ·
> policy/outcome · stale `D4` table · stale RETAIN layout freeze).
> ⇒ **A recut needs a review LOOP, not a single authoring pass.** Treat the next
> one that way from the start.
>
> ### ✅ FOLD-IN, NOT A BLOCK — `d2fdee73` closed both defects
>
> `evt_1tdq9g139snay`. `D4`/`AC-3`/RETAIN all accepted; *"the later clarification
> now explains rather than contradicts those authorities."* One held question
> answered by **ruling**, folded at `wp/steward-b2v-recut-5` = **`e4fa5ec5`**.
>
> ⭐⭐ **THE RULING TO CARRY — the header-constant face is `AC-1`, NOT `AC-10`.**
> `AC-10` closes an **admitted runtime value** under *emitted producer → valid
> word → separately compiled consumer*. The `fd4e7f08` header defect **never
> falsified that round trip**: the constant had **no consumer** and the published
> vector was large enough for every accessed field. The real fault is **one
> closed layout claimed while two inconsistent authorities exist, one unused** —
> an `AC-1` face. ⛔ **Widening `AC-10` to absorb every dead or drifting
> declaration would have destroyed the named predicate's boundary.**
>
> ⭐ **This cuts against the obvious instinct, so keep it.** I found an uncovered
> face and the pull was to **widen the nearest `AC` until it covered** — which
> quietly converts a **named** predicate back into an **enumerated** one, the
> exact failure this recut exists to end. Raising it as a **held question** was
> right because **the answer was a boundary, not an extension.**
>
> **Folded into operative `AC-1`:** the field inventory is the **sole layout
> authority**; a declared extent is **derived and consumed, or it does not
> exist**; publication emits exactly the derived extent; every emitted offset +
> field width lies within it; a causal control **reddens on independent drift**
> of inventory / published word count / declared extent / emitted offset.
> ⛔ **Constant-vs-constant equality does not discharge it.** Also now a
> **mandatory `AC-1` row in the QA map** — ⚠ `fd4e7f08`'s map was complete and
> honest and had **no such row**, which is exactly how 136-vs-144 passed a full
> `AC`→control review.
>
> ⚠ **RETAIN is not an acceptance control** (Architect). RETAIN keeps *"one
> derived layout"* as the architectural property; the **enforceable** obligation
> lives in `AC-1`, where a QA map can be held against it.
>
> ⚠ **Still owed:** Architect **bind** of `e4fa5ec5`; then a fresh QA
> `AC`→control map covering `AC-10` **and** the new `AC-1` layout-closure row.

### ▶ B2V candidate `fd4e7f08` — ⛔ **BLOCKED**, three production defects

> ⭐⭐ **THE PART TO CARRY: this block landed on a candidate QA had APPROVED**,
> with a complete `AC`→control map, a passing independent mutation proof
> (`NODE_LIMB_COUNT` → `NODE_FIELD_COUNT` reddened exactly at limb count),
> `ken-runtime` **398/0**, and honest residual accounting. Three production
> defects still sat **outside** the map. ⛔ **A green `AC`→control map is not
> coverage.**
>
> ⭐ **All three defects have ONE shape — the Rust side states the law and the
> emitted side does not enforce it:**
>
> 1. `BOUNDARY_REGION_HEADER_BYTES = 136` vs an **18-word (144-byte)**
>    `BoundaryRegion::publish`; the constant **has no consumer and no equality
>    pin** anywhere in the tree, so the reviewed "112 → 136" layout claim is both
>    false and unenforced.
> 2. Emitted `define_store_int_limbs` admits `len = 0`, leading-zero limbs and
>    **negative zero** — all forbidden by `RuntimeIntV1::canonical_sign_and_limbs`.
>    Its committed control uses an arbitrary nonzero seed and **never exercises
>    the invalid boundary.**
> 3. The region-limb reader computes **wrapping** `end = at + region_len` and
>    tests only `end <= live`, where the Rust oracle uses `checked_add`. A wrapped
>    span passes and forms an address — the source comment claims fail-closed.
>
> ⛔ **That is `B2V`'s own founding diagnosis one layer down** — this node exists
> because the aggregate-result path was *a Rust-side decode, not a value
> representation.* ✅ **RULED `evt_1tdq9g139snay`:** the recut predicate reaches
> **(2)** and **(3)**; **(1) is an `AC-1` layout-closure face, not an `AC-10`
> one**, and is folded there. See the fold-in box above for why that boundary
> matters more than the fix.

Measured on the push, not taken:

| claim | verified |
|---|---|
| base / merge-base `aecdb001` · 11 files `+7472/−4` · `diff --check` clean | ✅ |
| intersection vs `c72be0b0` | ✅ **empty** |
| fast-forward over blocked `ddff2fae` | ✅ — all four prior candidates stay reachable |

⚠ QA records `AC-10` as **`NO CONTROL — open residual`** — correct, since the
recut does not bind yet. ⛔ **Keep that row even while the `AC` is unbound:** an
`AC` with zero controls is invisible to a review that examines controls, so
*discharged* and *never asked* read identically. ⚠ **The last three residuals
were honest, correct — and turned out to be the predicate's uncovered faces.
A standing residual is a debt, not a disposition.**

⛔ **DEAD, NEVER PUBLISHABLE:** `78a57d90` (`dec_58gv9rmjqy49g` rejected),
`657f60a0` (`dec_1wpa1y2b3g7cn` rejected), and now `ddff2fae`. All stay on origin
as durable checkpoints — every candidate has been a **fast-forward**, so
`ea8d9824` and all three rejects remain reachable. **Do not force-push this
branch.**

⭐ **`ddff2fae`'s increment is test-layer only, VERIFIED not taken:** all hunks
fall in `3502–3803`, inside `mod tests` (`1976`–EOF) of
`boundary_value_clif.rs`, and only test fns/helpers are added or removed. That
is what lets `ea8d9824`'s production review carry. ⛔ **My first check of this
was vacuous** — it compared against the *first* `#[cfg(test)]` at line **51**,
a bar any change clears. The real boundary is the `mod tests` block.
⚠ Open for QA: the increment renames
`b2v_a_separately_compiled_consumer_distinguishes_…` to
`…constructs_…_by_content`, dropping the phrase the Architect's finding #1
turned on. Confirm separate compilation survived; bind it to an `AC` row.

The three Architect findings `ea8d9824` folds: a handle ABI lossy to emitted
code (spilled `Int` read as `0`; `Bytes`/`String` indistinguishable at equal
length), `ken_boundary_store_slot_local` accepting a caller-supplied `SlotId`
(emitted code could forge store identity), and `alloc` admitting the Cartesian
product of tag × class.

⚠ **`ea8d9824` touches one path outside `crates/ken-runtime` —
`docs/program/rt-fnsplit-b2v-evasion-table.md`. That is a DOC, so it is NOT
hard-stop #11.** The armed condition is a *production* path outside the fence.
Do not fire the trigger on it; equally, do not let it become the precedent that
lets production code sit outside.

### ▶ LANE 2 — `KW-THEOREM` — ✅ **MERGED** at `c72be0b0`, PR #977

> ✅ **LANDED and VERIFIED BY CONTENT** — the publisher exits 0 even on failure,
> so its own post-merge claim is not the evidence. Landed tree **`6f7cf51c`**,
> byte-identical to the tree asserted **before** publishing; all six changed
> files carry the candidate's exact blobs, ledger included. Tracker flipped to
> `merged`, **body-text tail corrected** (it still read *"Now `ready`"*),
> `gen-progress.sh` re-run.
> ✅ **RETROS ALL IN — the node is CLOSED.** doc ring `evt_7q33mga74cn35`, spec
> enclave `evt_12cvdyyfkpxfd`, language ring `evt_66bdnv195eaed`. Adversary
> notified and has reported (`evt_4q06tgtrw6bv`) — **triaged into the new
> `KW-ORACLE-CLOSURE` node**, two structural findings, both zero-live-instance.
> ⭐ Its strongest negative result: `library/SOURCE-ATTESTATIONS` re-derived
> **50/50 current** against the landed tree — the check it most expected to find
> red.
> ⚠ **I buried this landing mid-message and two leaders truncated before
> reaching it**, then sat waiting for a report I had already sent. **A gating
> fact must LEAD the message.** Re-sent leading with it; both picked it up.

| | |
|---|---|
| branch on origin | `wp/KW-THEOREM-surface-keyword-rename` = **`305dc6d5`** |
| base | `c2c1ba9f` · 124 files, `+1613/−1234` + a 6-file `+16/−8` repair |
| Decision `dec_74fwejgv6hda0` | ✅ **`resolved`**, `resolved_by=agt_37reqwresqc00` @ `23:21:22Z` — **read off the OBJECT** |
| Decision `dec_286hqjak5kjq8` | ⛔ **VOID — resolved, but bound to DEAD `963d36ac`** |

⚠ **`proposed_by == resolved_by`** — a self-resolution, so the status field alone
is not sufficient. Substance verified instead: three **fresh exact-SHA**
authorities on `305dc6d5` — Librarian PASS `evt_524fj8c43q7jg`, CV APPROVE
`evt_11tsr3hhmxfbj`, Architect APPROVE `evt_6hk6m7x8xmsn4`. Merge authority here
is **Spec + Architect**; both present.

⭐ **Integration asserted, not assumed.** ⛔ A conflict-free `merge-tree` is *not*
evidence of preservation — disjoint hunks merge as a silent union, which is
exactly how `SOURCE-ATTESTATIONS` merged clean **and wrong** on 07-22. So the
post-condition was predicted first, then measured blob-by-blob over all **128**
files: 122 candidate-only → candidate's blob; 6 main-only → main's blob; both
intersecting files → merged blob differs from **both** sides (a real merge).
**Violations: none.** merge-base `c2c1ba9f`, tree `6f7cf51c`, intersection
exactly the two `docs/program/` files.

⚠ **No §2a tracker-sync commit was added, deliberately** — it would move the SHA
and void the exact-SHA Decision three authorities had just approved. Tracker
syncs on the next batch.

⛔⛔ **THIS IS THE TRAP ON THIS BOARD: a RESOLVED Decision on a RED candidate.**
`resolved` + non-null `resolved_by` is necessary, **never sufficient**. PR #977
is blocked, the publisher was killed, and **nothing landed** — `main` never
moved off `fdda953f`.

```
ken-cli::ken_fmt strict_frozen_corpus_gate_is_green   FAILED
crates/ken-cli/tests/ken_fmt.rs:111 — frozen corpus is not canonical:
  catalog/guide/proof-techniques.ken.md
  catalog/packages/Core/Classes/EffectfulClasses.ken.md
  catalog/packages/Core/Logic/Transport.ken.md
  catalog/packages/Data/Collections/Map.ken.md
```

**Measured over the whole population, not a sample:** of the 23 changed catalog
`.ken.md`, **all 18 passing files have a longest `theorem` line ≤ 95 columns;
all 4 failing files are ≥ 97.** Perfect separation, boundary 96. `theorem` is
two characters longer than `lemma`, and every file's diff is an exact N-for-N
line swap.

⭐ **CONFIRMED from the source side (operator pointed at `kenfmt`):**

```
crates/ken-elaborator/src/layout.rs:12
pub const CANONICAL_WIDTH: usize = 96;
```

⛔ **This is corroboration, not an echo** — the empirical boundary was derived
from file contents *before* the formatter was opened, and the constant was read
from source. **No shared premise.** (Contrast the scanner-reproduces-the-
documented-count trap, where both sides used the same naive match.)

⇒ **Reads as the formatter being RIGHT and the corpus being STALE**: the
migration swapped the keyword line-for-line without re-emitting through
`ken fmt`, so lines that fit at `lemma` no longer fit at `theorem`. Remedy is
re-canonicalization, **which the ring must confirm by RUNNING the formatter** —
reading a constant is not observing output. ⚠ And validate against the whole
frozen corpus, not the four filenames in the failure message; CI stops at the
first failure, so four is a symptom count, not a scope.

⛔ **A fresh descendant SHA needs a FRESH Decision.** The Architect's, CV's and
spec-author's approvals were all **explicitly exact-SHA** and do not carry;
merge-tree `4932d845` and the empty-intersection result are void. Push the fix
as a descendant — **do not force over `963d36ac`.** Merge authority is
**Spec + Architect**, not §14a doc-only.

⭐ **Four independent reviews approved this SHA and every local targeted run was
green. CI was the only thing that caught it** — which is exactly what §12
asserts, and why *workspace-green* means green in **CI**, never a local run.
**The frame predicted this failure by name** (coupling #3: the formatter
keyword list *"is a canonicalization oracle that fails in CI, not in a targeted
build"*) and it still happened, because that warning lives in a section read
once at kickoff.

### ▶ ARMED COUNTERS — the SOLE count of record. Re-read at every hard-stop.

- **FNSPLIT hard-stop count of record = `10`. NEXT RESEARCH PULL = `#11`**
  (`runtime-leader` armed it in-fold: any closure needing a path outside
  `crates/ken-runtime` is #11).
  > ⭐ **`#11` CONFIRMED and propagated 2026-07-25.** Three tracked files carried
  > **`#12`** (the generic next-multiple-of-3 after the consumed `#9`) against
  > this briefing's `#11`. The steward playbook carries an **operator override**
  > (2026-07-24) spelling *"catch-up set to `#11`, then `#15`, `#18`, `#21`"*.
  > The readings cannot be reconciled from the dates, so it was settled by
  > **dominance, not guess**: `#11` is *required* under one reading and merely
  > *early* under the other, and early is explicitly fine; `#12` is wrong under
  > one. Operative anchors corrected in `RT-NATIVE-FNSPLIT.md` and
  > `RT-FNSPLIT-B2V.md`; **append-only history deliberately still reads `#12`.**
- **SYMPTOM INVENTORY = 3 entries. NEXT PREDICATE CHECK = the 6th entry.**
- ✅ **§5a-ii PREDICATE CHECK — FIRED 2026-07-25 AND ANSWERED `YES`.**
  `evt_2zxt6m9bg43r2`. Three consecutive Architect production blocks
  (`78a57d90`, `657f60a0`, `ddff2fae`) share one predicate; recut authored at
  `4d705465`. **New armed counter lives in the `B2V` WP frame's `Standing`
  section — it is the count of record, not this line.**
- ⛔ **CONSECUTIVE ARCHITECT PRODUCTION BLOCKS = `4`.** `#4` is **`fd4e7f08`**
  (`dec_7sd3enk81maws` **rejected on the object**, `evt_4bs6scfmt5ax0`).
  **NEXT PREDICATE CHECK = block `#6`.** ⚠ Recut-frame review blocks
  (`d6026a5c`, `cfe05e37`, `8f4f0d06`) are **NOT** production blocks and do
  **not** move this counter — they are the Steward's document, not the ring's
  mechanism. Keep those two populations separate or the counter stops meaning
  anything.
- ⛔⛔ **WHY THAT COUNTER HAD TO BE INVENTED — the lesson to carry.** §5a-ii
  counts **hard-stops**, and a review block is correctly **not** a hard-stop. So
  all three production blocks moved **neither** the hard-stop count **nor** the
  symptom inventory, and **every armed line in the repo read correct and current
  the whole time.** It fired only because it had been hand-armed here. ⇒ **A
  counter keyed on ONE event class is blind to a different event class producing
  the same failure.** The fix is never to loosen *"a review block is not a
  hard-stop"* — it is a **second counter, beside the work.**
- ⛔ **Ask the question ONLY. NEVER name a predicate** (§5a-ii) — that is the
  Architect's call, and naming one makes the Steward the de-facto designer of
  the recut. *"No, these are independent"* is a complete answer.
- ⛔ **A review block is NOT a hard-stop, and a clean WP is not one either.**
  `B2R` took two review blocks and the count correctly stayed at 10; `B2O` ran
  clean and did not move it. Inflating the count pulls the research trigger
  early and teaches the chain that *found incomplete* and *hit a wall* are one
  event. They are not.

### ▶ Durable refs on origin

```
wp/RT-FNSPLIT-B2V-executable-value-abi    fd4e7f08   (BLOCKED; ddff2fae, ea8d9824,
                                                      657f60a0, 78a57d90 all reachable)
wp/steward-b2v-recut-1..5                 SPENT      (squash-merged as PR #978;
                                                      cannot be continued -- the
                                                      frame lives on origin/main)
wp/KW-THEOREM-surface-keyword-rename      963d36ac   (CI-RED, kept, do not force over)
architect/work                            e560cb20
```

⛔ **NOTHING in this list is ever force-pushed.** Every blocked candidate stays
reachable — it is the artifact a reviewer's block is *about*.

Each verified by `ls-remote` **at the exact SHA after the push**, never from
push output.

### ▶ My queue

| # | item |
|---|---|
| #48 | ⚠ **IN PROGRESS.** Three tail corrections committed. Remaining: archive the superseded 07-21/07-22 narrative. ⛔ **Do NOT bulk-archive — ~half the tail is durable law.** |
| #54 | ⛔⛔ **THE SCALING GATE HAS NO TRACKED NODE.** `RT-NATIVE-FNSPLIT`'s merge condition — Boundary A's planner census **and** the n=3..7 empirical harness + analytical model + verdict — exists **only as prose inside `RT-NATIVE-FNSPLIT.md` and the recut frame.** ⚠ **That is the KW-THEOREM failure shape exactly: a requirement stated in a document nobody executes against.** Frame both as real nodes **before `B2F` lands**, so the gate is not discovered at the end. Carry into them: workers on the **product's 8 MiB stack** (not the 256 MiB `ken-cli` convention — 6 sites already blind); **`k` (recursive lowering frames) is UNKNOWN and must be measured** before the model can consume it; ⛔ **there is NO baseline** — report absolute values, and the historic n=4 `1,482 states` figure is **non-comparable**. |
| #51+#12 | ⭐ **Promote the candidate-boundary-control lesson** (all three `KW-THEOREM` rings converged on it **independently**) + B1R retros → playbook corpus. **Batch; do not publish singly.** The deliverable is the **executable edge** (the language ring's post-corpus formatter return edge), not the policy sentence. |
| #52 | Frame **`KW-ORACLE-CLOSURE`** (`draft`, filed 2026-07-25) — the adversary's two post-merge findings on the `KW-THEOREM` source oracle. Both **zero live instances**, both **structural**. ⛔ Do not close by re-running the measurements; and ⛔ **P2's fix must not be a sixth `classify` arm.** |
| #5 | Frame `ABI-S3` shovel-ready |
| #11 | `DOC-GATE-NEEDLE` — ⛔ **operator-HELD. Do not release, do not re-ask.** |

### ▶ Environment

Disk: `/` at 71%, `/workspaces/ken` at 80% after a ~64G reclaim. Mass is
`.worktrees/*/target` plus an orphaned `~/.cache/ken-sccache`; **verify the live
`SCCACHE_DIR` from the server's `/proc/<pid>/environ` before deleting any
cache**, and check live processes before touching any `target/` — a seat can
read idle in tmux while 30 rustc processes run.

⛔ **`pkill -f <pattern>` MATCHES YOUR OWN SHELL** if the pattern appears in its
command line. It killed my own bash while stopping the publisher. Use `pgrep`,
read the PID, then `kill` that PID.

⛔ **Busy-detector regex — the spinner verb is RANDOMIZED** (`Booping…`,
`Churned for…`). Never grep `Working (`:

```
esc to interrupt|\([0-9]+m [0-9]+s|\([0-9]+s ·|[0-9.]+k tokens|Compacting|Press up to edit queued
```

## Standing state

- **`origin/main = 8ebe370a`** (`PX8-F-CAP-41 Phase 1` — sealed checked buffer-
  capacity handle, §38 fold). Green. Recent landings behind it: `cbf6a298`
  PX8-SPAN-PROV Phase 2 → `b64ad9f3` Phase 1 → `4ac9141e` SEAL-2 →
  `238a5c5d` RT-ESCAPE.
- **⚡ THE BOX LOST POWER at ~16:19Z, and again at ~16:33Z** (two disconnect
  waves in the space feed; operator-confirmed). **Every seat re-oriented from
  cold.** Consequences that outlived the restart are in the next block — read
  it before diagnosing anything as a stall.

### ⛔ THE ONE LIVE CHAIN — RT-NATIVE-FNSPLIT, hard-stop #33

**The entire runtime frontier is serialized behind this single chain**, and
everything else downstream of it is blocked by construction:

```text
RT-NATIVE-FNSPLIT (active, runtime)
  └─ NATIVE-HANDLE-CARRIER (draft)  ← blocked on it
       └─ PX8-F-CAP-41 Phase 2      ← blocked on that
```

| seat | state @ 17:2xZ |
|---|---|
| `runtime-implementer` | **implementing the #33 ruling** off `d43b6933` (ack `evt_61n2zpnj5rgvm`) |
| `architect` | **#33 RULED** `evt_55c62m0anfyyk`; now working the **viability review** |
| `research` | #33 advisory delivered `evt_138ty4vycfgr5`; idle/ready |

**#33 ruled: choose A** — compose the selected head into the exact final
producer cursor *before* `reserve_partition_source_return`, giving
`producer_head = W(selected, successor=T)` / `live_producer_tail = T`, with STOP
kept strict and the consumed cursor admitted only as a **pointer-free spent
receipt**. Normal execution is exactly `W once → T once → CompletedTail(T)`.

### ⛔⛔ VIABILITY RULED 2026-07-24 — HOLD + RECUT. THE CADENCE IS OVER.

**`evt_3m1g3v4m2bj51`. Runtime is HELD at clean `b077eb7a` until I author the
recut frame — that is the Steward's owed act and nothing moves without it.**

- **Single-`Function` inlining is DEAD AND ALREADY REPLACED.** The root cause
  this WP was filed on is **stale**; `b077eb7a` already emits one function per
  `PartitionWorkItem`. The issue file's title + origin section are now marked
  superseded inline.
- **The mechanism family is VIABLE (Θ(n) reachable); this REPRESENTATION is
  not.** Helper identity is a Cartesian tuple of variable-width dimensions, so
  **Θ(n) states × Θ(n)-wide data ⇒ Θ(n²)** descriptor/comparison/frame work.
  ★ **Hash-consing children cannot save it** — it shares equal subterms, it
  cannot merge distinct tuples whose components merely overlap.
- **More per-hard-stop sealing has NO route to the gate.** Sealing is linear
  only in the graph it *receives*.
- **Count FROZEN at 33; research cadence SUSPENDED** with the machine it
  counted (#34 is evidence, not a ruled stop). Re-arm against the recut chain.
- ★ **Honest reading of n=4 preserved:** one point cannot establish an exponent
  — `370n`, `93n²` and a threshold-at-n=5 all pass through it. **The hold rests
  on code inspection rejecting an O(n) proof, not on curve-fitting.**

**Next unit is a planner/census recut — NOT #35:** generate n=3…7 *before*
lowering bodies; acceptance needs bounded first differences **and** structural
invariants (fixed K helpers per static node, constant max key/frame width).

★ **This is what the operator's viability call bought:** 33 hard-stops of
correct, converging semantic work were being spent on a representation that
provably could not reach the gate. **Every individual ruling was right; the
thing they were accumulating into was not.**

### ⚡ The escalation that produced it (for method, not status)

Operator ruled 2026-07-24 that 33 hard-stops with SP-A still scaling-red
warrants a step back. Escalated to the Architect as `evt_98j3z2n49bpg`:

> Can single-`Function` inlining + defunctionalized continuation sealing reach
> the O(n) scaling gate **at all**, or is 1,482 states at n=4 structural
> super-linearity further sealing cannot remove?

I framed the question only — the ruling is the Architect's. **Sequencing call I
made:** the implementer *continues* #33 meanwhile, because that ownership fix
moves toward the constant-width form under any outcome; the Architect can hold
it with one word. Told it to pull Research immediately rather than wait for #36.

★ **Best grounding came from the Architect's own #33 ruling:**
`PartitionProducerKontSiteActionKey::ApplyActiveEliminators` still carries
**vector-shaped** `pending`/scope/capture data (`partition.rs:5700–5777`) — it
called this *"not the scalable form of this ruling."* A measured defect, not a
forecast.

**Thread for everything on this chain: `thr_7vdcfhxgfw128`.** The open question
is *selected-head ownership at the STOP predicate* — candidates A (precompose
selected-head work into the exact final producer cursor) vs B (selected
ancestry/scope as typed dynamic activation subordinate to the producer cursor).
⚠ **That is the Architect's lane. Do not form or transmit a view on it.**

**Two stacked transport failures cost this chain a restart's worth of latency —
both repaired 2026-07-24 ~16:5xZ:**

1. The Architect's advisory request `evt_3vr382mrv99pe` posted with an **empty
   `mentions` array** ("mentioning Architect only" in its prose refers to
   *research's reply*, not its own routing). Research is a **no-poll seat**, so
   it was never notified and re-oriented to *"awaiting dispatch"* while the
   Architect sat `blocked-on-Research` **indefinitely**. Repaired by pointing
   research at the event verbatim (`evt_d2b3vahe7khj`) — **transport only, the
   question was not restated.**
2. **`architect` and `librarian` both died on `⚠ Selected model is at
   capacity`** mid-re-orientation. Research runs the *same* model and was fine,
   so **capacity is transient per-request, not a model-wide block** — a rouse
   clears it (architect roused → `Working`). `librarian` re-oriented fully and
   holds no obligation, so its idle is correct.

★ **The generalizable tell: a seat whose turn dies on a capacity error is
indistinguishable from a healthy idle seat.** Neither posts, neither pushes.
Only a **wide** `capture-pane` shows the `at capacity` line above the composer.

- **§5a research trigger is ARMED in the issue file** (it was not — that is why
  this chain ran **10 hard-stops dry**). **My tracker is the count of record**;
  the Architect re-derives its own across compactions and loses.

  > ### ⛔ THE NUMBERS ARE NOT HERE. **`▶ ARMED COUNTERS` in the LIVE block is
  > ### the single count of record.** This section carries only the lessons.
  >
  > **2026-07-25: this block used to restate the counters, and a briefing that
  > states them twice has no count of record at all** — two copies drift, both
  > read authoritative, and the reader cannot tell which is live. That is the
  > same defect as the one below, one level up, so the duplicate is gone rather
  > than re-synchronised.
  >
  > ⛔ **Two chains exist and their numbers are NOT interchangeable.** The
  > **original** chain is **FROZEN at 33 hard-stops — do not resume that
  > count.** The **live** chain is the recut one (`wp/RT-NATIVE-FNSPLIT-recut.md`
  > opened it at `1` on 2026-07-24, cadence `#3, #6, #9, #12, …`). This bullet
  > once carried the frozen chain's `33`/`#36` onto the live chain.
  >
  > ⛔ **Why that mattered more than a wrong number:** a `#36` anchor on a chain
  > standing at `9` makes the trigger **unreachable** — 27 hard-stops of
  > headroom on a mechanism that exists to fire every 3rd. The armed line was
  > *present*, so every *"is it armed?"* check passed. **An armed trigger with a
  > stale anchor reads exactly like a working one**, which is the same defect
  > class as the `10-hard-stops-dry` run it was written to prevent.
  >
  > ★ **A clean WP does NOT advance the count** — `RT-FNSPLIT-B2O` produced no
  > hard-stop and the count correctly stayed put. **Neither does a review
  > block:** `B2R` took two and stayed at 10. A hard-stop is the implementer
  > discovering it *cannot proceed*; a block is a candidate being *found
  > incomplete*. ⛔ Do not "catch up" the number for elapsed work — inflating it
  > pulls the research trigger early and teaches the chain that the two are one
  > event.
- **⛔ RT-NATIVE-FNSPLIT DOES NOT MERGE ON "the tests pass"** — the operator's
  scaling gate (`evt_4btfhwqhah1ye`) binds: empirical n=3..7 harness +
  research-grounded analytical growth order + a verdict. **SP-A is
  independently scaling-red at 1,482 states / 1,525 edges**; no advisory in this
  chain has cured that, and each one says so explicitly.
- **Build side is capped at TWO implementation tracks** (operator). Doc is the
  standing exception; the enclave is not a build team, so it does not count
  against the cap. **Track 1 = runtime/FNSPLIT. Track 2 is currently EMPTY.**
- **⛔ IDLE BUILD RINGS ARE CORRECT — DO NOT "FIX" IT.** Operator ruling
  2026-07-22: *"just doc plus two implementation tracks."* `kernel`, `language`,
  `ergo` idle with zero ready items is the **intended** state, not a stall.
  **There is NO WP-authoring obligation for them.**

### ⛔ Retraction: the stale-base "near-miss" was FALSE (2026-07-22 ~13:43Z)

I held ABI-REVOKE claiming a stale base *"would have silently DELETED"* two
DOC-W1-2 chapters, and wrote it up as a general rule. **Disproved by
measurement:** BUDGET-EFF was on the **same stale base**, **also lacked both
chapters entirely**, and after merging **both chapters survived** — `214bf4de`
has one parent (squash). **GitHub squash-merge applies `merge-base → branch`,
not `main → branch`.** Absence from a stale candidate is not a deletion.

★ **`git diff --name-status origin/main <sha>` is NOT a staleness detector** —
it fired identically on the safe candidate and the suspect one. **The correct
test is the INTERSECTION:**

```sh
BASE=$(git merge-base <sha> origin/main)
comm -12 <(git diff --name-only $BASE <sha> | sort) \
         <(git diff --name-only $BASE origin/main | sort)
```

Empty ⇒ staleness immaterial, publish. Non-empty ⇒ inspect those files, take
the union deliberately. The ABI-REVOKE rebase was still the **right call** —
its `library/REVISION` + `manifest.toml` bump was a genuine non-empty
intersection — **but for a reason I stated wrongly.** Retracted publicly in
`evt_54t014vnef2xr`.

### ★ QA bound-verdict attestations are now on `origin` (2026-07-22 ~12:53Z)

The commit-your-verdict workaround (used when a QA seat's convo outbound dies)
left verdicts on **one local ref in one clone** — `a4473ab0` had **no second
copy anywhere off this box**, and `handoff-gate-compact.sh` has already
hard-reset that exact branch once (`preserved/runtime-qa-work-7c86db36`).
Pushed to durable refs, each verified by `ls-remote`:

```
attest/runtime-qa-verdicts   a4473ab0   (53501ffe is an ancestor — both carried)
attest/ergo-qa-verdicts      cf791c7f
attest/verify-qa-verdicts    04efa001
```

**This fixes durability, NOT discoverability** — a branch name nobody watches
is still a pointer someone must deliver. **Transcribing these into the repo
proper is the actual close and needs a WP.** ⛔ Do not reset, clean, or
re-anchor `runtime-qa/work`, `ergo-qa/work`, or `verify-qa/work`.

★ **The transferable lesson:** *when a workaround relocates a failure mode, the
new location inherits none of the scrutiny the old one had.* The pattern fixed
a real selection error and quietly moved the fragility from the message layer
to the storage layer, where nobody was looking.

> ### 🚨 INFRA ESCALATION FOR THE OPERATOR — `runtime-qa` convo outbound is DEAD
>
> ⚠⚠ **UNVERIFIED SINCE THE 2026-07-24 POWER CYCLE — DO NOT CITE THIS, TEST IT.**
> This is a claim about **mutable external state** (COORDINATION §7a): it can
> become false without any file changing, and the box has since power-cycled
> twice, which restarts the MCP client this diagnosis was about. **The next time
> `runtime-qa` needs to post, have it try — do not pre-emptively route it to the
> commit-a-verdict workaround on the strength of this block, and do not escalate
> it to the operator again without a fresh failed attempt.** Everything below is
> evidence about 2026-07-22, not about now.
>
> **Inbound works** (it receives mentions and reviews normally); **it cannot
> POST.** Unchanged across 4 watchdog ticks and 2 `/mcp` reconnect attempts.
> Server-side is healthy — every other seat posts normally.
>
> ⛔ **Do NOT relay its self-posts to close a provenance gate.** The gate exists
> precisely so an attestation is *not* Steward-sourced; a relayed self-post is
> a contradiction and banks the gap permanently.
>
> **The working path, and it is better than a relay:** QA **commits its verdict
> to its own branch** through the shared clone, and reviewers read it by object.
> Slice 5 closed this way — `53501ffe`,
> `docs/program/qa-triage/RT-SPLIT-slice5-runtime-qa-verdict.md`, binding
> `APPROVE` on `744bda14` / base `1f70a71b`. Precedent existed already
> (`ergo-qa @ cf791c7f`, `verify-qa @ 04efa001`). **A relay verifies the
> selection; a commit eliminates the selection.**

### ▶ Track 1 — RT-NATIVE-FNSPLIT (Runtime ring) — THE ONLY LIVE BUILD TRACK

Full state in **Standing state** above. One line here so this section is not a
second source of truth: **held at hard-stop #33 / `d43b6933`; research advisory
in flight; Architect rules after it lands; operator scaling gate still unmet.**

### ▶ Track 2 — `DOC-VALIDATION-BINDING` (Verify ring), kicked 2026-07-24

Operator-directed fill of the empty second slot. Kicked `evt_3fv5d1men6r9y`;
ring compact-verified at `8ebe370a`; branch
`wp/DOC-VALIDATION-BINDING-gate-token-binding` off current `origin/main`.

⚠ **Re-homed doc → verify, and the reason matters.** I first offered this to the
operator as *"scoped to `library/`, doc-only, Architect does not vote"* — **that
was wrong.** The mechanism is `crates/ken-cli/tests/library_documentation_gates.rs`,
a 106 KB Rust gate harness. Consequences: the doc ring cannot own it (its
concurrency licence *is* path-disjointness from `crates/`), and **the
COORDINATION §14a doc-only exemption does not apply — the Architect must vote.**
Corrected to the operator in the same breath as acting on it.

★ **Transferable:** *a WP's tracked `owner:` field is a routing claim, not a
grounded one.* This one said `doc` and had said so since it was filed. **Grep
where the mechanism actually lives before you route on the field.**

`PX8-F-CAP-41` Phase 2 was the natural Track 2 but is **blocked** behind
`NATIVE-HANDLE-CARRIER` → `RT-NATIVE-FNSPLIT`, so the PX8 spine cannot fill this
slot while #33 is open.

### ✅ CLOSED 2026-07-25 — the four operator directives ARE on `main`

> ⛔ **This block used to read "FOUR OPERATOR DIRECTIVES ARE LAW AND ARE **NOT
> ON `main`**" and asserted that the fleet was reviewing in series and the
> adversary was running without its scope fence. All four have since landed.
> The escalation it ended on — *"awaiting the operator's call"* — is
> **DISCHARGED. Do not re-raise it.**

Re-measured on `origin/main` 2026-07-25:

| directive | located |
|---|---|
| `COORDINATION §8a` — Architect/Librarian parallel, over disjoint domains | `agent/COORDINATION.md:455` |
| `COORDINATION §10⁻a` — adversary channel report-only | `agent/COORDINATION.md:629` |
| steward playbook §2d — separate judgment from action (OODA) | `agent/playbooks/federation/steward.md:1111` |
| steward playbook — contention has a LEDGER axis | `agent/playbooks/federation/steward.md:260` |

⛔ **The measurement itself nearly went wrong, and that is the durable part.**
My first probe grepped the literal `in PARALLEL over disjoint` and reported §8a
**missing**. The real heading is `PARALLEL, OVER DISJOINT` — a comma. One
false negative would have sent a settled matter back to the operator as an open
one. ⇒ **Probe with several short, lowercase, single-line fragments and require
them to agree**; never one multi-word phrase, which in an 80-column file is
odds-on to span a wrap or a punctuation mark you did not predict. The rule was
already written ~700 lines below this line, in *Tooling traps* — **and it did
not fire, because a rule that far from the work never does.**

⚠ **Still true and still load-bearing:** `steward/work` runs far ahead of
`origin/main` against §6a's *"at most the current unpublished tracker delta"* —
**mostly the squash-merge trap, so do not treat branch-ahead as unmerged.** The
route is §6a step 2: cut `wp/steward-<slug>` from **current** `origin/main`,
apply only the intended changes, and never publish `steward/work` itself.

### ▶ Doc track — IDLE

`DOC-W0`/`DOC-W1` closed, `DOC-W1-*` slices merged. `DOC-W2` is `draft` (a MAP,
framed only when Wave 1's exit condition is met). `DOC-VALIDATION-BINDING` is
`ready` and unassigned — the cheapest genuinely-parallel item on the board.

### ⏭ Releasable frontier (generated — `IMPLEMENTATION-PROGRESS.md`)

All deps met, nothing blocking a kickoff. **Ownership shown is the tracked
owner, not an assignment.**

| item | owner | note |
|---|---|---|
| `DOC-VALIDATION-BINDING` | doc | validation vocabulary claims a 1:1 gate binding nothing enforces |
| `STR-BIJ` | spec-enclave | String/`List Char` bijection over-claim; ⚠ carries a **ledger-axis** sequencing constraint |
| `KW-THEOREM` | spec | rename surface keyword `lemma` → `theorem` |
| `F1-37` | runtime | bignum `Int` soundness review for K3 trusted-base promotion — ⛔ *same ring as FNSPLIT, so not concurrent* |
| `MODELS-TIER` | steward | MODELS.md — Runtime seating is the norm, not an exception |
| `PUB-VERIFY` | steward | `scripted-pr-automerge.sh` exits 0 on a failed push |

★ **`PUB-VERIFY` is the one with teeth:** a publisher that exits 0 on a failed
push means a "published" report can be false. It is *my* tooling, so §10⁻
applies — it waits behind product unless it actually blocks a landing.

### ⏭ Durable rules kept from the closed RT-SPLIT / BUDGET-EFF tracks

The track narrative moved to [`2026/Jul/22.md`](2026/Jul/22.md). These outlived
it:

- ★ **Path-disjointness is re-derived BY THE RECEIVING RING at pickup and
  reported before implementation starts — never asserted by me in a kickoff.**
  I once asserted it *from a string literal* (a constructor **name** inside
  quotes) and the ring caught it.
- ★ **Classify by COMPILATION REACH, never by a `production`/`cfg(test)`
  binary.** A predicate satisfiable with `test = false` is production-reachable,
  and a feature name containing `test-support` does not change that. Every
  independent sweep of that file — mine, the adversary's, the implementer's —
  partitioned on the two-cell binary and was blind to the third cell **the same
  way**.
- ★ **A gate can be incapable of seeing its own subject.** A `default-off`
  feature's rustdoc dump reads identically whether a re-export is right, wrong,
  or **missing** — *incapable*, not weak. Local gates stay green while another
  crate is broken; only CI catches it. ⛔ Never `--workspace` locally to chase
  this — name the one cross-crate test instead.
- ⏭ **Queued small fix:** rustfmt drift at `crates/ken-runtime/src/store.rs:602`,
  **pre-existing on `origin/main`** (verified there, not introduced by RT-SPLIT).
  Touches `crates/` ⇒ normal ring gates; a standalone item, **never a rider**.
- ⏭ Sweep ~20 stale `/tmp/ken-*` worktrees. **ASK before touching any with
  tracked changes.**


### ⛔ PUBLISH DISCIPLINE — tightened 2026-07-22 after invalidating a Decision

I moved `origin/main` under RT-SPLIT slice 2's merge Decision **22 seconds**
after it opened, with a docs-only publish. Third occurrence. **`list_decisions`
run at the top of the work answers a question about the wrong moment**, and no
adjacent check catches a 22-second race. **So the trigger is earlier: hold
Steward publishes whenever a build ring holds a QA-APPROVED CANDIDATE, not
merely an open Decision.** Re-check immediately before the publisher call; if a
publish is urgent, **announce the window in-channel first.** Path disjointness
is irrelevant — §14 is about identity, not conflict.

## Operator rulings — 2026-07-21 ~12:45Z. SETTLED, do not reopen.

**On Linux ABI II** (`research/linux-abi-ii-work-program-proposal.md`):

- **No "ratification."** The charter is a **planning document, not a
  commitment**. Nobody outside the project is watching, and nothing depends
  on our timelines or stated intentions. **I had imported a governance ritual
  that does not apply — do not re-raise status-correction as a decision.**
- **Where there is a gap between what was anticipated and what was done, fill
  the gap first.** Hence `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation. CROSS-PLATFORM IS INDEFINITELY DEFERRED**
  (restated by the operator 2026-07-21 after I re-raised it). A very late
  feature, behind a long line of other work. Manifest v2 = family-scoped and
  generated, **not** cross-target.

  ⛔ **This ruling ALREADY ANSWERS any non-linux finding — do not route one
  back as a scoping question.** I did exactly that with "`ken-host` has never
  compiled on any non-linux target" (28 `cfg(not(target_os = "linux"))`
  fail-closed sites, never built, `abi_v1.rs:747`). **Under this ruling that
  finding is inert**: the lane is deferred, nothing builds it, and the defect
  cannot bite. It is dead code for a deferred target, not an open decision.
  Record such findings as *observations against a deferred lane* and stop —
  a settled ruling is a fixed input, never a question to re-ask.
- **L2-0: all desirable, nothing deferred.** All nine
  `RepresentedUnavailable` operations get promoted.
- **Timing, timelines, and budget are the OPERATOR'S domain.** They monitor
  and adjust. **Do not reason about schedule or cost.**
- ★ **My lane is token efficiency in terms of delivered work.** That is the
  axis to optimize and the one to report on.

**Still genuinely open (lower stakes, no re-ask):**

- **Provider concentration** — only `runtime-implementer` and `adversary` are
  on the Anthropic pool.

**CI-TRACKER-GATE is RESOLVED.** The operator granted the app `workflows:
write`; verified present in the installation's permission set, and a
workflow-bearing push was accepted. Close the issue once PR #804 lands.

> ★ **Diagnosing a `workflows`-permission rejection.** A freshly minted token
> is NOT enough — but neither is assuming staleness. `mint-gh-token.sh` with
> its final extraction changed from `['token']` to `['permissions']` prints
> the installation's **actual** grants. That converts "the push failed, why?"
> into a direct answer. Note the publisher only mints a new token when `gh`
> is not already authenticated, so a cached ~1h token keeps its old scopes;
> force a fresh mint before concluding anything.

## The completion program — written, NOT started · COVERAGE VERIFIED

`docs/program/10-linux-abi-completion.md` — **on `main`**. Four tracks:
**ABI-R** reconcile, **ABI-A** availability promotion, **ABI-M** manifest
(native-target only), **ABI-S** synchronous floor, plus **Track T** the
committed exit (PX10/PX11/PX12).

> **⚠ IDs RENAMED 2026-07-22 (operator).** Tracks R/A/M/S now carry an
> **`ABI-` prefix** — the bare `A3` collided with `issues/A3.md`
> (catalog-coverage walker) and `R1`-`R3` collided with the adversary's
> finding labels. **`PX9`-`PX12` keep their charter IDs.** `L` was rejected
> as a prefix: `L1`-`L7` are existing WPs.

> **★ COVERAGE — CLOSED 2026-07-25. This read `0 of 18 items have an issue`;
> it is now `18 of 18`.** Re-measured on `origin/main`: `ABI-A1/A2/A3`,
> `ABI-M1/M2`, `ABI-R1/R3`, `ABI-REVOKE`, `ABI-S1`…`ABI-S6`, and
> `PX9/PX10/PX11/PX12` all exist under `docs/program/issues/`. ⛔ **Do not
> re-file them.** *Filed* is not *framed*, though — a tracked node with no
> shovel-ready brief in `docs/program/wp/` is still not releasable, and that
> gap is the real remaining work (`ABI-R1` is framed; `ABI-S3` is queued).
>
> **AND the document had a hole:** the charter's **runtime revocation
> membrane** (`09` §5) is absent from it. `RevocationHandle { revoked: bool }`
> (`ken-elaborator/src/capabilities.rs:256`) is still the static contract —
> **its own doc comment says the runtime membrane is DEFERRED** — and there is
> **zero** revocation code in `ken-host`/`ken-runtime`/`ken-interp`/`catalog`.
> PX7's generation-checked handle table is a *different* property
> (use-after-close, not withdrawal of delegated authority). **L2 assumes it**
> (§8.1 gate 9). **AWAITING OPERATOR: fold into Track ABI-S / split as its own
> WP / accept as a known limitation.**

Verified against `main`, not taken from the advisory: 22 ops, 13
`NativeTested` / 9 `RepresentedUnavailable`, no process/socket/poll family in
any state, PX7 landed — and **PX9 (cross-domain `System.Error`) is chartered
but undelivered**, which the advisory's "good filesystem floor" phrasing
obscured. PX9 gates most of Track T.

**Not started.** Next step when the operator says go: decompose the tracks
into `docs/program/issues/` entries.

## ✅ TRACK Q IS COMPLETE — nothing outstanding (2026-07-21 ~21:55Z)

**Q-RESIDUE merged: `origin/main @ 64337192` (PR #818)**, from
`wp/Q-RESIDUE-test-rework @ 3f752451`. Verified on main **by content** (the
crate diff against the approved SHA is empty), not by the publisher's exit
code. Issue closed, all three retros in, adversary notified
(`evt_4g7qasxqdy5s8`). Q1 and Q2 merged earlier the same day.

**The fleet is idle and home-clean, and that is CORRECT** — the queue below
is unstarted pending the operator's direction. Do not kick any of it
without their go.

> ### ⚠ THE STALL THAT LOOKED LIKE QUIESCENCE — expect this again
>
> Track Q sat still for ~20 minutes not because it was done but because
> **the architect's composer had two Q-RESIDUE vote requests stacked as
> `[Pasted Content …]` and never submitted.** No `Working`, no turn, no
> vote, and nothing in the system surfaces it. A bare `Enter` released it.
>
> **"The fleet is quiescent" is an observation with at least two causes.**
> Single-threading makes idle rings normal, which is exactly what
> camouflages a wedged pane. On any quiet tick, `capture-pane -S -200` the
> architect specifically and look for stacked pastes — the composer strands
> there repeatedly. See `sweep-wedged-panes-misses-stacked-paste-form`.

> ### ⚠ AND A GREP THAT NEARLY MIS-REPORTED THE MERGE
>
> Verifying the merge, `grep -c 'examples.len() == 5'` on main returned
> **1** — reading as "the frozen count survived." It had not: the match was
> inside a **comment documenting its removal**; the live assertion is
> `!examples.is_empty()`. **Grepping a name instead of the mechanism** —
> the precise failure class this WP existed to remove, committed while
> verifying the fix for it. Read the context, never the count.

> ### ★★ THE RESULT WORTH KEEPING FROM THIS WP
>
> **AC-2's mutation proof caught a bad test before it shipped, on its first
> application.** A first draft of the settlement-ordering test
> *"only hand-sequenced the two helper functions instead of invoking the
> real `unsafe extern "C"` entrypoint, so it wouldn't have caught a real
> regression."* It would have sat **green through an actual defect** — a
> test exercising a **proxy** instead of the **mechanism**, exactly the
> class this WP existed to remove.
>
> The final discriminator is confirmed **three times independently**:
> implementer's authoring run, QA's independent re-execution on the branch,
> and the captured panic (`abi_v1.rs:1590`, `left: 0, right: 1`).
>
> **Require the mutation proof on every future test-rework WP.** It is the
> only step that distinguishes a grounded assertion from a green one.
>
> ⚠ **But do not over-read the three runs.** They are three confirmations
> of **one** discriminator, not three discriminators — all three flipped
> the same seam. A wrong seam produces exactly this agreement. Same shape
> as `differential-oracle-is-blind-to-a-shared-premise`. Raised with the
> adversary at merge; unresolved by design, it is theirs to attack.

> ### ⚠ I MIS-READ THIS WP'S DIFF AND TOLD THE FLEET
>
> I flagged `abi_v1.rs` (+71) as a **production-surface ABI change** in a
> test-rework WP. It is **entirely inside `mod tests`.** I inferred it from
> the file path and line count without opening the diff — same shape as the
> §1 CI misdiagnosis. I corrected it in the flag itself and told the ring to
> **vote on the branch, not on my summary.** They did, and grounded their
> review against real code (`abi_v1.rs:824-841` + the C call site).

## ✅ TRACK Q — DONE (2026-07-21). Only Q-RESIDUE remains, and it is an S.

**Q1 landed. Q2 complete: 428 triaged, 100% classified, six rings in
parallel.** Result: `docs/program/qa-triage/FINDINGS.md`.

| class | count | share |
|---|---:|---:|
| durable-invariant | 392 | **91.6%** |
| compat-vector | 19 | 4.4% |
| transition-sentinel | 7 | 1.6% |
| UNCLASSIFIABLE | 10 | 2.3% |

**Q3–Q7 folded by the operator into ONE S** — `docs/program/issues/Q-RESIDUE.md`
(status `ready`, owner `runtime`). **Not kicked; awaiting operator go.**

> ### ★★ Q4 AND Q7 WERE EMPTY — THE LESSON, NOT JUST THE RESULT
>
> 147 tests flagged for asserting an outcome without naming the variant:
> **every one sound.** All 27 wall-clock flags: **sound.** Both tracks had
> been sized from **scan hit counts**, and hit counts carried almost no
> signal about defects. Authorizing Q3–Q7 off the totals would have reworked
> ~300 correct tests.
>
> **⛔ Do not re-derive Q-RESIDUE's scope from `scripts/qa-risk-scan.py`.**
> It emits a **review queue, not a defect list**. The inventory in the issue
> is the whole of the work.

> ### ★★ THREE DEFECTS IN MY OWN INSTRUMENTS — ALL FOUND BY OTHERS
>
> 1. **The scanner fabricated a test.** An unanchored `#[test]` matched the
>    attribute *in prose* (`rt_parity_native.rs:3` is a doc comment). The
>    phantom swallowed 480 lines of helpers. **Foundation found it by
>    reading source.** `--self-test` passed throughout — it only checked
>    files that HAVE the patterns, never one that would INVENT a row. It now
>    has a negative arm (`NEVER_A_TEST`).
> 2. **"Two counts agreed" was an echo, not corroboration.** I cited the
>    scanner reproducing the documented 1909 as proof it wasn't dropping
>    tests. Both used the same naive match, so both counted prose mentions.
>    True total **1905**. A differential oracle is blind to a shared premise.
> 3. **The aggregator read the wrong file** — Ergo parsed 23 vs a reported
>    71 (glob matched QA's partial share, not the leader's assembly). Caught
>    only because the leader's own count disagreed.
>
> **Every one was caught by an INDEPENDENT source, none by my own checks —
> which were built on the same premises as the things they checked.**

> ### ★ TRANSPORT: a mention proves the EVENT exists, never that it was READ
>
> **Five of six Q2 kickoffs silently failed to deliver.** Repair: `tmux
> send-keys` a pointer to the `evt_…` (point at it, never restate it).
>
> **It reproduces INSIDE a ring.** Kernel stalled at 50/72 — its leader
> delegated, the implementer never got it, and the leader went idle
> believing it had handed off. **Silence and done look identical from here.**
>
> **⛔ Do NOT detect "working" by grepping a spinner word** — the verb is
> randomized ("Gitifying…", "Calculating…", "Crunched for…"). Key on the
> duration/token signature: `\([0-9]+m? ?[0-9]*s · [^)]*\)`. Grepping a
> fixed word read all six busy leaders as dead and nearly caused six
> duplicate re-rouses.

## ▶ THE DOC PROGRAM IS RUNNING — a SECOND, CONCURRENT track (2026-07-21 ~22:5xZ)

`origin/main @ 7610d2a1`. **`DOC-W0` is `active`** and released to a new
**doc team** (`evt_1m7j5qvvm2p2m`). This is the fleet's **one standing
exception to single-threading** (operator): the doc track runs **concurrently**
with build work, because doc WPs touch `library/` and `agent/`, not `crates/`.
**The exception is contention-free-ness, not priority** — a doc WP that would
touch a path a build WP holds defers.

| seat | tier | agent id |
|---|---|---|
| `doc-leader` | T2 Sonnet 5 | `agt_37w6sznc4nw00` |
| `doc-author` | T2 Sonnet 5 | `agt_37w6t02849400` |
| `librarian` | **T1 `gpt-5.6-sol`** | (existing) — the team's **QA** |

**★ Judgment is concentrated on the REVIEWING end, not the authoring end** —
inverted from every other team, deliberately. Documentation fails by being
*confidently wrong*, not badly written; a page whose citation does not carry
its claim reads perfectly. That is a grounding problem, which is where T1 pays.

Frame: `docs/program/12-documentation-program.md` (§0 team, §1 four **settled**
decisions — do not reopen). Overlays: `agent/teams/doc/{leader,implementer}.md`.
There is **no `doc-qa` seat** and no `agent/teams/doc/qa.md`.

**Seat provisioning, if it ever needs repeating:** `moot init` is NOT usable —
its only incremental option is `--force`, which rotates keys for **all**
already-adopted agents and would kill every live seat. Use the API directly:
`POST /api/agents` → `POST /api/registration-tickets/{id}/exchange` for the
plaintext key → write into `.moot/actors.json` (gitignored). PAT is at
`.mootup/credentials`. OAS: `local/refs/convo/docs/api/openapi.yaml`
(operator-sanctioned read; clean-room bars `local/refs/` for *writing Ken's
code*, which this is not).

## My queue, in order

0. **BUDGET-EFF** — Handoff Gate the Spec enclave (spec-leader, spec-author,
   conformance-validator). **Spec erratum FIRST**: `38` self-contradicts
   (`:404-405`/`:443-444` say *effective*, `:419-420`/`:438-440` say
   *requested*), so a code-first fix re-derives the defect from a broken
   citation. It is a **plumbing gap, not a formula fix** —
   `TransferCountV1::new(read, effective)` validates then **discards**
   `effective`, so neither reifier can compute the bound. Two closures with
   different blast radii ⇒ **Architect call**. Oracle
   ⛔ **AC-3 REWRITTEN — my earlier pin was VOID.** The R1 oracle's conclusion
   compares two values computed from its OWN constants (`RAW-count` vs
   `effective-count` = `4 == 0`) and **never reads a reifier field**, so it
   fails on ANY implementation and its failure at `e892777c` confirmed
   nothing. "Must pass unchanged" is **withdrawn**; the oracle must be
   **rewritten to observe the mechanism**. ⛔ **"Confirmed by execution" was
   also FALSE** — the defect rests on **source inspection** of the two
   reifiers. ★ **An adversary repro demonstrates a defect; a completion oracle
   defines correctness. Pinning does not convert one into the other — verify
   an oracle observes the mechanism BEFORE making it an AC.** Detail:
   `docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`
2. **SEAL-2** — re-anchor on current `main`; evidence
   `adversary/SEAL2-repros@70a603da`.
3. Then STR-BIJ → enclave (`ready`, S) · F1 → Architect (`ready`) · F3 ·
   A3 · F4 · RT-SPLIT (**L**, 22k-line `cranelift_backend.rs`).

> ### ⚠ WHERE THIS QUEUE CAME FROM, AND THE `#N` TRAP
>
> **These items are the gap between what was actually DELIVERED and what
> `research/linux-abi-ii-work-program-proposal.md` ASSUMES was already done**
> (operator, 2026-07-21). That proposal is the second Linux-ABI campaign; this
> series fills the hole in front of it. Read the proposal before sizing any of
> them — an item only makes sense against what it assumes exists.
>
> **`#37` / `#39` are indices in a PRE-RESTART STEWARD TASK LIST. They are NOT
> GitHub issue numbers.** Six issue files asserted them as `github:`
> references and the tracker propagated that into a dedicated GitHub column,
> where a task-list index read as a verified external reference — and
> `github: 38` pointed at whatever real issue #38 happens to be. **Corrected:
> `github: null` on both survivors**, with the provenance stated in-file.
>
> **`#38`/`#32`/`#24`/`#25` are DROPPED** — they carried nothing but a number
> (operator: *"no use to anyone"*). Do not resurrect them; there is nothing to
> resurrect. The old `GH-` filename prefix baked the wrong origin into the
> identifier itself — `identifiers-are-claim-artifacts`, in a schema field.

**Readiness is thin behind BUDGET-EFF — only two items are releasable.**
STR-BIJ (`ready`, enclave) and F1 (`ready`, runtime → Architect first).
The rest are `draft`; **A3 has no owner, no size, and no brief** and blocks
F4. ⚠ Verify "no brief" claims by *reading `docs/program/wp/`*, not by
globbing on the ID — the F3 brief is `F2F3-reducer-degrade.md` (it covers F2
and F3 together) and I mis-reported it as missing once already.

## In flight

**`DOC-W0` — ✅ MERGED `origin/main @ 6be9754b` (PR #830), 2026-07-22 ~01:43Z.**
Verified by content: all 8 blobs byte-identical to reviewed `d56abbb1`;
`revision_resolved` in `scripts/gen-doc-status.sh`; both shallow-clone
regressions present by name. The fleet's first `library/` tree. Retros were
being collected at time of writing — **check they are in before treating the
issue as closed.**

**Nine review rounds, six findings, and NOT ONE was a different kind of
mistake.** Every one was a **proxy standing in for the property**:

| # | proxy checked | property that mattered | found by |
|---|---|---|---|
| 1 | rejects a *fake* revision | **accepts a real one, in CI's env** | CI (red) |
| 2 | test clones `file://{repo_root}` | an **independent** history source | librarian |
| 3 | `cat-file` says object present | present **AND** ancestry provable | architect |
| 4 | symlink not *discovered* | symlink **rejected and reported** | architect |
| 5 | SHA reviewed + approved | SHA **on `origin`** (see below) | steward |
| 6 | process fix *agreed to* | seat **can perform it** (see below) | doc-author |

**5 and 6 were mine.** #3 held because the Architect built an isolated depth-1
probe rather than reasoning from source. **What finally stopped the recursion
was naming the predicate once** (`revision_resolved()` = object present AND
ancestry provable) and deriving self-heal, every deepen checkpoint, the
unshallow fallback, and all diagnostics from it — not any individual fix.

**⇒ Carry for DOC-W1 and every gate after it: when a gate depends on an
environment property (history depth, credentials, checkout topology), state
that precondition as a NAMED PREDICATE before writing the check.** A gate whose
precondition is unwritten gets discovered one CI-red at a time, each round
closing an instance and leaving the next layer live.

**New behavior worth watching:** `gen-doc-status.sh` now performs **network
fetches inside a test gate**. Fail-closed-on-unreachable-origin was verified,
but hermeticity under a flaky remote was not reasoned through. Flagged to the
adversary at merge.

**`SPEC-38-ERRATUM` — CLOSED.** Merged `origin/main @ e5a400c7` (PR #827),
retros in. Enclave carry: *keep semantic target / conformance oracle /
implementation mechanism as **separate scopes**; re-anchor with both
current-base and reviewed-subtree byte-identity checks.*
**This unblocks `BUDGET-EFF`, which stays PARKED pending operator go.** The
closure-mechanism call (reply-carries-effective vs. host-caps-the-request-
record) is an **Architect** decision routing *with* that release, not before.

**⛔ `BUDGET-EFF` AC-3 was UNSATISFIABLE and is corrected on `main`.** `count`
cancels on both sides, reducing it to `8 == 4` — no implementation could ever
have discharged it. **And it cannot be fixed in place:** `remaining` does not
occur in `ken-host/src/effect_v1.rs` where the oracle lives; it is built at
`ken-interp/src/eval.rs:4934-4935` and
`ken-runtime/src/cranelift_backend.rs:13081-13082`. The rewrite is **two new
tests** — budget it as plumbing. **The R1 defect itself still stands** on
source inspection; only its demonstration was broken. Branch aligned on
`origin/main`, clean, no orphaned polls. Build fleet idle and home-clean,
which is **correct**, not a stall.

**Also queued, not started: `Q-CLAIM-CLOSURE`** (`issues/Q-CLAIM-CLOSURE.md`,
`ready`, owner runtime) — the adversary's post-merge findings on Q-RESIDUE.
Advisory, **no live defects**. Its generator is worth reading before framing
any future rework WP: *the ACs took the TEST as the unit when the load-bearing
unit was the CLAIM*, so a rework could strengthen one claim, mutation-prove it,
and silently drop its siblings while fully satisfying the criteria. R1 (ABI
fact inventory has no independent anchor — both sides of the check come from
one generator) is the one to sequence first.

**CLOSED — not a scoping question.** `ken-host` has never compiled on any
non-linux target (`abi_v1.rs:747`, `?` on an `Option` in a `Result`-returning
fn; pre-existing since PX5 `049628f8`, adversary confirmed by extracting and
compiling it). 28 `cfg(not(target_os = "linux"))` fail-closed sites, never
built. **Cross-platform is indefinitely deferred (operator) — so this is dead
code for a deferred lane and cannot bite.** Recorded as an observation; no
action, no decision pending. See the L2-1 ruling above.

Recently landed and verified by content: **#827** (SPEC-38-ERRATUM →
`e5a400c7`), **#828** (AC-3 correction + ABI gap status → `9fb90aab`),
**#818** (Q-RESIDUE), **#819**
(Track Q closeout), **#820** (doc program frame), **#821** (doc team),
**#822** (librarian T1 + DOC-W0 release).

## ⚠ FLEET IS MID-RESEAT — leader / implementer / QA seats → Sonnet 5

The operator is reseating the build-team seats (**not** spec-leader). Seats
were cycling as of ~19:00Z.

> ### ★ NEW TRAP — a reseated agent re-posts an ALREADY-CLOSED retro
>
> `kernel-leader` came up on a fresh seat and posted a §10 retro for
> **KTR-1, which closed 2026-07-14 with retros already in** (`65d68cfc`,
> PR #675). Not an error on its part — it reported what its context showed.
> But **counting such a re-post inflates the promotion ladder.**
>
> **Verify every post-reseat retro against the RECORDED state, never the
> report:** `docs/program/issues/<ID>.md` frontmatter (`status: closed`), or
> the diary for WPs predating the issue system. Expect more of these as the
> remaining seats come up.
>
> Contrast RT-PARITY, where the leader's near-identical announcement *was*
> actionable: its retros were genuinely in and only the frontmatter lagged at
> `merged`. **The two look the same from the outside — only the recorded
> state tells them apart.**

**Do not kick any WP until the reseat is complete and the operator releases
one.** Delivering into a seat that is about to restart is the transport
failure the Handoff Gate exists to prevent.

## Programs written, NOT started

- `docs/program/10-linux-abi-completion.md` — the work Linux ABI II presumes.
  Tracks R/A/M/S/T; **PX9 gates most of Track T**.
- `docs/program/11-test-suite-and-ci-remediation.md` — **Track C is DONE.
  CI went 47 min -> ~8 min, landed `8b09fb95`.** Skip the three slow native
  binaries + nextest + shard x4 + `opt-level = 2` on deps + rust-cache
  removed. **Do not shard further** — per-shard parallelism already fell
  3.96x -> 2.5x, so 8 shards would buy ~90s for double the compute. The next
  real reduction must come from `CI-SKIPPED-NATIVE-TESTS` getting faster.
  Details and full scorecard in §1a/§1b. **Track Q (the QA-advisory sweep)
  is untouched and still the actual point of the program.**

  **Skipped-test restoration — measured, not guessed** (§1c/§1d,
  `CI-SKIPPED-NATIVE-TESTS`). Any job finishing under the ~471s critical
  shard costs **zero** wall clock, and that headroom is the budget:
  - `px8f_write_partition` ✅ restored, own `native-slow` job. **C6 gave it
    −22.7%** (309s→239s) vs 8.4% suite-wide — C6's benefit is concentrated
    in cranelift-heavy code, exactly as predicted.
  - `px8f_buffer_native` ✅ restored, own `native-buffer` job. **Measured on
    run 29850680007: Test step 149s, job 224s** — well under the 482s
    critical shard, so it costs zero wall clock, as predicted.
  - `rt_parity_native` — **a ONE-TEST problem.** Parallelizes fine (7 tests,
    266.7s wall / 470.6s CPU), but
    `fs_write_at_malformed_offset_narrows_to_invalid_offset` takes **221.4s**
    vs **42.2s** for its near-identical sibling. Fix that one and the binary
    lands ~90s. **Do not just re-enable it** — today it fits by ~1s, which
    is noise.

  ✅ **Dedicated jobs are now scoped** (`-p <crate> --test <name>`), not
  `--workspace` — that was compiling all 200 test binaries to run one. Now
  **confirmed by isolation**: `px8f_write_partition`'s Test step went
  **241s → 129s (−112s)** across runs 29850405231 → 29850680007 with
  scoping as the only variable, against a ~124s estimate. The `Build` step
  stays `--workspace`: it is only ~65s and it is what proves the workspace
  compiles under `--locked`.

  **Only `rt_parity_native` is still skipped, and the target is ONE test.**

  > ⚠ **The `native-buffer` number was right for the wrong reason — do not
  > cite it as evidence C6 generalizes.** I projected ~240s for that job and
  > measured 224s. But that projection was of an *unscoped* job, and the
  > scoping change (−112s) landed in the **same PR**. Unscoped, the job
  > would have been ~336s — the projection was ~40% high and was rescued by
  > an unrelated bundled change. **Two changes in one PR made a wrong
  > prediction look confirmed.** C6's −22.7% on `px8f_write_partition`
  > remains the only clean measurement of C6 on cranelift-heavy code; it is
  > a single data point, not an established scaling law. Sibling of
  > `green-vs-green does not confirm a fix` — a number matching its forecast
  > is not evidence the reasoning behind the forecast was sound.

  > ★ **I had the CI diagnosis backwards, and the operator caught it.** I
  > claimed a cold dependency build dominated the wall clock. Measured:
  > **build 47s, test execution 44m14s — 95% of the run.** The error was
  > reasoning from *"there is no cache"* (true) to *"the build is the cost"*
  > (never checked) without opening a single run log. The logs were
  > available the entire time. **An explanation for why something COULD be
  > slow is not evidence that it IS.**

  Measured distribution: `cargo test` walks its **200 test binaries strictly
  in series**, and **three of them — nine tests — are 56.5% of the whole
  run** (`rt_parity_native` 14m41s, `px8f_buffer_native` 5m10s in a *single*
  test, `px8f_write_partition` 5m09s in a *single* test). The bottom 150
  binaries total **48 seconds**. All three fat binaries do a real native
  codegen-and-link per test case.

  > **Operator ruling 2026-07-21: remove `Swatinem/rust-cache` as part of
  > C6** (tracked as C8). No measurable benefit, and it is a third-party
  > dependency with access to the build — a supply-chain surface taken on
  > for nothing. **A dependency must earn its place.** My counter-argument
  > (it absorbs C6's rebuild) was weak: it defended a dependency on an
  > untested hypothesis and priced only time, never trust.
  >
  > ⚠ **C6 and C8 are in latent tension** — C6 can only *increase*
  > dependency compile time, and C8 makes every run pay it. **The C6 run
  > must report the Build step**, not just test numbers. Thresholds are
  > pre-committed in §3b; if the build blows up, **return it to the operator
  > with the number** rather than quietly reinstating the cache.
  >
  > ⚠ **C2 added `taiki-e/install-action@nextest`, unpinned — same class of
  > exposure.** It is defensible because nextest earns it (it fixes the
  > actual problem) where the cache did not, but **pin it to a commit SHA**.

  **Next steps are C2 → C6 → C7, re-measuring between each:** nextest (one
  global pool replaces the serial walk), `[profile.dev.package."*"]
  opt-level = 2` (cranelift runs its codegen unoptimized — **hypothesis,
  test it in CI, do not merge on plausibility**), and splitting the two
  1-test binaries (unsubdividable, so they become the critical path the
  moment C2 lands). C1 landed and bought ~5s — **do not report it as a
  throughput win**.

  ⚠ C7 and Q7 are one edit from two sides — splitting the native binaries
  for parallelism, and giving temp dirs per-test ownership so that
  parallelism is safe. Do them together or expect a flake that reads as
  "nextest broke the suite."

  **`scripts/ci-test-timings.sh <run-id>`** regenerates the per-binary table
  from any run's log. Granularity is the binary; per-`#[test]` needs C2.

Next step for either program when the operator says go: decompose its tracks
into `docs/program/issues/` entries.

## Tooling traps — distrust a clean negative

> ### ⛔⛔ `git maintenance` CAN STARVE THE WHOLE BOX — config now guards it
>
> **2026-07-21: `git pack-objects` consumed all 8 cores; load hit 14.** Root
> cause is structural and will recur if the config is ever reset:
>
> ```
> maintenance.lock  -> .git/worktrees/<name>/maintenance.lock   PER-WORKTREE
> gc.pid            -> .git/gc.pid                              shared
> objects/          -> .git/objects                             shared
> ```
>
> **`git maintenance` locks PER-WORKTREE but repacks the SHARED object
> store.** A run in one worktree is invisible to a run in another, so the
> concurrency ceiling is **the worktree count — 30**, each defaulting to all 8
> cores. The legacy `git gc --auto` path did *not* have this hole (`gc.pid` is
> common, so the second bails). `git maintenance` lost that protection.
>
> **Guard now set repo-locally** (covers all worktrees via the shared store):
> `maintenance.auto=false`, `gc.auto=0`, `pack.threads=2`,
> `pack.windowMemory=256m`. **`maintenance.auto=false` is the load-bearing
> one** — capping threads alone still allows 30 × 2 contending.
>
> **Consequence you are now carrying:** loose objects accumulate forever.
> A deliberate `git gc` is needed during a genuinely quiet window (fleet
> idle, no WP in flight). **Never run it while a team is working.**
>
> ⚠ **Trigger to avoid:** a `git add -A` run from `/workspaces/ken` (the MAIN
> worktree) sweeps untracked `.cache/`, `.targets/`, `.tmp-*` into the object
> store, blowing past the loose-object threshold and firing maintenance from
> every worktree at once. **A `cd` chained before a broad `git add` silently
> changes which repo it applies to.** Those blobs are still present as
> unreachable objects pending a prune.

> ### ⛔ A PANE SNAPSHOT IS NOT AGENT STATE — three variants seen in ONE day
>
> | symptom | actual state | repair |
> |---|---|---|
> | stacked `[Pasted Content …]`, no `Working` | alive, **never submitted** | bare `Enter` |
> | pane **entirely blank** (even `-S -200`) | alive, blocked on a **consent modal** rendered at the buffer's START | capture from `-S -` and `grep -v '^\s*$'`, then `Enter` |
> | empty prompt, looks idle | **actively working** — narrow `tail` caught a gap between renders | capture WIDE before repairing |
>
> **★ The third one recurred TWICE on 2026-07-22** — once reading `doc-author`
> as never-engaged (it had already finished), once rousing it *while it was
> mid-fix* with `is_symlink_escape` already in its diff. **Interrupting a
> working agent is forbidden by the playbook and I did it anyway**, because a
> `tail -4` showed a bare `❯`. The spinner renders **above** the prompt. A
> `WORKING` check must grep the whole capture for `esc to interrupt` — never
> judge from the last few lines.
>
> The third one bit me *while running the check designed to catch the first*.
> Had I trusted it I would have stacked a duplicate kickoff on a working seat.
> **Always `capture-pane -S -` piped through `grep -v '^[[:space:]]*$'` before
> concluding anything about a seat.** A new seat's first launch is exactly when
> nobody is watching — the consent modal for
> `--dangerously-load-development-channels` blocks silently and
> indefinitely.

- ⛔⛔ **AFTER EVERY MERGE, RE-BASE `steward/work` ONTO `main` BEFORE THE
  NEXT COMMIT.** Cost three publish cycles on 2026-07-21 — the same trap
  each time. `main` merges are **squash** merges, so `steward/work` never
  contains the resulting commit: its merge base stays at the *previous*
  main, and GitHub's three-way merge then conflicts on any file both sides
  touched, even when the content is compatible.

  > ★ **`git diff origin/main..HEAD` will NOT warn you.** A two-dot diff
  > shows the **net difference**; a merge asks a **different question** —
  > what happens when both sides' changes are replayed from a shared base.
  > A clean two-dot diff next to a conflicting merge is not a contradiction.
  > **The check that actually predicts a conflict is**
  > `git merge-base --is-ancestor origin/main HEAD` — if that fails, rebase
  > *before* committing anything further.

  Recipe (content-preserving, verified three times):
  `git tag -f preserved/<sha> HEAD` → `git reset --hard origin/main` →
  `git checkout <old-sha> -- <changed files>` → regenerate the dashboard →
  commit. Then confirm with `git diff <old-sha> HEAD`: the **only** expected
  delta is `IMPLEMENTATION-PROGRESS.md`'s timestamp line.
  ⚠ Do **not** verify with a bare `git diff` after `git checkout -- <path>`
  — that stages the files, so unstaged `git diff` reads empty and looks like
  a mismatch. Compare **commit to commit**.

- ⛔ **`scripts/scripted-pr-automerge.sh` exits 0 on failure** (4 times on
  2026-07-21). Its **first attempt after any merge always fails** with
  `stale info`, because the merge deletes the origin head branch and stales
  the local ref. **Always `git fetch origin --prune` before publishing.**
  ⚠ Its `--description-file` must exist **before** the call — a heredoc
  inside the same `&&` chain does not reliably land, and the script reports
  `description file not found` and exits.
  Redirect its output to a file — a `| tail` pipe block-buffers it to 0
  bytes. Afterwards it sleeps ~40 min polling a PR that may already have
  merged; verify `origin/main` by content and kill the orphan. Tracked as
  issue `PUB-VERIFY`.
- **A piped exit code belongs to the last command in the pipeline**, not to
  `git`. Verify `HEAD` moved.
- **Branch-ahead ⇏ unmerged** (squash-merge trap). Verify by content.
- **Concurrent subagents in one worktree share `.git`** — path-disjoint is
  **not** commit-disjoint. Two raced the index on 2026-07-21. Use
  `isolation: "worktree"`.
- **`convo` posting can fail while the channel stays up.** An absent post is
  not a stalled agent — check the pane.
- **A literal-string grep is a proxy, not the property.** "Four tools" had
  been upgraded to "Five tools"; the grep read as content loss and nearly
  discarded good work. Grep the theme, then read the hit.
- Liveness: `tmux capture-pane -p -S -300 -t moot-<seat>` — **`-S` must be
  negative**; a positive value returns ~1 line and reads every seat as dead.
- `Press up to edit queued messages` = **busy + queued. Do not resend.**

## ⛔ "COMMITTED" IS NOT "REACHABLE" — publish, then verify ON MAIN

**2026-07-22, caught by the adversary, not by me.** I corrected `BUDGET-EFF`'s
AC-3, announced *"all four folds are in"*, and it was true **about
`steward/work` only**. `COORDINATION §15` sends rings to **`main`**, so for
the whole window a ring picking up the WP would have read the **unsatisfiable**
AC-3. Five commits, zero publishes.

**The rule, mechanically:** a Steward doc edit is not done at `git commit`. It
is done when `git grep '<plain phrase>' origin/main -- <file>` returns the new
text. Route via `§6a` (corpus branch off *current* `origin/main` → publisher
path → **verify by content**).

Two amplifiers, both real:
- **The publisher prints `merge command succeeded` on a failed push** — that is
  the open `PUB-VERIFY` issue. Its exit code is worthless; only content on
  `main` counts.
- **`git grep` is case-sensitive and false-negatives on a phrase spanning a
  line break.** Three greps false-negatived on 2026-07-22 alone. Grep a short,
  lowercase, single-line fragment — never a sentence, never across `**bold**`.

Same family as the whole week: **verify the mechanism, not a proxy.**
"Committed" is the proxy; "on `main`" is the mechanism.

## ⛔ A `git_request` SHA IS *REVIEWED*, NOT *PUBLISHED* — `ls-remote` FIRST

**2026-07-22, DOC-W0, caught with ~30 seconds to spare.** `doc-leader` sent a
`git_request` for `d56abbb1`. **`origin` was still at `8f14ff83` — the exact
SHA that had already failed CI on PR #830.** Four folds lived only in
doc-author's worktree. Running the publisher as requested would have rebuilt
the known-red commit and looked like a *regression of an already-fixed bug*.

**Why nothing upstream catches it:** the agent worktrees **share one object
store** (`/workspaces/ken/.git`), so an unpushed SHA resolves **perfectly** for
`git log`, `git diff`, `merge-base --is-ancestor`, `git grep`, and a full
detached test run. **Librarian exact-SHA QA and Architect exact-identity review
both passed on a commit that was not on `origin`.** `git ls-remote` is the
*only* check that separates reviewed from published.

**What actually exposed it:** a **number disagreeing with a report** — my scope
diff printed **900** lines for the gate test where doc-author reported ~1200.
Cross-checking a reported magnitude against my own measurement caught it; no
identity check I ran did.

**⇒ HARD PRECONDITION of every publish, before `scripts/scripted-pr-automerge.sh`:**

```sh
git ls-remote origin refs/heads/<branch>   # MUST equal the requested SHA
```

If it does not match, **push it yourself** — mint via
`.devcontainer/mint-gh-token.sh`, then push to a URL **derived from `origin`**
(`git remote get-url origin | sed "s|https://|https://x-access-token:${T}@|"`) —
⛔ **never a hardcoded org**, which the 2026-07-25 `ken-topos` → `swe-toolkit`
rename would have silently broken behind GitHub's redirect —
and **re-verify with `ls-remote` after**. Often a clean fast-forward (the stale
head is an ancestor) — check before assuming a force is needed.

**⛔ DO NOT delegate this push to the authoring seat. NO BUILD SEAT HAS GITHUB
CREDENTIALS** — only the scripted publisher and the Steward. I issued exactly
that carry, doc-author accepted it, then *tested* it and hit `could not read
Username for 'https://github.com'`. **A process rule assigned to a seat that
cannot execute it is worse than the gap it closes** — everyone believes it is
handled. **Verify a seat CAN do a thing before making it their duty.** QA seats
may carry "candidate SHA present on `origin`" as a **detection** item; the
remedy always routes to the Steward.

## Standing discipline

A success signal says a thing **ran**, never that it did what you meant —
**verify by content**. `git rev-parse --abbrev-ref HEAD` must read
`steward/work` before any write. Local builds are **targeted only** via
`scripts/ken-cargo -p <crate>` — **never `--workspace`** (it OOMs the box).
Workspace-green means green in **CI**.
