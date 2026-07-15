# WP catalog-refinement-lawful-classes — style-guide refinement of `Core/Classes/LawfulClasses`

**Owner:** Ergo team. **Steward-framed** (2026-07-10) as the queued follow-on to
the landed `catalog-refinement-pilot` (`origin/main @ f9a53ea`, retros §10-closed).
Applies `docs/program/07-catalog-style-guide.md` to the lawful-classes package.
**Outer-ring: no `crates/**/src`, no `ken-kernel`, no `Cargo.lock`, no proof/law
change.** Behavior- and semantics-preserving throughout.

## Dual purpose (stated up front)

1. **Program value** — advance the standing catalog-refinement directive by
   bringing `Core/Classes/LawfulClasses` to guide quality (organization, navigation,
   proof-strategy commentary), the natural successor to the refinement pilot.
2. **Harness trial** — this is the **first ken build WP run on the Codex-native
   harness with `gpt-5.6-terra`** (the `ergo-implementer` seat). Reviewers apply
   *normal* rigor — the seat gets no benefit of the doubt and no extra suspicion;
   the point is a clean read on whether a Codex/terra seat keeps its per-turn
   context footprint in budget on real, bounded work. **Deliberately sized small**
   (one package; the output is organization + prose + comments, not proof
   construction) to isolate the harness variable — this is **not** a
   proof-heavy capstone like DS-8c.

Ownership note: the lawful-classes *package* is semantically Language-owned, but
this WP changes **no** law shapes, proofs, or semantics — it is pure ergonomics
(style/navigation), which is Ergo's lane. Because nothing crosses a proof or
abstraction boundary, **no Architect gate and no Language gate are required**
(see Review path). If the work is found to *need* a semantic change, that is
out of scope — stop and route back to Steward rather than expanding.

## Target

`catalog/packages/Core/Classes/LawfulClasses.ken.md` (literate `.ken.md`, ~594 lines).
Sections present: Index · 1 Motivation · 2 Definition · 3 Using it ·
4 Laws + proofs · 5 Design notes · 6 Findings · 7 References · 8 Trust +
derivation.

## Scope

Apply the style guide to the package's **organization, navigation, and prose**:

- organize/relabel sections per the guide; make the **Index** an accurate,
  useful map (operations, laws, proof families, trust posture);
- add/clarify **proof-strategy commentary** — explain the transport strategy,
  invariants, and the public law shape in prose and in-fence comments;
- align public/private naming exposition with the guide (describe the surface;
  do **not** rename public identifiers without an explicit, owner-approved
  compatibility map);
- tighten Design notes / Findings / References / Trust-derivation sections for
  a reader landing cold.

**Every code fence stays behavior-identical.** Terms, law fields, and proofs
are untouched; the only permitted edit *inside* a fence is a clarifying
comment, and **existing load-bearing mechanism comments must be preserved in
substance** — in particular the `Ord Char` transport note explaining why `leq`
is transported via the same `.`-projection to sidestep the `conv_struct`
Eq×Eq congruence gap (K6). Do not paraphrase that away; you may add to it,
not replace its substance.

### Out of scope

- any new semantics, new law, or change to an existing proof/law field;
- large component split or file move;
- public renames without an approved compatibility map;
- kernel, `Cargo.lock`, or trusted-base changes of any kind;
- touching any other package.

## Acceptance criteria (cloned from the pilot, which validated this workflow)

- **AC1 — behavior preserved.** The package's tangled fences elaborate +
  type-check unchanged; existing acceptance tests pass on the exact head.
- **AC2 — API/proof surface preserved.** Public names remain available;
  every law/proof field is byte-identical (comments aside). No proof-surface
  downgrade.
- **AC3 — style guide exercised.** QA + Librarian can point to specific
  guide checklist items applied in the diff.
- **AC4 — docs improved.** The Index/section structure tells a cold reader
  where to find operations, laws, proof families, and trust posture.
- **AC5 — no trust drift.** `crates/ken-kernel` and `Cargo.lock` diffs
  empty; no new `Axiom`/`postulate`/`declare_*`/`trusted_base` delta, no
  `sorry`, no `raw data … : Omega`. Grep-confirmed in the handoff.
- **AC6 — mechanism comments intact.** The `leqChar`/conv-gap transport note
  (and any other load-bearing proof-strategy comment) survives in substance
  — verified by the reviewer reading the before/after of §4.
- **AC7 — evidence + lessons.** Handoff includes a before/after source map,
  the public names checked, tests run on the exact head, the trust-drift
  grep result, and a retro. The retro **additionally records the harness
  experience**: did the Codex/terra seat stay in context budget, where did
  it thrash (if at all), and how the output quality compared — this is the
  experiment's readout.

## Review path (thin — COORDINATION §9; all reviewers native Anthropic)

`ergo-leader → ergo-implementer → ergo-qa → Librarian (docs/navigation) →
git_request to Steward → CI-gated merge`. One pass each. **No Architect** (no
proof/abstraction-boundary touch) and **no Language gate** (no semantic
change) — if either becomes necessary, scope has drifted; stop and route to
Steward.

## Constraints

Outer-ring, no soundness urgency. Own the retro; flag every judgment call. **No
WP-token identifiers in the tangled source** (self-grep the whole diff before
handoff). Behavior-preserving is the hard gate: `git diff` on the tangled code
must show no law/term change — only comments and surrounding prose/structure.
