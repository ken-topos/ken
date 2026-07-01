# Sec4 — the trust model & TCB (§64): the G5 kernel-audit posture

**Steward frame.** WP `Sec4`. Enclave WP (spec-hardening + conformance), `§2c`
process. Owner on release: spec-leader → spec-author (`/spec`) +
conformance-validator (`/conformance`). Reviewers: Architect (soundness), CV
(Spec on `/spec`), spec-author (Fidelity on `/conformance`).

## Why this WP, and why now

`Sec4` is the **G5 gate deliverable** on the security axis. Strategy G5
(soundness) requires two things: the kernel-soundness story, and — per
`spec/60-security/64-trust-model.md §3` — *"a **published, independent audit** of
the Rust kernel is a tier-1 deliverable, not a nicety (strategy G5)."* This WP
lands the **machine-checkable half** of that: the trust-model spec hardened to
normative, and a conformance corpus that pins the claims the whole security
posture rests on — the **enumerable TCB**, **authorship-independence**, and the
**trusting-trust invariant**. It is the natural successor to Sec1 (§61, IFC) and
Sec2 (§62, authority): each Sec-WP hardens one `60-security` chapter + authors
its conformance seed.

`Sec4` is **independent** (no unmet deps): the TCB machinery it pins is landed —
`GlobalEnv::trusted_base()` (`crates/ken-kernel/src/env.rs:383`),
`declare_postulate` (`crates/ken-kernel/src/check.rs:1055`), and the K-api
judgment surface (`18 §5`). It does **not** block on Lc-build, and it directly
feeds the G5 stop-point.

## Locus

- **Spec:** `spec/60-security/64-trust-model.md` — harden **DRAFT v0 →
  normative**. The chapter's structure is sound (TCB / de-Bruijn-as-security /
  trusting-trust / honest-limits / consumer-check / WS-K-deliverables); this WP
  pins the **contracts** the conformance corpus discriminates on and removes the
  DRAFT hedges on the settled decisions below.
- **Conformance (new):** `conformance/security/trust-model/` — does not exist
  yet (the security subtree has `capabilities/`, `ct/`, `ifc/`). Author the seed
  + discriminating cases. Follow the `conformance/security/seed-security.md`
  house style.

## Pinned decisions — settled, do NOT reopen

These are locked by `§64` + prior ADRs; the WP implements them, it does not
re-litigate them.

1. **The TCB is exactly three things** (`§64 §1`, `18 §5`): the kernel; the
   primitive reductions (`14 §5`); the postulates/`foreign` signatures (each in
   `trusted_base_delta`, `25 §3`). **Nothing else** is trusted (not the
   elaborator, prover, SMT, surface compiler, runtime, IFC discipline, or
   package tooling). This is a **closed** enumeration.
2. **The kernel enumerates its own trusted base on demand** (`§64 §1`):
   `trusted_base()` returns exactly the `Opaque` (postulate/hole/`foreign`) +
   `Primitive` decls, prelude vocabulary excluded (`env.rs:383` — the landed
   producer). The complete assumption set behind any artifact is therefore
   **mechanically listable** — this is the foundation of the §5 consumer audit.
3. **De-Bruijn criterion = authorship-independence** (`§64 §2`, `23 §1`): a bug
   or malice in any generator (prover, elaborator, SMT, AI) can cause a *failure
   to prove* or a *rejected certificate* — **never a false `proved`**. Admission
   is on the kernel's terms, never the author's say-so. The check path consults
   **no** provenance/authorship input.
4. **The trusting-trust invariant** (`§64 §3`, ADR 0001, S1/S2): the small Rust
   kernel stays **permanent** even after self-host, and **never depends on
   Ken-generated artifacts** — so the self-hosted stack always has an
   independent second checker (diverse double-compilation built into the
   architecture). Maintaining that independence is a **stated security
   invariant**, mechanically checkable on the kernel crate's dependency closure.
