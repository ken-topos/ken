# ADR 0016 — Re-export surface form and the canonical-identity invariant (MRES-9 / #36 · N5)

- **Status:** **Accepted** — the semantics were designed by the Architect
  (canonical-identity invariant, collision behaviour, MRES-4d instance-surface
  carry, the §32/§4 shape); the **operator chose the surface spelling on
  2026-07-13: Option B — the dedicated `export` declaration** (§2). The
  considered-but-not-chosen options A and C are retained in §2 for the record;
  §2.1, §3, and §4 are now written to the decided Option B.
- **Date:** 2026-07-13 (framed by the Architect, design round #36/N5; operator
  spelling pick — Option B — folded same day).
- **Deciders:** the operator (spelling pick — Option B, 2026-07-13); semantics
  framed by the Architect.
- **Relates to:** ADR 0014 (MRES-9 re-export defer, MRES-4d instance-surface
  carry, MRES-4c source==compiled, the program/package admission boundary),
  ADR 0015 (`use M` removed — so `pub use` is off the table), spec
  `30-surface/33-declarations.md` §3.2 (import forms), §3.3 (name resolution
  + the #39 general-clash rule), §4 (visibility / abstract export), §5.5.1
  (program/package admission — its SPEC-NOW/BUILD-LATER re-export bullet),
  and `30-surface/32-grammar.md` §1 (declaration grammar). Companion to the
  #39 MRES-6 general-clash amendment (same ADR 0014 design round).

## Context

MRES-9 (ADR 0014) **accepted deferring the re-export form** to a post-loader
round while **fixing the canonical-identity invariant now**. That deferral
has now been reached: packages have public topologies (the N2 loader landed;
the program/package admission boundary is specified in §5.5.1), so the form
must be designed before a public API can re-expose a name it did not itself
define.

Three fixed inputs constrain the design:

1. **`use M` is retired (ADR 0015).** Ken's only bring-all-unqualified form is
   gone; the `use` *production* is retired — the keyword is kept **reserved**
   with a migration diagnostic (#37, `KwUseReserved`), not freed. The re-export
   form therefore **cannot** spell as `pub use` — that vehicle no longer exists.
   The surviving import
   forms are the three provenance-preserving ones (`import M`, `import M as N`,
   `import M (…)`).
2. **Canonical identity is one-per-declaration (MRES-9).** Every declaration
   has exactly one canonical identity — its `defined-at` `GlobalId` in the
   flat `Σ` (spec §3/§4). A re-export **republishes** that identity under
   another module's interface; it must **never mint a second identity**. This
   is the invariant MRES-9 recorded and this ADR now discharges into a
   mechanism.
3. **Re-export carries the instance surface (MRES-4d).** Re-exporting a name
   also carries the instances that name's public API commits to into an
   admitting consumer's direct-use set (ADR 0014 MRES-4d; spec §5.5.1's
   SPEC-NOW/BUILD-LATER bullet). Re-export is the **one lever** an author
   uses to govern both the public **name** surface and the public **instance**
   surface.

**Nothing here is a kernel change.** Re-export is surface + elaboration only:
it adds a name to a module's `pub` interface and elaborates away to the flat
`Σ` the kernel already sees — **zero `trusted_base()` delta** (spec §3/§4.2,
the ES1 minimality invariant). The kernel re-checks every instance dictionary
value regardless, so the MRES-4d carry is a resolution-time admission concern,
not a trust surface.

## 1. What is decided here (semantics — Architect design authority)

The following are **decided** (not operator forks). Only the surface *spelling*
(§2) is deferred to the operator.

### 1.1 Canonical identity — `defined-at` vs `re-exported-as`

- **One identity, many surface paths.** A declaration's canonical identity is
  fixed at its **defining** module. If `M` defines `foo`, then `M.foo`, a
  re-export of it from `P`, and a re-export of `P`'s re-export from `R` are
  all the **same** canonical declaration — three surface paths, **one**
  `GlobalId`. Re-export allocates **no** new global.
- **Renamed re-export changes only the surface name.** A renamed re-export
  (`export M (foo as bar)`) publishes the `re-exported-as` surface name `bar`
  while the `defined-at` identity stays `identity(M.foo)`. Identity is invariant
  under renaming and under any number of re-export hops.
- **Abstract export composes (§4.2).** Re-exporting an abstractly-exported
  type re-exports the **opaque constant** — the constructors stay hidden,
  because visibility travels with the identity, not with the surface path.
  Re-export cannot widen an abstract type's constructor visibility.

This `defined-at` / `re-exported-as` distinction is the normative core; the
grammar and the visibility rule below are its surface and its
bookkeeping.

### 1.2 Collision checking — keyed on identity, coherent with #39

Re-export is designed to interlock with the **#39 general-clash rule** (the
companion ADR 0014 MRES-6 amendment). #39's rule fires when more than one of
{local def, selective/renamed import, prelude} binds one unqualified name to
**distinct canonical identities**. Re-export slots into that rule cleanly
**because the rule keys on identity, not spelling** — a coupling designed
deliberately so re-export never spuriously clashes:

