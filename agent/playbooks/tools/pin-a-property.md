---
name: pin-a-property
description: How to write a mechanical pin (test/scan/structural assertion) that actually guards the property it claims. Load before authoring or reviewing an acceptance criterion, a control, a tripwire, or a multi-arm validator. Covers property-vs-form, MEASURED/CLAIMED/GAP, per-pin compile-preserving evasion, fail-closed defaults, allowed-inventory over forbidden-list, advertised-vs-enforced law counts and arm reachability, mutation provenance, and honest residuals.
scope: tools
---

# Pin a property, not a spelling

A **pin** is any mechanical check standing in for a claim: a test, a source
scan, a structural assertion, a control that must redden. Pins are how this
project converts a design property into something CI can defend.

⛔ **The recurring failure of this corpus is not a missing pin. It is a pin that
is real, committed, green — and green for the wrong reason.** Every rule below
was paid for by a blocked candidate. Apply them **per pin**, not once per
candidate: reminders written per-candidate get satisfied by the most salient
control and silently skip the rest.

## 1. State the property, then ask what already enforces it

Write the pin's claim as a **property of the system**, before choosing any
mechanism. Then ask, in this order:

1. **Can the language make the violation unrepresentable?** A property the
   type system or module privacy refuses needs **no detector at all**, and no
   detector can be evaded. ⭐ **The compiler is a legitimate mechanism and
   usually the strongest one available.**

   > **Measured, `RT-FNSPLIT-B2O` (2026-07-25):** of eight attempted evasions,
   > **three came back `cannot compile`** — naming a private type from the wrong
   > module does not fail a test, it **fails to build**. Those three were
   > simultaneously the **strongest evidence in the WP and the cheapest to
   > obtain.** ⇒ Ask this question **first**, not as a fallback when detector
   > authoring gets hard. Both the implementer and the leader named it
   > independently in their retros, which is why it is now the leading bullet.
2. **Is it a behavioural property?** Then a fixture that *exhibits* the wrong
   answer beats any scan for the shape that causes it.
3. **Only then** reach for a source scan — and read §4 before you do.

⚠ **A pin phrased in terms of the artifact you most recently looked at is the
signature defect.** Stating a *population* requirement as a struct change, an
*authority* requirement as a call count, or a *module-boundary* requirement as a
spelling class are all the same error. **Name the property first; the artifact
is downstream of it.**

## 2. ⭐ MEASURED / CLAIMED / THE GAP — write it as its own sentence

For every pin, state three things explicitly and adjacently:

> **MEASURED:** ⟨exactly what the mechanism observes⟩
> **CLAIMED:** ⟨the property the AC asserts⟩
> **THE GAP:** ⟨what must *also* hold for the first to entail the second⟩

⛔ **An implication left implicit is never checked**, because prose slides from
the true half to the wanted half with no seam to inspect. A measured property
can be **fully rigorous, entirely true, and about something else** — rigour does
not supply relevance.

Worked examples that cost this project hard-stops:

| MEASURED | CLAIMED | the gap that was missed |
|---|---|---|
| every occurrence has an origin (**totality**) | threading is mechanical | **closure under parent→child reachability** — a parent's identity need not own the child's entries |
| two concrete types are module-private (**not nameable**) | no outside consumer can key on them | **naming ≠ capability**: derived `Ord`, an `impl Trait` return, or a derived ordinal leaks usable structure without leaking the name |
| the budget balances | the encoding is complete | a balanced total says nothing about which rows exist |

## 3. Attempt a compile-preserving evasion — for EVERY pin

**Try to defeat your own pin without breaking the build.** If you cannot
construct an evasion, say **why the surface is closed**, and ground that on
**visibility of the reachable surface**, never on the files you happened to
scan.

- Field privacy does **not** bound who can call a function —
  **item visibility** does. A `pub(in crate::<subsystem>)` item is reachable
  from every sibling module in that subsystem, not only from the caller you had
  in mind.
