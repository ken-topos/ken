# SURF-target-index-kernel-authority

**Owner:** Language build team.
**Reviewer:** Language QA and Architect. Kernel QA only if the D0 result
touches kernel contracts.
**Branch:** `wp/SURF-target-index-kernel-authority`.
**Status:** Steward frame, queued after `KM-target-index-positivity`.
**Size:** S. **Risk:** medium, because this sits at the surface/kernel
authority boundary for inductive admission.

## 0. Trigger

`KM-target-index-positivity` landed the conservative kernel authority for
constructor result target indices:

- kernel strict positivity now scans every constructor `target_indices` term;
- same-family occurrences in target indices reject before admission succeeds;
- no broader recursive target-index positivity rule was introduced;
- no elaborator, surface, spec, or conformance movement was needed.

The prior `SURF-gadt-elaboration` fix added a surface-side same-family guard
because the kernel did not yet inspect this stored payload. That was the right
bounded fix at the time. Now the kernel is authoritative, so Language should
decide whether the duplicated surface guard should remain as an early
diagnostic or be removed so producers prove the trusted gate is kernel
admission.

This WP is not a broad expressivity feature. The kernel result is conservative:
recursive same-family target indices are still rejected.

## 1. Objective

Audit and, if appropriate, refine the surface handling of constructor result
target indices after the kernel authority landed.

The desired end state is one of:

1. **No-op with explicit rationale:** keep the existing surface guard because it
   gives a better diagnostic or simpler staging, and document that the kernel
   remains the trusted authority.
2. **Bounded removal of duplicated authority:** remove only the redundant
   surface same-family target-index rejection so the same source shape reaches
   kernel admission and rejects there, with tests proving the kernel-side gate
   is exercised.

Do not admit any recursive target-index class that the kernel now rejects.

## 2. Required D0 Audit

Before implementation, post a D0 audit answering:

1. Where does `SURF-gadt-elaboration` currently reject same-family occurrences
   inside constructor result target indices?
2. What user-facing diagnostic changes if that guard is removed and the kernel
   rejection becomes the first authority?
3. Does removing the guard require any parser, resolver, generated eliminator,
   or conformance change?
4. Can a focused regression prove the surface path reaches kernel admission
   for a rejected same-family target index without weakening the final result?
5. Is the correct outcome a no-op? If yes, state why and route back to
   Steward/Architect without making code changes.

If D0 proposes admitting any target-index form beyond the conservative kernel
D-free rule, stop and route a new spec/kernel design fork. Do not build it in
this WP.

## 3. Deliverables

Depending on D0:

- For a no-op result:
  - post the D0 result with a clear reason why no code change is warranted;
  - state the next program step back to Steward.
- For a bounded implementation:
  - remove only the duplicated surface same-family target-index guard;
  - preserve the final rejection for same-family target indices through kernel
    admission;
  - add focused tests showing the rejected surface shape now reaches the kernel
    authority path;
  - preserve positive indexed-family examples such as Vec-style target indices.

## 4. Acceptance Criteria

AC1. The trusted authority is clear.

- The final handoff states whether the surface guard remains intentionally, or
  whether rejection now comes from kernel admission.

AC2. No broad expressivity change.

- Same-family target indices remain rejected.
- No broader recursive target-index class is admitted.

AC3. Existing GADT surface behavior stays intact.

- Existing explicit-data positives remain accepted.
- Constructor expressions, constructor patterns, and explicit-signature record
  labels remain unrelated and out of scope.

AC4. Scope remains bounded.

- Expected touched area, if any, is `crates/ken-elaborator/` tests and
  implementation.
- No `crates/ken-kernel`, `Cargo.lock`, `packages`, `spec`, or `conformance`
  movement unless D0 routes and receives approval for a scope fork.

## 5. Review Routing

Language implements only after D0 chooses the bounded implementation route.
Language QA reviews all D0/no-op or D1 outcomes. Architect review is mandatory
because this WP is about surface/kernel authority placement.

Kernel QA is required only if D0 routes a kernel-contract change. CV is
required only if conformance changes.

## 6. Do Not Reopen

- Do not change kernel positivity.
- Do not implement a broader recursive target-index positivity rule.
- Do not relax the landed kernel rejection for same-family target indices.
- Do not implement indexed coverage diagnostics, impossible-arm omission, or
  dependent-match expansion.
- Do not add new dependent-constructor syntax.
- Do not use `local/refs/`.
