# Clean-Room Policy

Ken is an **MIT-licensed, clean-room reimplementation**. It draws on the *design*
of an AGPLv3 prototype ("Yon", a sibling checkout at `../yon`) but must not be a
derivative of its **source code**. This policy is load-bearing: the permissive
license depends on it. (This is process guidance, not legal advice; confirm with
IP counsel before Phase 1.)

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

1. **Team Spec is the only conduit.** Team Spec may read the prototype to produce
   a written specification (`/spec`) and a black-box **conformance** corpus
   (`/conformance`). Those artifacts describe *behavior*, and contain no copied
   source.
2. **Implementation teams work from the spec**, with prototype source kept out of
   their working context. Implementation PRs cite spec sources, not prototype
   `file:line`.
3. **The merge gate enforces it.** The Integrator confirms each PR cites spec
   sources and introduces no AGPL-derived code; CI runs a provenance check.
4. **The planning docs `01-reality-check.md` … contain short prototype `file:line`
   citations as analysis/commentary.** They are knowledge artifacts, not
   implementation input, and should be excluded from any distributed release
   artifact.

## If in doubt

Stop and ask Team Spec. Never paste prototype source into an implementation
crate. A clean-room boundary is cheap to keep and expensive to repair.