5. **The honest limits are normative and first-class** (`§64 §4`, `§6`): spec≠
   intent is the dominant residual risk (`§4.1`); side channels/resource bounds
   are out of functional+flow proof (`§4.2`, CT is the `61 §5a` layered split);
   the kernel/FFI/runtime stay trusted-and-listed, not proven (`§4.3`); the
   social/registry layer is above the language (`§4.4`). A verified language
   that **over-claims is itself a security risk** — the limits must be surfaced,
   not buried.
6. **The four-point consumer check** (`§64 §5`): content-hash (identity) +
   kernel re-check (correctness, on *your* kernel) + `trusted_base_delta` audit
   (assumptions; empty ⟺ fully verified+confined) + provenance/SLSA (origin).

## Acceptance criteria (discriminating pairs; producer-grep gated)

Every AC is a **non-degenerate pair** that flips on the real mechanism, grepped
to a **landed producer** — never a synthetic literal or a hand-fed value. AC1–AC4
are testable; AC5 is a documentation-posture AC.

- **AC1 — TCB enumeration is SOUND (no phantom assumptions; empty ⟺ verified).**
  A term elaborated with **no** postulate/hole/`foreign` → `trusted_base()` (or
  the artifact's delta) is **empty** (the §5 "empty = fully verified+confined"
  reading). Discriminates against a term carrying one postulate → the delta is
  **non-empty, listing exactly that `GlobalId`**. Verdict flips on real
  `declare_postulate` presence. Producer: `GlobalEnv::trusted_base()`
  (`env.rs:383`) — grep it, do not hand-construct the delta.

- **AC2 — TCB enumeration is COMPLETE (★ security-critical: no hidden
  assumption).** A `foreign`/FFI signature (a listed postulate, `§4.3` / `38
  §3`) or a typed hole (`24 §2`) **must** surface in `trusted_base()`;
  discharging/removing it empties the delta. This is the **untrusted-omission
  net**: the entire "nothing else is trusted / mechanically listable" security
  claim fails silently if an assumption can hide. Discriminating pair on a real
  `foreign`/hole vs. its discharged form. Producer: the same `trusted_base()`
  filter over real `Decl::Opaque`/`Decl::Primitive` — the case must drive a
  **real** `foreign`/hole admission, not insert a decl by hand.

- **AC3 — authorship-independence (★ the de-Bruijn security reading).**
  Admission to `proved` is **independent of any authorship/provenance input**: a
  term asserting a **false** proposition is rejected at kernel check regardless
  of any "trusted-author" framing, and a genuine proof is accepted. Discriminates
  a false-proposition certificate (→ rejected, never enters `trusted_base()` as
  proved) against a valid proof (→ accepted). The net is that the kernel `check`
  path exposes **no** provenance channel by which say-so bypasses the check —
  pin that the discriminant is the kernel verdict, not a metadata flag. Producer:
  the real kernel check entry (`18 §5`), not an elaborator proxy.

- **AC4 — the trusting-trust invariant (★ structural).** The Rust kernel's
  build/dependency closure contains **no Ken-generated artifact** — the
  independent-second-checker property (`§3`). Mechanically checkable on
  `crates/ken-kernel/Cargo.toml`'s dependency graph: it must not link, embed, or
  build-depend on any self-hosted/Ken-emitted output. Discriminating: the check
  **passes** for the landed `ken-kernel` (self-contained, minimal deps) and
  **fails** for a hypothetical kernel that took a dependency on a Ken-generated
  crate. State this as a named **invariant test** (the property the self-host
  epoch must never break), even though self-host (G8) is out of the current
  scope — the invariant is what keeps the second checker independent.

- **AC5 — honest-limits are first-class and normative (doc-posture).** `§64 §4`'s
  four boundaries (spec≠intent; side-channels/resource-bounds; FFI/runtime
  trusted-not-proven; social/registry) are stated as **normative, externally
  legible** claims — surfaced, not buried — and the conformance seed **records
  them as characterizations to preserve** (a rendered-faithfully check on the
  limits prose, the analog of a doc-completeness AC). This is the "honest-limits
  documentation as a first-class artifact" `§6` deliverable. It is **not** a
  producer-grep; it is a fidelity check that the spec states each limit and the
  seed does not silently drop or soften one. Note: `§4.1` (spec≠intent) is the
  **dominant residual risk** — it must read as *the* headline limit, not a
  footnote.

