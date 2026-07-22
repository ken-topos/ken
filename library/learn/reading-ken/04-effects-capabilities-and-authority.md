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

## 2. Effects are checked both ways — a signature cannot over- or under-claim

The row is not a comment the elaborator merely tolerates: a definition's
*declared* row and its body's actual, transitively-inferred behavior are
checked against each other, in both directions
(`spec/30-surface/36-effects.md` §1.6.2, already cited in chapter
[02](02-types-contracts-and-proofs.md) §1 for the `fn`/`proc` purity rule).
`print`'s body calls `write`, a `Console`-effectful primitive; if its
signature had omitted `visits [Console]`, the same rule chapter 02 already
taught you — an effect the row doesn't declare is a hard error at the
definition site — would reject it. The declared row is therefore not
decoration: it is the one place a reader can find, by construction, the
complete list of what a definition might do beyond computing a return
value.

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
  one** — `attenuate : (c : Cap E) (w : Authority) → { c' : Cap E |
  authority c' ⊑ authority c ⊓ w }`, so a delegated helper cannot exceed
  the authority its caller handed it, by construction rather than by
  review
  (`spec/60-security/62-authority.md`
  [§3](../../../spec/60-security/62-authority.md#3-attenuation--hand-a-child-a-strictly-weaker-token-the-headline)).
  §7's worked examples (unavailable in checked form — spec pseudocode, not
  a catalog fragment) show this concretely: a capability attenuated to
  `/tmp` is accepted at a sink that only needs `/tmp`, and rejected at a
  sink that demands `/etc`
  (`spec/60-security/62-authority.md`
  [§7](../../../spec/60-security/62-authority.md#7-worked-examples)).

**Read that unavailability label honestly, not as a shortcut past it.**
This is not the same move as inventing a toy example: `62-authority.md` §7
*is* the actual normative source for the authority discipline, presented as
what it is — spec prose the catalog has not yet instantiated as checked
code — rather than as something you could go find and run today.

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
`spec/30-surface/36-effects.md` §§1, 1.6.2;
`spec/60-security/62-authority.md` §§1, 2, 3, 7.
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
