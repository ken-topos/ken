---
scope: roles/steward
audience: (see scope README) — anyone eyeballing a wrap-width finding before
  invoking the `wrap-md-80` skill
source: a spec-leader coherence pass that flagged two lines as needing a
  rewrap; the actual `wrap-md-80` run found 0 rewraps needed
---

# `awk length()` miscounts multi-byte characters against the 80-column rule

`awk '{print length, $0}'` piped through a `>85` filter is not a valid check
for the repo's 80-column convention (`CLAUDE.md`: target 80 *display*
columns / codepoints, where a multi-byte `—`, `→`, `Ω` counts as **one**
column). Depending on locale, `length()` can count bytes (UTF-8 multi-byte
sequences inflate the count 2-3x per symbol) or otherwise fail to match the
display-column rule the convention actually uses. On a spec-doc coherence
pass, two lines were flagged as needing a `wrap-md-80` pass; the real skill
run found **0 rewraps needed** — both lines were within the real ceiling
once measured correctly.

**Why:** wrap checks feed directly into review findings sent to another
seat — a false positive burns their tokens on a no-op fold and erodes trust
in the reviewer's grounding.

**How to apply:** never hand-roll a column-width check with `awk length()`
(or any byte-counting tool) on Ken spec/doc prose that may contain `→`, `Ω`,
`—`, or other multi-byte symbols. Either (a) delegate the check itself to
the `wrap-md-80` skill/subagent and trust its verdict, or (b) if
hand-checking, count codepoints (e.g. Python `len(line)` on a decoded
string), never raw byte length. The same portability trap bites `awk`-based
style spot-checks generally — see
[[splay-gate-only-as-good-as-its-detector-verify-the-check]], where `\s` in
an `awk` regex (a GNU/PCRE extension, not POSIX) silently matched nothing
under `mawk`. Prefer `[[:space:]]` over `\s`/`\d` in any `awk` you write.
