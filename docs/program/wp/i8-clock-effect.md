# I-8 — The `Clock` effect (wall-clock only, zero law)

**Team:** Runtime · **Size:** M · **Branch:** `wp/i8-clock-effect`
**Base:** `origin/main @ 7d3f4834` · **Gate:** Program I (Milestone B)
**Design ruling:** Architect, `evt_50605dwhdbb7a` — **transcribed here; do not
relitigate it.**
**Depends on:** I-7 (same team, lands first — no code overlap, but sequence it).

## 0. In one line

Add **one ambient `Clock` effect** with **one op** (`WallNow`) returning a
**structural `Instant`**, wired through the I-6 host seam so `CaptureHost` can
supply a **fixed/scripted clock**. **Zero law. Zero postulate. Zero `Axiom`.
Zero new primitive. Zero kernel-TCB.**

**This is the WP that finally completes I-6's deterministic-app story** — the
"fixed clock" line item I moved out of I-6 because **no clock effect existed to
fix**. Now there will be one.

## 1. ★ START HERE: the clock is ZERO percent built, and `rg Clock` will lie

`rg Clock crates/` returns ~10 hits. **Every one is a decoy:**

- `crates/ken-elaborator/src/effects/row.rs:14` — a **doc comment** listing
  *example* effect names: *"A named effect: `FS`, `Clock`, `Console`, `Net`,
  `Rand`, `Counter`, …"*
- `crates/ken-elaborator/tests/effects.rs:60,68,98,104` — **synthetic test
  fixtures**: `("now", EffectRow::singleton("Clock"))`.

**The effect-row engine is generic over effect-name STRINGS.** `"Clock"` exists
in a unit test because someone typed it in a string literal, exactly like
`"Counter"`. **There is no `IOOp` arm, no `HostHandler` method, no `now`, no
driver arm, no `PosixHost` time source.** You are building this from nothing.
*(Same trap named in I-7's frame; it is real, and it is the reason both frames
open with it.)*

## 2. Settled inputs — the Architect's three rulings, transcribed

**Verify each anchor against the landed code; this section is perishable.**

### 2.1 `Clock` is AMBIENT (Console-class), **not** capability-gated (FS-class)

**No `ClockCap`. No `ProgramCaps` field. No cap argument on the op.**

