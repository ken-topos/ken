# `KW-ORACLE-CLOSURE` — close the source oracle structurally

> **Node:** `docs/program/issues/KW-ORACLE-CLOSURE.md` — read it first; it carries
> the adversary's findings and the measurements. **This frame is the how.**
>
> **Owner:** Language ring. **Size:** S. **Branch:** `wp/KW-ORACLE-CLOSURE`.
> **Base:** `origin/main`. One file:
> `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs` (282 lines).

## ⚠ READ THIS BEFORE THE DELIVERABLES — THE CORPUS IS CLEAN TODAY

The adversary **ran both missing checks itself** against the full landed
population and found **zero live instances**. ⛔ **That is not a reason to close
this cheaply, and re-running those measurements is not the deliverable.** Both
findings are about an **oracle whose reach is narrower than the property it
advertises**, and a clean corpus is exactly the condition under which that goes
unnoticed. The deliverable is that the instrument can **see** the population.

## ⭐ Anchors — RE-DERIVED on current `origin/main`, not inherited

The node's citations are against `c72be0b0`; `main` has moved three times since.
**I re-derived every one and they are unchanged** — so you may trust these:

| symbol | line | fact |
|---|---|---|
| `classify` | `:55` | five-arm path allow-list |
| `is_retired_declaration` | `:166` | head-word matcher — what actually runs on the corpus |
| `retired_occurrence_offsets` | `:189` | the occurrence predicate |
| its **only** caller | `:278` | **its own self-test** |
| non-vacuity assertion | `:102-107` | asserts every *class* is populated |
| fence info match | `:130-133` | `markdown[ticks..].trim_end()` |

⛔ **Re-verify them yourself at your base anyway** (`grep -n`), because your base
is not necessarily mine.

## ⭐⭐ THE DESIGN FACT THAT MAKES P2 SMALL — measured, and it changes the shape

`candidate_inputs()` **already enumerates the whole tree**:

```rust
let tree = git(&["ls-tree", "-r", "--name-only", &candidate]);
```

⇒ **The enumeration is not the narrow part — `classify` is.** So P2 is **not**
"write a tree walker" and **not** "add a sixth arm". It is: **invert `classify`
from an allow-list of path shapes into a content predicate plus a closed,
stated exclusion complement.** Every file the tree contains that carries a
`ken`-family fence is in scope; anything excluded is excluded **by name, in one
list, with a reason**.

⛔ **A fix that adds `.md` to the arm list has reproduced the bug a third time.**
`AC-1` was already amended once because the hand-enumerated root list *"was wrong
three ways"*, and that amendment warns: *"a sweep that grew one arm per missed
file has reproduced the bug it exists to prevent."*

⚠ **And the population must be derived over the WHOLE tree, not the adversary's
scanned scope.** The hunt covered `crates/`, `catalog/`, `library/` only — **not**
`spec/`, `conformance/`, `docs/program/`, `tooling/`, `agent/`. ⛔ Inheriting that
scope freezes it in as the answer.

## D1 — P1: the occurrence predicate must run on the corpus

`retired_occurrence_offsets` (`:189`) has exactly one caller: its own self-test at
`:278`. What runs against the corpus is `is_retired_declaration` (`:166`), which
matches only when the line's **head word** is `lemma` / `pub lemma`.

`AC-1` asks for `lemma` / `lemmas` / possessive and plural forms **plus
surface-derived identifiers and anchors**. The head matcher can express **none**
of those. The instrument that can is the uncalled one.

⭐ **The self-test is the sharpest part of the finding, not a mitigation.** At
`:274-282` it probes `"LEMMA lemmas lemma's lemma_identifier"` and asserts all
four are seen. **A rigorous test of an uncalled function is the most convincing
possible form of no coverage** — the rigor is real and aimed at the wrong
question. Do not read it as partial credit.

**Deliverable:** the occurrence predicate runs against the corpus population.

## D2 — P2: derive the population, and close the fence-info hole

Replace the `classify` allow-list per the design fact above.

