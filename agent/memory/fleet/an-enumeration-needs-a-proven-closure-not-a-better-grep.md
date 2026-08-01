---
scope: fleet
audience: (see scope README) — anyone who must claim "there are N; here are all
  N": frame authors, D0 grounding audits, QA inventories, security audits
source: KTR-1 AC4 + its 91-site replay, 2026-07-14 — the Steward's own frame and then
  his own production audit; caught by the Architect, then by kernel-implementer;
  FOUR instances in one day — the 4th while knowing and having just written the fix.
  Fifth: runtime-implementer on RT-FNSPLIT-B2V, 2026-07-26, self-reported. Sixth:
  SPEC-STORE-SPLIT, 2026-07-27 — a property-removal census closed over subject
  NAMES but not over the generic predicates the subject inhabits; all four enclave
  seats derived the same rule independently in their retros, which is why it
  promoted on a single WP.
---

# An enumeration needs a **proven closure**, not a better grep

**We already knew: *"a grep SELECTS candidates; it never DECIDES, and it never
COUNTS."*** That rule is **true and insufficient** — it tells you what *not* to
trust and leaves you with no way to be right. **This is the missing half.**

## What happened

KTR-1 repairs a missing kernel admission gate, so its AC4 demanded an inventory
of **every inductive declaration in the repo**. The Steward wrote the AC, and —
knowing the trap — put a warning in it, in capitals:

> *" GREPPING `data` IN `.ken` SOURCES WILL MISS THE PRELUDE. The prelude's
> inductives are EMITTED FROM RUST."*

**And then named the prelude as *the* Rust producer, and stopped.** The real
production producers were **four**, and the one he named was not the biggest:

| producer | sites | in the AC? |
|---|---|---|
| `ken-interp/src/lib.rs` | **8** | |
| `ken-elaborator/src/prelude.rs` | 5 | *(the only one)* |
| `ken-elaborator/src/effects/state.rs` | **3** | |
| `ken-elaborator/src/data.rs` | 2 | implicitly |

> ** He corrected for the wrong LANGUAGE and then inherited the wrong
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

## THE FOURTH INSTANCE — and it reveals the rule was still too weak

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

> ** AND THE POPULATION MUST BE DEFINED BY THAT GATE — NEVER BY THE PROPERTY
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

## THE SIXTH INSTANCE — the population is defined by a PREDICATE

**…that your subject INHABITS, so its name appears nowhere in the carrier.**

**Measured 2026-07-27, `SPEC-STORE-SPLIT`, and all four enclave seats derived the
same rule independently in their own retros** (spec-author `evt_3kn9strac8n6y`,
CV `evt_6dyfxxa43xztf`, Architect `evt_2vszbkngvgj4d`, spec-leader
`evt_6jfvdjf00tc9d`). That convergence is why it promotes on one WP.

**The task was a property REMOVAL:** Map/Set byte canonicity stops being a
promise. The first fold repaired every carrier that **said "Map" or "Set"** — six
of them, correctly. **And the retired promise survived intact**, because
`spec/` also states the same property over **generic nouns**: *"closure-free
value"*, *"admitted value"*, *"any live value"*.

**The membership fact is the whole lesson: `Map`/`Set` ⊂ closure-free durably
transportable values.** So every generic byte clause **still applied to them** —
`36-effects §4.4`, `42-evaluation §3.1/§3.4/§3.7`, `44-capacity §1/§3`,
`OQ-Space`. CV blocked on those four. The Architect's independent pass then found
**four more of the same shape** — `41 §5`, all of `44-capacity`, `OQ-gc`'s
reclamation clause, and a capacity conformance seed whose byte-comparison
witnesses were not domain-qualified. The author's own final sweep found a ninth
(`minimality §D`) that neither reviewer had routed.

⇒ **A SUBJECT-NAME CENSUS IS NOT A PROPERTY CENSUS.** This is the same defect
as the sections above with the *direction* reversed: there, the population was
reached through **producers** you did not name; here, through **quantifiers**
whose domain silently contains your subject. In both, the grep was honest about
the bytes it received.

