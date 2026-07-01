# Supply-chain security

> Status: **DRAFT v0**. Normative for the consumption model and the artifact
> shape. **`OQ-provenance` DECIDED** (operator, 2026-06-27): keyless
> **sigstore/cosign** signing + **in-toto/SLSA** attestation, the two ladders
> kept distinct (§5), plus **policy attestation** (the governing policy travels
> in provenance, §2/§3). How a Ken package is consumed *safely* — the area where
> Ken's de Bruijn criterion pays an unusual dividend. ADR 0004 Decision 5.

## 1. The Ken superpower: consume = **re-check**, not re-prove, not re-trust

In most ecosystems, consuming a dependency means **trusting** that its author
built it honestly. In Ken, consuming a *verified* dependency means your **own
kernel re-checks its proof terms**. Trust does not flow from the author's word;
it flows from your kernel's re-verification (`64-trust-model.md`).

Two timings must not be confused:

- **Re-prove** (expensive) — re-run the prover (Z3, tactics) to *find* proofs.
  **Not needed on consume.**
- **Re-check** (cheap — checking ≪ proving) — re-run the **kernel** over the
  shipped proof terms to confirm `Γ ⊢ p : φ`. **This is what a consumer does.**

A package's proof terms are first-class, serialisable data; the consumer's
kernel loads them, **re-checks** them, and only then extends its environment `Σ`
(`../10-kernel/11 §4`) with the verified fragment. A tampered or fabricated
proof term **fails the re-check** — the author cannot ship a binary that merely
*claims* to be verified.

## 2. The package artifact

A Ken package is a content-addressed bundle:

```
ken-package :=
  ( source         -- content-hashed (ken.toml + ken.lock pin it, 33 §3)
  , artifact       -- compiled bytecode / native, content-hashed
  , interface      -- a compiled `.keni`: exported types + contract certificates
  , proof-bundle   -- the Σ-fragment: proof terms the consumer re-checks
  , trusted_base_delta   -- every postulate/hole/declassify-authority assumed
  , provenance     -- sigstore signature + in-toto/SLSA attestation
                   --   + the GOVERNING POLICY hash/version (61, 65)
  )
```

- **The interface file (`.keni`)** carries the *types* of exported definitions
  and the *certificates* for their contracts/refinements — what a consumer's
  kernel needs to add them to `Σ` — but **not** the proof bodies of internal
  lemmas the consumer doesn't depend on. This is the Coq/Agda/Lean
  `.vo`/`.agdai`/`.olean` pattern, and it is the format for consuming a binary
  package without re-verifying its internals while still **re-checking the
  contracts you rely on**.
- **`trusted_base_delta`** (`../20-verification/25 §3`) is the consume-time
  attestation: it lists every postulate, open hole, **FFI postulate**
  (`../30-surface/38 §3`), and **declassification authority** (`61 §4`) the
  package introduces. *A genuinely-verified, fully-confined package has an empty
  delta.*

## 3. The consume flow

```
1. Resolve dependency at its content hash (ken.lock).
2. Fetch (source/)artifact + .keni + proof-bundle + delta + provenance.
3. Verify content hash matches the lock           — identity (33 §3).
4. Verify provenance signature + SLSA attestation  — origin (OQ-provenance §5).
5. Kernel RE-CHECKS the proof-bundle / .keni certificates.   ← the trust step
6. Audit trusted_base_delta against your policy
     (empty? acceptable postulates? acceptable declassifications/FFI?)
     AND check the dep's governing policy is COMPATIBLE — monotone-tightening:
     a dependency's policy may be stricter than yours, never laxer (65).
7. Only on all-pass: extend your Σ with the verified fragment.
8. Your module's new obligations discharge against the extended Σ.
```

Step 5 is where trust is established **by your kernel**, not the author. Steps
3–4 add *identity* and *origin*; step 5 adds *verified correctness*; step 6
makes the residual *assumptions* a policy decision **and** confirms the
dependency honoured a policy compatible with yours (`65-policy.md`). Together:
**"this build provably honoured org policy `vX`"** is checkable across the whole
dependency graph.

## 4. Threat model (what each step defends)

| Threat | Ken's mitigation | Residual / needs |
|---|---|---|
| Tampered source | content-hash pin (`ken.lock`) | signing (§5) for *origin*, not just identity |
| Tampered binary / fake "verified" | proof terms **re-checked** by your kernel (§1) — fail closed | — |
| Compiler bug → wrong binary from right source | reproducible builds (same source+compiler ⇒ same hash) | hermetic build + SLSA attestation (§5) |
| Prover bug → "proves" something false | de Bruijn criterion: your kernel re-checks (`64`) | kernel audit (`64`) |
| Hidden postulates / holes / declassify | `trusted_base_delta` lists all (§2) | consumer **policy** on acceptable deltas |
| Malicious FFI | every `foreign` is a listed postulate; effects + clearance gate it (`61`,`62`) | the C code itself stays unverified |
| Dependency confusion / substitution | content-addressing: same name + different content = different hash | registry **namespace ownership** (§5) |
| Secret exfiltration by a dependency | **IFC** (`61`): a dep cannot flow your `Secret` data to its `Net` sink without an authorised, audited declassify | — |

