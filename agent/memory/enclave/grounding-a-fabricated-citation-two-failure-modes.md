---
scope: enclave
audience: (see scope README)
source: private memory `grounding-a-fabricated-citation-two-failure-modes`
---

# Fixing a laundered citation has two failure modes past stripping the token

When a laundered citation authority catch resolves on the **YES branch** — the
underlying decision was *real*, only the citation's **id + status** were
fabricated (Steward's WP-frame wrote `OQ-systems-target closed 2026-07-02`; no
such id was ever registered) — the fix is to author the **real** register entry
and reframe the citing prose. Reviewing that fix has **two failure modes past
the obvious "strip the false token,"** and both nearly shipped on WP
44-capacity-restate (2026-07-02, merged honest `3e30e4c` #214):

**(a) Over-strip the now-grounded content term.** The fabricated *authority/
status* tokens (`OQ-systems-target`, "closed"/"closed in favour", "cedes to
Rust") must go — but the **positioning term itself** (`systems-adjacent`) is now
the *grounded* content, cited to the real entry (`OQ-domain`) + the charter
(`PRINCIPLES §I.1`). A blanket strip-grep that greps the content term to zero
**false-blocks a correct fold** — it demands removing exactly what the YES
branch requires. CV's re-check #2 initially listed `systems-adjacent` as a strip
token; I caught it before they ran it. **Rule: strip set = fabricated
authority/status tokens ONLY; the grounded positioning term must SURVIVE,
cited.** (Plus the unrelated-homonym guard: `44:184`'s "**closed** sum
`New|Hit`" is legit intern-result content — scope the grep to the positioning
tokens in the section, never a blanket `closed` grep.)

**(b) The newly-authored DECIDED entry over-claims beyond the ruling's bounds.**
A fresh `DECIDED` register entry is a trust-level upgrade; it must not silently
promote an **OPEN** dependency into a **shipped** capability. Here the ruling
was *asymmetric*: lower bound (systems-adjacent) **settled+substantiated** now;
upper bound (application/edge/web/mobile) **directional, not delivered** — an
aspirational reach via native codegen, itself as-yet-unexplored
(`OQ-backend- target`, **OPEN**; `45`/X-series). An entry that stated the upper
bound as delivered would be the exact honesty-about-the-boundary failure (trust
level prose vs locked adr crosscheck) — a `DECIDED` silently upgrading an OPEN
codegen dependency into a claimed web/mobile capability. **Rule: gate the
aspirational/settled asymmetry as a HARD check — the entry + the citing prose
must keep "what's settled" vs "what's directional" explicit, tied to the OPEN
dependency.** (Operator enrichment via PRINCIPLES §8 honesty; CV owned this in
their over-claim lane, I gated the asymmetry.)

**Meta (the catch itself):** the finding was pure grep — `git grep` on
`origin/main` showed the cited id existed nowhere. The cheapest check (grep the
DECIDED-id against the register) is the one that de-launders, O(1) regardless of
how many adoption points the id passed through gaining false authority. Two
independent source-greps (mine + CV's), not concurrence, and the conjunction
held: fabricated-frame-id + baked-without-re-verifying both had to reach main,
neither did. Sibling of laundered citation authority + trust level prose vs
locked adr crosscheck.
