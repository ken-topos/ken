# kenfmt B3 — document algebra + layout printer design ruling

- **Status:** **Accepted** (Architect ruling, 2026-07-12). Grounded entirely in
  the already-normative layout clause `31-lexical.md §1d` (WP S),
  the landed B1 `FormattableSource` interface (`9df1f465`), and the landed B2
  token-kind canonicalization (`8945e887`). Needs no operator sign-off — it
  honors every locked constraint and only assigns the mechanism.
- **Deciders:** the Architect. Framed against a grounded B3 consult from the
  Steward, `docs/program/kenfmt-canonical-form-review.md`, and
  `docs/program/wp/kenfmt-work-program.md`.
- **Scope:** B3 (the document algebra + layout printers). Tool-internal —
  **zero `trusted_base()` delta**; the formatter is not kernel TCB.

## The four questions

B3 is the layout engine: it consumes B1's lossless token/trivia stream + B2's
canonical token spellings and produces the final canonical text (line-breaking,
indentation, the 88-column code fill). The consult asked for a ruling on
(1) the printer algebra, (2) each layout-canonicalization axis under the WP-S
determinism discipline, (3) comment/trivia composition with breaking, and
(4) whether B3 needs a spec sub-lane the way B2 had WP S.

The decisive grounding fact for the whole ruling: **WP S already made the full
structural layout normative in `§31 §1d`** — not just width and token
canonicalization, but declaration breaking, arrow chains, applications, lambdas,
`let`/`if`, `match`, declaration blocks, effects/contracts, comments, literate
roles, and open-spelling preservation. B3 is the *build to that existing spec*,
not a new design.

## Q1 — Algebra: Wadler/Leijen `Doc`, producing one canonical layout

**Ruling: a Wadler/Leijen document algebra** (`text` / soft `line` / hard
`line` / `group` / `nest` / `flatten`, with **display-width** measurement — not
byte width), **one printer per grammar production** over B1's typed AST
(`FormattableSource::typed_decls`). **Not** a Prettier-style fill model.

Two reasons, and they point the same way:

- **Ken's layout is prescriptive, not fill-driven.** Many breaks are
  *mandatory-structural*, independent of width: every `match` with ≥2 arms is
  multiline, a nested `match` is always compound, a block body always starts on
  the next line, a non-trivial sum prints one constructor per line. Prettier's
  fill model exists to *pack* a sequence maximally per line — exactly what Ken
  forbids (one arm per line, always). Wadler/Leijen expresses both régimes
  cleanly: a **hard `line`** for the mandatory-structural breaks (it propagates,
  so any `group` containing one can never flatten) and a soft `line` inside a
  `group` for the width-driven fit-or-break.
