---
id: VIS-BR-LITERAL
title: "visibility walk: raw-string prefixes br and cr are unrecognized by the literal scanner"
status: merged
owner: runtime
size: XS
gate: none
depends_on: []
blocks: []
github: null
origin: adversary post-merge hunt on ORACLE-VIS-PACKAGING @ fad29eed (evt_7x40bh0sgavcg, 2026-07-22)
---

**A reproduced fail-closed bypass; unreachable on `main` today, cheap to fix.**
The adversary could **not** defeat the visibility walk itself — the
three-outcome design held against every attack, including nested comments
and `pub // see [note]`. This is a gap in the **scanner that feeds** the
walk.

## The defect

In `string_literal_end`, the guard that prevents `foo_r"x"` from being read as
a raw-string opener also rejects **byte raw strings**:

```rust
if bytes[at] == b'r' {
    if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_') {
        return None;                     // <- `b` is alphanumeric
    }
```

In `br#"…"#` the `r` is preceded by `b`, so the literal is not recognized and
is scanned as **ordinary text**.

## Severity — LIVE FAIL-CLOSED BYPASS, reproduced and compiling

**A valid-Rust false GREEN exists and has been executed** (`evt_220k1j1rjfdb7`).
A `pub` helper reads as module-private, and **both** of the walk's structural
defences are *satisfied* rather than broken:

```rust
const C: &[u8] = br#"a " b ; /*"#;
pub // */
fn build_process_starter_executable_artifact(x: u8) -> u8 { x }
```

