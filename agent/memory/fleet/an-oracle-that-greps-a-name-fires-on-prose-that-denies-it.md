---
scope: fleet
audience: (see scope README) — anyone writing a "zero-X" acceptance oracle, or debugging one
source: SUB-1 CI red, 2026-07-14 — and the Steward made the identical mistake 40 minutes earlier
---

# An oracle that greps a NAME will fire on the prose that DENIES the name

DS-4 shipped this gate:

```rust
// Zero-Axiom acceptance bar over the whole file (a plain `.ken` source —
// no literate extractor; the entire file is checked code).
#[test]
fn zero_axiom_in_collections_ken() {
    assert!(!COLLECTIONS_KEN_MD.contains("Axiom"),
            "Collections.ken must contain zero Axiom literals");
}
```

**SUB-1 added two lines of PROSE to `Collections.ken.md`:**

> *"…an ordinary fold through `bytes_to_list` with no cached length or local
> **`Axiom`**."*

**The prose asserts there is NO `Axiom`. The oracle read it as an `Axiom`.** CI
went red on a WP whose entire achievement was *removing* the `Axiom` tax.

## Two independent defects, and the comment confesses one of them

1. **It greps a NAME, not an EMISSION.** *A grep SELECTS candidates; it never
   DECIDES.* The check it wanted is **"does the checked Ken code DECLARE an
   `Axiom`?"** — a question about **declarations**, which the word "Axiom"
   appearing in a sentence does not answer.
2. **It reads a literate file as if it were plain code.** The comment says *"a
   plain `.ken` source — no literate extractor; the entire file is checked
   code."* **The constant is `COLLECTIONS_KEN_MD`; the file is
   `Collections.ken.MD`.** It is **prose + code fences**
   ([[catalog-sources-are-literate-ken-md-not-ken]]). **The oracle's own comment
   is factually wrong about the file it reads** — and it passed for months only
   because no prose in that file had *happened* to use the word.

## The fix — extract first, then assert. It STRENGTHENS the gate.

```rust
let extracted = ken_elaborator::literate::extract_ken_md(COLLECTIONS_KEN_MD)
    .expect("must extract");
assert!(!extracted.contains("Axiom"), "…code must contain zero Axiom literals");
```

Still fails on a real `Axiom` in a fence (all it ever meant to catch); **stops
false-firing on prose that discusses `Axiom`s.**

**⛔ Do NOT "fix" this by deleting the assertion, or by rewording the prose to
dodge the grep.** The prose is *correct and valuable* — it documents the
zero-TCB property. **The oracle is the thing that is wrong.** Rewording the
document to appease a broken test is how you end up unable to *write down* the
property you just achieved.

## ★ This class is everywhere, and it bites the author of the rule too

**The same hour**, the Steward's own honesty gate screamed `!!! TCB GROWTH !!!`
at I-7 — because the diff contained **the WP frame**, whose §1 is two pages
*forbidding* `Axiom`/`Clock`/`lookup_env`, **and an acceptance test with a
negative-assertion list** `for forbidden in ["Axiom", "lookup_env", "Clock", …]`.
**The tokens were there because the test asserts they must NOT appear.** A clean
WP was nearly blocked by a guardrail its own author had written.

**A "zero-X" check must look at what the artifact DECLARES, never at whether the
string `X` occurs somewhere in it.** Prohibitions, negative assertions, doc
comments, and frames all *name the thing they forbid* — that is what forbidding
looks like.

## ★★ THE COUNTERMAND — the reflex this bug provokes is the real danger

**The first fix proposed was: *"rephrase the prose so `rg -n Axiom` is empty."***
The Steward countermanded it in flight. **Never reword a document to appease a
broken test.** Here, that fix would have:

1. **Deleted the record of the property the WP existed to establish** — and that
   exact sentence was **what the Architect's approval keyed on** (*"my AC3 audit
   read those exact two prose lines and verified them as a true zero-Axiom
   assertion"*). **You would be editing out the text the approval was granted
   on.**
2. **Left the trap armed** for the next author who documents a zero-`Axiom`
   property — **teaching the corpus that the word is unsayable**, which is
   absurd for a project whose thesis is *say the guarantee out loud*.
