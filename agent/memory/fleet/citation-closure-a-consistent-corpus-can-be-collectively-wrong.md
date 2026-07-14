---
scope: fleet
audience: (see scope README) — anyone writing or reviewing a claim about the kernel
source: primitive-Op honesty sweep, 2026-07-14 (CV's §10 carry, promoted verbatim in substance)
---

# Citation closure: a corpus can be internally consistent and collectively wrong

For most of Ken's life the spec said a **primitive operation reduces under
kernel conversion** — that `byteLength "abc" ≡ 3` holds *definitionally*, and
`Refl` can close it. **It was false the whole time.** `conv::whnf` does δ/β/ι
and **has no `Op` arm**; the value is produced by `ken-interp::prim_reduce`, at
run time, **opaque to conversion.**

It survived across **fifteen spec documents and four conformance seeds** — and
it survived *because* it was consistent. Each document cited another document.
**None of them cited the executable authority.** That is **citation closure**:
a claim that is locally verifiable everywhere and grounded nowhere.

## ★ The two things that looked like evidence and were the opposite

1. **Local consistency was anti-evidence.** Fifteen documents agreeing did not
   make the claim fifteen times more likely — they were **one claim, copied**.
   *Repetition across layers should* **raise** *your suspicion, not lower it:
   the more places a claim appears, the more likely nobody re-derived it.*
2. **Green runtime examples were anti-evidence.** Every example "worked" —
   because the **interpreter really does return `3`**. The examples confirmed a
   true statement about `ken-interp` and were read as confirming a false one
   about the **kernel**. **A green test at the wrong layer is not weak evidence;
   it is misleading evidence.**

## The gate — run it on any claim containing these words

> **"kernel" · "trusted" · "definitional" · "reduces" · "closes by `Refl`"**

The author must name, concretely:

1. **the exact executable CONSUMER** (which function acts on this?);
2. **the exact value PRODUCER** (which function emits it?);
3. **a two-axis discriminator run THROUGH the claimed layer** — *not hand-fed
   across layers.*

Here that means: **the interpreter returns `5` for `add_int 2 3`, AND — the
axis that was never run — the kernel independently leaves the application
NEUTRAL and rejects the proposed `Refl`.** Checked-literal equality
(`PrimReduction::Literal`, ADR-0013) supplies the **positive control**, which is
what keeps the correction from over-shooting into a second falsehood.

**A grep only SELECTS candidates. The gate is tracing emission/consumption and
executing both orientations.** Sibling of
[[kernel-backed-claim-grep-the-emission-not-the-name]] and
[[mechanism-citation-needs-own-empirical-probe]].

**Had this gate existed, `14`'s first `Op`-definitional sentence could not have
landed without a real `conv::whnf` witness, and the posture would have died at
its origin instead of spreading to nineteen files.**

## ★ And when the finding outgrows your assignment: STOP AT THE SCOPE LINE

The same sweep found the defect **outside the assigned file set**. The right
move is **not** to keep fixing (silent expansion changes the land-together set,
the ownership, and the review topology) and **not** to ignore it. It is:

- **freeze edits;**
- **enumerate the exact loci + the common producer;**
- **state which cases are PROVEN the same and which are only SAME-SHAPED**
  (that distinction is the whole ballgame — `Literal` and `Op` are same-shaped
  and *not* the same);
- **ask the scope owner for an atomic-or-split ruling, and continue only after
  it.**

*"Obviously the same root cause" does not grant mutation authority.* **This is
playbook-able, not mystique** (CV's words, and he is right).

Sibling of [[conformance-reconcile-inherits-spec-metatheory-bugs]] and
[[primitive-ops-do-not-reduce-under-conversion]] (the fact this lesson is about).
