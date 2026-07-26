---
id: SPEC-ALIGN-B1
title: "Split the frozen interoperability and provenance schemas into versioned protocol profiles, under a per-edge threat audit rather than a field count"
status: draft
owner: spec
size: L
gate: none
depends_on: [SPEC-ALIGN-A1]
blocks: []
github: null
origin: Operator dispatch 2026-07-26 on the conformance-validator's overspecification advisory with research's prior-art addendum — "Category 2 are things that we should address, but I'd like to get to them after the full linux ABI campaign finishes. Capture those in a work program, or issue, or other doc so we don't lose them." Advisory captured verbatim at docs/program/spec-mission-overspecification-advisory.md; dispositions at docs/program/14-spec-mission-alignment-campaign.md §5. Steward-filed per COORDINATION §2. NOT RELEASED — see the sequencing gate below.
---

> ## ⛔ THIS NODE IS DEFERRED BY OPERATOR DECISION — DO NOT PICK IT UP
>
> **Operator, 2026-07-26:** *"things that we should address, but I'd like to get
> to them after the full linux ABI campaign finishes."*
>
> ⛔ This is **not** a hold pending a question, and **not** a blocked node
> awaiting a ruling. It is **sequenced work with a named predecessor.** Nothing
> is owed to anyone on it. ⛔ Do not re-ask the operator whether it can start —
> the answer is already recorded here, and the gate below is mechanically
> checkable.
>
> ### The gate, measurably
>
> The `ABI-*` node set at `origin/main=9410d7b8` is **14 nodes**: `ABI-R1`
> **closed**, and `ABI-A1 A2 A3`, `ABI-M1 M2`, `ABI-R3`, `ABI-REVOKE`,
> `ABI-S1 S2 S3 S4 S5 S6` all **`draft`** — 13 open.
>
> ⚠ **Re-derive that set at release time; do not trust this line.** A campaign
> re-slice changes the set, and a stale enumeration would let this node release
> early against a set that no longer exists. The campaign docs are
> `09-posix-linux-abi-campaign.md` and `10-linux-abi-completion.md`.

## Why this node exists as a capture

The whole analysis behind it was delivered to
`local/spec-mission-overspecification-review.md`, and **`local/` is
gitignored.** A deferred campaign whose only source can be removed by a
`git clean` is not captured. The advisory is now tracked; this node holds the
**method**, because the method is the part that would otherwise be
reconstructed from memory months later.

## Scope — the frozen-schema families

| family | spec anchors |
|---|---|
| Ward export + trace schemas | `spec/70-behavioral/71-assumption-boundary.md §2`–`§3`; `73-conformance.md §2` |
| checked package + executable envelopes | `spec/40-runtime/46-checked-core-package.md`; `48-executable-artifact-contract.md` |
| named supply-chain products | `spec/60-security/63-supply-chain.md §5` |
| FFI + buffer protocol as an ABI profile | `spec/30-surface/38-ffi-io.md` — ⚠ intersects the live Linux ABI campaign, which is *why* this node's gate is the right clock |
| durable canonical value encoding | `spec/40-runtime/41-values.md §3a` — the durable half of the content-store item; the in-process half is `SPEC-ALIGN-A1` / fork C7 |

## ⭐ The three method commitments — the part that must not be lost

### 1. The per-edge threat audit

⛔ **"Duplicate hash" is not a finding.** Mature artifact formats repeat typed
digests at layer boundaries **on purpose**: an in-toto statement binds a
digest-identified subject to a separately typed predicate; OCI manifests bind
typed config and layer descriptors by digest; TUF binds versions, lengths, and
hashes specifically to defeat rollback and mix-and-match.

⇒ For **each** field and duplicated binding, ask:

> Which producer signs this field, which consumer checks it, and which concrete
> substitution, rollback, type-confusion, or stale-evidence attack succeeds if
> it is removed?

**Merge two bindings only when they share authority, signed scope, consumer,
lifetime, AND attack set.** A smaller compositional evidence graph is a good
goal; field count is weak evidence of overreach.

### 2. The version + algorithm agility audit

Several chapters fuse a **permanent semantic identity** with **one** hash,
signature product, or schema generation. OCI, in-toto, SLSA, TUF, and Sigstore
all carry explicit versions or typed envelopes. Audit every durable hash or
signature for:

- algorithm identifier and domain separation;
- migration without identity ambiguity;
- downgrade prevention;
- canonical bytes within a version;
- whether old artifacts remain independently checkable.

⭐ **The discriminator:** in-process FNV-1a needs no durable agility once it is
private. **Any hash crossing a process, package, provenance, or archival
boundary does.**

### 3. The three-way protocol-evolution rule

The spec currently prefers closed schemas with loud rejection of unknown
fields. That forces a **new major version for every additive diagnostic field**.
Prior art distinguishes three cases:

| unknown thing | disposition |
|---|---|
| **semantic** field affecting meaning or authority | ⛔ **reject** |
| **optional metadata** under a known major version | preserve or ignore per profile |
| **major version / type URI** | ⛔ **reject** |

⇒ Retains honesty (an unknown obligation still fails closed) while allowing
additive evolution.

## What the versioned profiles must carry

- an explicit **major version or type URI**;
- **canonical bytes within a version**;
- **monotone rules** for ignorable extensions;
- **fail-closed** handling when a consumer does not understand a semantic field.

⭐ **Keep normative and separate from the wire layout:** the one-way
no-promotion invariant, exact status meanings, accurate status projection, trace
fidelity, and reproducible contract identity. ⚠ The five `Q/P/Sigma/T/G`
*concepts* may well be the right semantic decomposition — the relaxable thing is
their **one current wire layout**, and ITF itself has revised its encoding twice
(2023 integer encoding, 2025 naming) while deliberately leaving `#meta` open.
That is evidence against treating one ITF byte shape as permanent Ken source
semantics; it is not evidence against the concepts.

Similarly for supply chain: SLSA 1.2 calls its own concrete provenance and
verification-summary formats **recommended, not required**. The mission-aligned
layer is authenticated provenance, subject binding, builder/source identity,
policy evaluation, and recheck-on-consume. Sigstore, Cosign, in-toto, SLSA, TUF,
or a commercial attestation service are **versioned deployment profiles beneath
that contract.** Ken may ship a preferred profile without making named products
permanent language semantics.

## ⛔ DO NOT RELAX

Recheck-on-consume; stable semantic identity; provenance; explicit unavailable
evidence; the one-way no-promotion invariant; and the prohibition on promoting
Ward, test, or monitor results to `proved`. **These are strongly mission-aligned
and are not what this node is about.**

## Reporting contract

Every landed relaxation records: the mission outcome still protected; the
observable or security invariant retained; the implementation choices newly
permitted; any external consumer requiring exact compatibility; and **a
conformance pair showing the relaxed contract still rejects an actual
mission-breaking implementation.**

⛔ **This node has no frame yet.** The frame is written at release time, against
the then-current `spec/` — ⚠ authoring it now would produce a frame anchored to
a tree that the entire ABI campaign will have moved, and a stale frame reads as
a current one.
