# LET-2b — Teach the `let` binding group

**Owner:** Team Language · **Size:** M · **Risk:** ★★
**Stream/gate:** WS-L / surface pedagogy · **Rides:** LET-4 (`26527c5a`)
**Unblocks:** LET-3 (catalog `let` pilot — Foundation)

**Status:** RELEASED · **AMENDED MID-FLIGHT (operator, 2026-07-14): D6 added,
AC4 REVERSED — the `FRAME_LINE_COUNTS` oracle is now DELETED, not preserved.**

> **Chain:** LET-4 made the multi-binding **group** the canonical `let`. It
> migrated the two `catalog/guide/*.ken.md` **fences** — because it *had to*, or
> `ken fmt --check` would have gone red on the frozen corpus — and **touched none
> of their prose.** This WP closes that gap.

---

## 0 · The defect, in one sentence

**The guides now SHOW group-form code that their own prose never explains.**

`catalog/guide/surface-reference.ken.md:476-486` renders a binding group.
Twenty-five lines above it, `:451-454` — the normative sentence that *introduces
the construct* — still says:

> *"`let name = rhs in body` gives an intermediate expression a local name…"*

**The `;` separator is never named. The group form is never named.** The section
is still titled *"Local `let`: naming an intermediate"* (`:25`, `:449`) —
singular.

**This is not a formatting problem. It is a teaching problem, and it is the one
thing LET-4 could not fix mechanically.** *A reader who learns `let` from this
guide learns a spelling the formatter will silently rewrite underneath them.*

---

## 1 · Fixed inputs — the spec is SETTLED. Do not reopen it.

LET-4 landed the normative text. **Cite it; do not relitigate it.**

- **`spec/30-surface/32-grammar.md:200-201`** — the production:
  ```
  let_expr    ::= "let" let_binding (";" let_binding)* "in" expr
  let_binding ::= ident (":" type)? "=" expr
  ```
- **`:209-213`** — one or more bindings; a `;` occurs **only between** two
  bindings; **a trailing `;` before `in` and a comma in place of a `;` are syntax
  errors.** ★ *"A one-binding `let` remains the same production with zero
  repetitions."*
- **`:215-221`** — bindings are **sequential and non-recursive**; duplicate names
  are rejected.
- **`spec/30-surface/31-lexical.md:228-231`** — the load-bearing sentence:
  > *"A directly nested sequential chain of **at least two** local lets has one
  > canonical surface form: the formatter **coalesces the maximal chain** into a
  > binding group. It does not retain the repeated `in let` spelling."*

---

## 2 · ⛔⛔ THE TRAP — AND IT IS AN OVER-APPLICATION TRAP, NOT AN OMISSION

**You are about to learn "the group is canonical." You will then want to convert
every `let` you see into a group. THAT IS WRONG, AND IT IS THE WAY THIS WP
FAILS.**

**A one-binding `let` is ALREADY CANONICAL.** The spec says so in as many words
(`32-grammar.md:212-213`): *the same production with zero repetitions.* And the
formatter only coalesces chains of **≥ 2** (`31-lexical.md:228`).

**⇒ Every one of these is CORRECT and MUST NOT BE TOUCHED:**

| Site | Why it stays |
|---|---|
| `surface-reference.ken.md:466, :468, :495, :504, :514, :522` | single-binding lets — canonical |
| `agent/playbooks/tools/write-ken.md:80` | single-binding — canonical |
| `examples/rosetta/gcd/gcd.ken:99` | the **only** `let` in all of `examples/` — canonical |

> **A "group of one" is not the canonical form of one binding. It is noise.**
> **The group is the canonical form of a CHAIN.** *If your diff converts a
> single-binding `let` into a group, you have inverted this WP.*

**There are ZERO surviving `let … in let …` chains in any teaching artifact** —
LET-4 already migrated the only two (`proof-techniques.ken.md:353-375`, `:396-407`;
`surface-reference.ken.md:476-486`). **Your job is the PROSE, not the code.**

### ★★ TRAP 2 — SUPERSEDED BY OPERATOR RULING. **The oracle is being DELETED.**

