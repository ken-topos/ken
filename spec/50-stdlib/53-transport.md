# Transport combinators — the derived `subst`/`cong`/`cast`/`sym`/`trans`

> Status: **DRAFT v0 (surface-transport WP, Map Gap A).** Five everyday
> equality-rewriting combinators, each a thin non-recursive `.ken` `view` over
> the single surface former **`J`** (`../30-surface/34 §3.4`). They add
> **nothing** to `trusted_base()`: every one reduces to `Term::J` → `cast`
> (`../10-kernel/15`/`16`), already trusted — no `declare_primitive`, no
> `declare_postulate`, no kernel change. They are **library, not formers**;
> `J` is the only new surface syntax the WP introduces.

## 1. What this module is

`J` (`../30-surface/34 §3.4`) is the complete primitive for transporting a goal
along a **propositional** equality `p : Eq A a b`. The combinators below are the
conventional named idioms derived from it — the surface `.ken` counterparts of
the kernel derivations already sketched in `../10-kernel/15 §3`/`§4`. Each is a
**non-recursive** `view` — SCT-trivial (no self-edge, so termination is
immediate); none is a kernel primitive, and none needs its own surface support.

The equality used throughout is the **kernel's native, computing `Eq`**
(`../10-kernel/16 §2`) — the one carrying `refl`/`J` — **not** the postulated
`Equal` slated for removal (`../30-surface/30 §6`, the taxonomy prune): only the
computing `Eq` can be transported, since `J`/`cast` reduce on it.

## 2. The five combinators

Each definition is exactly the `J`-motive the identity eliminator's typing rule
(`../30-surface/34 §3.4`) accepts; the surface intro for `Eq` is `Refl`.

```
view subst (A : Type) (a : A) (b : A) (P : A → Type)
           (p : Eq A a b) (pa : P a) : P b =
  J (\b' _. P b') pa p

view cong (A : Type) (B : Type) (a : A) (b : A) (P : A → B)
          (p : Eq A a b) : Eq B (P a) (P b) =
  J (\b' _. Eq B (P a) (P b')) (Refl (P a)) p

view cast (A : Type) (B : Type) (e : Eq Type A B) (t : A) : B =
  J (\X _. X) t e

view sym (A : Type) (a : A) (b : A) (p : Eq A a b) : Eq A b a =
  J (\b' _. Eq A b' a) (Refl a) p

view trans (A : Type) (a : A) (b : A) (c : A)
           (p : Eq A a b) (q : Eq A b c) : Eq A a c =
  J (\c' _. Eq A a c') p q
```

Each type-checks by the `J` rule (`34 §3.4`), reading `motive b eq` off the
motive:

- **`subst`** — motive `\b' _. P b'` (first domain `A`; base `P a`, given `pa`;
  result `P b`). Type-family transport.
- **`cong`** — motive `\b' _. Eq B (P a) (P b')`, whose codomain is
  `Eq B _ _ : Ω`; base `Eq B (P a) (P a)`, given `Refl (P a)`; result
  `Eq B (P a) (P b)`. This is the combinator that **relies on the unconstrained
  codomain sort** — its motive lands in `Ω`, not `Type`.
- **`cast`** — motive `\X _. X` (large elimination into `Type`); base `A`, given
  `t : A`; result `B`. Raw type-transport derives from `J` too — no separate
  former. Its `e : Eq Type A B` need not have `A ≡ B` (`34 §3.4`, cast rule).
- **`sym`** — motive `\b' _. Eq A b' a`; base `Eq A a a`, given `Refl a`; result
  `Eq A b a`.
- **`trans`** — motive `\c' _. Eq A a c'` over `q : Eq A b c`; base `Eq A a b`,
  given `p`; result `Eq A a c`.

## 3. Transporting an `Ω`-valued goal (the Map Branch-B case)

`subst` above is stated for a `Type`-valued family `P : A → Type`. A goal that
lives in **`Ω`** — every Branch-B `Map` law (`52 §5`), whose obligations are
proof-irrelevant `Ω` propositions — is transported by using the **`J` former
directly** with an `Ω`-valued motive `\b' _. G[b'] : Ω`, e.g.

```
  proof : G[leq k k']
  proof = J (\x _. G[x]) (pf_true : G[True]) (sym q)     -- q : Eq Bool (leq k k') True
```

The unconstrained motive codomain (`34 §3.4`) is exactly what admits this; a
sort-polymorphic `subst` is **not** needed and **not** provided. The four
comparison-dependent Map laws (preservation, found-after-insert, locality,
agreement) are gated on this WP together with the non-nullary dependent match of
Gap B (`52 §7d`, `map-verified-laws`).

## 4. Trust surface

Zero delta. Grep discipline for the build: no `crates/ken-kernel/` file is
touched, no new `Decl`/`Term` variant, and no
`declare_primitive`/`declare_postulate` — the five `view`s reduce through the
**existing** `Term::J`/`Term::Cast`
(`../10-kernel/15`/`16`), already in `trusted_base()`. A mis-typed combinator
application is caught by the ordinary kernel re-check of its emitted `J`/`cast`
term (`34 §3.4`), the same net as any other `view`.
