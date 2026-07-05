---
scope: enclave
audience: (see scope README)
source: private memory `class-dict-explicit-vs-implicit-abstract-tyvar`
---

# A class-dict blocker over an abstract tyvar is usually only the implicit path

**The finding (Ken Map-build generic-`Ord k` fork, 2026-07-03,
`evt_38nvzzdwb8ewn`).** foundation-leader escalated (well-grounded: grepped
`instance_search`/`ClassEnv`, file:line) that generic `insert`/`lookup` + the §5
proofs need a dictionary parametric in an abstract `k`, and "neither
`where Ord k` sugar nor a hand-written explicit `(d : Ord k)` param has any
landed path." Routing the JUDGMENT to Architect was correct; the conclusion was
an over-generalization.

**What grounding + probing showed (I read `infer_proj`/`classes.rs`, then RAN
probes at `origin/main` in a scratch worktree — did not rule from the report):**
- A class is a **Σ-record type** `C : Type -> sort` (`classes.rs`
  `ClassInfo.type_id`), so `Ord k` is a valid type for ANY `k` and `(d : Ord k)`
  is a well-formed Π-param.
- `.field` projection (`infer_proj`, `elab.rs`) identifies the class by matching
  `ci.type_id == class_type_id` (the class type-constructor's `GlobalId`) and
  builds **plain kernel Σ-projection** `proj1(proj2^idx(base))` off the dict
  VALUE. The head arg (`k`) is only substituted into field types — it is NEVER
  required concrete. So projection is **head-agnostic**: `(d).leq` on an
  abstract `d : Ord k` projects fine, **completely independent of
  `instance_search`**.
- `instance_search` (the flat `(class, head_name) -> GlobalId` map needing a
  concrete registered head) is ONLY the **implicit `where`-resolution** path.
  The elaborator's own comment: `where C a` "supplies the resolved dictionary …
  RProj … exactly as if it had been passed explicitly — no second mechanism." So
  explicit-dict is the PRIMARY path; `where` is sugar over it.
- **Probe results (decisive):** `(d).leq x y` over abstract `k` → `Ok` (probe1);
  unbundled bare-op + bare-law-hypothesis (`leq : k->k->Bool`,
  `reflLeq : (x:k)->IsTrue(leq x x)`, verified-sort style) over abstract `k` →
  `Ok`, laws in type position (probe3); `(d).field` in a TYPE position →
  `ParseError: expected RParen, found Dot` (probe2/4/5 — a fundamental
  **parser** gap, so the *bundled* dict can't state proof types, but the
  *unbundled* form doesn't need it). Landed precedent for the unbundled shape:
  `conformance/challenge/C5-verified-sort/sound-verified-sort.ken` threads
  `(leq : a -> a -> Bool)` over abstract `a`.

**Ruling shape:** generic class-dict use is a **surface-spelling /
implicit-sugar** gap, NOT a mechanism gap. The parametric-over-`k` operations
AND proofs the spec's premise needs are **buildable today** via the unbundled
explicit encoding (bare ops + bare law hypotheses), which is semantically
identical to `d : Ord k` (`Ord k` IS the Σ-bundle of those fields) → same
parametricity, same laws-agnosticism (Axiom-vs-real invisible to a proof that
only sees abstract hypotheses). Tag the `where`/bundled spelling `(oracle)`;
sequence the sugar as a **later reconcile**, not a hard prerequisite WP.
Fidelity guardrail (conformance lane): the `(oracle)` tag sits on the SPELLING,
proofs must stay genuinely `k`-parametric (threaded law hypotheses), never
silently specialized to a concrete instance — same discriminator as lawful class
instances must carry law proofs.

**How to apply.** When a build lead reports "feature X over an abstract type var
is unbuildable / needs a prerequisite," before ruling a hard-blocker: (1)
separate the IMPLICIT-resolution mechanism (search needing a concrete key) from
the EXPLICIT/primitive mechanism (Π-abstraction + Σ-projection) — the reported
blocker is often only the former; (2) grep the projection/resolution producer to
see if it's head-agnostic; (3) **probe it** — a 10-line elaborator test settles
"expressible today?" definitively where a careful grep can still
over-generalize. An explicit/unbundled encoding of a bundled-sugar feature is
frequently a faithful `(oracle)`-tagged interim. Sibling of named floor must be
grepped not assumed (a capability fork is answered by a landed precedent of the
exact shape) and kernel backed claim grep the emission not the name (grep the
producer/emission, not the named proxy).
