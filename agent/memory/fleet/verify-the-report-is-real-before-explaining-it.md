---
scope: fleet
audience: (see scope README) — anyone handed a failure report; anyone about to
  explain a surprising measurement; every reviewer and every Steward
source: LET-2 AC1 "guide strands don't parse", 2026-07-14 — the Steward built a
  confident mechanism for a measurement that never existed
---

# Verify the REPORT is real before you build a mechanism that explains it

**CV reported:** all three `catalog/guide/**` strands fail with
`ParseError: found Ident("program")` at `decomposition:163`, `proof-techniques:31`,
`surface-reference:44`.

**The Steward explained it, and every grep was TRUE:**

- `lexer.rs:451` — `"program" => Token::KwProgram`, **unconditionally**
- `parser.rs:223` — `Token::KwProgram => self.parse_boundary_decl(...)`, **inside
  `parse_decls`**
- ⇒ the grammar **accepts** `program`, so a `program` arriving as an **`Ident`**
  cannot be at a declaration boundary
- ⇒ **the parser must have been mid-expression** — an unterminated declaration
  upstream
- ⇒ *"the header is the crime scene, not the crime"*

**Elegant. Internally consistent. Every premise verified at the emission. And
completely wrong** — because **the observation was never real.** CV had invoked
`scripts/ken-cargo` from the **wrong worktree** and measured pre-candidate
content. Re-run properly (one file, at the ref, `pwd` and `HEAD` printed at the
point of invocation): **exit 0.** There was no `ParseError`. There was no
unterminated declaration.

## ★ The rule

> **When a report and a mechanism disagree, verify the REPORT is real before you
> build a mechanism that reconciles them.**

**A confident, internally-consistent explanation of a MEASUREMENT ERROR is worse
than no explanation.** It **launders the error into a finding**. The next reader
inherits it as established fact, with grep-citations attached. *It nearly went
into the tracker as the settled cause.*

## The tell you have already crossed the line

**You are reconciling two things that "should" both be true.** Here: *"the grammar
clearly accepts `program`"* **and** *"CV clearly sees it rejected."* **That tension
is the alarm.** Reaching for a clever bridge between them — *"ah, it must be
mid-expression!"* — **feels like insight and is actually the moment you stopped
testing the inputs.**

**The cheapest discriminating probe is almost always "does the reported failure
reproduce?"** — one file, at the ref, unmodified. **Run that BEFORE theorizing.**
Here it was one command and it would have ended the question instantly.

## The sibling: `pwd` and `HEAD` are PART OF THE MEASUREMENT

**A producer invoked from the wrong working directory returns a plausible, precise,
entirely wrong answer — and nothing in the output announces it.** The tool is
honest; the tree is the lie. Same family as
[[multi-worktree-cwd-drift-phantom-diff]] and a stale-base diff.

**So when you report a failure — and when you accept one — the report must carry
`pwd`, `HEAD`, and proof the file is unmodified at that ref.** CV's corrected
proof is the template:

```
detached worktree at /tmp/…      pwd printed
HEAD = 82cb8fd0…                 printed at the point of invocation
file diff to that SHA            EMPTY
fresh CARGO_TARGET_DIR           (no cross-tree cache)
ken check <the one file>         exit 0
```

**A validator that never blocks is useless; one that never retracts is dangerous.**
CV self-diagnosed and retracted within minutes — **that is the behavior to
copy.** Sibling of [[verify-the-guard-before-acting-on-it]]: *the guard fired, and
the guard was wrong* — and of
[[an-oracle-that-greps-a-name-fires-on-prose-that-denies-it]]: **the greps were all
true and the conclusion was still unsupported.**
