# WP Sec1ct — `@ct` constant-time discipline (IFC to leakage sinks)

**Owner:** Team Verify (WS-Sec build, per the operator's WS-Sec→Verify routing).
**Branch:** `wp/Sec1ct-constant-time` (cut from `origin/main`).
**Stream / gate:** WS-Sec (tier-1) → **G-Sec**. **Depends on:** Sec1 (IFC by
typing) — **merged** (`947cdc6`, spec `61`, conformance `ifc/`). **Spec
source:**
`spec/60-security/61-information-flow.md §5a` (+ §4 declassify, §8 *decided*,
§9).

> **This is the Steward *frame*** — scope, settled-decision pinning, deliverable
> outline, acceptance, guardrails. The **spec enclave elaborates it to
> team-ready
> rigor** (spec `61 §5a` → implementation-grade + conformance) before Team
> Verify
> builds. **Any "current state" line below is perishable — verify it against the
> landed code/spec at pickup, do not build from this line** (promoted
> K2c-series-2: a stale "what exists" misdirects the build).

## 1. Objective (one line)

Build the **`@ct` (timing-sensitive) constant-time discipline**: a distinct,
opt-in label whose values may **never flow into a leakage-relevant effect sink**
(secret-dependent branch guard, memory index, or variable-time primitive),
enforced **by typing** as an ordinary IFC constraint on the existing lattice —
turning the §5a **hook** that Sec1 shipped into the enforced **discipline**.

## 2. Settled inputs — FIXED, do not reopen

These are **decided** (`61 §8` "Decided (`OQ-relational`/constant-time, §5a)",
ADR 0004/0001). Pin them; a weaker model must not relitigate them:

1. **`@ct` is a distinct, opt-in label, *separate* from `Secret`
   confidentiality**
   (`61 §5a`). Confidentiality constrains *where a value goes*; `@ct` constrains
   *where its influence steers*. A value may be `Secret` without `@ct` and vice
   versa; crypto keys are both. It is **another axis of the lattice-parametric
   IFC** (`OQ-ifc`) → **no new metatheory, no kernel enlargement**.
2. **Enforcement is unary taint by typing — NOT the relational engine, NO
   product
   programs** (`61 §5a`, §8). "A `@ct` value reaching a leakage sink is a type
   error" is the FaCT / ct-verif "secret types" result: a **sound static
   enforcement of the 2-safety property**. Do not build product-programs or a
   relational obligation for `@ct` (that is Sec1's by-proof mode `§5`, a
   *different* path).
3. **Leakage sinks are a distinguished effect-sink class** (`61 §5a`,
   `30-surface/36 §3`): secret-dependent **branch guard**, **memory index**,
   **variable-time primitive**. Riding the **existing interaction-tree effect
   machinery** (L5, merged) — no new kernel construct.
4. **The sensitive span = the label's live span** — introduced at the secret
   source, ended by an **authorised `declassify`** (`61 §4`). **No
   `constant_time { … }` region construct** (a padding/balancing region is a
   *runtime mitigation* = Ward's, not Ken's).
5. **The honest split is fixed** (`61 §5a`, §8; `64 §4.2`, `63 §5a`): Ken
   statically guarantees the **source-level precondition** (no secret-dependent
   leakage op), exported as an assumption-boundary **`Q`**. The **timing
   guarantee itself** (codegen/hardware-relative, leakage-model + platform) is
   **delegated to Ward + the toolchain** — Ken does **not** own it and must
   **not** claim it. Honest-limits table `61 §H` is the discipline.
6. **Signature-level CT promise:** a function exports a *constant-time-in-this-
   parameter* promise for boundary checking + the Ward runtime-validation
   requirement (`63 §5a`). The label lattice, declassify, and labeled
   types/effects from **Sec1 are reused, not rebuilt**.

## 3. Mandated deliverable outline (each item ends in an implementable choice)

The enclave elaborates the spec; the brief fixes the shape. Deliver, in
`crates/ken-elaborator` (the Sec1 IFC home, `ifc.rs`) + spec `61 §5a`:

1. **The `@ct` lattice axis.** Pin how `@ct` rides the existing lattice as a
   distinct axis (product with the confidentiality/integrity axes), its `⊑`/join
   behavior, and that an un-annotated value is `@ct`-bottom (not timing-
   sensitive). Concrete: the label-type extension + where it parses
   (`30-surface`
   label syntax) — **value-set + invariants pinned, the literal token spelling
   `(oracle)`-tagged if the surface form isn't yet locked** (per
   assert-at-locked-granularity).
2. **The leakage-sink classification.** Enumerate the leakage-relevant sinks as
   a
   **sealed set** (branch-guard / memory-index / var-time-primitive) with **NO
   `_ => non-sink` catch-all** — adding an effect that can leak timing without
   classifying it must be a **compile error**, not a silent non-sink
   (exhaustive-by-construction, COORDINATION §7). Name each sink's exact
   trigger.
