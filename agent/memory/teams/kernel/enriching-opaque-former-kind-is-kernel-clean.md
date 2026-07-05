---
scope: teams/kernel
audience: (see scope README)
source: private memory `enriching-opaque-former-kind-is-kernel-clean`
---

# Enriching an opaque former's kind is kernel-clean

**Grounded gating `fs-flip-assemble @ e90e51a` (`dec_6d7bjnb7nxbd6`,
2026-07-04).** The `fs-read-file-lines-flip` design enriches the bare surface
`Cap : Type0` opaque postulate into an authority-indexed `Cap : Auth -> Type0`
(so `Cap APartial`, `Cap ANone` are the declared-authority types). I'd flagged
"can `declare_primitive` register a *higher-kinded* opaque former without a
kernel change?" as a hard AC1 STOP condition. It does **not** fire — here's the
recipe, all on `origin/main`:

- **Registration is the same path.** `declare_primitive`
  (`ken-kernel/src/ check.rs:1098`) does only `classify(env, &empty, &ty)?` then
  `env.add_decl(Decl::Primitive { ty, reduction: OpaqueType, .. })`. `classify`
  accepts any term whose `infer` whnf's to `Term::Type(l)`/`Term::Omega(l)`; a Π
  type `Auth -> Type0` infers to `Type1`, so it classifies. `OpaqueType`
  (`env.rs:89`) is a bare marker, **no kind restriction**. ⇒ the Π-typed former
  registers via the identical `Decl::Primitive`, **zero `ken-kernel` delta, no
  new `Term`/`Decl` variant**. The former's *type* changes (`Type0` →
  `Auth -> Type0`); it stays **one** opaque postulate; `trusted_base` gains no
  member (the index type `Auth` is a checked `data`, not a postulate).

- **Indexed instances are DISTINCT types — because an opaque former never
  δ-unfolds.** `OpaqueType` is referenced **nowhere** in the reducer (`conv.rs`
  is the sole reduce/conv file). `unfold_const` returns `None` for a
  non-transparent const (its doc: "opaque/primitive/inductive — no δ"), and
  `whnf`'s `App` arm only unfolds a `Const` head when
  `env.transparent_body(id).is_some()` — false for opaque `Cap` — so it falls to
  "stuck neutral application." ⇒ `Cap APartial` is a stuck neutral
  `App(Cap, APartial)`; conversion compares head + args, so
  `Cap APartial ≢ Cap ANone` (distinct `Auth` ctor args). This is what makes the
  index (a) meaningful as a type distinction and (b) **readable off the app
  spine** (the CLI reads the `Auth` arg from the normalized type — it never
  normalizes away).

**The reusable check for "is enriching an opaque former kernel-clean?":** (1)
does `declare_primitive`/`classify` accept the new (Π) former type? — yes,
classify is kind-agnostic; (2) is `OpaqueType` unreferenced in the reducer so no
delta rule fires? — grep `conv.rs`; if both hold, it's AC1-clean and the indexed
instances are distinct stuck-neutral types. Threading the same index through a
sibling inductive (`FSOp : Auth -> Type0`, `ReadFile (Cap a)`) is an ordinary
indexed inductive family — also no new kernel variant. **Build caveat:** an
added index can shift a ctor's runtime `op_args` positions (cap moves off `[0]`)
— a correctness/QA point, fail-closed under an opaque runtime rep (sole net
prefer structural self evidence over positional scalar), not a soundness hole.
Sibling of abstraction visibility feature soundness gate (reuse the existing
opaque constant, never a new kernel flag).