3. **Left the oracle's false comment in the suite.**

**Fix the oracle. Never the truth.** *A codebase that edits its documents until
the machine stops complaining ends up unable to state what it has achieved.*

## The census — the raw-grep shape, and where it hides

A sweep of `crates/*/tests/*.rs` for `contains("Axiom")` (2026-07-14):

| Shape | Sites | Verdict |
|---|---|---|
| `extract_ken_md(…).source.contains("Axiom")` | CC1–CC7, DS-2/3/7/8, … (most) | ✅ **correct** — asserts on extracted Ken |
| `RAW_INCLUDE_STR.contains("Axiom")` | `ds4_list_combinators_acceptance.rs:56` | 🔴 **fired** (SUB-1) |
| `RAW_INCLUDE_STR.contains("Axiom")` | `i2_console_floor.rs:132`, `i3_fs_floor.rs:141` | ⏳ **latent** — pass only because no prose in `Console.ken.md` / `FS.ken.md` has *yet* used the word |

**The latent pair is the lesson.** They are the tests **nearest** to the next
effect WP (I-8's clock) — **the ones an implementer would naturally crib the
zero-`Axiom` idiom from.** *That is how the trap propagates: not by being
written twice, but by being copied from the nearest neighbor.* **When you fix
one instance of a bad oracle idiom, sweep for its siblings and fix them in the
same breath** — I-8 carries their repair as a mandated Step 0.

Sibling of [[kernel-backed-claim-grep-the-emission-not-the-name]],
[[adding-a-file-to-a-globbed-corpus-trips-oracles-you-did-not-enumerate]], and
[[corpus-property-gate-only-as-strong-as-the-corpus]].

## ★★ The COUNTING variant — and it is the one that reports a false BREACH

Everything above is about a **boolean** gate (*does `X` appear?*). The same defect
in a **numeric** gate is nastier, because the output is a plausible number rather
than an obviously-wrong flag.

**SUB-1b, 2026-07-14.** The Steward's trust-delta gate counted the symbol
`declare_postulate` across `crates/**` before and after:

```
2c184550 : 213        55cc3941 : 215        ⇒ "+2 postulates" — a SCOPE BREACH
```

The WP authorized **exactly one**. **It was clean.** The second "postulate" was:

```rust
use ken_kernel::{declare_postulate, declare_primitive, GlobalEnv, GlobalId, Term};
```

**An import is not a call site.** Counting the *mechanism* — `declare_postulate\s*\(`
— gives **125 → 126. Exactly one.**

**⇒ A count over a bare symbol name counts imports, `use` lists, re-exports, doc
comments, negative assertions, and the WP frame's own prohibition text.** The
boolean version of this bug **false-passes** (a gate that never fires). **The
counting version FALSE-FAILS — it accuses a clean WP of a trust breach**, and a
trust breach is exactly the accusation everyone downstream will act on
immediately.

**Count the mechanism, never the name:** the call (`name(`), the declaration
(`^\s*fn name\b`), the emission. **And when a trust-delta gate reports a breach,
your FIRST move is to print the actual added lines** — not to escalate. The
number is a candidate. **It is never the decision.**

*The real gate here was in the code all along, and it was right:*
`if actual_delta != expected_delta { return Err(…) }` — **a fail-closed
set-equality assertion on the postulate IDs, comparing what the kernel actually
registered.** That is what a trust-delta check looks like when it interrogates the
mechanism instead of the text.

## And the meta-lesson: this is the targeted-tests rule working, not failing

**Three lanes approved SUB-1 — QA, CV, Architect terminal — and every one of them
was right about everything they checked.** None ran DS-4; they *couldn't* (local
tests are **targeted-only**, `COORDINATION.md §12`, and DS-4 is not in SUB-1's
blast radius by any obvious reading). **CI's full gate exists precisely so that a
change to a corpus file trips every oracle that reads it — including one nobody
remembered existed.** Don't respond to this by running more locally. Respond by
**expecting the globbed-oracle surprise**, and by **not writing name-greps.**
