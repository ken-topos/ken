# First-pass internal kernel audit (Ken)

> Status: **Internal, first-pass audit — NOT the external published report.**
> This is an internal independent read of the landed `ken-kernel` crate against
> the trusted-computing-base contracts in `spec/60-security/64-trust-model.md`,
> performed by the federation's Architect (soundness reviewer). It de-risks the
> G5 posture and is valuable regardless, **but it is not** the external,
> published, independent kernel-audit *report* for consumers — that report, and
> the external-vs-internal / publication governance choice, remain the
> operator's call (`64 §6`, `64 §3`). Companion narrative: the G5 soundness
> story, `docs/program/g5-soundness-story.md`.
>
> **Scope and method.** A point-in-time read of `crates/ken-kernel` at
> `origin/main` `0a06625`. The audit confirms that the kernel's **real trusted
> surface** (its code shape, admission entry points, and dependency closure)
> **matches** the §64 contracts the soundness story rests on. It does **not**
> re-prove the metatheory (subject reduction, normalization, canonicity) — those
> are trusted-as-code metatheorems (§64's framing: the kernel is the trust root
> and does not self-certify), and machine-checking them is out of scope for a
> first pass and named as a limit (§4).

## 1. TCB inventory — `ken-kernel` modules and size

The trusted core is a single crate, `crates/ken-kernel`, of **ten source
modules** totaling **≈5,508 source lines** (≈4,800 non-test, approximate — see
§4). No other crate is trusted: the elaborator (≈11.3k LOC), interpreter
(≈3.3k), runtime (≈1.5k), foundation (≈1.1k), and CLI (≈0.4k) all produce
artifacts the kernel re-checks and are **excluded** from the base.

| Module | Lines | Role in the trust root |
|---|---:|---|
| `check.rs` | 1171 | `check`/`infer` judgments; sort formation; the `declare_*` admission entry points |
| `inductive.rs` | 759 | inductive families, positivity/strict-positivity gate |
| `obs.rs` | 651 | observational equality layer (`Eq`, `J`, `cast`, truncation) |
| `conv.rs` | 642 | definitional equality (`convert`), whnf, Ω-PI shortcut |
| `subst.rs` | 588 | de Bruijn substitution / weakening (capture-avoidance) |
| `sct.rs` | 574 | size-change termination gate (recursion admission) |
| `term.rs` | 518 | the `Term` and `Sort` syntax; no security/modality formers |
| `env.rs` | 430 | `GlobalEnv`, `Decl`, `trusted_base()`, prelude vocabulary |
| `error.rs` | 108 | typed kernel errors (yes/no, never crash contract) |
| `lib.rs` | 67 | crate root / re-exports |

**Finding 1 (size).** The kernel is small enough for one team to review: ≈5.5k
source lines, ten modules, no submodule deeper than the crate. This is
consistent with ADR 0001's "small permanent kernel" and G5's "small enough to
audit." ✅ matches §64 §3.

## 2. The trusted surface — the three TCB items, in the code

Per `64 §1`, soundness depends on **exactly three** things. This audit locates
each in the landed code and confirms the enumeration/completeness contracts.

### 2.1 The admission entry points (item 1 gates + items 2/3 introductions)

There are **five** public admission entry points (`grep 'pub fn declare_'
crates/ken-kernel/src/*.rs`). They split cleanly into **re-checked** (no
per-program trust) and **trusted-assumption** (items 2/3):

| Entry point | Location | Class | Adds a per-program assumption? |
|---|---|---|---|
| `declare_inductive` | `check.rs:887` | re-checked (positivity) | No — re-checked |
| `declare_def` | `check.rs:944` | re-checked (type + SCT) | No — re-checked |
| `declare_recursive_group` | `check.rs:983` | re-checked (type + SCT) | No — re-checked |
| `declare_postulate` | `check.rs:1055` | **trusted (item 3)** → `Decl::Opaque` | **Yes — listed** |
| `declare_primitive` | `check.rs:1074` | **trusted (item 2)** → `Decl::Primitive` | **Yes — listed** |

**Finding 2 (choke-point).** Only `declare_postulate` and `declare_primitive`
introduce an unchecked assumption, and each lands **exactly one** `Decl::Opaque`
or `Decl::Primitive` (`check.rs:1062`, `check.rs:1081`). The definitional entry
points (`declare_def`, `declare_recursive_group`) gate on the type check **and**
`sct::sct_check` before `upgrade_to_transparent` (`check.rs:962`,
`check.rs:1033`) — they are re-checked, not trusted. This is TB-Complete's
choke-point, present in the code as claimed. ✅ matches `64 §1.2`.

### 2.2 The enumeration is the choke-point's filter (TB-Sound / TB-Complete)

`GlobalEnv::trusted_base()` (`env.rs:383`) is:

```
self.decls.iter()
    .filter(|d| matches!(d, Decl::Opaque { .. } | Decl::Primitive { .. }))
    .filter(|d| !self.is_prelude(d.id()))
    .map(|d| d.id())
    .collect()
```

