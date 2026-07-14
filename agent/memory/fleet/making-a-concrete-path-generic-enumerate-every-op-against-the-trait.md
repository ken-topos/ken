---
scope: fleet
audience: (see scope README) — WP authors especially; reviewers second
source: I-6 hard-stop, 2026-07-14 — the SECOND failure of the same audit
---

# Making a concrete path generic: enumerate EVERY op against the interface

When a WP proposes to take an **existing concrete code path** and make it
**generic over a trait**, the question is **not** *"is the trait public?"* It is:

> **Is EVERY step of the path I am replacing expressible through the trait?**

**The tell is precise and greppable:** the concrete type's **inherent methods**
that the call path uses, **which the trait does not declare**. Those are exactly
the operations a generic runner **cannot call** — and they are invisible if you
only check that the trait and the concrete types are `pub`.

## What it cost (twice)

**I-6** framed a generic `run_program<H: HostHandler>` after verifying that
`run_io<H>`, `HostHandler`, and `CaptureHost` were all public and re-exported.
All true — and the frame was still unbuildable. The runner mints the program's
capability via **`PosixHost::mint_fs_cap`**, an **inherent** method
(`eval.rs:2416`), with `CaptureHost` carrying its **own separate inherent** copy
(`eval.rs:3058`). **`HostHandler` has no mint operation at all.** So the generic
runner could not perform the one step the concrete runner performs. One line,
whole WP.

**This was the SECOND time the same audit failed.** I-5 failed it the first way
(the seam could not *carry* the value), which is why audit **(b′) SEAM/ABI** was
added to the Steward playbook. **I-6 failed it the second way** — the seam
existed, was public, and was *incomplete* — because I checked the seam the
design **named** and never enumerated the ones it **didn't**.

## The mechanical check

Before pinning "make path P generic over trait T":

```
# every method the path calls on the concrete type…
rg 'concrete_host\.\w+\(' <the path>
# …minus every method T declares. The difference is the gap.
rg '^\s*fn \w+' <the trait definition>
```

**Anything in the difference is a blocker, not a detail.** Either it moves onto
the trait (and that is a real, reviewable ABI change with a soundness question
attached), or the WP cannot be built as framed.

## The resolution shape (when it *is* the right move)

I-6's gap resolved by **lifting `mint_fs_cap` onto the trait** — sound because
**a `HostHandler` impl already IS the trusted boundary** (it implements
`fs_resolve` and the whole `fs_*_at` syscall family, so it can already perform
any FS op). **A capability is a strictly weaker, *gating* token** over what the
*program* may request; **authority to mint it is subsumed by authority to perform
the ops it gates.** Keep such a method **`&self`** — both impls root the cap in
the **host's own identity**, and that binding is precisely what a caller-supplied
cap (or a free-function `Cap::mint`) would destroy. See ADR-0017 §4a: I-5's step
0 already moved `fs_resolve`/`fs_*_at` onto the trait for the same reason, and
left `mint_fs_cap` behind only because the mint call was not yet parameterized.

Sibling of [[surface-the-seam-need-not-your-preferred-mechanism]] and the
Architect's carry: *"I grounded the axis I was focused on… but I stopped one
layer shallow."* **The layer you are not looking at is the one that breaks.**