3. **The flow rule.** One rule: `@ct`-tainted value into a leakage sink →
   **reject** (type error), `pc`-style influence tracking reused from Sec1's
   implicit-flow handling (`61 §3`). Each path backed by a reject case.
4. **The CT signature promise + `Q` export.** How a function declares
   constant-time-in-parameter-`x`, how it is checked at the boundary, and the
   exact `Q` it emits to the assumption boundary (B1's contract — coordinate the
   shape, do not pre-bind names; `(oracle)`-tag deferred tokens).
5. **Declassify ends the span.** Reuse `61 §4` declassify as the *only* span
   terminator; show a `@ct` value's influence is unconstrained after an
   authorised declassify.
6. **Honest-limits update (`61 §H`).** State exactly what Ken proves (source
   precondition `Q`, trusted-by-typing) vs. delegates (timing → Ward, under a
   stated leakage model). **Carry the three Architect `[Sec1-*]` reify-triggers
   into §H as named scoped work** ([Sec1-dual] IntegLabel ordering,
   [Sec1-launder]
   real `Vis`-routed label preservation, [Sec1-reduce] `Φ_post` reduction-
   faithfulness) — they are the kernel-blind surfaces the *next* increment nets;
   name them, do not silently absorb.

## 4. Testable acceptance criteria

- **AC1** A `@ct` value used as a **secret-dependent branch guard** is a
  **compile
  error** (reject). Correct (no such branch) accepts. *Verdict flips*
  (right=accept
  / wrong=reject) — not green-vs-green (discriminating-verdict).
- **AC2** A `@ct` value used as a **secret-dependent memory index** → reject.
- **AC3** A `@ct` value into a **variable-time primitive** sink → reject.
- **AC4** A value `Secret` but **not** `@ct` may branch freely (the two axes are
  independent — AC4 must flip while AC1 stays green, the [Sec1-dual] discipline:
  a real distinguishing case, not a degenerate one).
- **AC5** After an authorised `declassify`, the formerly-`@ct` value's influence
  into a sink is **accepted** (span ended) — and the declassify shows in the
  `trusted_base_delta` (`61 §4`).
- **AC6** A function's **CT-in-parameter signature promise** is checked: a body
  that leaks on that parameter is rejected; the accepted function **emits the
  `Q`**
  (structural assertion on the emitted boundary obligation — not just
  "accepts").
- **AC7 (honesty / no over-claim)** The timing guarantee is **delegated to
  Ward**:
  no test/claim asserts Ken proves constant-time *execution*; `61 §H` carries
  the
  split + the three `[Sec1-*]` triggers. (Mirror of the Sec1-build honesty gate:
  a kernel-blind / delegated surface gets a named deferral with a reify-trigger,
  never false coverage.)
- **Conformance:** `conformance/security/ifc/` extended (or a `ct/` sibling)
  with
  the AC1–AC7 cases; **per-case verdict-flip + a cross-case sweep** (the `@ct`
  reject class agrees; the Secret-not-`@ct` accept class agrees). **QA gate (the
  fresh 2-team build-qa lesson):** each negative case must **route through the
  actual leakage sink**, not *predicate about* a synthetic label — a test that
  asserts `is_ct(lit) && is_sink(op)` over hand-assigned literals guards
  nothing;
  the test must drive a real `@ct` value into a real sink and observe the
  reject.

## 5. Do-not-reopen guardrails

- **No product programs / no relational engine for `@ct`** — it is unary taint
  by
  typing (§2.2). The relational by-proof mode (`61 §5`) is Sec1's, for bespoke
  non-interference claims; do not conflate.
- **No `constant_time { }` region construct** (§2.4) — span = label lifecycle.
- **No kernel enlargement** — labels erase before the kernel; the flow rules are
  **trusted, conformance is the net** (the N1 two-soundnesses lesson — enumerate
  what the kernel does not see; the `@ct`→sink rule is erased, so the
  discriminating conformance cases ARE the trust boundary).
- **Do not claim the timing guarantee** — Ken owns `Q`, Ward owns the binary
  (§2.5). Over-claiming here is the exact Sec1-build trap the Architect caught.
- **Reuse Sec1** — lattice, declassify, labeled effects, `pc` tracking are
  merged; extend, don't rebuild.

## 6. Sequencing notes

- Sec1ct is **first in the WS-Sec queue** after Sec1; **Sec2 (capabilities)
  follows** on Team Verify. The CT signature promise (`Q`) couples to **B1**
  (the
  export emitter, Kernel/WS-B) — coordinate the `Q` shape via the spec, do not
  hard-bind field names cross-WP (defer-spelling-not-concept).
- Standard pipeline: this frame → **spec-leader elaborates** `61 §5a` to
  implementation grade + conformance → merges to `main` → **Team Verify
  compacted, then kicked off** on this branch.
