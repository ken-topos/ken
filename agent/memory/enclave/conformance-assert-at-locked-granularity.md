---
scope: enclave
audience: (see scope README)
source: private memory `conformance-assert-at-locked-granularity`
---

# A conformance case must assert at the spec's locked granularity

A conformance case codifies the contract the whole build fleet codes against, so
its **granularity** must match the spec's exactly — assert **neither looser nor
tighter** than what the spec locks normatively.

**T1 (2026-06-30, `25-protocol.md` wire serialization, shipped `1e9448c`).** The
spec simultaneously (a) locked `countermodel.verdict`'s **concept + value-set**
`{false,unknown}` and said "**renaming** it **fails**" (§6/§9), and (b) deferred
the literal JSON field **names** to the agent-team software, finalized later
(§preamble/§8). Authoring naively, I'd have pinned the literal field *spellings*
as normative `expect:` values — **over-freezing a deferred degree of freedom**.
Those cases would falsely fail (or block a valid implementation) the moment the
names finalize: a **wrong conformance case** licensing nothing. I surfaced it as
a pre-authoring **lock-point (LP-1)**; spec-author + spec-leader both confirmed
the reading. The resolving discriminator:

> the **stable surface** the spec locks is the **concept + value-set + >
cross-field invariant**, **not** the literal token.

So the seed asserts value-sets (`status ∈ {proved,disproved,incomplete}`,
`verdict ∈ {false,unknown}`, `kind ∈ {4}`, `region ∈ {2}`), the cross-walk
agreement, id/`hole_id`-stability, and `trusted_base_delta`-emptiness — and
**`(oracle)`-tags every literal field spelling** (blanket header note), to be
reconciled against the finalized names when the agent-team software lands. The
value-set/invariant assertions stand regardless of the final spelling.

**Why this is a new trap-class.** V1–V4's traps were **metatheory-fidelity**
(false-vs-unknown, the Glivenko boundary — conformance reconcile inherits spec
metatheory bugs). This is **normative-scope**: the case is *semantically* right
but pitched at the wrong **granularity**. It is the **dual** of the K2 "pin the
level precisely" discipline (skill playbook): there the failure is
*under*-specifying — a loose `Omega, level-poly` that hides impredicativity;
here it is *over*-specifying — freezing a token the spec deliberately left free.
One unifying rule:

> **Match the spec's exact granularity.** Pin every degree of freedom the spec >
**locks**; `(oracle)`-tag every degree of freedom it **defers**. Read "what is >
normative here?" precisely before writing the `expect:`.

**How to apply.** When grounding a conformance seed, for each value a case
asserts, ask: *does the spec lock this exact value, or only the
concept/value-set it draws from?* A spec phrase like "a reference finalized with
X", "the exact names are OQ-…", "normative for the shape, not the spelling" is
the tell — pin the locked layer (value-set + invariants), `(oracle)`-tag the
deferred layer (the conformance oracle grounding fallback mechanism, used here
for a *spec-deferred* token rather than an un-mounted oracle). Surface it as a
**pre-authoring lock-point** — the design-time-lock-point protocol (conformance
reconcile inherits spec metatheory bugs) held a 2nd consecutive WP (V4 → T1,
2/2) and generalizes beyond metatheory traps to this scope trap: 16/16
first-pass, zero rework.