## Trust faces — what this WP delivers vs. what it does NOT

- **Delivered (machine-checkable):** the enumerable TCB (AC1/AC2), the
  authorship-independence property (AC3), the trusting-trust structural
  invariant (AC4), and the normative honest-limits (AC5). Every soundness AC
  bottoms out in **landed** kernel producers (`trusted_base()`,
  `declare_postulate`, the `check` path) — **no new kernel feature**; this is
  pure characterization + conformance over the existing trust root.
- **★ NOT in this WP — flagged for operator (G5 governance follow-on):** the
  **external, published, independent kernel audit *report*** (§64 §3 "published
  kernel-audit posture"; §6). The *machine-checkable posture* (this WP) is the
  substrate; the **human-facing audit report** for CISOs is (a) T4-class public
  documentation — **deferred** under the current re-scope (T4 = agent-context
  only, human docs not yet prioritized), and (b) has a **governance dimension**
  (independence-from-the-builders: an external auditor vs. the Architect as
  internal independent reviewer; publication) that is an **operator call**, not
  an autonomous one. The Steward will surface this to the operator as the G5
  capstone that follows Sec4 — do **not** author the external report inside this
  WP. A lightweight **TCB inventory** (kernel LOC/modules + the `trusted_base()`
  surface, agent-context grade) MAY accompany the seed if cheap, but the polished
  external report is out of scope.

## Producer-grep gate (HIGH-signal, mirrors Lc/L6)

The soundness ACs (AC1–AC4) must drive **real** producers:
- AC1/AC2 → `GlobalEnv::trusted_base()` (`env.rs:383`) over real
  `declare_postulate`/`foreign`/hole admissions. **Not** a hand-built
  `Vec<GlobalId>`, **not** an env-insert that fakes a postulate.
- AC3 → the kernel `check` verdict (`18 §5`). **Not** an elaborator-side
  accept/reject proxy.
- AC4 → the actual `ken-kernel` dependency graph. **Not** a comment asserting
  independence.

Grep the **producer**, not the test. A green case that hand-feeds the delta or
the verdict is green-vs-green and does not net the claim.

## Process (`§2c` / `§14`)

1. spec-leader routes to spec-author (`/spec §64` hardening) +
   conformance-validator (`/conformance/security/trust-model/`), independence
   preserved (author ≠ reviewer of the same artifact).
2. Merge Decision, three gates: **Architect — soundness** (the TCB-closure claim,
   authorship-independence, and the trusting-trust invariant are the
   soundness-critical faces; AC2 + AC4 are the untrusted-omission nets); **CV —
   Spec** on `/spec §64`; **spec-author — Fidelity** on the `/conformance` seed.
3. Integrator merges on green (3-way merge / rebase onto current `main`; spec +
   conformance + docs only, no crates — clean disjoint paths).
4. Retros in → Steward. This closes the machine-checkable G5 security
   deliverable; the Steward then surfaces the external-audit-report governance
   call to the operator.

## References (verify targets, don't launder)

- `spec/60-security/64-trust-model.md` (the chapter being hardened; `§1`–`§6`).
- `spec/60-security/README.md §3` (the property taxonomy — Meta/spec≠intent is
  "not solvable by a language").
- `crates/ken-kernel/src/env.rs:383` (`trusted_base()` — the enumeration
  producer), `crates/ken-kernel/src/check.rs:1055` (`declare_postulate`).
- `spec/10-kernel/18-judgments.md §5` (the K-api / kernel surface + TCB
  definition); `spec/20-verification/23 §1` (never-a-false-`proved`), `25 §3`
  (`trusted_base_delta`).
- ADR 0001 (small permanent kernel), ADR 0004 (security tier-1).