- **Same identity via two paths is NOT a clash** (the "re-export is not
  ambiguous" carve-out). `import M (foo)` and `import P (foo)` where `P`
  re-exports `M.foo` both resolve to `identity(M.foo)` — one identity, no
  `AmbiguousReference`. This is the idempotent case #39's carve-out admits.
- **Two different identities under one name IS a clash.** A re-exported `foo`
  and a genuinely different local or imported `foo` are two identities for
  one unqualified name → `AmbiguousReference`, resolved by the #39 escape
  hatches (drop/rename one selective item, qualify, or rename the local).
- **Re-export-site collision.** A module may **not** re-export two *different*
  identities under one surface name — that would republish an ambiguity. This
  is a hard surface error at the re-export site, with the survey-P5
  diagnostic form: "defined at `X` vs re-exported as the same name from
  `Y`." Re-exporting an identity already in the module's interface under the
  same surface name is **idempotent** (no error).

### 1.3 MRES-4d — instance-surface carry (now-designed mechanism)

ADR 0014 MRES-4d accepted that re-export carries the instance surface; spec
§5.5.1 records it as SPEC-NOW/BUILD-LATER. The concrete mechanism:

- **Which instances ride.** When `P` re-exports a name whose public surface
  mentions a type `T` (re-exporting `T` itself, or a value whose type surface
  mentions `T`), the instances carried into an admitting consumer's
  **direct-use set** are the canonical **structure** instances — defined
  anywhere in `P`'s coherence closure — whose `(class, head-type)` key's
  head-type or class is part of `P`'s **re-exported public surface**.
  Property (Ω-valued) instances are proof-irrelevant and never conflict (spec
  §5.5), so they carry trivially.
- **Admitting `P` grants dispatch on the carried instances.** A consumer that
  admits `P` may dispatch the carried instances **without** listing their
  defining package `Q`. An instance `P` did **not** re-export stays
  **coherence-only**; a consumer unit that dispatches it makes `Q` a direct
  dependency that must be admitted (`UnadmittedInstance`, spec §5.5.1). One
  lever — re-export — governs both `P`'s public name surface and its public
  instance surface, exactly MRES-4d.
- **Boundary, not trust.** The carry is computed at the admission boundary
  (spec §5.5.1), keyed on the re-export set. It is an elaboration-time
  direct-use-set computation — **no new TCB** (the kernel re-checks every
  dictionary value irrespective of admission).

## 2. Surface spelling — DECIDED: Option B (`export` declaration)

`use` is retired, so `pub use` was unavailable. Three concrete,
well-differentiated spellings were surfaced to the operator, each realizing the
identical §1 semantics and differing only in ergonomics and granularity. **The
operator chose Option B — the dedicated `export` declaration (2026-07-13)** for
its clean import/export split, per-name granularity, and native renamed + local
re-export; the new `export` keyword is a net **+1** reserved word (#37 keeps
`use` reserved, not freed) — a small, accepted cost for the clean split.
Options A and C, and the rejected `pub`-alias shape, are retained below for the
record; the resolved Option B semantics are in §2.1.

### Option A — `pub` on the import statement (statement-level re-export)

```ken
pub import M (foo, Bar)      -- every name this import brings is re-exported
pub import M as N            -- re-exports M's namespace under N
                             --   (client sees N.foo)
pub import M                 -- re-exports M's qualified namespace
                             --   (client sees M.foo)
```

Marking an `import` `pub` republishes the names it brings into this module's
`pub` interface, each carrying its canonical identity.

- **For.** Reuses the **existing `pub` lever** — one concept, "`pub` = part of
  my interface," now applied to imports as well as definitions. **No new
  keyword.** Reads naturally; the re-export decision sits on the statement
  that introduces the names.
- **Against.** **Statement-granular** — re-exports *all* names the import
  brings or none; re-exporting a subset means splitting into two import
  statements (one `pub`, one plain). `pub import M` re-exporting a whole
  qualified namespace is coarse.

### Option B — a dedicated `export` declaration ✅ CHOSEN (operator, 2026-07-13)

```ken
export M (foo, Bar)          -- re-export selected names from M
export M (foo as bar)        -- renamed re-export
export foo, Bar              -- re-export names already in scope
                             --   (imported or local)
```

A statement distinct from `import`, dedicated to republishing.

- **For.** **Clean separation** of "bring into my scope" (`import`) from
  "republish to my clients" (`export`); re-exports imports **and** local names
  uniformly; **per-name granularity** and renamed re-export fall out naturally.
- **Against.** **New reserved keyword `export`** (costs one identifier from the
  surface — #37 keeps `use` reserved with a migration diagnostic, so `export` is
  a net **+1** reserved keyword). Two forms (`import` + `export`) where the
  `pub`-lever options reuse one; slight conceptual overlap with `pub`.

### Option C — per-item `pub` inside the selection list

```ken
import M (pub foo, Bar)      -- re-export foo; keep Bar
                             --   imported-but-private
import M (pub foo as bar)    -- renamed re-export of a single item
```

- **For.** **Finest granularity** — per-name re-export control in one place,
  on the name itself; no new statement or keyword (reuses `pub`); renamed
  re-export composes with the existing selective `as`.
- **Against.** **Denser selection lists**; `pub` now appears in two positions
  (declaration-level and selection-item-level) — mild overloading of the
  token. Does not by itself re-export a *local* name (but a local `pub`
  definition already exports, so this is not a real gap).

### Rejected (documents the invariant) — `pub` transparent alias

```ken
pub const foo = M.foo        -- NOT a re-export
```

A public transparent alias **mints a new canonical identity**
(`thisModule.foo`, a fresh `GlobalId`) that δ-unfolds to `M.foo`. Two
identities for one thing is exactly the hazard MRES-9's invariant forbids
(§1.1). This is a definitional alias, not a re-export, and is **not** an
acceptable spelling for the form — listed to fix the boundary: a real
re-export republishes an identity, it does not wrap it in a new one.

### Comparison

| Axis | A `pub import` | B `export` decl | C per-item `pub` |
|---|---|---|---|
| New keyword | none | `export` | none |
| Granularity | statement | per-name | per-name |
| Re-exports locals | no (use `pub` def) | yes | no (use `pub` def) |
| Renamed re-export | via `import … as` | native | via item `as` |
| Concept count | reuses `pub` | new statement | reuses `pub` |

All three realize §1 identically; the operator's pick (Option B) selects the §3
grammar branch and the §4 spelling below.

## 2.1 Option B — resolved semantics (Architect, decided)

The operator's pick surfaces four details that Option B specifically must settle;
each is decided here so the enclave spec edit is unambiguous.

- **Two forms, each earning its place.**
  - **`export M (…)` (module-selection)** re-exports names taken **directly from
    `M`'s exports** — a **pure facade republish** that does **not** require a
    prior `import M` and does **not** bind those names into the current module's
    own body scope. This is the package-facade pattern (a top module republishes
    submodule names it does not itself consume). The `export M (…)` statement is
    itself a loader dependency edge to `M` (same role-blind path identity as
    `import`, §3.2.1).
  - **`export foo, Bar` (name-list)** re-exports names **already resolved in the
    current module's scope** — whether brought by an `import` or defined locally.
    This is the **compose-to-use-and-republish** pairing: `import M (foo)` then
    `export foo` both *uses* `foo` locally and republishes it. A name in the list
    that resolves to nothing in scope is an ordinary unresolved-name surface
    error.
- **The import/export split is real (the reason Option B was chosen).** `import`
  brings names **into this module's body scope**; `export` puts names **into
  this module's public interface**. They compose rather than overlap: to use *and*
  republish, pair them (`import` + `export foo`); to republish without using,
  `export M (…)` alone. `export M (…)` deliberately does **not** also bind into
  the body — keeping the two levers orthogonal is exactly the clean split the
  operator selected.
- **Renaming is available in both forms.** `export M (foo as bar)` and
  `export foo as bar` both republish the source identity under the surface name
  `bar`; identity is unchanged (§1.1). An `export` item is `name ("as" name)?`.
- **Re-export of a local name is accepted and identical to `pub`.** `export foo`
  where `foo` is a local definition adds `foo`'s own identity to the interface —
  the same effect as `pub` on that definition. `pub`-on-definition remains the
  idiomatic way to export a *local*; `export` of a local is accepted (uniform
  with re-exporting an imported name) and mints no second identity.
- **Re-export-site collision under B (§1.2 applied).** An `export` participates
  in the §1.2 identity-keyed rules: a module may not `export` two **different**
  identities under one surface name (e.g. a local `pub const foo` of identity `A`
  and `export M (foo)` of identity `B`) — that is a hard re-export-site error
  ("defined at `X` vs re-exported as the same name from `Y`"). Re-exporting an
  identity already in the interface under the same name is **idempotent**. In a
  **consumer**, an `export`ed name entering scope participates in the #39
  general-clash rule, keyed on identity (§1.2) — same-identity is not a clash.

## 3. Grammar additions (§32) — Option B (pinned)

Surface-only productions; each **elaborates away** (zero `trusted_base()`
delta). Final spec text is the enclave's; the pinned Option B branch:

```
export_decl ::= "export" ( module_path selection_list | export_item_list )
selection_list  ::= "(" export_item ("," export_item)* ")"
export_item_list ::= export_item ("," export_item)*
export_item ::= name ( "as" name )?
```

- Reserve **`export`** as a keyword — a net **+1** reserved word; #37 keeps
  `use` reserved (retired with a migration diagnostic, not freed).
- The `module_path` uses the role-blind path identity of §3.2.1; `export M (…)`
  is a loader dependency edge to `M`.
- The re-exported name's canonical identity is the imported (or defining)
  declaration's; **no production introduces a new `GlobalId`**.

Options A (`import_decl ::= "pub"? "import" …`) and C (`selection_item ::=
"pub"? …`) are **not** taken; their sketches are dropped from the normative path
and survive only in §2's record.

## 4. Visibility additions (§4) — Option B

A normative §4 addition (a new §4.3 or an extension of §4.1):

- **Interface = own `pub` defs ∪ `export`ed names.** A module's `pub` interface
  is the union of (a) its own `pub` top-level definitions and (b) the names it
  `export`s. An `export`ed name contributes to the **interface**, not a new
  **declaration**: its canonical identity remains its defining module's.
- **`defined-at` owns identity; `re-exported-at` republishes it.** This is the
  normative distinction (§1.1). `grep`-recoverable provenance is preserved: an
  `export`ed name's origin is recoverable from the `export` statement, exactly
  as a selective import's origin is recoverable from its import block (the ADR
  0015 provenance discipline extends to `export`).
