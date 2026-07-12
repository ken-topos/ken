# WP N2 — in-repo cross-file loader (ADR 0014 round 2, the spine)

Owner path: **spec enclave (Lane A) → Language team (Lane B)**. Design source of
truth: `docs/adr/0014-cross-package-resolution-and-fail-closed-collision.md`
(Accepted, no open forks), forks **MRES-1 + MRES-2 + MRES-3(a strict)**.
Program: `docs/program/wp/adr0014-work-program.md`. Size **M**. Deps: **none
hard** (acts on the import graph within one repo — no package manager, no
instance-visibility change), but **sequenced after N1** and is the **spine
N3/N4 depend on**.

Runs on the proven `#29` / N1 template: **Lane A (spec + conformance golden,
enclave) merges first; Lane B (build, Language) implements against the landed
spec.**

## Scope boundary — N2 is the PURE name-loader spine (READ FIRST)

The ADR's package-abstraction reshape assigns *some* loader work to N2 and
*some* to N4. **N2 lands only the cross-file `import`/name resolution spine.**
The following ride **N4** (they require the `package`/`admits` grammar, which
N4 introduces — they are not expressible until it exists), and are **out of
scope here**:

- **Source-vs-compiled package *detection*** and the **instance-manifest** —
  need the `package` abstraction (N4). N2 resolves cross-file *names*; instance
  dictionaries stay **ambient** exactly as today (MRES-4 option A), untouched.
- **Catalog package-file enrichment** (each importable package gets a `package`
  file) — the `package` file is N4 grammar; it lands with N4, not N2.
- **`admits` / `program` / `package` grammar and the admission gate** — all N4.

N2 therefore changes **name/module resolution only**: today a cross-file
`import` yields `UnboundName` (packages inline their helpers); after N2 it
resolves to another in-repo compilation unit. **Zero** instance-visibility,
`admits`, kernel, prelude, semantics, or `trusted_base()` delta.

**Two points the enclave should NOT second-guess (Architect-confirmed,
`evt_2506m0052hmen`):**

1. **N2 having no admission gate is not a soundness gap.**
   Ambient-with-coherence (MRES-4A) is the *sound baseline*: the orphan-ban +
   overlap coherence checks (ADR 0008) already run and are **unchanged** at N2.
   `admits` is an **auditability / explicitness** layer N4 lays on top — it is
   **not** what makes instances coherent. So widening name resolution across
   files with instances still ambient is coherent and sound on its own.
2. **No package files exist in-tree at N2 — there is nothing to detect.**
   Package files are authored by the N4 catalog-enrichment step. The loader
   discovers units purely by **`import` edges + the path↔file bijection**;
   package-file detection therefore **cannot and need not** be pulled forward.
   The dotted path uniquely identifies a file whether or not packaging exists
   later (MRES-2b path-rooting), so N4's boundaries compose **over** N2's global
   bijection without reworking it.

## Fixed inputs — SETTLED, do not reopen

Accepted in ADR 0014; the WP implements them, it does not relitigate:

- **MRES-1 (a).** Round one is an **in-repo cross-file loader only**. The
  content-addressed manifest / lockfile / registry (package manager) is a
  **later round** (couples to supply-chain `63`) — **out of scope**.
- **MRES-1 multi-catalog forward-compat.** The addressing must **not preclude**
  multiple catalog roots (standard + org + vendor). The in-repo loader is
  written so a single root is **one entry of a plural-ready root list**, never a
  hard-coded singleton; the resolution API takes the **plural** form. Multi-root
  resolution / precedence is **deferred** (package-manager round) — a data
  change, not a rewrite. Populate exactly **one** root now.
- **MRES-2.** Reuse the catalog WP's **total path↔file bijection** as the
  addressing. **Cycle = hard error** (matches the surface-diagnostic posture).
  **Lazy** discovery from **import edges**. **Cache on `ElabEnv`** (per-run
  module cache). Surface layer — no kernel / `trusted_base()` delta.
- **MRES-3 (a) strict.** Keep the **total, role-blind** path↔file bijection:
  strict leaf-file identity; importing a build unit does **not** implicitly open
  its nested namespaces. The relaxation (b) (nested logical namespaces without a
  forced 1:1 layout) is **recorded for a later round** if nested-namespace
  ergonomics bite — **not built here**.

## Lane A — spec + conformance golden (enclave) · merges FIRST

**Deliverable (mandated outline; each item ends in a concrete choice):**

