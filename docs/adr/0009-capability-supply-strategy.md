# ADR 0009 — Capability supply strategy: bracket the untrusted, expose verification as a typed, governed choice

- **Status:** Accepted
- **Date:** 2026-07-02
- **Deciders:** the operator

## Context

Ken is spec-complete but its *interpretation* is thin: the Rosetta campaign
found a language that type-checks yet has little attached functionality — "a
first-order theory with no interpretation." The BUILTINS Phase-1 audit
(`spec/10-kernel/18a-primitive-registry.md`) then made the recurring shape
explicit: for many capabilities the **simple specification and the fast, real
implementation diverge widely.** Arithmetic is the mild case (bignum vs Peano);
regex is the vivid one (the language-theoretic definition is trivial; the
state-machine compiler/executor that is actually used is complex, and the
PCRE-flavoured extensions people want — backreferences, lookaround — are not
even regular, so they have *no* clean spec at all). SMT, crypto, collections,
serialization all share this shape.

For every such capability there is a spectrum of ways to supply it, trading
**security / trust** against **time-to-usefulness**, **performance**, and
**proof (token) cost**:

1. an **opaque foreign** implementation (an external engine via FFI, admitted
   as a `declare_postulate` with *no* stated property);
2. the same, but with a **spec'd axiom** (`match p s ↔ s ∈ L(p)`) asserted;
3. a **certifying** fast engine whose output a small **proved checker**
   re-validates (proof-carrying code);
4. a **real algorithm in Ken, tested-not-trusted** (netted by a differential
   corpus, the posture the interpreter already has);
5. a **real algorithm in Ken, fully proved**;
6. a **naive matcher, fully proved** (cheap proof, unusable performance).

The naming matters: the kernel is fixed Rust and **never grows**; what a foreign
or postulated component grows is `trusted_base()` — a **visible, enumerated,
audited** entry, not a hidden kernel edit (`18 §5`, `63 §2`, PRINCIPLES §5). The
small-TCB principle is about that set being small and *auditable*, not empty.

Two observations reframe the whole spectrum:

- **The trust a capability must buy is a function of where its result is
  *consumed*, not of what the algorithm is.** A wrong result is catastrophic
  only if it can reach a *proof-relevant* position; gated to the outer ring it
  is merely a wrong *value*. This is the same reachability-precondition that
  made the BUILTINS `Int`/`Decimal` bugs (`18a §4`) wrong-values-not-false-proofs:
  the kernel keeps `Eq` at a primitive type neutral and has no evaluator
  dependency, so an evaluator result **cannot** inhabit a false kernel proof.
- **An untrusted component embedded in Ken is strictly better than the same
  component in an unverified language** — because Ken verifies the two things
  *bracketing* it even when it cannot verify its internals: the **provenance** of
  its inputs (IFC labels, capabilities, `@ct`, taint — ADR 0004) and the **use**
  of its results (reachability-gating, label propagation, policy — ADR 0007).
  The untrusted core is sandwiched between a verified pre-condition and a
  verified post-condition, so embedding it is a *monotonic* improvement over
  baseline at zero internal verification.

Together these reframe supplying functionality as **curation, not
construction.** Because the bracket makes an embedded untrusted component
strictly better than the same component in an unverified language, the dominant
move for most capabilities is to **select a component the software industry has
already battle-tested** — a mature regex engine, a widely-audited SMT solver, an
established crypto library — vendor and pin it (`63`), and bracket it. That is
simultaneously **net better** (the bracket adds provenance-and-use guarantees no
other language offers), **low token cost** (no from-scratch authoring or proof),
and **fast to deliver** (reuse over reinvention). Construction — writing and
proving a capability in Ken (tiers b/c) — is reserved for where curation cannot
buy the trust the consumption position needs *and* the value justifies the proof
project.

## Decision

A single strategy governs how every builtin/package/foreign capability is
supplied. It is the operating policy behind PRINCIPLES §5 (small TCB), §6
(reflect-don't-extend), §8 (honesty about the boundary), and §11 (security as
substrate).

1. **Bracket the untrusted.** Prefer bounding a component's *boundary* over
   verifying its *internals*. An opaque component is admissible when Ken
   verifies (a) what may reach it (provenance) and (b) where its result may flow
   (use) — in particular that the result is **gated out of proof-relevant
   positions**, so a wrong result is a wrong value, never a false proof. Safety
   comes from the bracket, not the picture.

