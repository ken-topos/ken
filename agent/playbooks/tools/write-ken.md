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
fn bool_and (a : Bool) (b : Bool) : Bool = match a { True ↦ b ; False ↦ False }

-- Both endpoints reduce to the SAME nullary constructor (`True`) => tt.
const collapsed : Equal Bool (bool_and True True) True = tt

-- The endpoint is a STUCK application over an abstract x => Refl.
fn stuck (x : Bool) : Equal Bool (bool_and x x) (bool_and x x) = Refl
```

Never write "the base case is `Refl`" or "the base case is `tt`" as a
blanket rule for an inductive proof — check each case's *reduced* endpoints
independently. This is the single highest-value check before you commit a
law proof; the full discriminator, with the mismatched-arity gotcha
(`Refl` where a non-nullary head has one neutral component), is
`catalog/guide/proof-techniques.ken.md §1`.

## A brief's pinned syntax is intent, not a spelling — probe it first

A design brief (an enclave pin, a WP frame) states *what* to build in Ken
terms, but its literal syntax is not verified-parseable until you actually
run it through the elaborator — the same discipline as the `tt`-vs-`Refl`
rule above, applied to syntax instead of proof terminals. DS-1's first
build attempt assumed `data Empty : Type0 =` and `absurd : (C:Type) -> Empty
-> C` were directly typeable as written; neither was (a zero-constructor
`data` didn't parse yet, and `absurd` turned out to be reserved sugar that
silently shadows a same-named user declaration rather than erroring on
redeclaration — the shadowing only surfaced by testing a *downstream call
site*, not by reading the declaration's own clean elaboration). Before
committing to a brief's exact spelling: write the smallest possible
standalone probe and run it through `ken run`/`ken check`, don't reason from
the brief's prose or the parser grammar in isolation.

This applies with extra force to a **kernel-direct piece** (a `.ken.md`
entry that needs a prelude addition the surface can't spell — an Ω-sorted
type parameter, a large-eliminating family): confirm the new mechanism
**admits** with a standalone bare-kernel harness (no elaborator, no
prelude) *before* touching landed code. It's cheap and freely rewritable,
and it catches de-Bruijn/motive-shape mistakes in a file with zero blast
radius, rather than inside `prelude.rs`.

## `class`/`instance` values are the synthesized `C_instance_T`, not `(C T)`

`(C T)` is the class **applied to its head** — the dictionary's *type*, not
a value; using it where a value is expected fails immediately (`§5` of the
surface reference strand has the checked discriminator). The value you want
is the synthesized global `C_instance_T`, projected exactly like any other
record (`(C_instance_T).field`). This is easy to burn a probe cycle or two
on the first time — it's not obvious from the `class`/`instance` declaration
syntax alone, only from how `elab_instance_decl` names what it registers.

## Prefer inlining a small helper over a cross-package dependency

For a pilot or example-shaped catalog entry, inline a small (~10-line)
landed helper (a `cong`, a `boolDichotomy`, a `sym`/`trans`) rather than
depending on the package that defines it. This isn't just a style
preference: cross-package `import` doesn't resolve to a real cross-file
load yet (`07-catalog-style-guide.md §13`), so a self-contained entry is
often the only one that can actually be verified end-to-end today.
`proof-techniques.ken.md`'s own `cong` and DS-1's `EmptyDec.ken.md` both
inline this way.

## In a `.ken.md` file, the prose is the comment layer

A `-- ` comment inside a checked or tangled `` ```ken `` fence is an
anti-pattern: it duplicates, in a worse place, what the surrounding
Markdown prose should say. Explain in prose, immediately before (or after)
the fence; keep the fence itself as clean code. Reserve an in-fence `--`
for the rare case where an annotation must point at one specific token and
genuinely can't live in prose — the default is prose. This holds uniformly
for every `.ken.md` — a package entry whose fences tangle to shipped source
and a teaching doc whose fences are only checked follow the identical rule;
there is no fence-role exception. `surface-reference.ken.md §8` has the full
fence-role table this rule sits next to.

## A required fact goes in the language, never in a comment

`docs/PRINCIPLES.md` #14: a comment is unchecked prose, so nothing that's
*required* — a contract, an invariant, a proposition, a trust boundary — may
live only in one. If you catch yourself writing a comment because "this has
to be said somewhere and nothing checks it otherwise," that is the signal to
reach for the language construct built to check it, not to write the
comment. Where each kind of required fact goes:

| Required fact | Language-proper home |
|---|---|
| A contract or precondition/postcondition | `requires`/`ensures`, or a refinement type on the constrained value |
| A proposition (a law, an invariant that must hold) | `law`/`prop`/`lemma`, discharged by `prove` — or, as the catalog most often states one, an ordinary `fn`/`const` whose result type *is* the property and whose body *is* the checked proof term |
| A trust boundary (an unproved postulate, an audited primitive) | `Axiom`, recorded as a `trusted_base()` delta in the entry's Trust & derivation section (`07-catalog-style-guide.md §7`) |

If a required fact genuinely has no home in one of these, that's not license
to fall back on a comment — it's a **Finding** (`§"Findings loop"` below):
the language is missing a construct, and the gap should be named and routed,
not papered over in prose no checker reads. Comments and surrounding prose
remain for genuine narrative only — proof strategy, naming rationale, why a
thing exists — never for the thing itself.

## Casing: PascalCase class-like, snake_case instance-like

Effective now for all NEW catalog authoring (operator-ruled, `ds-campaign-
judgment-log.md` §L6; full standard and examples at
`07-catalog-style-guide.md §9`): types, type classes, and data constructors
are PascalCase (`Either`, `Functor`, `Left`/`Some`/`Ok`); functions,
combinators, class methods, and record fields are snake_case
(`get_or_else`, `map_err`, `concat_map`). This reverses the camelCase you'll
see throughout already-landed catalog code — that's pre-L6 and **not** to be
renamed on sight; the bulk casing pass rides the future `.ken`→`.ken.md`
literate transformation. Write new identifiers to the standard; don't touch
old ones incidentally while you're in a file for another reason.

## Validating a `.ken.md` file: `ken run` vs. `ken check`

- **`ken run <file>`** — the file's last tangled declaration must be a
  nullary `proc main`; it elaborates, checks every fence, then actually
  drives the IO. Use it for a runnable file (a CLI fixture, a demo, the
  guide's own strands — each carries a `proc main` for exactly this).
- **`ken check <file>`** — elaborates and checks every fence, then stops
  before the IO-drive; exits 0 iff elaboration and every fence behaved. Use
  it for a **pure-library** entry (no `proc main` — the common shape for a
  catalog package). `ken run` on a pure-library file always fails with
  `"last definition is not an IO tree"` — that is not a fence defect, it is
  the wrong command for the file's shape. Cite `ken check`'s exit code as
  your fence-validation evidence for such an entry, not a `ken run` attempt.

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