- An evasion that a reviewer supplies later is the same evasion you could have
  written first. **Budget for it.**

## 4. Source scans: granularity, defaults, and self-matching

If a scan is genuinely the right mechanism:

- ⛔ **Match TOKENS, not lines or substrings.** A needle like
  `line.contains(".foo(")` is a claim about **formatting**: split the call
  across lines and it matches nothing. Strip comments, split on every
  non-identifier character, compare **whole tokens**. This also stops `foos`
  from being read as `foo`.
- ⛔ **Make "cannot determine" a third outcome that FAILS.** If unknown input
  falls through to pass, every gap in your parsing is a silent green and no
  amount of coverage converges. *"I could not tell"* and *"it is fine"* are
  different answers and only one is evidence.
- ⚠ **Beware needles that collide with unrelated language surface.** A scan for
  `.entry(` cannot distinguish a domain type's field from `BTreeMap::entry`.
  When the needle is ambiguous, tightening it buys false positives, not closure.
- ⚠ **The assertion's needle must not be caller-supplied, and the message must
  not match the oracle.** Count declarations, not substring hits, when the
  failure message itself names the forbidden spelling.

## 5. Pin the ALLOWED inventory, not the forbidden list

A detector that enumerates what it **forbids** is only as complete as its list.
Invert it: assert the **exact permitted set** — the items in a visible surface,
the fields of a variant, the exports of a module, the trait impls of a type —
so that **any addition reddens**, including one nobody imagined.

⚠ **Pin the inventory at the granularity the property needs.** A name list
misses an existing item whose *return type* changes; a `#[derive]` list misses a
hand-written `impl`. An item enumerator that omits `impl` blocks misclassifies.

⛔ **And enumerate items by their HEADS, never by their punctuation.** The same
enumerator lost `impl` and then, separately, `mod x { … }` — **two holes, one
cause**: it filtered candidate declarations on `trimmed.ends_with(';')`, so its
real population is *lines ending in a semicolon*, while a Rust item is
**brace-shaped**. `fn f() {}`, `struct S {}`, and `trait T {}` are the same hole
waiting to be found.

> ⇒ When the second hole in one enumerator turns out to share the first's cause,
> **a third accepted spelling is not the fix** — key on the leading keyword after
> attributes and visibility, and add a **positive control per item form** so each
> braced shape is provably *seen*. An enumerator that grew a `mod` arm and
> nothing else has reproduced the bug at a smaller size.

## 6. Every negative check needs a positive control

**A negative check passes for any reason**, including a broken harness, a
mis-set path, or a fixture that never exercised the mechanism. So:

- **Feed the detector the case you believe it should catch**, in a form you did
  **not** write it against.
- **Prove non-vacuity**: on the fixture, the wrong key and the right key must
  actually **differ**. A control that would pass on a fixture with no split
  proves nothing about the split.
- ⚠ **Positive controls can themselves be spelling-scoped.** Having one is
  necessary, not sufficient.

## 6a. ⭐ A checker's ADVERTISED law count is a claim, not a guarantee

§6 asks whether the detector was **reached**. This asks **which arm fired** — a
different question with a different failure mode, and §6's control does not cover
it. In this one the input **is** constructed, the checker **does** reject, the
test **is** green, and an **earlier arm** returned the error while the arm you
meant to exercise is unreachable code.

**Measured in a landed, reviewed, fail-closed validator (2026-07-25):**
`validate_function_units` advertises **twelve laws** and has **five live
detectors**. Its "scheduling entry has an incoming static body edge" arm cannot
fire, because the function's own **first statement** calls a partitioner that
rejects the identical condition with a different message. Six further arms had no
witness — every attempt to reach them landed on an earlier detector — and one
conjunct compared two values that are equal by construction.

