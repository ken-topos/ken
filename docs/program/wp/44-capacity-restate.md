# WP 44-restate — Content-store spec: trim to the settled decisions

## Objective

Correct `spec/40-runtime/44-capacity.md` so its **presentation matches the
settled decisions**. The chapter currently *centers* the Leech/Golay/Co₀
lattice — it titles on it and devotes §2 (a Leech-ceiling framing) and all of
§4 (the three-role table) to a flourish that **OQ-6 decided out of the core**.
It reads as an inherited design-candidate document the resolution hollowed out.
This is an **erratum-class trim**: it reopens **no decision**; it aligns the
chapter's emphasis with OQ-5 / OQ-6 / OQ-gc as already resolved, and records the
operator's now-settled **systems-adjacent** positioning as the rationale for the
memory model.

## Fixed inputs — settled; do NOT reopen

- **OQ-5 (capacity), OQ-6 (lattice), OQ-gc (reclamation) are DECIDED** (operator
  2026-06-27; `90-open-decisions`). This WP changes **presentation only** — no
  verdict moves. Capacity = engineering-chosen wide handles + loud refusal (no
  Leech ceiling); lattice = out of core, optional WS-R packages, never the hot
  path; reclamation = manual+region default, automatic GC an optional,
  semantics-invisible, deferred implementation detail.
- **Positioning ruling (operator, 2026-07-02): Ken is systems-*adjacent*, NOT a
  bare-metal systems language.** Ken keeps the software-engineering / verified
  aspiration and **yields the true-systems-language space** (freestanding,
  manual-memory, against-the-OS-kernel) **to Rust**. The content-addressed
  managed heap + optional semantics-invisible reclamation is the **correct**
  model for that positioning — not a compromise to apologize for. State this as
  the rationale; it is **not** a new fork (`OQ-systems-target` is NOT opened —
  the operator closed that fork in favor of systems-adjacent).
- §1 store mechanics (interned content heap, hash+memcmp, global dedup) are
  correct as-is — not touched.

## Scope

**IN — `spec/40-runtime/44-capacity.md` only:**

- **Retitle** so capacity + reclamation are the substance and the lattice is
  demoted (e.g. drop "and the lattice" from the H1).
- **§2 (capacity):** lead with the engineering stance — wide handles → no
  practical ceiling, loud refusal over silent degradation, dedup-aware
  accounting. Keep a **brief** note that the Leech 196,560 ceiling was
  considered and rejected as aesthetic — enough to preclude re-proposal, not the
  section's spine.
- **§3 (reclamation):** state the **systems-adjacent rationale** — manual+region
  default is systems-appropriate (arenas/regions are a systems-native technique,
  deterministic, no mandatory collector); automatic GC is optional +
  semantics-invisible (immutable values + content identity ⇒ reclaiming an
  unreachable slot changes nothing observable) + droppable, so Ken is **not** "a
  GC language" and the model does **not** chase bare-metal. Cite the 2026-07-02
  positioning ruling.
- **§4 (lattice):** demote to a short "considered, rejected as core" note — the
  Golay-EC / kissing-number-bitmap / Co₀-canonicalization roles survive **only**
  as optional WS-R packages, never the hot path. The three-role table may remain
  as an *appendix-weight* forward-pointer to potential WS-R packages, not a
  core §.
- Keep §5 (X4 scale) and §6 (WS-X deliverables) substantively intact; fix any
  cross-ref affected by the retitle.

**OUT:**

- No decision reopened (OQ-5 / OQ-6 / OQ-gc stand; substance frozen).
- No change to §1 store mechanics, `41` (heap), or `45` (native backend).
- `OQ-systems-target` is **not** opened — the operator chose systems-adjacent.
- **Optional flag (routes to Steward, do not act on here):** whether a one-line
  *"what Ken is **not** — a bare-metal systems language; that space is Rust's"*
  belongs in `PRINCIPLES §7`. PRINCIPLES is Steward-owned — **surface** it in
  the handoff, don't edit it in this WP.

## Acceptance (testable — the reviewer confirms)

1. The H1 title and §2/§4 emphasis no longer center the lattice; a first-time
   reader takes away the **engineering capacity stance + the reclamation
   model**, with the lattice as a clearly-optional aside.
2. **OQ-5 / OQ-6 / OQ-gc verdicts are unchanged in substance** — the diff is
   presentation, not decision. (Reviewer cross-checks the trimmed text against
   the three DECIDED entries in `90-open-decisions`.)
3. The **systems-adjacent positioning** is stated in §3 as the memory-model
   rationale, cited to the 2026-07-02 operator ruling.
4. Cross-refs resolve: the only landed use of the old title string is the
   chapter's own H1 (`git grep "capacity, reclamation, and the lattice"`);
   external refs cite `44 §4` / OQ-6 by section and are unaffected. Confirm no
   dangling reference after the retitle.
5. 80-col wrapped; Mermaid for any diagram (no ASCII art).

## Guardrails (do-not-reopen)

- **Erratum-class: presentation only.** If the enclave finds a spot where the
  *substance* seems wrong (not just the emphasis), **STOP and flag Steward** —
  that is a decision reopening, out of scope here.
- **Don't delete the WS-R lattice pointer entirely** — it's a real forward-
  pointer to optional packages; demote it, don't erase it.

## Logistics

- **Owner:** spec enclave — spec-author elaborates, CV Spec + Architect
  soundness gate as a normal `/spec` Decision. **No build team** (spec-only).
- **Branch:** `wp/44-capacity-restate` off `origin/main`.
- **Pipeline:** Steward frame (this) → spec-leader elaboration → merge Decision
  (touches `/spec` → Architect soundness + CV Spec) → Integrator merge to
  `main`. Ends at merge (no downstream build).
- **Priority:** **low, non-blocking** — runs in parallel with the F1 build;
  nothing depends on it. The operator flagged §44 as at odds with their
  recollection of the design decisions; this closes that gap.