- **One canonical layout, not merely a valid one.** This is the `§1d` mandate
  ("one deterministic canonical form… a deterministic fit decision, not
  formatting latitude"). Wadler/Leijen's `group` is a **binary** decision — flat
  if the flattened group fits the remaining width, else broken — a pure function
  of `(subtree, current column, 88)`. That yields exactly one layout per input.
  A fill model has packing latitude (multiple valid fillings), which is the
  non-determinism `§1d` prohibits. So the structural fit *and* the determinism
  argument both select Wadler/Leijen.

Two constraints on the implementation, both already in `§1d`:

- **`nest` is relative to the enclosing syntax, never a coincidental visual
  column.** Continuation indent is one level (2 spaces) past the enclosing
  construct — never aligned to where a head token happened to end. **No global
  alignment combinators** (they make a local edit reflow unrelated siblings and
  can push short code over width).
- **The `group` fit test measures display width**, so canonical glyphs (`λ`,
  `→`, `Ω`) count at their terminal width, matching `§1d`'s "88 Unicode display
  columns."

## Q2 — The axes are already ruled; B3 implements, the golden pins both ways

Every axis the consult lists **already has a normative keep/normalize ruling in
`§31 §1d`.** B3 does not re-decide them; it builds them. Enumerated with their
settled deterministic disposition:

- **Blank-line runs** — 1 between top-level decls; 0 between siblings; around an
  attached comment **at most one preserved** (`0→0`, `1→1`, `2+→1`); the
  formatter otherwise owns vertical space.
- **Break points** — fit-or-break at 88 for soft groups; a **mandatory** break
  for `match` ≥2 arms / nested `match` / block body / non-trivial sum; a comment
  between tokens **forces** the enclosing group to break.
- **Alignment** — **never** align sibling arrows / colons / equals / bodies;
  single space, indentation alone expresses structure.
- **Separators** — semicolon **between** siblings, **omitted after the last**
  (declaration blocks); comma in record literals, patterns, named-arg forms.
- **Indent width** — exactly **two ASCII spaces** per level, enclosing-relative;
  tabs forbidden.
- **Parentheses** — precedence-required + the three mandatory-clarity cases
  kept; **"any other parenthesis is removed"** (the mandate form).

The WP-S determinism lesson — the redundant-parens "permission vs mandate" hole
— **is already closed** in `§1d` (the parenthesis rule reads "is removed", not
"may be removed"). So the discipline's remaining home is the **conformance
golden**: `conformance/surface/formatting/seed-canonical-format.md` already
carries the eight semantic-gate laws plus formatter-output cases marked
**red-until-B3**. B3 turns those green. The one companion deliverable is to
**confirm each axis above has a both-orientation output oracle** (an
input-in-alias-form → canonical case *and* a canonical → canonical idempotence
case). That is a **conformance extension owned by CV/the enclave**, authored
alongside B3 — *not* a spec change and *not* a B3-owned decision.

**Highest-risk axis — parenthesization.** It is the only place B3 changes
*token content* rather than only whitespace (it removes redundant parens and
re-adds precedence-required ones). A wrong paren choice reparses to a different
precedence grouping — a different AST — so its gate is **parse-preservation over
the whole catalog** (AST-equal modulo trivia/spans/sanctioned aliases), with
**elaboration-preservation** as the backstop that catches a fixity/resolution
interaction parser-equality alone could miss. Treat the paren printer as the
part of B3 that most needs adversarial catalog coverage.

## Q3 — Comments are hard-line-bearing `Doc` nodes

B1 already attaches every comment deterministically to the smallest enclosing
node as leading / trailing / interstitial (my B1 review confirmed
exactly-one-home totality). B3 lifts those attachments into the `Doc`:

- **Interstitial** (between tokens, mid-construct): emits a **hard `line`** into
  the enclosing group, which by propagation **forces that group to its broken
  form** (`§1d`: "forces the surrounding group to break"). It is emitted at the
  break at the enclosing node's indent and **never crosses a syntactic
  boundary** — B1 already keyed it to the node by byte offset, so B3 inherits
  the no-relocation guarantee.
- **Leading** (own line above a node): a hard-`line`-separated line at the
  node's indentation. A doc comment binds to the following declaration.
- **Trailing / end-of-line**: stays inline **iff** `code + two spaces + comment
  ≤ 88`; otherwise it moves to the line immediately above the node. That
  88-column threshold is itself a determinism axis — its golden oracle must pin
  **both** sides (fits → inline, overflows → moved-above).

Composition rule, stated once: **any group carrying an attached interstitial or
leading comment cannot be flattened.** That single invariant makes comment
placement a deterministic consequence of attachment + width, with no separate
heuristic.

## Q4 — B3 needs **no** spec sub-lane

**Ruling: no.** WP S was authored as the single spec clause for the *whole*
formatter, and `§31 §1d` already contains the complete normative layout (see the
decisive fact above). B1 (layout-neutral lossless layer), B2 (token-kind
canonicalization), and B3 (layout) are **all builds to that one clause** — B2
did not get its own spec sub-lane either; it built to `§1b/§1d`. B3 is the same.

What B3 *does* need is the **conformance deliverable** from Q2 — the per-axis
layout output oracles extending `seed-canonical-format.md`'s red-until-B3 cases
— CV/enclave-owned conformance work, not a normative spec round. Framing a spec
sub-lane here would duplicate `§1d` and invite drift between two homes for the
same rules (subsume-don't-proliferate).

## Sequencing and dependencies

- **Inputs, both landed.** B3 consumes B1's `FormattableSource` (`9df1f465`) for
  the typed AST + gapless trivia + comment attachments, and B2's canonical token
  spellings (`8945e887`). B3 is **CST-agnostic**: it reads the
  `FormattableSource` interface, so a future CST backing that interface needs no
  B3 rework (the door-open clause of the B1 ruling).
- **The preservation gate transitions.** B1's gate was **byte round-trip**
  (layout-neutral). B3 **changes layout**, so byte round-trip no longer holds;
  B3's gate is **parse-preservation** (AST-equal modulo trivia/spans/aliases)
  **+ idempotence** (`fmt∘fmt == fmt`, byte-exact) **+ the 88-column width
  property** (every >88 line classified indivisible/verbatim; no breakable
  syntax silently overflows). This runs continuously over the **whole catalog**,
  read-only — the one-time reformat is the capstone C, not B3.
- **P0 (field separator) — not a B3 blocker.** `§1d`, the grammar
  (`field (";" field)*`), and the parser are currently settled on
  **semicolon-between / no-trailing** for declaration blocks (comma stays for
  record literals, patterns, and named-argument forms). B3 builds to that. The
  parser carries an in-progress note about a comma-unification; **if** P0 ever
  flips the declaration-block separator, it is a **one-token printer constant**,
  not a redesign. Route P0's final status to the Steward (scope owner); do not
  gate B3 start on it.

## Revisit if

- A concrete **error-recovering CST** lands (the B1 revisit trigger): B3 is
  unaffected — it already targets the `FormattableSource` interface, and the
  `ken ignore`/`ken reject` unparseable-fragment exemption (`§1d`, token-aware
  canonicalization only, no structural layout) stays as specified.
- **`OQ-syntax`** settles type-application spelling (juxtaposition vs brackets):
  B3 gains one canonicalization it currently must *preserve as parsed* (`§1d`
  "preserved open spelling").
- **P0** flips the declaration-block separator: one printer constant changes.
