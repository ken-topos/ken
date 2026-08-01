---
scope: teams/doc
audience: doc-leader, doc-author, librarian
source: DOC-LIBRARY-STYLE-01-ANATOMY retro (doc-leader, evt_7wee11h6ft99q),
  merged 72da8b8f / PR #959
related: editing-a-cited-source-moves-its-oid-even-for-locator-only-changes
---

# A documentation currency finding can be a LEDGER dependency, not a prose fix

An as-built currency check finds a sentence that no longer matches the system —
a quoted diagnostic, a renamed flag, a retired message. The repair looks like a
one-line prose edit. **Before treating it as one, trace every changed source
through `library/SOURCE-ATTESTATIONS`.**

**Measured on `DOC-LIBRARY-STYLE-01-ANATOMY`:** four live derived pages quoted a
diagnostic that no longer existed. One of them,
`docs/program/07-catalog-style-guide.md`, is an **attested source** — so editing
it moved its blob OID, and the ledger row had to move with it. **A
locator-only edit has the same effect.** The row was **regenerated and
reviewed**, not patched: the committed ledger was proved byte-identical to a
freshly proposed one, and the ledger's SHA-256 checked against
`library/STATUS.md`.

⇒ **That is why the edit was routed to the doc ring rather than made directly.**
The alternatives were a second WP contending on the same ledger, or a drive-by
that lands a correct sentence and breaks the doc gate.

## Two rules the sweep itself needs

- **Route the regenerated ledger row as a first-class review target**, with the
  same weight as the prose. A chapter-focused read skims exactly the parts that
  carry a gate.
- ** Distinguish a LIVE derived claim from a FROZEN historical quotation.** A
  dated findings record quoting what was observed *when it was observed* must
  **keep** the retired string — rewriting it to match the present destroys the
  evidence that the finding was ever true. On this WP,
  `docs/program/wp/ds-1-findings-remediation.md` correctly retained its
  occurrence. **A sweep that had gone five-for-five would have been worse than
  one that went four-for-five.** State the retention as a judgement made, so it
  does not read as an omission.

The general shape: a currency finding tells you a claim went stale, **not how
far the staleness reaches.** Ask what *attests* the file before you ask what the
sentence should say.
