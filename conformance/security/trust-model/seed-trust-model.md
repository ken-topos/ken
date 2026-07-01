# Trust model & TCB conformance — seed cases (Sec4)

Format: `../../README.md`. These pin the **trust model** of
`spec/60-security/64-trust-model.md` (Sec4, normative): the **enumerable TCB**
(the trusted base is exactly the kernel + primitive reductions +
postulates/`foreign`, and nothing else — mechanically listable on demand),
**authorship-independence** (the de-Bruijn criterion read as a security
property — admission is on the kernel's terms, never the author's say-so), the
**trusting-trust invariant** (the small Rust kernel stays an independent second
checker, never depending on a Ken-generated artifact), and the **honest limits**
(a verified language that over-claims is itself a security risk). This is the
**machine-checkable half** of the G5 kernel-audit posture (`64 §3`, `§6`); the
external published audit *report* is out of scope (a governance / T4 follow-on).
They sit beside the Sec1 IFC (`../ifc/seed-ifc.md`), Sec1ct CT (`../ct/`), and
Sec2 authority (`../capabilities/`) seeds, and subsume the `trust/`
`independent-recheck` placeholder in `../seed-security.md`.

Grounding (landed `§`-bodies + landed code on this branch, content-reconciled —
not the plan): the four normative contracts — `64 §1.1` **TB-Sound**
`[landed producer]`, `64 §1.2` **TB-Complete** `[by construction]`, `64 §2.1`
**AI-Indep** `[structural]`, `64 §3.1` **Invariant TT** `[structural]` — plus
`64 §4` (honest limits, normative) / `§6`, `64 §5` (the four-point consumer
check; empty delta ⟺ fully verified+confined); `18 §5` (the K-api / TCB
definition), `23 §1` (never a false `proved`), `25 §3` (`trusted_base_delta`).
**Landed code pinned against:** `GlobalEnv::trusted_base()`
(`crates/ken-kernel/src/env.rs:383`) — filters `decls` to
`Decl::Opaque | Decl::Primitive`, excluding only the prelude via `is_prelude`
(`env.rs:256`, matching **only** `top_id`/`bottom_id`); `declare_postulate`
(`check.rs:1055`) → `Decl::Opaque`, the single assumption-introducing
choke-point that `foreign`
signatures (`crates/ken-elaborator/src/foreign.rs:171`), typed-hole / `ensures`
runtime-check slots (`foreign.rs:185`), and unproved prover goals
(`prover.rs:367`, `23 §1.3`) all route through; `upgrade_to_transparent`
(`env.rs:349`, `Decl::Opaque` → `Decl::Transparent` on discharge); the kernel
`check(env, ctx, t, ty)` entry (`check.rs:373`) — four parameters, no
provenance; `crates/ken-kernel/Cargo.toml` (the kernel's dependency closure).

## Reading these cases — the Sec4-specific disciplines

**The TCB is a closed enumeration, and `trusted_base()` IS the enumerator
(`64 §1`).** Soundness rests on exactly three things — the kernel, the primitive
reductions (`14 §5`), the postulates/`foreign` (each in `trusted_base_delta`,
`25 §3`). Nothing else is trusted (not the elaborator, prover, SMT, surface
compiler, runtime, IFC discipline, or package tooling — they emit artifacts the
kernel re-checks). The landed `trusted_base()` (`env.rs:383`) realizes this: it
returns exactly the `Decl::Opaque` (postulate/hole/`foreign`) +
`Decl::Primitive` decls, minus the fixed prelude vocabulary. A checked
`Decl::Transparent` definition (and inductives) is **re-checked, not trusted**,
and never appears (`64 §1`). So "empty delta ⟺ fully verified+confined"
(`64 §5`) has two independent faces this seed pins as a **non-degenerate pair**:
**SOUND** (no phantom — a verified term's delta is empty; a `Transparent` def is
never listed; AC1/§A, `64 §1.1`) and **COMPLETE** (no hidden — every assumption
surfaces; AC2/§B, `64 §1.2`, the ★ security-critical net). One face without the
other is a green-vs-green half-guard.

**The completeness net is a by-construction property of the admission surface,
not a runtime check (`64 §1.2`, AC2).** There is no `trusted_base()` bug the
kernel "catches" — the enumeration is a plain filter over `Σ`. What makes it
complete is that **every** assumption-introducing path (`foreign` sigs
`foreign.rs:171`, typed holes / `ensures` slots `foreign.rs:185`, unproved
prover goals `prover.rs:367`) routes through the **single
`declare_postulate`→`Decl::Opaque` choke-point**, and the only exclusion is
`is_prelude` = {`Top`, `Bottom`}. So the net for the "nothing else is trusted /
mechanically listable" claim is **exhaustiveness of the choke-point + narrowness
of the exclusion** — an absent-clause / no-silent-catch-all assertion on the
producer's *shape* (§B), never a hand-fed `Vec<GlobalId>`.

**Trust faces — get the LEVEL right, do not over-claim "kernel-backed."** The
chapter itself states that nothing in it is a claim the kernel *proves*: the
kernel is the trust root, so its own properties are *trusted-as-code*, held by
audit (`64 §3`). The five ACs sit at four distinct trust levels, tagged per case
to match the contract stamps:
- **`[landed producer]`** — AC1 (§A, `64 §1.1`): the consumer *reads*
  `trusted_base()`'s output (empty ⟺ verified). This is the one AC bottoming out
  in a live producer output a consumer reads.
- **`[by construction]`** — AC2 (§B, `64 §1.2`): the completeness *guarantee*
  (every assumption routes through the `declare_postulate`→`Opaque` choke-point,
  which coincides with the `trusted_base()` filter) is **trusted-as-code, not
  kernel-proved**. The cases still drive the **real** producer + a **real**
  `foreign`/hole admission — but the guarantee they net is by-construction, not
  a producer output.
- **`[structural]`** — AC3 (§C, `64 §2.1`): the accept/reject is the landed
  kernel `check` verdict (kernel-checked), but **authorship-independence
  itself** is a structural property of the check *signature* — no provenance
  channel exists to exploit, because there is no provenance input. **Not** a
  kernel-proved theorem.
- **`[structural / architectural]`** — AC4 (§D, `64 §3.1`): the trusting-trust
  invariant is a **mechanical check on `ken-kernel`'s Cargo dependency
  closure** — **not** kernel-proved and not a runtime property.
- **doc-posture fidelity** — AC5 (§E, `64 §4`): a rendered-faithfully check that
  the honest limits are stated normatively and none is silently softened; **no**
  producer.

This mirrors Sec2's "kernel-backed vs trusted-by-typing" honesty (`62 §H`) and
the B1 "trusted-by-typing is not kernel-proved" line — a security corpus that
mislabels a trusted-by-construction property as kernel-backed over-claims
exactly where a CISO reads the guarantee.

**Authorship-independence is a verdict-flip on proposition validity, keyed on
the kernel verdict — never on metadata (`64 §2.1`, AC3).** The de-Bruijn
security reading: a bug or malice in *any* generator (prover, elaborator, SMT,
AI) can cause a failed proof or a rejected certificate — **never a false
`proved`** (`23 §1`). The discriminating pair is the **same certificate shape**
(a `refl`) at a **true vs false** proposition: `refl` at `Id Nat 0 0` accepts,
`refl` at `Id Nat 0 1` rejects on conversion (`0 ≢ 1`). The net is that the
discriminant is the kernel `check` verdict on `(t, ty)` alone; "trusted author"
is inexpressible at the API (§C-C3), so it cannot bypass the check.

## A. TCB enumeration is SOUND (AC1) — empty ⟺ verified, no phantom

> The non-degenerate pair is **{A1, A2}** on the **same** `trusted_base()`
> producer: a no-assumption term → empty, a one-postulate term → lists exactly
> it. **A3** adds the phantom guard (a checked `Transparent` def is not an
> assumption). All three flip on real `Decl::Opaque`/`Decl::Transparent`
> presence — never a hand-built delta.

### security/trust-model/verified-term-has-empty-trusted-base
- spec: `64 §1.1` (Contract TB-Sound), `64 §5`, `18 §5`
- given: a `GlobalEnv` built with only checked declarations — a `declare_def`
  (`Decl::Transparent`) and/or inductives — and **no** postulate, `foreign`,
  hole, or registered primitive
- expect: `trusted_base()` returns **`[]`** (the empty delta)
- why: (soundness) AC1. Empty delta = the "fully verified+confined" reading
  (`64 §5`). Producer: the landed `trusted_base()` (`env.rs:383`) — the empty
  result is computed from `Σ` containing no `Opaque`/`Primitive`, not asserted.
  **Flip:** case A2 (one postulate) makes the *same* producer return a non-empty
  delta. **Trust level: `[landed producer]`.**

### security/trust-model/single-postulate-lists-exactly-itself
- spec: `64 §1.1` (TB-Sound), `25 §3` (`trusted_base_delta`)
- given: the same env plus **one** `declare_postulate c : A` (`check.rs:1055` →
  `Decl::Opaque`), `c`'s `GlobalId` fresh (≠ prelude)
- expect: `trusted_base()` returns **`[c]`** — exactly that one `GlobalId`,
  nothing more, nothing less
- why: (soundness) AC1. The delta is **exact**, not merely "non-empty" — it
  names precisely the assumption inherited. **Flip:** removing the postulate
  (case A1) empties it; the verdict flips on real `Decl::Opaque` presence, on
  the landed producer, not a synthetic count.
  **Trust level: `[landed producer]`.**

### security/trust-model/checked-definition-absent-from-trusted-base
- spec: `64 §1` (defs/inductives re-checked, excluded from the base), `18 §5`
- given: an env with a `declare_def d := body` admitted as `Decl::Transparent`
  (kernel-checked, `18 §4`) — a *proved* definition, not an assumption
- expect: `d` is **absent** from `trusted_base()` (the delta does not list it)
- why: (soundness) AC1, the **phantom-assumption guard**. A checked term is not
  a trust root; listing it would over-report the TCB and break "verified ⟹
  empty." Pins the `matches!(Opaque | Primitive)` filter (`env.rs:383`) excludes
  `Transparent`. **Absence is guard-gated, not coincidental:** `d` is absent
  **because** it is `Transparent`, not because the env is empty (a `declare_def`
  is present) — a producer that wrongly listed `Transparent` decls fails this
  case while A1 still passes. **Trust level: `[landed producer]`.**

## B. TCB enumeration is COMPLETE (AC2 ★) — no hidden assumption

> The ★ security-critical net: the entire "nothing else is trusted /
> mechanically listable" claim fails **silently** if an assumption can hide. The
> pair is **{B1, B2}** on the same `GlobalId` across the `Opaque`→`Transparent`
> transition (present-as-assumption → absent-after-discharge); **B3** pins the
> exclusion filter's narrowness (the no-silent-catch-all assertion); **B4**
> exercises the filter's **second** arm (a registered `Primitive`, TCB item 2).
> Every case drives a **real** admission through the choke-point — never a
> hand-inserted decl.

### security/trust-model/foreign-signature-surfaces-in-delta
- spec: `64 §1.2` (Contract TB-Complete), `64 §4.3`, `38 §3`
- given: a real `foreign` C signature elaborated — admitted through the
  `foreign` → `declare_postulate` → `Decl::Opaque` choke-point
  (`foreign.rs:171`, "the foreign type is ASSUMED, not verified")
- expect: the foreign's `GlobalId` **surfaces** in `trusted_base()`, listed
  exactly (alongside any `ensures` runtime-check hole slots, `foreign.rs:185`)
- why: (soundness ★) AC2, the completeness net. A `pure` annotation on a
  `foreign` is a claim, not a check (`64 §4.3`) — so it **must** appear in the
  delta the consumer audits. The case drives the **real** foreign admission, not
  a hand-built delta. **Flip:** case B2 (discharge) removes it. **Trust level:
  `[by construction]` — the case drives the real `trusted_base()` producer, but
  the completeness *guarantee* (choke-point coincides with the filter) is
  trusted-as-code, not kernel-proved (`64 §1.2`).**

### security/trust-model/discharged-hole-empties-delta
- spec: `64 §1.2` (TB-Complete), `64 §5`, `24 §2` (typed holes)
- given: a typed hole admitted as an opaque postulate (`foreign.rs:185`,
  "already admitted as an opaque postulate"), then **discharged** by
  `upgrade_to_transparent(id, body)` (`env.rs:349`) with a real body
- expect: after discharge the hole's `GlobalId` **leaves** `trusted_base()` (the
  delta shrinks by exactly it) — an assumption became a checked definition
