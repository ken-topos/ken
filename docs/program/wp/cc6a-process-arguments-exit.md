# WP CC6a — `Process.Arguments` + `System.Exit`

The pure, application-facing values around the runtime ABI: what a program
**receives** (argv) and what it **returns** (an exit policy). This is the last
piece **`ArgParse` (CC7) actually depends on** — and CC7 is the **Milestone-C
exit criterion.**

**Program II (catalog closure), CC6a.** Owner: **Foundation**. Reviewer:
**Architect** — **CV only if `conformance/` is touched**. Size: **M**.
Base: `origin/main @ 3c7d6ce5`. Branch: `wp/cc6a-process-arguments-exit`.

**Zero trust delta** — ordinary kernel-checked catalog Ken; no kernel rule, no
primitive, no postulate, no `Axiom` in CC6a's fences.

## ⚠ SCOPE: `System.Path.Posix` is NOT in this WP — it is HELD

The original CC6 was `Process.Arguments` + `System.Exit` + `System.Path.Posix`.
**`Path.Posix` is deliberately held**, pending an **operator decision**, and
**you must not build it here.**

**Why (and this is your own carry doing the work):** splitting a path on `/`
requires iterating a `Bytes`. `bytes_at`/`bytes_length` traffic in **opaque
`Int`**, and there is **no `Int → Nat` anywhere** — so a terminating structural
fold over a bare `Bytes` **cannot be written**. The only route that builds is the
**cached-`Nat` carrier** (CAT-5's `Source`, CC3's `ArgBytes`). That would be the
**fourth** hand-paid instance — and **CC3's promoted retro carry says the third
should force a substrate decision, not another local copy.** The Steward is
honoring that carry: the decision (land a `bytes_length : Bytes → Nat` primitive
or a certified `Int → Nat`, versus keep paying the tax) is **with the operator**.

**Holding it costs nothing on the critical path: CC7 needs argv tokenization,
which is not path parsing.** If you find yourself needing to split a raw `Bytes`
in CC6a, **STOP and escalate — the shape drifted.**

## Fixed inputs (settled — do NOT reopen)

Grounded against `origin/main @ 3c7d6ce5`. **Treat every anchor as perishable. If
a fixed input is FALSE, say so with exact tree anchors and ESCALATE — do not
build around it.** (That clause has now caught a bad pin of mine **five** times.
It is the most valuable line in every frame I write. Use it.)

1. **The ABI values are ALREADY LANDED in the prelude — do NOT re-declare them.**
   - `data ExitCode = Success | Failure UInt8`
   - `data ProcessInput = MkProcessInput (List Bytes) (List (Prod Bytes Bytes)) Bytes`
     — i.e. **argv** (`List Bytes`), **environment** (`List (Prod Bytes Bytes)`),
     and a third `Bytes` field. **Verify what that third field is against the
     landed runner before you name it in an accessor** — do not guess from this
     frame.
   CC6a builds the **pure application-facing values over these**, and adds **no
   new ABI type.**

2. **★ `Process.Arguments` CONSUMES CC3's LANDED CARRIER — it does not mint a new
   one.** CC3 already landed **`ArgBytes`** (the proof-carrying cached-`Nat`
   argv carrier) and **`ArgLocation`** (arg index + byte range) in
   `Parsing/Cursor`. **Those are exactly "raw argv bytes + index + byte range."**
   **Reuse them.** Building a second argv location type would be the
   proliferation this catalog exists to avoid — and would be a **fifth**
   hand-paid carrier.
   - **⇒ CC6a needs ZERO new cached-`Nat` carriers.** If you reach for one,
     **STOP and escalate.**

3. **`System.Exit` is a POLICY layer, not a new type.** `ExitCode` is landed
   (fixed input 1). CC6a supplies the **explicit, total** application-facing
   policy: success, failure with a code, and the mapping a program uses to
   *choose* its exit status — **with no host knowledge of the program's result
   datatype** (the I-1 contract property: the runner must not inspect an app's
   result shape). **No new `ExitCode`. No `Int` exit codes** — the landed type is
   `UInt8`.