2. **Opaque, never axiomatized.** Keep an untrusted/foreign component **opaque**
   — its result usable as a *value*, proof-gated. Do **not** assert a spec
   property (a spec'd axiom / `foreign` with a stated law) to make the result
   usable *in a proof*. The axiom re-couples the blast radius to the unverified
   internals (it voids the bracket), and for a real engine it is usually **false**
   (the implementation is not its clean spec — PCRE is not a regular language;
   an engine has bugs). Proof-relevance is **earned** (by certification or
   proof), never **asserted**. Opaque-now is also the *migration-compatible*
   choice: a later proof **adds** the law under the same signature; a false
   axiom shipped now would contradict it.

3. **Three tiers, one interface; migrate upward.** A capability is supplied at
   the **lowest tier that unblocks its use**, behind a **stable interface**, and
   migrated upward only as the value justifies:
   - **(a) opaque-foreign, bracketed** — fastest to usefulness, native speed,
     one audited `trusted_base()` line, proof-gated;
   - **(b) tested-not-trusted / certified** — a real algorithm netted by a
     differential corpus, or a fast untrusted producer behind a small **proved
     checker** (it is cheaper to prove a checker than a solver, and the checker
     is reusable across engines);
   - **(c) proved-native** — kernel-level trust *and* native speed.
   Tier (c)'s only cost — proved-Ken is slow — is **dissolved by native
   compilation** (`spec/40-runtime/45`, the X-series): a proved Ken engine that
   compiles to native is as fast as the C one and carries kernel trust. The
   native backend is therefore the **enabler of the verified-library endgame**,
   not merely a performance feature.

4. **Verification is a typed, defaulted, governed choice.** Where a capability
   admits both a **provable core** and an **unprovable-but-expressive extension**
   (regular vs PCRE regex; exact `Int` vs IEEE `Float`), expose them as
   **distinct types that carry the tradeoff on their face**, default to the
   provable core, and let **policy-as-code (ADR 0007) + capabilities (ADR 0004)**
   mandate or permit tiers per trust-domain (e.g. "no unprovable regex on the
   auth path" becomes a checkable policy, not a code-review hope). The unprovable
   extension is a **feature** — an explicit, legible, enforceable engineering
   choice with its cost in view — not a defect. **An interface must separate the
   provable core from the unprovable extension** so the migratable part can
   migrate; a monolithic surface that mixes them can never migrate any of itself.

5. **The supply rubric** (how to choose the component *and* tier for a
   capability X — **curate before you construct**):
   1. **Is there a battle-tested external component for X?** If a mature,
      widely-adopted, well-audited implementation exists, the default is to
      **curate** it — vendor, pin, re-check on update (`63`) — and bracket it
      opaque (tier a). Its *earned industry trust* is the selection criterion;
      construction is justified only when a later question forces higher trust
      and the value funds it.
   2. **Is X's spec cleanly statable in Ken?** No → proof is off the table;
      choose among {opaque-foreign, tested-not-trusted}.
   3. **Is X's result cheaply *checkable* vs computing it?** (sort → sorted +
      permutation; SAT → the assignment; regex-positive → the derivation) Yes →
      **certification (tier b) is the sweet spot** — fast *and* kernel-trusted.
   4. **Can X's result reach a proof-relevant position?** No (gated) →
      tested-not-trusted/opaque is soundness-safe at zero trust cost. Yes → it
      must be proved, certified, or a *sound* axiom.
   5. **Perf class?** Cold path → naive-proved is fine. Hot path → tier (b)/(c).
   6. **Proof budget?** Proving the fast algorithm is a real project;
      certification amortizes; opaque-foreign is cheapest up front but a
      permanent trusted-base line.

## Consequences

- **Every builtin/package/foreign WP frame cites this ADR**, and the PRINCIPLES
  §5 burden-of-proof for a `trusted_base()` addition extends to: *why not
  bracketed-opaque, or certified-over-untrusted, or tested-not-trusted-gated?*
  Native/trusted status is the exception that must be earned (the BUILTINS
  adversarial default), not the default.
- **Curation is the default supply mode.** Filling in Ken's functionality is
  primarily a *curation* exercise — sourcing and bracketing components the
  software industry already trusts — not a *construction* one. A WP proposing to
  author (and prove) a capability from scratch must justify why curate-and-bracket
  is insufficient: a proof-relevant consumption position curation cannot reach, no
  trustworthy external artifact, or a value that funds the proof project. This is
  what makes the strategy *net better, low token cost, and fast to deliver* at
  once.
- **Type names and APIs must carry the tradeoff on their face** (PRINCIPLES §8
  at the API level): a `PcreRegex` must telegraph "not provable, can ReDoS"
  distinctly from a provable `Regex`, or the "explicit choice" is theoretical.
- **The native backend (X-series, `45`) gains strategic weight**: it is the
  tier-(c) enabler, so the verified-library endgame is gated on it — proved-fast
  replaces bracketed-opaque behind stable interfaces over time, retiring
  trusted-base entries wholesale.
- **Already instantiated.** The BUILTINS registry (`18a`) *is* this policy
  applied to arithmetic: `Float` equality is `NATIVE`-but-non-proof while `Int`
  is provable (the typed choice, §5.4); the demotions
  (`Decimal`/`Char`/`Bool`/`checked`/`saturating` → derived) are tier migration
  under the adversarial burden; `String` stays native on a real cliff. The
  registry validated the policy before the policy was written.
- **Coupling to §63.** An external binary is a supply-chain dependency —
  vendored, pinned, and re-checked on update per `spec/60-security/63` ("consume
  = re-check"); the certification pattern (tier b) is that same re-check
  discipline applied *inside* a library (the kernel's own architecture, one
  scale down).

## Relation to prior decisions

Extends PRINCIPLES §5/§6/§8/§11; builds on ADR 0004 (IFC/capabilities as the
bracketing substrate) and ADR 0007 (policy-as-code as the governance dial);
depends on the X-series native backend (`45`) for the tier-(c) endgame; the
supply-chain re-check story is `spec/60-security/63`.
