# WP CC5 — `Pretty.Doc` (document algebra + deterministic renderer)

A small ordinary-Ken document algebra and a **deterministic,
width-parameterized renderer**, with laws. This is the piece that turns
structured values into text — and it exists precisely because **CC4 kept
`Diagnostic` render-free.**

**Program II (catalog closure), CC5.** Owner: **Foundation**. Reviewer:
**Architect** (soundness/design) — **CV only if `conformance/` is touched**.
Size: **M**. Base: `origin/main @ edb99c1e`. Branch: `wp/cc5-pretty-doc`.

**Zero trust delta** — ordinary kernel-checked catalog Ken; no kernel rule, no
primitive, no postulate, no `Axiom` in CC5's fences.

## Fixed inputs (settled — do NOT reopen)

Grounded against `origin/main @ edb99c1e`. **Treat every anchor as perishable.
If a fixed input is FALSE against the landed code, say so with exact tree
anchors and ESCALATE — do not build around it.** (This clause has caught a bad
pin of mine four times in two days. It is not decoration, and I would rather
learn an audit of mine is insufficient than believe it worked.)

1. **★ THE CONSTRUCTIBILITY PIN — `Doc`'s text leaf stores `List Char`, NOT
   `String`. This is the WP's central decision and it is SETTLED.**
   I ran the audit against the tree so you do not have to re-derive it:
   - **There is NO `string_length` primitive.** The only landed `String` ops are
     `string_to_list_char` and `list_char_to_string`. `String` is **opaque** —
     **constructible but not destructible.**
   - A renderer **must measure width** (`group`/`alt` must know whether a `Doc`
     fits in the remaining columns). On an opaque `String` there is **no width
     to read.**
   - **`List Char` IS destructible** — its length is structural (`Nat`), so
     width is computable *and* the renderer's **laws are provable**. A universal
     law over `∀ (s : String)` would **not reduce** (`string_to_list_char` on a
     neutral var is stuck) — the same wall CC2 hit and confined.
   - **⇒ `Doc`'s text leaf carries `List Char`. The `String`-facing API is a thin
     conversion at the BOUNDARY** (`string_to_list_char` in, `list_char_to_string`
     out), and **no verified law crosses the `String` boundary** — exactly CC2's
     landed confinement discipline (`Text/Numeric`'s round-trip law lives at the
     digit-list level and never touches `String`).
   - **★ WHY NOT a cached-`Nat` width (the CAT-5 `Source` / CC3 `ArgBytes`
     idiom)? Because it would be the THIRD occurrence** — and Foundation's own
     CC3 carry says a third *"should trigger an explicit substrate-unification
     decision rather than another local copy."* **`List Char` needs no cache, no
     certificate, and no TCB question.** We take the option that does not spend
     the operator's decision. **If you find yourself wanting a cached width,
     STOP and escalate — it means the shape drifted.**

2. **★ DEPENDENCY-DAG PIN — `Pretty.Doc` depends on NOTHING but `Data`/`Core`.**
   It must **NOT** depend on `Diagnostic` (its client). An abstraction module
   that depends on its clients is a cycle — the trap CC3 and CC4 both had to
   correct.
   - **The `Diagnostic → Doc` renderer is NOT in CC5.** Building it here would
     force `Diagnostic` to know about rendering, **destroying CC4's AC4** (the
     value knows its location, not its rendering) — and CC4 landed that
     deliberately so CC5 would have something to do. **Do not undo it.**
   - When that bridge is built, it lives in a **third module** that depends on
     both — never inside `Diagnostic.Core`, never inside `Pretty.Doc`. Out of
     scope here; name it, don't build it.

3. **The algebra is CLOSED at the report's constructors:** `text` / `line` /
   `concat` / `nest` / `group` / `alt`. **Ship exactly these.** No `column`, no
   `fill`, no `align`, no speculative combinator zoo — add one only when a real
   consumer needs it (CC7 `ArgParse` is the next real consumer; it is not here
   yet).

4. **The renderer is DETERMINISTIC and width-parameterized.** Same `Doc` + same
   width ⇒ **byte-identical output**, always. No ambient state, no locale, no
   randomness.

5. **Home:** `Pretty.Doc` → `catalog/packages/Pretty/Doc.ken.md` (§13's identity
   map).

6. **Package model — unchanged.** No cross-file `import`/`pub` (the catalog has
   no disk loader); dependency-bearing packages are elaborated **in order into
   ONE shared `ElabEnv`** (AC1). A standalone `ken check` of a dependent package
   is **expected to fail** — the known package-model gap, **not** a bug to route
   around. **Escalate; do not smuggle `import`.**

## Mandated deliverable outline

1. **The `Doc` algebra** — the six constructors (fixed input 3), with `text`
   carrying `List Char` (fixed input 1). A `Doc` is inert data: it makes **no
   layout decision** by itself.

