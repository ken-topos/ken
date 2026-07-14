# I-4 §D — boundary-header surface parser (Language)

**Owner:** Team Language · **Size:** M · **Gate:** unblocks I-4 §B (Runtime)
**Depends on:** I-4 §C (`origin/main @ c11ed3de`) — **merged**; its grammar is the
normative input.
**Blocks:** I-4 §B steps 2, 3, 4, 7 (the Runtime ring is compacted and idle,
waiting on this).

## Objective

Ken's `/spec` declares a boundary header that **no parser implements**. Make the
landed grammar real, end-to-end: source text → tokens → boundary AST → loader →
the two existing readers. Nothing downstream can read a declaration that nothing
parses; §B's mint, its static-containment gate, its I-3 lit path, and its typed
wrapper are **all** blocked on exactly this.

**Frame by objective + acceptance.** Every "current state" claim below is
**perishable — verify it against the landed code at pickup**, not against this
line (§2c). The grounding pass that produced this frame ran at
`origin/main @ c11ed3de`.

## Settled inputs — PINNED, do not re-derive or re-open

The grammar is **already normative and merged**. Build to it; do not redesign it,
and do not relitigate the keyword choice (`capabilities` was selected over
`caps`/`grants`/`requires` and justified in §31/§32 — that fork is closed).

From `spec/30-surface/32-grammar.md` (landed, lines 11–18):

```
unit                ::= boundary_hdr? import* decl*
boundary_hdr        ::= program_hdr | package_hdr
program_hdr         ::= "program" admits_clause? capabilities_clause?
package_hdr         ::= "package" admits_clause?
admits_clause       ::= "admits" ModPath ("," ModPath)*
capabilities_clause ::= "capabilities" capability_decl ("," capability_decl)*
capability_decl     ::= ConId ConId
```

- **The header is ANONYMOUS.** `program App` is **rejected** — there is no name
  field (N4 + §C; CV blocked a candidate on exactly this stale claim).
- **All four combinations are legal** on a program: `admits` alone,
  `capabilities` alone, both, neither (§32 line 94).
- **`capability_decl` = family then authority** — first `ConId` resolves as an
  effect family, second as an authority for it. v1 declaration is `FS AFull` (or
  another member of `Auth`). **Unknown family, invalid authority, or a second
  declaration for one family is a SURFACE ERROR** (§32 lines 95–98) — a named
  diagnostic, not a panic and not a silent accept.
- **`capabilities` is already reserved** in the lexical keyword set
  (`spec/30-surface/31-lexical.md:352`). `program` / `package` / `admits` are
  **not yet in the lexer's keyword table** — verify at pickup.
- **`admits` and `capabilities` are ORTHOGONAL** — separate namespaces, separate
  checks, separate purposes, and **separate readers**: the **elaborator** reads
  `admits` at its admission gate; the **runner** reads `capabilities` at mint
  (ADR-0014 as amended by §C). **Do NOT couple them**, and do not let one clause's
  absence affect the other's reading.
- **Declaration is the sole authority source.** No CLI grant, no launch grant, no
  ambient default (that is concern (2) — out of scope, §C).

## Deliverables — mandated outline

Each ends in a concrete implementable choice; no surveys.

1. **Lexer.** Add the missing boundary keywords (`program`, `package`, `admits`;
   `capabilities` per §31) to the token set and keyword table
   (`crates/ken-elaborator/src/lexer.rs` — the table around `"import" =>
   Token::KwImport`, verify at pickup). Reserved-word collision behavior follows
   the existing convention for `import`/`module`.
2. **Boundary AST.** A `boundary_hdr` node carrying: which header (`program` vs
   `package`), the `admits` module paths, and the declared capabilities as
   **(family, authority) pairs** — anonymous, no name field. Model the two
   clauses as **independent optionals**, so orthogonality is structural rather
   than a convention a later reader has to honor.
3. **Parser.** Parse `boundary_hdr` at unit head per the production above, ahead
   of `import*`. Enforce the §32 surface errors with **named diagnostic
   variants** — unknown family, invalid authority, duplicate family declaration,
   and a **named** header — never a bare error, never `is_err`.
4. **Loader / reader wiring.** Thread the parsed header through to the boundary
   the existing consumers already read from, so that:
   - the **elaborator's admission gate** consumes `admits` from the *declaration*
     (not a hand-built manifest), and
   - the **declared capabilities are reachable to the runner** as the authority
     source §B will mint `ProgramCaps a` from.
   **Do NOT mint, and do NOT implement §B.** §D's job ends at *the declaration is
   readable*; Runtime consumes it. If you find yourself editing
   `crates/ken-cli/src/main.rs:293-299` (the hard-coded `AUTH_PARTIAL`), **stop**
   — that line is §B step 2's, not yours.
5. **Conformance.** Turn the §C fixtures whose only unmet precondition is
   **"parser dependency"** from honest-RED to green — `conformance/surface/
   modules/seed-modules.md` (landed at `c11ed3de`). Fixtures **additionally**
   gated on I-4 §B **stay RED** and keep their honest precondition label; do not
   hand-feed them green.

## Acceptance criteria (testable)

- A `program` unit with **both** clauses parses; the boundary AST carries the
  `admits` paths and the `(FS, AFull)` pair. Assert the **AST shape**, not that
  parsing merely succeeded.
- **All four** combinations parse (admits-only / capabilities-only / both /
  neither) and each yields exactly the expected AST — the absent clause reads as
  *absent*, never as empty-coupled-to-the-other. **This is the orthogonality
  discriminator: it needs the non-degenerate pair, not one positive case.**
- `package` + `admits` parses; `package` + `capabilities` is a **surface error**
  (the production admits no `capabilities_clause` on `package_hdr`).
- **`program App` is REJECTED** with the named anonymous-header diagnostic.
- Each §32 surface error fires its **own named variant** — one case per error
  (unknown family / invalid authority / duplicate family) plus one where the
  problem hides behind an otherwise-valid clause. A shared error variant across
  three distinct causes fails this AC.
- The elaborator's admission gate resolves an `admits` path **taken from the
  parsed declaration** — demonstrated on a program whose admission outcome
  *changes* with the clause present vs absent (a non-vacuous pair).
- §C's parser-gated conformance fixtures pass; the §B-gated ones remain honestly
  RED with their labels intact.
- **Validate TARGETED only** — `scripts/ken-cargo -p ken-elaborator` (+ the one
  affected suite). **NEVER `--workspace`** (COORDINATION §12, operator hard
  rule); CI owns the locked/workspace gate.
- **Zero kernel delta; `trusted_base()` before == after.** This is pure surface
  syntax — if you find yourself adding a kernel rule or a trusted primitive,
  **STOP and flag the Steward.**

## Do-not-reopen guardrails

- **Do not redesign the grammar or the keyword.** It is merged and normative.
- **Do not implement `export`.** It is likewise spec-ahead-of-parser
  (`09acda41`), but it is a *decl*-level surface and its own future WP — out of
  scope here. Resist the adjacency.
- **Do not couple `admits` to `capabilities`** (separate readers — the whole
  point of the §C amendment).
- **No CLI/launch grant surface** (concern (2), out of scope).
- **Do not reopen the I-3 producers**, and do not touch §B's mint site.

## Sequencing

Language owns this ring alone. The **Architect is on-call** for design questions
(`any team → Architect`, COORDINATION §9); the **spec is the contract** — if the
grammar looks wrong or under-determined, that is a **Spec** question (route to
spec-leader), **not** a local fix. On merge, the Steward releases I-4 §B steps
2–7 back to Runtime.