⚠ **The dead arm was also the only quadratic check in the validator** —
`Vec::contains` inside a loop over all edges, both operands scaling with program
size, paid on every call for a law that never runs. **An unreachable law is not
free.**

**So, for any multi-arm checker you author or review:**

- **Per advertised law, produce a witness that reaches THAT arm** — and assert the
  **exact** error, never `is_err`/`expect_err`. ⭐ Asserting the exact message
  rather than mere failure is the single choice that makes an arm's reachability
  observable at all; with `expect_err`, all twelve above read green.
- **An arm with no witness is reported as such** — *"no witness: shadowed by
  ⟨arm⟩"* — **not silently counted as a law.**
- **Resolve a genuinely subsumed arm by DELETING it** (and its cost) or by
  re-ordering deliberately. Twelve stated laws over five live ones is a
  maintenance trap, not defence in depth.
- ⛔ **Do not weaken a live detector on the strength of the count.** If a reader
  believes an arm covers something, they may relax the check that is *actually*
  load-bearing. This validator already carried a comment correcting exactly that
  mistake about one of its laws; the identical hazard applied to every arm below
  that comment.

★ **Why this is a pin-authoring lesson and not a code-review nicety: the gap is
INHERITED SILENTLY.** A downstream consumer reads the validator as its guarantee
and gets five laws while counting twelve. Nothing reddens, because every law is
*stated*, the checker *is* fail-closed on the paths that do fire, and no test
anywhere measures which arm caught what. **Same family as §2's MEASURED /
CLAIMED / THE GAP, one layer down: the checker's own advertised surface is a
claim that needs its own evidence.**

## 7. When a pin is defeated repeatedly, ask what the defeats SHARE

Two defeats mean stop patching forms and look at the mechanism's structure.
**Then diagnose before redesigning:**

- shared **granularity** error (lines where the language has tokens) ⇒ one
  change fixes the whole class;
- shared **default** direction (unknown ⇒ pass) ⇒ make undetermined fail;
- shared **scope** error (scanned the wrong surface) ⇒ re-derive the surface.

⭐ **The FIX RATE is the tell, and it is available before you have a theory:
when each correct fix exposes the next surface, your instrument is a
hand-enumerated list standing in for a population.** Three review rounds found
claim-bearing prose that the author's own sweeps had missed — one of them **four
lines below** text just rewritten — because each round fixed the sentence it was
pointed at and re-ran a needle list (`cannot reach`, `closed inventory`,
`load-bearing`), which kept coming back clean on prose that was still wrong: the
next one was spelled `cannot drift`, `one route`, `what is closed is`.

⇒ **When the population is "every sentence that makes a claim," the instrument is
READING them and classifying each — not grepping for phrasings.** A
prose-honesty AC cannot be discharged by a phrase sweep, and a frame must not ask
a build seat to discharge one that way. ⚠ This is the *same* defect as a
spelling-keyed source scan, in a second substrate — there the grammar was Rust's,
here it is English's. **If you have written "the closure is reading them, not
grepping them" and then reached for the grep, that is a habit, and a habit is
what a promoted rule is for.**

⛔ **A defeat count NEVER licenses the conclusion "this property cannot be
mechanically enforced."** That is a strong claim which *weakens a gate*, so it
must be **demonstrated** — by building the candidate mechanism and showing it
cannot work — never inferred from failure tallies. "My detector's granularity is
wrong" is cheap to test and common; try it first, every time.

## 8. Narrow honestly, and give the residual a cell

Some properties are global negatives over arbitrary code and **no test can
discharge them** without whole-program dataflow. When that is genuinely shown:

1. **Narrow to the statements a mechanism can enforce**, and list them.
2. **Record the residual explicitly** — what is review-enforced rather than
   mechanically guarded — **in the source, next to the enforced statements**, so
   the next reader inherits the limit instead of the overclaim.