The last row is new with ADR 0004 and is significant: with information-flow
control, **a dependency physically cannot exfiltrate labeled data** it was
given, absent an explicit declassification that shows in its delta. That is a
structural answer to the "malicious package phones home with your secrets"
attack.

## 5. Provenance — the complementary axis (`OQ-provenance` DECIDED)

Content-addressing proves **identity** ("this is the artifact with that hash");
it does **not** prove **origin** ("this came from that author and that build").
The completion (decided, operator 2026-06-27):

- **Signing — keyless sigstore/cosign + in-toto/SLSA attestation.** The author
  signs `(source, artifact, interface, proof-bundle, trusted_base_delta,
  policy)` hashes; **sigstore** (OIDC identity + public transparency log)
  provides the signing/identity and **in-toto/SLSA** provides the attestation
  *content*. **Keyless is the default** — no long-lived signing keys for humans
  *or agents* to manage and leak, with identity backed by a transparency log;
  the right fit for an agent-driven ecosystem.
- **SLSA build attestation — aim high.** Records *what built the artifact and
  from what* (the Ken compiler version + a hermetic, reproducible build),
  closing the compiler-substitution gap. Ken's reproducible builds (content hash
  from source + compiler) make a high SLSA level natural.
- **Policy attestation.** The provenance carries the **governing policy
  hash/version** the package was built under (`65-policy.md`); a consumer
  verifies the dependency honoured a **monotone-compatible** policy (§3 step 6).
  This is what makes "provably honoured org policy" checkable, not just
  asserted.

**Keep the two ladders distinct.** SLSA/provenance attests the **build pipeline
and origin** (*trusted*); Ken's proofs attest the **program** (*re-checked*,
zero-trust). A high SLSA level does not imply correctness, and a Ken proof does
not imply a trustworthy build — *complementary*, not the same ladder. A strong
posture wants **all** of: provenance (origin) + re-checked proofs (correctness)
+ audited delta (assumptions) + compatible policy (compliance). Ken supplies the
proof ladder natively and re-checks it; provenance is the adopted-standards
origin ladder.

*(Implementation sequences after the core toolchain; this fixes the shape and
the standards.)*

## 5a. The discharge attestation (post-build validation)

Provenance (§5) attests **origin** and Ken's proofs attest **correctness**; a
third artifact attests **post-build validation** — that the behavioral
obligations Ken *delegated* (`../70-behavioral/71`) were actually discharged by
the sibling (`Ward`). In enterprise compliance "the tests ran and passed" is
already a required artifact; Ken/`Ward` replace text logs + coverage XML with a
**signed, runtime-checkable discharge attestation**.
`OQ-discharge-attestation` is **DECIDED** — `Ward` has finalized and merged its
half (ward `f33276b`), and this section **ratifies Ken's half**: the Ken-visible
contract surface, the outcome vocabulary, and the hard trust-boundary
invariants. The three-check **deployment gate** on Ken's provenance verifier is
the sequenced-behind build follow-on (Team Verify, WS-Sec); it delivers the
*runtime* face of the invariants this section pins *statically*.

**The Ken-visible field set (the ratified contract surface).** The attestation
is a `Ward` artifact; Ken emits, enforces, or depends on **exactly** the subset
below — each already produced by the **B1 export** (`../70-behavioral/71 §2.1`),
so ratifying it adds **no new transport and no new trust edge** beyond the
already-pinned `ward.version`:

| Field | Ken source | Role |
|---|---|---|
| `export.hash` | B1 content-hash of `Q`/`P`/`Σ`/`T`/`G` | binds the attestation to exactly the contract asked; also the **revocation mechanism** (a stale hash fails the gate, fail-closed — no revocation list) |
| `export.contractVersion` | the `71` contract version | the export schema version |
| `ward.version` | `Ward` runner version | **the one load-bearing trust edge** (below) |
| `obligations[].id` | obligation identity over `Σ` | stable key **across `export.hash` changes** (the regression key) |
| `obligations[].field` | the export channel — `T` / `P` / `Q@ct` | which channel the obligation came from |
| `obligations[].outcome` | the four-way outcome (below) | the discharge result |
| `signature` | keyless sigstore, incl. `ward.version` | the §5 governance ladder |

Everything else in the attestation is **`Ward`-internal**, and Ken **must not**
depend on it (the boundary below). The **literal field tokens** and the
`predicateType` URI (`ward.dev/attestation/discharge/v1`, `(oracle)`) are
`Ward`'s **wire spelling**, finalized with `Ward` under the `OQ-export-wire`
token-coordination discipline: Ken locks the **concept, value-set, and
cross-field invariants** here; the tokens are oracle-tagged and
reference-spelled in conformance, so the test logic (reject-missing-required,
accept-ignore-unknown, `id`-stable-across-`export.hash`-change) is
spelling-agnostic.

**The four-way outcome — total, and classifies *epistemic status*, not
*mechanism*.** `obligations[].outcome` is a **total** classification:

- **`discharged`** — decided (a decision procedure / exhaustive check settled
  it).
- **`bounded`** — **partial evidence under a stated bound**: model-check depth
  **or** sampled test coverage (L2). The specific bound (depth-`k`, sample size)
  is recorded **`Ward`-internally**; Ken reads only *that* the status is
  `bounded`.
- **`monitored`** — established over observed windows (a runtime monitor, `73`).
- **`failed`** — the obligation was not met.

This **widens** the prior `bounded-to-`k`` to a single **`bounded`** label: the
category must cover sampled coverage as well as depth, and both mean the same
thing — partial evidence under a stated bound. Ken classifies the **epistemic
status**; the **source** (depth-`k` vs. sample) is exactly the `Ward`-internal
mechanism Ken must not couple to, so it stays one label with the bound recorded
`Ward`-side. The four-way is **total** — there is **no fifth outcome**.

**The one-way gate — no `outcome` promotes a `T` to `proved` (hard soundness,
I4).** **No `outcome` value — not even `discharged` — promotes a delegated
obligation to `proved`.** A discharge re-enters Ken **only as an attestation
record**, never as a kernel certificate: the obligation stays
`delegated`/`tested` in the four-way verification status (`../20-verification/21
§5`) and rides in `trusted_base_delta`. This is **invariant I4**
(`../70-behavioral/71 §5.1`/`§2.1`), realized in the B1 emitter as the
**absence of a code path** — there is **no function** from a `Ward` outcome to a
`proved` status — not a runtime check that could be bypassed. A discharge is a
lower-trust, **classically-discharged** artifact: it projects to
**`P`/`tested`**, **never `Q`** (a discharge is not kernel certification). The
**live home** of this net is B1's export gate (`71 §2.1` I4); this section
**reaffirms** it over the discharge outcomes and introduces **no new**
promotion-prevention mechanism (subsume-don't-proliferate).

**The `Ward`-internal boundary — Ken must not depend on it.** The attestation
also carries `Ward`-internal fields — the **`policy`** used (sampling / model /
monitor config), the **`bound`** (depth-`k`, sample size), the **`evidence`**
(seeds, coverage, equivalence classes), the **`ct.method`**, and the
**`regression`** record. These are `Ward`'s mechanism; **no Ken correctness
judgment — no gate, no consumer — may read or branch on any of them.** Ken
depends only on the Ken-visible set above: the binding (`export.hash`), the
trust edge (`ward.version`), the stable keys (`obligations[].id`), and the
epistemic outcomes. A consumer that branches on a `Ward`-internal field is
**rejected** — keeping the abstraction boundary exactly where I4 draws it: Ken
never couples to *how* `Ward` discharged, only *that* it did, under a pinned
version.

**Constant-time validation — carried, not implemented.** A `Q@ct`-channel
obligation (`OQ-relational`: the `@ct` timing guarantee is delegated to `Ward`
under a leakage model, `61 §5a`) carries its verdict as an ordinary
`obligations[].outcome`. The **CT-validation *method*** (`Ward`'s leakage
model / measurement, ward §13) is **`Ward`-internal and open** — Ken neither
implements nor depends on the method, only **carries and enforces the verdict**
through the same field set and the same one-way gate (a `@ct` discharge is
still `delegated`, never `proved`).

**The pinned `Ward` version — the one trust edge.** `ward.version` is
load-bearing, not bureaucratic: the translation-faithfulness proof
(`../70-behavioral/71 §5`) holds **relative to an axiomatized `Ward`
semantics**, and this pin is the one explicit, version-bounded assumption that
`Ward` implements it. Because both `export.hash ↔ build` (provenance) and
`export.hash ↔ discharge` (this attestation) are reproducible, a **stale
discharge** is caught by the gate's hash-match (fail-closed). The attestation is
governed on the same ladder as the policy attestation (`65 §5`): same keyless
signing, same provenance transport, runtime-enforceable. The **deployment gate**
(the build follow-on) then enforces that an artifact carries the validation its
**target environment requires** — e.g. "external endpoints: no `bounded` on a
`Q@ct` obligation."
**That requirement is Ken governance policy (`64`/`65`) — Ken owns it**; `Ward`
specifies the gate's *check*, not what each environment demands.

## 6. The registry (ecosystem governance — above the language)

A package registry enforces what the language cannot: **namespace ownership**
(only `alice` publishes `alice/*`), **mandatory provenance** before listing, and
**automated `trusted_base_delta` display** so a consumer sees a package's
assumptions at a glance. This is social/governance layer (`64 §4`), not a
language feature, but the language makes it *meaningful* by giving the registry
real attestations (deltas, proof bundles) to police.

## 7. What WS-L / tooling must deliver here

The package artifact format (§2) incl. the `.keni` interface; the consume flow
with **kernel re-check** (§3); `trusted_base_delta` emission + a consumer policy
engine; and the provenance integration (signing + SLSA, `OQ-provenance`).
Acceptance: a tampered proof bundle is **rejected** on consume; a package's
assumptions are visible in its delta before use; a content-hash mismatch fails
closed. Conformance: `../../conformance/security/supply-chain/`.