`rustc --edition 2021 --crate-type lib` ⇒ compiles clean (only *"constant C is
never used"*). Driving `blank_comments` + `string_literal_end` lifted verbatim
from `fad29eed`:

```
walk sees:       'const C: &[u8] = br#"a " b ;'
ends with ';'    -> ModulePrivate  =>  GREEN
declaration name -> PRESENT, so the non-vacuity control passes too
```

**Why it works:** the `br` gap makes the `/*` inside the literal start a *real*
comment scan. The literal content ends `; ` immediately before it, so the
whitelist later sees `;` as a genuine item terminator. The scan is closed by
`*/` inside a **line** comment — `pub // */` — which is legal Rust.

⚠ **The `//` handling is NOT a second defect.** At `depth > 0` the scanner is
inside what it believes is a block comment, and in real Rust `//` has no
special meaning there while `*/` *does* terminate. Its behaviour is correct
**for a comment**; the single defect is that it is in that state at all.
⇒ **One condition to fix.**

### ⛔ Two corrections happened here — record both

1. **Mine.** I first wrote the consequence into this issue as established from a
   traced branch. `runtime-implementer` executed it and **could not reproduce
   it** — every construction failed closed.
2. **Theirs.** They stated the bound honestly (*"absence of my construction is
   not proof there isn't one"*), and the adversary executed that negative result
   and **completed the construction** — the missing piece being the
   line-comment terminator, which they had no reason to try.

★ **Neither party was right first, and both corrections came from running the
thing.** A mechanism story and a bounded search failed in the same way.

## Reachability — measured, re-checked after the construction

```
br literals in ken-runtime/src   -> none  (re-verified post-construction)
```

**Unreachable on `main` today** — exploiting it requires someone to add a `br`
literal. ⇒ **That bounds URGENCY, not SEVERITY.** This is a real hole in a
just-landed guard, not a scanner nit.

## ⛔ Ledger coupling — the changed file is a library-attested source

`crates/ken-cli/tests/px4b_native_production.rs` is listed in
`library/SOURCE-ATTESTATIONS` (cited by `library/learn/reading-ken/06-execution.md`
:172/261/288 and `solutions.md:127`, which claim it exists and "drives real
programs"). **Changing it makes `library/STATUS.md` stale**, so a crates-only
edit here trips the gen-doc-status currency gate in CI (`status_md_generation_
is_idempotent`, `library_documentation_gates.rs:892`) — invisible to merge-tree
/ touched-path checks. **The PR MUST include the re-attestation** (run
`gen-source-attestations.sh` + `gen-doc-status.sh`, commit the `library/` delta)
or merging turns `main` red. That companion `library/` delta needs **Librarian**
review (§8a); the Architect's `crates/` approval carries forward unchanged. This
surfaced on PR #908 (2026-07-23) — the Steward skipped the §7b ledger check.

## Scope

⛔ **The fix is `(b|c)?r`, NOT "permit `b` before `r`."** Rust's raw-string
prefixes are exactly **`r`, `br`, `cr`** (C raw strings stable since 1.77). The
identifier-boundary guard rejects `r` after *any* alphanumeric byte, so it
misses `br` **and** `cr`. `runtime-implementer` reproduced both end-to-end
(`evt_5trt38vfrp830`): `cr#"a " b ; /*"#` + `pub // */` compiles clean on rustc
1.96 and reads `ModulePrivate` for the same reason. `br##"…"##` (more hashes)
also. **A `b`-only patch leaves `cr` open** — and a future reader "finishing"
this against the original single-prefix report would ship exactly that gap.

1. Recognize the raw-string prefix `(b|c)?r` **at a token boundary** — parse the
   prefix, do not special-case one letter. Still reject a genuine identifier
   ending in `b`/`c`/`r` (all valid identifier characters), so the token-boundary
   check is load-bearing. ⛔ Do not simply drop the guard.
2. **Plain `b"…"` / `c"…"` (non-raw) are already handled correctly** — they do
   not start with `r`, so the guard is irrelevant and the ordinary `"` branch
   catches them. Verified (`evt_5trt38vfrp830`). Do not add machinery for them;
   just don't regress them.

## Acceptance

**The end-to-end proof IS constructible — the fixture is supplied above.** (An
earlier draft of this section said it was not, on the strength of the
implementer's bounded search. The adversary then built it. Assert end-to-end.)

- **End-to-end, the load-bearing AC:** the construction above returns a
  **non-`ModulePrivate` FAIL** post-fix. (The original draft predicted
  `Undetermined`; the *actual* post-fix verdict is **`Widened("pub")`** —
  once the raw literal is consumed correctly, `pub` survives unambiguously, so
  `Widened` is the precise fail-closed classification. Both are FAILs; the
  load-bearing property is the flip **away from the false `ModulePrivate`
  GREEN**, which the tests pin together with exclusion of `ModulePrivate`.
  Corrected post-merge per Architect `evt_3rbgjz940w8nc`.)
- **Mutation proof:** that same fixture returns **`ModulePrivate` (GREEN)**
  against the **pre-fix** artifact — already observed, so this is a
  reproduction, not a hunt. ⛔ A probe that only passes post-fix proves nothing.
- **Cover BOTH prefixes end-to-end:** the `cr#"a " b ; /*"#` + `pub // */`
  fixture must ALSO return `Undetermined` post-fix and `ModulePrivate` pre-fix
  — already reproduced (`evt_5trt38vfrp830`). A test that only pins `br` would
  pass a `b`-only patch that leaves `cr` open. Add `br##"…"##` (extra hashes)
  too.
- **Unit-level, on `string_literal_end`:** `br#"…"#`, `br"…"`, and `cr#"…"#`
  return `Some(end)` at the true literal end; `None` pre-fix.
- **Negative control (load-bearing here):** `foo_r"x"` and any identifier
  ending in `b`/`c`/`r` must **still** be rejected. The fix permits characters
  the guard previously refused, so **over-permission is the live risk** and the
  control is the only thing that catches it.
- **Discriminator:** `r#"a " b /* c"#` (no prefix letter) behaves correctly
  today — keep it paired so the test pins the `(b|c)?r` prefix boundary
  specifically rather than raw strings generally.
- **Regression guards:** plain `b"…"` and `c"…"` (non-raw) must stay
  correctly-handled `FailWidened` — they were never broken; a prefix-parsing
  fix must not accidentally start treating them as raw.
- ⛔ **Do not fix by relaxing the `//`-at-depth handling.** That behaviour is
  correct Rust comment semantics; changing it would break real nested comments
  and leave the actual defect in place.

★ **Fifth instance of the same shape** (adversary's own framing): when a ring
hardens a mechanism and invites attack, **the mechanism holds and its input
handling does not**. Sibling of the publisher gate's plumbing, the probe
harness's extraction, and the pane detector's capture window. Audit the
plumbing separately from the interesting part.

## ⛔ The stopgap closes the SET; it does not close the CLASS

`(b|c)?r` is complete **today** — the adversary asked rustc directly rather
than reasoning from the reference: `r`/`br`/`cr` lex, and `dr`/`er`/`fr`/… are
hard lex errors (`evt_6m4tkgtce5dtp`). And the token boundary must sit **before
the `b`/`c`**, not before the `r` — `xbr` is a valid identifier.

**But this is the THIRD false-GREEN closed by enumeration in one evening** —
nested comments, then `br#`, then `cr#`. They are one shape: `blank_comments`
is a **partial hand-rolled Rust lexer, and every gap in it is a silent pass.**
The prefix *set* is closed now; the *class* — "a Rust token form the scanner
tokenizes wrong" — is not, and will not be while the mechanism is a text matcher
over Rust source.

⇒ **This is the day's own enumeration lesson happening to the fix itself:**
three passes, each completing the last's bounded search by *running the thing*,
each finding one more member of a set the previous pass thought closed. See
[[an-enumeration-needs-a-proven-closure-not-a-better-grep]] and
[[repeated-defeats-of-one-checker-mean-the-default-branch-is-wrong]] — this is
the second-defeat trigger, three times over.

**The listless fix is already scoped: F7 option 1, the in-crate compile probe.**
A probe from inside `ken-runtime` has no lexer of its own to get wrong, because
`rustc` does the lexing — so it needs neither the prefix list, nor the
comment-nesting logic, nor the string-escape logic to be complete.

**Sizing, explicit:** ship `(b|c)?r` **as the XS stopgap** — it is cheap, and
the three-outcome design already fails *closed* on parse gaps it recognizes;
this bug is the case it does not recognize. ⛔ **But record in the fix that the
recurrence IS the signal** — the third lexer patch in one evening is the
evidence the enumeration has no closure, and F7's harness is what ends it.
`runtime-leader` already ruled the text pin stays until that harness exists;
this note is the standing pointer to why it should be built, not a request to
switch mechanisms now.
