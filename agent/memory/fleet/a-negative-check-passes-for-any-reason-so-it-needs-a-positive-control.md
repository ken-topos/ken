---
scope: fleet
audience: (see scope README) — anyone writing or reviewing a negative
  assertion: `compile_fail`, `expect_err`/`is_err`, `#[should_panic]`,
  "should reject" fixtures, refusal matrices, any check whose job is to
  confirm something is FORBIDDEN or ABSENT
source: ORACLE-VIS-CHECK / ORACLE-VIS-PACKAGING / Q-CLAIM-CLOSURE /
  RT-FNSPLIT-B2F / RT-MATCH-FRAME-FP, 2026-07-22 through 2026-07-27 — seven
  compounding instances of the same defect class, several self-caught,
  one caught by an adversary, one by QA
---

# A negative check passes for ANY reason — it needs a positive control

A check that asserts something **fails** (`compile_fail`, `expect-error`,
`assert!(result.is_err())`) passes when the target fails for **any** reason —
including a typo, an unrelated error, or a harness that never ran. Pair every
negative assertion with a positive control, and write the negative probe so
it can only fail for the reason you mean.

## The trap, caught live on the first run

Replacing a source-text visibility oracle with `compile_fail` doctests asking
the compiler whether a test-only helper is reachable as public API from
outside the crate, the first probe used a *use-the-item* form:

