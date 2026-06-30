# WP L7 — the `foreign` FFI and the trust boundary

**Owner:** Team Foundation (L-stream). **Branch:** `wp/L7-ffi` (cut from
`origin/main`). **Stream / gate:** L-stream → **G6** (≥1 FFI call in the
verified
component, trust base showing what is assumed). **Depends on:** L6 (`Bytes` +
`(ptr,len)` marshalling) — merged; B1 (the export's `trusted_base_delta`) —
merged; couples to Sec2 (capabilities gate foreign calls). **Spec source:**
`spec/30-surface/38-ffi-io.md §2–§4` (+ `11 §4`/`18 §5` postulates, `25 §3`
trusted base, `21 §5` runtime contracts, `36` effects).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `38 §2–§4` to team-ready
> rigor + conformance before Team Foundation builds. **Perishable:** the
> foreign-postulate path rides the **landed** B1 export `trusted_base_delta` and
> the kernel's postulate machinery (`18 §5`) — pin against the code, not this
> line.

## 1. Objective (one line)

Deliver Ken's **`foreign` FFI** — typed, effect-rowed C-ABI bindings — and the
**trust-boundary discipline** that marks, *honestly and visibly*, exactly where
Ken's guarantees stop: a foreign function is a **listed postulate**, `pure` is a
**claim not a check**, boundary contracts become **runtime-checked**, and
effects
are **mandatory**.

## 2. Settled inputs — FIXED, do not reopen

Per `38 §2–§4` (the trust boundary is "the load-bearing part"):

1. **A `foreign` decl binds a Ken name to a C-ABI symbol (§2)** with a **Ken
   type**
   + an **effect row** (`pure` ≡ empty row), `symbol`/`library` attributes.
   C-ABI
   marshalling follows the primitive lowering (`41`): scalars as machine types,
   **`Bytes` as `(ptr, len)`** (the L6 boundary). A **general** FFI, **not** a
   fixed allowlist of externals.
2. **A `foreign` function is a POSTULATE (§3, `11 §4`/`18 §5`).** The kernel
   cannot check C, so its type is **assumed** and it appears in the **trusted
   base / `trusted_base_delta`** (`25 §3`, the B1 export). A verified artifact's
   trust base **lists exactly which foreign functions (and assumed contracts) it
   relies on — visible, not hidden.** (This is the honesty-about-the-boundary
   principle made structural.)
3. **`pure` on a `foreign` is a CLAIM, not a check (§3).** Declaring purity
   asserts referential transparency the kernel **cannot** verify; it is part of
   the trusted boundary. A wrong `pure` is a soundness bug **confined to that
   postulate — and it is *listed*.**
4. **Boundary contracts are runtime-checked (§3, `21 §5`).**
   `requires`/`ensures`
   on a `foreign`, where statically unprovable, become **runtime-checked**
   fail-fast assertions — the place runtime contracts earn their keep (untrusted
   input, FFI results).
5. **Effects are MANDATORY at the boundary (§3).** Any `foreign`/I/O that
   touches
   the world MUST carry the appropriate effect row (`36`); a **`pure`-but-
   effectful** foreign is the one gap the discipline cannot catch — the reviewer
   must (flag it as the named residual).
6. **Capabilities gate foreign calls (§4).** I/O effects (`FS`/`Net`/…) gate
   which
   foreign functions a computation may call (couples to **Sec2**); the verified
   core stays pure, the trusted boundary a **small, enumerated set** of
   postulates
   + capabilities.

## 3. Mandated deliverable outline (each ends in an implementable choice)

Deliver the FFI surface + the trust-boundary plumbing:

1. **The `foreign` declaration + C-ABI marshalling.** Parse/elaborate `foreign f
   :
   T = symbol "…" library "…" [pure]`; bind the Ken type + effect row; marshal
   scalars↔machine types and `Bytes`↔`(ptr,len)` (reuse L6). Pin the decl form +
   the marshalling table.
2. **Foreign-as-postulate → trusted base.** A `foreign` decl emits a
   **postulate**
   (`18 §5`) whose type is assumed; it appears in **`trusted_base_delta`** (the
   B1
   export) so the artifact's trust base **lists** it. Pin the wiring to the
   landed
   export.
3. **`pure`-as-claim + the residual gap.** `pure` records an **unverified**
   claim
   in the trusted boundary (never a kernel certification — projects to
   *trusted*,
   never `Q`, per B1's discriminator). Name the `pure`-but-effectful residual as
   a reviewer-surfaced flag.
4. **Boundary contracts → runtime checks.** `requires`/`ensures` on a `foreign`
   that are statically unprovable lower to **runtime-checked** assertions (`21
   §5`), fail-fast.
5. **Capability/effect gating.** A `foreign` world-action requires its effect
   row
   (`36`) + the gating capability (couples to Sec2's `Cap_FS`/`Cap_Net`).

## 4. Testable acceptance criteria

- **AC1 (`foreign` binds + marshals)** A `foreign` decl elaborates to a typed,
  effect-rowed binding; a call marshals `Bytes`↔`(ptr,len)` + scalars↔machine
  types (structural on the binding/marshalling, not "compiles").
- **AC2 (foreign-as-listed-postulate — the honesty headline, discriminating)** A
  verified artifact that **relies on** a `foreign f` has `f`'s postulate in its
  **`trusted_base_delta`**; one that does **not** rely on it does **not** —
  assert
  the foreign postulate **shows up in the trust base** (visible), route through
  the
  **real** B1 export. Verdict flips (relied-on listed / not-relied-on absent).
- **AC3 (`pure` is a claim, projects to *trusted* not `Q`)** A `pure foreign`'s
  guarantee is **assumed**, not kernel-certified — it lands in the trusted base
  and **never** projects to `Q` (B1's discriminator; under-claim is the safe
  direction). A wrong `pure` is confined to that postulate.
- **AC4 (boundary contracts runtime-checked)** A statically-unprovable
  `requires`/`ensures` on a `foreign` becomes a **runtime-checked** assertion
  (fail-fast), not a silently-assumed one — observe the emitted runtime check.
- **AC5 (effects mandatory — discriminating)** A world-touching `foreign`
  **without**
  its effect row is rejected; **with** the row, accepts. The
  `pure`-but-effectful
  case is the named residual the type discipline cannot catch (flagged, not
  silently accepted as sound).
- **Conformance:** `conformance/surface/ffi-io/` — AC1–AC5 + a **serialization
  round-trip proof** (G6) and a `foreign` decl whose postulate appears in
  `trusted_base_delta`. **QA gate:** AC2/AC3 route a **real** `foreign` decl
  through the **actual** export/trust-base machinery (postulate listed, never
  `Q`); no synthetic trust-base literal.

## 5. Do-not-reopen guardrails

- **`foreign` is explicitly-trusted** (`18 §5` postulate) — its type is
  **assumed +
  LISTED** in `trusted_base_delta`, visible not hidden (§2.2).
  Honesty-about-the-
  boundary is the core.
- **`pure` is a claim, never a kernel check** (§2.3) — projects to *trusted*,
  never
  `Q`; a wrong `pure` is confined + listed.
- **Effects mandatory at the boundary** (§2.5) — no untracked foreign
  world-action;
  the `pure`-but-effectful residual is named, not hidden.
- **The trusted boundary is small + enumerated** (§2.6) — the verified core
  stays
  pure; FFI is general but every use is a listed concession.
- **Marshalling reuses L6** — `Bytes` as `(ptr,len)`; don't re-derive.

## 6. Sequencing notes

- L7 is the **final trust-boundary piece** of the L-stream — it closes G6 (a
  verified component making ≥1 FFI call with the trust base showing what's
  assumed). Couples to **B1** (the `trusted_base_delta` listing) and **Sec2**
  (capability-gated foreign calls) — keep both seams clean.
- The `pure`-but-effectful residual is the **one** soundness gap the type
  discipline names but cannot mechanically catch — pin it as a reviewer-surfaced
  flag, the honest limit (`64`).
- Standard §2c: frame → spec-leader elaborates `38 §2–§4` + conformance → merge
  (Architect + conformance-validator) → Team Foundation compacted, then kicked
  off.