- why: (soundness ★) AC2. Discharging an assumption confines it. **Absence is
  guard-gated:** the id is gone **because** `upgrade_to_transparent` replaced
  `Decl::Opaque` with `Decl::Transparent` and the filter excludes `Transparent`
  — the exact `Opaque`→`Transparent` transition, not a coincidental emptiness.
  Paired with B1 (which *fails* under a hide-the-foreign bug), the before/after
  transition nets completeness.
  **Trust level: `[by construction]` (TB-Complete); real producer + real
  discharge path.**

### security/trust-model/user-assumption-never-prelude-hidden
- spec: `64 §1.2` (TB-Complete), `16 §1.3` (prelude excluded)
- given: a user postulate/`foreign` with a fresh `GlobalId` (≠ `top_id`,
  ≠ `bottom_id`)
- expect: it is **never** excluded by `is_prelude` — it surfaces in
  `trusted_base()`; the enumeration's **only** exclusion is the two fixed
  prelude constants `Top`/`Bottom` (`env.rs:256`)
- why: (soundness ★) AC2, the **no-silent-catch-all** structural assertion. The
  security claim fails silently if the exclusion filter over-matches and
  swallows a user assumption. Pins `is_prelude` = {`Top`, `Bottom`} exactly —
  nothing user-introduced can hide behind the prelude filter. **Flip:** a
  producer with a broadened exclusion (dropping a category of user `Opaque`)
  fails this case while A1/A2 still pass. **Trust level: `[by construction]` —
  the admission-surface exhaustiveness invariant, trusted-as-code (`64 §1.2`).**

