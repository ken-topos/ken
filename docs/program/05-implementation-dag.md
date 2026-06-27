# Implementation DAG (post-design-freeze)

> **For the Steward.** The open-decisions register is fully resolved
> (`spec/90-open-decisions.md`, ADRs 0005–0008), so the implementation sequence
> can now be pinned to the *settled* design. This is the work-package dependency
> graph to decompose into the catalog (`03-program-of-work.md`). It
> **refreshes** the WP-level dependency summary in `02-roadmap.md §dependency`
> for the settled design; `02`'s phase narrative and the gates G0–G8 still hold.
> Read `../PRINCIPLES.md` first — the small-TCB and reflect-don't-extend
> invariants shape the critical path.

## What changed vs. the old roadmap (the deltas to absorb)

1. **The kernel is observational (OTT), not cubical/`Id`** (ADR 0005). The K-WPs
   are Π/Σ/inductives/checked-universes **plus** `Eq`-by-type, `cast`,
   strict-prop **Ω**, set-quotients, truncation — and conversion computes those.
2. **Security is a tier-1 workstream (WS-Sec), not absent.** IFC, capabilities,
   supply-chain, trust-model, policy-as-code (`spec/60-security/`) become WPs
   that ride the effect system and the de Bruijn re-check.
3. **The behavioral seam is Ken's half of a two-engine system (WS-B).** Ken
   emits the assumption-boundary export, `Temporal`-as-data, and the trace
   contract (`spec/70-behavioral/`). **Ward** — the consuming engine — is a
   *sibling project* with its own roadmap, not a Ken WP.
4. **Decided disciplines land in existing WPs:** strict CBV-with-sharing (X1),
   obligation-generating overflow (L1), typeclasses-as-subobjects with canonical
   coherence (L-classes), read-optimized notation + a mandated formatter +
   confusable-resistant lexer (L-fmt), the reflective SMT certificate route
   (V3).

## Workstreams

The build spine **K → V → L → X** is unchanged; **F** is the always-on
foundation; **T** is tooling; **S** is deferred self-hosting; **R** is
never-a-gate research (all per `01-strategy §4`). Two workstreams are added:

- **WS-Sec — security (tier-1).** Rides the effect system + kernel;
  `60-security/`.
