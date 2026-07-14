---
scope: fleet
audience: (see scope README) — anyone handed a research report, audit, threat
  model, or design doc that ends in a "Risks / failure modes" section; anyone
  regrounding a stale document against a moved tree
source: the POSIX/Linux ABI campaign, 2026-07-14 — the report's own §16 risk was
  already a live defect, in the security path, in the repo the report inspected
---

# A risk register is a GREP LIST, not a forecast

The POSIX/Linux gap report ended with *"Risks and likely failure modes."* Entry
three:

> **"ABI facts drift silently.** Hard-coded offsets or numbers may compile and
> corrupt memory. Bind every raw package to a generated manifest and test it
> against the actual target."

**It was already true.** In the same repository the report had just inspected,
`crates/ken-interp/src/eval.rs:2371-2394`:

```rust
#[cfg(unix)]
const O_NOFOLLOW_KEN: i32 = 0o400000;   // ← Linux values…
#[cfg(unix)]
const AT_REMOVEDIR_KEN: i32 = 0x200;    // …under a cfg that also selects macOS/BSD

#[cfg(unix)]
unsafe extern "C" { fn openat(..); fn unlinkat(..); /* +3 */ }
```

And `O_NOFOLLOW` is not decoration — **it is the enforcement mechanism for
`SymlinkPolicy::NoFollow`**, an ADR-0017 capability-confinement property. **A
security guarantee resting on a number someone remembered.**

## ★ Why the report's own author missed it

**Because "risk" is written in the future tense, and future tense doesn't get
grepped.** The author was in *forecasting* mode — "here is what could go wrong
if you build this badly" — and never turned the sentence around into a *query*
against the tree they were standing in. The finding was three greps away.

> **⇒ Every entry in a risk register is a QUESTION, and the question is
> `is this already true?` — not `might this become true?`** Run the register
> against the tree **before** you accept it as a forecast. The entries were
> written by someone who understood the failure mode well enough to name it;
> that is exactly the person whose list is worth executing.

## ★★ The second half: a `cfg` gate broader than the fact it guards

`#[cfg(unix)]` compiles on Linux, macOS, and every BSD. **The values are
Linux's.** The gate and the fact **do not have the same extension** — so the
code compiles, links, and hands those bits to real syscalls on targets where
they mean something else, **and every test stays green, because the tests run on
the box where the numbers happen to be right.**

**Generalize it:** a conditional-compilation gate, a feature flag, a capability
check, a `match` arm — **whenever a guard is WIDER than the fact it protects,
the excess is silent and untested by construction.** Ask of every gate: *what is
the exact set this fact is true on, and is that the set I gated on?* If the gate
is `unix` and the fact is `linux`, **you have shipped an untested platform.**

## The rule

1. **Receive a risk register ⇒ execute it as a grep list, entry by entry**,
   against the current tree. Report which are already realized. **A regrounding
   pass is when you do this** — and it is the highest-yield thing about
   regrounding a stale document.
2. **A stale report is not a worthless report.** This one's "current state"
   section was 2/3 wrong, and its risk section found a live security defect.
   **Reground the facts; execute the risks.**
3. **Audit every gate for extension mismatch.** `cfg`, feature flag, or
   predicate — if it selects a wider set than the fact holds on, the difference
   is a hole nothing will ever exercise on your machine.

This is the [[never-pin-a-shape-that-cannot-state-its-own-contract]] family from
a new direction: the magic constant had **nowhere to state its obligation**, so
nothing could check it. Sibling of
[[deriving-from-the-contract-cannot-detect-a-defective-contract]] — **again the
gates were all green and were measuring something else.**
