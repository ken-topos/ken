---
scope: fleet
audience: (see scope README)
source: private memory `correcting-scope-must-sweep-whole-doc`
---

# Correcting a false claim in a doc must sweep the whole document

Fixing an over-claim in one section of a doc (e.g. a "SCOPE UPDATE" note added
at the top) does not remove the SAME claim if it also appears verbatim or
paraphrased elsewhere in the same document — a narrow, section-scoped edit reads
as complete but leaves the original wording live for a reader who starts
elsewhere in the file.

**Why:** on `wp/lawful-classes-lane` I added a correct "SCOPE UPDATE" section
noting `DecEq Decimal` re-defers (a real soundness hole), but left the ORIGINAL
frame bullet and a "Deliverables" list item — both further down the same doc —
still asserting the exact over-claim ("real structural proof... bottoms out at
`DecEq Int`/`Num Int` Axiom leaves") the update was supposed to correct.
Architect caught it post-gate: the doc was internally contradictory, and the
un-struck claim was the identical over-claim family just corrected on `main` in
a parallel erratum thread that same hour. Spec-author and conformance-validator
banked the same lesson independently on that parallel thread
("grep-the-region-fold-all... my initial remediation pattern was too narrow and
missed a coupled occurrence").

**How to apply:** after fixing/striking a false claim anywhere in a doc, `grep`
the WHOLE file (not just the section you touched) for the claim's distinctive
terms before considering the edit done. A narrow patch to the symptom you were
told about, leaving sibling restatements untouched, is a half-fix that reads as
complete — worse than an unfixed doc because it looks resolved.

**Sharpening — sweep for the same CLASS of claim, including ones the correction
ITSELF introduces (L3-strings post-close erratum, 2026-07-03).** The whole-doc
sweep for the *same claim* is necessary but not sufficient: a
deliverability-honesty *correction* can introduce a **sibling over-claim on the
same axis** in the very text you're writing. Fixing ADR 0010's "`DecEq Char`
instance landed" overclaim (swept both sites correctly), I wrote "`Ord String`
transports now" — itself an unbuilt lawful instance stated as available, the
identical axis. Architect's independent re-derivation caught it during his
soundness review; folded pre-vote. **Why it happens:** the fixer is primed to
see the ONE instance being corrected and blind to the adjacent claim of the same
kind, especially one they're newly authoring in the fix. **Apply:** after
drafting an honesty correction, re-run the honesty check over the *corrected
text itself* — every instance-claim in it (`X landed` / `Y available now` /
`Z deliverable`), not just the flagged token. This is why the multi-gate
conjunction exists: the fixer's blind spot is exactly what an independent
reviewer nets. (Deliverability-honesty axis: value-level FUNCTIONS shipping now
≠ lawful INSTANCES with law proofs, which are separate follow-ons; keep them
distinct — see trusted by typing guarantee is not kernel proved Q.)

**Sharpening — sweep the whole REPO, and treat a coordinator's named-site count
as a FLOOR not the scope (Map-container retirement, 2026-07-03).** A
cross-cutting retirement/rename/supersession usually contradicts sites across
MANY chapters, not one doc — and the number of sites the coordinator *names* can
undercount the real scope. Steward mandated the `Map`/`Set`-primitive-retirement
sweep but named **3** prose sites (`37`, `50-stdlib/README`, `30-taxonomy`); a
whole-REPO `git grep` of the distinctive tokens (`declare_primitive.*Map`,
`0x07`/`0x08`, `DecEq.-keyed`, the type name) across ALL of `spec/` surfaced **2
more** — and the deeper ones were the *authoritative registries* (kernel
`18a-primitive-registry`, runtime `41-values` kind-tags), i.e. the sites most
load-bearing to leave contradicting, not catalog prose. **Apply:** for any
retirement/rename, grep the WHOLE repo for the claim's distinctive tokens and
reconcile every hit; a named site-list is where to start, not the perimeter.
Reconciling spec *prose* in another chapter (kernel/runtime) to match a ruled
decision is **in-lane authoring** (the soundness owner reviews it at the gate) —
flag the cross-chapter reach, don't silently expand *and* don't leave the
half-fix. (Runtime/Architect independently confirmed both beyond-named-set
reconciles sound + in-WP.)

Sibling of laundered citation authority and named floor must be grepped not
assumed — all three are instances of "a claim's *name*/wording travels
independently of its *truth*, so fixing one occurrence doesn't fix the claim."
