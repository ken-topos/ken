# Supply-chain security

> Status: **DRAFT v0**. Normative for the consumption model and the artifact
> shape; signing/registry mechanics are flagged **OQ-provenance**. How a Ken
> package is consumed *safely* — the area where Ken's de Bruijn criterion pays
> an unusual dividend. ADR 0004 Decision 5.

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
  , provenance     -- (OQ-provenance) signature + SLSA attestation
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
     (empty? acceptable postulates? acceptable declassifications/FFI?).
7. Only on all-pass: extend your Σ with the verified fragment.
8. Your module's new obligations discharge against the extended Σ.
```

Step 5 is where trust is established **by your kernel**, not the author. Steps
3–4 add *identity* and *origin*; step 5 adds *verified correctness*; step 6
makes the residual *assumptions* a policy decision.

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

## 5. Provenance — the complementary axis (OQ-provenance)

Content-addressing proves **identity** ("this is the artifact with that hash");
it does **not** prove **origin** ("this came from that author and that build").
The completion:

- **Signing.** The author signs `(source hash, artifact hash, interface hash,
  proof-bundle hash, trusted_base_delta)` — keyless via sigstore/cosign (OIDC
  identity + transparency log) or in-toto link metadata. Consumers verify
  against a known identity / key-transparency log.
- **SLSA build attestation.** Records *what built the artifact and from what* —
  the Ken compiler version and a hermetic build environment — closing the
  compiler-substitution gap.

**Keep the two axes distinct.** SLSA attests the **build pipeline**; Ken's
proofs attest the **program**. They are *complementary*, not the same ladder: a
high SLSA level does not imply correctness, and a Ken proof does not imply a
trustworthy build. A strong posture wants **both** — provenance (origin) *and*
re-checked proofs (correctness) *and* an audited delta (assumptions). Ken is
unusual in supplying the middle one natively; `OQ-provenance` tracks adding the
first.

## 6. The registry (ecosystem governance — above the language)

A package registry enforces what the language cannot: **namespace ownership**
(only `alice` publishes `alice/*`), **mandatory provenance** before listing, and
**automated `trusted_base_delta` display** so a consumer sees a package's
assumptions at a glance. This is social/governance layer (`64 §limits`), not a
language feature, but the language makes it *meaningful* by giving the registry
real attestations (deltas, proof bundles) to police.

## 7. What WS-L / tooling must deliver here

The package artifact format (§2) incl. the `.keni` interface; the consume flow
with **kernel re-check** (§3); `trusted_base_delta` emission + a consumer policy
engine; and the provenance integration (signing + SLSA, `OQ-provenance`).
Acceptance: a tampered proof bundle is **rejected** on consume; a package's
assumptions are visible in its delta before use; a content-hash mismatch fails
closed. Conformance: `../../conformance/security/supply-chain/`.
