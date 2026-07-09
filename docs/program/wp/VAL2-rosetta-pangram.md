# VAL2 — Rosetta pangram (surface validation, wave 2)

**Steward frame. Light QA mini-WP → Team Language.** This is **not** a spec/build
WP: **no `§2c` pipeline, no `/spec` elaboration, no Architect gate, no CV gate.**
It continues VAL1 (`docs/program/wp/VAL1-rosetta-surface.md`) — its rulings
(dir-per-item self-checking regressions, gaps-are-the-output, cross-lane routing
to the Steward) carry unchanged. Owner: **language-leader → language-implementer
(runner + examples) → language-qa (run + verify + confirm idiomatic).** Gate:
**light** — Steward-approved, Integrator merges (examples + a black-box test
runner; no kernel/`trusted_base` surface). Findings → **Steward**.

## Why
Validate the whole surface end-to-end on 10 diverse, recognizable programs — a
"quick brown fox" pangram for Ken. The **valuable output is the gaps**: a task
that can't be expressed cleanly, or is unergonomic, is a *finding*, not a
failure. The 6 landed examples (`hello-world, fizzbuzz, factorial, fibonacci,
gcd, ackermann`) cover output/loops/simple-recursion/bignum/single-fn-SCT; this
wave reaches the axes they don't: strings, collections/maps, ADTs+match+trees,
higher-order fns, typeclasses/polymorphism, effects/IO, Option, and deliberate
totality/state stressors.

## Shared-first — build/reuse the common substrate ONCE (operator-directed)
No repeated work across the examples. The shared substrate is built once or
already exists:
- **Interpreter** — `ken-interp`, already built; examples run on it as-is.
- **Landed packages** — `catalog/packages/collections` (7-combinator `List`/`Nat` floor +
  string surface `concat`/`slice`/`charAt`/`eq`/`compare` + local `OrdResult`),
  `catalog/packages/lawful-classes` (`Ord`/`Eq`/`DecEq`). Examples **import and reuse**
  these — **no example re-derives** `list_append`, a sort primitive, or a string
  op.
- **The differential runner** — the one new shared piece; **built first** (see
  Deliverable 1), wired against the existing 6 before any of the 10 are added.
- A shared `.ken` helper is factored to **one** support module **only if it
  genuinely recurs across ≥2 examples**; otherwise reuse landed packages. **Do
  not speculatively build a helper library** (YAGNI).

## Deliverable 1 — the differential runner (built FIRST)
A small **differential harness** that turns `examples/rosetta/` into
CI-enforced regressions so they don't rot. Contract:
- Glob `examples/rosetta/*/`; for each, run `<slug>.ken` through the `ken` CLI,
  capture stdout, and compare to the dir's oracle.
- Each example dir declares its status by which oracle file it carries:
  - **`expected`** (must-match) → hard **pass/fail**; a mismatch reds CI.
  - **`KNOWN-GAP.md`** (documented finding: what's missing / can't-express /
    unergonomic, + the finding routed to Steward) → **recorded non-blocker**:
    the runner lists it in a findings summary but does **not** red CI. This is
    how a gap-probe lives in the corpus as a *documented finding* instead of a
    silent skip or a broken build.
  - A task that fails to compile/run with **neither** oracle present is a **hard
    error** (never a silent skip).
- Prefer a **cargo integration test** (`crates/ken-cli/tests/rosetta.rs`) so CI
  runs it for free. It is **soundness-inert** — drives the compiler as a black
  box and diffs text; no kernel/`trusted_base` touch → light gate.
- **Generous per-example timeout.** A known `ken-interp` perf characteristic
  (deep unary-`Nat` recursion ~O(n^3.5–4), forward-tracked from
  L3-strings-surface) means some deep-recursion tasks are slow-but-correct — the
  runner must not time-bomb them; if an example trips it, that's a **noted
  Runtime-lane finding**, not a fix here.
- **Runner-first proof:** land the runner green over the existing 6 examples
  before adding any of the 10.

