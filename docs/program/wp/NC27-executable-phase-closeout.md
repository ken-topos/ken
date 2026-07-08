# NC27 - Executable Phase Closeout

**Owner:** Runtime/Verify-led, with Architect review. **Branch:**
`wp/NC27-executable-phase-closeout`. **Size:** M. **Risk:** medium-high.

## Objective

Close the NC19-NC27 native executable phase with a starter corpus gate and a
clear report of supported, unavailable, and deferred native-executable claims.

## Scope

In scope:

- final starter executable corpus selection;
- phase report summarizing native executable support and gaps;
- confirmation that library generation remains deferred to NC37-NC45;
- recommendation for whether to enter broad validation NC28-NC36 or first
  frame targeted prerequisites.

Out of scope:

- new lowering or backend behavior;
- library artifacts;
- self-hosting;
- whole-compiler proof.

## Deliverables

- NC19-NC27 closeout report.
- Corpus manifest tying each starter executable to checked-core, runtime-IR,
  native artifact, and trust-report identities.
- Go/no-go recommendation for NC28.
- Backlog of prerequisite WPs for any native-executable blockers found.

## Acceptance

- The starter native executable corpus builds, runs, and reports through the
  full NC19-NC26 chain.
- Every gap is classified as unsupported, unavailable, failed, tested,
  validated, or proved.
- The report does not claim library ABI, interop, cross-package native linking,
  or source-to-binary proof.
- Architect review confirms that the next phase can start or that a specific
  prerequisite must be routed first.

## Guardrails

- Do not close the phase on green smoke tests alone.
- Do not hide unsupported constructs by removing them from the corpus without
  naming the exclusion.
- Do not start NC28 until the report names the actual evidence boundary.