> **⚠ AMENDMENT (Pat, 2026-07-14, mid-flight).** This section previously said
> *"⛔ DO NOT EDIT `FRAME_LINE_COUNTS` — if Gate C trips, STOP and route."*
> **That guardrail is WITHDRAWN and REVERSED. Pat ruled: remove the checks.**
> **See D6. The correct action is now to DELETE the table, not tiptoe around it.**

**Why it goes** (and the fleet already half-knew this — **ten existing WP frames
carry a standing *"Add NO row to `FRAME_LINE_COUNTS`, it is a discharged
historical baseline"*** — everyone has been routing around a gate nobody
believed in):

`FRAME_LINE_COUNTS` records each corpus file's line count **as it was BEFORE the
capstone-C reformat**, and asserts the reformat did not pathologically expand it
(≤4.5× per file, ≤3× corpus-wide). **The file says so itself at `:11`:
*"Historical, discharged capstone-C migration baseline."***

**That migration is over.** What remains is:

- a **4.5× growth cap anchored to a finished migration**, which now accidentally
  caps *content authoring* — a job it never had — with **66% slack** (the closest
  file to firing is `hello-world.ken` at **15 lines against a 45-line cap**);
- **`assert_eq!(frame_total, 32_594)`**, which is **just the sum of the 39 rows**
  — *a checksum on the TABLE, asserting nothing whatsoever about Ken.* Its only
  function is to fail if someone edits the table.

**And the sibling test 50 lines below it — `canonical_frozen_corpus_is_a_39_file_fixed_point()`
(`:68`) — re-formats every LIVE file and asserts byte-identity.** *That is a
current-anchored fixed point, and it is doing all the real work.* **A discharged
one-shot proof left wired as a live gate is how you get a test everyone routes
around — which is exactly what happened.**

---

## 3 · Deliverables

**D1 · `catalog/guide/surface-reference.ken.md` §8 — the primary artifact.**
Rewrite the §8 prose (`:449-454`, and the section title at `:25`/`:449`) to teach
the construct as it actually is: **one or more bindings, `;`-separated**;
sequential and non-recursive; duplicates rejected; **the formatter coalesces a
maximal chain of ≥2 into a group**; a one-binding `let` is the same production.
The migrated fence at `:476-486` is your worked example — **narrate the code that
is already there.**

**D2 · `catalog/guide/proof-techniques.ken.md` §6.** LET-4 rewrote the two proof
fences into groups and left the surrounding prose (`:331-341`, `:378-382`,
`:410-417`) narrating a nested chain that **is no longer on the page.** Re-narrate
it to match the code the reader can actually see.

**D3 · `agent/playbooks/tools/write-ken.md:57-98`.** The `let` section never
mentions the group. **Keep the single-binding example at `:80-85` — it is
canonical** — and **add** a group example. Refresh the pointer at `:92-94`.

**D4 · `docs/program/07-catalog-style-guide.md` §6.1 (`:182-216`).** Pure prose,
no examples. Give catalog authors the rule: **≥2 sequential bindings ⇒ write the
group.**

**D5 · `agent/teams/foundation/{leader,implementer,qa}.md` — ★ LOAD-BEARING.**
These overlays are silent on the group. **Foundation authors LET-3, the catalog
`let` pilot, next.** *If they author it in the single-binding spelling, the
formatter canonicalizes every one into a group and we churn the catalog twice —
which is the exact reason LET-3 was blocked behind LET-4 in the first place.*
**One clear line each is enough. Do not skip this because it is the smallest
item; it is the one with a consumer waiting.**

**D6 · ★ DELETE EVERY CORPUS-INVENTORY ORACLE.**
*(Operator ruling, mid-flight, **twice**. This is the one code change in an
otherwise prose-only WP — and it is a DELETION.)*

> ### The operator's principle — this is the whole of D6, and it decides every case
>
> > *"We have full history in git. These counts are unnecessary. **We should be
> > free to split, concat, delete, etc. catalog files without having to make a
> > coordinated change to a test.**"*
>
> **⇒ THE TEST: does restructuring the corpus force a coordinated edit to this
> assertion?** **If YES, it dies.** *No exceptions, and no "but I'll just raise
> the number" — **raising the number IS the coordinated change**, and my first
> amendment made exactly that mistake.*