### security/trust-model/registered-primitive-surfaces-in-delta
- spec: `64 §1` (TCB item 2 — primitive reductions), `64 §1.2` (TB-Complete),
  `14 §5`
- given: a registered primitive type/operation admitted via `declare_primitive`
  (`check.rs`, `14 §5`) as a `Decl::Primitive` — a fresh `GlobalId`, ≠ prelude
- expect: it **surfaces** in `trusted_base()`, listed alongside postulates — the
  filter's **second** arm (`Decl::Primitive`) enumerates item-2 of the TCB
- why: (soundness ★) AC2, the second filter arm. The primitive reductions are
  TCB item 2 (`64 §1`) — audited kernel-registered ops, trusted-and-*listed*,
  not proven. Exercises the `Primitive` arm of `matches!(Opaque | Primitive)`
  that B1–B3 (the `Opaque` arm) leave untouched: a producer that dropped the
  `Primitive` arm passes B1–B3 but **fails** here, and A1's "empty" is exact
  only when **no** primitive is registered either.
  **Trust level: `[landed producer]` enumeration + `[by construction]`
  completeness (both filter arms).**

## C. Authorship-independence (AC3 ★) — the de-Bruijn security reading

> The pair is **{C1, C2}** — the **same** certificate shape (a `refl`) at a
> **false vs true** proposition, flipping on the kernel `check` verdict alone.
> **C3** pins the structural net: `check`'s signature has no provenance channel,
> so no "trusted author" framing can bypass the verdict.

