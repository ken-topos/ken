---
scope: roles/conformance-validator
audience: (see scope README)
source: split out of private memory `conformance-validator-casts-spec-review-
  vote` (instances X2, L6, B3, Sec2, L7, L2, all 2026-06-30) — decomposed
  2026-07-28: this is a re-derivation technique, orthogonal to the
  vote-casting-authority rule that file states
related: conformance-validator-casts-spec-review-vote
---

# Verify every spec cross-reference by its CONTENT, not by its number

When casting the independent Spec vote, a citation check that stops at "the
`§`-number resolves to a real, existing section" is not the check. A cite can
resolve — right file, real section, plausible neighborhood — and still not
host the claimed content. **Open every cited target and read what it actually
says; confirm it hosts the claimed content**, not just that the number exists.
Six instances (all 2026-06-30) found this same defect class via the same
discipline; the rule has also repeatedly *confirmed* good cites (it is not a
bug-hunting-only tool).

## X2 — a dangling forward-reference into an untouched section

`44 §2` cited "Placed in `43 §2` fault taxonomy," but `43-termination.md §2` is
a flat 4-item list with **no resource-fault class**, and the file was untouched
on the branch. The number resolves; the content it's supposed to host doesn't
exist. Non-blocking, shipped tracked as a live erratum at `e18f4aa` under
resolve-and-track — flagged so "non-blocking" doesn't silently evaporate.

## L6 — the rule confirms as fast as it catches (first clean run)

On `38 §1` (Bytes + binary I/O), all **8** cited targets (`14 §5`, `41 §3a`,
`36 §1.4`, `31 §3`, `14 §8.4`, `34 §5`, `18 §5`, `43 §2`) hosted the claimed
content. Vote APPROVE, no flags, merged `cb90621` clean. The same pass that
catches X2's dangling `43 §2` slot discriminates it from L6's *legitimate*
`43 §2` partiality cite — same section number, opposite verdict, only
distinguishable by reading the body.

## B3 — filename-in-my-head is the wrong axis; and the rule is author-agnostic

Expected a file named `63-discharge-attestation.md`; the real file is
`63-supply-chain.md`, and its §5a hosts the real discharge-attestation content.
Resolved by checking **§-content, not the filename** you expected. This run was
also the first on a seed CV did **not** author (spec-author's `seed-trace.md`)
— the rule held identically reviewing someone else's conformance, confirming
it's a pure independent re-derivation, not a self-check artifact.

Same WP: verify-on-main must also check the **index/README pointer**, not only
the spec+seed+invariants triad. `conformance/README.md`'s Seeds index was
un-updated for `behavioral/trace` — the export/layout entries landed but the
README index row didn't (non-blocking; CI gates the seed, not the index, but
the human-facing index is now incomplete).

## Sec2 — wrong-section-in-the-right-file (the sharpest form)

`62-authority.md §9`'s level table cited `record / Σ-Form (13 §1)`. `13 §1` is
*Dependent functions — Π*; Σ-Form is actually `13 §2`. Right file, real
section, adjacent content — a number-resolves check passes it, a heading-only
check passes it, **only reading what §1 actually hosts (Π, not Σ) catches it**.
Non-blocking provenance pointer, flagged into `wp/spec-errata`.

## L7 — verify a wrong-section claim with ABSOLUTE line numbers

Chasing a cross-ref via a piped `sed '/## 2/,/## 4/p' | grep -n`, the line
numbers are **relative to the extract**, not the file — reading them as
absolute produced a **false** wrong-section finding against spec prose that
was actually correct. Recomputing with `git show <ref>:file | grep -nE
'pattern'` (absolute file line numbers) showed the spec body was right; the
*seed* was the one with the (non-blocking) wrong-section-in-right-file slip.
**Rule: before asserting a wrong-section mis-cite, re-verify the target's
location with absolute line numbers from `git show :file | grep -n`, never
relative-to-extract numbers** — the discipline that catches a real slip is the
same one that can manufacture a false one if the line numbers aren't absolute.

## L2 — an example can violate its OWN rule while every §-body reconciles

A §-body content-reconcile is necessary but not sufficient: a spec's own
*illustrative example* can contradict its own normative rules while every
cited section, read on its own, is internally consistent. `12 §5`'s
`head {a} (xs : NonEmpty a) = match xs { Cons x _ => x }` omits the `Nil` arm.
Under `§5`'s own carrier encoding (`NonEmpty a` → carrier `List a`, the
`xs ≠ Nil` proof **erased**) plus `§4.1`'s coverage rule, the scrutinee inside
`head` is type-possibly `Nil` with no proof in scope to refute it — `§4.1`
*requires* the `Nil` arm. The defect only surfaces by re-deriving the
*example's behavior* against a **different** section's rule (§4.1 coverage
over the §5 carrier), not by reconciling either section against its own prose.
Non-blocking (normative rules are sound; this is a conformance gap — no
refinement-vs-index-coverage case existed).

## How to apply

- Open **every** cited cross-file target on the reviewed branch/SHA and read
  the body — confirm it says the thing the case relies on (the verdict, the
  stage, the exact level), not just that the `§`-number exists.
- Check by **content**, never by filename-as-remembered or heading-as-title.
- When asserting a *wrong-section* finding, ground the target's location with
  **absolute** file line numbers (`git show <ref>:file | grep -n`), never
  numbers read off a piped extract.
- For a normative example, re-derive its behavior against **every** section
  whose rule governs it, not just the section it's printed under.
- Run this pass identically whether the cited spec is your own authoring or
  someone else's — it is a re-derivation, not a self-check.
- A confirmed-clean sweep is a real, reportable result, not just a non-event —
  state it (APPROVE, no flags) with the same weight as a caught defect.