3. **Name every residual arm**, not the first one you thought of.
4. ⛔ **Do not claim the residual is detected.** A narrowing that admits its
   boundary is a truthful gate; one that quietly keeps the old wording is a
   waiver wearing a pin's clothing.

⚠ **A taxonomy with no cell for the honest answer reads as complete.** If your
AC list has nowhere to record *"guarded by review, not by CI,"* it will be
recorded as *"guarded."*

## 9. The pin's NAME is part of its claim

A test named for an inference it does not prove propagates the overclaim —
because the name is the part future readers quote. **Rename the pin to what it
actually establishes**, and never leave a corrected body under an uncorrected
name.

## 10. Mutation hygiene

⛔ **A verdict from a mutation that did not apply is not evidence — and it looks
exactly like evidence.** The run compiles, the test executes, the output is
well-formed. **"The mutation ran" and "the mutation changed the subject, and only
the subject" are different claims, and only the second licenses a verdict.**

**One campaign of eight produced four distinct invalid kinds (2026-07-25) —
report each, never discard it:**

| kind | why its outcome is not evidence |
|---|---|
| **never applied** (bad anchor) | the test ran against pristine source |
| **broke the build** | nothing was measured |
| **inserted a comment** | the pin had nothing to catch, so GREEN is vacuous |
| ⛔ **edited the DETECTOR along with its subject** | the oracle was rewritten to match its rewritten subject and reported success **on a sound pin** |

That last one is the dangerous shape: it **would have filed a spurious finding
against a correct detector.** The mutation respaced *every* occurrence of its
needle — including the copy inside the test's own string literal. Re-run against
declaration lines only, it reddened.

⇒ **Every mutation needs a provenance check before its outcome is recorded:**
assert the occurrence count **before** mutating; and where the result is **green**,
prove the change reached the compiled artifact (symbol present, binary mtime after
the edit). **Reporting the invalid rows is what makes the valid ones
trustworthy** — a campaign that silently drops them is indistinguishable from one
that had no failures.

### ⛔ And the provenance check itself fails in BOTH directions

**Measured 2026-07-25, on the very next candidate after the rows above.** A
provenance check reported `applied=False` for two mutations that **had** landed:
it re-counted the anchor **after** the replacement, and **the replacement string
contained the anchor.** The campaign would have discarded **two sound results as
unproven.**

⚠ **An instrument that certifies evidence can fail toward throwing good evidence
away, and that is exactly as corrupting as passing bad evidence through** — it is
merely quieter, because a discarded row leaves no trace to audit. The rule above,
read only in the false-positive direction (*"did it change the subject, and only
the subject"*), does not catch this.

⇒ **Count the anchor BEFORE the edit and compare against a PREDICTED
post-count** — do not re-match a needle the replacement may still contain. A
provenance check is itself a pin, so §6's positive control and §6a's
which-arm-fired question apply to it: **feed it a mutation you know landed and a
mutation you know did not, and confirm it distinguishes them.**

- Apply each mutation at its **natural production site**, not at a convenient
  one; a mutation the real code path never reaches proves nothing.
- **Restore byte-identically** and verify with `git diff --quiet`.
  ⚠ `git diff --stat` **always exits 0** and is not an emptiness test.
- **Commit the real fix before any mutation-proof reset.**
- When a resource cliff (stack, RSS, timeout) fires, **measure the base's
  MARGIN**, not just pass/fail — attribution needs the margin. And **fixing a
  cliff by raising a limit spends a detector**: name which one, and where its
  replacement belongs.

## 11. Reproduction recipes

If a pin rests on captured constants, a re-capture after the change would
produce byte-identical values — **so nothing distinguishes a genuine baseline
from a re-recording.** Record the base SHA, the probe names, and the exact
worktree + invocation, and ⛔ **specify the sanctioned invocation verbatim**
(`scripts/ken-cargo`, targeted) or the recipe will document a procedure the
fleet is not allowed to run. **Demonstrate the binding; do not testify to it.**