### security/trust-model/false-proposition-certificate-rejected
- spec: `64 §2.1` (Contract AI-Indep), `23 §1` (never a false `proved`), `18 §5`
- given: a **false**-proposition certificate — `refl` offered at type
  `Id Nat 0 1` — submitted to the kernel `check` (`check.rs:373`),
  **regardless of any "trusted-author" framing**
- expect: `check` **rejects** (conversion fails, `0 ≢ 1`) — the term never
  enters `trusted_base()` as `proved`; no provenance input can change this
- why: (soundness ★) AC3. A generator's bug or malice yields a *rejected*
  certificate, never a false `proved` (`23 §1`). **Flip:** case C2 (the true
  proposition) accepts the same cert shape — the verdict flips on proposition
  validity, on the real kernel `check`, not an elaborator proxy. **Trust level:
  `[structural]` — the accepting/rejecting verdict is the landed kernel check.**

### security/trust-model/genuine-proof-accepted
- spec: `64 §2.1` (AI-Indep), `18 §4`
- given: a genuine proof of a **true** proposition — `refl` at type
  `Id Nat 0 0` — submitted to `check` (`check.rs:373`)
- expect: `check` **accepts** (`Ok(())`) — admission on the kernel's terms
- why: (soundness) AC3, the accepting half of the pair. Same producer (`check`),
  same cert shape as C1; the **only** difference is the proposition's truth.
  **Trust level: `[structural]` — landed kernel verdict, origin-independent.**

### security/trust-model/check-signature-exposes-no-provenance-channel
- spec: `64 §2.1` (Contract AI-Indep), `18 §5`
- given: the kernel check entry `check(env, ctx, t, ty) -> KernelResult<()>`
  (`check.rs:373`)
- expect: the signature takes **exactly** `(env, ctx, term, type)` — **no**
  author / provenance / trust-tier / metadata parameter; the accept/reject
  verdict is a function of `(t, ty)` and `Σ` alone