4. **Homes:** `Process.Arguments` → `catalog/packages/Process/Arguments.ken.md`;
   `System.Exit` → `catalog/packages/System/Exit.ken.md` (§13's identity map).

5. **Package model — unchanged.** No cross-file `import`/`pub`; dependency-bearing
   packages elaborate **in order into ONE shared `ElabEnv`** (AC1). A standalone
   `ken check` of a dependent package is **expected to fail** — the known
   package-model gap, **not** a bug to route around. **Escalate; do not smuggle
   `import`.**

## Mandated deliverable outline

1. **`Process.Arguments`** — the pure view over `ProcessInput`'s argv: the arg
   list, positional access **by index**, and the **byte-range location** of a
   slice within an arg — **all expressed over CC3's landed `ArgBytes` /
   `ArgLocation`** (fixed input 2). **Byte-preserving throughout**: argv values
   are `Bytes`, **never decoded to `String`** (the "POSIX bytes, not `String`"
   guardrail — decoding is an explicit *library* choice a caller makes, not
   something this package does for them).

2. **`System.Exit`** — the total exit policy over the landed `ExitCode`
   (fixed input 3). A program's result → an exit status, **explicitly**, with no
   ambient/default coercion and no host inspection of the result datatype.

## Acceptance criteria (testable)

- **AC1 — DS-7/8 ordered shared-`ElabEnv` harness.**
  `crates/ken-elaborator/tests/cc6a_process_arguments_exit_acceptance.rs`,
  following `cc5_pretty_doc_acceptance.rs`: ONE shared `ElabEnv`, dependency
  closure elaborated **IN ORDER** (… → `Parsing.Cursor` (for `ArgBytes`) →
  `Process.Arguments`, `System.Exit`), then every checked fence. **NOT a
  standalone `ken check`.**
- **AC2 — argv is BYTE-PRESERVING.** A **non-UTF-8** argv value survives
  round-trip through `Process.Arguments` **byte-identically**. **This is the
  discriminator that proves we did not quietly decode** — a `String`-based
  implementation fails it, and a UTF-8-only test would not catch that. **Use a
  genuinely invalid UTF-8 byte sequence.**
- **AC3 — location faithfulness at a NON-DEGENERATE position.** A slice at
  **arg 2, bytes 3–5** reports exactly `(2, 3, 5)`. **Non-zero index AND non-zero
  offset** — an arg-0-only or offset-0 case passes a broken implementation.
- **AC4 — `System.Exit` is total and explicit.** Every result maps to an exit
  status; `Failure` carries the landed `UInt8`. **No default/ambient coercion.**
- **AC5 — zero new carriers.** No new cached-`Nat`-over-opaque-`Bytes` type;
  `Process.Arguments` **consumes CC3's `ArgBytes`**. Grep-able. (Fixed input 2 —
  this is the AC that keeps the substrate decision the operator's.)
- **AC6 — zero trust delta.** No `Axiom` in CC6a's fences; `trusted_base()` before
  == after; no kernel/prelude/`Cargo`/lock delta; **no new primitives**; **no
  re-declaration of `ExitCode`/`ProcessInput`.**
- **AC7 — corpus-wide oracles (BOTH).** New catalog files ⇒ run **both**
  `crates/ken-cli/tests/ken_fmt.rs` **and**
  `crates/ken-elaborator/tests/kenfmt_c_capstone.rs` targeted before release.
  **Add NO row to `FRAME_LINE_COUNTS`.**
- **AC8 — scope discipline.** Only `Process/Arguments.ken.md`,
  `System/Exit.ken.md`, and the AC1 harness. **NO `Path.Posix`** (held).

## Do-not-reopen guardrails

- **No `System.Path.Posix`** — held on an operator decision. If you need to split
  a raw `Bytes`, **escalate.**
- **No new cached-`Nat` carrier** — consume CC3's `ArgBytes` (AC5).
- **No decoding argv to `String`** — byte-preserving throughout (AC2). Decoding is
  the *caller's* explicit choice.
- **No re-declaring `ExitCode`/`ProcessInput`** — they are landed prelude types.
- **No new primitives.** No `import`/`pub` smuggling — escalate package-model gaps.

## Sequencing & review chain

Foundation builds → Foundation QA → **Architect** (he should press hardest on
**AC2 — is argv genuinely byte-preserving, or is there a `String` hop hiding in
a helper?** A quiet decode would pass every UTF-8 test and silently break
non-UTF-8 argv, which is exactly the guarantee the work program pins) → CV only
if `conformance/` is touched → `git_request` to the Steward → honesty gate +
publish. CC6a closes when it lands **and** its §10 retros are in.

**Next: CC7 (`ArgParse`) — the MILESTONE-C EXIT CRITERION.** It consumes
`Process.Arguments` (this WP), `Parsing.Decoder`/`Cursor` (CC3), `Diagnostic`
(CC4), `Pretty.Doc` (CC5), and `Validation` (CC1). **Everything it needs will
exist when this lands.**