- **Abstract export composes (§4.2).** `export` carries the visibility that
  travels with the identity — `export`ing an abstract type re-exports the opaque
  constant, constructors still hidden.
- **`export` of a local vs `pub` (§2.1).** `export foo` for a local definition
  is accepted and equal to `pub foo` on that definition; `pub`-on-definition
  stays idiomatic for locals, `export` for republishing imported names.

## 5. MRES-9 status flip (recorded in ADR 0014)

The companion edit flips ADR 0014's **MRES-9** from "defer the form; fix the
invariant now" to: **form DESIGNED AND DECIDED (this ADR)** — the
canonical-identity invariant and MRES-4d instance-surface carry are specified,
and the surface spelling is **Option B, the dedicated `export` declaration**
(operator, 2026-07-13). MRES-4d and the canonical-identity invariant are
recorded there as **now-designed**, with a cross-reference to this ADR.

## 6. Follow-on WPs (named, not authored here)

This ADR is **design-only**. The operator spelling decision is **made** (Option
B), so the follow-ons below are **unblocked** and the Steward frames them:

1. **Spec-enclave WP** — normative edits for Option B: §3.2 (the `export`
   declaration, both `export M (…)` and `export foo, Bar` forms, §2.1), §3.3
   (`export`ed names participate in the #39 clash rule, keyed on identity — the
   §1.2 carve-out), §4 (interface = `pub` defs ∪ `export`ed names;
   `defined-at` vs `re-exported-at`), §32 (the pinned §3 `export_decl`
   grammar), and flipping the §5.5.1 re-export SPEC-NOW/BUILD-LATER bullet to
   normative.
