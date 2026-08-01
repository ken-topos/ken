# Effects and Authority

Chapters [01](01-anatomy.md)–[03](03-assurance-and-trust.md) taught you to
read a declaration's shape, its contract, and its assurance class. This
chapter asks a different question of the same signature: not "what does it
compute?" but "what is it allowed to *do* to the world outside its own
inputs?" — and which parts the checked catalog can show, versus which belong
to the trusted host/runner rather than to Ken programs.

## Effect Rows

A definition that may touch the world beyond its inputs and outputs declares
an **effect row**: `proc … visits [E₁, …]`
([§1](../../../spec/30-surface/36-effects.md#1-effects-as-a-static-row)). You
met the `proc` keyword itself in chapter [01](01-anatomy.md) as "a
potentially impure definition carrying an explicit effect row"; this is what
that row says. The selected
[console fragment](../../../catalog/packages/Capability/Console/Text.ken.md)
shows it in checked code:

```ken
proc print (text : String) : IO (Result IOError Unit) visits [Console] =
  write Stdout (bytes_encode text)
```

Read the signature before the body, as chapter [01](01-anatomy.md) taught:
`print`'s
type already tells you it is not pure — `visits [Console]` says the *only*
thing it may do to the world is act on the `Console` effect, nothing else.
`eprint`/`printLine`/`eprintLine` in the same file repeat the same shape
against `Stdout`/`Stderr`. A function with **no** `visits` clause — every
`fn` and `const` in every fragment you have read so far — carries the empty
row: it can compute, and nothing else.

The row is not a comment the elaborator merely tolerates, but the rule is
an **inclusion**, not an equation: the body's actual, transitively-inferred
effects must be a **subset** of the declared row — `ρ_inf ⊆ ρ_decl` — not
equal to it
([§1.4](../../../spec/30-surface/36-effects.md#14-checking--declared-rows-and-the-escape-error)).
Under-declaring is the error: `print`'s body calls `write`, a
`Console`-effectful primitive, so if its signature had omitted
`visits [Console]` the missing effect would escape the declared (empty)
row and be rejected. **Over-declaring is not an error** — a `proc` may
name more in its row than its body performs, reserving
headroom for a stable interface that grows later without a signature
change. So the declared row is the complete list of what a definition is
**permitted** to do, an upper bound a reader can trust; it is not, by
itself, proof of what the body currently *does* do.

A second, narrower check reads the row the other way, but only to police
the *keyword*, not the row's contents: a `proc` whose declared row is
**empty** (and which isn't a `space` operation) is flagged as a
should-be-`fn`/`const` mismatch — the reverse-direction purity check
chapter [02](02-types-contracts-and-proofs.md) already taught, restated
here at the row level rather than the keyword level
([§1.6.2](../../../spec/30-surface/36-effects.md#162-the-bidirectional-check--the-keyword-cannot-lie)).
A `proc` that declares `visits [Console]` and never performs it in its
body is still valid under this check — only an
*empty*-row `proc` is suspect.

## Capabilities

The row tells a reader *which* effects a definition may perform. It does
not, by itself, explain *who is allowed* to have that row perform an
effect at all — that is the authority discipline. The checked
[filesystem authority fragment](../../../catalog/packages/Capability/Filesystem/Authority.ken.md)
now exhibits its Ken-visible surface: an explicit `Cap a` parameter beside
the `[FS]` effect row. Its elaboration controls show missing-capability
rejection and authority-index separation. This is checked evidence of
authority-as-signature, not authority for the rule itself.

The authority discipline remains a committed, normative part of the
language (`OQ-8a` DECIDED,
[authority specification](../../../spec/60-security/62-authority.md)). Read
the runnable catalog fragment alongside that source: the fragment exhibits
what Ken code can receive and require, while the specification defines the
trusted host/runner management complement that Ken code cannot call.

- **No ambient authority.** A computation can act on the world only with an
  authority it was explicitly given, and only via an effect its type
  declares; a definition with no effect row and no capability parameter is,
  by its type, inert
  ([§1](../../../spec/60-security/62-authority.md#1-no-ambient-authority)).
- **A capability (`Cap E`) is an unforgeable authority token.** It is part
  of a function's type, so the signature *is* the authority manifest; the
  default authority of any function is none
  ([§2](../../../spec/60-security/62-authority.md#2-capabilities-are-static-visible-and-least)).
- **Attenuation derives a strictly weaker capability, never a stronger
  one — and it is not something Ken code calls.** A trusted runner/host
  action derives a child capability `c'` from a held `c` and a bound `w`
  satisfying `authority c' ⊑ authority c ⊓ w`; this relation is **not a Ken
  declaration or callable signature**
  ([§3](../../../spec/60-security/62-authority.md#3-attenuation--hand-a-child-a-strictly-weaker-token-the-headline)).
  Ken code never invokes `attenuate` — the name is deliberately absent
  from the Ken environment ([§3.2](../../../spec/60-security/62-authority.md#32-no-amplification--assert-the-absence-and-net-the-orientation))
  — it instead **receives** an
  already-attenuated capability as an ordinary parameter, refined by that
  same bound, through an existing privileged route
  ([§2.2](../../../spec/60-security/62-authority.md#22-unforgeability-the-abstraction-boundary)):
  a child helper's signature can require exactly
  `{ c' : Cap_FS | authority c' ⊑ authority c ⊓ only_dir "/tmp" }`, and
  the caller supplies a capability already narrowed that way — the
  narrowing itself happened outside Ken, not by a Ken-callable operation.
- **No amplifying or attenuating operation is bound in Ken at all.**
  `attenuate`, `revoke`, `strengthen`, and any public `Cap` constructor or
  producer are simply **unbound names** — calling any of them from Ken
  code is rejected as `UnboundName`, the same class of error as
  referencing any other undeclared identifier
  ([§3.2](../../../spec/60-security/62-authority.md#32-no-amplification--assert-the-absence-and-net-the-orientation)).
  The checked
  [filesystem authority fragment](../../../catalog/packages/Capability/Filesystem/Authority.ken.md)
  exhibits the callable half: `capability_read` receives an explicit
  `(cap : Cap a)` beside its `[FS]` effect row. The named
  `capability_filesystem_authority_fragment_elaborates` and
  `filesystem_program_requires_the_declared_capability_and_accepts_its_twin`
  tests in
  [`cat_capex_authority.rs`](../../../crates/ken-elaborator/tests/cat_capex_authority.rs)
  check the fragment and its missing-capability boundary. §7's worked spec
  example shows the complementary half: a `sandbox` function receives an
  already-narrowed `/tmp`-scoped capability as a plain parameter and passes
  it on, while `attenuate c (full_authority)` is rejected as `UnboundName`
  ([§7](../../../spec/60-security/62-authority.md#7-worked-examples)).

The catalog now instantiates authority-as-signature as checked code. Its
exemplar deliberately does not exhibit capability minting, attenuation,
revocation, admission, settlement, or audit. That boundary is not a missing
example: `attenuate`, `revoke`, and `strengthen` are, by design, never going
to be operations a Ken program calls. Narrowing happens in a trusted
runner/host outside Ken; Ken code only ever *receives* the narrowed result.

## Corpus Boundary

The checked
[filesystem authority fragment](../../../catalog/packages/Capability/Filesystem/Authority.ken.md)
speaks to authority through capability-typed declarations. It exhibits an
explicit `Cap a` parameter beside `[FS]`, distinguishes `Cap AFull` from
`Cap APartial`, and has paired controls showing that a program without the
declared filesystem capability is rejected while its otherwise identical
capability-bearing twin elaborates. It accepts capabilities as inputs; it
does not define a constructor, producer, wrapper, or management binding for
`Cap`, and it does not exercise a runtime capability identity.

A neighboring
[filesystem fragment](../../../catalog/packages/Capability/Filesystem/Errors.ken.md)
names the complementary confinement boundary in prose: "`Full` retains all
rights, including write and delete, but exercises them only within its
`FsScope`." It also says downstream filesystem resolution enforces confinement.
Read that against the capability authority discipline: the catalog checks
authority as a visible type-level input and pairs it with a per-scope bound
rather than ambient or unconfined filesystem authority.

You can now read an effect row as a checked upper bound and a capability as
the authority needed to exercise an effect. The catalog demonstrates both
that relationship and authority-index separation in checked code. It does
not demonstrate the host/runner complement: minting, attenuation, revocation,
admission, settlement, or audit. Keeping that complement explicit prevents
the checked exemplar from being mistaken for the whole normative capability
model.

---

**Sources:**
[effect rows §§1, 1.4, 1.6.2](../../../spec/30-surface/36-effects.md#1-effects-as-a-static-row);
[authority §§1–3.2, 7](../../../spec/60-security/62-authority.md#1-no-ambient-authority);
[checked filesystem authority fragment](../../../catalog/packages/Capability/Filesystem/Authority.ken.md);
[elaboration controls](../../../crates/ken-elaborator/tests/cat_capex_authority.rs);
[availability record](../../../docs/program/issues/DOC-W1.md);
[registered fragments](fragments.md).
This explanatory chapter distinguishes checked effect-row and
capability-as-signature examples from host-side authority management, which
remains unavailable to Ken programs.