**Finding 3 (sound + complete enumeration).**

- It is a **filter over the real `Σ`** (`self.decls`), so it cannot report a
  phantom assumption — TB-Sound's "no phantom entries." ✅
- Its filter predicate is **exactly** `Opaque | Primitive` — precisely the set
  the two trusted entry points (Finding 2) land. The choke-point that admits
  assumptions **coincides** with the filter that lists them → **no assumption
  can hide** (TB-Complete). ✅ matches `64 §1.1`, `64 §1.2`.
- The prelude exclusion is `is_prelude` (`env.rs:256`) = `self.top_id ==
  Some(id) || self.bottom_id == Some(id)` **and nothing else** — only the two
  fixed `Top`/`Bottom` prelude constants are excluded (kernel vocabulary, `16
  §1.3`), so the exclusion cannot silently drop a *user* assumption. ✅

**Note (arm coverage).** The filter is a **two-arm** `matches!` (`Opaque`
**and** `Primitive`). The conformance corpus must exercise **both** arms — a
corpus that drives only postulates is green-vs-green against a bug that drops
registered primitives from the audited delta. This arm-coverage requirement is
landed in the Sec4 conformance seed (B4 Primitive-arm completeness case, commit
`d600c3c`); the audit confirms the code has both arms and the seed nets both.

## 3. Trust-root facts confirmed against the landed code

Each fact the soundness story and `64` rest on, checked at `0a06625`:

### 3.1 `check()` has no provenance channel (AI-Indep)

`pub fn check(env: &GlobalEnv, ctx: &Context, t: &Term, ty: &Term)`
(`check.rs:373`) — **four** parameters: environment, context, term, type.
**Finding 4:** there is **no** provenance, author, or trust-level parameter. The
verdict cannot be swayed by "this came from a trusted source" because there is
no such input. `infer` (`check.rs:204`) has the same shape. ✅ matches `64 §2.1`
(AI-Indep) — "never a false `proved`, any author" holds *structurally*.

### 3.2 Both-keyed `sort_sigma` (the Sigma-sort soundness fix)

`sort_sigma(s1, s2)` (`check.rs:192`):

```
match (s1, s2) {
    (Sort::Omega(_), Sort::Omega(_)) => Term::Omega(lvl),
    _ => Term::Type(lvl),
}
```

**Finding 5:** a Sigma-type is classified `Omega` **only when both components
are propositions** (the conjunction case); a subset `{x:A | phi}` with a
*relevant* (`Type`-sorted) carrier stays in `Type`. Keying on the codomain alone
(as `Pi` soundly may) would misclassify the relevant-carrier subset as a
proposition, letting Ω-PI erase the carrier and close to `Empty` via a transport
motive. The landed match keys on **both** components. The doc comment
(`check.rs:187–191`) records the trap. ✅ matches the `13 §4` erratum fix (commit
`badc78d`).

### 3.3 Ω-PI proof-irrelevance is coherence-free (a type test, not a value test)

The proof-irrelevance shortcut in `convert` (`conv.rs:336`) fires iff
`is_omega_type(env, ctx, ty)` (`conv.rs:317`), which infers the type of `ty` and
checks it whnf-reduces to `Term::Omega(_)`. **Finding 6:** the guard is a test
on the **type** — "is this a proposition?" — and consults **no** instance
identity, canonicity fact, dictionary, or provenance of the two proof terms.
This is what makes Ω-PI both sound and **coherence-free**: at `Omega`, all
proofs are already definitionally equal, so the kernel never needs to know
"which typeclass instance produced this proof." Typeclass coherence (ADR 0008)
is therefore an elaborator-level convention, **not** a kernel-soundness input
(confirmed absent from the kernel — §3.5). ✅ matches `16 §8.2` and the soundness
story §3.3.

### 3.4 `[dependencies]` is empty (Invariant TT — dependency independence)

`crates/ken-kernel/Cargo.toml` has an **empty** `[dependencies]` section — the
kernel takes **no** external crate dependency at all, and *a fortiori* no
Ken-generated one. **Finding 7:** Invariant TT (`64 §3.1` — the kernel's
dependency closure contains no Ken-emitted artifact) **holds** at `0a06625`,
with maximal margin: there is nothing in the closure but the standard library.
This keeps the permanent Rust kernel a genuinely independent second checker for
the self-host epoch (diverse double-compilation, Thompson defense). ✅ matches
`64 §3.1`.

> **Guard for the self-host epoch.** TT holds *today* trivially. The risk is
> *future*: a self-host WP that lets `ken-kernel` build-depend on a Ken-emitted
> crate would silently break it. The audit recommends a **named regression
> test** asserting the empty/Ken-free dependency closure on the kernel crate, so
> the invariant fails loudly rather than eroding (tracked; see §5). This is
> stated as a follow-on, not a landed gap in the *current* kernel.

### 3.5 No security/modality/class formers in the kernel syntax