Reading the clock **grants no resource access and has no scope to confine** —
*you cannot scope which time.* So I-5's scoped-capability machinery does not
apply, and a cap would be pure proliferation (**#7**).

**The effect row already gives the confinement that matters:** a program without
`Clock` in its row **provably cannot read the clock** — you cannot perform an
effect that is not in your row. A cap would add only *scope*, and a clock has
none. **The row subsumes what a cap would buy.**

**Mirror `Console` exactly.** Grounded at `crates/ken-interp/src/eval.rs:2196`:

```rust
fn console_read(&mut self, stream: ConsoleStream, limit: usize) -> io::Result<HostRead>;
fn console_write(&mut self, stream: ConsoleStream, bytes: &[u8]) -> io::Result<()>;
```

`&mut self`, **no cap argument.** That is the shape. (Contrast the `fs_*`
family, which threads handles — that is the capability-gated class, and it is
**not** yours.)

### 2.2 ONE effect, and — for now — ONE op. Carrier is STRUCTURAL.

- **One `Clock` effect**, subsumed at the **effect** level (**#7**) — one row
  entry, one handler family — exactly as `Console` is one effect with
  `Read|Write|Flush|IsTerminal`. **Do not make wall and monotonic two effects.**
- **Carrier: `data Instant = MkInstant Int`** (nanoseconds). **STRUCTURAL, over
  the landed `Int` — NOT a new opaque primitive.**

> **★ WHY THIS MATTERS MORE THAN IT LOOKS.** An **opaque** `Instant` would hit
> **the exact wall SUB-1 exists to fix**: its ordering would be a
> `PrimReduction::Op`, **opaque to kernel conversion**, so nothing could be
> proved about it and every consumer would reach for an **`Axiom`**. A
> structural `Int`-carried instant lets comparison/diff use the **landed `Int`
> ops**, and for the *known* values a `CaptureHost` supplies, comparisons decide
> **at run time**. **Zero new opaque primitive; zero new K3 surface for the
> carrier.**

### 2.3 The host seam — same treatment as I-6's mint

A **required `HostHandler` method** for the clock read:

- **`PosixHost`** — reads the real OS clock.
- **`CaptureHost`** — returns a **scripted/fixed sequence**, **`&mut self`** so
  the sequence advances (the landed scripted-stdin pattern), **plus a trace
  entry** (`clock_trace`, sibling of `console_trace`/`fs_trace`) **so a test can
  assert the read happened, and in what order.**

**This is a runtime driver, exactly like Console. Zero kernel-TCB.**

## 3. ★★ THE HARD RULING: I-8 SHIPS WALL-CLOCK ONLY. NO MONOTONIC. NO LAW. ★★

**Do not add a monotonic clock. Do not add an ordering law. Do not write an
`Axiom`.** This is the whole risk of the WP and the Architect quarantined it
deliberately.

**Why wall-only is the HONEST design, not the lazy one:**

- **A wall clock legitimately jumps** (NTP, DST, an operator setting the date).
  **It has no ordering guarantee**, so it ships with **zero law, zero postulate,
  zero `Axiom`** — just an `Int` timestamp. **Claiming monotonicity for a wall
  clock would be a lie** (**#8**).
- **A monotonic clock's whole value IS its law** (`t₁ ≤ t₂` for successive
  reads). That law is **not definitional and not provable from the values** —
  they are runtime-unknown — and **`leq_int` is itself a `PrimReduction::Op`,
  opaque to conversion.** **`Refl` cannot discharge it.** It is a **runtime
  property of the driver**, the same trust class as FS confinement.
- **⇒ Under PRINCIPLES #15 it must become ONE fixed, named, audited postulate in
  the trusted base** — which every consumer reasons **from** — and **never** a
  per-consumer `Axiom`. That N-consumers-×-N-ad-hoc-axioms shape is *exactly*
  the unbounded-TCB disease #15 was written to kill.

> **★ AND HERE IS THE CATCH THAT MAKES THE DEFERRAL CORRECT RATHER THAN MERELY
> CONVENIENT — read it, because it is the best thing in this frame:**
>
> **A bare `monotonic_now : {Clock} → Int` CANNOT EVEN STATE ITS OWN LAW.** You
> get two `Int`s with **no handle relating them** — there is no way to say *"this
> read is ≥ the last one"* without a **session** that carries the prior instant.
>
> **So a usable monotonic clock is not a bare effect op at all.** It needs a
> **session/handle abstraction** (a `MonoClock` carrying the last instant, plus
> the postulate that the next read is `≥` it) so the **single** trusted-base
> postulate flows to consumers **as evidence they reason from** — rather than
> something each of them re-`Axiom`s. **That is a real design increment, and it
> is a separate WP.**

**⇒ Monotonic-clock + its one audited postulate + the session that makes it
usable = `I-8b`, a NAMED follow-up. Not this WP.**

**If you find yourself writing `Axiom`, or a `≤` law, or a second clock op —
STOP. You are in I-8b.**

## 4. Mandated deliverables

### ⚠ STEP 0 (FIRST, own commit) — repair two oracles you are about to copy

**`crates/ken-interp/tests/i2_console_floor.rs:132` and
`crates/ken-interp/tests/i3_fs_floor.rs:141` are BROKEN**, in the exact way that
took SUB-1's CI red this morning:

```rust
const CONSOLE_PACKAGE: &str = include_str!(…/Text.ken.md);   // ← RAW literate file
assert!(!CONSOLE_PACKAGE.contains("Axiom"));                    // ← greps PROSE + fences
```

**These are the two tests nearest to I-8, they are the ones you will crib the
zero-`Axiom` idiom from, and they are wrong.** They grep the **raw `.ken.md`** —
prose *and* code — for the substring `Axiom`. **They pass today only because no
prose in `Text.ken.md` / `Errors.ken.md` has yet happened to use the word.** The
moment an author documents *"this package needs no `Axiom`"* — **a true and
valuable sentence** — CI goes red on a lie. **That already happened**: DS-4's
identical oracle fired on SUB-1's prose *asserting the absence of an `Axiom`*.

**Fix both to extract first, then assert on the extracted Ken code** — the idiom
CC6a already uses correctly (`cc6a_process_arguments_exit_acceptance.rs:390`,
which asserts on `extract_ken_md(…).source`):

```rust
let extracted = ken_elaborator::literate::extract_ken_md(CONSOLE_PACKAGE)
    .expect("Text.ken.md must extract");
assert!(!extracted.source.contains("Axiom"), "Text.ken code must declare no Axiom");
```

**This STRENGTHENS both gates** — still fails on a real `Axiom` in a fence (all
they ever meant to catch), stops false-firing on prose. **Do NOT weaken or
delete either assertion, and NEVER reword a document to appease a grep.**

**★ The law you are inheriting: a "zero-X" gate checks what the artifact
DECLARES — never whether the string `X` occurs in it.** Prohibitions, negative
assertions, doc comments, and WP frames all **name the thing they forbid** —
*that is what forbidding looks like.* **A grep SELECTS candidates; it never
DECIDES.** (Fleet memory:
`agent/memory/fleet/an-oracle-that-greps-a-name-fires-on-prose-that-denies-it.md`.)

### Then the WP proper

1. **Surface:** `data Instant = MkInstant Int`; `wall_now : {Clock} → Instant`
   (catalog package, e.g. `Capability/Time/WallClock.ken.md`).
2. **Effect:** one `Clock` entry in the effect row; `ClockOp` with the `WallNow`
   op; the response/`clock_resp` plumbing, mirroring `Console`.
3. **Host ABI:** a **required** `HostHandler` clock method (`&mut self`, **no cap
   arg**).
4. **Drivers:** `PosixHost` → real OS clock. `CaptureHost` → **scripted/fixed
   sequence + `clock_trace`.**
5. **The deterministic test that is the point of the WP** (below).

## 5. Acceptance criteria

1. **AC1 — a Ken program can read the wall clock**, and it computes.
2. **AC2 — ★ THE HEADLINE: a program that reads the clock TWICE produces a
   BYTE-IDENTICAL snapshot across two runs under `CaptureHost`.** This is I-6's
   AC5 in a new hat, and it is **the reason the WP exists.** Assert the effects
   **reached the injected host** (`clock_trace` shows the reads, **in order**) —
   **not** merely that a `CaptureHost` was constructed. *A construct-then-ignore
   test fails this AC.*
3. **AC3 — the row genuinely confines.** A program **without `Clock` in its
   declared row** that calls `wall_now` is **REJECTED**, and the error **names
   `Clock`** with the witness. *(Assert the specific error variant, not
   `is_err()`.)* **This is the AC that carries the whole "no cap needed"
   argument — if it does not hold, the ambient ruling is unsound and you must
   stop and report.**
4. **AC4 — `trusted_base()` MUST NOT GROW.** Zero new `Axiom`, zero
   `Decl::Opaque`, zero new opaque primitive, zero kernel change.
   **★ ASSERT THIS AT THE EMISSION, NOT AT THE TOKENS.** The check is
   `trusted_base()` **before == after** across elaboration of the new package,
   plus **zero trust-adding DECLARATIONS** in the *extracted* Ken. **Do not gate
   on a token grep of the diff** — this frame, and your own acceptance test's
   forbidden-list, both spell `Axiom` and `Clock` *while prohibiting them*, and a
   naive grep reads a prohibition as a violation. **(The Steward's own I-7 gate
   did exactly that and nearly blocked a clean WP; see Step 0.)** The one grep
   that *is* sound here is the **path** probe — `crates/ken-kernel` must be
   **absent from the diff** — because that asks about files, not meaning.
5. **AC5 — no ordering law anywhere.** No `≤`/`leq` law on `Instant`, no
   monotonicity claim in the docs, **no `Axiom` DECLARED anywhere.** **A green
   diff that added one still FAILS this WP.** *Saying the words "no ordering
   law" in prose is required and correct; **declaring** one is the violation.*
   **State the absence of the law out loud** — §3 explains why a wall clock
   genuinely has none, and that honesty is a deliverable, not a gap.
6. **AC6 — no regression.** Green **in CI** (never a local `--workspace` run —
   `COORDINATION.md §12`). **Enumerate every corpus-walking oracle your new
   catalog file will be swept into** (`ken_fmt`, `kenfmt_c_capstone`, and any
   test that globs `catalog/packages/`) and run them — that enumeration is
   Runtime's own I-7 carry, and it is what CI will do to you anyway.

## 6. Do-not-reopen guardrails

- **No `ClockCap`, no `ProgramCaps` field, no cap arg** (§2.1 — ruled).
- **No monotonic clock, no ordering law, no `Axiom`** (§3 — ruled).
- **No opaque `Instant`.** Structural over `Int`. An opaque carrier walks
  straight into the SUB-1 wall.
- **No second effect.** Wall and monotonic are two **ops** of **one** effect —
  and only one of them ships here.
- **No `Rand`, no `Net`, no `Counter`.** They appear beside `Clock` in that same
  doc-comment. They are not in scope and they do not exist either.

## 7. Standing hard-stop authority

**§2's anchors are perishable and I did not read every line of the interpreter.**
If a pinned anchor contradicts the landed code — a trait shape, an effect-row
mechanism, an assumption that the row genuinely rejects an undeclared effect —
**STOP BEFORE EDITING**, keep the tree clean, post exact `file:line` anchors, and
name any bypass you could have taken but didn't.

**Two WPs in a row have been saved this way, one of them by killing a false pin
in my own frame pre-edit.** Do it again if this frame is wrong.