⭐ **One concrete hole to close while you are in there, verified:** at `:130` the
fence info is `markdown[ticks..].trim_end()`, matched against exactly `"ken"`,
`"ken ignore"`, `"ken reject"`, `"ken example"`. Leading **indentation** is
handled (`indent <= 3` is stripped), but a **space between the ticks and the
info** — ` ``` ken` — yields `info == " ken"` and **escapes the match**. There are
no instances today. ⛔ Fold it into the structural closure rather than leaving a
second narrow matcher behind the new wide population.

⛔ **The non-vacuity assertion cannot catch any of this.** `:102-107` requires the
candidate to populate every structural **class**, and is discharged by the five
classes being non-empty. It says nothing about **which files reach them** — the
same shape as the finding it exists to guard against, one level up. Do not extend
it and call P2 done.

## ⛔ AC → CONTROL MAP — REQUIRED. An AC with no control is invisible.

Every row must name the control **and** the mutation that proves the control is
load-bearing. ⛔ A row whose control is "the suite passes" does not discharge
anything.

| AC | control | mutation that MUST redden |
|---|---|---|
| `AC-C1` occurrence predicate reaches corpus files | a test asserting the predicate is applied to the derived population | widen one corpus file's occurrence set **beyond a declaration head** (e.g. add `lemmas` in prose, or a `lemma_`-derived identifier) — must redden |
| `AC-C2` population is derived, not enumerated | a test that a **new** `.md` file carrying a `ken` fence enters scope with **no** change to any arm list | add such a file; if it must also be registered somewhere, `AC-C2` is **not** discharged |
| `AC-C3` exclusions are a closed, stated complement | the exclusion list is one enumerated place with a reason per entry | remove an exclusion and the population grows **predictably** — an unexplained change means the complement is not closed |
| `AC-C4` fence-info hole closed | a positive control with ` ``` ken` (space after ticks) that IS classified | revert the info normalization — must redden |
| `AC-C5` whole-tree scope | a control asserting a file **outside** `crates/`/`catalog/`/`library/` participates | restrict the population to the adversary's three roots — must redden |

⛔ **`AC-C1` needs a positive control, not only the mutation.** A negative check
passes for any reason — including the predicate never running. The positive arm
must show a real occurrence being **found**, so "found nothing" and "never
looked" are distinguishable.

⚠ **`AC-C2`/`AC-C5`: verify WHICH detector caught your mutation.** A widened
population plausibly reddens several assertions at once. If the redden is
**broader than the blast radius**, suspect the build broke rather than that your
control fired.

## Contention

⛔ **Check the file set against every WP in flight, not just the frontier.** At
authoring time this WP touches **one** test file in `ken-elaborator`, which no
in-flight WP touches — `SPEC-CLOSURE-BOUNDARY` is Markdown, `RT-VALUE-TOTALITY`
is `ken-runtime` and unreleased. **Re-derive this at release**, and run the
cited-source check too (`library/SOURCE-ATTESTATIONS`) — a test file is not
normally attested, but the check is one command and path-intersection-empty is
not the same as publishable.

## Out of scope — do not touch

⛔ **`provide_lemma` stays.** `SuggestedAction::ProvideLemma` /
`"kind": "provide_lemma"` remain on the protocol wire
(`crates/ken-elaborator/src/diagnostics.rs:184`, `protocol.rs:145-148`) as a
deliberate residue under the operator's 2026-07-24 directive — an **API token,
not a language construct**. ⚠ The adversary independently reached the same
boundary and **declined to rule**, noting it is an identifier on the wire while
`AC-1`'s occurrence set includes *"surface-derived identifiers and anchors"* — so
the cell is genuinely ambiguous. It is recorded so the ambiguity is
review-visible. ⛔ **Only the operator reopens it.**

⛔ Do not re-litigate what the adversary attacked and could not break: the
attestation ledger was 50/50 current, no stale line citations, the fence-info
family is complete across the tree, and re-canonicalization moved no signature
content.

## Standing

- ⛔ **Targeted builds only** — `scripts/ken-cargo test -p ken-elaborator --test
  kw_theorem_source_oracle`, and the full `-p ken-elaborator` suite before
  handoff. **Never `--workspace`**; the full gate runs in CI, and
  "no-regression" means **green in CI**.
- ⚠ **A wider population can surface latent instances in newly reachable
  files.** If it does, that is a **finding to route**, not a reason to narrow the
  population back. Report it and hold.
- ⛔ You have **no** GitHub credential. Commit, report the **exact SHA**, and keep
  going — the Steward pushes. Report an unpushed ref rather than stalling.
- Wrap markdown at 80 columns. ⛔ Never `git stash` (≈70 shared worktrees).