2. **The renderer** — `render : Nat -> Doc -> List Char` (width in, text out;
   the `String` wrapper is a boundary conversion). Deterministic. `group`
   chooses flat-if-it-fits, else broken; `alt` picks the first alternative that
   fits. **State the fitting rule explicitly and implement exactly it** — a
   renderer whose fitting rule is implicit is a renderer nobody can reason about.

3. **The three laws** (the report's, and they are the WP's real content):
   - **Render preserves text tokens** — the concatenation of the rendered
     output's text content equals the concatenation of the `Doc`'s `text`
     leaves. **Layout adds separators; it never adds, drops, or reorders
     content.**
   - **Width affects layout, not content** — for any two widths, the rendered
     outputs differ only in whitespace/line structure, never in text content.
     (This is the law that makes the renderer *trustworthy*: you can change the
     width and know you did not change what it says.)
   - **Idempotence** — rendering is a fixed point in the sense the report
     intends; state the exact form you prove and prove it.

## Acceptance criteria (testable)

- **AC1 — DS-7/8 ordered shared-`ElabEnv` harness.**
  `crates/ken-elaborator/tests/cc5_pretty_doc_acceptance.rs`, following
  `cc4_diagnostic_core_acceptance.rs`: ONE shared `ElabEnv`, dependency closure
  elaborated **IN ORDER**, then every checked fence; assert the checked globals
  are real, transparent, kernel-checked terms. **NOT a standalone `ken check`.**
- **AC2 — the three laws are KERNEL-CHECKED, not tested.** They are proofs in
  CC5's fences, `Axiom`-free. **A test that renders two examples and compares
  strings is NOT the law** — it is a sample. The laws are the deliverable; the
  examples are evidence.
- **AC3 — determinism, discriminating.** Same `Doc` + same width ⇒ byte-identical
  output across runs. **And a NON-DEGENERATE pair:** a `Doc` that **fits** at
  width `w` renders flat, and **the same `Doc`** at width `w-1` renders **broken**
  — proving the width actually drives the choice. (A single width proves nothing:
  a renderer that always breaks, or never breaks, passes it.)
- **AC4 — `group`/`alt` fitting is exercised at the BOUNDARY.** Test at exactly
  the width where the fit flips, and at ±1. Off-by-one in a fitting rule is the
  classic renderer bug and a mid-range test will not find it.
- **AC5 — no `String`-crossing law.** No verified law in CC5's fences quantifies
  over an opaque `String`. Laws live at the `List Char` level (fixed input 1).
  The `String` wrappers are **functions**, not proof-carrying.
- **AC6 — zero trust delta.** No `Axiom` in CC5's fences; `trusted_base()` before
  == after; no kernel/prelude/`Cargo`/lock delta; no new primitives. **In
  particular: no `string_length` primitive** — if you think you need one, the
  shape drifted (fixed input 1); **escalate.**
- **AC7 — corpus-wide oracles (BOTH).** CC5 adds a new catalog file, so run
  **both** `crates/ken-cli/tests/ken_fmt.rs` **and**
  `crates/ken-elaborator/tests/kenfmt_c_capstone.rs` targeted before release.
  **Add NO row to `FRAME_LINE_COUNTS`** — it is a discharged historical baseline
  (CC3 re-scoped it to a coverage check precisely so new files never force a
  fabricated row). **Keep it that way.**
- **AC8 — scope discipline.** Only `Pretty/Doc.ken.md` + the AC1 harness. **No
  `Diagnostic` change** (fixed input 2 — CC4's render-free property is
  load-bearing). No conformance touch unless deliberate.

## Do-not-reopen guardrails

- **`Doc`'s text is `List Char`** — not `String`, not a cached-`Nat` width. The
  cached-`Nat`-over-opaque idiom is at **two** occurrences and a **third is an
  operator-facing substrate decision**, not a build-WP choice.
- **`Pretty.Doc` never depends on `Diagnostic`** — and CC5 does **not** build the
  `Diagnostic → Doc` bridge.
- **The algebra is closed at six constructors** — no speculative combinators.
- **The laws are proofs, not tests** (AC2). A green example suite is not a law.
- **No new primitives**, especially no `string_length`.
- **No `import`/`pub` smuggling** — escalate package-model gaps.

## Sequencing & review chain

Foundation builds → Foundation QA → **Architect** (soundness/design; he should
press hardest on **whether the three laws are genuinely proved or merely
sampled** — a renderer whose "laws" are example-tests is a renderer with no
laws) → CV **only if** `conformance/` is touched → `git_request` to the Steward
→ honesty gate + publish. CC5 closes when it lands **and** its §10 retros are in.

**Next in the chain:** CC6 (`Process.Arguments` / `System.Exit` /
`System.Path.Posix`) → **CC7 (`ArgParse`)**, which is the **Milestone-C exit
criterion** and the first real consumer of `Doc` (usage/help rendering). Design
`Doc` so CC7 can use it — but **do not build CC7's needs speculatively.**