### Delete these FIVE — they are the complete set

**In `crates/ken-elaborator/tests/kenfmt_c_capstone.rs`:**
1. the `FRAME_LINE_COUNTS` const (`:14-66`) **+ its comment block (`:11-13`)**;
2. the whole test `canonical_reformat_has_no_pathological_line_expansion()`
   (`:117-173`) — the table's only consumer;
3. the floors in `canonical_frozen_corpus_is_a_39_file_fixed_point()` (`:80-89`)
   — `literate.len() >= 14`, `plain.len() >= 17`.

**In `crates/ken-cli/tests/ken_fmt.rs`:**
4. the floors in `strict_frozen_corpus_gate_is_green()` (`:98`, `:103`) —
   `catalog.len() >= 14`, `rosetta.len() >= 16`.
5. **★ `assert_eq!(runnable.len(), 19, …)` (`:158`)** in
   `every_checked_runnable_root_declares_its_fs_authority()`. *The Steward did not
   flag this one; it fell out of the sweep. It is an **exact-equality** inventory
   oracle — **the most coordination-forcing of the five.** Add one runnable root
   and it demands a test edit. **It dies by the same principle.***

### ⛔ REPLACE EACH WITH A NON-VACUITY GUARD — `assert!(!xs.is_empty())`

**A count floor and a non-vacuity guard are NOT the same thing, and only one of
them costs coordination.**

| | split / concat / delete a file | broken glob (zero files) |
|---|---|---|
| `len() >= 14` | **FAILS — coordinated edit** ⛔ | fails |
| `assert_eq!(len(), 19)` | **FAILS — coordinated edit** ⛔ | fails |
| **`assert!(!is_empty())`** | **passes** ✅ | **FAILS** ✅ |

> **`!is_empty()` satisfies the operator's principle EXACTLY — it can never force
> a coordinated change under any legitimate restructure — while still refusing to
> let the loop go vacuous.**
>
> **★ Why this is not me smuggling the count back in:** every one of these tests
> is a `for path in <glob> { assert!(…) }`. **If the glob ever resolves to zero
> files, the loop body never runs and the test passes — GREEN, having checked
> nothing.** *That is the exact failure this whole WP exists to delete: a gate
> that cannot fail.* **Replacing a count with `> 0` removes the ledger and keeps
> the floor under the vacuum.** *If the operator wants even that gone, say so and
> it goes — but it is not what the principle asks for.*

### Keep, UNTOUCHED

- **`canonical_frozen_corpus_is_a_39_file_fixed_point()`'s actual body** — format
  every **live** file, assert **byte-identity**. *This is the real canonicity
  gate, it globs whatever is there, and **it needs no coordination when you split,
  concat, or delete** — so the principle does not touch it.* **Only its two floors
  go.** *(Its NAME says "39_file" and is now a lie — **rename it**
  `canonical_live_corpus_is_a_fixed_point`.)*
- `balanced_corpus_rejects_the_known_over_width_splay_shape()`, the per-file
  FS-authority assertion inside `every_checked_runnable_root_…`, and every helper.
- **Every `.len()` assertion about SEMANTIC content** (`result.guarantees.len() == 1`,
  `ind.constructors.len() == 9`, …). **Those are not corpus inventories. Do not
  touch them.** *Only the assertions that count FILES IN A GLOB die.*

**Accepted loss, stated plainly:** we give up corpus-shrink detection entirely.
**A file deletion is a line in `git log` and a line in the diff. That is where it
belongs — not in a test that must be renegotiated every time the catalog moves.**

---

## 4 · Acceptance criteria

**AC1 — the prose teaches the construct.** Both `.ken.md` guides name the **`;`
separator**, the **group**, **sequential non-recursive scope**, and the **≥2-chain
maximal-coalescing** rule, citing `31-lexical.md:228-231` / `32-grammar.md:200-221`.

**AC2 — ★ ZERO Ken-fence changes.** LET-4 already migrated every fence. **Your
diff should touch prose only.** `ken fmt --check` is green on both guides **before
and after** — run it and show it. *If you believe a fence must change, STOP and
route: that means LET-4 left something non-canonical and it is a finding.*

