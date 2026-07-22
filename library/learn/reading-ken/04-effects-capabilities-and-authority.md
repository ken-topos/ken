# 04 — Effects, capabilities, and authority: what the program may do

Chapters [01](01-anatomy.md)–[03](03-assurance-and-trust.md) taught you to
read a declaration's shape, its contract, and its assurance class. This
chapter asks a different question of the same signature: not "what does it
compute?" but "what is it allowed to *do* to the world outside its own
inputs?" — and, honestly, where this curriculum's fragment set can and
cannot yet show you the answer from checked code.

## 1. An effect row is part of the type, and you have already seen one

A definition that may touch the world beyond its inputs and outputs declares
an **effect row**: `proc … visits [E₁, …]`
(`spec/30-surface/36-effects.md`
[§1](../../../spec/30-surface/36-effects.md#1-effects-as-a-static-row)). You
met the `proc` keyword itself in chapter [01](01-anatomy.md) §3 as "a
potentially impure definition carrying an explicit effect row"; this is what
that row actually says. `catalog/packages/Capability/Console/Text.ken.md` —
a selected fragment — is real, current, checked code that shows exactly
this:

```ken
proc print (text : String) : IO (Result IOError Unit) visits [Console] =
  write Stdout (bytes_encode text)
```

Read the signature the way chapter 01 §4 taught: before the body, `print`'s
type already tells you it is not pure — `visits [Console]` says the *only*
thing it may do to the world is act on the `Console` effect, nothing else.
`eprint`/`printLine`/`eprintLine` in the same file repeat the same shape
against `Stdout`/`Stderr`. A function with **no** `visits` clause — every
`fn` and `const` in every fragment you have read so far — carries the empty
row: it can compute, and nothing else.

## 2. The declared row is a checked upper bound, not an exact equation

The row is not a comment the elaborator merely tolerates, but the rule is
an **inclusion**, not an equation: the body's actual, transitively-inferred
effects must be a **subset** of the declared row — `ρ_inf ⊆ ρ_decl` — not
equal to it
(`spec/30-surface/36-effects.md`
[§1.4](../../../spec/30-surface/36-effects.md#14-checking--declared-rows-and-the-escape-error)).
Under-declaring is the error: `print`'s body calls `write`, a
`Console`-effectful primitive, so if its signature had omitted
`visits [Console]` the missing effect would escape the declared (empty)
row and be rejected. **Over-declaring is not an error** — a `proc` may
name more in its row than its current body actually performs, reserving
headroom for a stable interface that grows later without a signature
change. So the declared row is the complete list of what a definition is
**permitted** to do, an upper bound a reader can trust; it is not, by
itself, proof of what the body currently *does* do.

A second, narrower check reads the row the other way, but only to police
the *keyword*, not the row's contents: a `proc` whose declared row is
**empty** (and which isn't a `space` operation) is flagged as a
should-be-`fn`/`const` mismatch — the reverse-direction purity check
chapter [02](02-types-contracts-and-proofs.md) §1 already taught, restated
here at the row level rather than the keyword level
(`spec/30-surface/36-effects.md`
[§1.6.2](../../../spec/30-surface/36-effects.md#162-the-bidirectional-check--the-keyword-cannot-lie)).
A `proc` that declares `visits [Console]` and never performs it in its
current body is still perfectly honest under this check — only an
*empty*-row `proc` is suspect.

## 3. What performing an effect actually requires — and the gap this curriculum is honest about

The row tells a reader *which* effects a definition may perform. It does
not, by itself, explain *who is allowed* to have that row perform an
effect at all — that is the authority discipline, and here this
curriculum's fragment set runs out of checked code to show you.

I looked for a checked fragment anywhere in `catalog/packages/` — not just
this curriculum's seven registered ones — carrying an explicit capability
token, an attenuation call, or an authority comparison in real, `ken
check`-passing surface code. **None exists yet.** `Console/Text.ken.md`
shows the effect row; no catalog entry today shows the capability value
that authorizes performing it. This is not a gap this chapter papers over
— it is itself something worth knowing: the authority discipline below is a
committed, normative part of the language (`OQ-8a` DECIDED,
`spec/60-security/62-authority.md`), and it is honestly **unavailable** in
checked-fragment form in the catalog as it stands today, the same
current/partial/planned/unavailable labelling chapter 03 §4 used for the
`tested` status's own missing tagged construct.

So the rest of this section teaches the authority discipline from its
actual normative source, labelled for exactly what it is: not a fragment
you could open and run.

- **No ambient authority.** A computation can act on the world only with an
  authority it was explicitly given, and only via an effect its type
  declares; a definition with no effect row and no capability parameter is,
  by its type, inert
  (`spec/60-security/62-authority.md`
  [§1](../../../spec/60-security/62-authority.md#1-no-ambient-authority)).
- **A capability (`Cap E`) is an unforgeable authority token.** It is part
  of a function's type, so the signature *is* the authority manifest; the
  default authority of any function is none
  (`spec/60-security/62-authority.md`
  [§2](../../../spec/60-security/62-authority.md#2-capabilities-are-static-visible-and-least)).
- **Attenuation derives a strictly weaker capability, never a stronger
  one — and it is not something Ken code calls.** A trusted runner/host
  action derives a child capability `c'` from a held `c` and a bound `w`
  satisfying `authority c' ⊑ authority c ⊓ w`; this relation is **not a Ken
  declaration or callable signature**
  (`spec/60-security/62-authority.md`
  [§3](../../../spec/60-security/62-authority.md#3-attenuation--hand-a-child-a-strictly-weaker-token-the-headline)).
  Ken code never invokes `attenuate` — the name is deliberately absent
  from the Ken environment (§3.2 below) — it instead **receives** an
  already-attenuated capability as an ordinary parameter, refined by that
  same bound, through an existing privileged route
  (`spec/60-security/62-authority.md`
  [§2.2](../../../spec/60-security/62-authority.md#22-unforgeability-the-abstraction-boundary)):
  a child helper's signature can require exactly
  `{ c' : Cap_FS | authority c' ⊑ authority c ⊓ only_dir "/tmp" }`, and
  the caller supplies a capability already narrowed that way — the
  narrowing itself happened outside Ken, not by a Ken-callable operation.
- **No amplifying or attenuating operation is bound in Ken at all.**
  `attenuate`, `revoke`, `strengthen`, and any public `Cap` constructor or
  producer are simply **unbound names** — calling any of them from Ken
  code is rejected as `UnboundName`, the same class of error as
  referencing any other undeclared identifier
  (`spec/60-security/62-authority.md`
  [§3.2](../../../spec/60-security/62-authority.md#32-no-amplification--assert-the-absence-and-net-the-orientation)).
  §7's worked examples (unavailable in checked form — spec pseudocode, not
  a catalog fragment) show both halves together: a `sandbox` function
  receives an already-narrowed `/tmp`-scoped capability as a plain
  parameter and passes it on to a helper, while a later line —
  `attenuate c (full_authority)` — is marked rejected, `UnboundName`, in
  the same worked example
  (`spec/60-security/62-authority.md`
  [§7](../../../spec/60-security/62-authority.md#7-worked-examples)).

**Read that unavailability label honestly, not as a shortcut past it.**
This is not the same move as inventing a toy example: `62-authority.md` is
the actual normative source for the authority discipline, presented as
what it is — spec prose the catalog has not yet instantiated as checked
code — rather than as something you could go find and run today. And read
the mechanism precisely: the honest boundary here is not merely "no
checked fragment exists yet" — it is that `attenuate`/`revoke` are, by
design, never going to be something a Ken program calls at all. The
narrowing happens in a trusted runner/host outside Ken; Ken code only ever
*receives* the narrowed result.

## 4. A capability's own honest limit, stated in the corpus's own words

One real, current, checked-corpus artifact does speak to authority
directly, even though it is prose rather than a capability-typed
declaration: `catalog/packages/Capability/Filesystem/Errors.ken.md` states,
in its second paragraph, before any code, that "the current authority check
is coarse and is **not** path-confined. An `AFull` capability permits
writes and deletes anywhere the host process can access." Chapter
[03](03-assurance-and-trust.md) §7 already showed you this sentence as an
example of a fragment naming its own honest limit. Read now against §3's
authority discipline: `AFull` is named as a *capability* in this entry's
own words, and its own words tell you plainly that today's check does not
yet narrow that capability's authority to particular paths — the
kind of scoping `attenuate` is built to express is not yet exercised by
this entry. This is a real corpus artifact naming a real, current
limitation in its own voice; it is not, itself, checked capability-typed
code, and this chapter does not present it as such.

## Reader can now answer

- Given a `proc … visits [ρ]` signature, what exactly does `ρ` promise, and
  what happens if the body tries to do something `ρ` doesn't list?
- What does a capability token add on top of an effect row — what
  question does "may this effect be performed here" leave unanswered that
  a capability answers?
- Why does this chapter tell you, explicitly, that no catalog fragment
  today shows a capability-typed declaration — rather than silently
  teaching the authority discipline as if a real example existed?

---

**Grounds this page:**
`spec/30-surface/36-effects.md` §§1, 1.4, 1.6.2;
`spec/60-security/62-authority.md` §§1, 2, 2.2, 3, 3.2, 7.
Authority class: `explanatory` — this page orders and interprets those
sections and the cited fragments' own text; it does not assert a rule they
do not already state. The effects half is grounded in real, current,
checked fragment code (`Console/Text.ken.md`); the authority/capability
half is grounded directly in its normative spec source and labelled
**unavailable** in checked-fragment form, per `docs/program/issues/DOC-W1.md`
§3's capability-labelling discipline — no catalog fragment today carries an
explicit capability token, attenuation, or authority comparison, and this
page does not imply otherwise. Fragments cited are drawn from the
already-selected, registered set in [`fragments.md`](fragments.md); this
chapter does not introduce a fresh selection.