1. **Loader semantics in `spec/30-surface/33` §3.2 (or the section that holds
   the surface module system).** State normatively:
   - **Path anchoring** — the total, role-blind path↔file bijection (cite the
     catalog WP's pinned bijection and MRES-3a); a cross-file `import M`
     resolves `M` to the unique file at its bijected path under a catalog root.
   - **Root list** — resolution is against a **root list** (plural), of which
     round one populates exactly one; record that multi-root precedence is a
     later round, so the normative rule is written over the plural form.
   - **Discovery** — **lazy**, driven by **import edges** (a unit is loaded when
     an import names it), not an eager whole-tree scan.
   - **Cycle** — an import cycle is a **hard error** with a specific diagnostic
     (name the cycle), not SCC-tolerant silent acceptance.
   - **Caching posture** — a compilation unit is loaded/elaborated **once per
     run** and cached (posture stated normatively; the cache *location*
     `ElabEnv` is a Lane-B implementation detail, not a spec mandate).
   Keep the wording **role-blind** (the bijection is about *paths*, not
   declaration kinds) and cite the ADR fork tags (MRES-1/2/3a).
2. **Do NOT spec instance visibility, `admits`, or the program/package
   abstraction.** Instances stay ambient (MRES-4A, unchanged); the boundary is
   **N4**. §3.3 import-shadowing (MRES-6) is **N3** — untouched here.
3. **Conformance golden.** Extend the appropriate `conformance/surface/…` seed
   with, at minimum: (a) a **cross-file `import`** that resolves a name defined
   in another in-repo unit → **accepted** (the name is now bound, not
   `UnboundName`); (b) an **import cycle** (A imports B imports A) → **rejected**
   with the specific cycle diagnostic; (c) the accept arm exercised **through
   the plural root API with a single populated root**, proving the plural form
   works with one root. (a) vs (b) must be a genuine **accept vs reject flip** on
   the cross-file-resolution axis (same surrounding context; resolvable import
   present vs a cycle), not green-vs-green. The (a) accept arm is **red until
   Lane B** (the loader has not landed) — mark it so, per the F3b conformance
   convention.

**AC (Lane A).** loader semantics normative and self-consistent; the strict
bijection + cycle-error + plural-root posture recorded; golden encodes the
cross-file accept + cycle reject + single-root-through-plural-API arms; the
accept arm is red-until-built (marked). Doc/spec/conformance-only; **no crates
delta**. Hand the SHA to Steward; Steward publishes; **Lane B unblocks on the
landed spec.**

## Lane B — build (Language team) · merges SECOND, on landed Lane A

**Deliverable.** An in-repo cross-file loader on `ElabEnv`:
- **Resolve** a cross-file `import M` to the in-repo unit at `M`'s bijected path
  under the (single, plural-ready) catalog root — replacing today's
  `UnboundName`.
- **Discover lazily** from import edges (load a unit when an import names it).
- **Detect cycles** and reject with a **specific error variant** (name the
  cycle) — assert the variant, not merely `is_err`.
- **Cache** each loaded/elaborated unit **once per run** on `ElabEnv`.
- **Plural-ready root API** — the root anchor is a **list**; populate one root;
  the resolution entry point takes the plural form so multi-root is a later data
  change, not a rewrite.

Package manager / content-addressing is **explicitly OUT** (couples to `63`).
Instance visibility is **untouched** (stays ambient; the `admits` boundary is
N4). No `package`/`program`/`admits` grammar.

**AC (Lane B).**
- A cross-file `import` resolves to the other in-repo unit (was `UnboundName`).
- An import cycle → rejected with the **specific** error variant (assert the
  variant).
- The single root resolves **through the plural root API** (the API is plural;
  one root is populated).
- The Lane A conformance golden's accept arm flips **red → green**; the cycle
  reject arm stays red-reject.
- Full `scripts/ken-cargo test --workspace` green **and** the literal CI command
  `cargo build --workspace --locked && cargo test --workspace --locked` green
  (N2 is resolution-**widening**, not rejection-adding, but confirm the literal
  CI oracle regardless — cross-file resolution can perturb in-tree
  examples/rosetta fixtures that previously relied on inlined helpers); `git
  diff --check` clean; scope = `crates/ken-elaborator` (+ tests) only; **zero**
  kernel / prelude / semantics / Cargo / lock / `trusted_base()` delta.

**Review.** Architect-terminal (loader design coherence, cycle-detection
completeness, cache correctness, the plural-ready root shape, and that instance
visibility / `admits` are genuinely untouched). CV's golden is the acceptance
oracle.

## Do-not-reopen guardrails

- **No package manager / content-addressing** — the persisted manifest,
  lockfile, and registry are the package-manager round (MRES-1(a); couples to
  `63`). N2 is in-repo only.
- **No `admits` / `program` / `package` grammar, no admission gate, no
  instance-manifest, no source-vs-compiled detection** — all **N4**. N2 does not
  touch instance visibility; instances stay ambient (MRES-4A).
- **No §3.3 import-shadowing reversal** — that is **N3** (MRES-6). N2 resolves
  cross-file names; it does not change local-vs-import clash behavior.
- **Strict bijection preserved** — do not relax toward nested logical namespaces
  (MRES-3 relaxation (b) is a later round).
- **Multi-root resolution/precedence deferred** — build the **plural-ready API**
  with **one** populated root; do not implement root precedence/ordering (that
  is the package-manager round).