## Deliverable 2 — the 10 example programs
Dir-per-item under `examples/rosetta/<slug>/`: `<slug>.ken` + (`expected` **or**
`KNOWN-GAP.md`) + `README.md` (task statement + notes). **Simplest first**,
reusing landed packages. Each probes a distinct axis / suspected gap:

| # | Task (slug) | Rosetta | Axis it validates / gap it probes |
|---|---|---|---|
| 1 | `palindrome` | Palindrome detection | strings (L3 surface), `Char` eq, reverse/fold |
| 2 | `merge-sort` | Sorting/Merge sort | structural recursion + SCT, `Ord` typeclass, parametric polymorphism |
| 3 | `tree-traversal` | Tree traversal | user inductive `data Tree`, `match`, structural recursion (BFS ⇒ explicit queue-threading friction) |
| 4 | `closures` | Closures/Value capture | higher-order fns, closures, functions as first-class list elements |
| 5 | `letter-frequency` | Letter frequency | **Map gap** (no `Map` primitive) + strings + basic I/O |
| 6 | `read-file-lines` | Read a file line by line | `[FS]` effect, streaming-under-totality, `Option` (missing file) |
| 7 | `rpn-calculator` | Parsing/RPN calculator | `Option`/`Result` composition **without exceptions** (parse + stack-underflow + div0); monad-ergonomics probe (subsumes standalone div-by-zero) |
| 8 | `mutual-recursion` | Mutual recursion | **SCT mutual recursion** (`declare_recursive_group` at the surface); tiny/high-signal |
| 9 | `hailstone` | Hailstone sequence | SCT vs **conjectured-not-proved** termination → fuel/`partial` idiom probe |
| 10 | `accumulator-factory` | Accumulator factory | pure/total vs **mutable state** → `[State]` effect or state-threading finding |

## The valuable output: gaps, not just green checks
For any task that can't be expressed, is unergonomic, or fails: write a
`KNOWN-GAP.md` in its dir (what's missing / what broke / why the encoding is
awkward) **and route a finding to the Steward immediately** — do not batch to
the end; do not paper over a gap with a hack that hides it. **QA polices
"idiomatic"** — a solution that smuggles past a gap with an ugly workaround is a
finding, not a pass.

## The gap → capability boundary (what keeps the light gate safe)
Examples are written against the **current** language. A gap whose **fix needs a
new capability** — a `partial`/fuel escape (hailstone), a `Map`/`Vector` stdlib
type (letter-frequency), a `[State]` effect (accumulator) — is **not** patched
inside this light-gated WP. It **exits as a routed finding to the Steward** and
becomes its **own properly-gated WP** (spec/Architect as warranted). A bug rooted
**outside Language's lane** (`ken-interp`, kernel, runtime) likewise routes to
the Steward → owning team; **Language fixes only its own lane** (elaborator,
parser, stdlib surface). This preserves "no Architect on the mini-WP" without
ever letting an unsound capability-fix land under the light gate.

## Acceptance
- Runner lands green over the existing 6 first, then over the 10 (must-match
  dirs pass; known-gap dirs recorded).
- Each of the 10 is either (a) `expected`-matching and idiomatic, or (b) a
  documented `KNOWN-GAP.md` with the finding routed. No papered-over hacks.
- All findings routed to the Steward as they surface; capability-fixes spun out
  as their own WPs, not folded in.

## Parked: tier-2 stressors (fast-follow, decided after wave-1 velocity/cost)
`nth-root` (rationals + fuel iteration), `haversine` (declares the no-floats
boundary), `y-combinator` (typed-`Fix`/general-recursion story), `sieve`
(mutable-array/`Vector` gap), `lcs` (2D memo), `continued-fraction` (codata vs
totality). Run these as a wave-3 only if wave-1 proves quick + cheap.

## Sequencing
Team Language is mid-WP on L3-strings-surface (QA in flight). VAL2 releases at
that WP's **close seam** — retros in → Handoff-Gate compaction of the team →
kickoff — per the one-WP-per-team discipline. Staged now; fires the moment the
current build closes.
