---
name: write-ken
description: Point at the Ken authoring guide (catalog/guide/) and inline the single highest-value proof technique, so any agent about to write or prove Ken code loads real practice, not just the spec contract or a guess.
scope: tools
model: claude-sonnet-5
---

# Write Ken

You are about to write or prove Ken code — a catalog entry, a conformance
fixture, a spec example, or ordinary `.ken`/`.ken.md` source. Before writing
a line, load the **Ken authoring guide**: `catalog/guide/README.md` and its
three strands (`surface-reference.ken.md`, `proof-techniques.ken.md`,
`decomposition-abstraction.ken.md`). It is the practice companion to
`spec/30-surface` (the contract) — read the guide for *how*, the spec for
*what's normative*. Every example in the guide is real, checked Ken; if
something you need isn't there yet, that's a guide Finding, not a reason to
guess at syntax.

## The one thing to know before you write your first law

**A proof's terminal step at an equation goal is `tt` or `Refl` — never
assume which from the shape of the case, always check.** This single
discriminator has caused more silent proof-authoring stalls than any other
mistake in Ken's history (four independent recurrences across the fleet,
`agent/memory/enclave/tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases.md`).
The rule:

1. **Reduce both endpoints of the goal.**
2. If they land on the **same constructor** (a nullary one, or a
   non-nullary one whose every component also collapses) — the goal has
   observationally collapsed to `Top`. Close with **`tt`**. `Refl` fails
   here (`"Refl expects an Eq-shaped goal"`) because the goal is no longer
   `Eq`-shaped by the time it's checked.
3. If either endpoint is genuinely **stuck** (a bare variable, or an
   application that can't reduce further because something in it is
   abstract) — the goal stays `Eq`-shaped. Close with **`Refl`**. `tt`
   fails here — there is nothing collapsed to `Top` for it to introduce.

```ken example
fn bool_and (a : Bool) (b : Bool) : Bool = match a { True ⇒ b ; False ⇒ False }

-- Both endpoints reduce to the SAME nullary constructor (`True`) ⇒ tt.
const collapsed : Equal Bool (bool_and True True) True = tt

-- The endpoint is a STUCK application over an abstract `x` ⇒ Refl.
fn stuck (x : Bool) : Equal Bool (bool_and x x) (bool_and x x) = Refl
```

Never write "the base case is `Refl`" or "the base case is `tt`" as a
blanket rule for an inductive proof — check each case's *reduced* endpoints
independently. This is the single highest-value check before you commit a
law proof; the full discriminator, with the mismatched-arity gotcha
(`Refl` where a non-nullary head has one neutral component), is
`catalog/guide/proof-techniques.ken.md §1`.

## What to load next, by task

- Writing your first `fn`/`data`/`class` in this session →
  `surface-reference.ken.md`.
- A law won't close, or you're not sure how to structure a case-split with
  more than one hypothesis → `proof-techniques.ken.md §2` (the
  case-split-then-lambda binder-ordering discipline).
- Deciding between a `class` and a bare threaded parameter, or whether to
  fold a new need into an existing mechanism → `decomposition-abstraction.ken.md`.
- Authoring a full catalog entry (not just a fragment) → also read
  `docs/program/07-catalog-style-guide.md` for the required section order,
  fence-role table, and Findings/References requirements.

## Findings loop

If writing against this guide surfaces a gap, a clearer technique, or a
recurring shape the surface should sugar, that is a **Finding**
(`docs/program/06-catalog-campaign.md §"Retro discipline"`) — record it in
your work's own Findings section, and route it: guide/skill gaps fold back
into `catalog/guide/` directly; a sugar candidate goes to Ergo; a reusable
`def`/`lemma`/`prop` gets promoted into the catalog; a kernel-reduction
defect goes to Kernel via the enclave.
