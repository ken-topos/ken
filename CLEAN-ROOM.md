# Clean-Room Policy

Ken is an **MIT-licensed, clean-room reimplementation**. It was *inspired* by
Yon, an AGPLv3 research prototype — but Yon is an **excluded inspiration**: it is
**not mounted in any worktree** and is **not consulted in any environment**, and
its core design (cubical paths / cofibrations) was deliberately not adopted (Ken
chose observational type theory, ADR 0005). Ken's design is its own; the
permissive license rests on the **absence of AGPLv3 source contact** — strictly
cleaner than reading-then-not-copying. This policy is load-bearing.

## What is reusable vs. not

**Reusable** — ideas, language design and semantics, the topos/HoTT approach,
content-addressing as identity, and all mathematics (lattices, codes, groups,
hashing). Ideas, methods, and mathematics are not copyrightable, and interfaces
are defensible to reimplement.

**Not reusable** — copying or close paraphrase of the prototype's source code
into this repository.

**Dependencies** — permissive or reimplemented only. (`mmgroup` is BSD-2,
reusable with attribution; LLVM/Cranelift are permissive.)

## The process

1. **The Spec enclave authors from permissive references + first principles.**
   It produces a written specification (`/spec`) and a black-box **conformance**
   corpus (`/conformance`) from the permissive shelf (Lean, Agda, cooltt, …),
   settled decisions, and first principles — **not** from Yon (which is absent).
   Those artifacts describe *behavior* in Ken's own words, with no copied source.
2. **Implementation teams work from the spec**, with prototype source kept out
   of their working context. Implementation PRs cite spec sources, not prototype
   `file:line`.
3. **The merge gate enforces it.** Federation review and the scripted publisher
   path confirm each PR cites spec sources and introduces no AGPL-derived code;
   CI runs a provenance check.

## Local reference implementations (`local/refs/`)

The operator keeps reference implementations under `local/refs/` (gitignored —
never in this repo). They fall in two tiers:

- **The AGPLv3 prototype (Yon)** is **not mounted** in any environment and is
  **not a reference** — it is the excluded inspiration (see the top of this
  file). Zero AGPLv3 contact is the posture; **no agent should seek it out.** If
  it ever becomes available in a future environment, **stop and ask the
  operator** before consulting it.
- **Permissive references** (Lean, Agda, smalltt/elaboration-zoo, Z3, cvc5, F\*,
  Interaction Trees, Koka, LIO, sigstore/in-toto, Quint/Apalache, QuickChick,
  Unison, …) — Apache-2.0 / BSD / MIT-style. The Architect / Spec enclave, **the
  research agent, and the adversary agent** may **read them to understand** and to
  resolve `(oracle)`-tagged spec details (the adversary reads for *known prior-art
  failure modes*); Ken's code is then written **from the spec**,
  in Ken's own words. Do **not** vendor their source into the repo (keeps Ken
  uniformly MIT). Implementer agents build from `/spec`, never from
  `local/refs/`. The full curated shelf and its spec-section mapping live in
  `local/refs/README.md`.
- **Copyleft references** (GPL / AGPL / CeCILL — e.g. SMTCoq, Spot, Jif) —
  permitted for the Spec enclave, **the research agent, and the adversary agent**
  to **read for *approach and behavior* only** (the adversary for known failure
  modes), under a stricter discipline: describe the *what* (the algorithm's
  behavior, the design idea) in Ken's own words, never the *how* (the source's
  structure, identifiers, comments, ordering). **Never vendored, never read by
  implementers.** Because a copyleft transcription into the spec would taint the
  whole MIT channel, these are subject to the **leakage recheck** below — which
  binds the research and adversary agents exactly as it binds the enclave.
- **★ FIRST-PARTY SIBLINGS — `ward/` and `keep/` — are NOT references at all,
  and are freely readable by every role.** `local/refs/ward` (and its sibling
  `keep`) are **Ken's own projects**: same operator, same author, **MIT**, part of
  the same program (`ken-topos/ward`, `ken-topos/keep`; Ken's `G-Ward-seam` gate
  is *their* seam). They live under `local/refs/` **only because that is where
  sibling checkouts are mounted** — a **directory accident, not a policy
  classification.**

  > **Read them. You must, to build the seam.** Ken's `spec/70-behavioral/`
  > specifies *Ken's half* of an interface whose *other half* is written down in
  > `ward/spec/10-seam/`. An agent building the export emitter, a resource
  > obligation, or a trace contract **cannot do the job from Ken's side alone.**
  >
  > **Why this line exists (added 2026-07-14):** the standing rule is *"when
  > unsure whether you may look at something under `local/refs/`, the answer is
  > no."* That rule is right, and it was **about to mis-fire** — an agent asked to
  > build the Ken↔Ward seam would have correctly refused to read Ward, and been
  > unable to do its work. **A quarantine that silently swallows a first-party
  > sibling is a quarantine with a bug.** There is no license question here: it is
  > our own MIT code.

## The copyleft-leakage recheck (the originality gate)

The spec is the channel from copyleft-readable references to MIT implementation,
so the spec must stay **provably original** with respect to any copyleft source
it consulted. Two layers:

1. **Front-line discipline** (always). A spec section informed by a copyleft ref
   describes *behavior and design intent* in Ken's own words and structure, and
   reproduces none of the source's *expression* — identifiers, comment text,
   code structure, or statement ordering.
2. **The recheck gate** (before a spec area is handed to the build teams, and
   whenever a section is refined against a flagged ref). An **independent
   originality review**: a reviewer or agent confirms, for each spec section
   that consulted a copyleft ref, that it is original expression (the *what*,
   not the *how*). A textual/structural **similarity scan**
   (`scripts/originality-scan.py spec local/refs/<ref> --fail 0.04`, k-gram
   shingles) between the spec prose and the flagged source is run as a *flagging
   aid*; long matched **runs** are the suspicious signal (short matches over
   shared domain vocabulary are expected), and any flagged span is escalated to
   a human. The reviewer is never the author of the section. Run it only against
   the **copyleft** refs (the ones that taint the MIT channel) — not against
   Ken's own docs, which legitimately share phrasing.

**Provenance.** A section that consulted a copyleft ref records it (an internal
note, not in the normative text) so the recheck knows where to look. The current
spec was authored from first principles + permissive material *before* this
copyleft shelf existed, so the recheck's live scope is the **refinement phase**
— as the enclave uses these references to sharpen the spec and resolve
`(oracle)` points. Run it as a gate on that phase, not retroactively on text it
never touched.

## If in doubt

Stop and ask Team Spec. Never paste prototype source into an implementation
crate. A clean-room boundary is cheap to keep and expensive to repair.
