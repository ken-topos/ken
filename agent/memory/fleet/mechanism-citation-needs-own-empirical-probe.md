---
scope: fleet
audience: (see scope README) — anyone citing a specific error mechanism or
  named substrate gap in prose (spec text, catalog Findings, WP frames)
source: DS-8 catalog entry, spec/conformance track
---

# A mechanism citation needs its own empirical probe

When prose cites a specific elaborator/kernel mechanism as the reason
something fails (e.g. "this hits the same wall a sibling entry documents"),
that citation needs its own direct empirical check — the actual error
variant produced by the actual probe — not just a resemblance to a
previously-documented, similarly-shaped gap.

**Why:** in a catalog entry (DS-8), a `Functor`-style instance was written
up as blocked by the same empty-context `UnresolvedCon` wall a sibling
entry (DS-7) documents for a different type. This was PLAUSIBLE — both
cases involve a parametric instance head with free type variables — but
WRONG: an actual probe showed the head *resolves* fine; the real failure
is a distinct `KernelRejected(TypeMismatch { expected: Type -> Type, found:
Type })`, a parametric-instance-head KINDING limitation, not the same
mechanism at all. This citation error slipped past the author's own
writing AND an independent review (endorsed as "accurate, not hand-waved")
— only a direct elaborator probe against a dummy class caught it. It
required a second gate cycle to fix, and a follow-on catalog entry's
framing was coupled to the same error (falsely implying a downstream
capability landing would let the instance "assemble").

**How to apply:** before citing ANY specific error mechanism/gap by name in
prose (spec text, catalog Findings, WP frames), run the actual probe that
produces the actual error and check the VARIANT, not just whether the
general shape (free type var in an instance head, abstract codomain, etc.)
matches a gap already known. Two different-looking mechanisms can produce
superficially similar symptoms (both "instance doesn't declare") for
entirely different underlying reasons. This is the error-citation sibling
of grounding a named floor by grepping it rather than assuming it — same
discipline, applied to *why* something fails rather than *whether* a named
capability exists. See [[frame-pseudocode-diverges-from-landed-mechanism]]
(same family: prose that describes a mechanism the code doesn't actually
run).