- **WS-B — behavioral seam (Ken's half).** Feeds the sibling **Ward**;
  `70-behavioral/`.

## The DAG, in dependency layers (waves)

Each wave may start once its predecessors' *interfaces* are stable, not their
completion — overlap aggressively where the table permits.

```
Wave 0  Foundations (WS-F)         F1 ─ F2(spec) ─ F3(ADRs) ─ F4(content-addr design)
                                        │  (F2/F3 largely DONE: spec written, ADRs 0001–0008)
                                        ▼
Wave 1  Kernel trust root (WS-K)   K1 ─ K2 ─ K2c ─ K-api      + K3(value model) ── X1(interp) ── V0(elab)
        + vertical slice           └────────────────────────────────────────────────────────┘  ► G1
                                        │
Wave 2  Verification (WS-V)         V1 ─ V2 ─ V3(prover) ─ V4(diag) ; T1(protocol)              ► G2+G3+G4
                                        │
Wave 3  Surface & stdlib (WS-L)     L1 L2 L3 L4 L5(effects) L6 L7 L8 ; L-classes ; L-fmt ; X2    ► G6
        (starts at G1, || Wave 2)        │
Wave 3s Security (WS-Sec)           Sec1(IFC)+Sec1ct(@ct) ─ Sec2(caps) ─ Sec3(supply) ─ Sec5(policy) ; Sec4(trust doc)
        (rides L5 + kernel)             │                                                         ► G-Sec
Wave 4  Behavioral seam (WS-B)      B1(export) ─ B2(Temporal) ─ B3(trace contract) ; B4(agentic doc)  ► G-Ward-seam
        (rides V + L5 + X)              │   → feeds sibling **Ward** (separate project)
Wave 5  Native & scale (WS-X)       X3(native backend) ─ X4(scale/limits)                          ► G5-perf
Wave 6  Self-hosting (WS-S)         S1(subset compiler) ─ S2(full self-host)                        ► G8
Wave 7  Ergonomics (WS-T)           T2(REPL) T3(test fw) T4(docs) T5(ecosystem)                     ► G5+G7
Parallel WS-R research              never a gate
```

## Work packages & dependencies (the table the Steward decomposes)

| WP | Deliverable (spec source) | Depends on | Notes |
|---|---|---|---|
| **F1** | repo / MIT / Rust workspace / IP hygiene | — | done/ongoing |
| **F2** | spec + conformance corpus (`spec/`) | F1 | **spec written**; corpus grows with WPs |
| **F3** | ADRs (`docs/adr/`) | F1 | 0001–0008 **landed** |
| **F4** | content-addressing + value-model design (`41`,`44`) | F1 | feeds K3 |
| **K1** | Π, dependent Σ, inductives, **checked non-cumulative universes** (`11–14`) | F2, F3 | OTT-ready core |
| **K2** | **observational layer**: `Eq`-by-type, `cast`, strict-prop **Ω**, set-quotients, truncation (`15–16`) | K1 | the ADR-0005 headline |
| **K2c** | conversion: lazy-WHNF NbE computing the obs ops + Ω proof-irrelevance + **SCT** (`17`) | K2 | decidable conversion |
| **K-api** | typing judgment + stable kernel API (`18`) | K2c | the TCB boundary |
| **K3** | content-addressed value model, O(1) eq, FNV+memcmp (`41`) | F4, K1 | no Leech on the hot path |
| **X1** | reference interpreter — **strict CBV + sharing** (`42`) | K1, K3 | the oracle |
| **V0** | minimal elaborator surface→kernel (`39`) | K1 | ► **G1** with X1 |
| **V1** | spec syntax: `requires`/`ensures`/refinements + **four-way status** (`21`) | V0 | `old` scoped to space ops |
| **V2** | obligation generation / VC (`22`) | K-api, V1 | |
| **V3** | prover: classifier + Z3 + **Kripke embedding + reflective certificate** (`23`) | V2 | route (a) target; recon hedge |
| **V4** | proof-failure diagnostics (`24`) | V2, T1 | countermodels, holes, `unknown` |
| **T1** | machine-readable diagnostic protocol (`25`) | V2 | schema early in F |
| **L1** | `Int`(arb-prec)/`Decimal`/fixed-width + **obligation-overflow** (`35`) | K1 | `OQ-1a` |
| **L2** | sum types, `match`, exhaustiveness, `Result`/`Option` (`34`) | L1 | |
| **L3** | strings, collections, generators/`Lazy` streams (`37`) | K1 | no coinduction |
| **L4** | modules + package manager, content-addressed lockfiles (`33`) | K1 | feeds Sec3 |
| **L5** | **effects**: static rows + interaction-tree denotation + capabilities + tail-resumptive handlers + `space`/message-passing (`36`) | K1 | the hub for WS-Sec/WS-B |
| **L6** | `Bytes` + binary I/O (`38`) | K1 | |
| **L7** | FFI (`38 §3`) | L6 | listed postulates |
| **L8** | curated lawful stdlib (`50`) | L1–L3, L-classes | |
| **L-classes** | typeclasses-as-subobjects, **canonical coherence**, orphan = error (`33 §5`,`39`) | K1, V0 | ADR 0008 |
| **L-fmt** | **mandated formatter** (ASCII↔Unicode canonical) + **confusable-resistant lexer** (TR39) (`31 §1a/§1b`) | V0 | read-optimized syntax |
| **X2** | content-addressed runtime hardening; engineering capacity; manual+region reclaim (`44`) | K3 | GC later, no language fork |
| **Sec1** | IFC: lattice-parametric non-interference **by typing**, labels on interaction-tree nodes (`61`) | L5 | DLM standard lattice |
| **Sec1ct** | `@ct` constant-time discipline (IFC to leakage sinks) (`61 §5a`) | Sec1 | timing guarantee → Ward |
| **Sec2** | capabilities: authority, attenuation, revocation, audit (`62`) | L5 | first-class tokens |
| **Sec3** | supply-chain: package/`.keni`, **re-check on consume**, provenance (sigstore/SLSA) (`63`) | L4, K-api | de Bruijn dividend |
| **Sec4** | trust-model doc + **published kernel audit** (`64`) | K-api | G5 input |
| **Sec5** | policy-as-code: separately-authored, compiler-enforced, non-weakenable (`65`) | Sec1, L4 | ADR 0007 |
| **B1** | assumption-boundary **export emitter** (generated `Q`/`P`/`Σ`/`T`/`G`, ITF) (`71`) | V1, L5 | broadcast contract |
| **B2** | `Temporal` datatype + surface notation (data, not modalities) (`72`) | L2, B1 | |
| **B3** | trace/instrumentation contract: `Σ`-events, correlation, runtime `Q`/`P`/`T` (`73`) | B1, X1 | feeds Ward monitor |
| **B4** | agentic boundary (envelope = Sec1/Sec2; metamorphic; RV) — mostly doc/conformance (`74`) | Sec1, Sec2, B3 | no new mechanism |
| **X3** | native backend (Cranelift/LLVM) behind the interpreter, differential-tested | X1, L-core | |
| **X4** | scale & limits validation; `space` runtime realization | X2 | loud at limits |
| **S1** | Stage-1 Ken-subset compiler atop the Rust kernel | L-complete | kernel stays Rust |
| **S2** | full self-hosted elaborator/codegen | S1 | |
| **T2** | REPL (the *Little Prover* loop) | V4, X1 | |
| **T3** | test/property framework | L2 | |
| **T4** | pedagogy + reference docs | G2 | |
| **T5** | package-ecosystem seeding | L4, T3 | |

## Critical path

**F2/F3 (done) → K1 → K2 → K2c → K-api → V1 → V2 → V3** is the spine to the
thesis (G2–G4): the kernel's observational core and conversion gate everything,
and the prover is the differentiator. **L5 (effects)** is the second pivot — it
is the hub that **WS-Sec and WS-B both hang off**, so sequence it early in Wave
3 (it only needs K1). Everything in WS-Sec and WS-B is blocked until L5's
interaction-tree denotation is stable.

## Gates (extending G0–G8)

The existing gates hold; security and the seam thread into them rather than
bolting on at the end (security is tier-1, not a finishing pass):

- **G-Sec** (new, lands within/just after G6): IFC **by-typing**
  non-interference is demonstrable on a real flow; a capability is
  attenuated+revoked; a dependency is **re-checked** on consume with its
  `trusted_base_delta` surfaced; one policy is compiler-enforced. *Acceptance
  ties to ADR 0004.*
- **G-Ward-seam** (new): Ken emits a reproducible export + trace contract a
  *stub* consumer can read; no `Ward` result is ever recorded as `proved`.
- **G5** (soundness) gains **Sec4** (the published kernel audit) as a named
  input.

## Ken vs. Ward (the project boundary)

Ken WPs stop at the **export and the trace contract** (B1–B3). **Ward** — the
model-checker / test-generator / runtime-monitor family that consumes them, plus
the deferred `OQ-sampling-policy` and `OQ-discharge-attestation` schemas — is a
**separate project** the Steward tracks as a sibling, brought up after the seam
(B1–B3) is stable. Design the two as one system (ADR 0006); build them as two.

## Sequencing guidance for the Steward

1. **Front-load the kernel observational core (K2/K2c).** It is the
   highest-risk, highest-rigor work and gates the whole spine; it is also where
   a feasibility doubt (if any) would surface — retire it early.
2. **Pull L5 forward.** As the WS-Sec/WS-B hub, a stable interaction-tree
   denotation unblocks two workstreams; treat it as a pivot, not a mid-Wave-3
   WP.
3. **Keep security in-band.** Sequence Sec1/Sec2 *with* the surface, not after —
   the IFC-as-effect-indexing design means they are nearly free riders on L5,
   and tier-1 status forbids deferring them to a finishing phase.
4. **Hold the Ken/Ward line.** Do not let WS-B grow Ward-side WPs into the Ken
   catalog; the seam (B1–B3) is the contract, Ward is the sibling.
5. **WS-R never gates.** Harvest research back through normal WPs only.
