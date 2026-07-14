---
scope: fleet
audience: (see scope README) — anyone who must claim "there are N; here are all
  N": frame authors, D0 grounding audits, QA inventories, security audits
source: KTR-1 AC4 + its 91-site replay, 2026-07-14 — the Steward's own frame and then
  his own production audit; caught by the Architect, then by kernel-implementer;
  FOUR instances in one day — the 4th while knowing and having just written the fix
---

# An enumeration needs a **proven closure**, not a better grep

**We already knew: *"a grep SELECTS candidates; it never DECIDES, and it never
COUNTS."*** That rule is **true and insufficient** — it tells you what *not* to
trust and leaves you with no way to be right. **This is the missing half.**

## What happened

KTR-1 repairs a missing kernel admission gate, so its AC4 demanded an inventory
of **every inductive declaration in the repo**. The Steward wrote the AC, and —
knowing the trap — put a warning in it, in capitals:

> *"⛔ GREPPING `data` IN `.ken` SOURCES WILL MISS THE PRELUDE. The prelude's
> inductives are EMITTED FROM RUST."*

**And then named the prelude as *the* Rust producer, and stopped.** The real
production producers were **four**, and the one he named was not the biggest:

| producer | sites | in the AC? |
|---|---|---|
| `ken-interp/src/lib.rs` | **8** | ❌ |
| `ken-elaborator/src/prelude.rs` | 5 | ✅ *(the only one)* |
| `ken-elaborator/src/effects/state.rs` | **3** | ❌ |
| `ken-elaborator/src/data.rs` | 2 | ✅ implicitly |

> **★ He corrected for the wrong LANGUAGE and then inherited the wrong
> CATEGORY.** He knew the enumeration had to move from `.ken` to Rust — **and let
> ONE EXAMPLE of a Rust producer stand in for THE EXTENT OF THE KIND.**
>
> **This is the same error as PX0's `:2370`-vs-`:2355`** *(reading from a line a
> citation pointed at, rather than from where the kind begins)* — **and that error
> was cited as a warning, in capitals, two paragraphs above the mistake, in the
> same document.** *Knowing the lesson did not prevent it. That is the whole
> reason this memory exists.*

## The fix — what the Architect did that the Steward did not

He did **not** produce a better grep. He produced a **closure argument**:

```
git grep '[^[:alnum:]_]declare_inductive(' -- '*.rs'   →  89 call sites, 28 files
git grep 'add_decl(Decl::Inductive'        -- '*.rs'   →  ONE hit: check.rs:953
                                                          …INSIDE declare_inductive
```

> **There is exactly ONE raw insertion path into the environment, and it lives
> inside `declare_inductive`.** Therefore **every** inductive that reaches the
> kernel **must** pass through it — so enumerating its call sites is **complete by
> construction**, not thorough-by-effort.

**That second grep is the whole trick.** It does not find declarations. **It
proves that nothing can get in another way.** Without it, `89` is just a bigger
number than `5` and equally unjustified.

## ★★ THE FOURTH INSTANCE — and it reveals the rule was still too weak

**Same day, same Steward, one hour later, *knowing all of the above*, and having
just written it down.** Asked "which PRODUCTION code declares inductives?", he:

1. grepped for **the idiom** (`Term::ty(Level::Zero)`) → got a list of 38 files;
2. asked **which of THOSE** files build a `CtorSpec`;
3. answered **"exactly two: `data.rs` and `prelude.rs`."**

**There are three.** `effects/state.rs` calls `declare_inductive` three times from
`register_prelude` — **unconditionally, in production.** It never appeared,
because **it does not contain the idiom**, so it never survived step 1.

> ***He defined the closed set by the SYMPTOM instead of by the GATE.***
>
> **A symptom-derived set silently omits every member that has the gate without
> the smell.** And the omission is invisible: he *did* run a closure argument —
> **on the wrong universe.** The set was already wrong before the reasoning
> started. **It came back clean.**

**The tell he missed:** his candidate list came from a grep for *the thing he was
looking for*. **If your enumeration starts by searching for the defect, your
population is the defect — and you can never find a member that lacks it.**

## The rule

**Before you claim "there are N; here are all N," answer a DIFFERENT question
first: *what is the narrowest gate every member of this kind MUST pass
through, and how do I know nothing bypasses it?***

> **★ AND THE POPULATION MUST BE DEFINED BY THAT GATE — NEVER BY THE PROPERTY
> YOU ARE TESTING FOR.** *Enumerate at `declare_inductive` (the gate), then apply
> the `Δₖ`-sort predicate (the property) to each. **Never** collect the files that
> smell of the property and then look for the gate inside them.* **Population from
> the gate; verdict from the property. Reversing them is undetectable.**

1. **Find the choke point** — the single constructor, the sole insertion path,
   the one admission function, the unique writer.
2. **PROVE it is the only one.** Grep for the *bypass*, not the *instances*:
   the raw `add_decl`, the direct field write, the `unsafe` construction, the
   `impl` that skips the builder. **A closure argument is a grep whose EMPTY (or
   singleton, and accounted-for) result is the evidence.**
3. **Only then enumerate at that gate**, and report the count.

**The two greps have opposite jobs and you need both:**

| grep | finds | its job |
|---|---|---|
| **instances** (`declare_inductive(`) | the members | gives you N |
| **bypasses** (`add_decl(Decl::Inductive`) | **the holes** | **makes N MEAN something** |

**⇒ Naming a producer is not enumerating a kind. Ask what makes your list
CLOSED — and if you cannot answer, you do not have an inventory, you have a
sample.** *"I named a place. He found the closure."*

## Where this bites hardest

**Any claim of the form "we checked all of X."** Security audits (every FFI
boundary, every `unsafe`, every capability check), migration sweeps (every call
site), trust-root gates (every declaration), corpus oracles (every file the glob
reaches). **In all of them the failure is silent and reads as success**: an
incomplete sweep comes back **clean**, and clean is exactly what you were hoping
for.

**And it is load-bearing downstream.** KTR-1's inventory feeds the open question
*"did any existing certificate depend on the missing gate?"* — **that question
cannot be answered against an environment enumerated from two of four
producers.** A bad inventory does not merely under-report; **it silently
invalidates every conclusion drawn on top of it.**

Sibling of [[grep-the-producer-not-the-cited-proxy]] (there: verify a *value*
against its true producer; here: enumerate a *kind* against its true closure),
[[named-floor-must-be-grepped-not-assumed]], and
[[a-risk-register-is-a-grep-list-not-a-forecast]].
</content>