### THE TELL, AND IT IS THE MOST ACTIONABLE THING HERE

> **Repeated population growth across review passes is evidence the ENUMERATION
> METHOD is wrong — not that the list is longer.**

Three passes each found *more* sites. The instinct at pass two is *"one more
missed path"*, and it is wrong every time: a method that under-counts by
construction under-counts again. **When a second pass expands the population,
stop adding paths and change how you enumerate.** All four seats named this
independently, which is the strongest signal in this file.

### The artifact this demands

**Two columns, built BEFORE drafting, not after a block:**

| column | how you get it | what it proves |
|---|---|---|
| **named carriers** | grep the subject / the property's spelling | candidates only |
| **entailed carriers** | derive the subject's **supertypes and categories**, then sweep every **generic quantifier** over each | closure |

Then **classify every consumer** — retained / narrowed / historical / rejected —
across normative text, producers, cross-case prose, summaries, open-decision
registers, and deliverables. ⇒ **The closure argument is the membership
derivation, not the search.** *"Grep selects candidates; it cannot prove the
population is complete."*

**And keep positive homonym controls, or you will over-correct.** Record
declaration-order encoding and `Cat4 keys-ascending-off-tolistordered` sit
textually *next to* the removed property, mention ordering and bytes, and are
**different live properties**. Both were held blob-identical through every fold
and named as controls in every vote. **A removal sweep needs a positive
control exactly as much as a detector does** — without one, "removed the retired
mechanism" and "removed the neighbours too" read identically.

## THE FIFTH INSTANCE — TWO searches, each EXHAUSTIVE, neither complete

**Measured 2026-07-26, `runtime-implementer` on `RT-FNSPLIT-B2V`, reported against
itself.** The rule above was already in the corpus. This instance matters because
**nothing about it looks like a lazy grep** — and it names the operational form of
*"population from the gate."*

The task was to inventory the sites consuming a representation authority. The
located list missed **two**:

| missed site | why the search could not see it |
|---|---|
| `:1194` | **same defect, but no constant to grep for** — the search keyed on a *name* the defect happened to spell at the other sites |
| `:715` | not a `NODE_CLASS`, so it fell **outside the fold's private notion of "class"** |

⇒ **Two searches, each exhaustive within a boundary neither search wrote down.**
Not carelessness — each was *complete* against the domain its author had in mind.
The domains were never stated, so they were never checked.

**The operational rule, and it is one keystroke different from the failing one:**

```console
grep 'BoundaryTag::'      # ✅ uses of the AUTHORITY  -> found the missing site instantly
grep 'FIRST_HANDLE_TAG'   # ⛔ occurrences of the DEFECT'S NAME -> never could
```

**Enumerate uses of the AUTHORITY, not occurrences of the DEFECT'S NAME.** This is
*"population from the gate, verdict from the property"* made concrete: the
authority's type/module path **is** the gate, and it is greppable. The defect's
name is a property of the sites you already found — so keying on it can only
re-find them. ⇒ **When you catch yourself grepping the string that appears in the
bug you just fixed, you are enumerating from the property.**

**And write the domain down.** The cheap fix for two unstated boundaries is one
sentence per search saying what population it claims to cover. An unstated domain
cannot be wrong, which is exactly why it cannot be checked.

**A control that fires on its own author is the cheapest evidence it is real:**
the source scan built for this inventory **matched its own needle literal** on the
first run, and was caught by its own *undetermined-parse ⇒ fail* branch. That is
the fail-closed branch working, on the person who wrote it, before anyone else saw
it. ⇒ **A scanner that searches source it is itself part of must exclude itself,
and the exclusion is a case worth a control** — the related trap is an assertion
whose needle is **caller-supplied**, which passes by construction because the
caller hands in the very string being sought.

⇒ Per-site coverage is downstream of this: see
[[a-differential-over-an-aggregate-is-an-existential-not-a-universal]]. **You
cannot per-site anything until the site count is closed** — an existential pin and
an unstated enumeration domain hide each other, because the pin stays green while
the inventory stays short.

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