The `Term` enum (`term.rs`) has these formers (grep-confirmed): `Var`, `Const`,
`IndFormer`, `Constructor`, `Elim`, `Let`, `Lam`, `Pi`, `App`, `Sigma`, `Pair`,
`Proj1`, `Proj2`, `Type`, `Omega`, `Eq`, `Refl`, `J`, `Cast`, `Ascript`, `Quot`,
`QuotClass`, `QuotElim`, `Trunc`, `TruncProj`. **Finding 8:** there is **no**
label, IFC, `@ct`, modality, policy, temporal, typeclass, or dictionary former
in the kernel — a `grep -niE
'temporal|policy|modal|label|instance|clearance|@ct'` over
`crates/ken-kernel/src/` returns **zero** non-comment hits. Every such feature
is elaborator-level (labels erased before the kernel; dictionaries reified as
Sigma-records). ✅ matches the no-new-kernel record.

### 3.6 The kernel has been frozen since the K-series (empirical no-new-kernel)

**Finding 9:** the last commit to modify `crates/ken-kernel/src/` is `badc78d`
(the K-series Sigma-sort fix). Between `badc78d` and `0a06625`, **85 commits
landed on `main` and none touched the kernel source** — the entire security
spine (Sec1/Sec2/Sec4/Sec5), behavioral (B2/B3/B4), verification, and language
surface (L2–L7, Lc). The small-TCB claim is thus not merely asserted; it is a
**checkable property of the git history**: the trusted surface at the end of the
program is byte-for-byte the K-series kernel. ✅ strongest evidence for `64 §1` +
soundness-story §5.

## 4. Honest limits of *this audit*

This is a first pass, and its own boundaries must be stated (mirroring the
soundness story's discipline):

1. **Not the external report.** This is an *internal* audit by a federation
   role, not the external, published, independent kernel-audit report `64 §6`
   names. It does not carry that report's independence or governance weight; the
   external-vs-internal and publication decisions are the operator's.
2. **Code-shape, not metatheory.** The audit confirms the landed code *matches
   the §64 contracts* (entry points, filter, signature, dependency closure,
   syntax). It does **not** machine-check the metatheorems those contracts
   assume (subject reduction, strong normalization / conversion termination,
   canonicity, positivity soundness). Those remain **trusted-as-code**
   metatheorems, held by review — auditing them (e.g. a mechanized metatheory)
   is a distinct, larger effort and a candidate for the external report's remit.
3. **Approximate LOC split.** The ≈4,800 non-test figure is a heuristic (lines
   before a top-level `#[cfg(test)]`); modules with inline `mod tests` blocks
   are under-counted as code. A precise trusted-vs-test split wants
   `cargo`-level tooling (e.g. `tokei`, coverage). The **total** (≈5,508) and
   the **module count** (ten) are exact.
4. **Point-in-time.** Findings are pinned to `0a06625`. TT (§3.4) in particular
   is a *future* risk surface (self-host); the audit recommends a standing
   regression, not a one-time check.
5. **The gates' internal correctness is trusted-as-code.** The admission gates
   (positivity in `inductive.rs`, SCT in `sct.rs`, quotient respect in `obs.rs`)
   are re-run on every input, but *that each gate is itself correct* is a
   trusted-as-code property this audit reads for shape, not a proof it verifies.
   (The program's own gate-soundness history — e.g. the K1 `subst_tel` and K2
   `check_respect` holes caught by deep-impl review — is why gate correctness is
   named a trust assumption, not assumed away.)

## 5. Findings summary and recommendations

**No soundness gap found in the current kernel.** All nine findings **confirm**
the §64 contracts against the landed code. The trusted surface is small (ten
modules, ≈5.5k LOC), the assumption choke-point coincides with the enumeration
filter (no assumption hides), the check path has no provenance channel, the
Sigma-sort and Ω-PI disciplines are correct, and the dependency closure is
empty.

Recommendations (all **follow-on**, none a current gap):

- **R1 — Standing TT regression.** Add a named test asserting `ken-kernel`'s
  dependency closure is Ken-free (empty today), so the self-host epoch cannot
  silently break Invariant TT (§3.4). Low cost, high durability.
- **R2 — Precise TCB LOC accounting.** Wire a `tokei`/coverage figure for an
  exact trusted-vs-test line split when the external report is scoped (§4.3).
- **R3 — Metatheory scope for the external report.** The external audit `64 §6`
  should decide whether mechanized metatheory (subject reduction, normalization,
  canonicity) is in its remit; this internal pass confirms code-shape only
  (§4.2).
- **R4 — Re-run on kernel change.** This audit is point-in-time; re-run §2–§3 on
  any future commit that touches `crates/ken-kernel/src/` (Finding 9 makes such
  commits rare and conspicuous, which is the point).

---

*This is a first-pass internal audit. It attests that the landed kernel's
trusted surface matches the §64 trust-model contracts at `0a06625`; it does not
substitute for the external, published, independent kernel-audit report, which
remains the operator's governance call (`64 §6`).*