- why: (soundness ★) AC3, the **structural** net. Authorship-independence holds
  **by construction**: a "trusted author" framing is *inexpressible* at the
  check API, so say-so cannot bypass the check — the property C1/C2 rest on.
  **This is a structural fact about the check signature, NOT a kernel-proved
  theorem** — the corpus must not label it "kernel-backed." **Trust level:
  `[structural]`.**

## D. The trusting-trust invariant (AC4 ★) — the independent second checker

> A named **invariant** the self-host epoch (G8, out of current scope) must
> never break: the small Rust kernel stays an independent second checker only
> while it never depends on a Ken-generated artifact (`64 §3.1`, ADR 0001,
> S1/S2).

### security/trust-model/kernel-has-no-ken-generated-dependency
- spec: `64 §3.1` (Invariant TT), `64 §3` (trusting-trust), ADR 0001
- given: the `ken-kernel` crate's build / dependency closure
  (`crates/ken-kernel/Cargo.toml`)
- expect: the closure contains **no Ken-generated / self-hosted artifact** — it
  does not link, embed, or build-depend on any output of the Ken toolchain
  (currently `[dependencies]` is **empty**: zero deps, so the invariant holds
  maximally)
- why: (soundness ★, invariant) AC4. The self-hosted stack always has an
  independent second checker — a backdoor would have to compromise *two*
  independent checkers identically (Thompson's trusting-trust defence,
  architected in). **Discriminating:** the check **passes** for the landed
  `ken-kernel` (self-contained) and **fails** for a hypothetical kernel manifest
  that took a normal/build dep on a Ken-emitted crate. State as the named
  invariant self-host must preserve — the property that keeps the second checker
  independent. **This is a mechanical build-graph invariant on the manifest,
  NOT kernel-proved and not a runtime property** — do not label it
  kernel-backed. **Trust level: `[structural / architectural]`.**

## E. Honest limits are first-class (AC5) — over-claim is itself a risk

> Not a producer-grep: a **doc-posture fidelity** check that `64 §4`'s limits
> are stated normatively and none is silently dropped or softened. A verified
> language that over-claims is itself a security risk (`64` header) — so the
> limits must be surfaced, not buried, and `§4.1` (spec ≠ intent) must read as
> **the** headline residual risk.

### security/trust-model/honest-limits-stated-normative-not-buried
- spec: `64 §4` (normative), `§4.1`–`§4.5`, `64 §6`
- given: the `§64 §4` honest-limits section, hardened to normative (a conforming
  presentation MUST surface each of §4.1–§4.5, not bury or soften it)
- expect: all boundaries are stated as **normative, externally-legible** claims,
  surfaced not buried, and this seed records them as characterizations to
  preserve —
  1. **spec ≠ intent** (`§4.1`) — the kernel proves code-matches-spec, says
     nothing about whether the spec captures intent; verification *relocates*
     the trust question to "is the spec right?", a human judgment. **This is the
     dominant residual risk — it must read as THE headline, not a footnote.**
  2. **side channels & resource bounds** (`§4.2`) — functional+flow proofs don't
     cover timing/cost; CT is the layered split (`61 §5a`), the timing guarantee
     hardware/codegen-relative and delegated below Ken.
  3. **kernel / FFI / runtime stay trusted, not proven** (`§4.3`) — a `pure`
     annotation on a `foreign` is a claim, not a check; minimised and *listed*
     (it surfaces in `trusted_base()` by TB-Complete, §B), not proven.
  4. **the social / registry layer** (`§4.4`) — squatting, dependency
     confusion, key compromise, governance live *above* the language; Ken makes
     that layer effective but does not replace it.
  5. **regulated industries** (`§4.5`) — Ken is a powerful input to
     certification (it emits assurance artifacts) but does **not** eliminate
     process; it is not a substitute for it.
- why: (fidelity, doc-posture) AC5, the "honest-limits documentation as a
  first-class artifact" `64 §6` deliverable. **Not a verdict-flip:** the check
  is spec-states-each-limit-plainly vs spec-drops-or-softens-one — a
  rendered-faithfully / doc-completeness assertion (the analog of decided-vs-
  rendered). Flagged honestly as such: it has no landed producer; it guards
  against the spec/seed over-claiming by burying a limit. §4.1 (spec ≠ intent)
  is the dominant residual risk and must remain the headline. **Trust level:
  doc-posture fidelity.**
