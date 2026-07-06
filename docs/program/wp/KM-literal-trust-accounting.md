# KM-literal-trust-accounting

**Owner:** Language build team, with Architect review. **Branch:**
`wp/KM-literal-trust-accounting`. **Size:** M. **Risk:** high: this touches the
trusted-base accounting boundary. **Blocks:** CAT-5 D3 Boolean grammar.

## 1. Objective

Make checked surface literals accounting-neutral in `trusted_base()` without
hiding real trusted dependencies.

CAT-5 D3 requires ordinary package code to write concrete ASCII tokens such as
`"true"`, `"and"`, `"("`, `")"`, and numeric offsets. Today those literals are
registered as primitive `literal` globals, so package code that merely writes a
literal causes trusted-base growth. That makes CAT-5's strict zero-delta gate
and the D3 grammar contract unsatisfiable together.

## 2. Fixed Ruling

Architect ruled route 2 at `evt_5pbe709raae57`:

- keep CAT-5 G1 zero-delta absolute;
- do not add a CAT-5-local literal allowlist;
- fork a prerequisite mechanism WP for trusted-base-neutral literal accounting;
- resume CAT-5 D3 unchanged after this lands.

## 3. Scope

The mechanism must cover numeric and string literals at minimum. It should also
leave a clear path for byte-oriented package constants needed by parsing and
printing.

In scope:

- literal declaration/accounting status;
- `GlobalEnv::trusted_base()` and per-definition `trusted_base_delta` behavior;
- tests for transparent package definitions containing literals;
- tests proving real primitives, foreigns, opaque holes, and open obligations
  remain accounted.

Out of scope:

- changing CAT-5 D3's grammar contract;
- waiving the package zero-delta gate;
- hiding all `Decl::Primitive` entries globally;
- changing kernel soundness or primitive reductions;
- implementing CAT-5 D3 itself.

## 4. Acceptance Criteria

1. A package snippet with checked numeric and string literals can elaborate
   without new trusted-base entries attributable to those literals.
2. A transparent definition that uses literals has no per-definition
   `trusted_base_delta` beyond the preexisting primitive operations it truly
   relies on.
3. A snippet that declares or uses a real new primitive still produces the
   expected trusted-base/delta entry.
4. Foreign bindings, opaque postulates, open obligations, and real primitive
   operations remain visible in trust accounting.
5. The CAT-5 D3 blocker is represented by a regression test that fails on the
   current behavior and passes after the mechanism.
6. No `crates/ken-kernel` TCB expansion unless Architect explicitly approves a
   narrower kernel-surface change.

## 5. Implementation Guidance

The exact mechanism belongs to the build team. Acceptable directions include a
dedicated literal declaration/classification or an equivalent audited exclusion
keyed to literal metadata. The invariant is what matters: a literal constant
that is part of checked surface syntax is not a trusted primitive assumption,
but genuine trusted operations remain accounted.

## 6. Review Path

Language implementer builds the mechanism and focused tests. Language QA
re-derives the trust-accounting discriminator in both directions:
literal-neutral and real-primitive-still-counted. Architect reviews the
trusted-base boundary before Integrator merge. After merge and retros, Steward
releases CAT-5 D3 unchanged from its existing package-owned grammar contract.