2. **Conformance WP** — fixtures: identity preservation (`export` → same
   `GlobalId`), renamed `export` (`export M (foo as bar)`), the facade
   `export M (…)` and in-scope `export foo` forms (§2.1), re-export-site
   collision (two identities, one name → reject), the same-identity
   **non-clash** positive case, instance-surface carry into the direct-use set
   (MRES-4d), and abstract `export` keeping constructors hidden.
3. **Language build WP** — `export` elaboration: reserve the keyword, parse
   `export_decl` (§3), republish the source identity (no new `GlobalId`),
   extend the module-interface computation (§4), enforce the import/export
   split (`export M (…)` does not bind into the body — §2.1), and wire the
   MRES-4d instance-surface carry into the §5.5.1 admission gate. Queues behind
   Language's current kenfmt + #37 (Steward's sequencing).

## Revisit if

- A concrete workflow needs a re-export granularity none of A/B/C serves (e.g.
  wildcard re-export of a whole interface) — weigh against the ADR 0015
  provenance discipline (a wildcard re-export reintroduces the `use`-style
  origin-loss the removal was meant to end).
- The compiled-package manifest round (MRES-4c) surfaces a re-export
  interaction the source-world design did not anticipate (the manifest must
  record re-exported instance-surface commitments identically to a source
  rebuild — the source==compiled invariant extends to re-export).