**AC3 — ⛔ NO single-binding `let` becomes a group.** Enumerate **every** `let`
your diff touches and show each was a **≥2 chain**. *If that list is empty, that
is the correct and expected answer.*

**AC4 — ★ REVERSED TWICE BY OPERATOR RULING. Every corpus-inventory oracle is
DELETED.** *(The ORIGINAL AC4 said `git diff` must NOT include
`kenfmt_c_capstone.rs`. **It now MUST** — and `ken_fmt.rs` too. My first
amendment then said "raise the floors to 38/17"; **that was wrong — raising a
number IS the coordinated change the operator is eliminating.**)*

- **ZERO corpus-inventory assertions remain.** These greps over `crates/` return
  **no hits**: `FRAME_LINE_COUNTS` · `32_594` · `pathological` ·
  `literate.len() >=` · `plain.len() >=` · `catalog.len() >=` ·
  `rosetta.len() >=` · `runnable.len(), 19`.
- **★ THE ACCEPTANCE PROOF — it is a RESTRUCTURE, not a test run.** In a
  **disposable** tree: **delete one `catalog/**/*.ken.md`, and concatenate two
  others into one file.** Then run all three named suites. **They must be GREEN
  with no test edit whatsoever.** *That is the property the operator bought;
  demonstrate it, do not assert it.* **Report the restructure and the runs.**
- **★ AND PROVE THE NON-VACUITY GUARD STILL BITES:** point a glob at an empty
  directory (or otherwise make one resolve to zero files) and show the test
  **FAILS** on `!is_empty()`. *We are deleting gates that could not fail. Do not
  ship a replacement that cannot fail.* **Report it.**
- **Every other test in both files still passes, unmodified.** The `39_file`
  fixed-point test is **renamed**, and its **body is untouched**.

**AC5 — the Foundation overlays teach the group** (D5), so LET-3 is authored
canonically the first time.

**AC6 — targeted gates only** (`COORDINATION §12`; **never `--workspace`
locally**):
- `scripts/ken-cargo test -p ken-cli --test ken_fmt` (Gate A: strict frozen corpus)
- `scripts/ken-cargo test -p ken-elaborator --test kenfmt_c_capstone` (Gates B/C)
- `scripts/ken-cargo test -p ken-elaborator --test kenfmt_b1_lossless` (Gate D)

**AC7 — ★ FULL CI ON PUBLISH. NEVER `--doc-only`.** `catalog/**/*.ken.md` is
**CODE** — `ken_fmt.rs:89` formats it. *LET-2 landed two guide strands behind
`--doc-only`, went red on the frozen-corpus gate, and blocked the publisher for
every WP behind it. That does not happen twice.*

---

## 5 · Guardrails — do not reopen

- **The spec is settled and landed.** Do not edit `spec/`. Cite it.
- **Do not touch the Ken fences.** LET-4 owns them; they are canonical.
- **~~Do not re-baseline `FRAME_LINE_COUNTS`.~~ WITHDRAWN — DELETE it, and every
  other corpus-inventory count (D6).** *Three positions in one day, so here is the
  final one and its reasoning:* **re-baselining** an oracle to match the artifact
  it constrains is a rubber stamp (my first guard, and still true in general);
  **retiring** a discharged proof is right (Pat's first ruling); **but a count
  floor of ANY value is itself the disease** — *every legitimate corpus
  restructure renegotiates it* (Pat's second, and the one that generalizes).
  **Git history records deletions. A test must not.**
- **Do not convert single-binding lets to groups.** (§2. This is the failure mode.)
- **Do not touch `kenfmt_c_capstone.rs` or `ken_fmt.rs` beyond D6** — the
  surviving tests, the per-file FS-authority assertion, and every helper stay
  exactly as they are.
- **⛔ Do NOT delete a `.len()` assertion about SEMANTIC content** (`Q` has one
  entry; `Temporal` has nine constructors). **Only assertions that COUNT FILES IN
  A GLOB die.** *Over-applying this ruling is the way D6 goes wrong.*
- **`agent/`, `docs/`, `spec/`, `tooling/` are swept by no formatter gate** — only
  `catalog/**` and `examples/rosetta/**` are. Know which of your files is code.