```rust
//! ```compile_fail
//! let _ = ken_runtime::cranelift_backend::emit_process_entrypoint_object_with_cranelift;
//! ```
```

It passed. **It would have passed forever** — including after the helper was
made fully public. The helper's signature takes `entry_symbol: impl
Into<String>`, i.e. **generic**, so `let _ = <path>;` fails with `E0283`
(type annotations needed) *whether or not the path resolves*. The block
asserted "this doesn't compile" and got a free yes from an **inference**
error that has nothing to do with **visibility**.

⇒ That would have replaced a text-pin proxy with a vacuous check — the exact
failure mode the WP existed to remove, reintroduced one level down, and
strictly worse because it *looks* like the compiler is answering.

## What caught it

A **positive control**: a plain (non-`compile_fail`) block naming a
genuinely public item from the same module in the same form. It failed
immediately on `E0283` and made the inference confound visible. Without it,
everything was green and wrong.

## The two fixes, and they are different

1. **Pair every negative assertion with a positive control.** A
   `compile_fail` block passes when the snippet fails for *any* reason — a
   typo in the crate name, a missing import, an unrelated error. The control
   is what separates "correctly rejected" from "harness broken".
2. **Write the negative probe so it can only fail for the reason you mean.**
   `use <path> as _;` resolves a path and checks visibility and *nothing
   else* — no call, no argument types, no inference. Narrowing the probe to
   the property is what makes the negative result mean something.

Fix 2 is the one most likely to be missed: a positive control tells you the
harness works, but only an isolated probe tells you the negative is *about*
your property.

## THIRD axis, ~1h later: FRESHNESS

Rebuilding the same check in a different harness — compiling probe snippets
against the crate's built `.rlib` from an integration test — shipped a
version that passed its own direction-1 mutation: making the item genuinely
`pub` and the check stayed green.

**Cause:** the rlib was picked with `candidates.sort()`, which sorts by
**filename hash**. `target/debug/deps` accumulates **one rlib per build** —
there were 15, spanning a full day. The probe was compiling against
hours-old source, **and the positive control compiled against the stale
artifact perfectly happily.**

⇒ **A positive control proves the harness WORKS. It cannot prove the harness
is looking at the CURRENT code.** Those are different questions.

**How to apply — for any check that reads a BUILT ARTIFACT rather than
source:**

- Ask *"which build produced the thing I just measured?"* Applies to rlibs,
  compiled binaries, generated files, snapshots, caches, `target/` anything,
  and any glob over a directory that accumulates across builds.
- **Never select an artifact by name/hash order when the directory
  accumulates.** Order by mtime (newest), or better, derive the path from
  the build system rather than globbing for it — and write down the
  invariant the choice rests on.
- If a mutation proof passes when it should fail, **suspect a stale input
  before doubting the mutation**. That is the signature.

## FOURTH axis, next WP, same day: the attack was NAMED and shipped anyway

A handoff listed, as attack point #1 for QA:

> *"The residual text pin is the soft spot and I'd hit it first... Try a
> declaration form I didn't imagine — **a line break between the visibility
> and `fn`**, a macro-generated declaration, `pub (crate)` with a space."*

**QA ran exactly that and it passed green.** `pub` ⏎ `fn name(` is legal
Rust, compiles clean, is a genuine widening to `pub`, and the scan matched
visibility against the **same line's prefix** — which is empty on the `fn`
line.

⇒ The attack was identified, written down, handed to someone else — and
nobody spent the four minutes to run it before handoff.

★ **If you can name the attack in your handoff, run it BEFORE you hand off.**
Listing your own known weakness reads like rigor and is actually deferral.
The rule is not "flag it for QA" — it is: **a weakness specific enough to
write down is specific enough to test**, and the writing-down is the moment
you have the strongest evidence you should. QA's job is the attack you
*couldn't* imagine.

**The fix, generalized:** when a check parses text, its **parser** is a
second oracle. Enumerate the legal-syntax variants of the thing being
matched (whitespace, newlines, interposed modifiers, comments, grouping) and
mutate each one — do not enumerate only the semantic variants. The closing
move here was to stop being line-based entirely: locate the declaration,
then walk *backwards over arbitrary whitespace*, skipping every token the
grammar allows in between. **Prefer a mechanism with no line-structure
assumption over a better line-structure assumption.**

**How to apply:**

- Applies to every negative check, not just doctests: `assert!(x.is_err())`
  (assert the specific error variant), expect-fail tests, "should reject"
  fixtures, `#[should_panic]` (always give it `expected =`).
- **Mutation-prove the negative in both directions.** Make the thing you
  claim is forbidden actually happen and watch the check fail; then make a
  property-preserving change and watch it still pass. One direction alone is
  not evidence — a check that only ever passes is indistinguishable from a
  check that cannot fail.
- ⚠ **When constructing the "make it violate" mutation, confirm the system
  still builds.** A first attempt that ungates a module + `pub fn` can make
  the crate fail to compile outright, so `compile_fail` blocks "pass" for
  the wrong reason. **A mutation that breaks the build proves nothing** — it
  silently tests the harness against rubble.
- Ask of any green negative check: *what is the cheapest way this passes
  without my property holding?* If you can name one in ten seconds, the
  probe is too wide.

## 5th axis, same day, 4th instance: ASK A QUESTION THAT NEEDS NO CORRECT LIST

A probe harness's own header said *"a harness that silently sources nothing
passes every negative check"* — and implemented that exact failure forty
lines below the sentence:

- The integrity check was `grep -q "^$fn() {"` per known function. That
  asserts each function's **opening line and never its body.** Truncation
  removes **tails**, so it was structurally blind to the one drift it
  existed to catch.
- The runner used `set -uo pipefail` with **no `-e`** and no guard on
  `source`, so a slice that failed to parse **still exited 0** with every
  negative assertion passing vacuously.
- Containment cannot notice an **addition**: the slice defined 10 functions,
  9 were asserted, and the 10th sat unasserted and green.

⇒ **The fix is not a longer list. It is a different question.**

| the question that keeps failing | the question that survives |
|---|---|
| "are the things I know about present?" | "is this artifact **well-formed and complete on its own terms**?" |
| `grep` for each known name | `bash -n` — catches *any* truncation, anywhere, needs no list |
| containment against a hand-kept set | **derive** the set from the source and assert **EQUALITY** |
| enumerate the duration formats | match *"is there a duration"* |
| reason about the diff | build the RESULT, assert a **predicted** post-condition |

### CORRECTION, same day — over-corrected within the hour

The first write-up said *"adding the missing item to the list is always the
enumeration move; **deriving** the list is the fix."* **That is too broad,
and following it would have removed the check that does the real work.**

Measured on the same harness: drop a whole function *cleanly* from the
slice and

- `bash -n` **passes** — the result is still well-formed;
- the **derived** set-equality **passes** — it derives *both sides from the
  slice*, so it compares "text present" against "defines on source" and is
  structurally incapable of seeing something the slice never contained;
- the **hand-kept external map FAILS** and is the only thing that catches
  it.

⇒ **A derived check cannot detect an omission in the thing it derives
from.** It needs an **external anchor** — a declared list, a predicted
count, a spec — that does *not* move when the artifact moves.

| use | when |
|---|---|
| **derive + assert equality** | to catch **additions** and drift *within* the artifact |
| **an external declared reference** | to catch **omissions**, which derivation cannot see |

**They do different jobs; you usually need both.** The real lesson is not
"never keep a list" — it is *"know which of your checks is load-bearing for
which failure direction,"* and a list that is an **external** reference is
a feature, not the enumeration smell.

## A TRANSCRIPT IS NOT A PROBE

One acceptance-matrix row was discharged by a hand-run transcript (a
cross-worktree lock refusal, deliberately proved cross-worktree). It was
**not re-runnable, and it does not fail when someone changes the lock
path** — which is the exact change it exists to catch. A proof that cannot
fail later is a screenshot, not a gate. If a row's evidence is prose or a
pasted terminal capture, it is undischarged; put it in the harness.

## And the proxy trap on the way out

A coverage check grepped the probe file for each gate function's name.
**Wrong in both directions:** a function genuinely exercised via the marker
it writes was never *named*, so it read as uncovered — while a bare mention
in a comment would have satisfied it. A name-grep measures neither presence
nor absence of coverage. Replaced with a **declared coverage map asserted
for set equality**: every element is driven by a named probe or excluded
**with a stated reason**, so an addition forces acknowledgment.

★ **The stable root cause, stated precisely:** enumerating the ways the
**PROPERTY** can fail while never enumerating the ways the **PLUMBING** can
fail — the plumbing gets one pass while the interesting part gets five.
Four instances in one day. When you harden a mechanism, **audit what holds
it up as a separate piece of work with its own mutants.**

## A GREEN *REMOVAL* PROBE NEEDS TWO CONTROLS, AND THEY ANSWER DIFFERENT QUESTIONS

To test whether a callee ever reads a caller-environment tail, deleting the
tail at all 11 cross-owner sites produced **444 passed, 0 failed**. A green
*removal* is the most seductive result there is — it reads as *"the thing
was dead."* It is equally consistent with **"the corpus never reaches those
sites."**

⭐ **One control cannot separate them, because the two failure modes are
independent.** Both are needed, and each is one run:

| control | mutation | what a GREEN vs RED tells you |
|---|---|---|
| **reachability** | replace the removed statement with `panic!("REACHED-<line>")` | RED naming a site ⇒ the corpus **executes** it. A site that never appears is **uncovered**, and must be reported as uncovered rather than credited to the claim. |
| **non-vacuity** | keep the statement, `assert!(!thing.is_empty())` first | GREEN ⇒ the thing removed was **actually there**. RED names the sites where it was empty, i.e. where the removal was trivially free. |

Measured: reachability named **all 11**; non-vacuity showed the tail
non-empty at **9 of 11**. *Reached, non-empty, and removing it changed
nothing* — only then does the green mean "unread."

**How to apply:**

- ⭐ Whenever a probe **deletes** something and stays green, ask the two
  questions separately: *"did the code run?"* and *"was there anything to
  delete?"* Neither is implied by the other, and the green is worthless
  without both.
- **`panic!` is the cheapest reachability oracle** — one run yields
  per-site coverage from the distinct panic messages, no instrumentation
  harness needed.
- ⚠ **Report the uncovered sites as uncovered.** The temptation is to fold
  "never reached" into "removal was safe"; they are opposite epistemic
  states.
- ⚠ **Check the redden is narrow.** 16 of 444 failing is a targeted result;
  a near-total redden usually means the build broke and you measured
  rubble.

## 7th axis: N NEGATIVES CAN SHORT-CIRCUIT BEFORE THE FIXTURE-VALIDITY CHECK

Four controls on one fixture: three rejections plus one *"and the good case
still lowers."* All three rejections were **green**. Only the positive one
failed — with *"checked marker template has no structural occurrence."*

⇒ The fixture had an empty marker-locations field, so it could never have
lowered under any circumstances. The three negatives each hit their own
detector and returned *before* the plan-validity check ran, so every one of
them passed on an object that was structurally dead.

★ **This is not "the negative passed for an unrelated reason" — it is worse
and harder to see: the negatives passed for exactly the reason intended, on
a fixture that could not have produced the positive outcome.** Each control
was individually correct. The *set* was still vacuous.

⭐ **Early-return ordering makes negatives systematically cheaper than the
positive.** A rejection control only has to reach *its* guard; the positive
control has to traverse **every** guard, so it is the only one that
exercises the fixture's well-formedness. ⇒ **In any set of N rejection
controls, the positive is not the N+1th control — it is the one that
validates the other N.**

**How to apply:**

- ⛔ **Never ship a rejection-only control set.** If every row of an AC
  asserts a refusal, at least one row must assert the *acceptance* on the
  same fixture.
- ⭐ Ask: *"how far into the pipeline does each control get?"* If your
  negatives all return at guard 2 of 9, they have said nothing about guards
  3–9 — and nothing about whether the fixture would have survived them.
- ⚠ Applies to `expect_err` suites, "should reject" corpora, and refusal
  matrices generally, not just compile-fail.

Related fleet lessons on the same family:
[[a-differential-over-an-aggregate-is-an-existential-not-a-universal]] (the
sibling on *observation granularity*),
[[withdraw-and-relocate-test-different-properties]] (the sibling on
*perturbation shape*), [[verify-the-report-is-real-before-explaining-it]].
