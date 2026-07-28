---
scope: fleet
audience: (see scope README) — anyone authoring a normative §-mechanism
  statement (soundness/admission/elaboration prose) for spec or conformance
source: case-eq WP and proof-vocabulary WP, two consecutive recurrences
---

# Frame pseudocode diverges from the landed mechanism

When authoring a normative §-mechanism statement (soundness/admission/
elaboration prose), the WP frame's **own pseudocode is not the producer** —
even when it was authored as a "normative admission algorithm." Implementers
often deliver the same *observable property* by a different *mechanism*,
and the spec is bound to the mechanism that ships.

**Concrete recurrence (2 in a row):**
- case-eq: the frame/ruling sketched a materialized `Or_N` dichotomy; the
  landed elaborator built a direct eliminator-with-equation-motive.
  Conformance validation caught it; the mechanism prose was re-grounded on
  the producer.
- proof-vocabulary: the frame's Phase-1 pseudocode said a **"scope-wide
  signatures-first" pre-pass delivers forward references.** The landed
  code has **no such pass** — forward refs come from **dependency-ordered
  SCC processing** (callee component before caller; condensation edges =
  union of all members' out-edges), and signatures-first is only the
  *within-recursive-component* step. Re-grounding on the producer flipped
  the wording. (Currency check, 2026-07-28: the SCC-order function and the
  mutual-group elaborator both still exist — grep `scc_dependency_order` in
  `crates/ken-elaborator/src/modules.rs` and `elaborate_mutual_group` in
  `crates/ken-elaborator/src/elab.rs`.)

**Why:** a frame's pseudocode is a design-intent artifact written *before*
implementation; the team may satisfy the AC by another route. Reflecting
the pseudocode verbatim puts a mechanism in normative prose that the
producer never runs — a fidelity bug that conformance validation will catch
(or worse, won't).

**How to apply:** before committing a mechanism claim, read the *landed*
function(s) named in the WP and describe what the code literally does —
the control flow, the ordering, what delivers each property — not what the
frame's pseudocode says. Cite the function, not the frame. When a code
candidate gets a repair (e.g. an ordering fix), re-ground against the exact
QA/Architect-cleared head — the mechanism prose may need to change with
it. See [[mechanism-citation-needs-own-empirical-probe]].
